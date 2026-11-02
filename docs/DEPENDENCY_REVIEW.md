# Dependency repair and current candidate audit

The initial p256 0.13.2 / crypto-bigint 0.5.5 candidate had one known vulnerability
and was held. [Initial analysis](DEPENDENCY_REVIEW_INITIAL.md) and
[exact initial scan](candidate-initial/advisories.json) remain preserved. It was
not published. No finding was suppressed and verification-only use was not used
as an exemption.

The revised experimental candidate pins p256 **0.14.0** and locks crypto-bigint
**0.7.10**, in RustSec's patched range for
[RUSTSEC-2026-0387](https://rustsec.org/advisories/RUSTSEC-2026-0387.html).
Official registry metadata lists 0.7.10 non-yanked with MSRV **1.87**; the minimum
patched 0.7.6 is yanked and not selected. The crate therefore corrects its declared
MSRV to **1.88**, required by the complete locked graph's sha2 0.11.1, and validates
that compiler. A 1.87 package check failed and is not reported as PASS. The same
issue was independently identified as a P2 review finding and corrected. No published Rust compatibility promise
existed. Source Gem API/runtime is unchanged.

Implementation adaptation is one private method rename, `to_encoded_point(false)`
to `to_sec1_point(false)`. Public uncompressed P-256 coordinates, canonical public
SPKI PEM, original signing input, ES256 high/low-S range, result/Evidence contracts,
trust selection and C ABI remain fixed. Existing 61 complete results (not just
status) and seven invalid scalar/signature cases match the retained old library.
Independent cryptography validates both high-S and low-S signatures in the new
stack. The old artifact is only one-time comparison evidence, never a prerequisite
of packaged tests or the independent normative oracle.

Current cargo-audit **0.22.2**, official DB commit
`2295466c4e9df26647fd67d8fa1c705ac2fc7f81`: **zero known vulnerabilities, zero
warnings, exit 0**, including yanked checks. Registry data is prepared in Docker;
audit runs offline and fails closed if data is missing. The [scan](candidate/advisories.json)
and [context](candidate/audit-context.json) bind the exact Cargo.lock SHA256.
Native bundling rejects a mismatched lock, nonzero exit, finding or warning.
No ignore entry or --no-yanked option is used. This is a dated known-advisory
check, not proof of overall security or supply-chain integrity.

Conservative SPDX Cargo graph and verbatim license texts cover resolved direct,
transitive, build and other-platform packages; their count/expressions and file
hashes are recorded with the final candidate. Rust standard-library notices are
bundled separately. System libc/libgcc/loader are not bundled and their ELF
requirements are recorded. SPDX license conclusions remain NOASSERTION; this is
inventory/notice preparation, not automatic legal approval or a whole-container
SBOM. Legacy MIT/Apache syntax normalization preserves the original metadata.

No registry publication, public binary artifact, tag or GitHub Release occurred.
The next decision is limited release scope/approval after the exact final checks,
not an advisory waiver or a request to ship the known-vulnerable initial build.
