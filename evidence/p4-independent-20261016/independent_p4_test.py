#!/usr/bin/env python3
"""Independent, source-free GE4G P4 SDK verification, authored outside engine repo.

Only standard-library Python, public .h/.md, sample config, and shipped ELF are used.
Tests do not import SDK's consumer.py or inspect GE4G source.
"""
from __future__ import annotations
import copy
import os
import ctypes
import hashlib
import json
import pathlib
import random
import subprocess
import sys
import threading
import time
import zipfile

ROOT=pathlib.Path(__file__).resolve().parent
SDK=pathlib.Path(os.environ.get('GE4G_P4_SDK',str(ROOT/'sdk'))).resolve()
LIB=SDK/'libge4g_pentomino_native.so'
REPORT=ROOT/'independent_results.json'
TESTS=[]

def check(name, fn):
    t=time.monotonic()
    try:
        fn()
    except Exception as e:
        TESTS.append({'name':name,'status':'FAIL','seconds':round(time.monotonic()-t,4),'error':repr(e)})
        print('FAIL',name,repr(e),flush=True)
    else:
        TESTS.append({'name':name,'status':'PASS','seconds':round(time.monotonic()-t,4)})
        print('PASS',name,flush=True)

def integrity():
    outer=pathlib.Path(os.environ.get('GE4G_P4_OUTER',str(ROOT.parent/'p4-native-public-latest.zip')))
    assert hashlib.sha256(outer.read_bytes()).hexdigest()=='af98895c3baf92fb502b13951b056d85ad558f5eb4c915b6d5177c41cbf8600c'
    expected=(pathlib.Path(os.environ.get('GE4G_P4_ARCHIVE',str(ROOT/'p4-public-sdk.zip'))).with_suffix('.zip.sha256')).read_text().split()[0]
    assert hashlib.sha256(pathlib.Path(os.environ.get('GE4G_P4_ARCHIVE',str(ROOT/'p4-public-sdk.zip'))).read_bytes()).hexdigest()==expected
    manifest=json.loads((SDK/'MANIFEST.json').read_text())
    for item in manifest['files']:
        path=SDK/item['path']
        assert path.is_file(), path
        raw=path.read_bytes()
        assert len(raw)==item['bytes'] and hashlib.sha256(raw).hexdigest()==item['sha256'],item
    assert hashlib.sha256(LIB.read_bytes()).hexdigest()==next(e['sha256'] for e in manifest['files'] if e['path']=='libge4g_pentomino_native.so')
    assert b'ge4g_p4_v1_request' in (SDK/'ge4g_p4_v1.h').read_bytes()

class API:
    def __init__(self):
        self.lib=ctypes.CDLL(str(LIB))
        self.lib.ge4g_p4_v1_abi_version.argtypes=[]
        self.lib.ge4g_p4_v1_abi_version.restype=ctypes.c_uint32
        self.lib.ge4g_p4_v1_request.argtypes=[ctypes.c_void_p,ctypes.c_size_t]
        self.lib.ge4g_p4_v1_request.restype=ctypes.c_void_p
        self.lib.ge4g_p4_v1_string_free.argtypes=[ctypes.c_void_p]
        self.lib.ge4g_p4_v1_string_free.restype=None
        assert self.lib.ge4g_p4_v1_abi_version()==1

    def raw(self,bs:bytes|None):
        if bs is None:
            address=None;size=0
        else:
            address=ctypes.create_string_buffer(bs);size=len(bs)
        ptr=self.lib.ge4g_p4_v1_request(address,size)
        assert ptr,'null response instead of JSON'
        try:
            obj=json.loads(ctypes.string_at(ptr))
        finally:
            self.lib.ge4g_p4_v1_string_free(ptr)
        assert obj['version']==1 and isinstance(obj['ok'],bool)
        assert (obj['error'] is None) == obj['ok']
        return obj
    def req(self,op,**opts):
        return self.raw(json.dumps({'version':1,'command':{'op':op,**opts}},ensure_ascii=False,allow_nan=False,separators=(',',':')).encode())
    def good(self,op,**opts):
        r=self.req(op,**opts);assert r['ok'],(op,r['error']);return r['result']
    def bad(self,op,codes=None,**opts):
        r=self.req(op,**opts);assert not r['ok'],(op,r)
        if codes is not None:
            assert r['error']['code'] in (codes if isinstance(codes,tuple) else [codes]),(op,r)
        return r['error']['code']

api=API()
config=json.loads((SDK/'game.json').read_text())

def new(seed=42,mode='linked',cfg=None):
    return api.good('create',config=cfg or config,seed=seed,mode=mode)['session']

def observe(h):return api.good('observe',session=h)
def save(h):return api.good('save',session=h)
def frame(h):return api.good('frame',session=h)
def step(h,n,controls=None):
    if controls is None: controls=[config['attack_control']]
    return api.good('step',session=h,input={'version':1,'target_step':n,'controls':controls})
def close(h):return api.good('close',session=h)

def assert_frame(f):
    assert f['version']==1
    assert len(f['rgba'])==f['width']*f['height']*4
    assert hashlib.sha256(bytes(f['rgba'])).hexdigest()==f['sha256']
    assert all(isinstance(x,int) and 0<=x<=255 for x in f['rgba'])

def snapshot(h,with_frame=True):
    return (save(h),observe(h),frame(h) if with_frame else None)
def same_snapshot(h,baseline,with_frame=True):
    assert snapshot(h,with_frame)==baseline

def c_compile():
    exe=ROOT/'independent-c-smoke'
    cmd=['cc','-std=c11','-Wall','-Wextra','-Werror',str(SDK/'smoke.c'),'-I'+str(SDK),'-L'+str(SDK),'-lge4g_pentomino_native','-Wl,-rpath,'+str(SDK),'-o',str(exe)]
    subprocess.run(cmd,check=True,capture_output=True)
    run=subprocess.run([str(exe)],check=True,capture_output=True,text=True)
    assert run.returncode==0

def ffi_basic():
    assert api.bad('nonesuch')=='invalid_input'
    assert not api.raw(b'bad json')['ok']
    assert not api.raw(b'')['ok']
    assert not api.raw(None)['ok']
    for x in (b'{}',b'[]',b'null',b'"x"',b'{"version":2,"command":{"op":"foo"}}',b'\xff\xfe'):
        assert not api.raw(x)['ok'],x
    h=new()
    try:
        obs=observe(h)
        assert obs['logical_step']==obs['core_tick']==obs['presentation_tick']==0
        assert obs['battle']=={'hp':[12,11],'turn':0,'winner':-1,'attack_count':0}
        assert [r['local'] for r in obs['records']] == sorted([r['local'] for r in obs['records']])
        assert [r['owner'] for r in obs['histories']] == sorted([r['owner'] for r in obs['histories']])
        f=frame(h);assert_frame(f)
        assert f['sha256']=='b21c52050cf1052af6cbf55fc46f367d1828fcd42ebdf995f17b6e92fd28ebb4'
        info=api.good('describe',session=h)
        assert info['mode']=='linked' and info['content_binding'].startswith('p4:')
        assert obs['scenes'] and len(obs['entities'])==2 and len(obs['camera_frames'][0]['items'])==2
        assert save(h)['core_save2'].startswith('{') and save(h)['presentation_save1'].startswith('{')
    finally: close(h)

def turn_combat_independent():
    h=new(seed=5)
    try:
        # Five alternating attacks for the declared fixed damage rule.
        expect=[([12,11],0,-1,0),([12,7],1,-1,1),([9,7],0,-1,2),([9,3],1,-1,3),([6,3],0,-1,4),([6,0],0,0,5)]
        for n,(hp,turn,winner,count) in enumerate(expect):
            v=observe(h) if n==0 else step(h,n)
            assert v['battle']=={'hp':hp,'turn':turn,'winner':winner,'attack_count':count},(n,v['battle'])
            assert v['logical_step']==n and v['core_tick']==n*2
            assert v['presentation_tick']==n
        final=frame(h)
        assert_frame(final)
        assert final['sha256']=='d7b751331b32525afc089b64be4cd870458daa4ff5cb7a3d681dc816d9f56558',final['sha256']  # unmodified camera golden
        api.bad('step','invalid_input',session=h,input={'version':1,'target_step':6,'controls':['attack']})
        assert step(h,6,[])['battle']['attack_count']==5
        assert observe(h)['core_tick']==12
    finally:close(h)

def no_op_and_replay():
    h1=new(seed=0);h2=new(seed=0);h3=new(seed=999)
    try:
        assert save(h1)==save(h2)
        for n,control in enumerate(([],['attack'],[],['attack'],['attack'],[],['attack'],['attack']),1):
            a=step(h1,n,control);b=step(h2,n,control);c=step(h3,n,control)
            assert a==b
            assert a['battle']==c['battle']
            assert save(h1)==save(h2)
            assert frame(h1)['sha256']==frame(h2)['sha256']
            assert a['core_tick']==2*n and a['presentation_tick']==n
    finally:
        for h in [h1,h2,h3]:close(h)

def invalid_input_atomicity():
    h=new()
    try:
        step(h,1)
        base=snapshot(h)
        invalid=[
          {'version':1,'target_step':1,'controls':['attack']},
          {'version':1,'target_step':3,'controls':['attack']},
          {'version':1,'target_step':2,'controls':['attack','attack']},
          {'version':1,'target_step':2,'controls':['unknown']},
          {'version':1,'target_step':2,'controls':['p4.state.hp0']},
          {'version':1,'target_step':2,'controls':[5]},
          {'version':1,'target_step':2,'controls':'attack'},
          {'version':1,'target_step':2,'controls':None},
          {'version':1,'target_step':2,'controls':[],'extra':1},
          {'version':2,'target_step':2,'controls':['attack']},
          {'version':1,'target_step':-1,'controls':['attack']},
          {'version':1,'target_step':2**64,'controls':['attack']},
          {'version':1,'target_step':2,'controls':['attack'], 'kind':'action'},
        ]
        for x in invalid:
            api.bad('step',session=h,input=x)
            same_snapshot(h,base)
        step(h,2)
        assert observe(h)['battle']['attack_count']==2
    finally:close(h)

def restore_atomicity():
    h=new()
    try:
        step(h,1);step(h,2)
        base=snapshot(h)
        original=save(h)
        modifications=[
            ('top version', lambda d:d.update(version=10)),
            ('seed', lambda d:d.update(seed=17)),
            ('binding',lambda d:d.update(content_binding='p4:bogus')),
            ('step',lambda d:d.update(logical_step=500)),
            ('presentation tick',lambda d:d.update(presentation_tick=91)),
            ('state-json',lambda d:d.update(core_save2='{}')),
            ('view-json',lambda d:d.update(presentation_save1='{}')),
            ('reversed tick',lambda d:d.update(logical_step=1)),
            ('extra top field',lambda d:d.update(unexpected=True)),
            ('invalid view',lambda d:d.update(view_family='Nope')),
        ]
        for label, mutate in modifications:
            bad=copy.deepcopy(original);mutate(bad)
            api.bad('restore',session=h,save=bad)
            same_snapshot(h,base)
        # Canonical (parseable) but inconsistent core state should reject.
        for label,mut in [
            ('core tick',lambda d:d.update(tick=999)),
            ('core seed',lambda d:d.update(seed=88)),
            ('bad core content binding',lambda d:d.update(content_binding='p4:changed')),
            ('core extra',lambda d:d.update(unknown=1)),
        ]:
            bad=copy.deepcopy(original);core=json.loads(bad['core_save2']);mut(core)
            bad['core_save2']=json.dumps(core,separators=(',',':'),sort_keys=True)
            api.bad('restore',session=h,save=bad)
            same_snapshot(h,base)
        # success on valid saved state after progressing
        step(h,3); advanced=save(h)
        api.good('restore',session=h,save=original)
        step(h,3)
        assert save(h)==advanced
    finally:close(h)

def detach_reinstall():
    h=new(seed=77)
    try:
        step(h,1);step(h,2)
        before=observe(h)
        api.good('gameplay',session=h,enabled=False)
        dropped=observe(h)
        assert not dropped['gameplay_attached']
        assert before['battle']==dropped['battle']
        assert before['entities']==dropped['entities']
        assert before['scenes']==dropped['scenes']
        assert before['records']==dropped['records']
        assert [p['descriptor']['id'] for p in api.good('describe',session=h)['core']['plugins']]==['p4.state']
        absent=snapshot(h)
        api.bad('step','gameplay_detached',session=h,input={'version':1,'target_step':3,'controls':['attack']})
        same_snapshot(h,absent)
        api.bad('gameplay',session=h,enabled=False)
        same_snapshot(h,absent)
        step(h,3,[]);assert observe(h)['battle']==before['battle']
        api.good('gameplay',session=h,enabled=True)
        api.bad('gameplay',session=h,enabled=True)
        assert observe(h)['battle']==before['battle']
        assert [p['descriptor']['id'] for p in api.good('describe',session=h)['core']['plugins']]==['p4.state','p4.turn_combat']
        assert step(h,4)['battle']['attack_count']==3
    finally:close(h)

def camera_and_view_atomicity():
    h=new()
    try:
        core=save(h)['core_save2'];f0=frame(h)['sha256']
        valid={'version':1,'zoom':1.4,'tilt':1.,'shear':0.,'origin':[0.,0.],'transition_ticks':0}
        api.good('camera',session=h,request=valid)
        assert save(h)['core_save2']==core and frame(h)['sha256']!=f0
        api.good('view',session=h,family='TopDown')
        assert save(h)['core_save2']==core
        base=snapshot(h)
        invalid=[
          {'version':1,'zoom':0.,'tilt':1.,'shear':0.,'origin':[0.,0.],'transition_ticks':0},
          {'version':1,'zoom':5.,'tilt':1.,'shear':0.,'origin':[0.,0.],'transition_ticks':0},
          {'version':1,'zoom':1.,'tilt':0.0,'shear':0.,'origin':[0.,0.],'transition_ticks':0},
          {'version':1,'zoom':1.,'tilt':1.,'shear':0.8,'origin':[0.,0.],'transition_ticks':0},
          {'version':1,'zoom':1.,'tilt':1.,'shear':0.,'origin':[0.,0.],'transition_ticks':121},
          {'version':1,'zoom':1.,'tilt':1.,'shear':0.,'origin':[10000.,0.],'transition_ticks':0},
          {'version':1,'zoom':1.,'tilt':1.,'shear':0.,'origin':[0.,0.],'transition_ticks':0,'other':5},
        ]
        for x in invalid:
            api.bad('camera',session=h,request=x)
            same_snapshot(h,base)
        api.bad('view',session=h,family='Full3D')
        same_snapshot(h,base)
        api.good('camera',session=h,request={'version':1,'zoom':1.1,'tilt':1.,'shear':0.,'origin':[0.,0.],'transition_ticks':2})
        assert save(h)['core_save2']==core
        step(h,1,[]); assert_frame(frame(h))
        step(h,2,[]); assert_frame(frame(h))
    finally:close(h)

def lifecycle_and_limit():
    handles=[]
    try:
        handles=[new(seed=i) for i in range(16)]
        assert len(set(handles))==16
        err=api.bad('create',config=config,seed=17,mode='linked')
        assert err in ('budget_exceeded','overflow'),err
        for h in handles:
            assert observe(h)['core_tick']==0
        dead=handles.pop(7);close(dead)
        api.bad('observe','invalid_session',session=dead)
        api.bad('close','invalid_session',session=dead)
        reuse=new(seed=9);handles.append(reuse)
        assert reuse>max(handles[:-1]) and reuse!=dead
        other=observe(handles[0]);assert other['core_tick']==0
    finally:
        for h in handles:close(h)

def cross_thread():
    h=new()
    errors=[]
    def worker():
        for op in ('observe','save','close'):
            try:
                code=api.bad(op,'invalid_session',session=h)
                errors.append(code)
            except Exception as e:errors.append(repr(e))
        other=new(seed=23)
        assert observe(other)['battle']['attack_count']==0
        # Abandon to verify implicit thread table cleanup.
    t=threading.Thread(target=worker);t.start();t.join()
    assert errors==['invalid_session']*3,errors
    assert observe(h)['logical_step']==0
    close(h)

def headless_restoration_and_binding():
    linked=new(seed=17);headless=new(seed=17,mode='headless')
    changed=copy.deepcopy(config);changed['fighters'][0]['damage']=7
    other=new(seed=17,cfg=changed)
    try:
        api.bad('frame','unsupported',session=headless)
        assert save(linked)['core_save2']==save(headless)['core_save2']
        assert api.good('describe',session=linked)['content_binding']!=api.good('describe',session=other)['content_binding']
        for n in (1,2):
            assert step(linked,n)==step(headless,n)
        checkpoint=save(linked)
        headless_state=save(headless)
        # mode-independent restoration
        step(headless,3)
        api.good('restore',session=headless,save=checkpoint)
        assert save(headless)==headless_state
        # seed/config mismatch must reject atomically
        otherbefore=save(other)
        api.bad('restore',session=other,save=checkpoint)
        assert save(other)==otherbefore
    finally:
        for h in (linked,headless,other):close(h)

def randomized_short_games():
    rng=random.Random(20261016)
    for i in range(24):
        cfg=copy.deepcopy(config)
        hp0=rng.randint(1,35);hp1=rng.randint(1,35)
        damage0=rng.randint(1,15);damage1=rng.randint(1,15)
        cfg['fighters'][0]['hp']=hp0;cfg['fighters'][1]['hp']=hp1
        cfg['fighters'][0]['damage']=damage0;cfg['fighters'][1]['damage']=damage1
        h=api.good('create',config=cfg,seed=i,mode='headless')['session']
        try:
            hp=[hp0,hp1];turn=0;winner=-1;count=0
            for n in range(1,25):
                if winner!=-1: break
                target=1-turn
                hp[target]=max(0,hp[target]-(damage0 if turn==0 else damage1))
                count+=1
                if hp[target]==0: winner=turn
                else:turn=target
                result=api.good('step',session=h,input={'version':1,'target_step':n,'controls':['attack']})['battle']
                assert result=={'hp':hp,'turn':turn,'winner':winner,'attack_count':count},(i,n,result,hp,turn,winner,count)
        finally:close(h)

def malformed_fuzz():
    rng=random.Random(41)
    h=new()
    try:
        base=save(h)
        for i in range(80):
            random_bytes=bytes(rng.getrandbits(8) for _ in range(rng.randrange(0,220)))
            assert not api.raw(random_bytes)['ok']
        assert save(h)==base
    finally:close(h)

def main():
    plan=[('01 integrity digest+full manifest',integrity),('02 independent C11 linking',c_compile),('03 ABI envelope/discovery/frame',ffi_basic),('04 alternating battle model+golden RGBA',turn_combat_independent),('05 deterministic multi-session replay',no_op_and_replay),('06 invalid-input transaction rollback',invalid_input_atomicity),('07 corrupt save transaction rollback',restore_atomicity),('08 true gameplay detach/reinstall',detach_reinstall),('09 camera/view+transition rollback',camera_and_view_atomicity),('10 session capacity/lifecycle',lifecycle_and_limit),('11 thread confinement',cross_thread),('12 cross-mode restore+content binding',headless_restoration_and_binding),('13 independent randomized battle model 24 configs',randomized_short_games),('14 malformed input fuzz 80 cases',malformed_fuzz)]
    for name,fn in plan:check(name,fn)
    report={'test_count':len(TESTS),'pass_count':sum(x['status']=='PASS' for x in TESTS),'fail_count':sum(x['status']=='FAIL' for x in TESTS),'lib_sha256':hashlib.sha256(LIB.read_bytes()).hexdigest(),'artifact_sha256':hashlib.sha256(pathlib.Path(os.environ.get('GE4G_P4_OUTER',str(ROOT.parent/'p4-native-public-latest.zip'))).read_bytes()).hexdigest(),'candidate':'GE4G P4 public SDK produced from GitHub Actions run 38414825262 (PR checkout merge commit 62b7118a477d40341bb8a64221d0adca70dddbce; PR source SHA 72d49a63f730e2a3ee2032e83948894e4fd480ba)','independent_source_free':True,'environment':'Linux x86_64 Python ctypes cc; test process imports no engine source. QA agent has source/toolchain access; not an enforced blind environment.','tests':TESTS}
    REPORT.write_text(json.dumps(report,indent=2,ensure_ascii=False)+'\n')
    print('RESULT',report['pass_count'],'PASS',report['fail_count'],'FAIL',flush=True)
    return 0 if not report['fail_count'] else 1

if __name__=='__main__': sys.exit(main())
