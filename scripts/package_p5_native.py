#!/usr/bin/env python3
"""Package an already-built P5 public SDK; never signs or publishes."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import zipfile

ROOT = Path(__file__).resolve().parents[1]
p = argparse.ArgumentParser()
p.add_argument('--library', type=Path, required=True)
p.add_argument('--out', type=Path, required=True)
a = p.parse_args()
a.out.mkdir(parents=True, exist_ok=True)
for source, name in [(a.library, a.library.name),
                     (ROOT/'samples/pentomino_p5/game.json', 'game.json'),
                     (ROOT/'samples/pentomino_p5/consumer.py', 'consumer.py'),
                     (ROOT/'samples/pentomino_p5/smoke.c', 'smoke.c'),
                     (ROOT/'crates/ge4g-pentomino-native/include/ge4g_p4_v1.h', 'ge4g_p4_v1.h'),
                     (ROOT/'crates/ge4g-pentomino-native/include/ge4g_p5_v1.h', 'ge4g_p5_v1.h'),
                     (ROOT/'docs/pentomino/P5_CONTRACT.md', 'P5_CONTRACT.md'),
                     (ROOT/'LICENSE', 'LICENSE')]:
    shutil.copyfile(source, a.out/name)
hashes = {f.name: hashlib.sha256(f.read_bytes()).hexdigest() for f in sorted(a.out.iterdir()) if f.is_file()}
(a.out/'SHA256SUMS.json').write_text(json.dumps(hashes, indent=2)+'\n')
archive = Path(str(a.out)+'.zip')
with zipfile.ZipFile(archive, 'w', zipfile.ZIP_DEFLATED) as z:
    for f in sorted(a.out.iterdir()):
        if f.is_file(): z.write(f, f.name)
print(json.dumps({'archive':str(archive), 'sha256':hashlib.sha256(archive.read_bytes()).hexdigest()}))
