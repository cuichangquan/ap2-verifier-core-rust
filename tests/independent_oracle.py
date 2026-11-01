"""Pinned official SDK + cryptography oracle. No Ruby verdict is consulted.
Synthetic graph design is shared; this is not real-issuer provenance proof.
"""
import base64,ctypes,hashlib,json
from pathlib import Path
from cryptography.hazmat.primitives import hashes,serialization
from cryptography.hazmat.primitives.asymmetric import ec,utils
from jwcrypto.jwk import JWK
from ap2.sdk.sdjwt import sd_jwt
from ap2.sdk.generated.checkout_mandate import CheckoutMandate

ROOT=Path('/work')
lib=ctypes.CDLL(str(ROOT/'target/debug/libap2_verifier_core.so'))
lib.ap2_verify_v1.argtypes=[ctypes.c_void_p,ctypes.c_size_t,ctypes.c_void_p,ctypes.c_size_t]
lib.ap2_verify_v1.restype=ctypes.c_ssize_t

def rpc(op,args):
    source=json.dumps(dict(contract_version='ap2-core-request.1',operation=op,arguments=args)).encode()
    output=ctypes.create_string_buffer(262144)
    n=lib.ap2_verify_v1(source,len(source),output,len(output))
    assert n>0
    return json.loads(output.raw[:n])
def b64(v):return base64.urlsafe_b64encode(v).decode().rstrip('=')
def unb64(v):return base64.urlsafe_b64decode(v+'='*(-len(v)%4))
def digest(v):return b64(hashlib.sha256(v.encode()).digest())
def decode(v):return json.loads(unb64(v))
def verify_jws(wire,pem):
    h,p,s=wire.split('.')
    assert decode(h)['alg']=='ES256'
    sig=unb64(s);assert len(sig)==64
    der=utils.encode_dss_signature(int.from_bytes(sig[:32],'big'),int.from_bytes(sig[32:],'big'))
    serialization.load_pem_public_key(pem.encode()).verify(der,f'{h}.{p}'.encode(),ec.ECDSA(hashes.SHA256()))
    return decode(p)

def sign(payload,key,typ='JWT'):
    h=b64(json.dumps(dict(alg='ES256',typ=typ),separators=(',',':')).encode());p=b64(json.dumps(payload,separators=(',',':')).encode())
    r,s=utils.decode_dss_signature(key.sign(f'{h}.{p}'.encode(),ec.ECDSA(hashes.SHA256())))
    return f'{h}.{p}.{b64(r.to_bytes(32,"big")+s.to_bytes(32,"big"))}'

cases=json.loads((ROOT/'conformance/v1/cases.json').read_text())
checks=0
for case in cases:
    a=case['arguments']
    if case['operation']=='verify_checkout' and case['expected_status']=='verified_research':
        trust=json.loads(a['trust_json']);selection=json.loads(a['selection_json']);snapshot=json.loads(trust['snapshot']['body'])
        keys={role:next(r['public_key_pem'] for r in snapshot['records'] if r['role']==role and all(r[k]==selection[role][k] for k in ['identity','purpose','key_ref'])) for role in ['Issuer','Agent','Merchant']}
        resolved=sd_jwt.verify(a['root_presentation'],JWK.from_pem(keys['Issuer'].encode()))
        opening=resolved['delegate_payload'][0]
        verify_jws(a['root_presentation'].split('~')[0],keys['Issuer'])
        proof=verify_jws(a['terminal_jws'],keys['Agent']);checkout=verify_jws(a['host_checkout_jws'],keys['Merchant'])
        assert proof['sd_hash']==digest(a['root_presentation'])
        assert proof['delegate_payload'][0]['checkout_hash']==digest(a['host_checkout_jws'])
        assert opening['vct']=='mandate.checkout.open.1'
        checks+=5
    if case['operation']=='direct':
        resolved=sd_jwt.verify(a['closed_sd_jwt'],JWK.from_pem(a['trusted_provider_public_key_pem'].encode()))
        assert resolved['delegate_payload'][0]['checkout_jwt']==a['expected_checkout_jwt'];checks+=1
    if case['operation']=='receipt':
        claims=verify_jws(a['jwt'],a['trusted_public_key_pem'])
        assert claims['reference']==digest(a['closed_sd_jwt']);checks+=1

# Independent official-SDK issuance -> Rust Direct, not Ruby-issued only.
key=ec.generate_private_key(ec.SECP256R1())
pem=key.public_key().public_bytes(serialization.Encoding.PEM,serialization.PublicFormat.SubjectPublicKeyInfo).decode()
jwk=JWK.from_pyca(key)
source=next(c for c in cases if c['id']=='checkout')['arguments']
checkout=decode(source['jwt'].split('.')[1]);wire=sign(checkout,key)
closed=dict(vct='mandate.checkout.1',checkout_jwt=wire,checkout_hash=digest(wire),iat=1800000000,exp=1800000060)
issuance=sd_jwt.create(CheckoutMandate(**closed),jwk).sd_jwt_issuance
parts=issuance.split('~');parts[1:-1]=sorted(parts[1:-1],key=lambda v:len(decode(v)));issuance='~'.join(parts)
args=dict(closed_sd_jwt=issuance,trusted_provider_public_key_pem=pem,trusted_merchant_public_key_pem=pem,expected_checkout_jwt=wire,expected_merchant_id='shop',verified_at=1800000000)
assert rpc('direct',args)['verified'];checks+=1
for mutation in [dict(verified_at=1800000060),dict(expected_checkout_jwt=wire+'x'),dict(closed_sd_jwt=issuance.replace('~','~~',1))]:
    assert not rpc('direct',args|mutation)['verified'];checks+=1
# Signed causal merchant negatives: valid signature cannot bless unknown shape,
# fractional quantities, oversized integers, extras, totals or bad currency.
for mutate in [lambda c:c.update(currency='ZZZ'),lambda c:c.update(extra=True),lambda c:c['line_items'][0].update(quantity=2.0),lambda c:c['totals'][0].update(amount=1),lambda c:c['line_items'][0]['item'].update(price=9007199254740992),lambda c:c['line_items'].append(c['line_items'][0])]:
    c=json.loads(json.dumps(checkout));mutate(c);bad=sign(c,key)
    assert verify_jws(bad,pem)==c
    result=rpc('checkout',dict(jwt=bad,trusted_public_key_pem=pem,expected_merchant_id='shop',profile_ref='ap2.checkout.single_merchant_multi_item'))
    assert not result['verified'] and result['evidence'] is None;checks+=1
# Signature mutation, wrong key, unknown JWK/header lookup, duplicate JSON keys.
for payload in ['{"alg":"ES256","alg":"ES256","typ":"JWT"}','{"alg":"ES256","typ":"JWT","jwk":{}}']:
    h=b64(payload.encode());p=b64(json.dumps(checkout).encode());r,s=utils.decode_dss_signature(key.sign(f'{h}.{p}'.encode(),ec.ECDSA(hashes.SHA256())))
    bad=f'{h}.{p}.{b64(r.to_bytes(32,"big")+s.to_bytes(32,"big"))}'
    assert not rpc('checkout',dict(jwt=bad,trusted_public_key_pem=pem,expected_merchant_id='shop',profile_ref='ap2.checkout.single_merchant_multi_item'))['verified'];checks+=1
print(json.dumps(dict(independent_checks=checks,official_ap2='b4587ac1d055888a73b4b21750973cffba961793',oracle='official sd-jwt + cryptography 46.0.5',network=False)))
# Causal negatives retain independently valid signatures at all three stages.
# SDK differences (for example hidden/unknown fields) are not treated as ACCEPT.
causal_checks=0
for case in cases:
    if not case['id'].startswith('causal-'):continue
    a=case['arguments'];selection=json.loads(a['selection_json']);records=json.loads(json.loads(a['trust_json'])['snapshot']['body'])['records']
    keys={role:next(r['public_key_pem'] for r in records if r['role']==role) for role in ['Issuer','Agent','Merchant']}
    verify_jws(a['root_presentation'].split('~')[0],keys['Issuer'])
    proof=verify_jws(a['terminal_jws'],keys['Agent']);checkout=verify_jws(a['host_checkout_jws'],keys['Merchant'])
    assert proof['sd_hash']==digest(a['root_presentation']) and proof['delegate_payload'][0]['checkout_hash']==digest(a['host_checkout_jws'])
    assert not rpc('verify_checkout',a)['verified'];causal_checks+=4
print(json.dumps(dict(causal_signature_and_binding_checks=causal_checks)))
# Independent bounded unit-enumeration oracle, deliberately different from Rust
# residual max-flow. Test fixture quantities are small; never use this in core.
def exact_units(requirements,lines):
    slots=[]
    for r in requirements:
        slots += [{i['id'] for i in r['acceptable_items']}] * r['quantity']
    units=[l['item']['id'] for l in lines for _ in range(l['quantity'])]
    if len(slots)!=len(units):return False
    def search(i,remaining):
        if i==len(units):return True
        return any(units[i] in allowed and search(i+1,remaining[:j]+remaining[j+1:]) for j,allowed in enumerate(remaining))
    return search(0,slots)
for ident,expected in [('multi-host',True),('causal-shortage',False),('causal-extra',False),('causal-wrong-product',False)]:
    a=next(c for c in cases if c['id']==ident)['arguments']
    records=json.loads(json.loads(a['trust_json'])['snapshot']['body'])['records']
    pem=next(r['public_key_pem'] for r in records if r['role']=='Issuer')
    root=sd_jwt.verify(a['root_presentation'],JWK.from_pem(pem.encode()))
    c=decode(a['host_checkout_jws'].split('.')[1])
    assert exact_units(root['delegate_payload'][0]['constraints'][0]['items'],c['line_items']) is expected
print(json.dumps(dict(independent_exact_quantity_checks=4)))
