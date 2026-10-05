#!/usr/bin/env python3
"""Controlled author/repair tool-context benchmark, not an LLM performance claim."""
import argparse,base64,hashlib,json,shutil,subprocess,tempfile,time
from pathlib import Path
import tiktoken
ROOT=Path(__file__).resolve().parents[1]
ENCODING_SHA='223921b76ee99bde995b7ff738513eef100fb51d18c93597a113bcffe865b2a7'
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=ROOT/'artifacts/flatland/task-context.json');args=p.parse_args()
 enc=tiktoken.get_encoding('cl100k_base');vocab=b''.join(base64.b64encode(b)+b' '+str(rank).encode()+b'\n' for b,rank in sorted(enc._mergeable_ranks.items(),key=lambda p:p[1]));assert hashlib.sha256(vocab).hexdigest()==ENCODING_SHA;cli=ROOT/'target/debug/ge4g';source=ROOT/'examples/flatland_signal_yard'
 reports=[]
 with tempfile.TemporaryDirectory(prefix='ge4g-context-') as tmp:
  outputs=[]
  for route in ['full_resource_rewrite','revision_checked_patch']:
   project=Path(tmp)/route;shutil.copytree(source,project);file=project/'data/systems.json5';doc=json.loads(file.read_text());original=file.read_text();inputs=[];outputs_text=[];calls=0;failures=0;start=time.perf_counter()
   def call(*a,expected=0):
    nonlocal calls,failures
    cmd=[str(cli),*map(str,a),'--json'];inputs.append(' '.join(str(x).replace(str(project),'PROJECT') for x in cmd));r=subprocess.run(cmd,capture_output=True,text=True);calls+=1;outputs_text.append(r.stdout+r.stderr)
    if r.returncode:failures+=1
    assert r.returncode==expected,(a,r.stdout,r.stderr)
    return json.loads(r.stdout)
   task='Change blade attack damage from 3 to 4; repair an invalid damage=-1 attempt, preserve all other behavior.';inputs.append(task)
   if route=='full_resource_rewrite':
    inputs.append((ROOT/'docs/FLATLAND_AUTHORING.md').read_text());inputs.append(original);inputs.append(json.dumps(call('schema','attack')))
    doc['gameplay']['attacks']['blade']['damage']=-1;authored=json.dumps(doc,ensure_ascii=False,indent=2)+'\n';outputs_text.append(authored);file.write_text(authored);call('validate',project,expected=1)
    doc['gameplay']['attacks']['blade']['damage']=4;authored=json.dumps(doc,ensure_ascii=False,indent=2)+'\n';outputs_text.append(authored);file.write_text(authored);call('validate',project)
   else:
    inputs.append((ROOT/'docs/FLATLAND_QUICKSTART.md').read_text());inputs.append(json.dumps(call('schema','attack')));resource=call('resource',project,'data/systems.json5','gameplay/attacks/blade');inputs.append(json.dumps(resource));revision=resource['revision']
    patch=Path(tmp)/'patch.json';patch.write_text('{"damage":-1}');outputs_text.append(patch.read_text());call('patch',project,'data/systems.json5','gameplay/attacks/blade','--expected',revision,'--patch',patch,expected=2)
    assert file.read_text()==original
    patch.write_text('{"damage":4}');outputs_text.append(patch.read_text());call('patch',project,'data/systems.json5','gameplay/attacks/blade','--expected',revision,'--patch',patch)
   final=json.loads(file.read_text());expected=json.loads(original);expected['gameplay']['attacks']['blade']['damage']=4;assert final==expected
   # Both routes produce the same authoritative outcome, not just valid JSON.
   capture=Path(tmp)/(route+'.png');replay=project/'replays/journey.json';call('capture',project,'--replay',replay,'--tick','714','--out',capture)
   from PIL import Image
   with Image.open(capture) as im:frame=hashlib.sha256(im.convert('RGBA').tobytes()).hexdigest()
   outputs.append(frame)
   reports.append({'route':route,'task':task,'input_tokens':sum(len(enc.encode(x)) for x in inputs),'output_tokens':sum(len(enc.encode(x)) for x in outputs_text),'input_bytes':sum(len(x.encode()) for x in inputs),'output_bytes':sum(len(x.encode()) for x in outputs_text),'tool_calls':calls,'validation_failures':failures,'repair_iterations':1,'tool_completion_seconds':round(time.perf_counter()-start,3),'final_resource_sha256':hashlib.sha256(json.dumps(final,sort_keys=True).encode()).hexdigest(),'frame_rgba_sha256':frame})
  assert outputs[0]==outputs[1]
 report={'engine':'0.2.0','tokenizer':{'package':'tiktoken','version':tiktoken.__version__,'encoding':'cl100k_base','vocabulary_sha256':ENCODING_SHA},'scope':'Scripted same-task authoring/repair benchmark. Counts supplied docs, schema/resource responses, authored output and tool results. Includes one controlled error and repair per route. Times are tool execution, not model generation or human effort. Excludes this conversation and asset prompts; assets are deterministic CC0 Python generation. No claim of universal LLM efficiency.','tasks':reports,'functional_equivalence':True}
 args.out.parent.mkdir(parents=True,exist_ok=True);args.out.write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n');print(json.dumps(report,ensure_ascii=False))
if __name__=='__main__':main()
