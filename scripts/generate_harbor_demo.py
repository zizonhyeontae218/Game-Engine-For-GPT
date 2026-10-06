#!/usr/bin/env python3
"""Generate original CC0 Harbor Workshop content; no third-party art or map reference."""
from pathlib import Path
import hashlib,json,math,wave,struct
from PIL import Image,ImageDraw
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'examples/flatland_harbor'
def write(name,data):
 p=OUT/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def actor(id,x,y,tag=None,body='pass',size=(16,16),**fields):
 e={'id':id,'position':[x,y],'size':list(size),'flatland':{'body':body,**fields}}
 if tag:e['tags']=[tag]
 return e
def main():
 (OUT/'assets').mkdir(parents=True,exist_ok=True);
 for unused in ['roof_only.png','facade_only.png']:(OUT/'assets'/unused).unlink(missing_ok=True)
 (OUT/'audio').mkdir(exist_ok=True)
 for direction in ['down','up','left','right']:
  for phase in range(3):
   im=Image.new('RGBA',(16,24));d=ImageDraw.Draw(im);d.rectangle((4,3,11,11),fill='#efc889');d.rectangle((3,11,12,18),fill='#397ba5');d.rectangle((4,18,6,22-(phase==1)),fill='#28394b');d.rectangle((9,18,11,22-(phase==2)),fill='#28394b');d.rectangle((3,1,12,5),fill='#243647');
   if direction!='up':d.point((5 if direction=='left' else 10,7),fill='#14252d')
   im.save(OUT/f'assets/walker_{direction}_{phase}.png')
 for name,color in [('roof_only','#698cbb'),('facade_only','#deb882')]:
  im=Image.new('RGBA',(32,24),color);d=ImageDraw.Draw(im)
  for x in range(0,32,8):d.line((x,0,x,24),fill='#4c6176' if name=='roof_only' else '#987659')
  im.save(OUT/('assets/지붕.png' if name=='roof_only' else 'assets/벽면.png'))
 for name,hz in [('music',220),('hit',330),('door',440),('quest',660)]:
  with wave.open(str(OUT/f'audio/{name}.wav'),'wb') as f:
   f.setparams((1,2,22050,0,'NONE','not compressed'));duration=1 if name=='music' else .12
   f.writeframes(b''.join(struct.pack('<h',int(1800*math.sin(2*math.pi*hz*n/22050)*(1-n/(22050*duration)))) for n in range(int(22050*duration))))
 frames=lambda direction:[f'assets/walker_{direction}_{n}.png' for n in [0,1,0,2]]
 player=actor('player',256,224,step_walk=True,hp=30,team='hero',attack='tap',visual_size=[16,24],anchor=[0,-8],animation={'frames':frames('down'),'ticks':7,'directions':{d:frames(d) for d in ['up','left','right']}});player['sprite']={'texture':frames('down')[0],'layer':1};player['player']={'speed':68}
 view=[{'op':'choice','text':'시점은 화면 표현만 바꿉니다. 위치·높이·충돌은 그대로 유지됩니다.','options':[{'id':id,'text':label,'next':1,'actions':[{'op':'view','mode':id}]} for id,label in [('top','위에서 / TOP'),('depth','경사 / DEPTH'),('alternate','반대 경사 / ALTERNATE')]]},{'op':'return'}]
 roster={'seedling':{'name':'연두','max_hp':38,'attack':12,'defense':4,'speed':8,'moves':[{'id':'scratch','name':'할퀴기','power':100,'pp':25,'fx':'slash'},{'id':'pulse','name':'빛 씨앗','power':130,'pp':12,'fx':'projectile'}]}}
 gameplay={'seed':19,'combatants':roster,'views':{'top':{'label':'TOP'},'depth':{'label':'DEPTH','zoom':115,'tilt':68,'shear':12},'alternate':{'label':'ALTERNATE','zoom':115,'tilt':72,'shear':-12}},'default_view':'top','items':{'potion':{'name':'회복약','stack':99,'use_actions':[{'op':'heal','entity':'$player','amount':16}]},'badge':{'name':'공방 수료증','stack':1}},'quests':{'workshop':{'name':'바람항 공방 실습','objectives':{'battle':1,'crate':1,'combat':1},'rewards':[{'op':'give','item':'badge','count':1},{'op':'set','key':'harbor.completed','value':True}]}},'attacks':{'tap':{'damage':4,'range':28,'startup':1,'active':1,'recovery':8},'bolt':{'damage':3,'range':8,'projectile_speed':160,'lifetime':80}},'events':{'view':view,'intro':[{'op':'say','text':'바람항 공방 / 0.2\nZ: 대화·입장 / X: 시점 / C: 걷기·달리기\n건물·인물은 경사 시점에서도 서 있습니다. 연두의 HP와 PP는 전투 후에도 유지됩니다. 진료소에서만 명시적으로 회복합니다.'},{'op':'return'}],'battle':[{'op':'battle','stage':{'hold_result':True},'fighters':[{'id':'hero','combatant':'seedling'},{'id':'rival','name':'연습 구름','hp':34,'attack':8,'defense':2,'speed':4,'enemy':True}],'victory':[{'op':'give','item':'potion','count':1}]},{'op':'return'}],'tour':[{'op':'camera','at':[128,64]},{'op':'wait','ticks':12},{'op':'say_bubble','actor':'minimal_actor','speaker':'안내인','text':'여기가 바람항 공방이야. 집은 발밑의 바닥을 차지하고, 지붕은 그림일 뿐이야.'},{'op':'say_bubble','actor':'minimal_actor','speaker':'안내인','text':'시점을 돌려도 길과 벽은 그대로야. 걷기와 달리기를 바꿔서 둘러봐.'},{'op':'say_bubble','actor':'minimal_actor','speaker':'안내인','text':'연습 전투의 HP와 PP도 그대로 남아. 지친 연두는 진료소에서 회복시켜 줘.'},{'op':'return'}]},'lua_events':['action.pace','tick']}
 rules=[{'id':'view','on':'action.view','actions':[{'op':'event_scene','event':'view'}]},{'id':'init','on':'start','once':True,'actions':[{'op':'quest','quest':'workshop','status':'active'},{'op':'give','item':'potion','count':3},{'op':'music','cue':'music','volume':25},{'op':'event_scene','event':'intro'}]},{'id':'rival','on':'interact','target_tag':'rival','actions':[{'op':'event_scene','event':'battle'}]},{'id':'clinic','on':'interact','target_tag':'heal','actions':[{'op':'combatant_reset','combatant':'seedling'},{'op':'say','text':'연두의 HP와 PP를 회복했습니다.'}]},{'id':'tour','on':'interact','target_tag':'tour','actions':[{'op':'event_scene','event':'tour'}]},{'id':'attack','on':'action.attack','actions':[{'op':'attack','entity':'$player','attack':'bolt'}]},{'id':'battle_progress','on':'battle_win','once':True,'when':{'op':'quest_is','quest':'workshop','status':'active'},'actions':[{'op':'objective','quest':'workshop','objective':'battle'}]},{'id':'combat_progress','on':'death','target_tag':'dummy','once':True,'actions':[{'op':'objective','quest':'workshop','objective':'combat'}]}]
 rules.append({'id':'restore_pace','on':'start','when':{'op':'state','key':'harbor.running','eq':True},'actions':[{'op':'pace','entity':'$player','speed':112}]})
 rules.append({'id':'finish','on':'tick','once':True,'when':{'op':'all','conditions':[{'op':'quest_is','quest':'workshop','status':'active'},{'op':'objectives_complete','quest':'workshop'}]},'actions':[{'op':'quest','quest':'workshop','status':'completed'},{'op':'sound','cue':'quest'}]})
 # Independent plaza: four diagonal workshop islands around a central open square.
 buildings=[]
 for id,x,y,building in [('clinic',80,80,{'footprint':[64,32],'material':'plaster','roof':'flat'}),('library',336,96,{'footprint':[64,32],'material':'wood','roof':'gable','roof_surface':'assets/지붕.png'}),('home',80,288,{'footprint':[64,32],'material':'brick','roof':'shed','facade_surface':'assets/벽면.png'}),('workshop',336,288,{})]:
  house=actor(id,x,y,body='fixed',size=(64,32),building=building)
  if id=='workshop':house['flatland'].pop('body')  # deliberately omit solidity/size defaults
  buildings.append(house)
  door=actor(id+'_door',x+16,y+48,'heal' if id=='clinic' else None);door['interaction']={'range':22,'dialogue':'진료소' if id=='clinic' else '공방 입구'}
  if id!='clinic':door['interaction']['transition']={'scene':'inside_'+id,'spawn':'entry'}
  buildings.append(door)
 minimal=actor('minimal_actor',240,160,'rival',body='fixed',visual_size=[24,24]);minimal['interaction']={'range':28,'dialogue':'기본 연출 전투를 시작합니다.'}
 dummy=actor('dummy',288,240,'dummy',body='fixed',hp=12,team='enemy',health_bar=True)
 crate=actor('crate',192,272,body='push');crate['sprite']={'color':[172,115,58,255]}
 sign=actor('tour_sign',256,80,'tour',body='fixed');sign['interaction']={'range':28,'dialogue':'항구 전경'}
 entities=[player,*buildings,minimal,dummy,crate,sign,actor('pass_marker',208,272,body='pass')]
 rows=['#'*32]+['#'+'.'*30+'#' for _ in range(24)]+['#'*32]
 for y in range(18,25):rows[y]=rows[y][:1]+'~'*3+rows[y][4:]
 tiles={'#':{'color':[38,69,66,255],'solid':True},'.':{'color':[118,157,109,255]},'~':{'color':[65,122,170,255],'solid':True},'+':{'color':[214,200,166,255]}}
 for y in range(2,24):rows[y]=rows[y][:14]+'++++'+rows[y][18:]
 for y in range(12,16):rows[y]=rows[y][:4]+'+'*24+rows[y][28:]
 common={'schema_version':2,'background':[35,54,56,255],'script':'data/공방.lua','sounds':{s:f'audio/{s}.wav' for s in ['music','hit','door','quest']},'gameplay':gameplay,'rules':rules}
 import copy
 town=copy.deepcopy(common);town.update(id='항구',spawns={'entry':[256,224],**{id+'_exit':[x+16,y+64] for id,x,y in [('library',336,96),('home',80,288),('workshop',336,288)]}},entities=entities,map={'cell':16,'rows':rows,'tiles':tiles});town['gameplay']['camera']={'target':'player','bounds':[0,0,192,176]};write('scenes/항구.json5',town)
 scenes={'항구':'scenes/항구.json5'}
 for id in ['library','home','workshop']:
  inside=copy.deepcopy(common);inside['rules']=[r for r in rules if r['id']!='init'];inside['gameplay'].pop('camera',None);inside['gameplay']['events'].pop('tour',None);inside['rules']=[r for r in inside['rules'] if r['id']!='tour']
  e=actor('exit',144,176);e['trigger']={'scene':'항구','spawn':id+'_exit'}
  p=copy.deepcopy(player);p['position']=[144,160]
  inside.update(id='inside_'+id,spawns={'entry':[144,160]},entities=[p,e],map={'cell':16,'rows':['#'*20]+['#'+'.'*18+'#' for _ in range(13)]+['#'*20],'tiles':{'#':tiles['#'],'.':{'color':[199,181,146,255]}}})
  write(f'scenes/{id}.json5',inside);scenes['inside_'+id]=f'scenes/{id}.json5'
 (OUT/'data').mkdir(exist_ok=True);(OUT/'data/harbor.lua').unlink(missing_ok=True);(OUT/'data/공방.lua').write_text('''return function(ctx)
 if ctx.event == "action.pace" then
  local run = not ctx.state["harbor.running"]
  return {{op="set",key="harbor.running",value=run},{op="pace",entity="$player",speed=run and 112 or 68}}
 end
 if ctx.event == "tick" and ctx.scene == "항구" and not ctx.state["harbor.crate"] and entity("crate").position[1] >= 208 then
  return {{op="set",key="harbor.crate",value=true},{op="objective",quest="workshop",objective="crate"}}
 end
 return {}
end
''',encoding='utf-8')
 manifest='schema_version=2\nname="FlatLand 0.2 / 바람항 공방"\nstart_scene="항구"\nfeatures=["combat","inventory","quests","event_scenes","turn_battle","planes","music","lua_rng","step_walk","battle_stage","view_projection","billboard_projection","persistent_combatants","battle_fx","building_presentation","entity_defaults","cutscene_bubbles","solid_buildings","contact_ordering"]\n[window]\nwidth=320\nheight=240\n[scenes]\n'+''.join(f'"{id}"="{file}"\n' for id,file in scenes.items())+''.join(f'[state."harbor.{id}"]\nkind="bool"\ndefault=false\n' for id in ['running','completed','crate'])
 (OUT/'ge4g.toml').write_text(manifest,encoding='utf-8')
 controls=ROOT/'examples/flatland_nuvema/controls'
 for f in ['layouts.json','bindings.json']:
  data=json.loads((controls/f).read_text());
  if f=='bindings.json':
   data['game_id']='demo.flatland.harbor'
   for profile in data['profiles'].values():profile['buttons']['space']=['attack'];profile['keys']['Space']=['attack']
  else:
   for profile in data['profiles']:
    mirror=profile['id']=='left_handed'
    for button in profile['buttons']:
     x=.72 if button['id']=='z' else .9
     button.update(x=1-x if mirror else x,y=.9 if button['id']=='c' else .68,width=.16,height=.15)
    profile['buttons'].append({'id':'space','label':'발사','x':.28 if mirror else .72,'y':.9,'width':.16,'height':.15})
  write('controls/'+f,data)
 write('assets/설명.json',{'title':'바람항 공방','scene':'항구','encoding':'UTF-8'})
 assets=[]
 for p in sorted(list((OUT/'assets').glob('*'))+list((OUT/'audio').glob('*'))):
  assets.append({'file':p.relative_to(OUT).as_posix(),'author':'GE4G project','source_url':'https://github.com/zizonhyeontae218/Game-Engine-For-GPT/blob/main/scripts/generate_harbor_demo.py','license':'CC0-1.0','license_version':'1.0','modification_allowed':True,'attribution_required':False,'modified':False,'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'provenance':'Original procedural GE4G work, generated without third-party assets or map references.'})
 write('asset-licenses.json',{'schema_version':1,'assets':assets})
 commands=[{'op':'choice','tick':0,'id':'continue'}]
 def do(tick,actions):commands.append({'op':'do','tick':tick,'actions':actions})
 do(1,[{'op':'view','mode':'depth'}]);do(2,[{'op':'goto','scene':'inside_home','spawn':'entry'}]);do(3,[{'op':'goto','scene':'항구','spawn':'entry'}]);do(4,[{'op':'event_scene','event':'battle'}])
 for tick,move in [(5,'scratch'),(114,'pulse'),(223,'scratch'),(332,'scratch')]:
  # Last turn is deliberately generous: first three attacks win with this seed.
  if tick<332:commands.append({'op':'choice','tick':tick,'id':f'move:{move}:rival'})
 commands.append({'op':'choice','tick':332,'id':'battle_continue'})
 do(333,[{'op':'view','mode':'alternate'}]);do(334,[{'op':'view','mode':'top'}]);do(335,[{'op':'move','entity':'player','at':[176,272]}])
 do(374,[{'op':'move','entity':'player','at':[272,240]},{'op':'face','entity':'player','vector':[1,0]},{'op':'attack','entity':'player','attack':'tap'}]);do(385,[{'op':'attack','entity':'player','attack':'tap'}]);do(396,[{'op':'attack','entity':'player','attack':'tap'}])
 write('replays/journey.json',{'schema_version':2,'ticks':420,'inputs':[{'start':336,'end':352,'actions':['right']}],'commands':commands})
 (OUT/'ge4g.toml').write_text(manifest+'\n[[tests]]\nname="harbor-final-regression"\nreplay="replays/journey.json"\ngolden_rgba_sha256="568334e7d0aaf36ebd4ed958f431dbfb1da0713e2f1ba4dc3a29f5bc084448a5"\n[[tests.assertions]]\ntick=420\nscene="항구"\nstate={"harbor.crate"=true,"harbor.completed"=true}\n',encoding='utf-8')
 (OUT/'README.md').write_text('''# 바람항 공방 / Harbor Workshop — FlatLand 0.2

Original GE4G sample, CC0-1.0 art, audio and map. No third-party branding/map reference.
Z interacts/enters; X selects persistent presentation-only view; C toggles 68/112 px/s.
The plaza intentionally contains four under-authored buildings: semantic-only clinic,
custom roof without walls, custom facade without roof, and default workshop. The NPC
and battle opponent use engine visual defaults. Space fires a sprite-free projectile.
The rival is north of the starting square. Battle offers slash and projectile effects;
HP/PP persist until explicit clinic reset. Save lives in the client menu/battle screen.
Library, home and workshop have interiors and named exits. Camera signs temporarily
show the harbor and restore the user-selected view. Push the brown crate right and
hit the purple dummy to exercise independent simulation primitives.

Every distributed asset is listed in asset-licenses.json with source/hash/license.
Reproduce with scripts/generate_harbor_demo.py (Pillow required).
''',encoding='utf-8')
if __name__=='__main__':main()
