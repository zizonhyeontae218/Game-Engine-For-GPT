#!/usr/bin/env python3
"""Public C-ABI consumer. Requires only the shared library, game.json and this file.

This is reproducible internal consumer evidence, not an independent external
tester certification. No engine source or Rust compiler is accessed at runtime.
"""
import argparse
import copy
import ctypes
import hashlib
import json
from pathlib import Path
import threading


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("library", type=Path)
    parser.add_argument("--config", type=Path, default=Path(__file__).with_name("game.json"))
    parser.add_argument("--out", type=Path, default=Path("consumer-output"))
    args = parser.parse_args()
    manifest_path = args.library.resolve().parent / "MANIFEST.json"
    if manifest_path.is_file():
        manifest = json.loads(manifest_path.read_text())
        for entry in manifest["files"]:
            data = (manifest_path.parent / entry["path"]).read_bytes()
            assert len(data) == entry["bytes"] and hashlib.sha256(data).hexdigest() == entry["sha256"], entry["path"]
    lib = ctypes.CDLL(str(args.library.resolve()))
    lib.ge4g_p4_v1_abi_version.restype = ctypes.c_uint32
    assert lib.ge4g_p4_v1_abi_version() == 1
    lib.ge4g_p4_v1_request.argtypes = [ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t]
    lib.ge4g_p4_v1_request.restype = ctypes.c_void_p
    lib.ge4g_p4_v1_string_free.argtypes = [ctypes.c_void_p]
    lib.ge4g_p4_v1_string_free.restype = None

    def raw(payload):
        buffer = (ctypes.c_uint8 * len(payload)).from_buffer_copy(payload)
        result = lib.ge4g_p4_v1_request(buffer, len(payload))
        assert result, "native returned null response"
        try:
            return json.loads(ctypes.string_at(result))
        finally:
            lib.ge4g_p4_v1_string_free(result)

    def call(op, expected_error=None, **fields):
        response = raw(json.dumps({"version": 1, "command": {"op": op, **fields}}, separators=(",", ":")).encode())
        assert response["version"] == 1
        if expected_error:
            assert not response["ok"] and response["error"]["code"] == expected_error, response
            return response["error"]
        assert response["ok"], response
        assert response["error"] is None
        return response["result"]

    config = json.loads(args.config.read_text())
    linked = call("create", config=config, seed=42, mode="linked")["session"]
    headless = call("create", config=config, seed=42, mode="headless")["session"]
    call("frame", session=headless, expected_error="unsupported")
    initial = call("frame", session=linked)
    assert len(initial["rgba"]) == initial["width"] * initial["height"] * 4
    assert hashlib.sha256(bytes(initial["rgba"])).hexdigest() == initial["sha256"]
    start = call("observe", session=linked)
    assert len(start["records"]) == 3 and len(start["camera_frames"][0]["items"]) == 2

    def step(handle, number, controls=None):
        return call("step", session=handle, input={"version": 1, "target_step": number,
                    "controls": [config["attack_control"]] if controls is None else controls})

    for number in (1, 2):
        linked_result = step(linked, number)
        headless_result = step(headless, number)
        assert linked_result == headless_result
        assert call("save", session=linked)["core_save2"] == call("save", session=headless)["core_save2"]
    checkpoint = call("save", session=linked)
    checkpoint_frame = call("frame", session=linked)
    for bad in (["missing"], [config["attack_control"], config["attack_control"]], ["p4.state.hp0"]):
        call("step", session=linked, input={"version": 1, "target_step": 3, "controls": bad}, expected_error="invalid_input")
        assert call("save", session=linked) == checkpoint
        assert call("frame", session=linked) == checkpoint_frame
    for field in ("core_save2", "presentation_save1"):
        bad = copy.deepcopy(checkpoint)
        bad[field] = "{}"
        response = raw(json.dumps({"version": 1, "command": {"op": "restore", "session": linked, "save": bad}}).encode())
        assert not response["ok"]
        assert call("save", session=linked) == checkpoint
        assert call("frame", session=linked) == checkpoint_frame

    step(linked, 3)
    continued = call("save", session=linked)
    call("restore", session=linked, save=checkpoint)
    step(linked, 3)
    assert call("save", session=linked) == continued
    authority = call("observe", session=linked)
    call("gameplay", session=linked, enabled=False)
    detached = call("observe", session=linked)
    for field in ("battle", "records", "scenes", "entities"):
        assert detached[field] == authority[field]
    assert not detached["gameplay_attached"]
    discovery = call("describe", session=linked)
    assert [p["descriptor"]["id"] for p in discovery["core"]["plugins"]] == ["p4.state"]
    call("step", session=linked, input={"version": 1, "target_step": 4, "controls": [config["attack_control"]]}, expected_error="gameplay_detached")
    step(linked, 4, [])
    assert call("observe", session=linked)["battle"] == detached["battle"]
    call("gameplay", session=linked, enabled=True)
    core = call("save", session=linked)["core_save2"]
    call("camera", session=linked, request={"version": 1, "zoom": 1.25, "tilt": 1.0,
         "shear": 0.0, "origin": [0.0, 0.0], "transition_ticks": 0})
    call("view", session=linked, family="TopDown")
    assert call("save", session=linked)["core_save2"] == core
    final = step(linked, 5)
    assert final["battle"]["hp"] == [6, 3]
    final = step(linked, 6)
    assert final["battle"]["hp"] == [6, 0] and final["battle"]["winner"] == 0
    call("step", session=linked, input={"version": 1, "target_step": 7, "controls": [config["attack_control"]]}, expected_error="invalid_input")
    frame = call("frame", session=linked)
    assert frame["sha256"] != initial["sha256"]
    assert hashlib.sha256(bytes(frame["rgba"])).hexdigest() == frame["sha256"]
    assert frame["core_tick"] == 12 and len(frame["rgba"]) == 160 * 96 * 4
    foreign = []
    thread = threading.Thread(target=lambda: foreign.append(call("observe", session=linked, expected_error="invalid_session")))
    thread.start()
    thread.join()
    assert foreign
    for handle in (linked, headless):
        call("close", session=handle)
        call("observe", session=handle, expected_error="invalid_session")
        call("close", session=handle, expected_error="invalid_session")
    assert not raw(b"invalid JSON")["ok"]
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "frame.rgba").write_bytes(bytes(frame["rgba"]))
    result = {"ok": True, "abi": "ge4g_p4_v1", "library_sha256": hashlib.sha256(args.library.read_bytes()).hexdigest(),
              "core_tick": frame["core_tick"], "battle": final["battle"], "frame_sha256": frame["sha256"],
              "renderer": "ge4g-render2d via actual P3 CameraFrame", "external_certification": "UNVERIFIED"}
    (args.out / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result))


if __name__ == "__main__":
    main()
