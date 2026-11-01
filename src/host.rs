//! Authenticated Host assertions remain a caller obligation. Hashes bind bytes,
//! not their provenance. No challenge is issued, fetched or consumed here.
use crate::{array, crypto, epoch, exact, field, number, text, Error, Result, MULTI, SINGLE};
use serde_json::{json, Value};
const REFS: [&str; 4] = [
    "trust_snapshot_id",
    "policy_version",
    "trust_content_hash",
    "profile_definition_hash",
];
const IDS: [&str; 8] = [
    "tenant_id",
    "buyer_account_id",
    "order_id",
    "merchant_id",
    "audience",
    "profile_ref",
    "order_version",
    "profile_version",
];
const HASHES: [&str; 3] = [
    "approved_agent_key_fingerprint",
    "root_presentation_digest",
    "checkout_jwt_digest",
];
const RECORD: [&str; 13] = [
    "role",
    "identity",
    "tenant_id",
    "purpose",
    "key_ref",
    "profile_ref",
    "profile_version",
    "public_key_pem",
    "key_fingerprint",
    "algorithm",
    "active",
    "not_before",
    "not_after",
];
fn err() -> Error {
    Error::reject("invalid_trust_context")
}
fn reject(code: &'static str) -> Error {
    Error::reject(code)
}
fn digest(v: &Value) -> Result<()> {
    let s = text(v, 64)?;
    if s.len() != 64
        || !s
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(err());
    }
    Ok(())
}
fn window(
    v: &Value,
    now: u64,
    inactive: &'static str,
    expired: &'static str,
) -> Result<(u64, u64)> {
    let active = v["active"].as_bool().ok_or_else(err)?;
    let start = epoch(&v["not_before"])?;
    let end = epoch(&v["not_after"])?;
    if start >= end {
        return Err(err());
    }
    if !active {
        return Err(reject(inactive));
    }
    if start > now || now >= end {
        return Err(reject(expired));
    }
    Ok((start, end))
}
fn envelope(v: &Value, ids: &[&str], hash: &str, max: usize) -> Result<Value> {
    let mut fields = ids.to_vec();
    fields.extend([hash, "body"]);
    exact(v, &fields)?;
    digest(&v[hash])?;
    let wire = field(v, "body", max)?;
    if crypto::hex(wire) != v[hash] {
        return Err(reject("trust_context_mismatch"));
    }
    let body = crate::json::parse(wire.as_bytes())?;
    for k in ids {
        text(&v[*k], 256)?;
        if body[*k] != v[*k] {
            return Err(reject("trust_context_mismatch"));
        }
    }
    Ok(body)
}
struct Context {
    selection: Value,
    selected: Vec<Value>,
    keys: Vec<String>,
    start: u64,
    end: u64,
}
fn context(trust: &str, selection: &str, now: u64, merchant: &Value) -> Result<Context> {
    let t = crate::json::parse(trust.as_bytes())?;
    let s = crate::json::parse(selection.as_bytes())?;
    exact(&t, &["snapshot", "profile"])?;
    exact(
        &s,
        &[
            "trust_snapshot_id",
            "policy_version",
            "trust_content_hash",
            "profile_ref",
            "profile_version",
            "profile_definition_hash",
            "tenant_id",
            "Issuer",
            "Agent",
            "Merchant",
        ],
    )?;
    for k in [
        "trust_snapshot_id",
        "policy_version",
        "profile_ref",
        "profile_version",
        "tenant_id",
    ] {
        text(&s[k], 256)?;
    }
    for k in ["trust_content_hash", "profile_definition_hash"] {
        digest(&s[k])?;
    }
    for role in ["Issuer", "Agent", "Merchant"] {
        exact(&s[role], &["identity", "purpose", "key_ref"])?;
        for k in ["identity", "purpose", "key_ref"] {
            text(&s[role][k], 256)?;
        }
    }
    if s["Merchant"]["identity"] != *merchant {
        return Err(reject("trust_context_mismatch"));
    }
    let snapshot = envelope(
        &t["snapshot"],
        &["id", "policy_version"],
        "content_hash",
        65536,
    )?;
    exact(
        &snapshot,
        &[
            "id",
            "policy_version",
            "active",
            "not_before",
            "not_after",
            "records",
        ],
    )?;
    for (a, b) in [
        ("id", "trust_snapshot_id"),
        ("policy_version", "policy_version"),
        ("content_hash", "trust_content_hash"),
    ] {
        if t["snapshot"][a] != s[b] {
            return Err(reject("trust_context_mismatch"));
        }
    }
    let (mut start, mut end) = window(
        &snapshot,
        now,
        "inactive_trust_snapshot",
        "trust_snapshot_time_mismatch",
    )?;
    envelope(
        &t["profile"],
        &["profile_ref", "version"],
        "definition_hash",
        8192,
    )?;
    let profile = field(&s, "profile_ref", 256)?;
    let canonical = match profile {
        SINGLE => include_str!("../conformance/v1/single_profile.json"),
        MULTI => include_str!("../conformance/v1/multi_profile.json"),
        _ => return Err(reject("unsupported_trust_profile")),
    };
    if t["profile"]["body"] != canonical
        || t["profile"]["profile_ref"] != profile
        || t["profile"]["version"] != "1"
        || s["profile_version"] != "1"
        || t["profile"]["definition_hash"] != crypto::hex(canonical)
        || s["profile_definition_hash"] != crypto::hex(canonical)
    {
        return Err(reject("unsupported_trust_profile"));
    }
    let records = array(&snapshot["records"], 1, 16)?;
    for r in records {
        exact(r, &RECORD)?;
        for k in [
            "identity",
            "tenant_id",
            "purpose",
            "key_ref",
            "profile_ref",
            "profile_version",
        ] {
            text(&r[k], 256)?;
        }
        if !["Issuer", "Agent", "Merchant"].contains(&field(r, "role", 256)?) {
            return Err(err());
        }
        // Validate even unselected windows/keys without requiring them active.
        r["active"].as_bool().ok_or_else(err)?;
        if epoch(&r["not_before"])? >= epoch(&r["not_after"])? {
            return Err(err());
        }
        if r["profile_ref"] != profile || r["profile_version"] != "1" || r["algorithm"] != "ES256" {
            return Err(reject("unsupported_trust_profile"));
        }
        digest(&r["key_fingerprint"])?;
        let pem = field(r, "public_key_pem", 1024)?;
        crypto::key(pem).map_err(|_| reject("invalid_trust_key"))?;
        if r["key_fingerprint"] != crypto::hex(pem) {
            return Err(reject("invalid_trust_key"));
        }
    }
    let mut keys = Vec::new();
    let mut selected = Vec::new();
    for role in ["Issuer", "Agent", "Merchant"] {
        let matches: Vec<_> = records
            .iter()
            .filter(|r| {
                r["role"] == role
                    && r["tenant_id"] == s["tenant_id"]
                    && ["identity", "purpose", "key_ref"]
                        .iter()
                        .all(|k| r[*k] == s[role][*k])
            })
            .collect();
        if matches.is_empty() {
            return Err(reject("trust_key_not_found"));
        }
        if matches.len() != 1 {
            return Err(reject("ambiguous_trust_key"));
        }
        let r = matches[0];
        let (lo, hi) = window(r, now, "inactive_trust_key", "trust_key_time_mismatch")?;
        start = start.max(lo);
        end = end.min(hi);
        keys.push(field(r, "public_key_pem", 1024)?.to_string());
        let mut projection = r.clone();
        projection
            .as_object_mut()
            .ok_or_else(err)?
            .remove("public_key_pem");
        selected.push(projection);
    }
    Ok(Context {
        selection: s,
        selected,
        keys,
        start,
        end,
    })
}
fn binding(v: &Value) -> Result<()> {
    let mut fields = IDS.to_vec();
    fields.extend(HASHES);
    fields.extend(["currency", "expected_total"]);
    exact(v, &fields)?;
    for k in IDS {
        text(&v[k], 256)?;
    }
    for k in HASHES {
        digest(&v[k])?;
    }
    if !["USD", "EUR", "JPY"].contains(&field(v, "currency", 3)?) {
        return Err(reject("invalid_host_context"));
    }
    number(&v["expected_total"], 0, 9_007_199_254_740_991)?;
    Ok(())
}
pub fn verify(a: &Value) -> Result<Value> {
    exact(
        a,
        &[
            "trust_json",
            "selection_json",
            "host_json",
            "root_presentation",
            "terminal_jws",
            "host_checkout_jws",
            "verified_at",
        ],
    )?;
    let now = epoch(&a["verified_at"])?;
    let root = field(a, "root_presentation", 131072)?;
    let terminal = field(a, "terminal_jws", 32768)?;
    let checkout = field(a, "host_checkout_jws", 16384)?;
    let host = crate::json::parse(field(a, "host_json", 32768)?.as_bytes())?;
    exact(
        &host,
        &[
            "binding",
            "challenge",
            "expected_challenge_issuer_id",
            "clock_source_id",
        ],
    )?;
    field(&host, "expected_challenge_issuer_id", 256)?;
    field(&host, "clock_source_id", 256)?;
    let b = &host["binding"];
    let c = &host["challenge"];
    binding(b)?;
    exact(
        c,
        &[
            "challenge_id",
            "nonce",
            "issuer_id",
            "B",
            "not_before",
            "not_after",
            "state",
            "trust_snapshot_id",
            "policy_version",
            "trust_content_hash",
            "profile_definition_hash",
        ],
    )?;
    binding(&c["B"])?;
    for k in [
        "challenge_id",
        "nonce",
        "issuer_id",
        "trust_snapshot_id",
        "policy_version",
    ] {
        text(&c[k], 256)?;
    }
    for k in ["trust_content_hash", "profile_definition_hash"] {
        digest(&c[k])?;
    }
    let cb = epoch(&c["not_before"])?;
    let ce = epoch(&c["not_after"])?;
    if cb >= ce {
        return Err(reject("invalid_host_context"));
    }
    if c["state"] != "issued" {
        return Err(reject("challenge_state_mismatch"));
    }
    if c["B"] != *b || c["issuer_id"] != host["expected_challenge_issuer_id"] {
        return Err(reject("host_binding_mismatch"));
    }
    if now < cb || now >= ce {
        return Err(reject("challenge_time_mismatch"));
    }
    let ctx = context(
        field(a, "trust_json", 131072)?,
        field(a, "selection_json", 16384)?,
        now,
        &b["merchant_id"],
    )?;
    for k in REFS {
        if c[k] != ctx.selection[k] {
            return Err(reject("challenge_reference_mismatch"));
        }
    }
    for k in ["tenant_id", "profile_ref", "profile_version"] {
        if b[k] != ctx.selection[k] {
            return Err(reject("host_binding_mismatch"));
        }
    }
    if b["approved_agent_key_fingerprint"] != ctx.selected[1]["key_fingerprint"]
        || b["root_presentation_digest"] != crypto::hex(root)
        || b["checkout_jwt_digest"] != crypto::hex(checkout)
    {
        return Err(reject("host_binding_mismatch"));
    }
    let checked = crate::scoped(
        &json!({"root_presentation":root,"terminal_jws":terminal,"host_checkout_jws":checkout,
        "trusted_issuer_public_key_pem":ctx.keys[0],"expected_agent_public_key_pem":ctx.keys[1],"trusted_merchant_public_key_pem":ctx.keys[2],
        "expected_aud":b["audience"],"expected_nonce":c["nonce"],"verified_at":now,"expected_checkout_id":b["order_id"],"expected_merchant_id":b["merchant_id"],
        "expected_currency":b["currency"],"expected_total_minor":b["expected_total"],"profile_ref":ctx.selection["profile_ref"]}),
    )?;
    // Re-read already authenticated terminal for validity/projection only.
    let t = crypto::verify(terminal, &ctx.keys[1], "kb+sd-jwt", 32768)?;
    let closed = &t["delegate_payload"][0];
    let start = ctx
        .start
        .max(cb)
        .max(epoch(&closed["iat"])?)
        .max(epoch(&t["iat"])?);
    let end = ctx
        .end
        .min(ce)
        .min(epoch(&closed["exp"])?)
        .min(epoch(&t["iat"])? + 301);
    if start > now || now >= end {
        return Err(reject("invalid_host_validity"));
    }
    let mut challenge = c.clone();
    let o = challenge.as_object_mut().ok_or_else(err)?;
    o.remove("nonce");
    o.insert(
        "verified_nonce_sha256".into(),
        json!(crypto::hex(field(&t, "nonce", 512)?)),
    );
    let mut evidence = json!({"contract_version":"ap2.core-evidence.1","implementation_revision":"ap2.core-rust.1","revision_kind":"static_identifier_not_runtime_attestation",
        "profile_ref":ctx.selection["profile_ref"],"profile_version":ctx.selection["profile_version"],"profile_definition_hash":ctx.selection["profile_definition_hash"],
        "verified_at":now,"clock_source_id":host["clock_source_id"],"valid_from":start,"valid_until":end,
        "trust_snapshot":{"id":ctx.selection["trust_snapshot_id"],"policy_version":ctx.selection["policy_version"],"content_hash":ctx.selection["trust_content_hash"]},
        "selected_keys":ctx.selected,"checkout_binding":b,"verified_challenge":challenge,"root_presentation_digest":crypto::hex(root),"checkout_jwt_digest":crypto::hex(checkout),"terminal_digest":crypto::hex(terminal)});
    // Intentional different Evidence contract: serialized facts are forgeable,
    // not credentials or a reconstruction of the Ruby private Evidence class.
    evidence["verified_scope"] = json!([
        "strict_inputs",
        "host_trust_profile_input_checks",
        "issuer_root_signature",
        "original_root_graph_closure",
        "agent_key_confirmation",
        "agent_terminal_signature",
        "closed_original_checkout_binding",
        "merchant_checkout_signature",
        "fixed_subset_schema_constraints",
        "host_challenge_order_binding",
        "validity_intersection"
    ]);
    Ok(
        json!({"contract_version":"ap2-core-result.1","status":"verified_research","code":"scoped_research_verified","verified":true,"facts":checked,"evidence":evidence}),
    )
}
