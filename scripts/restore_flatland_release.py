#!/usr/bin/env python3
"""Restore the fixed v0.2.0 public artifacts, verify, package and upload on Windows CI."""
import hashlib
import json
import os
import subprocess
import urllib.parse
import urllib.request
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REPO = "zizonhyeontae218/Game-Engine-For-GPT"
TAG = "v0.2.0"


def download(name, url, expected, folder):
    parsed = urllib.parse.urlparse(url)
    if parsed.scheme != "https" or not parsed.hostname or not parsed.hostname.endswith(".oaiusercontent.com"):
        raise ValueError("Expected a temporary artifact download URL")
    try:
        with urllib.request.urlopen(url, timeout=120) as response:
            data = response.read()
    except Exception:
        raise RuntimeError("Download failed: " + name) from None
    if len(data) != expected["bytes"] or hashlib.sha256(data).hexdigest() != expected["sha256"]:
        raise ValueError("Original checksum mismatch: " + name)
    (folder / name).write_bytes(data)


def main():
    manifest = json.loads((ROOT / "docs/releases/flatland-final-drive.json").read_text())
    urls = json.loads(os.environ["RELEASE_DOWNLOADS"])
    if set(urls) != {item["name"] for item in manifest}:
        raise ValueError("Exactly the 14 recorded public artifacts are required")
    folder = ROOT / "dist/release-recovery"
    folder.mkdir(parents=True, exist_ok=True)
    for item in manifest:
        download(item["name"], urls[item["name"]], item, folder)
        print("Verified:", item["name"])

    sdk = folder / "c-sdk"
    sdk.mkdir(exist_ok=True)
    for name in ("main.c", "ge4g_client.def"):
        (sdk / name).write_bytes((ROOT / "examples/c_abi" / name).read_bytes())
    (sdk / "include").mkdir(exist_ok=True)
    (sdk / "include/ge4g_client.h").write_bytes((ROOT / "crates/ge4g-client/include/ge4g_client.h").read_bytes())
    with zipfile.ZipFile(folder / "GE4G-Harbor-0.2.0-Windows.zip") as archive:
        (sdk / "ge4g_client.dll").write_bytes(archive.read("ge4g-harbor-windows/ge4g_client.dll"))
    # Only checksum-verified sample files are extracted; reject paths leaving the SDK.
    with zipfile.ZipFile(folder / "FlatLand-Harbor-0.2.0-Source.zip") as archive:
        for entry in archive.infolist():
            target = (sdk / "game" / entry.filename).resolve()
            if not target.is_relative_to((sdk / "game").resolve()):
                raise ValueError("Unsafe sample archive path")
            if not entry.is_dir():
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(archive.read(entry))
    for command in (["lib", "/nologo", "/def:ge4g_client.def", "/machine:x64", "/out:ge4g_client.lib"],
                    ["cl", "/nologo", "/W4", "/Iinclude", "main.c", "ge4g_client.lib", "/Fe:ge4g-c.exe"]):
        subprocess.run(command, cwd=sdk, check=True)
    requests = '{"op":"validate","project":"game"}\n{"op":"open","project":"game"}\n'
    result = subprocess.run([str(sdk / "ge4g-c.exe")], cwd=sdk, input=requests,
                            text=True, encoding="utf-8", capture_output=True, check=True)
    replies = [json.loads(line) for line in result.stdout.splitlines()]
    if len(replies) != 2 or not all(reply.get("ok") for reply in replies) or not replies[1].get("session"):
        raise RuntimeError("C example failed to validate/open the actual 0.2 DLL")
    readme = (ROOT / "examples/c_abi/README.md").read_text(encoding="utf-8")
    readme = readme.replace("../../docs/releases/README.md", f"https://github.com/{REPO}/blob/main/docs/releases/README.md")
    (sdk / "README.md").write_text(readme, encoding="utf-8")
    (sdk / "LICENSE").write_bytes((ROOT / "LICENSE").read_bytes())
    (sdk / "PROVENANCE.json").write_text(json.dumps({"engine_tag": TAG,
        "engine_commit": "ed3301c7af12ac36dd0b5074d79693d203aeeb28",
        "dll_source": "GE4G-Harbor-0.2.0-Windows.zip", "abi": 1,
        "example_commit": subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip(),
        "c_windows_validate_open": "PASS"}, indent=2))
    with zipfile.ZipFile(folder / "GE4G-0.2.0-C-SDK-Windows.zip", "w", zipfile.ZIP_DEFLATED) as archive:
        for path in sorted(sdk.rglob("*")):
            if path.is_file() and path.suffix != ".obj":
                archive.write(path, path.relative_to(sdk))

    kit = folder / "GE4G-0.2.0-AgentKit.zip"
    subprocess.run(["git", "archive", "--format=zip", "--output=" + str(kit), TAG], cwd=ROOT, check=True)
    with zipfile.ZipFile(kit, "a", zipfile.ZIP_DEFLATED) as archive:
        archive.writestr("START-HERE.md", "# GE4G 0.2.0 AgentKit\n\n"
            "Stable engine source, public docs and editable examples at v0.2.0.\n"
            "Read AGENTS.md and docs/FLATLAND_QUICKSTART.md; start with examples/flatland_harbor.\n"
            "Prompt: Read the authoring contract, modify the sample game without changing the engine, "
            "then validate and replay-test it using ge4g-cli.\n"
            "Pentomino 0.3 development roles belong to main; clone main and use QUICKSTART.ko.md.\n"
            "No private signing key or Drive access is required.\n")
    files = sorted(path for path in folder.iterdir() if path.is_file())
    # Preserve the original SHA256SUMS.txt; include the two new packages in a separate manifest.
    sums = folder / "GE4G-0.2.0-SHA256SUMS.txt"
    sums.write_text("".join(hashlib.sha256(path.read_bytes()).hexdigest() + "  " + path.name + "\n"
                            for path in files), encoding="utf-8")
    files.append(sums)
    subprocess.run(["gh", "release", "upload", TAG, *map(str, files), "--clobber", "--repo", REPO], check=True)
    print("Uploaded verified originals, C SDK, AgentKit and extended checksums")


if __name__ == "__main__":
    main()
