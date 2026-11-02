# Dated dependency review — publication HOLD

2026-11-02 UTC: cargo-audit **0.22.2**, official RustSec DB commit
`2295466c4e9df26647fd67d8fa1c705ac2fc7f81` (1395 advisories), Cargo.lock's
45 dependency packages. Registry index information was fetched during Docker build
preparation; audit ran with `--network none --no-fetch`, including yanked checks.
No advisory was ignored or suppressed. Tool archive checksum reported by official
crates.io: `700c2b240f7fd330c24b675fe429f73a5b676531fcc6300400b2b67f155ba12a`.

Result: **one vulnerability, no warnings; audit exit 1**. This is an actual
security finding, not a green audit. First attempt also lacked some registry
entries for yanked checks; those were fetched and the complete offline audit was
rerun. That operational failure was not treated as success.

## RUSTSEC-2026-0387

`p256 0.13.2` / `elliptic-curve 0.13.8` resolve `crypto-bigint 0.5.5`.
[RustSec primary advisory](https://rustsec.org/advisories/RUSTSEC-2026-0387.html)
reports secret-dependent timing in some intended constant-time integer operations.
Its published patched range is **>=0.7.6**. The snapshot and machine-readable
[scan](candidate-initial/advisories.json) preserve the actual finding and upstream CC-BY-4.0
attribution/license. The core is not declared unaffected.

Runtime boundary: this project imports VerifyingKey, accepts canonical public
SPKI keys and verifies signatures. It exposes no signing, private-key import or
issuance operation. This narrows the apparent private-key exposure, but does not
prove every reachable arithmetic/message-data operation harmless, eliminate
side channels or justify a blanket exemption. No formal impact proof was made.
This is a Rust dependency finding; the source Ruby Gem was neither changed nor
scanned as if it used this Rust stack.

**Release gate: HOLD.** Do not add an ignore rule just to pass. A next bounded
change can investigate a maintained RustCrypto chain that actually resolves a
patched crypto-bigint, review its MSRV/API/semantics, rerun signed conformance,
independent oracle, package/native/MSRV and memory checks, and repeat the current
advisory scan. A separately reviewed impact disposition is an alternative only
with evidence, not assumed from "verification-only".

## Licenses and inventory

Offline metadata inventory covers 46 packages including this crate, conservatively
including build/other-platform dependencies. Original license texts are copied
verbatim into each private native bundle, with Rust standard-library notices.
License metadata includes MIT/Apache alternatives, BSD-3-Clause, Unlicense OR MIT,
Apache-2.0 OR BSL-1.0, Unicode-3.0 conjunction and LLVM exception alternatives.
Legacy Cargo `MIT/Apache-2.0` is normalized to `MIT OR Apache-2.0` only in SPDX;
original metadata remains recorded. No missing original license files were found.
SPDX package license conclusions remain NOASSERTION, not automatic legal approval.

The SBOM is a Cargo resolved-graph inventory, not a complete OS/toolchain/Python/
Ruby container SBOM. Toolchain notices and ELF system requirements are separate;
system libc/libgcc/loader are not bundled. License suitability and actual chosen
alternative/notice obligations must be reviewed for any public distribution.
No scan proves absence of undisclosed vulnerabilities or supply-chain compromise.
