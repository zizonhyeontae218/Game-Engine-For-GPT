import ctypes,json,copy,time
lib=ctypes.CDLL('./target/debug/libge4g_pentomino_native.so')
lib.ge4g_p5_v1_request.argtypes=[ctypes.c_void_p,ctypes.c_size_t]
lib.ge4g_p5_v1_request.restype=ctypes.c_void_p
lib.ge4g_p5_v1_string_free.argtypes=[ctypes.c_void_p]
def call(c):
 b=json.dumps({'version':1,'command':c},separators=(',',':')).encode(); buf=ctypes.create_string_buffer(b); p=lib.ge4g_p5_v1_request(buf,len(b));
 try:return json.loads(ctypes.string_at(p))
 finally:lib.ge4g_p5_v1_string_free(p)
def ok(c):
 r=call(c); assert r['ok'],r; return r['result']
def f(i):return {'id':f'f{i}','team':i//2,'hp':10000,'resource':1,'attack':0,'defense':10000,'speed':4-i,'position':[20+i*40,72],'color':[20+i*40,100,200,255]}
game={'version':2,'width':180,'height':96,'tick_hz':60,'fighters':[f(i) for i in range(4)],'actions':[{'id':'wait','kind':'resource','target':'self_only','cost':0,'power':0,'effect':None,'duration':0}]}
h=ok({'op':'create','config':game,'seed':41,'mode':'headless'})['session']
t=time.perf_counter()
for i in range(300):
 a=i%4; r=ok({'op':'act','session':h,'command':{'id':i+1,'actor':a,'action':0,'target':a}}); assert r['battle']['turn']==(i+1)%4; assert r['battle']['fighters'][a]['hp']==10000
s=ok({'op':'save','session':h}); assert s['logical_step']==300
core=json.loads(s['core_save2']); provider=[p for p in core['plugins'] if p['descriptor']['id']=='p5.state'][0]
assert provider['history']['emitted']==300 and len(provider['history']['history'])==256
ok({'op':'restore','session':h,'save':s}); assert ok({'op':'save','session':h})==s
for patch in ['hp','id','trace','binding','odd_tick']:
 bad=copy.deepcopy(s)
 if patch=='hp':
  c=json.loads(bad['core_save2']); [p for p in c['plugins'] if p['descriptor']['id']=='p5.state'][0]['records']['status']['fields']['f0_hp']={'I64':9999}; bad['core_save2']=json.dumps(c,separators=(',',':'))
 elif patch=='id': bad['trace'][-1]['command']['id']=301
 elif patch=='trace': bad['trace'].pop()
 elif patch=='binding': bad['content_binding']='p5:bad'
 elif patch=='odd_tick': bad['logical_step']=301
 r=call({'op':'restore','session':h,'save':bad}); assert not r['ok'],patch; assert ok({'op':'save','session':h})==s
ok({'op':'gameplay','session':h,'enabled':False}); ds=ok({'op':'save','session':h}); ok({'op':'restore','session':h,'save':ds}); ok({'op':'gameplay','session':h,'enabled':True});
assert not call({'op':'act','session':h,'command':{'id':300,'actor':0,'action':0,'target':0}})['ok']
ok({'op':'act','session':h,'command':{'id':301,'actor':0,'action':0,'target':0}})
ok({'op':'close','session':h}); print(json.dumps({'audit':'P5 public C internal-independent probe','commands':301,'retained_history':256,'tamper_cases':5,'lifecycle_restore':True,'outcome':'PASS','elapsed_seconds':round(time.perf_counter()-t,3)}))
