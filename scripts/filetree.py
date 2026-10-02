#!/usr/bin/env python3
"""Maintain FILETREE.md and FILETREE.hash.json without third-party packages."""

from __future__ import annotations
from pathlib import Path
import hashlib
import json
import sys
import os

ROOT = Path(__file__).resolve().parents[1]
TREE = ROOT / "FILETREE.md"
HASHES = ROOT / "FILETREE.hash.json"

IGNORE_DIRS = {".git", "target", ".idea", ".vscode", "__pycache__", "artifacts", "dist"}
IGNORE_FILES = {"FILETREE.md", "FILETREE.hash.json", ".DS_Store", "save.json"}

PURPOSE = {
    "AGENTS.md": "Compact agent routing and non-negotiable repository rules.",
    "Agent.md": "Compatibility pointer to AGENTS.md.",
    "00_MASTER_CONCEPT.md": "Canonical GE4G concept and design philosophy.",
    "GOAL.md": "Basement observable product goal and scope.",
    "F(x).md": "Implemented stable cross-system state/identifier registry.",
    "CODEX_START_PROMPT.md": "Bootstrap task prompt for Codex Cloud.",
    "README.md": "Human entry point and ILCX target.",
    ".agent/PLANS.md": "ExecPlan policy for long/multi-stage work.",
    "docs/BASEMENT_SPEC.md": "0.1 product and runtime specification.",
    "docs/ARCHITECTURE.md": "Basement architecture and subsystem boundaries.",
    "docs/CLI_CONTRACT.md": "Agent-facing CLI contract.",
    "docs/TESTPLAN.md": "Acceptance and deterministic verification plan.",
    "docs/FAILURE_NOTES.md": "Durable record of non-obvious recurring failures.",
    "docs/AGENT_WORKFLOW.md": "Task-specific context/tool discipline guidance.",
    "docs/RESEARCH_BASIS.md": "Research basis for the agent setup.",
    "templates/EXECPLAN_TEMPLATE.md": "Template for checked-in execution plans.",
    "scripts/filetree.py": "FILETREE update/lint helper.",
    "scripts/interactive_smoke.py": "Real window/headless replay equivalence check.",
    "scripts/package.py": "Bundle a built executable, demo and user documentation.",
    "docs/AUTHORING.md": "Manifest, components, replay and assertion authoring guide.",
    "docs/RELEASE_NOTES.md": "Release evidence and honest human acceptance status.",
    "Cargo.toml": "Rust 2024 workspace and shared dependencies.",
    "Cargo.lock": "Pinned dependency resolution for reproducible builds.",
    ".github/workflows/ci.yml": "Clean-checkout headless and window acceptance CI.",
}

def relpaths():
    out = []
    for directory, dirs, files in os.walk(ROOT):
        dirs[:] = sorted(d for d in dirs if d not in IGNORE_DIRS)
        for name in files:
            if name not in IGNORE_FILES:
                out.append((Path(directory) / name).relative_to(ROOT).as_posix())
    return sorted(out)

def digest(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()

def render(paths):
    lines = [
        "# FILETREE",
        "",
        "Generated navigation index. It lists files that actually exist; do not predeclare future source files.",
        "",
        "`FILETREE.md` and `FILETREE.hash.json` are excluded from their own hash registry.",
        "",
    ]
    for path in paths:
        purpose = PURPOSE.get(path)
        if purpose is None and path.startswith("crates/"):
            purpose = "Engine source, package configuration or behavioral verification."
        if purpose is None and path.startswith("examples/basement_demo/"):
            purpose = "Playable reference demo data, replay or inspected framebuffer golden."
        if purpose is None:
            purpose = "Repository file; inspect only when relevant to the task."
        lines.append(f"- `{path}` — {purpose}")
    lines.append("")
    return "\n".join(lines)

def update():
    paths = relpaths()
    TREE.write_text(render(paths), encoding="utf-8")
    data = {p: digest(ROOT / p) for p in paths}
    HASHES.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"updated {TREE.name}: {len(paths)} files")

def lint():
    paths = relpaths()
    expected_tree = render(paths)
    expected_hashes = {p: digest(ROOT / p) for p in paths}
    ok = True
    if not TREE.exists() or TREE.read_text(encoding="utf-8") != expected_tree:
        print("FILETREE.md is stale", file=sys.stderr)
        ok = False
    try:
        actual_hashes = json.loads(HASHES.read_text(encoding="utf-8"))
    except Exception:
        actual_hashes = None
    if actual_hashes != expected_hashes:
        print("FILETREE.hash.json is stale", file=sys.stderr)
        ok = False
    return 0 if ok else 1

def main():
    cmd = sys.argv[1] if len(sys.argv) > 1 else "lint"
    if cmd in {"init", "update"}:
        update()
        return 0
    if cmd == "lint":
        return lint()
    print("usage: filetree.py [init|update|lint]", file=sys.stderr)
    return 2

if __name__ == "__main__":
    raise SystemExit(main())
