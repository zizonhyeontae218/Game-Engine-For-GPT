import ctypes,json,copy
l=ctypes.CDLL('./target/debug/libge4g_pentomino_native.so');l.ge4g_p5_v1_request.argtypes=[ctypes.c_void_p,ctypes.c_size_t];l.ge4g_p5_v1_request.restype=ctypes.c_void_p;l.ge4g_p5_v1_string_free.argtypes=[ctypes.c_void_p]
def c(cmd):
 b=json.dumps({'version':1,'command':cmd}).encode();p=l.ge4g_p5_v1_request(b,len(b))
 try:return json.loads(ctypes.string_at(p))
 finally:l.ge4g_p5_v1_string_free(p)
g=json.load(open('samples/pentomino_p5/game.json'));h=c({'op':'create','config':g,'seed':1,'mode':'headless'})['result']['session'];assert c({'op':'act','session':h,'command':{'id':1,'actor':0,'action':0,'target':1}})['ok'];s=c({'op':'save','session':h})['result'];bad=copy.deepcopy(s);v=json.loads(bad['core_save2']);p=next(x for x in v['plugins'] if x['descriptor']['id']=='p5.state');field=p['records']['status']['fields']['f0_hp'];assert field['type']=='i64';field['value']-=1;bad['core_save2']=json.dumps(v,separators=(',',':'));r=c({'op':'restore','session':h,'save':bad});assert not r['ok'];assert c({'op':'save','session':h})['result']==s;assert c({'op':'close','session':h})['ok'];print(json.dumps({'schema_valid_hp_tamper':'PASS rejected','atomic_save_preservation':'PASS','error':r['error']['code']}))
