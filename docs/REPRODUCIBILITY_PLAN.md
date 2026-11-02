# Follow-up validation scope

Owner authorized continuation: reproducibility and FFI safety, no new APIs or
publication. Preserve PR #1 evidence; do not replace historical records with new
claims. Cryptographic/profile/acceptance rules remain unchanged.

- Clean committed-source archive in a fresh temporary tree; separate target tree
  per architecture; fixed official Rust/Ruby/Python image digests.
- Independent oracle: official PyPI only, exact 16 packages and wheel hashes;
  no source builds, automatic upgrades or unpinned dependencies. Fixed official
  AP2 source archive (commit + SHA256). Runtime validation uses no network.
- Rust debug/release, strict Clippy/fmt; Ruby FFI and README example on both MRIs;
  fixture hashes and independent official SDK/cryptography oracle.
- Docker Linux/arm64 native, then Linux/amd64 under existing Docker emulation.
  Observe results separately; emulation is not native x86 hardware certification.
- FFI: reject overlapping/overflowing ranges before access; valid range sweeps,
  guard bytes, unchanged input, repeated caller allocation/drop, actual internal
  panic containment and deterministic structured-input mutation smoke.
- Date-pinned official Rust nightly for strict-provenance Miri (FFI library tests)
  and AddressSanitizer (all tests) on Linux/arm64. No host tools installed.

Budget: zero paid services/CI; local Docker runtime --cpus=2, --memory<=2g. Build
preparation may download only official vendor/registry resources; runtime tests
--network none. No workflow added or Actions triggered. Sustained fuzz/load,
macOS/Windows builds, publishing, release tags/assets and production authorization
remain outside this phase. Required review is independent latest-HEAD Codex, no
substantive unresolved findings, checked merge SHA. No Owner-identity APPROVE review.
