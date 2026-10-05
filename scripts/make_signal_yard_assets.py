#!/usr/bin/env python3
"""Generate original CC0 Signal Yard PNG/WAV assets deterministically."""
from pathlib import Path
import json,math,wave,struct
from PIL import Image,ImageDraw
root=Path(__file__).resolve().parents[1]/'examples/flatland_signal_yard'
for d in ['scenes','data','assets','audio','controls','replays']: (root/d).mkdir(parents=True,exist_ok=True)
def write(p,obj): (root/p).write_text(json.dumps(obj,ensure_ascii=False,indent=2)+'\n')
def action(op,**kw):return dict(op=op,**kw)
def say(text):return dict(op='say',text=text)
# Original tiny industrial sprites, drawn deterministically and licensed CC0.
for actor,color in [('hero',(95,205,240)),('guide',(210,255,56)),('guard',(249,89,82)),('drone',(255,155,52))]:
 for frame in range(4):
  im=Image.new('RGBA',(16,16));p=ImageDraw.Draw(im)
  p.rectangle((4,1,11,5),fill=color+(255,));p.rectangle((3,6,12,12),fill=color+(255,));p.rectangle((5,3,10,4),fill=(15,20,25,255));p.rectangle((4+(frame%2),13,6+(frame%2),15),fill=(230,230,210,255));p.rectangle((9-(frame%2),13,11-(frame%2),15),fill=(230,230,210,255));p.point((4,7+frame),fill=(255,255,255,255));im.save(root/f'assets/{actor}_{frame}.png')
# Atlas of the guide for actual crop/marker playback.
atlas=Image.new('RGBA',(64,16))
for i in range(4):atlas.paste(Image.open(root/f'assets/guide_{i}.png'),(i*16,0))
atlas.save(root/'assets/guide_atlas.png')
for name,color,kind in [('crate',(194,143,78),'box'),('key',(248,225,48),'key'),('terminal',(179,110,247),'terminal'),('gate',(249,89,82),'gate'),('canopy',(65,100,75),'canopy'),('ramp',(80,180,195),'ramp')]:
 size=(32,32) if name=='canopy' else (16,16);im=Image.new('RGBA',size);p=ImageDraw.Draw(im)
 if kind=='key':p.ellipse((2,3,9,10),fill=color+(255,));p.rectangle((8,8,14,10),fill=color+(255,));p.rectangle((12,9,14,13),fill=color+(255,))
 else:p.rectangle((1,1,size[0]-2,size[1]-2),fill=color+(255,),outline=(15,20,25,255),width=2)
 if kind=='terminal':p.rectangle((4,4,11,8),fill=(210,255,56,255));p.line((4,11,11,11),fill=(15,20,25,255),width=2)
 if kind=='box':p.line((3,3,12,12),fill=(40,40,25,255),width=2);p.line((12,3,3,12),fill=(40,40,25,255),width=2)
 if kind=='ramp':p.polygon([(3,11),(8,4),(13,11)],fill=(210,255,56,255))
 im.save(root/f'assets/{name}.png')
(root/'assets/LICENSE-CC0.txt').write_text('Signal Yard sprites and synthesized audio are original assets dedicated to CC0 1.0 Universal.\nhttps://creativecommons.org/publicdomain/zero/1.0/\nGenerated from the checked-in scripts/make_signal_yard_assets.py; no external asset dependency.\n')
for name,notes,duration in [('hit',[220,110],.18),('door',[660,880],.22),('quest',[440,660,880],.4),('music',[220,277,330,440,330,277,247,330],4.0)]:
 rate=22050;samples=[]
 for i in range(int(rate*duration)):
  seg=int(i/(rate*duration/len(notes)));freq=notes[min(seg,len(notes)-1)];phase=i/rate;env=min(1,(i%max(1,int(rate*duration/len(notes))))/(rate*.01))*max(0,1-(i%max(1,int(rate*duration/len(notes))))/(rate*duration/len(notes)));samples.append(int(math.sin(2*math.pi*freq*phase)*env*3000))
 with wave.open(str(root/f'audio/{name}.wav'),'wb') as w:w.setnchannels(1);w.setsampwidth(2);w.setframerate(rate);w.writeframes(struct.pack('<'+'h'*len(samples),*samples))
