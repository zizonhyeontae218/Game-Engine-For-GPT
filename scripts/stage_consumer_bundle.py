#!/usr/bin/env python3
"""Stage only explicitly allowlisted public artifacts in a fresh workspace outside the engine repo."""
import argparse,hashlib,json,shutil,sys
from pathlib import Path,PurePosixPath
PUBLIC=('docs/public/','public/','dist/','releases/','examples/public/')
EXACT={'README.md','LICENSE','LICENSE.md','CHANGELOG.md','consumer/CONSUMER_TASKS.md'}
DENY={'.git','.codex','.agents','node_modules','__pycache__','internal','secrets','tests','.github'}
class StageError(ValueError):pass
def validate_name(s):
    if not s or '\\' in s or '\x00' in s or s.startswith('/') or str(PurePosixPath(s))!=s or any(p in ('..','.') for p in s.split('/')):raise StageError('Unsafe relative path: '+s)
    p=PurePosixPath(s)
    if any(x.lower() in DENY or x.lower().startswith('.env') for x in p.parts):raise StageError('Private path: '+s)
    if p.suffix.lower() in {'.pem','.key','.pfx','.p12'}:raise StageError('Possible private key: '+s)
    if s not in EXACT and not s.startswith(PUBLIC):raise StageError('Not public artifact scope: '+s)
    return p
def stage(root,allowlist,dest):
    root=Path(root).resolve(strict=True);dest=Path(dest).resolve(strict=False)
    if not root.is_dir():raise StageError('root not directory')
    if root==dest or root in dest.parents or dest in root.parents:raise StageError('Consumer destination must be separate from source and its ancestors')
    if dest.exists():raise StageError('Destination exists; use fresh directory')
    entries=[x.strip() for x in Path(allowlist).read_text(encoding="utf-8").splitlines() if x.strip() and not x.strip().startswith('#')]
    if not entries:raise StageError('Allowlist empty; publish real public docs/distribution before consumer test')
    if len(set(entries))!=len(entries):raise StageError('Duplicate paths')
    sources=[]
    for s in entries:
        p=validate_name(s);src=root.joinpath(*p.parts)
        if any(q.is_symlink() for q in [src,*[root.joinpath(*p.parts[:i]) for i in range(1,len(p.parts))]]):raise StageError('Symlink forbidden: '+s)
        if not src.is_file() or not src.resolve().is_relative_to(root):raise StageError('Missing/unsafe source: '+s)
        sources.append((s,src))
    dest.mkdir(parents=True)
    manifest=[]
    for s,src in sources:
        dst=dest/s;dst.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(src,dst,follow_symlinks=False)
        manifest.append({'path':s,'bytes':dst.stat().st_size,'sha256':hashlib.sha256(dst.read_bytes()).hexdigest()})
    challenge=root/'consumer/CONSUMER_TASKS.md'
    if challenge.is_file() and 'consumer/CONSUMER_TASKS.md' not in entries:
        dst=dest/'CONSUMER_TASKS.md';shutil.copyfile(challenge,dst)
        manifest.append({'path':'CONSUMER_TASKS.md','bytes':dst.stat().st_size,'sha256':hashlib.sha256(dst.read_bytes()).hexdigest()})
    (dest/'PUBLIC_BUNDLE_MANIFEST.json').write_text(json.dumps({'files':manifest},indent=2)+'\n', encoding='utf-8')
    return manifest
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for arg in ('root','allowlist','dest'):p.add_argument('--'+arg,required=True,type=Path)
    a=p.parse_args()
    try:result=stage(a.root,a.allowlist,a.dest)
    except (StageError,OSError) as e:print('BLOCKED:',e,file=sys.stderr);sys.exit(1)
    print(f'STAGED {len(result)} public files at {a.dest}')
    print('SECURITY NOTE: Do not grant the consumer session access to source files, shared mounts or private connectors.')
