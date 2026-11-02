# Preserved initial candidate finding

Initial local candidate source `269fbea1d0ad81dc4b6b1a1d8b3e05be2494b635`
used p256 0.13.2 / crypto-bigint 0.5.5 and had RUSTSEC-2026-0387: one known
vulnerability, no warnings, cargo-audit exit 1. It was never published. Its exact
source/native archive checksums, SBOM and results are retained separately; no scan
waiver or deletion of that finding is permitted.

The owner-authorized next investigation is a patched RustCrypto dependency chain
that preserves the public C ABI and existing verification acceptance semantics.
Official registry metadata (2026-11-02) lists crypto-bigint 0.7.10 non-yanked,
MSRV 1.87; 0.7.6's advisory patched lower bound is yanked and is not selected.
The initial 1.85.0 build/test evidence remains valid for the old dependency stack,
not a claim that a new patched stack supports 1.85.

If patched dependencies require 1.87, this unpublished experimental crate may
correct its declared MSRV to 1.87 with actual tests; no released Rust API/support
promise is being revoked. No trust/model/C ABI change is authorized by this note.
Full independent signature/disclosure/constraint, package/native, MSRV, memory
and current advisory checks must pass before treating the finding as resolved.
