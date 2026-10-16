# AP2 Verifier Core (Rust)

Rust-based AP2 verification core with Ruby FFI integration, designed for secure, reusable, cross-language mandate verification.

> **Status: Vision / planning only.** No Rust implementation has started. The existing Ruby verifier project remains the immediate priority.

<img width="1448" height="1086" alt="RustコアとRuby FFIの構成図" src="https://github.com/user-attachments/assets/8ccd3b1d-c0f0-4d0f-ac58-782f627809f1" />

## Vision

Implement the AP2 v0.2 mandate verification rules once in a reusable **Rust core**, with language-specific adapters.

```text
Ruby Gem  -- FFI adapter ----+
Python    -- future adapter -+--> Rust AP2 Verifier Core --> structured ACCEPT / REJECT
TypeScript-- future adapter -+
Go        -- future adapter -+
```

The first integration target is **Ruby Gem + Rust FFI**, embedded locally without requiring a separate verification server. The exact FFI design, native packaging, and API contract remain to be decided.

## Intended responsibilities

- Rust core: strict input parsing, signature and SD-JWT disclosure verification, mandate constraints, delegation-chain checks, and fail-closed results, according to the verified AP2 v0.2 profile.
- Language adapters: ergonomic SDK interfaces, input/output conversion, and native packaging for the target runtime.
- Host application: provision trusted issuers/keys and policy, and implement external stateful services such as persistent nonce tracking. Trust decisions must not silently default to trusting arbitrary inputs.

This is a **target architecture**, not a claim that these capabilities are implemented.

## Reuse the existing Ruby research

Source project: [ap2-mandate-verifier](https://github.com/cuichangquan/ap2-mandate-verifier).

Reuse, after checking provenance and current validity:
- AP2 specification research and security boundaries.
- Language-neutral fixtures, including signed positive/negative and boundary cases.
- Independent oracle and documented expected outcomes.
- Threat models and fail-closed acceptance rules.

Do **not** treat agreement between Ruby and Rust alone as proof of correctness. Retain independent signature/fixture verification and Rust-specific security tests, including unsafe/FFI boundaries, panic/error handling, memory/resource limits, and cross-platform packaging.

## Suggested development sequence

1. Finish the current Ruby verifier and identify which research results are verified.
2. Extract a versioned, language-neutral conformance test suite with input fixtures, trust configuration, and expected results.
3. Define the Rust core API, error codes, and ABI/FFI boundary.
4. Implement a small Rust core slice and compare against the independent oracle and Ruby baseline.
5. Build the Ruby FFI adapter; test distribution on supported platforms.
6. Consider Python, TypeScript/WASM, and Go adapters only when justified by demand.

## Non-goals for now

- No immediate rewrite or replacement of the Ruby Gem.
- No Rust code, CI, external downloads, paid infrastructure, or publishing until separately decided.
- No premature assertion of full AP2 v0.2 compliance.

## Design decisions to revisit

- Rust core as a native library vs other delivery forms.
- FFI approach for Ruby (for example, a native extension or explicit C ABI).
- Canonical fixtures/schema/versioning and independent oracle.
- Trusted key and policy injection, nonce storage boundary, and error/result model.
- Platform support, builds, release signing, and security audit scope.
