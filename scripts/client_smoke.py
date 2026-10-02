#!/usr/bin/env python3
"""Prove the packaged Flutter executable uses the authoritative replay and pixels."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('bundle', type=Path)
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix='ge4g-client-smoke-') as work:
        output = Path(work)
        env = dict(os.environ, GE4G_CLIENT_DATA=str(output / 'user'), GE4G_SMOKE_OUTPUT=str(output / 'evidence'))
        # A real display (Xvfb is acceptable) is required. No synthesized frames.
        command = [str(args.bundle.resolve() / 'ge4g_client'), '--smoke-test']
        subprocess.run(command, check=True, env=env, timeout=90)
        evidence = output / 'evidence'
        result = json.loads((evidence / 'result.json').read_text())
        assert result['ok'] and result['mode'] == 'embedded' and result['tick'] == 160 and result['scene'] == 'room_b', result
        reference = output / 'headless.json'
        subprocess.run([str(ROOT / 'target/debug/ge4g'), 'run', str(ROOT / 'examples/basement_demo'), '--headless', '--replay', str(ROOT / 'examples/basement_demo/replays/journey.json'), '--ticks', '160', '--snapshot-out', str(reference)], check=True, capture_output=True)
        expected = json.loads(reference.read_text())
        actual = json.loads((evidence / 'snapshot.json').read_text())
        assert actual == expected, 'Flutter/native snapshot diverges from CLI/headless replay'
        from PIL import Image
        with Image.open(evidence / 'frame.png') as frame:
            digest = hashlib.sha256(frame.convert('RGBA').tobytes()).hexdigest()
        assert digest == '38cc4352e7160dcf9104cbe19acd709e36d8165989b3c99d8b6611f0f76e932b', digest
        destination = ROOT / 'artifacts/client-smoke'
        destination.mkdir(parents=True, exist_ok=True)
        import shutil
        for name in ['result.json', 'snapshot.json', 'frame.png']:
            shutil.copy2(evidence / name, destination / name)
        print(json.dumps({'ok': True, 'tick': 160, 'scene': 'room_b', 'rgba_sha256': digest, 'snapshot_equal': True}))

if __name__ == '__main__':
    main()
