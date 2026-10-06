#!/usr/bin/env python3
"""Windows UTF-8 author -> validate -> package -> Flutter import/launch/capture."""
from pathlib import Path
import argparse,ctypes,json,os,shutil,subprocess,time
if hasattr(os.sys.stdout,'reconfigure'):os.sys.stdout.reconfigure(encoding='utf-8')
ROOT=Path(__file__).resolve().parents[1]
WORK=ROOT/'dist'/'한글 검증'; PROJECT=WORK/'공방 프로젝트'; PACKAGE=WORK/'바람항 공방.ge4g'; EVIDENCE=ROOT/'artifacts'/'rc5-windows'
def prepare():
 if WORK.exists():shutil.rmtree(WORK)
 WORK.mkdir(parents=True);shutil.copytree(ROOT/'examples/flatland_harbor',PROJECT)
 validator=ROOT/'target/debug'/('ge4g.exe' if os.name=='nt' else 'ge4g')
 result=subprocess.run([str(validator),'validate',str(PROJECT),'--json'],capture_output=True,encoding='utf-8',check=True)
 report=json.loads(result.stdout);assert report['ok'],report
 assert any('항구' in f for f in report['files_checked'])
 subprocess.run([os.sys.executable,str(ROOT/'scripts/pack_game.py'),str(PROJECT),'--game-id','demo.flatland.harbor','--version','0.2.0-rc.5','--out',str(PACKAGE),'--validator',str(validator)],check=True,encoding='utf-8')
 EVIDENCE.mkdir(parents=True,exist_ok=True);(EVIDENCE/'validator-utf8.json').write_text(result.stdout,encoding='utf-8')
def launch():
 if os.name!='nt':raise SystemExit('Windows acceptance must run on Windows')
 destination=WORK/'내장 실행기'
 subprocess.run([os.sys.executable,str(ROOT/'scripts/bundle_desktop.py'),'--client',str(ROOT/'client/build/windows/x64/runner/Release'),'--game',str(PACKAGE),'--platform','windows','--out',str(destination)],check=True)
 user32=ctypes.windll.user32
 from ctypes import wintypes
 callback=ctypes.WINFUNCTYPE(wintypes.BOOL,wintypes.HWND,wintypes.LPARAM)
 user32.GetWindowTextW.argtypes=[wintypes.HWND,wintypes.LPWSTR,ctypes.c_int]
 user32.GetWindowThreadProcessId.argtypes=[wintypes.HWND,ctypes.POINTER(wintypes.DWORD)]
 user32.SetWindowPos.argtypes=[wintypes.HWND,wintypes.HWND,ctypes.c_int,ctypes.c_int,ctypes.c_int,ctypes.c_int,ctypes.c_uint]
 for label,size in [('portrait',(430,900)),('landscape',(1000,650))]:
  output=EVIDENCE/label
  env=dict(os.environ,GE4G_CLIENT_DATA=str(WORK/f'저장 {label}'),GE4G_SMOKE_OUTPUT=str(output))
  process=subprocess.Popen([str(destination/'ge4g_client.exe'),'--rc5-evidence',*(['--capture-landscape'] if label=='landscape' else [])],env=env)
  @callback
  def resize(hwnd,_):
   pid=wintypes.DWORD();user32.GetWindowThreadProcessId(hwnd,ctypes.byref(pid))
   title=ctypes.create_unicode_buffer(256);user32.GetWindowTextW(hwnd,title,256)
   if pid.value==process.pid and title.value=='GE4G / GameEngineForGPT':user32.SetWindowPos(hwnd,None,30,30,*size,0x0040)
   return True
  deadline=time.monotonic()+90
  try:
   while process.poll() is None and time.monotonic()<deadline:
    user32.EnumWindows(resize,0);time.sleep(.15)
   assert process.wait(timeout=5)==0,'Flutter rendered acceptance failed'
   result=json.loads((output/'result.json').read_text(encoding='utf-8'));assert result['ok'] and result['view_entities_equal']
   top=json.loads((output/'01-top.json').read_text(encoding='utf-8'));depth=json.loads((output/'02-depth.json').read_text(encoding='utf-8'))
   assert top['entities']==depth['entities'] and top['scene']=='항구'
   from PIL import Image
   images={name:Image.open(output/f'{name}-frame.png').convert('RGBA').tobytes() for name in ['01-top','02-depth','03-battle-before','04-battle-effect','05-battle-impact','06-battle-hp','07-battle-settle']}
   assert images['01-top'] != images['02-depth'], 'ground projection must visibly change'
   assert len({images[n] for n in ['03-battle-before','04-battle-effect','05-battle-impact','06-battle-hp']}) == 4, 'attack phases must visibly differ'
   # HP bars/menu are Flutter overlays; settled canonical actors may match pre-attack.
   before_ui=Image.open(output/'03-battle-before-ui.png').convert('RGBA').tobytes()
   settled_ui=Image.open(output/'07-battle-settle-ui.png').convert('RGBA').tobytes()
   assert before_ui != settled_ui, 'HP/menu feedback must reflect the resolved round'
   stories=[json.loads((output/f'17-bubble-{i}-status.json').read_text(encoding='utf-8'))['waiting'] for i in range(1,4)]
   assert all(s['kind']=='bubble' and s['actor']=='minimal_actor' and 'screen_anchor' in s for s in stories)
   assert len({s['id'] for s in stories})==3
   for mode in ['top','depth','alternate']:
    snap=json.loads((output/f'16-roof-{mode}.json').read_text(encoding='utf-8'))
    actor=next(e for e in snap['entities'] if e['id']=='player')
    assert actor['position']=={'x':336*60,'y':272*60} and actor['flatland']['plane']==0
   for name in ['13-top-shadow','14-occlusion-behind','15-occlusion-front','11-guard','12-heal','18-restored']:
    assert (output/f'{name}-ui.png').is_file()
   restored=json.loads((output/'18-restored-status.json').read_text(encoding='utf-8'))
   assert restored['systems']['view']=='depth' and restored['systems']['combatants']['seedling']['remaining_pp']['scratch']<25

  finally:
   if process.poll() is None:process.kill();process.wait()
 print(json.dumps({'ok':True,'platform':'windows','utf8_pipeline':'authoring-validation-package-import-launch','physical_android_acceptance':False,'evidence':str(EVIDENCE)},ensure_ascii=False))
if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--prepare',action='store_true');parser.add_argument('--launch',action='store_true');args=parser.parse_args()
 if args.prepare:prepare()
 if args.launch:launch()
