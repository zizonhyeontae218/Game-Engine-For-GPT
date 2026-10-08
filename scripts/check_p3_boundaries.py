#!/usr/bin/env python3
"""Verify the Camera consumer dependency direction and P2/legacy preservation."""
import hashlib
import json
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def command(*args):
    return subprocess.check_output(args, cwd=ROOT, text=True)


def main():
    baseline = json.loads((ROOT / "docs/pentomino/evidence/p3/preservation-baseline.json").read_text())
    for entry in baseline["preserved_files"]:
        assert hashlib.sha256((ROOT / entry["path"]).read_bytes()).hexdigest() == entry["sha256"], entry["path"]
    preserved = ["crates/ge4g-pentomino/src", "crates/ge4g-pentomino/tests",
                 "crates/ge4g-pentomino-legacy/src", "crates/ge4g-pentomino-legacy/tests",
                 "crates/ge4g-core/src", "crates/ge4g-project/src", "crates/ge4g-runtime/src",
                 "crates/ge4g-render2d/src", "crates/ge4g-client/src", "client", "examples"]
    assert not command("git", "diff", baseline["source_base"], "--", *preserved), "preserved production/test source changed"
    metadata = json.loads(command("cargo", "metadata", "--locked", "--format-version=1"))
    packages = {p["id"]: p for p in metadata["packages"]}
    graph = {n["id"]: n["dependencies"] for n in metadata["resolve"]["nodes"]}

    def closure(name):
        start = next(p["id"] for p in packages.values() if p["name"] == name)
        seen = set()
        pending = list(graph[start])
        while pending:
            item = pending.pop()
            if item not in seen:
                seen.add(item)
                pending.extend(graph[item])
        return {packages[p]["name"] for p in seen}

    assert not {p for p in closure("ge4g-pentomino") if p.startswith("ge4g-")}
    native_engine_deps = {p for p in closure("ge4g-pentomino-view") if p.startswith("ge4g-")}
    assert native_engine_deps == {"ge4g-pentomino"}, native_engine_deps
    compat = closure("ge4g-pentomino-view-legacy")
    assert {"ge4g-pentomino-view", "ge4g-pentomino", "ge4g-runtime", "ge4g-render2d"} <= compat
    stripped = re.compile(r'//[^\n]*|/\*[\s\S]*?\*/|"(?:\\.|[^"\\])*"')
    # This supplements dependency/source review; it does not prove a native-code sandbox.
    for path in (ROOT / "crates/ge4g-pentomino-view/src").rglob("*.rs"):
        tokens = stripped.sub(" ", path.read_text())
        assert not re.search(r"\b(CoreHost|CoreTransaction|World)\b", tokens), str(path)
    print(json.dumps({"ok": True, "baseline": baseline["source_base"],
                      "preserved_files": len(baseline["preserved_files"]),
                      "native_engine_dependencies": sorted(native_engine_deps),
                      "core_changes": 0, "external_validation": "PENDING"}))


if __name__ == "__main__":
    main()
