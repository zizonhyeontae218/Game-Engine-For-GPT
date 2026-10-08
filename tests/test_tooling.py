import json,sys,tempfile,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];sys.path.insert(0,str(ROOT/'scripts'))
from validate_pentaworks import validate
from stage_consumer_bundle import StageError,stage,validate_name
from check_gate_report import check
class Tests(unittest.TestCase):
    def test_roles(self):self.assertEqual([],validate(ROOT))
    def test_allowlist_guards(self):
        for x in ('../secret','src/kernel.py','docs/public/../secret','/etc/passwd','.codex/agents/a.toml','docs/public/.env','docs/public/a\\b'):
            with self.subTest(x=x),self.assertRaises(StageError):validate_name(x)
    def test_staging(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d)/'src';root.mkdir();(root/'README.md').write_text('public');(root/'secret.py').write_text('secret')
            (root/'docs/public').mkdir(parents=True);(root/'docs/public/api.md').write_text('api')
            allowed=Path(d)/'allow.txt';allowed.write_text('README.md\ndocs/public/api.md\n')
            target=Path(d)/'blind';manifest=stage(root,allowed,target)
            self.assertEqual(2,len(manifest));self.assertFalse((target/'secret.py').exists())
            self.assertEqual(2,len(json.loads((target/'PUBLIC_BUNDLE_MANIFEST.json').read_text(encoding="utf-8"))['files']))
    def test_symlink_block(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d)/'root';root.mkdir();(root/'private').write_text('secret');(root/'README.md').symlink_to(root/'private')
            allow=Path(d)/'list';allow.write_text('README.md\n')
            with self.assertRaises(StageError):stage(root,allow,Path(d)/'blind')
    def test_gate_evidence(self):
        data={'release_id':'0.3a1','source_commit':'abcdef','gates':[{'id':f'G{i}','status':'UNVERIFIED','evidence':[]} for i in range(9)]}
        self.assertEqual([],check(data));data['gates'][2]['status']='PASS'
        self.assertTrue(check(data));data['gates'][2]['evidence']=[{'command':'test','observed':'passed','artifact':'abcdef'}]
        self.assertEqual([],check(data))
if __name__=='__main__':unittest.main()
