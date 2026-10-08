#!/usr/bin/env python3
"""Build a source-free, compiler-pinned P2 public SDK ZIP outside the repository."""
import argparse
import hashlib
import json
import shutil
import subprocess
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

RUNNER = '''#!/usr/bin/env python3
"""Compile a public consumer against the supplied SDK; this is not an external-gate report."""
import argparse, hashlib, json, subprocess
from pathlib import Path
root = Path(__file__).resolve().parent
manifest = json.loads((root / "MANIFEST.json").read_text())
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--check", action="store_true")
parser.add_argument("--source", type=Path)
parser.add_argument("--test", action="store_true")
args = parser.parse_args()
for entry in manifest["files"]:
    path = root / entry["path"]
    assert path.stat().st_size == entry["bytes"], entry["path"]
    assert hashlib.sha256(path.read_bytes()).hexdigest() == entry["sha256"], entry["path"]
try:
    compiler = subprocess.check_output(["rustc", "-vV"], text=True)
except (OSError, subprocess.CalledProcessError):
    raise SystemExit("BLOCKED: rustc is unavailable or cannot start; install the compiler listed in MANIFEST.json") from None
if compiler != manifest["rustc"]:
    raise SystemExit("BLOCKED: compiler differs from compiler-pinned MANIFEST.json")
if args.check:
    print(json.dumps({"checksums": "PASS", "compiler": "PASS", "commit": manifest["commit"], "external_gate": "UNVERIFIED"}))
else:
    source = args.source.resolve() if args.source else root / "examples/p2_public.rs"
    output = root / "consumer-output"
    command = ["rustc", "--edition=2024", str(source), "--extern", "ge4g_pentomino=" + str(root / manifest["core_rlib"]),
               "-L", "dependency=" + str(root / "lib"), "-o", str(output)]
    if args.test:
        command.append("--test")
    print("COMMAND:", " ".join(command), flush=True)
    subprocess.run(command, check=True)
    subprocess.run([str(output)], check=True)
'''


def command(*args):
    return subprocess.check_output(args, cwd=ROOT, text=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    out = args.out.resolve()
    if out == ROOT or ROOT in out.parents or out in ROOT.parents or out.exists():
        parser.error("--out must be a new directory outside the repository")
    production = ["Cargo.toml", "Cargo.lock", "crates/ge4g-pentomino/src",
                  "crates/ge4g-pentomino/examples", "crates/ge4g-pentomino/Cargo.toml"]
    if command("git", "status", "--porcelain", "--", *production).strip():
        parser.error("commit Core production inputs before packaging")
    commit = command("git", "rev-parse", "HEAD").strip()
    build = subprocess.run(["cargo", "build", "--locked", "--release", "-p", "ge4g-pentomino",
                            "--example", "p2_public", "--message-format=json"],
                           cwd=ROOT, capture_output=True, text=True, check=True)
    artifacts = [json.loads(line) for line in build.stdout.splitlines() if line.startswith("{")]
    out.mkdir(parents=True)
    (out / "lib").mkdir()
    (out / "examples").mkdir()
    core_rlib = None
    for artifact in artifacts:
        if artifact.get("reason") != "compiler-artifact":
            continue
        for filename in artifact["filenames"]:
            path = Path(filename)
            if path.suffix not in {".rlib", ".so"}:
                continue
            if path.is_symlink() or not path.is_file():
                raise ValueError("unsafe compiler artifact")
            target = out / "lib" / path.name
            shutil.copyfile(path, target)
            if artifact["target"]["name"] == "ge4g_pentomino" and path.suffix == ".rlib":
                core_rlib = str(target.relative_to(out))
    if core_rlib is None:
        raise ValueError("missing public Core library artifact")
    for path in sorted((ROOT / "docs/public/pentomino-p2").glob("*.md")):
        shutil.copyfile(path, out / path.name)
    shutil.copyfile(ROOT / "docs/pentomino/P2_CONTRACT.md", out / "CONTRACT.md")
    shutil.copyfile(ROOT / "LICENSE", out / "LICENSE")
    shutil.copyfile(ROOT / "crates/ge4g-pentomino/examples/p2_public.rs", out / "examples/p2_public.rs")
    (out / "run_public.py").write_text(RUNNER)
    files = [{"path": str(path.relative_to(out)), "bytes": path.stat().st_size,
              "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
             for path in sorted(out.rglob("*")) if path.is_file()]
    manifest = {"artifact": "GE4G0.3.0-alpha.1-P2-public-candidate", "commit": commit,
                "contract_version": "2.0.0", "save_version": 2,
                "external_gate": "UNVERIFIED", "source_free": True,
                "rustc": command("rustc", "-vV"), "core_rlib": core_rlib, "files": files}
    (out / "MANIFEST.json").write_text(json.dumps(manifest, indent=2) + "\n")
    (out / "SHA256SUMS").write_text("".join(f'{entry["sha256"]}  {entry["path"]}\n' for entry in files)
                                   + hashlib.sha256((out / "MANIFEST.json").read_bytes()).hexdigest()
                                   + "  MANIFEST.json\n")
    archive = out.with_suffix(".zip")
    if archive.exists():
        raise ValueError("archive destination exists")
    with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED) as bundle:
        for path in sorted(out.rglob("*")):
            if path.is_file():
                bundle.write(path, str(path.relative_to(out)))
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    archive.with_suffix(".zip.sha256").write_text(f"{digest}  {archive.name}\n")
    print(json.dumps({"archive": str(archive), "sha256": digest, "bytes": archive.stat().st_size,
                      "commit": commit, "files": len(files), "external_gate": "UNVERIFIED"}))


if __name__ == "__main__":
    main()
