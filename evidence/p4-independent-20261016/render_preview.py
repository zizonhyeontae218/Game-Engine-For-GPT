import sys,hashlib

from independent_p4_test import api,config
from PIL import Image
from pathlib import Path
O=Path(__file__).resolve().parent
h=api.good('create',config=config,seed=42,mode='linked')['session']
try:
 for stage in range(7):
  if stage:
   if stage==4:
    api.good('gameplay',session=h,enabled=False)
    api.good('step',session=h,input={'version':1,'target_step':4,'controls':[]})
    api.good('gameplay',session=h,enabled=True)
    api.good('camera',session=h,request={'version':1,'zoom':1.25,'tilt':1.0,'shear':0.0,'origin':[0,0],'transition_ticks':0})
   else:
    api.good('step',session=h,input={'version':1,'target_step':stage,'controls':['attack']})
  if stage in (0,5,6):
   f=api.good('frame',session=h);image=Image.frombytes('RGBA',(f['width'],f['height']),bytes(f['rgba']));image.resize((800,480),resample=Image.Resampling.NEAREST).save(O/('preview-%d.png'%stage))
   print(stage,f['sha256'],api.good('observe',session=h)['battle'])
finally:api.good('close',session=h)
