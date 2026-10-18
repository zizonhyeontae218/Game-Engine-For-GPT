"""Remaining native gates; reuse immutable independent QA assertions.

GE4G_LIB points to newly built library. No old report is overwritten.
"""
import argparse, collections, copy, ctypes, hashlib, json, os, pathlib
import platform, statistics, sys, time, traceback

ROOT = pathlib.Path(__file__).resolve().parent
REPO = ROOT.parent.parent

def load():
    original = REPO/'evidence/p4-independent-20261016/independent_p4_test.py'
    source = original.read_text(encoding='utf-8')
    old = "LIB=SDK/'libge4g_pentomino_native.so'"
    assert source.count(old) == 1
    source = source.replace(old, "LIB=pathlib.Path(os.environ['GE4G_LIB']).resolve()")
    os.environ['GE4G_P4_SDK'] = str(REPO/'samples/pentomino_p4')
    scope = {'__file__':str(original), '__name__':'reused_prior_qa'}
    exec(compile(source, str(original), 'exec'), scope)
    return scope

def rss():
    if os.name != 'nt':
        return int(pathlib.Path('/proc/self/status').read_text().split('VmRSS:')[1].split()[0])
    class Counters(ctypes.Structure):
        _fields_ = [('cb',ctypes.c_ulong),('faults',ctypes.c_ulong)]+[(n,ctypes.c_size_t) for n in ['peak','working','pagedpeak','paged','nonpagedpeak','nonpaged','pagefile','pagefilepeak']]
    c=Counters();c.cb=ctypes.sizeof(c)
    k=ctypes.WinDLL('kernel32');k.GetCurrentProcess.restype=ctypes.c_void_p
    p=ctypes.WinDLL('psapi');p.GetProcessMemoryInfo.argtypes=[ctypes.c_void_p,ctypes.c_void_p,ctypes.c_ulong]
    assert p.GetProcessMemoryInfo(k.GetCurrentProcess(),ctypes.byref(c),c.cb)
    return c.working//1024

def smoke(q, out):
    names=['ffi_basic','turn_combat_independent','invalid_input_atomicity','restore_atomicity',
           'detach_reinstall','camera_and_view_atomicity','lifecycle_and_limit','cross_thread',
           'headless_restoration_and_binding']
    for name in names:q['check'](name,q[name])
    report={'platform':platform.platform(),'python':sys.version,'library':os.environ['GE4G_LIB'],
            'sha256':hashlib.sha256(pathlib.Path(os.environ['GE4G_LIB']).read_bytes()).hexdigest(),
            'tests':q['TESTS']}
    out.write_text(json.dumps(report,indent=2)+'\n')
    assert all(x['status']=='PASS' for x in q['TESTS'])

def endurance(q, mode, seconds, out):
    a=q['api'];cfg=copy.deepcopy(q['config']);seed=20261018
    for f in cfg['fighters']:f.update(hp=10000,damage=1)
    def create():return q['new'](seed=seed,mode=mode,cfg=cfg)
    h=create();battle={'hp':[10000,10000],'turn':0,'winner':-1,'attack_count':0}
    n=total=cycles=restores=cameras=detaches=frames=0
    history_boundary=False;longest=0;lat=collections.deque(maxlen=4096)
    start=time.monotonic();checkpoint=start;error=None
    log=out.with_suffix('.jsonl').open('w',encoding='utf-8',buffering=1)
    def emit(final=False):
        s=q['save'](h);values=sorted(lat)
        row={'mode':mode,'elapsed_seconds':time.monotonic()-start,'steps':total,'session_step':n,
             'rss_kib':rss(),'latency_window':len(values),'p50_ms':statistics.median(values) if values else 0,
             'p95_ms':values[int((len(values)-1)*.95)] if values else 0,
             'p99_ms':values[int((len(values)-1)*.99)] if values else 0,
             'errors':0 if error is None else 1,'state_sha256':hashlib.sha256(json.dumps(s,sort_keys=True).encode()).hexdigest(),
             'battle':battle,'cycles':cycles,'restores':restores,'cameras':cameras,'detaches':detaches,
             'frames_verified':frames,'history_boundary_seen':history_boundary,'longest_battle_attacks':longest,'final':final}
        log.write(json.dumps(row)+'\n');print(json.dumps(row),flush=True)
        assert row['rss_kib'] < 768*1024, 'RSS resource cap exceeded'
        return row
    try:
        emit()
        while time.monotonic()-start < seconds:
            t=time.perf_counter();n+=1;total+=1
            attack=battle['winner']==-1 and total%11!=0
            if attack:
                turn=battle['turn'];target=1-turn;battle['hp'][target]-=1;battle['attack_count']+=1
                if battle['hp'][target]==0:battle['winner']=turn
                else:battle['turn']=target
            obs=q['step'](h,n,['attack'] if attack else [])
            assert obs['battle']==battle,(obs['battle'],battle)
            assert obs['core_tick']==2*n and obs['logical_step']==n
            longest=max(longest,battle['attack_count'])
            for hist in obs['histories']:
                history_boundary |= hist['history']['dropped']>0
            if total%64==0:
                s=q['save'](h);a.good('restore',session=h,save=s)
                assert q['save'](h)==s;restores+=1
            if total%127==0:
                s=q['save'](h)
                a.good('camera',session=h,request={'version':1,'zoom':1+(total%3)*.1,'tilt':1.,'shear':0.,'origin':[0.,0.],'transition_ticks':2})
                a.good('view',session=h,family=['Classic2D','TopDown','Side','Vertical'][(total//127)%4])
                assert q['save'](h)['core_save2']==s['core_save2'];cameras+=1
                bad=q['save'](h);a.bad('view',session=h,family='Invalid');assert q['save'](h)==bad
            if total%509==0:
                before=q['observe'](h);a.good('gameplay',session=h,enabled=False)
                assert q['observe'](h)['battle']==battle
                a.good('gameplay',session=h,enabled=True)
                assert q['observe'](h)['records']==before['records'];detaches+=1
            if mode=='linked' and total%16==0:
                q['assert_frame'](q['frame'](h));frames+=1
            if total%2003==0:
                s=q['save'](h);old=h;q['close'](h);a.bad('observe','invalid_session',session=old)
                h=create();a.good('restore',session=h,save=s);assert q['save'](h)==s;cycles+=1;restores+=1
            if battle['winner']!=-1:
                q['close'](h);h=create();n=0;battle={'hp':[10000,10000],'turn':0,'winner':-1,'attack_count':0};cycles+=1
            lat.append((time.perf_counter()-t)*1000)
            if time.monotonic()-checkpoint>=30:emit();checkpoint=time.monotonic()
            # Cap load to at most 50 iterations/s per process without idle-only test.
            time.sleep(max(0,.02-(time.perf_counter()-t)))
        assert history_boundary and longest>256
    except BaseException:
        error=traceback.format_exc();raise
    finally:
        row=emit(True);row.update(status='PASS' if error is None else 'FAIL',error=error,
                                  requested_seconds=seconds,seed=seed,platform=platform.platform(),
                                  library_sha256=hashlib.sha256(pathlib.Path(os.environ['GE4G_LIB']).read_bytes()).hexdigest())
        out.write_text(json.dumps(row,indent=2)+'\n');q['close'](h);log.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('kind',choices=['smoke','endurance']);p.add_argument('--mode',default='headless');p.add_argument('--seconds',type=int,default=7200);p.add_argument('--out',type=pathlib.Path,required=True)
    args=p.parse_args();q=load()
    if args.kind=='smoke':smoke(q,args.out)
    else:endurance(q,args.mode,args.seconds,args.out)
