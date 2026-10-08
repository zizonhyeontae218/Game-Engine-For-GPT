#!/usr/bin/env python3
"""Check gate evidence report shape, never certify engine functionality."""
import argparse,json,sys
from pathlib import Path
GATES={f'G{i}' for i in range(9)}
STATES={'PASS','FAIL','BLOCKED','UNVERIFIED','NOT_APPLICABLE'}
def check(data):
    if not isinstance(data,dict):return ['root not object']
    errors=[]
    if not data.get('release_id') or str(data['release_id']).startswith('EXAMPLE'):errors.append('real release_id required')
    if not data.get('source_commit'):errors.append('source_commit required')
    entries=data.get('gates')
    if not isinstance(entries,list):return errors+['gates not list']
    ids=[g.get('id') for g in entries if isinstance(g,dict)]
    if len(ids)!=9 or set(ids)!=GATES:errors.append('G0..G8 required exactly once')
    for g in entries:
        if not isinstance(g,dict):errors.append('invalid gate entry');continue
        name=g.get('id');state=g.get('status');evidence=g.get('evidence')
        if state not in STATES:errors.append(f'{name}: invalid status')
        if not isinstance(evidence,list):errors.append(f'{name}: evidence not list');continue
        if state=='PASS' and not evidence:errors.append(f'{name}: PASS without evidence')
        if state=='PASS':
            for item in evidence:
                if not isinstance(item,dict) or not all(isinstance(item.get(x),str) and item[x].strip() for x in ('command','observed','artifact')):errors.append(f'{name}: evidence needs command/observed/artifact')
        if state=='NOT_APPLICABLE' and not g.get('rationale'):errors.append(f'{name}: N/A needs rationale')
    return errors
if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('report',type=Path);a=parser.parse_args()
    try:errors=check(json.loads(a.report.read_text(encoding="utf-8")))
    except Exception as e:print('ERROR:',e);sys.exit(2)
    for e in errors:print('INVALID:',e)
    if errors:sys.exit(1)
    print('PASS: structural gate evidence completeness only. Actual execution and truth NOT verified.')
