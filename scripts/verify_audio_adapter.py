#!/usr/bin/env python3
"""Linux native adapter virtual-PCM proof; use a display (e.g. xvfb-run)."""
import argparse,json,os,subprocess,tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=ROOT/'artifacts/flatland/audio-adapter.json');args=p.parse_args()
 with tempfile.TemporaryDirectory(prefix='ge4g-pcm-') as work:
  raw=Path(work)/'mixed.raw';config=Path(work)/'alsa.conf'
  config.write_text('pcm.null { type null }\npcm.!default { type file slave.pcm "null" file "'+str(raw)+'" format "raw" hint { show on description "GE4G verification output" } }\n')
  env=dict(os.environ,ALSA_CONFIG_PATH=str(config));project=ROOT/'examples/flatland_signal_yard'
  result=subprocess.run([str(ROOT/'target/debug/ge4g'),'run',str(project),'--replay',str(project/'replays/journey.json'),'--ticks','714','--json'],env=env,capture_output=True,text=True,check=True,timeout=45)
  response=json.loads(result.stdout);assert response['state']['yard.complete']
  count=nonzero=0
  with raw.open('rb') as f:
   while block:=f.read(1024*1024):count+=len(block);nonzero+=sum(b!=0 for b in block)
  assert count>10000 and nonzero>0
  report={'adapter':'native rodio/cpal ALSA file output','pcm_bytes':count,'nonzero_bytes':nonzero,'audible_human_check':False,'scope':'Actual decoder/mixer and virtual PCM output; not physical speaker acceptance.'}
  args.out.parent.mkdir(parents=True,exist_ok=True);args.out.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
if __name__=='__main__':main()
