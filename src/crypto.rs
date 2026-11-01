use crate::{exact, json::parse, text, Error, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use p256::{
    ecdsa::{signature::Verifier, Signature, VerifyingKey},
    pkcs8::{DecodePublicKey, EncodePublicKey, LineEnding},
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
pub fn hash(wire: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(wire.as_bytes()))
}
pub fn hex(wire: &str) -> String {
    format!("{:x}", Sha256::digest(wire.as_bytes()))
}
pub fn decode(s: &str, limit: usize) -> Result<Vec<u8>> {
    if s.is_empty() || s.len() > limit.div_ceil(3) * 4 {
        return Err(Error::reject("invalid_root_jws"));
    }
    let b = URL_SAFE_NO_PAD
        .decode(s)
        .map_err(|_| Error::reject("invalid_root_jws"))?;
    if b.len() > limit || URL_SAFE_NO_PAD.encode(&b) != s {
        return Err(Error::reject("invalid_root_jws"));
    }
    Ok(b)
}
pub fn key(pem: &str) -> Result<VerifyingKey> {
    let err = || Error::unavailable("issuer_key_unsupported");
    if pem.len() > 1024 || !pem.is_ascii() || !pem.starts_with("-----BEGIN PUBLIC KEY-----\n") {
        return Err(err());
    }
    let k = VerifyingKey::from_public_key_pem(pem).map_err(|_| err())?;
    if k.to_public_key_pem(LineEnding::LF).map_err(|_| err())? != pem {
        return Err(err());
    }
    Ok(k)
}
pub fn jwk(pem: &str) -> Result<Value> {
    let k = key(pem)?;
    let p = k.to_encoded_point(false);
    Ok(
        json!({"kty":"EC","crv":"P-256","x":URL_SAFE_NO_PAD.encode(p.x().ok_or_else(||Error::unavailable("issuer_key_unsupported"))?),"y":URL_SAFE_NO_PAD.encode(p.y().ok_or_else(||Error::unavailable("issuer_key_unsupported"))?)}),
    )
}
pub fn syntax(wire: &str, typ: &str, max: usize, payload_max: usize) -> Result<(Value, Signature)> {
    if wire.len() > max {
        return Err(Error::reject("input_too_large"));
    }
    if !wire.is_ascii() {
        return Err(Error::reject("invalid_root_jws"));
    }
    let p: Vec<_> = wire.split('.').collect();
    if p.len() != 3 {
        return Err(Error::reject("invalid_root_jws"));
    }
    let h = parse(&decode(p[0], 2048)?)?;
    let payload = parse(&decode(p[1], payload_max)?)?;
    let sig =
        Signature::from_slice(&decode(p[2], 64)?).map_err(|_| Error::reject("invalid_root_jws"))?;
    let o = h
        .as_object()
        .ok_or_else(|| Error::reject("unsupported_root_header"))?;
    if o.keys()
        .any(|k| !["alg", "typ", "kid"].contains(&k.as_str()))
    {
        return Err(Error::reject("unsupported_root_header"));
    }
    if let Some(v) = o.get("kid") {
        text(v, 128).map_err(|_| Error::reject("unsupported_root_header"))?;
    }
    if h["alg"] != "ES256" {
        return Err(Error::reject("unsupported_root_algorithm"));
    }
    if h["typ"] != typ {
        return Err(Error::reject("unsupported_root_type"));
    }
    if !payload.is_object() {
        return Err(Error::reject("invalid_root_payload"));
    }
    Ok((payload, sig))
}
pub fn verify(wire: &str, pem: &str, typ: &str, max: usize) -> Result<Value> {
    let (p, s) = syntax(wire, typ, max, 24576)?;
    let k = key(pem)?;
    let dot = wire
        .rfind('.')
        .ok_or_else(|| Error::reject("invalid_root_jws"))?;
    k.verify(&wire.as_bytes()[..dot], &s)
        .map_err(|_| Error::reject("invalid_root_signature"))?;
    Ok(p)
}
pub fn disclosure(wire: &str, size: usize, direct: bool) -> Result<Value> {
    let v = parse(&decode(wire, 24576)?)?;
    let a = v
        .as_array()
        .ok_or_else(|| Error::reject("invalid_disclosure"))?;
    if a.len() != size {
        return Err(Error::reject("invalid_disclosure"));
    }
    let salt = text(&a[0], 128)?;
    if salt.len() < if direct { 22 } else { 8 }
        || direct
            && !salt
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
    {
        return Err(Error::reject("invalid_disclosure"));
    }
    Ok(v)
}
pub fn direct(wire: &str) -> Result<Value> {
    if !wire.is_ascii() || wire.len() > 32768 {
        return Err(Error::reject("invalid_direct_presentation"));
    }
    let p: Vec<_> = wire.split('~').collect();
    if p.len() != 4 || !p[3].is_empty() {
        return Err(Error::reject("invalid_direct_presentation"));
    }
    let (payload, _) = syntax(p[0], "example+sd-jwt", 32768, 2048)?;
    exact(&payload, &["_sd_alg", "delegate_payload"])?;
    if payload["_sd_alg"] != "sha-256" || payload["delegate_payload"] != json!([{"...":hash(p[1])}])
    {
        return Err(Error::reject("invalid_direct_presentation"));
    }
    let body = disclosure(p[1], 2, true)?;
    let mut closed = body[1].clone();
    exact(&closed, &["_sd", "checkout_hash", "exp", "iat", "vct"])?;
    if closed["vct"] != "mandate.checkout.1" || closed["_sd"] != json!([hash(p[2])]) {
        return Err(Error::reject("invalid_direct_presentation"));
    }
    crate::times(&closed, None, 600)?;
    let leaf = disclosure(p[2], 3, true)?;
    let checkout = text(&leaf[2], 16384)?;
    if leaf[1] != "checkout_jwt" || closed["checkout_hash"] != hash(checkout) {
        return Err(Error::reject("checkout_binding_mismatch"));
    }
    let (c, _) = syntax(checkout, "JWT", 16384, 12000)?;
    crate::constraints::checkout(&c, crate::MULTI)?;
    closed
        .as_object_mut()
        .ok_or_else(|| Error::reject("invalid_direct_presentation"))?
        .insert("checkout_jwt".into(), leaf[2].clone());
    Ok(closed)
}
