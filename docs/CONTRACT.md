# Experimental API and C ABI v1

`verify_json(&[u8]) -> serde_json::Value` accepts original UTF-8 JSON:

```json
{"contract_version":"ap2-core-request.1","operation":"checkout","arguments":{"jwt":"...","trusted_public_key_pem":"...","expected_merchant_id":"shop","profile_ref":"ap2.checkout.single_merchant_multi_item"}}
```

Unknown/missing request and operation arguments reject. No implicit clock, network,
key discovery, token-selected trust, environment-selected policy or durable state.
The current **entire request** is limited to 131072 bytes (including JSON escaping),
which is stricter than the Gem's separate per-input limits. This is an intentional
experimental transport limit, not complete input-size parity. Keys are canonical
public SPKI PEM for P-256 only, maximum 1024 bytes. ES256 JOSE signature is 64 bytes;
original JWS signing input is used without claim reserialization. High-S and low-S
ECDSA signatures retain the baseline's mathematical acceptance range.

Operations and required arguments are enumerated in `src/lib.rs` / `src/host.rs`.
`verify_checkout` accepts the same seven named inputs as Ruby's complete core;
`scoped_constraints` takes explicit keys/order/challenge context and `profile_ref`.
`root_signature` and `root_disclosures` are partial checks. Explicit profile required
where applicable; no default that could silently change semantics.

Results always use `ap2-core-result.1` with `status`, `code`, `verified`, `facts`,
`evidence`. `verified_research/scoped_research_verified` is produced only after full
Open→Closed + independent Host inputs. `verified` for Direct/Checkout/Receipt means
only that named operation succeeded. `unverified` partial results never authorize
or produce Evidence. `rejected` and `unavailable` have false `verified` and null
facts/Evidence; panic maps to unavailable/internal_panic. Reject codes are stable
Rust codes, not a promise of exact Ruby exception/code parity. No raw parser errors
or input values appear in failures. Unknown, missing, malformed, process/ABI failure
and missing response all fail closed.

Evidence is a distinct `ap2.core-evidence.1` JSON projection, not Ruby's private
ResearchEvidence object or an authenticated credential. Ruby freezes all returned
values. Serialized Evidence can be forged; only use the result of the local call
in the same trusted process. Checked facts are not Host ALLOW or payment success.
An authenticated Error Receipt is verified while `purchase_succeeded` is false.

C header: `include/ap2_verifier.h`. `ap2_verify_v1` retains no pointers and allocates
no memory for the caller to free. Caller supplies readable initialized input and a
non-overlapping writable output buffer; lengths are byte lengths. Return >0 is
written JSON length without NUL; <0 is required capacity, with output untouched;
0 is ABI/internal failure. Maximum output capacity is 262144 bytes. Null output
is permitted only for zero-capacity query. Input/output lifetime, alignment for
bytes and valid pointers are caller obligations: dangling pointers, aliasing or
lying lengths are undefined behavior, not catchable panic. FFI catches unwind,
not abort, OOM, hardware faults or malicious in-process native code. Ruby allocates
one output buffer per call with Fiddle::RUBY_FREE; no shared mutable Rust state.

Host must independently authenticate and refresh registry/profile snapshots,
issuer/provider/agent/merchant enrollment, revocation, order/challenge/clock.
Before acting: recheck latest state/validity, atomically consume nonce with order
CAS, handle uncertain effects, then decide authorization/execution/payment.
The verifier never creates or consumes a nonce, authenticates storage or approves
an actual purchase. Real Issuer interoperability and production authorization are
unproved. No general AP2 conformance, full Schema support or performance guarantee.
