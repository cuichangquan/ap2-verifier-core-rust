# Candidate readiness and replacement boundaries

This phase prepares repeatable local validation. It does not authorize publication,
assert production readiness or replace the completed Ruby SDK. `publish = false`
remains unchanged; no release tag, public native artifact or GitHub Release.

## Scope that must remain explicit

| Concern | Current contract / requirement before broader replacement |
| --- | --- |
| Input sizes | Rust's whole JSON request <=131072 bytes, including escaping. The Gem separately permits root/trust <=131072, selection/checkout <=16384, Host/terminal <=32768. A valid seven-input Gem request can exceed this transport. Full range parity needs a separately reviewed transport design and boundary evidence; no limit silently relaxed here. |
| Partial checks | Signature/disclosure/constraint checks remain `unverified`, cannot mint Evidence. Preserve this boundary in every wrapper. |
| Result/Evidence | Rust has versioned JSON statuses/codes and `ap2.core-evidence.1`; not Ruby's exceptions/private immutable class or authenticated credential. A drop-in adapter needs deliberate mapping and invalid-input/unavailable parity tests. |
| Issuance and preparation | Signing/ConditionalCheckout/Buyer issuance and InputBuilder/saved format assembly remain in Ruby. Only verification moved. |
| Profiles | Exactly the three documented profiles; all disclosed candidates, closed-world grammar, original-wire byte hashes. No generic AP2/Schema support. |
| Trust/state | Host authenticates current registry, policy, enrollment/revocation, order/challenge and clock; durable nonce CAS and uncertain execution remain external. No verifier-only production GO. |
| Platform | Observed Docker Linux targets only; emulated amd64 must be identified as such. macOS/Windows/musl remain separate NOT_RUN gates. |
| Memory safety | Miri/ASan and bounded smoke cover valid caller memory and specific bad range shapes. Dangling pointers/allocation lies/hostile native code, OOM/abort, sustained fuzzing/load are not proved. |

## Requirements before requesting a release decision

1. Select initial version/API stability and explicitly limited supported target set.
2. Repeat clean validation from reviewed commit; retain fixture/input hashes,
   independent normative/SDK evidence and exact toolchain/image/dependency pins.
3. Review native source packaging, crate contents, platform library naming/loading,
   licensing/SBOM/dependency advisories and distribution checksums. No universal
   binary promise from two Linux Docker runs.
4. Resolve applicable substantive review findings; inspect required checks and
   protections; obtain latest-HEAD independent review. Review is not certification.
5. Decide whether the initial crate is experimental or a supported replacement;
   keep transport/result/SDK gaps explicit. MSRV 1.85 is declared but still needs
   its own validation or revised support statement before an MSRV guarantee.
6. Owner approval for exact publication actions/artifacts/registry/version/tag.
   No `cargo publish`, public binaries, tags or GitHub Release before that approval.

CI is deliberately local in this phase: no GitHub workflow or Actions spend is
introduced. Scripts are suitable for a separately approved future CI job. They
fetch dependencies only in build preparation; all test containers disable network.
Build caches are allowed; "clean" means no previous source `target/`, old task
oracle environment or undeclared source checkout is required. It does not claim
bit-identical `.so` bytes or independent supply-chain audit.
