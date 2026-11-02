# Unpublished 0.1.0-alpha.1 candidate — decision materials

This is an experimental verification library, not the stable Ruby SDK's replacement.
No source-Gem change, registry publication, public binary upload, tag or Release.
Cargo source and normalized package retain `publish = false` / empty publish list.
[Limited decisions and recovery](RELEASE_PROCEDURE.md) separate merge from release.

## Scope and changes

Explicit source package include list: Rust source/lock, C header, Ruby adapter,
conformance fixtures, Rust/Ruby/independent tests, example, package-specific README,
MIT/source notices, security/ABI/candidate contracts. Development scripts, Docker
files, old task prerequisites, baseline-only Ruby test, logs, caches and binaries
are excluded. The native bundle contains that exact extracted source tree plus
`lib/libap2_verifier_core.so`, per-package original notices, Rust standard-library
notices, conservative SPDX JSON and ELF/ABI/source metadata.

The known initial crypto-bigint 0.5.5 advisory was repaired with p256 0.14.0,
elliptic-curve 0.14.3 and crypto-bigint 0.7.10. No waiver was used. The private SEC1
method rename is the only runtime-source adaptation. MSRV is **1.88**, required
by sha2 0.11.1 in the locked patched graph; 1.85 passed on the old stack and 1.87
failed on the patched graph. Those failures/initial results are not mislabeled.
[Dependency review](DEPENDENCY_REVIEW.md), [original evidence](candidate-initial/artifacts.json).

## Evidence and interpretation

Packaging scripts create a committed-source snapshot, Cargo's local `.crate`,
then a new extraction directory with empty target. Rust tests/build run there;
arm64 additionally uses the declared minimum compiler. The private native archive
is extracted again into a new directory before both MRIs load its exact library
and run FFI cases/example; the independent official SDK/cryptography oracle runs
against that same packaged library. No source Gem or previous task image is needed.
All runtime validation containers are offline with <=2 CPUs /2 GiB; preparation
uses fixed official tools/registries. Artifact archives stay under ignored `.cache/`.
The final manifest provides exact sizes/checksums/source and observed results.

Source archive was generated independently per target; identical SHA256 is checked.
Native gzip/timing/toolchain output is not claimed bit-reproducible. Native ELF
requirements and absence of RPATH/RUNPATH are checked, exported C ABI v1 is present.
Observed environment is Docker Debian bookworm/glibc; other distributions are not
certified solely by ELF symbol versions. Source SDK/transport/result limitations
remain as documented. No performance or complete security guarantee is inferred.

The advisory gate binds its dated scan to the exact Cargo.lock SHA256; a mismatch,
nonzero audit exit, vulnerability or warning prevents native candidate bundling.
Initial finding and private initial archives remain preserved; only the revised
candidate with the patched graph is under consideration.

## Remaining release decisions

- Keep private, or approve a later exact experimental source-only release.
- Choose arm64-only versus adding explicitly experimental emulated-x86 binaries;
  native x86 hardware, macOS, Windows, musl and sustained fuzz/load remain NOT_RUN.
- Review chosen dependency license alternatives/notice obligations and registry
  ownership/name; no credential or security/ownership changes were performed.
- A future approved publish allowlist change produces a different `.crate`; its
  fresh exact bytes must be retested and approved before uploading. Today's gated
  archive must not be presented as byte-identical to a future publishable one.

## Final observed candidate

Exact candidate source: `9ba1a5b9cbb22638c55615391a6b740de08dc227`.
[Artifact manifest](candidate/artifacts.json) records every size/SHA256, complete
source-package file list, library metadata and ELF requirements. Both independently
created source archives are **84173 bytes**, SHA256
`1dd24cd73f81ab1ff72473e7d6afe71255e64bafd605c4a9cebfb710181beb71`.
No archive or binary is committed/uploaded; exact files remain in the manifest's
ignored local `.cache/candidate/<source-sha>/<arch>/` paths.

| Target/check | Observed |
| --- | --- |
| aarch64-unknown-linux-gnu, Docker bookworm | Exact source package: Rust 1.88.0 and 1.90.0 test/build PASS; packaged native library: both MRIs + example + independent oracle PASS |
| x86_64-unknown-linux-gnu, Docker emulation on Apple Silicon | Exact source package: Rust 1.90.0 test/build PASS; packaged native library: both MRIs + example + independent oracle PASS. MSRV 1.88 on x86 hardware/emulation NOT_RUN |
| MRI 3.4.12 /4.0.9 on each target | Each 85 FFI checks +200 calls/8 threads +3 String override checks PASS |
| Independent oracle on each packaged target | 47 basic +40 causal signature/binding +4 quantity +2 high/low-S checks PASS |
| Old versus new backend | All 61 result objects and 7 invalid scalar/signature results exactly match |
| Patched stack Miri/ASan | Strict-provenance Miri 5 library tests; ASan 5 library +5 integration tests PASS |
| Current dated audit | 60 dependency packages, 0 vulnerabilities, 0 warnings, exit 0; Cargo.lock-bound context |
| License inventory | 61 resolved packages including root; original texts for every package, no missing texts; conservative other-target/build graph included |

Logs: [arm64 package](candidate/package-arm64.log),
[amd64 package](candidate/package-amd64.log),
[backend comparison](candidate/backend-compat.log),
[patched memory checks](candidate/patched-safety.log),
[scan](candidate/advisories.json), [SBOM](candidate/sbom.spdx.json),
[license inventory](candidate/dependency-licenses.json).
The memory checks used `f50e9ae5da364a2cab9d44d12a4f0c6c42b8c535`, whose runtime
Rust source and Cargo.lock are byte-identical to the final candidate; subsequent
changes corrected MSRV metadata and license collection. No runtime tests were
repeated merely for documentation-only commits.

Native bundle sizes: arm64 **553679 bytes**, x86_64 **568212 bytes**. Each exports
`ap2_verify_v1`, has no RPATH/RUNPATH and requires libc/libgcc; x86 also lists the
loader. Highest observed GLIBC symbol version is 2.34; tested distro is bookworm,
not a certification of every glibc>=2.34 system. Libraries are local release-mode
Rust 1.90 builds; no universal platform packages or binary reproducibility claim.

Audit and license gates did real work: a first scan lacked registry entries and
failed, the actual vulnerability then failed the complete scan, Rust 1.87 failed
MSRV verification, and r-efi's license grant was found in AUTHORS rather than a
LICENSE file. These were fixed and not waived; original evidence is retained.
Final publication status is **OWNER_APPROVAL_REQUIRED**, with no remaining known
advisory in the pinned snapshot. No released source-Gem API/runtime was changed.
