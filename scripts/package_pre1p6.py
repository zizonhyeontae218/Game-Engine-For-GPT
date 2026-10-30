#!/usr/bin/env python3
"""Bundle an existing Windows release for local acceptance; never publishes."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import zipfile

p = argparse.ArgumentParser()
p.add_argument('--client', type=Path, required=True)
p.add_argument('--out', type=Path, required=True)
a = p.parse_args()
assert (a.client/'ge4g_client.exe').is_file(), 'actual Windows EXE required'
if a.out.exists():
    raise SystemExit('refusing to overwrite existing acceptance bundle')
shutil.copytree(a.client, a.out)
(a.out/'PLAY-PENTOMINO.cmd').write_bytes(b'@echo off\r\ncd /d "%~dp0"\r\nstart "" "ge4g_client.exe" --pentomino\r\n')
(a.out/'README.txt').write_text('GE4G 0.3 Pentomino pre1P6\n\n압축을 모두 풀고 PLAY-PENTOMINO.cmd를 실행하세요.\n행동과 대상을 고른 뒤 실행합니다. 두 팀의 턴을 번갈아 조작합니다.\n일반 라이브러리는 ge4g_client.exe로 실행할 수 있습니다.\n확인할 것: 화면 표시, 행동/대상 선택, 승패까지 플레이.\n자동 검증과 실제 기기 사용자 수용은 구분됩니다.\n서명/정식 릴리스가 아닌 검증용 빌드입니다.\n', encoding='utf-8')
hashes = {str(f.relative_to(a.out)).replace('\\', '/'):hashlib.sha256(f.read_bytes()).hexdigest() for f in sorted(a.out.rglob('*')) if f.is_file()}
(a.out/'SHA256SUMS.json').write_text(json.dumps(hashes, indent=2)+'\n')
archive = Path(str(a.out)+'.zip')
with zipfile.ZipFile(archive, 'w', zipfile.ZIP_DEFLATED) as z:
    for f in sorted(a.out.rglob('*')):
        if f.is_file(): z.write(f, str(f.relative_to(a.out)))
sha = hashlib.sha256(archive.read_bytes()).hexdigest()
Path(str(archive)+'.sha256').write_text(sha+'  '+archive.name+'\n')
print(json.dumps({'artifact':str(archive), 'sha256':sha, 'bytes':archive.stat().st_size}))
