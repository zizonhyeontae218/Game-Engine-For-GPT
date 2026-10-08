#!/usr/bin/env python3
"""Focused current-product release assertions, not a natural-language linter."""
import argparse
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CURRENT = [
    "README.md", "AGENTS.md", "GOAL.md", "00_MASTER_CONCEPT.md",
    "docs/FLATLAND_AUTHORING.md", "docs/FLATLAND_QUICKSTART.md",
    "docs/FLATLAND_RELEASE.md", "docs/CLIENT.md", "docs/TESTPLAN.md",
    "docs/RELEASE_NOTES.md", "docs/ARCHITECTURE.md", "docs/BASEMENT_SPEC.md",
    "docs/FLATLAND_SPEC.md", "docs/LAUNCHER_PHILOSOPHY.md",
    "examples/flatland_harbor/README.md", "F(x).md", "docs/AUTHORING.md",
    "client/README.md", "client/packages/ge4g_native/README.md",
    "docs/pentomino/STATUS.md", "CODEX_START_PROMPT.md",
]
RULES = {
    "obsolete Lua claim": r"Lua.{0,30}(?:not (?:yet )?(?:implemented|included)|미구현)|Lua, 3D,.*이번 버전 범위에 포함하지",
    "obsolete audio claim": r"실제 음원 재생은 구현하지 않았|audio playback is not implemented|both adapters currently use a silent sink",
    "candidate product heading": r"(?m)^#{1,6} [^\n]*(?:rc[345]|0\.2\.0-rc\.[345]|[Cc]urrent candidate)",
    "candidate current version": r"(?:current (?:release|candidate|product)|현재.{0,20}버전|Current candidate):?[^\n]{0,100}0\.2\.0-rc\.[345]",
    "old current versionCode": r"(?:versionCode|Androidcode)\s*[:=]?\s*(?:[0-7]|[0-9]{2,})(?![0-9])",
    "third-party current sample": r"(?:current|public|공개|현재).{0,40}(?:sample|demo|샘플|데모).{0,40}(?:Nuvema|Pokémon|누베마)",
    "suspended client build": r"flutter build (?:linux|ios|macos)|unsigned iOS app|Windows/Arch clients.*required|Actual release CI covers Android, iOS",
}

def violations(text):
    return [label for label, pattern in RULES.items() if re.search(pattern, text, re.I)]

def self_test():
    stale = ["Lua is not implemented", "audio playback is not implemented",
             "# FlatLand rc5", "Current candidate:0.2.0-rc.5",
             "Android versionCode7", "current public sample is Nuvema",
             "flutter build linux --release"]
    assert all(violations(text) for text in stale)
    assert not violations("Lua5.4 is embedded. Audio adapters play cues. Android versionCode8. Preserve rc1–rc5 history. Linux clients deferred until0.3.")

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
    errors = []
    for name in CURRENT:
        for label in violations((ROOT/name).read_text(encoding="utf-8")):
            errors.append(f"{name}: {label}")
    workspace = tomllib.loads((ROOT/"Cargo.toml").read_text(encoding="utf-8"))
    if workspace["workspace"]["package"]["version"] != "0.3.0-alpha.1":
        errors.append("Rust workspace must be0.3.0-alpha.1")
    if not re.search(r"(?m)^version: 0\.3\.0-alpha\.1\+9$", (ROOT/"client/pubspec.yaml").read_text(encoding="utf-8")):
        errors.append("Flutter must be0.3.0-alpha.1+9")
    lock = tomllib.loads((ROOT/"Cargo.lock").read_text(encoding="utf-8"))
    members = workspace["workspace"]["members"]
    names = {tomllib.loads((ROOT/member/"Cargo.toml").read_text(encoding="utf-8"))["package"]["name"] for member in members}
    locked = {item["name"]: item["version"] for item in lock["package"] if item["name"] in names}
    if set(locked) != names or any(v != "0.3.0-alpha.1" for v in locked.values()):
        errors.append("Workspace lockfile must match0.3.0-alpha.1")
    for name in ("README.md", "AGENTS.md", "GOAL.md", "CODEX_START_PROMPT.md", "docs/pentomino/STATUS.md"):
        text = (ROOT/name).read_text(encoding="utf-8")
        if "Pentomino" not in text or "0.3.0-alpha.1" not in text:
            errors.append(f"{name}: missing Pentomino alpha development identity")
    android = (ROOT/"client/android/app/build.gradle.kts").read_text(encoding="utf-8")
    if 'applicationId = "dev.ge4g.ge4g_client"' not in android or "versionCode = flutter.versionCode" not in android:
        errors.append("Android identity/versionCode source changed")
    if (ROOT/"client/android/signing-certificate.sha256").read_text().strip() != "d6d5ca948e5c1ed644478d7c4c3243efcfcbc30a196f03c39ef7af7118fc00d4":
        errors.append("Pinned signing certificate changed")
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print(f"Pentomino0.3.0-alpha.1 consistency: {len(CURRENT)} current documents and release identity passed; historical documents excluded")
    return 0

if __name__ == "__main__":
    sys.exit(main())
