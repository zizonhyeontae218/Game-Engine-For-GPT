#!/usr/bin/env python3
"""Regenerate original CC0 Nuvema-inspired demo art (requires Pillow)."""
from pathlib import Path
from PIL import Image, ImageDraw
import json, random, shutil, math
R=Path('examples/flatland_nuvema');A=R/'assets'
for p in [A,R/'scenes',R/'data',R/'controls',R/'audio',R/'replays']:p.mkdir(parents=True,exist_ok=True)
rng=random.Random(7303)
def save(im,n):im.save(A/(n+'.png'))
def box(d,r,c,o=None):d.rectangle(r,fill=c,outline=o)
# Original CC0 art, reference town layout rebuilt from scratch.
ground=Image.new('RGBA',(512,576),(105,177,117,255));d=ImageDraw.Draw(ground)
for _ in range(24000):
 x,y=rng.randrange(512),rng.randrange(576);c=rng.choice(['#77b681','#80bc87','#67a879','#89bf8c']);d.line((x,y,x+2,y-1),fill=c)
# town dirt loop, upper exit and coastal promenade
path=Image.new('L',(512,576));pd=ImageDraw.Draw(path)
pd.line([(328,0),(328,144),(155,156),(148,282),(226,299),(332,296),(328,144)],fill=255,width=29)
pd.line([(226,294),(226,439)],fill=255,width=32)
pd.line([(104,437),(363,437)],fill=255,width=35)
pd.line([(144,363),(148,294)],fill=255,width=18);pd.line([(320,363),(328,297)],fill=255,width=18)
for x,y in [(328,144),(155,156),(148,282),(226,299),(332,296),(226,439)]:pd.ellipse((x-14,y-14,x+14,y+14),fill=255)
for y in range(478):
 for x in range(512):
  if path.getpixel((x,y)):
   v=rng.randrange(-8,9);ground.putpixel((x,y),(230+v,212+v,156+v,255))
for _ in range(1800):
 x,y=rng.randrange(512),rng.randrange(470)
 if path.getpixel((x,y)):d.point((x,y),fill='#cdbb85')
# coastal rock face, railed overlook, ocean
box(d,(0,463,511,575),'#2368ad')
for y in range(465,576):
 for x in range(512):
  v=rng.randrange(12);ground.putpixel((x,y),(28+v,89+v+(y%13==0)*10,159+v,255))
box(d,(0,451,511,488),'#a47b54')
for x in range(0,512,7):d.polygon([(x,452),(x+5,454),(x+4,482),(x,488)],fill=rng.choice(['#b98a5e','#916e51','#ceaa79']))
d.polygon([(52,451),(400,451),(386,489),(71,489)],fill='#a4aaa0',outline='#e4e0c4')
for x in range(64,393,12):box(d,(x,456,x+3,477),'#757f7a');box(d,(x,455,x+4,458),'#f4e9ce')
d.line((60,456,390,456),fill='#e7e2ca',width=4);d.line((68,478,386,478),fill='#727e79',width=3)
for x in range(0,512,5):d.line((x,492+(x%9),x+4,494+(x%9)),fill='#c2e5ee')
for _ in range(270):
 x,y=rng.randrange(512),rng.randrange(497,576);d.line((x,y,x+3,y),fill='#5ba4d8')
for x,y in [(73,245),(90,229),(370,244),(381,263),(369,426),(385,438),(82,268)]:
 d.ellipse((x-3,y-3,x+3,y+3),fill='#fdf4d2');box(d,(x-1,y-1,x+1,y+1),'#f2aa68')
# hedge and flower garden beside player house
for x in [191,352]:
 for y in range(210,256,7):d.ellipse((x,y,x+9,y+12),fill='#68912c',outline='#476925')
save(ground,'town_ground')
# tree sprites retain foreground occlusion
im=Image.new('RGBA',(32,48));dd=ImageDraw.Draw(im)
dd.ellipse((3,37,30,47),fill=(23,67,46,65));box(dd,(14,32,19,45),'#6d6448')
for y,rad in [(22,15),(14,13),(5,9)]:
 dd.polygon([(16,y),(16+rad,y+19),(16,y+25),(16-rad,y+19)],fill='#2d7659',outline='#245443')
 dd.polygon([(16,y+2),(16+rad-3,y+17),(16,y+21),(4 if rad==15 else 16-rad+3,y+17)],fill='#438f68')
 dd.line((16,y+4,12,y+14),fill='#66a77a',width=2)
save(im,'pine')
def house(name,w,h,roof,lab=False):
 im=Image.new('RGBA',(w,h));dd=ImageDraw.Draw(im)
 dd.ellipse((5,h-13,w-1,h),fill=(21,57,44,70))
 box(dd,(7,h//2,w-9,h-5),'#c8a77a','#594e46');box(dd,(w-9,h//2-6,w-2,h-10),'#91745b','#594e46')
 for yy in range(h//2,h-6,5):dd.line((8,yy,w-10,yy),fill='#b19472')
 dd.polygon([(5,h//2),(w//2,3),(w-4,h//2),(w-4,h//2+9),(w//2,14),(5,h//2+10)],fill=roof,outline='#29444e')
 dd.polygon([(w//2,3),(w-4,h//2),(w-4,h//2+9),(w//2,14)],fill='#205c8f' if not lab else '#994b22')
 for xx in range(w//2+4,w-5,6):dd.line((xx,xx-w//2+3,xx,xx-w//2+14),fill='#69b8dd' if not lab else '#e29a4d')
 dd.polygon([(9,h//2+6),(w//2,19),(w-12,h//2+6)],fill='#b29370',outline='#5e534a')
 box(dd,(w//2-7,h-29,w//2+7,h-5),'#524438','#352f2b');box(dd,(w//2-5,h-26,w//2+4,h-7),'#7a5c43');dd.point((w//2+3,h-16),fill='#efd477')
 for xx in [14,w-29]:
  box(dd,(xx,h-29,xx+12,h-17),'#628e9d','#eee3c8');dd.line((xx+6,h-29,xx+6,h-17),fill='#dedaca');dd.line((xx,h-23,xx+12,h-23),fill='#dedaca')
 box(dd,(w//2-9,h-4,w//2+9,h-2),'#d9cbb0')
 if lab:
  box(dd,(w//2-16,h//2+7,w//2+16,h//2+19),'#77522e','#ddbc78')
  for xx in range(w//2-11,w//2+12,5):box(dd,(xx,h//2+11,xx+2,h//2+15),'#e9d07c')
 save(im,name)
house('house_blue',80,96,'#277fbb');house('lab',128,112,'#c56f30',True)
# small objects
for name,c in [('crate','#b77f46'),('sign','#a8aaa0'),('mailbox','#576b74'),('dummy','#cf625c'),('console','#7593a4'),('orb','#74bfd3')]:
 im=Image.new('RGBA',(16,24));dd=ImageDraw.Draw(im)
 if name=='orb':dd.ellipse((3,3,13,13),fill=c,outline='#d9f6e4')
 else:
  box(dd,(3,4,13,15),c,'#343f42');box(dd,(7,16,9,23),'#5c6354')
  if name=='crate':dd.line((3,4,13,15),fill='#6b4b2e');dd.line((13,4,3,15),fill='#6b4b2e')
  else:dd.line((5,7,11,7),fill='#e2ddae');dd.line((5,10,11,10),fill='#e2ddae')
 save(im,name)
# 4 directions x 3 walking poses; original trainers/npcs
for kind,shirt,hair in [('hero','#2c6da6','#473b36'),('friend','#62a94b','#704d36'),('professor','#eeeece','#695349'),('mom','#c78499','#62452c')]:
 for direction in ['down','up','left','right']:
  for phase in range(3):
   im=Image.new('RGBA',(24,32));dd=ImageDraw.Draw(im);dd.ellipse((4,25,21,31),fill=(17,40,33,60))
   step=phase-1
   box(dd,(8,21,11,28+max(step,0)),'#343d50');box(dd,(14,21,17,28+max(-step,0)),'#343d50')
   box(dd,(7,13,18,22),shirt,'#343c47');box(dd,(5,14,7,21),'#e7bc91');box(dd,(18,14,20,21),'#e7bc91')
   box(dd,(6,4,19,13),'#eac29a','#463e3a');box(dd,(6,2,19,7),hair)
   if direction!='up':
    ex=[9,16] if direction=='down' else [7] if direction=='left' else [17]
    for x in ex:box(dd,(x,9,x+1,10),'#303847')
   if kind=='hero':box(dd,(5,2,20,5),'#eee9d5');box(dd,(13 if direction!='left' else 3,5,21 if direction!='left' else 12,6),'#d55048')
   save(im,f'{kind}_{direction}_{phase}')
# two original companion creatures front/back, no copyrighted Pokemon sprite copies
for name,color in [('sprout','#71b15d'),('ember','#e19550'),('tide','#68a8c6'),('moss','#a2ac68')]:
 for back in [False,True]:
  im=Image.new('RGBA',(80,80));dd=ImageDraw.Draw(im)
  dd.ellipse((13,58,70,73),fill=(34,51,40,45));dd.polygon([(16,55),(4,32),(30,44)],fill=color,outline='#344b47')
  dd.ellipse((20,29,65,68),fill=color,outline='#344b47',width=2)
  dd.ellipse((23,12,68,48),fill=color,outline='#344b47',width=2)
  if name in ['sprout','moss']:
   dd.polygon([(38,18),(23,2),(48,7),(59,0),(62,23)],fill='#456e43',outline='#344b47')
  elif name=='ember':dd.polygon([(59,24),(70,3),(74,30)],fill='#edb761',outline='#8a5235')
  else:dd.polygon([(30,18),(14,5),(24,32)],fill='#a6d1e2',outline='#344b47')
  if not back:
   dd.ellipse((34,25,42,36),fill='#f9edd1');dd.ellipse((53,25,61,36),fill='#f9edd1');box(dd,(38,27,41,34),'#2b414a');box(dd,(56,27,59,34),'#2b414a');dd.line((43,42,53,42),fill='#37514b',width=2)
  else:dd.arc((30,22,59,43),0,160,fill='#4d755b',width=2)
  for x in [24,51]:dd.ellipse((x,62,x+17,72),fill=color,outline='#344b47')
  save(im,name+('_back' if back else ''))
# battle grass field background
im=Image.new('RGBA',(320,240),'#b6d0af');dd=ImageDraw.Draw(im)
for y in range(240):dd.line((0,y,319,y),fill=(164+y//6,204+y//12,170+y//15,255))
for y in [94,198]:dd.ellipse((156 if y==94 else 12,y-13,303 if y==94 else 150,y+15),fill='#91b58b',outline='#709475')
for _ in range(600):
 x,y=rng.randrange(320),rng.randrange(85,235);dd.line((x,y,x+2,y-1),fill='#a1c797')
save(im,'battle_field')
# Home and lab interiors
for name in ['home','laboratory']:
 im=Image.new('RGBA',(320,240),'#b5a17f');dd=ImageDraw.Draw(im)
 for y in range(0,240,16):
  for x in range(0,320,16):box(dd,(x,y,x+15,y+15),'#c7b18b' if (x//16+y//16)%2 else '#cdbb99','#b2a07e')
 box(dd,(0,0,319,47),'#ddd4b5');box(dd,(0,43,319,47),'#77654e');box(dd,(0,0,15,239),'#75654e');box(dd,(304,0,319,239),'#75654e')
 for x in [48,240]:box(dd,(x,8,x+31,34),'#e7e3bf','#836e54');box(dd,(x+3,10,x+28,31),'#73b3c6');dd.line((x+16,10,x+16,31),fill='#e9e5cf',width=2)
 if name=='home':
  box(dd,(32,66,80,126),'#655e70');box(dd,(35,70,77,98),'#ede3c4');box(dd,(35,98,77,121),'#7499ab')
  box(dd,(216,65,287,94),'#6c6250');box(dd,(223,66,281,87),'#ede8c9');box(dd,(124,105,191,138),'#90734f','#5a5144')
 else:
  for x in [32,224]:box(dd,(x,60,x+63,92),'#739294','#455c63');box(dd,(x+5,63,x+58,84),'#bed3c4')
  for x in [96,128,160]:dd.ellipse((x,66,x+14,80),fill='#dcddaf',outline='#5e7474')
 box(dd,(144,219,175,239),'#836b4f')
 save(im,name+'_ground')

from PIL import ImageDraw
p=A/"town_ground.png"
im=Image.open(p);d=ImageDraw.Draw(im)
for x in range(304,368,8):d.rectangle((x,144,x+2,154),fill="#846e46")
d.line((304,147,367,147),fill="#ece0b2",width=2);im.save(p)
