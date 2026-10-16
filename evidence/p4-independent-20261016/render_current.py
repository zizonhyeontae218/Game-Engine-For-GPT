from pathlib import Path
from PIL import Image
from independent_p4_test import new,frame,step,close
h=new()
try:
 for n in range(6):
  if n:step(h,n)
  if n in (0,5):
   f=frame(h);Image.frombytes('RGBA',(f['width'],f['height']),bytes(f['rgba'])).resize((640,384),Image.Resampling.NEAREST).save(Path(__file__).resolve().parent/f'current-{n}.png')
finally:close(h)
