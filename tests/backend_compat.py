"""One-time backend-update comparison; old artifact is optional local evidence,
never a dependency of packaged tests or the independent normative oracle.
"""
import base64,ctypes,json,os
from pathlib import Path

def load(path):
    lib=ctypes.CDLL(path);fn=lib.ap2_verify_v1
    fn.argtypes=[ctypes.c_void_p,ctypes.c_size_t,ctypes.c_void_p,ctypes.c_size_t];fn.restype=ctypes.c_ssize_t
    return lib,fn
old,oldfn=load(os.environ['OLD_CORE_LIBRARY'])
new,newfn=load('/work/target/release/libap2_verifier_core.so')
def invoke(fn,case):
    b=json.dumps(dict(contract_version='ap2-core-request.1',operation=case['operation'],arguments=case['arguments']),separators=(',',':')).encode()
    out=ctypes.create_string_buffer(262144);n=fn(b,len(b),out,len(out))
    return json.loads(out.raw[:n]) if n>0 else dict(abi_return=n)
cases=json.loads(Path('/work/conformance/v1/cases.json').read_text())
for c in cases:assert invoke(oldfn,c)==invoke(newfn,c),c['id']
c=next(c for c in cases if c['id']=='checkout');jwt=c['arguments']['jwt'];h,p,_=jwt.split('.')
order=int('FFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551',16)
signatures=[b'\0'*64,(1).to_bytes(32,'big')+b'\0'*32,order.to_bytes(32,'big')+(1).to_bytes(32,'big'),(1).to_bytes(32,'big')+order.to_bytes(32,'big'),b'\xff'*64,b'\0'*63,b'\0'*65]
for signature in signatures:
    test=json.loads(json.dumps(c));test['arguments']['jwt']=h+'.'+p+'.'+base64.urlsafe_b64encode(signature).decode().rstrip('=')
    a=invoke(oldfn,test);b=invoke(newfn,test);assert a==b,(a,b)
    assert a['verified'] is False and a['evidence'] is None
print(json.dumps(dict(retained_full_result_comparisons=len(cases),invalid_scalar_signature_comparisons=len(signatures),old_and_new_exact_result_match=True)))
