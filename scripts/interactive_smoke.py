#!/usr/bin/env python3
"""Compare the actual window adapter to headless play using the same replay.

Run with a display, for example: xvfb-run -a python3 scripts/interactive_smoke.py
This is automated adapter evidence; it is not a human acceptance test.
"""
from pathlib import Path
import json
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def main():
    binary = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else ROOT / "target/debug/ge4g"
    project = ROOT / "examples/basement_demo"
    replay = project / "replays/journey.json"
    outputs = []
    for mode in [[], ["--headless"]]:
        result = subprocess.run(
            [str(binary), "run", str(project), "--ticks", "160", "--replay", str(replay), "--json", *mode],
            capture_output=True, text=True, timeout=30, check=False,
        )
        if result.returncode:
            raise RuntimeError(f"adapter failed ({result.returncode}): {result.stdout}\n{result.stderr}")
        response = json.loads(result.stdout)
        if not response["ok"]:
            raise RuntimeError(response)
        outputs.append(response)
    if outputs[0]["snapshot"] != outputs[1]["snapshot"]:
        raise AssertionError("window and headless snapshots/events diverged")
    snapshot = outputs[0]["snapshot"]
    if snapshot["scene"] != "room_b" or snapshot["tick"] != 160:
        raise AssertionError("demo did not finish in room_b at tick 160")
    print(f"window/headless match: tick 160, room_b, {outputs[0]['deterministic_sha256']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
