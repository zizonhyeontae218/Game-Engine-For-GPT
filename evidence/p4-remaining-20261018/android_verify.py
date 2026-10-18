"""Run independent assertions through an actual adb native C ABI consumer."""
import hashlib,json,os,pathlib,subprocess,sys
from native_verify import load,ROOT

adb=os.environ['GE4G_ADB'];serial='emulator-5580'
device='/data/local/tmp/ge4g-p4-qa-20261018'
q=load()
process=subprocess.Popen([adb,'-s',serial,'shell','-T',device+'/consumer'],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=(ROOT/'android-consumer.stderr').open('wb'))
class Remote(q['API']):
    def __init__(self):self.calls=0
    def raw(self,bs):
        assert b'\n' not in (bs or b'')
        process.stdin.write((bs or b'')+b'\n');process.stdin.flush()
        line=process.stdout.readline();assert line,'Android consumer terminated'
        result=json.loads(line);self.calls+=1
        assert result['version']==1 and (result['error'] is None)==result['ok']
        return result
q['api']=remote=Remote()
try:
    for name in ['turn_combat_independent','invalid_input_atomicity','restore_atomicity',
                 'detach_reinstall','camera_and_view_atomicity','lifecycle_and_limit',
                 'headless_restoration_and_binding']:
        q['check'](name,q[name])
finally:
    process.stdin.close();process.wait(timeout=30)
    report={'platform':'Android emulator x86_64; native executable/C ABI, no Flutter UI',
            'tests':q['TESTS'],'abi_calls':remote.calls,'exit_code':process.returncode,
            'library_sha256':hashlib.sha256(pathlib.Path(os.environ['GE4G_ANDROID_LIB']).read_bytes()).hexdigest()}
    (ROOT/'android-native.json').write_text(json.dumps(report,indent=2)+'\n')
assert process.returncode==0 and all(t['status']=='PASS' for t in q['TESTS'])
