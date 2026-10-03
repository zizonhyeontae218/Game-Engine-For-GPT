#!/usr/bin/env python3
"""Measure static authoring/query context; optional tiktoken is a developer dependency."""
import argparse
import json
from pathlib import Path
import subprocess
import tomllib

ROOT=Path(__file__).resolve().parents[1]
def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out',type=Path,default=ROOT/'artifacts/flatland/context.json')
    args=parser.parse_args()
    cli=ROOT/'target/debug/ge4g'
    project=ROOT/'examples/flatland_pacman'
    scene=json.loads((project/'maze.json5').read_text())
    manifest=tomllib.loads((project/'ge4g.toml').read_text())
    source=(project/'maze.json5').read_text()
    (ROOT/'artifacts/flatland').mkdir(parents=True,exist_ok=True)
    snapshot=json.loads(subprocess.check_output([str(cli),'run',str(project),'--headless','--ticks','0','--snapshot-out',str(ROOT/'artifacts/flatland/context-snapshot.json'),'--json'],text=True))
    # Compare broad scene observation to one actor, independently of source size.
    expanded=json.loads(subprocess.check_output([str(cli),'inspect',str(project),'scene','--json'],text=True))
    full_schema=subprocess.check_output([str(cli),'schema','scene','--json'],text=True)
    body_schema=subprocess.check_output([str(cli),'schema','body','--json'],text=True)
    one_entity=subprocess.check_output([str(cli),'inspect',str(project),'entity','player','--json'],text=True)
    full_snapshot=(ROOT/'artifacts/flatland/context-snapshot.json').read_text()
    try:
        import tiktoken
        encoder=tiktoken.get_encoding('o200k_base')
        token=lambda s:len(encoder.encode(s))
        tokenizer={'package':f'tiktoken {tiktoken.__version__}','encoding':'o200k_base'}
    except Exception:
        token=lambda s:None
        tokenizer=None
    def metrics(s):return {'bytes':len(s.encode()),'tokens':token(s)}
    report={'engine':json.loads(full_snapshot).get('engine_version'), 'tokenizer':tokenizer,
        'scope':'Static source/query size only; excludes agent conversation, repair iterations and asset prompts.',
        'source_scene':metrics(source),'scene_observation':metrics(json.dumps(expanded,separators=(',',':'))),
        'schema_scene':metrics(full_schema),'schema_body':metrics(body_schema),
        'full_snapshot':metrics(full_snapshot),'one_entity_observation':metrics(one_entity),
        'declared_actor_instances':len(scene['entities']),'map_rows':len(scene['map']['rows']),
        'game_schema':manifest['schema_version']}
    args.out.parent.mkdir(parents=True,exist_ok=True)
    args.out.write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report))
if __name__=='__main__':main()
