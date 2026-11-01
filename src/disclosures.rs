use crate::{array, crypto, Error, Result, MULTI, SINGLE};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
fn err() -> Error {
    Error::reject("unsupported_disclosure_profile")
}
fn allowed(p: &str, multi: bool) -> bool {
    if p == "/delegate_payload/0" || p == "/delegate_payload/0/disclosed/constraints/1/allowed/0" {
        return true;
    }
    let parts: Vec<_> = p.split('/').skip(1).collect();
    parts.len() == 9
        && parts[..6]
            == [
                "delegate_payload",
                "0",
                "disclosed",
                "constraints",
                "0",
                "items",
            ]
        && parts[7] == "acceptable_items"
        && parts[6]
            .parse::<usize>()
            .is_ok_and(|i| i < if multi { 8 } else { 1 })
        && parts[8]
            .parse::<usize>()
            .is_ok_and(|i| i < if multi { 8 } else { 1 })
}
struct Walker {
    mapping: HashMap<String, Value>,
    used: HashSet<String>,
    paths: HashSet<String>,
    nodes: usize,
    multi: bool,
}
impl Walker {
    fn walk(&mut self, v: &Value, path: &str, depth: usize) -> Result<Value> {
        self.nodes += 1;
        if self.nodes > 2048 || depth > 24 && (v.is_object() || v.is_array()) {
            return Err(Error::reject("disclosure_limit_exceeded"));
        }
        match v {
            Value::Object(o) => {
                if o.len() > 32
                    || o.contains_key("_sd")
                    || o.contains_key("...")
                    || !path.is_empty() && o.contains_key("_sd_alg")
                {
                    return Err(err());
                }
                let mut out = serde_json::Map::new();
                for (k, v) in o {
                    out.insert(
                        k.clone(),
                        self.walk(
                            v,
                            &format!("{path}/{}", k.replace("~", "~0").replace("/", "~1")),
                            depth + 1,
                        )?,
                    );
                }
                Ok(Value::Object(out))
            }
            Value::Array(a) => {
                if a.len() > 16 {
                    return Err(Error::reject("disclosure_limit_exceeded"));
                }
                let mut out = Vec::new();
                for (i, v) in a.iter().enumerate() {
                    let p = format!("{path}/{i}");
                    if let Some(d) = v.get("...") {
                        if v.as_object().is_none_or(|o| o.len() != 1) || !allowed(&p, self.multi) {
                            return Err(err());
                        }
                        let digest = d.as_str().ok_or_else(err)?;
                        if crypto::decode(digest, 32)?.len() != 32 {
                            return Err(err());
                        }
                        if !self.used.insert(digest.into()) {
                            return Err(Error::reject("duplicate_commitment"));
                        }
                        let revealed = self
                            .mapping
                            .get(digest)
                            .ok_or_else(|| Error::reject("missing_disclosure"))?
                            .clone();
                        self.paths.insert(p.clone());
                        out.push(self.walk(&revealed, &format!("{p}/disclosed"), depth + 1)?);
                    } else {
                        out.push(self.walk(v, &p, depth + 1)?);
                    }
                }
                Ok(Value::Array(out))
            }
            _ => Ok(v.clone()),
        }
    }
}
pub fn root(wire: &str, pem: &str, profile: &str) -> Result<Value> {
    if ![SINGLE, MULTI].contains(&profile) {
        return Err(err());
    }
    let multi = profile == MULTI;
    if !wire.is_ascii() || wire.len() > 131072 {
        return Err(Error::reject("invalid_root_presentation"));
    }
    let p: Vec<_> = wire.split('~').collect();
    if p.len() < 3
        || p.len() > if multi { 68 } else { 18 }
        || p.last() != Some(&"")
        || p[..p.len() - 1].iter().any(|s| s.is_empty())
    {
        return Err(Error::reject("invalid_root_presentation"));
    }
    let payload = crypto::verify(p[0], pem, "example+sd-jwt", 32768)?;
    if payload.as_object().is_none_or(|o| {
        o.keys()
            .any(|k| !["delegate_payload", "_sd_alg"].contains(&k.as_str()))
    }) || payload.get("_sd_alg").is_some_and(|v| v != "sha-256")
    {
        return Err(err());
    }
    array(&payload["delegate_payload"], 1, 1)?;
    let mut walker = Walker {
        mapping: HashMap::new(),
        used: HashSet::new(),
        paths: HashSet::new(),
        nodes: 0,
        multi,
    };
    for d in &p[1..p.len() - 1] {
        if d.len() > 8192 {
            return Err(Error::reject("disclosure_limit_exceeded"));
        }
        let v = crypto::disclosure(d, 2, false)?;
        if !v[1].is_object() {
            return Err(err());
        }
        if walker
            .mapping
            .insert(crypto::hash(d), v[1].clone())
            .is_some()
        {
            return Err(Error::reject("duplicate_disclosure"));
        }
    }
    let resolved = walker.walk(&payload, "", 1)?;
    let mut expected = HashSet::from([
        "/delegate_payload/0".to_string(),
        "/delegate_payload/0/disclosed/constraints/1/allowed/0".to_string(),
    ]);
    let items = array(
        &resolved["delegate_payload"][0]["constraints"][0]["items"],
        1,
        if multi { 8 } else { 1 },
    )?;
    for (i, r) in items.iter().enumerate() {
        for j in 0..array(&r["acceptable_items"], 1, if multi { 8 } else { 1 })?.len() {
            expected.insert(format!(
                "/delegate_payload/0/disclosed/constraints/0/items/{i}/acceptable_items/{j}"
            ));
        }
    }
    if walker.paths != expected {
        return Err(err());
    }
    if walker.used.len() != walker.mapping.len() {
        return Err(Error::reject("extra_disclosure"));
    }
    Ok(resolved)
}
