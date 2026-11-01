use ap2_verifier_core::verify_json;
use serde_json::{json, Value};
#[test]
fn retained_baseline_cases() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("../conformance/v1/cases.json")).unwrap();
    for c in cases {
        let result=verify_json(&serde_json::to_vec(&json!({"contract_version":"ap2-core-request.1","operation":c["operation"],"arguments":c["arguments"]})).unwrap());
        assert_eq!(
            result["status"], c["expected_status"],
            "{}: {}",
            c["id"], result
        );
        // Rejections have Rust stable schema codes; exact Ruby reject-code parity is not promised.
        if c["expected_status"] != "rejected" {
            assert_eq!(result["code"], c["expected_code"], "{}", c["id"]);
        }
    }
}
#[test]
fn malformed_boundaries() {
    for bytes in [
        b"{\"a\":1,\"a\":2}".as_slice(),
        b"{\"a\":1,\"\\u0061\":2}",
        b"\xef\xbb\xbf{}",
        b"{/*comment*/}",
        b"[1,]",
        b"{\"n\":NaN}",
        b"{} {}",
        b"\xff",
    ] {
        let v = verify_json(bytes);
        assert_eq!(v["status"], "rejected");
        assert_eq!(v["evidence"], Value::Null);
    }
    assert_eq!(
        verify_json(b"{\"a\":1,\"\\u0061\":2}")["code"],
        "duplicate_key"
    );
    assert_eq!(verify_json(&vec![b' '; 131073])["code"], "input_too_large");
    let deep = format!("{}0{}", "[".repeat(25), "]".repeat(25));
    assert_eq!(verify_json(deep.as_bytes())["code"], "input_too_deep");
}
#[test]
fn unknowns_fail_closed() {
    for op in ["verify", "payments", "arbitrary"] {
        let v = verify_json(
            serde_json::to_string(
                &json!({"contract_version":"ap2-core-request.1","operation":op,"arguments":{}}),
            )
            .unwrap()
            .as_bytes(),
        );
        assert_eq!(v["code"], "unsupported_operation");
        assert_eq!(v["verified"], false);
    }
}

#[test]
fn malformed_fuzz_smoke_never_mints_evidence() {
    let mut seed = 0x123456789ABCDEF0u64;
    for n in 0..2048 {
        let mut bytes = vec![0; n % 512];
        for b in &mut bytes {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            *b = seed as u8;
        }
        let r = verify_json(&bytes);
        assert!(!r["verified"].as_bool().unwrap());
        assert!(r["evidence"].is_null());
    }
}

#[test]
fn structured_mutation_smoke_preserves_result_contract() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("../conformance/v1/cases.json")).unwrap();
    let mut seed = 0x394d37fe357902f1u64;
    for case in cases.iter().filter(|v| v["expected_status"] != "rejected") {
        let original = serde_json::to_vec(&json!({"contract_version":"ap2-core-request.1","operation":case["operation"],"arguments":case["arguments"]})).unwrap();
        for _ in 0..16 {
            let mut bytes = original.clone();
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let i = (seed as usize) % bytes.len();
            bytes[i] ^= (seed >> 32) as u8 | 1;
            let result = verify_json(&bytes);
            assert_eq!(result["contract_version"], "ap2-core-result.1");
            if result["status"] == "rejected" || result["status"] == "unavailable" {
                assert!(result["evidence"].is_null());
                assert!(result["facts"].is_null());
            }
            if result["status"] == "unverified" {
                assert!(result["evidence"].is_null());
            }
        }
    }
}
