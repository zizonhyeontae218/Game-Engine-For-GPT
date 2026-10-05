#!/usr/bin/env python3
"""Create a validated, data-only, portable Basement .ge4g package."""
from pathlib import Path
import argparse
import hashlib
import json
import re
import subprocess
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def pack(project, game_id, version, output, validator, layouts=None, mapping=None):
    if not re.fullmatch(r"[a-z0-9][a-z0-9._-]{0,95}", game_id):
        raise ValueError("game_id must be a stable 1..96 character lowercase slug")
    if not re.fullmatch(r"[A-Za-z0-9._+-]{1,64}", version):
        raise ValueError("version must be a 1..64 character version label")
    project = project.resolve()
    result = subprocess.run([str(validator.resolve()), "validate", str(project), "--json"], capture_output=True, text=True, check=False)
    if result.returncode:
        raise ValueError(f"Basement project validation failed: {result.stdout} {result.stderr}")
    manifest = tomllib.loads((project / "ge4g.toml").read_text())
    entries = {}
    for source in sorted(project.rglob("*"), key=lambda p: p.relative_to(project).as_posix()):
        if source.is_symlink():
            raise ValueError(f"game package cannot contain symlinks: {source}")
        if not source.is_file():
            continue
        relative = source.relative_to(project)
        if source.name == "save.json" or any(p in {".git", "target", "artifacts", "dist", "__pycache__"} for p in relative.parts):
            continue
        name = "game/" + relative.as_posix()
        if "\\" in name or source.stat().st_size > 16 * 1024 * 1024:
            raise ValueError(f"invalid path or file exceeds 16 MiB: {relative}")
        entries[name] = source.read_bytes()
    authored_layouts = project / "controls/layouts.json"
    authored_bindings = project / "controls/bindings.json"
    if authored_layouts.exists() != authored_bindings.exists():
        raise ValueError("game controls require both layouts.json and bindings.json")
    entries["controls/layouts.json"] = (layouts or (authored_layouts if authored_layouts.exists() else ROOT / "client/assets/default_layouts.json")).read_bytes()
    bindings = json.loads((mapping or (authored_bindings if authored_bindings.exists() else ROOT / "client/assets/default_bindings.json")).read_text())
    bindings["game_id"] = game_id
    entries["controls/bindings.json"] = (json.dumps(bindings, ensure_ascii=False, indent=2) + "\n").encode()
    if sum(map(len, entries.values())) > 256 * 1024 * 1024 or len(entries) > 4095:
        raise ValueError("game package exceeds 256 MiB or 4096 files")
    if len({name.lower() for name in entries}) != len(entries):
        raise ValueError("game package paths collide on case-insensitive platforms")
    metadata = {"schema_version": manifest["schema_version"], "game_id": game_id, "name": manifest["name"], "version": version, "engine_abi": 1, "project": "game/ge4g.toml", "layouts": "controls/layouts.json", "bindings": "controls/bindings.json", "files": {name: hashlib.sha256(data).hexdigest() for name, data in entries.items()}}
    if manifest["schema_version"] == 2:
        metadata["game_schema"] = 2
    entries["bundle.json"] = (json.dumps(metadata, ensure_ascii=False, indent=2) + "\n").encode()
    layout = json.loads(entries["controls/layouts.json"])
    if layout.get("schema_version") not in (1, 2) or bindings.get("schema_version") != 1:
        raise ValueError("layouts require schema_version 1 or 2; bindings require 1")
    ids = [profile["id"] for profile in layout["profiles"]]
    if len(ids) != len(set(ids)) or not 1 <= len(ids) <= 16 or set(ids) != set(bindings["profiles"]) or layout["active_profile"] not in ids:
        raise ValueError("layout and binding profile IDs must match")
    for profile in layout["profiles"]:
        buttons = [button["id"] for button in profile["buttons"]]
        if not (1 if layout["schema_version"] == 1 else 0) <= len(buttons) <= 16 or len(set(buttons)) != len(buttons) or set(buttons) != set(bindings["profiles"][profile["id"]]["buttons"]):
            raise ValueError("layout and binding button IDs must match")
    output.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for name, data in sorted(entries.items()):
            info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, data)
    if output.stat().st_size > 64 * 1024 * 1024:
        raise ValueError("game package exceeds 64 MiB compressed")
    return metadata


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("project", type=Path)
    parser.add_argument("--game-id", required=True)
    parser.add_argument("--version", default="0.1.0")
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--validator", type=Path, default=ROOT / "target/debug/ge4g")
    parser.add_argument("--layouts", type=Path)
    parser.add_argument("--bindings", type=Path)
    args = parser.parse_args()
    try:
        metadata = pack(args.project, args.game_id, args.version, args.out, args.validator, args.layouts, args.bindings)
    except (OSError, ValueError, KeyError, TypeError) as error:
        parser.exit(1, f"pack game: {error}\n")
    print(json.dumps({"ok": True, "game_id": metadata["game_id"], "files": len(metadata["files"]), "path": str(args.out)}))


if __name__ == "__main__":
    main()
