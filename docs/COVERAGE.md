# Coverage and limitations

Baseline: Gem 1.0.0 source `59e344ad865ed867d0993120456f9411d82bcdec`;
AP2 v0.2.0 normative/official SDK commit `b4587ac1d055888a73b4b21750973cffba961793`.
Runtime source code is adapted, not a dependency on invoking Ruby. No source Gem
files or released API behavior were changed.

| Gem capability | Rust / local Ruby adapter | Evidence boundary |
| --- | --- | --- |
| InputValidator / legacy syntax `verify` | Strict parser internally; legacy entry intentionally absent | Duplicate keys including escaped duplicates, UTF-8, depth/bytes; no syntax success Evidence |
| RootSignature | `root_signature` | Partial `unverified` only |
| RootDisclosures | `root_disclosures`, both profiles | Complete supported commitment closure; partial `unverified` |
| PurchaseBinding / ScopedConstraints | `scoped_constraints` | Exact signatures, cnf, aud/nonce/time, wire hashes, merchant/order identity; partial `unverified` |
| Legacy single-SKU v1 | Full `verify_checkout` | One requirement/candidate/line; titles and quantities match |
| Multi-item v1 | Full `verify_checkout` | 1–8 lines/requirements/candidates, repeated product IDs on distinct lines, integer max flow, exact demand/supply |
| TrustContext / HostBinding / ResearchResult | `verify_checkout`, same seven inputs | Canonical profile body/hash, immutable snapshot bytes, all records/key fingerprints, unique selected roles/windows, challenge/order binding and validity intersection |
| Buyer Direct verification | `direct` | Provider + Merchant signatures, exact two-disclosure grammar, original order bytes, 600s limit, checked facts only |
| CheckoutJWT verification | `checkout` | Merchant signature, strict checkout profile and expected identity |
| CheckoutReceipt verification | `receipt` | Whole final wire reference, trusted issuer, explicit time window, Success vs Error distinction |
| Conditional issuance / Buyer issuance / merchant signing | Remain Ruby-only; their output verifies in Rust | No Rust key generation, issuance or signing API |
| InputBuilder assembly / saved formats | Remain Ruby-only; seven-input output accepted | No new persistence format or automatic preparation during verification |
| Diagnostics / Ruby exception classes | No parity promised | Stable Rust status/code JSON, no input leakage |

Important differences: 131072-byte whole JSON transport limit includes escaping;
explicit profile selection; Rust reject code names need not match Gem codes;
Evidence uses a separate versioned JSON contract, not Ruby's private object. These
prevent claiming drop-in replacement. No arbitrary algorithms, additional hops,
undisclosed constraints, tax/shipping/discounts, external links, currencies beyond
USD/EUR/JPY, or real trust enrollment. Unknowns reject.

## Validation

- 61 retained cases: 42 historical Host/verification inputs, 10 newly re-signed
  causal negatives, multi-item max-flow and Host positives, single Host positive,
  Direct, Checkout and both Receipt outcomes across Direct and multi-item wires.
- Rust tests include strict parser limits/duplicates/malformed encodings, unknown
  operations/contracts, 2048 deterministic malformed-input smoke cases, caller
  output size queries/guard bytes/null-shape handling. This is not exhaustive fuzzing.
- Ruby FFI tests replay cases, mutate Host order/challenge/wires, and make 200 calls
  from 8 threads; the source Gem is never loaded. Both MRI versions tested below.
- Independent oracle uses pinned official SD-JWT verify/create and cryptography
  ES256 checks; original signatures and hashes, official-issued Direct accepted by
  Rust, re-signed bad checkout schemas rejected. Valid causal-negative signatures
  are independently checked. Oracle does not calculate expected results from Ruby.
  Fixture design/profile assumptions are shared; not independent certification.
- Rustfmt, Clippy `-D warnings`, locked offline build/test. No CI added/triggered.

| Target / runtime | State |
| --- | --- |
| aarch64-unknown-linux-gnu in Docker Desktop on Apple Silicon | Locally tested; dev and release build |
| Ruby MRI 3.4.12 Linux/arm64 Fiddle | Locally tested |
| Ruby MRI 4.0.9 Linux/arm64 Fiddle | Locally tested |
| x86_64-unknown-linux-gnu + both MRIs in Docker on Apple Silicon | Clean build/tests observed under emulation; native x86 hardware NOT_RUN |
| Clean pinned independent Python oracle, both Linux architectures | PASS; old task image no longer required |
| Strict-provenance Miri / AddressSanitizer Linux/arm64 | PASS for scoped FFI library / all Rust tests respectively |
| macOS arm64/x86_64 native dylib | NOT_RUN |
| Linux musl | NOT_RUN |
| Windows DLL / MSVC | NOT_RUN |
| Sustained coverage-guided fuzzing / load / memory profiling | NOT_RUN |
| ABI across untrusted in-process native code / dangling caller pointers | Not supported; caller memory contract |

Observed development targets are Linux/arm64 and emulated Linux/x86_64. No universal native packages,
performance SLA, real issuer interoperability or production GO. Resource limits
bound request/graph/arrays/depth; they do not prove a wall-time/RSS upper bound.
Publication remains disabled (`publish = false`) pending separate Owner approval
and a separately scoped platform/distribution decision. Do not ship local debug
or release binaries as public artifacts.

The follow-up [reproducibility/safety record](REPRODUCIBILITY.md) gives exact source,
tools and scope for these new observations. The PR #1 evidence remains historical and unchanged.
[Candidate criteria](RELEASE_READINESS.md) distinguish these checks from replacement
or publication approval.

The [candidate report](CANDIDATE_REPORT.md) adds exact extracted-package/native
validation and the patched p256 0.14.0 / crypto-bigint 0.7.10 stack. Current MSRV is
1.88 (observed on Linux/arm64); x86 candidate testing used 1.90. Earlier 1.85/1.87
observations are historical and not current support promises. The independent
oracle now additionally checks both valid ES256 high-S and low-S signatures.
