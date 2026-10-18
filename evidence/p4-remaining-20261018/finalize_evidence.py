"""Validate completed gate evidence and build a compact machine-readable ledger."""
import hashlib,json,pathlib,re,subprocess

root=pathlib.Path(__file__).resolve().parent
def read(name):return json.loads((root/name).read_text(encoding='utf-8-sig'))
def tests(name,count):
    r=read(name);assert len(r['tests'])==count
    assert all(t['status']=='PASS' for t in r['tests']);return r
windows=tests('windows-native.json',9);android=tests('android-native.json',7)
assert android['abi_calls']==294 and android['exit_code']==0
exits=read('process-exits.json')
assert exits['headless']==exits['linked']==0 and not exits['resource_stop_triggered']
endurance={}
for mode in ['headless','linked']:
    r=read(f'endurance-{mode}.json')
    assert r['status']=='PASS' and r['elapsed_seconds']>=7200 and r['errors']==0
    assert r['cycles']>0 and r['restores']>0 and r['cameras']>0 and r['detaches']>0
    assert r['history_boundary_seen'] and r['longest_battle_attacks']>256
    if mode=='linked':assert r['frames_verified']>0
    rows=[json.loads(s) for s in (root/f'endurance-{mode}.jsonl').read_text().splitlines()]
    assert rows[-1]['final'] and all(v['errors']==0 for v in rows)
    assert all(a['steps']<=b['steps'] and a['elapsed_seconds']<=b['elapsed_seconds'] for a,b in zip(rows,rows[1:]))
    assert max(v['rss_kib'] for v in rows)<768*1024
    r['checkpoint_count']=len(rows);r['rss_min_kib']=min(v['rss_kib'] for v in rows);r['rss_max_kib']=max(v['rss_kib'] for v in rows)
    r['max_checkpoint_gap_seconds']=max(b['elapsed_seconds']-a['elapsed_seconds'] for a,b in zip(rows,rows[1:]))
    assert r['max_checkpoint_gap_seconds']<90, 'Continuity gap requires review; cannot certify continuous endurance'
    endurance[mode]=r
for name in ['valgrind-ownership.log','valgrind-thread.log']:
    log=(root/'linux-first/valgrind'/name).read_text(encoding='utf-8')
    assert 'in use at exit: 0 bytes in 0 blocks' in log and 'ERROR SUMMARY: 0 errors' in log
instrumentation={}
for gate in ['address','thread']:
    log=(root/'linux-final'/gate/'driver.log').read_text(encoding='utf-8')
    count=sum(int(n) for n in re.findall(r'test result: ok\. (\d+) passed',log))
    assert count==20 and 'GATE EXECUTION PASS' in log
    assert (root/'linux-final'/gate/'binaries.sha256').stat().st_size>0
    instrumentation[gate]={'PASS':count,'FAIL':0,'first_execution_also_passed':20,
                           'first_job_failed_after_tests':'QA hash collection assumed old Cargo deps path; fixed and rerun to retain binary hashes'}
summary={'implementation':'72d49a63f730e2a3ee2032e83948894e4fd480ba',
         'previous_qa':'7258d48e1b02ec2c8bdca9099267bd17da924ecc',
         'windows_native_groups':{'PASS':9,'FAIL':0},'android_native_groups':{'PASS':7,'FAIL':0,'abi_calls':294},
         'sanitizer_final_tests':instrumentation,'valgrind_processes':{'PASS':2,'FAIL':0,'heap_at_exit_bytes':0,'errors':0,'suppressed':0},
         'endurance':endurance,'gates':{'A':'PASS Windows native C ABI; Flutter UI NOT TESTED',
         'B':'PASS Android x86_64 emulator native C ABI; ARM/physical device/Flutter UI NOT TESTED',
         'C':'PASS ASan instrumented Rust/FFI + rebuilt std','D':'PASS Valgrind original CI SDK ELF, two processes',
         'E':'PASS TSan instrumented Rust/FFI + rebuilt std','F':'NOT TESTED enforced blind new-agent consumer; available tools cannot restrict agent filesystem/tool read access',
         'G':'PASS two separate continuous >=7200-second mixed-load executions'},
         'confirmed_engine_defects':0,'product_code_changes':0,'qa_defects_fixed':2,
         'prior_evidence_only':['CI SDK24/24','local13/13','workspace169/169','headlessCLI9/9','five legacy games/fmt/strictClippy/boundaries'],
         'approval':{'code_merge':'Technically supportable for experimental P4 code; no demonstrated engine defect. Full P4 acceptance remains conditional on independent consumer gate; no merge executed.',
                     'platform':'Windows x64 and Android x86_64 emulator minimal native consumption approved only within tested scope. App/ARM/physical device delivery not approved.',
                     'release':'WITHHELD: strict independent consumer missing; platform app delivery and signing/device validation absent.'}}
(root/'summary.json').write_text(json.dumps(summary,indent=2)+'\n',encoding='utf-8')
hashes={str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(root.rglob('*')) if p.is_file() and p.name!='SHA256SUMS.json' and '__pycache__' not in p.parts}
(root/'SHA256SUMS.json').write_text(json.dumps(hashes,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'gates_passed':6,'not_tested':1,'windows_groups':9,'android_groups':7,'sanitizer_final_tests':40,'valgrind_processes':2,'endurance_seconds':{m:r['elapsed_seconds'] for m,r in endurance.items()}},indent=2))
