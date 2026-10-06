#!/usr/bin/env python3
"""Bundle the already-built release executable and playable demo."""
from pathlib import Path
import platform
import tarfile
import tomllib

ROOT = Path(__file__).resolve().parents[1]


def main():
    executable = "ge4g.exe" if platform.system() == "Windows" else "ge4g"
    binary = ROOT / "target/release" / executable
    if not binary.is_file():
        raise SystemExit("Build first: cargo build --locked --release -p ge4g-cli")
    version = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))["workspace"]["package"]["version"]
    output = ROOT / "dist" / f"ge4g-basement-{version}-{platform.system().lower()}-{platform.machine().lower()}.tar.gz"
    output.parent.mkdir(exist_ok=True)
    files = [(binary, executable)]
    files += [(ROOT / name, name) for name in ["README.md", "LICENSE", "AGENTS.md", "F(x).md"]]
    files += [(p, p.relative_to(ROOT).as_posix()) for p in sorted((ROOT / "docs").rglob("*.md"))]
    files += [(p, p.relative_to(ROOT).as_posix()) for p in sorted((ROOT / "examples/basement_demo").rglob("*")) if p.is_file() and p.name != "save.json"]
    for demo in ["flatland_pacman","flatland_signal_yard","flatland_harbor"]:
        files += [(p,p.relative_to(ROOT).as_posix()) for p in sorted((ROOT/"examples"/demo).rglob("*")) if p.is_file() and p.name!="save.json"]
    with tarfile.open(output, "w:gz") as archive:
        for source, name in files:
            archive.add(source, arcname=f"ge4g-basement/{name}", recursive=False)
    print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
