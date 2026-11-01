# Local validation record — 2026-11-02 JST

Implementation commit: `9c8ecc778924b7c7a92924a743204d76573a2731`.
The subsequent evidence-only commit adds these logs/records without changing code.

| Check | Observed result |
| --- | --- |
| Rustfmt / Clippy all-targets with warnings denied | PASS |
| Rust debug and release | 1 ABI unit test + 4 integration tests each PASS; 61 conformance cases inside test |
| Parser malformed smoke | 2048 deterministic byte sequences, no verified/Evidence |
| MRI 3.4.12 Linux/arm64, direct Rust FFI | 85 checks + 200 calls / 8 threads PASS |
| MRI 4.0.9 Linux/arm64, direct Rust FFI | 85 checks + 200 calls / 8 threads PASS |
| Source Gem complete-core comparison | 54 cases, status + successful Evidence projection match |
| Fixture manifest SHA256 | PASS |
| Independent official SDK + cryptography | 47 basic + 40 causal signature/binding + 4 exact-quantity checks PASS |
| README local Ruby→Rust release-library example | verified_research/scoped_research_verified; evidence=true |

Raw bounded test output: [Rust/Ruby](validation/rust-and-ruby.log),
[independent oracle](validation/independent-oracle.log),
[source comparison](validation/ruby-baseline.log).

Rust: official `rust:1.90.0-bookworm` manifest digest
`sha256:3914072ca0c3b8aad871db9169a651ccfce30cf58303e5d6f2db16d1d8a7e58f`;
Cargo.lock fixes all transitive dependencies. Local cache image keeps dependencies
and official rustfmt/clippy in Docker. Runs after preparation used `--network none`,
2 CPUs / at most 2 GiB memory per run. Host is Apple Silicon with Docker Desktop
Linux/arm64; no macOS toolchain was installed. Test processes/containers use `--rm`.
Local build images and repository `target/` cache remain for reproducible continuation.
No deployment, Actions workflow or public binary artifacts.

Independent Python environment was reused, with no new Python dependencies:
local image `ap2-merchant-interop:task3`, ID
`sha256:343795b18b60e2af526034a74995bee53c7c9f1ea4203972c377f7f52678ccd4`,
Python 3.11.2, cryptography 46.0.5, jwcrypto 1.5.6, sd-jwt 0.10.4,
pydantic 2.12.5, jsonschema 4.26.0. The exact official SDK checkout was mounted
read-only at `/official`, PYTHONPATH `/official/code/sdk/python`; run
`/opt/ap2-python/bin/python tests/independent_oracle.py` in `/work` with the Rust
repository mounted read-only. This local image is not a published prerequisite.
For reproduction outside this machine, provision these pinned dependencies in an
isolated container and the official pinned source; that clean provision is NOT_RUN.

Source comparison uses existing pinned Ruby dependency image with source mounted
read-only at `/baseline`: `ruby tests/ruby_baseline.rb`. `scripts/validate.sh`
reproduces Rust and Gem-free Ruby FFI checks. Source export scripts are not rerun
for routine tests; the suite is frozen and digest-bound.

Self-audit: no runtime Ruby process invocation, dynamic key lookup, network/state,
unsafe blocks outside ABI, raw failure text, private key files or nonce in Evidence.
Strict keys/limits at request/JSON/JWS/disclosure/constraint/trust/Host boundaries.
Graph path keys escape JSON Pointer components to avoid slash-key aliases. Native
output is caller-owned, one buffer per call, no retained pointers/shared mutable
state. C caller pointer UB and OOM/abort are outside panic containment.

Independent Codex review on the implementation commit (2026-11-02 JST) found no
actionable correctness/security defects in the documented experimental scope.
This is limited source review, not certification. PR final-HEAD review and merge
status are recorded on the PR rather than creating a self-referential commit.

An early extraction attempt called a partial API for full seven-input fixtures;
it was corrected before freezing the suite. Missing rustfmt was added only inside
Docker. An initial FFI adapter oversized-request exception was changed to the
structured rejected result before final validation. None was treated as a product
PASS while failing. No permission denial was bypassed.

Remaining limits: [platform/test gaps](COVERAGE.md), [API differences](CONTRACT.md).
Unpublished, `publish = false`; no release tag/assets/GitHub Release/cargo publish.
Owner approval remains required for any library release.
