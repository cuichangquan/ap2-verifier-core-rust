use crate::{Error, Result};
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value};
use std::fmt;

struct Seed(usize);
impl<'de> DeserializeSeed<'de> for Seed {
    type Value = Value;
    fn deserialize<D: de::Deserializer<'de>>(self, d: D) -> std::result::Result<Value, D::Error> {
        d.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for Seed {
    type Value = Value;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("strict JSON")
    }
    fn visit_bool<E: de::Error>(self, v: bool) -> std::result::Result<Value, E> {
        Ok(Value::Bool(v))
    }
    fn visit_i64<E: de::Error>(self, v: i64) -> std::result::Result<Value, E> {
        Ok(Value::Number(v.into()))
    }
    fn visit_u64<E: de::Error>(self, v: u64) -> std::result::Result<Value, E> {
        Ok(Value::Number(v.into()))
    }
    fn visit_f64<E: de::Error>(self, v: f64) -> std::result::Result<Value, E> {
        Number::from_f64(v)
            .map(Value::Number)
            .ok_or_else(|| E::custom("invalid_json"))
    }
    fn visit_str<E: de::Error>(self, v: &str) -> std::result::Result<Value, E> {
        Ok(Value::String(v.into()))
    }
    fn visit_string<E: de::Error>(self, v: String) -> std::result::Result<Value, E> {
        Ok(Value::String(v))
    }
    fn visit_none<E: de::Error>(self) -> std::result::Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_unit<E: de::Error>(self) -> std::result::Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> std::result::Result<Value, A::Error> {
        if self.0 >= 24 {
            return Err(de::Error::custom("input_too_deep"));
        }
        let mut out = Vec::new();
        while let Some(v) = a.next_element_seed(Seed(self.0 + 1))? {
            out.push(v)
        }
        Ok(Value::Array(out))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> std::result::Result<Value, A::Error> {
        if self.0 >= 24 {
            return Err(de::Error::custom("input_too_deep"));
        }
        let mut out = Map::new();
        while let Some(k) = a.next_key::<String>()? {
            if out.contains_key(&k) {
                return Err(de::Error::custom("duplicate_key"));
            }
            out.insert(k, a.next_value_seed(Seed(self.0 + 1))?);
        }
        Ok(Value::Object(out))
    }
}
pub fn parse(bytes: &[u8]) -> Result<Value> {
    if bytes.len() > 131072 {
        return Err(Error::reject("input_too_large"));
    }
    std::str::from_utf8(bytes).map_err(|_| Error::reject("invalid_encoding"))?;
    let mut d = serde_json::Deserializer::from_slice(bytes);
    let v = Seed(0).deserialize(&mut d).map_err(|e| {
        let m = e.to_string();
        Error::reject(if m.starts_with("duplicate_key") {
            "duplicate_key"
        } else if m.starts_with("input_too_deep") {
            "input_too_deep"
        } else {
            "invalid_json"
        })
    })?;
    d.end().map_err(|_| Error::reject("invalid_json"))?;
    Ok(v)
}
