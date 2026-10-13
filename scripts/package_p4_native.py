#!/usr/bin/env python3
"""Package a Linux-hosted source-free P4 C ABI sample, not a platform release."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    out = args.out.resolve()
    if out.exists() or out == ROOT or ROOT in out.parents or out in ROOT.parents:
        parser.error("--out must be a new directory outside the repository")
    if not sys.platform.startswith("linux"):
        parser.error("this development SDK packaging path is only verified on Linux")
    inputs = ["Cargo.toml", "Cargo.lock", "crates/ge4g-pentomino-gameplay", "crates/ge4g-pentomino-native",
              "samples/pentomino_p4", "docs/pentomino/P4_CONTRACT.md", "scripts/package_p4_native.py"]
    dirty = subprocess.check_output(["git", "status", "--porcelain", "--", *inputs], cwd=ROOT, text=True)
    if dirty.strip():
        parser.error("commit source/public inputs before packaging")
    subprocess.run(["cargo", "build", "--locked", "-p", "ge4g-pentomino-native"], cwd=ROOT, check=True)
    out.mkdir(parents=True)
    files = {
        "libge4g_pentomino_native.so": ROOT / "target/debug/libge4g_pentomino_native.so",
        "ge4g_p4_v1.h": ROOT / "crates/ge4g-pentomino-native/include/ge4g_p4_v1.h",
        "consumer.py": ROOT / "samples/pentomino_p4/consumer.py",
        "smoke.c": ROOT / "samples/pentomino_p4/smoke.c",
        "game.json": ROOT / "samples/pentomino_p4/game.json",
        "P4_CONTRACT.md": ROOT / "docs/pentomino/P4_CONTRACT.md",
        "README.md": ROOT / "samples/pentomino_p4/README.md",
        "LICENSE": ROOT / "LICENSE",
    }
    for name, source in files.items():
        shutil.copyfile(source, out / name)
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--locked", "--format-version=1"], cwd=ROOT, text=True))
    dependencies = []
    for package in metadata["packages"]:
        if package["source"] is None:
            continue
        name = f"{package['name']}-{package['version']}"
        dependencies.append({"name": package["name"], "version": package["version"], "license": package["license"]})
        for path in Path(package["manifest_path"]).parent.iterdir():
            if path.is_file() and path.name.upper().startswith(("LICENSE", "LICENCE", "COPYING", "COPYRIGHT")):
                destination = out / "licenses" / name / path.name
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(path, destination)
    (out / "DEPENDENCY_LICENSES.json").write_text(json.dumps(dependencies, indent=2) + "\n")
    manifest = {
        "version": 1, "contract": "ge4g_p4_v1", "profile": "debug", "platform": "Linux native development infrastructure",
        "commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "rustc": subprocess.check_output(["rustc", "-vV"], cwd=ROOT, text=True),
        "external_certification": "UNVERIFIED", "files": [],
    }
    for path in sorted(out.rglob("*")):
        if path.is_file():
            data = path.read_bytes()
            manifest["files"].append({"path": str(path.relative_to(out)), "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()})
    (out / "MANIFEST.json").write_text(json.dumps(manifest, indent=2) + "\n")
    archive = out.with_suffix(".zip")
    with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED) as bundle:
        for path in sorted(out.rglob("*")):
            if path.is_file():
                bundle.write(path, arcname=str(path.relative_to(out)))
    checksum = hashlib.sha256(archive.read_bytes()).hexdigest()
    archive.with_suffix(".zip.sha256").write_text(f"{checksum}  {archive.name}\n")
    print(json.dumps({"ok": True, "path": str(archive), "sha256": checksum, "commit": manifest["commit"],
                      "external_certification": "UNVERIFIED"}))


if __name__ == "__main__":
    main()
