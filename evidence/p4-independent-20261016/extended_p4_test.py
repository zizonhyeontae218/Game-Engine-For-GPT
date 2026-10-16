#!/usr/bin/env python3
"""Public C ABI stress tests. Run beside independent_p4_test.py and sdk/."""
import concurrent.futures
import copy
import hashlib
import json
import os
from pathlib import Path
import random
import statistics
import subprocess
import sys
import threading
import time
import traceback
from independent_p4_test import api, config, new, close, save, observe, step, frame, assert_frame

ROOT = Path(__file__).resolve().parent
RESULTS = []

def check(name, fn):
    start = time.monotonic()
    try:
        data = fn()
        row = dict(name=name, status='PASS', details=data)
    except Exception as exc:
        traceback.print_exc()
        row = dict(name=name, status='FAIL', error=repr(exc))
    row['seconds'] = round(time.monotonic()-start, 3)
    RESULTS.append(row)
    print(json.dumps(row), flush=True)
    (ROOT/'extended_results.json').write_text(json.dumps(RESULTS, indent=2)+'\n')

def rss():
    for line in Path('/proc/self/status').read_text().splitlines():
        if line.startswith('VmRSS:'): return int(line.split()[1])

def request_bounds():
    h = new(mode='headless')
    try:
        baseline = save(h)
        request = json.dumps({'version':1,'command':{'op':'observe','session':h}}).encode()
        limit = 24*1024*1024
        assert api.raw(request+b' '*(limit-len(request)))['ok']
        assert api.raw(request+b' '*(limit+1-len(request)))['error']['code']=='budget_exceeded'
        for raw in [b'['*256+b'0'+b']'*256, b'{"version":1,"version":1,"command":{"op":"close","session":0}}', b'{"version":1,"command":{"op":"observe","session":18446744073709551616}}', b'{"version":1,"command":{"op":"observe","session":NaN}}']:
            assert not api.raw(raw)['ok']
        assert save(h)==baseline
        return {'exact_24MiB':'accepted', '24MiB_plus_1':'budget_exceeded','deep_json':'rejected'}
    finally: close(h)

def config_bounds():
    bad = []
    for key, values in [('width',[31,513]),('height',[31,513]),('tick_hz',[0,241]),('attack_control',['','a'*65,'한글','a.b'])]:
        for value in values:
            cfg=copy.deepcopy(config);cfg[key]=value;bad.append(cfg)
    for key,values in [('hp',[0,10001]),('damage',[0,10001]),('id',['','a'*129,'a.b']),('position',[[-1,0],[10000,10000]]),('color',[[0,0,0,256]])]:
        for value in values:
            cfg=copy.deepcopy(config);cfg['fighters'][0][key]=value;bad.append(cfg)
    cfg=copy.deepcopy(config);cfg['fighters'][1]['id']=cfg['fighters'][0]['id'];bad.append(cfg)
    for cfg in bad: api.bad('create',config=cfg,seed=0,mode='headless')
    cfg=copy.deepcopy(config)
    cfg.update(width=512,height=512,tick_hz=240,attack_control='a'*64)
    for i,f in enumerate(cfg['fighters']): f.update(id=('a' if i==0 else 'b')*128,hp=10000,damage=10000)
    h=new(seed=2**64-1,cfg=cfg)
    try:
        assert_frame(frame(h))
        assert len(frame(h)['rgba'])==512*512*4
        assert api.good('step',session=h,input={'version':1,'target_step':1,'controls':[cfg['attack_control']]})['battle']['winner']==0
    finally: close(h)
    return {'invalid_configs_rejected':len(bad),'max_viewport_rgba_bytes':1048576,'u64_max_seed':'PASS'}

def save_mutations():
    h=new()
    rejected=0
    try:
        step(h,1);step(h,2)
        base=save(h);pixels=frame(h)['sha256']
        cases=[]
        for field,val in [('logical_step',2**64-1),('presentation_tick',0),('gameplay_attached',False),('core_save2',' '*8388609),('presentation_save1',' '*1048577)]:
            b=copy.deepcopy(base);b[field]=val;cases.append(b)
        for inner in ['core_save2','presentation_save1']:
            for raw in ['null','[]','{}','[[]]','{"version":999}','[ '*140+'0'+' ]'*140]:
                b=copy.deepcopy(base);b[inner]=raw;cases.append(b)
        # Re-encoding changes canonical order/spacing and must be rejected.
        for inner in ['core_save2','presentation_save1']:
            b=copy.deepcopy(base);b[inner]=json.dumps(json.loads(b[inner]),indent=1);cases.append(b)
        for b in cases:
            api.bad('restore',session=h,save=b);rejected+=1
            assert save(h)==base and frame(h)['sha256']==pixels
        return {'mutations_rejected_atomically':rejected}
    finally: close(h)

def fuzz():
    rng=random.Random(888168)
    h=new(mode='headless')
    try:
        baseline=save(h)
        for i in range(2000):
            raw=bytes(rng.getrandbits(8) for _ in range(rng.randrange(0,1024)))
            assert not api.raw(raw)['ok']
        for i in range(500):
            input={'version':1,'target_step':1,'controls':[]}
            key=rng.choice(list(input));input[key]=rng.choice([None,{},-1,2**65,True,1.5,['attack','attack']])
            api.bad('step',session=h,input=input)
        assert save(h)==baseline
        return {'random_bytes':2000,'structured_invalid_inputs':500,'seed':888168}
    finally: close(h)

def thread_stress():
    barrier=threading.Barrier(8)
    def worker(i):
        handles=[new(seed=i,mode='headless') for _ in range(16)]
        try:
            api.bad('create','budget_exceeded',config=config,seed=i,mode='headless')
            barrier.wait(timeout=60)
            for n in range(1,51):
                for h in handles[:2]: assert step(h,n,[])['core_tick']==2*n
            return handles
        finally:
            for h in handles: close(h)
    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
        handles=sum(list(pool.map(worker,range(8))),[])
    assert len(set(handles))==128
    for h in handles: api.bad('observe','invalid_session',session=h)
    return {'threads':8,'simultaneously_live_sessions':128,'steps':800,'unique_handles':128}

def process_worker():
    h=new(seed=888,mode='headless')
    for n in range(1,101): step(h,n,[])
    digest=hashlib.sha256(json.dumps(save(h),sort_keys=True).encode()).hexdigest()
    close(h)
    print(digest)

def multiprocess():
    def run(i):
        r=subprocess.run([sys.executable,__file__,'--worker'],capture_output=True,text=True,timeout=90)
        assert r.returncode==0,(r.returncode,r.stderr)
        return r.stdout.strip()
    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool: hashes=list(pool.map(run,range(4)))
    assert len(set(hashes))==1
    return {'processes':4,'steps_each':100,'equal_save_sha256':hashes[0]}

def history_rollover():
    cfg=copy.deepcopy(config)
    for f in cfg['fighters']: f.update(hp=10000,damage=1)
    h=new(mode='headless',cfg=cfg)
    try:
        for n in range(1,301):
            result=step(h,n)
            assert result['battle']=={'hp':[10000-n//2,10000-(n+1)//2],'turn':n%2,'winner':-1,'attack_count':n}
            if n in (63,64,65,127,128,129,255,256,257,300):
                s=save(h);api.good('restore',session=h,save=s)
                print('history_checkpoint',n,[(x['owner'],x['history']['emitted'],x['history']['dropped']) for x in result['histories']],flush=True)
        return {'attacks':300,'restore_checkpoints':10,'final':result['battle']}
    finally: close(h)

def soak():
    out={}
    for mode in ['headless','linked']:
        h=new(mode=mode);times=[];samples=[]
        try:
            for n in range(1,5001):
                t=time.perf_counter();result=step(h,n,[]);times.append((time.perf_counter()-t)*1000)
                assert result['core_tick']==2*n and result['battle']['attack_count']==0
                if n%1000==0:
                    s=save(h);api.good('restore',session=h,save=s)
                    samples.append({'step':n,'median_last_100_ms':statistics.median(times[-100:]),'rss_kib':rss()})
                    print('soak_checkpoint',mode,n,flush=True)
            out[mode]={'steps':5000,'p50_ms':statistics.median(times),'p95_ms':sorted(times)[4750],'samples':samples,'last_first_latency_ratio':statistics.median(times[-500:])/statistics.median(times[:500])}
        finally:close(h)
    return out

def memory_churn():
    samples=[]
    for i in range(1001):
        h=new(mode='headless');close(h)
        if i%100==0:samples.append({'cycles':i,'rss_kib':rss()})
    # Bounded empirical plateau check, not a general leak proof.
    growth=samples[-1]['rss_kib']-samples[2]['rss_kib']
    assert growth<16384, samples
    return {'cycles':1001,'samples':samples,'rss_growth_after_warmup_kib':growth,'threshold_kib':16384}

def camera_fuzz():
    rng=random.Random(1416);h=new();other=new(mode='headless')
    try:
        for i in range(128):
            req={'version':1,'zoom':rng.uniform(.25,4),'tilt':rng.uniform(.25,2),'shear':rng.uniform(-.5,.5),'origin':[rng.uniform(-1024,1024),rng.uniform(-1024,1024)],'transition_ticks':rng.choice([0,1,12,120])}
            core=save(h)['core_save2']
            api.good('camera',session=h,request=req);api.good('camera',session=other,request=req)
            family=rng.choice(['Classic2D','TopDown','Side','Vertical'])
            api.good('view',session=h,family=family);api.good('view',session=other,family=family)
            assert save(h)['core_save2']==core
            assert observe(h)==observe(other)
            checkpoint=save(h);api.good('restore',session=h,save=checkpoint)
            assert_frame(frame(h))
        return {'camera_cases':128,'view_families':4,'cross_mode':'equal observations'}
    finally:close(h);close(other)

if __name__=='__main__':
    if '--worker' in sys.argv: process_worker();sys.exit(0)
    for name,fn in [('15 request limits/deep JSON',request_bounds),('16 configuration/resource extrema',config_bounds),('17 save boundary mutation atomicity',save_mutations),('18 malformed/structured fuzz',fuzz),('19 eight-thread isolation/capacity',thread_stress),('20 four-process replay',multiprocess),('21 attack-history retention boundary',history_rollover),('22 10000-step soak/latency',soak),('23 1001-session memory churn',memory_churn),('24 randomized cameras/views',camera_fuzz)]:check(name,fn)
    sys.exit(any(r['status']=='FAIL' for r in RESULTS))
