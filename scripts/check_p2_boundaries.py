#!/usr/bin/env python3
"""Check the P2 dependency boundary and untouched P1/legacy compatibility baseline."""
import hashlib
import json
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BASE = "2d1ffd060389109f341fb5854ce07890f445f3f9"
P1_TEST_SHA = "ce87260ccd1c7c5fe98daac774d70db90f7c6aed981c6c1767e953a288e8461a"


def command(*args):
    return subprocess.check_output(args, cwd=ROOT, text=True)


def main():
    metadata = json.loads(command("cargo", "metadata", "--locked", "--format-version=1"))
    packages = {p["id"]: p for p in metadata["packages"]}
    graph = {p["id"]: p["dependencies"] for p in metadata["resolve"]["nodes"]}
    core = next(p for p in packages.values() if p["name"] == "ge4g-pentomino")
    visited = set()
    pending = list(graph[core["id"]])
    while pending:
        dep = pending.pop()
        if dep in visited:
            continue
        visited.add(dep)
        assert not packages[dep]["name"].startswith("ge4g-"), packages[dep]["name"]
        pending.extend(graph[dep])
    adapter = next(p for p in packages.values() if p["name"] == "ge4g-pentomino-legacy")
    direct = {packages[d]["name"] for d in graph[adapter["id"]]}
    assert {"ge4g-pentomino", "ge4g-core", "ge4g-project", "ge4g-runtime"} <= direct
    assert adapter["id"] not in visited
    forbidden = re.compile(
        r"\b(camera|viewport|sprite|layer|coordinate_system|flatland|collision|combat|"
        r"platformer|top_down|vertical_scroll|forge)\b", re.I
    )
    # Exclude prose/string values; check Rust identifier tokens, not plugin-authored data.
    comments_strings = re.compile(r'//[^\n]*|/\*[\s\S]*?\*/|"(?:\\.|[^"\\])*"')
    for path in sorted((ROOT / "crates/ge4g-pentomino/src/p2").glob("*.rs")):
        tokens = comments_strings.sub(" ", path.read_text())
        assert not forbidden.search(tokens), str(path)
    p1 = ROOT / "crates/ge4g-pentomino/tests/lifecycle_contract.rs"
    assert hashlib.sha256(p1.read_bytes()).hexdigest() == P1_TEST_SHA, "P1 tests changed"
    root_path = "crates/ge4g-pentomino/src/lib.rs"
    original = command("git", "show", f"{BASE}:{root_path}")
    current = (ROOT / root_path).read_text()
    assert current.replace("pub mod p2;\n\n", "", 1) == original, "P1 implementation changed"
    preserved = ["crates/ge4g-core/src", "crates/ge4g-project/src", "crates/ge4g-runtime/src",
                 "crates/ge4g-render2d/src", "crates/ge4g-native/src", "client", "examples"]
    assert not command("git", "diff", BASE, "--", *preserved), "legacy baseline modified"
    print(json.dumps({"ok": True, "core_transitive_engine_dependencies": [],
                      "adapter_direction": "legacy -> adapter -> typed core",
                      "p1_tests_sha256": P1_TEST_SHA, "compatibility_baseline": BASE}))


if __name__ == "__main__":
    main()
