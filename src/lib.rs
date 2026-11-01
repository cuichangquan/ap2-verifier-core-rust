//! Experimental fixed-profile verifier. Checked facts never authorize a purchase.
mod constraints;
mod crypto;
mod disclosures;
mod ffi;
mod host;
mod json;
use serde_json::{json, Value};
pub const SINGLE: &str = "ap2.checkout.single_merchant_sku";
pub const MULTI: &str = "ap2.checkout.single_merchant_multi_item";
pub const DIRECT: &str = "ap2.checkout.direct.single_merchant";
pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug)]
pub struct Error {
    pub status: &'static str,
    pub code: &'static str,
}
impl Error {
    pub fn reject(code: &'static str) -> Self {
        Self {
            status: "rejected",
            code,
        }
    }
    pub fn unavailable(code: &'static str) -> Self {
        Self {
            status: "unavailable",
            code,
        }
    }
}
fn exact(v: &Value, keys: &[&str]) -> Result<()> {
    let o = v
        .as_object()
        .ok_or_else(|| Error::reject("invalid_schema"))?;
    if o.len() != keys.len() || keys.iter().any(|k| !o.contains_key(*k)) {
        return Err(Error::reject("invalid_schema"));
    }
    Ok(())
}
fn text(v: &Value, max: usize) -> Result<&str> {
    let s = v.as_str().ok_or_else(|| Error::reject("invalid_schema"))?;
    if s.is_empty() || s.len() > max {
        return Err(Error::reject("invalid_schema"));
    }
    Ok(s)
}
fn number(v: &Value, min: u64, max: u64) -> Result<u64> {
    v.as_u64()
        .filter(|v| *v >= min && *v <= max)
        .ok_or_else(|| Error::reject("invalid_schema"))
}
fn array(v: &Value, min: usize, max: usize) -> Result<&Vec<Value>> {
    v.as_array()
        .filter(|a| a.len() >= min && a.len() <= max)
        .ok_or_else(|| Error::reject("invalid_schema"))
}
fn epoch(v: &Value) -> Result<u64> {
    number(v, 0, 4_102_444_800)
}
fn times(v: &Value, now: Option<u64>, max: u64) -> Result<()> {
    let i = epoch(&v["iat"])?;
    let e = epoch(&v["exp"])?;
    if e <= i || e - i > max || now.is_some_and(|n| n < i || n >= e) {
        return Err(Error::reject("invalid_purchase_time"));
    }
    Ok(())
}
fn field<'a>(v: &'a Value, k: &str, max: usize) -> Result<&'a str> {
    text(&v[k], max)
}
fn checkout_verify(a: &Value) -> Result<Value> {
    exact(
        a,
        &[
            "jwt",
            "trusted_public_key_pem",
            "expected_merchant_id",
            "profile_ref",
        ],
    )?;
    let profile = field(a, "profile_ref", 256)?;
    let wire = field(a, "jwt", 16384)?;
    let c = crypto::verify(
        wire,
        field(a, "trusted_public_key_pem", 1024)?,
        "JWT",
        16384,
    )?;
    constraints::checkout(&c, profile)?;
    if c["merchant"]["id"] != field(a, "expected_merchant_id", 512)? {
        return Err(Error::reject("checkout_identity_mismatch"));
    }
    Ok(json!({"checkout":c,"checkout_hash":crypto::hash(wire),"profile_ref":profile}))
}
fn direct_verify(a: &Value) -> Result<Value> {
    exact(
        a,
        &[
            "closed_sd_jwt",
            "trusted_provider_public_key_pem",
            "trusted_merchant_public_key_pem",
            "expected_checkout_jwt",
            "expected_merchant_id",
            "verified_at",
        ],
    )?;
    let now = epoch(&a["verified_at"])?;
    let wire = field(a, "closed_sd_jwt", 32768)?;
    let c = crypto::direct(wire)?;
    crypto::verify(
        wire.split('~').next().unwrap_or(""),
        field(a, "trusted_provider_public_key_pem", 1024)?,
        "example+sd-jwt",
        32768,
    )?;
    times(&c, Some(now), 600)?;
    let expected = field(a, "expected_checkout_jwt", 16384)?;
    if c["checkout_jwt"] != expected {
        return Err(Error::reject("checkout_binding_mismatch"));
    }
    let checkout = checkout_verify(
        &json!({"jwt":expected,"trusted_public_key_pem":a["trusted_merchant_public_key_pem"],"expected_merchant_id":a["expected_merchant_id"],"profile_ref":MULTI}),
    )?;
    Ok(
        json!({"checkout":checkout["checkout"],"checkout_hash":c["checkout_hash"],"reference":crypto::hash(wire),"expires_at":c["exp"]}),
    )
}
fn reference(wire: &str, profile: &str) -> Result<String> {
    if profile == DIRECT {
        crypto::direct(wire)?;
    } else {
        if ![SINGLE, MULTI].contains(&profile)
            || wire.len() > 32769
            || !wire.ends_with('~')
            || wire.bytes().filter(|c| *c == b'~').count() != 1
        {
            return Err(Error::reject("invalid_receipt_reference"));
        }
        let (v, _) = crypto::syntax(&wire[..wire.len() - 1], "kb+sd-jwt", 32768, 24576)?;
        exact(&v, &["aud", "delegate_payload", "iat", "nonce", "sd_hash"])?;
        for k in ["aud", "nonce", "sd_hash"] {
            field(&v, k, 512)?;
        }
        epoch(&v["iat"])?;
        crypto::decode(field(&v, "sd_hash", 43)?, 32)?;
        let c = &array(&v["delegate_payload"], 1, 1)?[0];
        exact(c, &["checkout_hash", "checkout_jwt", "exp", "iat", "vct"])?;
        let i = epoch(&c["iat"])?;
        let e = epoch(&c["exp"])?;
        if c["vct"] != "mandate.checkout.1" || e <= i {
            return Err(Error::reject("invalid_receipt_reference"));
        }
        let checkout = field(c, "checkout_jwt", 16384)?;
        let (claims, _) = crypto::syntax(checkout, "JWT", 16384, 12000)?;
        constraints::checkout(&claims, profile)?;
        if c["checkout_hash"] != crypto::hash(checkout) {
            return Err(Error::reject("checkout_binding_mismatch"));
        }
    }
    Ok(crypto::hash(wire))
}
fn receipt_verify(a: &Value) -> Result<Value> {
    exact(
        a,
        &[
            "jwt",
            "closed_sd_jwt",
            "trusted_public_key_pem",
            "expected_issuer",
            "not_before",
            "not_after",
            "verified_at",
            "profile_ref",
        ],
    )?;
    let before = epoch(&a["not_before"])?;
    let after = epoch(&a["not_after"])?;
    let now = epoch(&a["verified_at"])?;
    if before > now || now > after {
        return Err(Error::reject("receipt_time_mismatch"));
    }
    let r = reference(
        field(a, "closed_sd_jwt", 32769)?,
        field(a, "profile_ref", 256)?,
    )?;
    let c = crypto::verify(
        field(a, "jwt", 16384)?,
        field(a, "trusted_public_key_pem", 1024)?,
        "JWT",
        16384,
    )?;
    let keys = match c["status"].as_str() {
        Some("Success") => vec!["iss", "status", "iat", "reference", "order_id"],
        Some("Error") => vec![
            "iss",
            "status",
            "iat",
            "reference",
            "error",
            "error_description",
        ],
        _ => return Err(Error::reject("invalid_receipt")),
    };
    exact(&c, &keys)?;
    for k in keys.iter().filter(|k| **k != "iat") {
        field(&c, k, if *k == "error_description" { 2048 } else { 512 })?;
    }
    if c["iss"] != field(a, "expected_issuer", 512)? {
        return Err(Error::reject("receipt_issuer_mismatch"));
    }
    if c["reference"] != r {
        return Err(Error::reject("receipt_reference_mismatch"));
    }
    let issued = epoch(&c["iat"])?;
    if issued < before || issued > now {
        return Err(Error::reject("receipt_time_mismatch"));
    }
    Ok(json!({"outcome":c["status"],"purchase_succeeded":c["status"]=="Success","receipt":c}))
}
fn scoped(a: &Value) -> Result<Value> {
    exact(
        a,
        &[
            "root_presentation",
            "terminal_jws",
            "host_checkout_jws",
            "trusted_issuer_public_key_pem",
            "expected_agent_public_key_pem",
            "trusted_merchant_public_key_pem",
            "expected_aud",
            "expected_nonce",
            "verified_at",
            "expected_checkout_id",
            "expected_merchant_id",
            "expected_currency",
            "expected_total_minor",
            "profile_ref",
        ],
    )?;
    let profile = field(a, "profile_ref", 256)?;
    let now = epoch(&a["verified_at"])?;
    let root_wire = field(a, "root_presentation", 131072)?;
    let agent = field(a, "expected_agent_public_key_pem", 1024)?;
    let root = disclosures::root(
        root_wire,
        field(a, "trusted_issuer_public_key_pem", 1024)?,
        profile,
    )?;
    let open = &root["delegate_payload"][0];
    exact(open, &["cnf", "constraints", "exp", "iat", "vct"])?;
    if open["vct"] != "mandate.checkout.open.1" {
        return Err(Error::reject("unsupported_purchase_profile"));
    }
    times(open, Some(now), 7200)?;
    exact(&open["cnf"], &["jwk"])?;
    if open["cnf"]["jwk"] != crypto::jwk(agent)? {
        return Err(Error::reject("agent_key_mismatch"));
    }
    let t = crypto::verify(field(a, "terminal_jws", 32768)?, agent, "kb+sd-jwt", 32768)?;
    exact(&t, &["aud", "delegate_payload", "iat", "nonce", "sd_hash"])?;
    if t["sd_hash"] != crypto::hash(root_wire) {
        return Err(Error::reject("root_binding_mismatch"));
    }
    if t["aud"] != field(a, "expected_aud", 512)? {
        return Err(Error::reject("audience_mismatch"));
    }
    if t["nonce"] != field(a, "expected_nonce", 512)? {
        return Err(Error::reject("nonce_mismatch"));
    }
    let ti = epoch(&t["iat"])?;
    let oi = epoch(&open["iat"])?;
    let oe = epoch(&open["exp"])?;
    if ti < now.saturating_sub(300).max(oi) || ti > now || ti >= oe {
        return Err(Error::reject("invalid_purchase_time"));
    }
    let closed = &array(&t["delegate_payload"], 1, 1)?[0];
    exact(
        closed,
        &["checkout_hash", "checkout_jwt", "exp", "iat", "vct"],
    )?;
    if closed["vct"] != "mandate.checkout.1" {
        return Err(Error::reject("unsupported_purchase_profile"));
    }
    times(closed, Some(now), 7200)?;
    if closed["iat"] != open["iat"] || closed["exp"] != open["exp"] {
        return Err(Error::reject("delegation_claim_mismatch"));
    }
    let wire = field(a, "host_checkout_jws", 16384)?;
    if closed["checkout_jwt"] != wire || closed["checkout_hash"] != crypto::hash(wire) {
        return Err(Error::reject("checkout_binding_mismatch"));
    }
    let c = crypto::verify(
        wire,
        field(a, "trusted_merchant_public_key_pem", 1024)?,
        "JWT",
        16384,
    )?;
    if c["id"] != field(a, "expected_checkout_id", 256)?
        || c["merchant"]["id"] != field(a, "expected_merchant_id", 256)?
    {
        return Err(Error::reject("checkout_identity_mismatch"));
    }
    constraints::check(
        open,
        &c,
        profile,
        field(a, "expected_currency", 3)?,
        number(&a["expected_total_minor"], 0, 9_007_199_254_740_991)?,
    )?;
    Ok(
        json!({"profile_ref":profile,"checkout_hash":crypto::hash(wire),"root_presentation_digest":crypto::hex(root_wire),"valid_until":oe.min(ti+301),"checkout":c}),
    )
}
fn run(request: &Value) -> Result<Value> {
    exact(request, &["contract_version", "operation", "arguments"])?;
    if request["contract_version"] != "ap2-core-request.1" {
        return Err(Error::reject("unsupported_contract"));
    }
    let a = &request["arguments"];
    let op = field(request, "operation", 64)?;
    if op == "verify_checkout" {
        return host::verify(a);
    }
    let (code, facts, partial) = match op {
        "root_signature" => {
            exact(a, &["root_jws", "trusted_issuer_public_key_pem"])?;
            crypto::verify(
                field(a, "root_jws", 32768)?,
                field(a, "trusted_issuer_public_key_pem", 1024)?,
                "example+sd-jwt",
                32768,
            )?;
            ("root_signature_verified_only", Value::Null, true)
        }
        "root_disclosures" => {
            exact(
                a,
                &[
                    "presentation",
                    "trusted_issuer_public_key_pem",
                    "profile_ref",
                ],
            )?;
            disclosures::root(
                field(a, "presentation", 131072)?,
                field(a, "trusted_issuer_public_key_pem", 1024)?,
                field(a, "profile_ref", 256)?,
            )?;
            ("root_disclosures_verified_only", Value::Null, true)
        }
        "scoped_constraints" => ("scoped_constraints_verified_only", scoped(a)?, true),
        "checkout" => ("checkout_verified", checkout_verify(a)?, false),
        "direct" => ("direct_checkout_verified", direct_verify(a)?, false),
        "receipt" => ("receipt_verified", receipt_verify(a)?, false),
        _ => return Err(Error::reject("unsupported_operation")),
    };
    Ok(
        json!({"contract_version":"ap2-core-result.1","status":if partial{"unverified"}else{"verified"},"code":code,"verified":!partial,"facts":facts,"evidence":null}),
    )
}
/// Bounded UTF-8 JSON request. No network, clock, trust lookup or persistent state.
/// Partial checks deliberately return `unverified`; they cannot mint Evidence.
pub fn verify_json(bytes: &[u8]) -> Value {
    let result = std::panic::catch_unwind(|| json::parse(bytes).and_then(|v| run(&v)));
    match result {
        Ok(Ok(v)) => v,
        Ok(Err(e)) => failure(e),
        Err(_) => failure(Error::unavailable("internal_panic")),
    }
}
fn failure(e: Error) -> Value {
    json!({"contract_version":"ap2-core-result.1","status":e.status,"code":e.code,"verified":false,"facts":null,"evidence":null})
}
