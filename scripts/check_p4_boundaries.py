#!/usr/bin/env python3
"""Check P4 dependency direction and additive native ABI; not a sandbox proof."""
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--locked", "--format-version=1"], cwd=ROOT, text=True))
    packages = {p["id"]: p for p in metadata["packages"]}
    graph = {n["id"]: n["dependencies"] for n in metadata["resolve"]["nodes"]}

    def engine_dependencies(name):
        start = next(p["id"] for p in packages.values() if p["name"] == name)
        pending = list(graph[start])
        seen = set()
        while pending:
            item = pending.pop()
            if item not in seen:
                seen.add(item)
                pending.extend(graph[item])
        return {packages[p]["name"] for p in seen if packages[p]["name"].startswith("ge4g-")}

    assert engine_dependencies("ge4g-pentomino") == set()
    assert engine_dependencies("ge4g-pentomino-gameplay") == {"ge4g-pentomino"}
    assert engine_dependencies("ge4g-pentomino-view") == {"ge4g-pentomino"}
    native = engine_dependencies("ge4g-pentomino-native")
    assert {"ge4g-pentomino", "ge4g-pentomino-gameplay", "ge4g-pentomino-view", "ge4g-render2d"} <= native
    strip = re.compile(r'//[^\n]*|/\*[\s\S]*?\*/|"(?:\\.|[^"\\])*"')
    for path in (ROOT / "crates/ge4g-pentomino-native/src").rglob("*.rs"):
        source = strip.sub(" ", path.read_text())
        assert not re.search(r"\bWorld\b|\bge4g_runtime\b", source), path
        exports = re.findall(r'extern\s+"?\s*C\s*"?\s+fn\s+(\w+)', path.read_text())
        assert all(name.startswith(("ge4g_p4_v1_", "ge4g_p5_v1_")) for name in exports), exports
    print(json.dumps({"ok": True, "gameplay_engine_dependencies": ["ge4g-pentomino"],
                      "native_engine_dependencies": sorted(native), "abi": "ge4g_p4_v1",
                      "external_certification": "UNVERIFIED"}))


if __name__ == "__main__":
    main()
