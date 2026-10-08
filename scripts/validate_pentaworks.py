#!/usr/bin/env python3
"""Static bundle validation. This is not a test of the GE4G engine."""
from pathlib import Path
import sys, tomllib
ROOT = Path(__file__).resolve().parents[1]
NAMES = {'pentomino_scout', 'pentomino_architect', 'pentomino_core', 'pentomino_view', 'pentomino_plugin', 'pentomino_test', 'pentomino_auditor', 'pentomino_consumer', 'pentomino_optimizer', 'pentomino_release'}
READONLY = {'pentomino_scout','pentomino_auditor','pentomino_consumer'}
SKILLS = ('pentomino-orchestrate','pentomino-consumer-eval')
def validate(root=ROOT):
    problems=[]
    try:
        conf=tomllib.loads((root/'.codex/config.toml').read_text(encoding="utf-8"))
        if conf['agents'].get('enabled') is not True: problems.append('Agents disabled')
        if conf['agents'].get('max_concurrent_threads_per_session',0)<1: problems.append('Invalid concurrency')
    except Exception as e: problems.append(f'Config invalid: {e}')
    names=set()
    for f in (root/'.codex/agents').glob('*.toml'):
        try:
            role=tomllib.loads(f.read_text(encoding="utf-8"))
            name=role.get('name');names.add(name)
            if name!=f.stem: problems.append(f'{f}: name mismatch')
            if not all(isinstance(role.get(x),str) and role[x].strip() for x in ('name','description','developer_instructions')): problems.append(f'{f}: missing required field')
            if name in READONLY and role.get('sandbox_mode')!='read-only':problems.append(f'{name}: not read-only')
        except Exception as e:problems.append(f'{f}: invalid TOML: {e}')
    if names!=NAMES:problems.append(f'Agent inventory differs: missing {NAMES-names}; extra {names-NAMES}')
    for skill in SKILLS:
        f=root/'.agents/skills'/skill/'SKILL.md'
        try:
            t=f.read_text(encoding="utf-8")
            if not t.startswith('---\n') or f'name: {skill}' not in t.split('---',2)[1] or 'description:' not in t.split('---',2)[1]: problems.append(f'{skill}: invalid front matter')
        except Exception as e:problems.append(f'{skill}: {e}')
    for f in ('AGENTS.md','docs/pentomino/BRIEF.md','docs/pentomino/GATES.md','templates/RUNDOWN.md','consumer/CONSUMER_TASKS.md','docs/pentomino/HANDOFF.md','QUICKSTART.ko.md'):
        if not (root/f).is_file():problems.append(f'Missing {f}')
    return problems
if __name__=='__main__':
    errors=validate()
    for x in errors: print('FAIL:',x)
    if errors:sys.exit(1)
    print('PASS: project config, 10 custom Codex roles, 2 skills, design/brief/report templates')
    print('NOT TESTED: actual Codex spawning, GE4G runtime, plugin functionality or release gates')
