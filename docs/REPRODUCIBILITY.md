# Clean validation and FFI safety — follow-up to PR #1

Scope and resource budget: [plan](REPRODUCIBILITY_PLAN.md). Candidate/replacement
limits: [release readiness](RELEASE_READINESS.md). No acceptance-profile changes,
new operations, public artifacts, release tags, GitHub Release or cargo publication.

The new scripts use `git archive HEAD` into an empty temporary directory, so only
committed source/fixtures enter the test and no previous `target/` survives. Each
architecture gets a separate tree and library. Docker layer caches may be reused;
this is not a claim that registry caches are empty or binary bytes reproducible.

```sh
./scripts/validate-clean.sh linux/arm64
./scripts/validate-clean.sh linux/amd64
./scripts/validate-safety.sh
```

Preparation requires network to official Docker Hub/Rust/crates.io/PyPI/GitHub.
All runtime test containers disable network and cap CPUs/memory (2 / at most 2 GiB).
The scripts remove temporary source/build trees; local Docker images remain as
caches. No host tool installation, CI workflow/Actions cost, old task image,
external Ruby source checkout or service credentials are prerequisites.

## Pinned independent oracle

`Dockerfile.oracle` starts from official Python 3.11.15 slim-bookworm digest
`d29f48a31a8b408ed19272ca1e7b10ebae13b240a27e862d3d4217c528e2e0c3`.
`requirements-oracle.lock` fixes all 16 packages and permitted wheel SHA256 values;
`--only-binary=:all: --require-hashes` rejects source builds/unpinned fallback.
`pip check` confirms closure. This replaces the old local task-image prerequisite.
The wheel hash list includes both architectures; it is not platform certification.
`scripts/lock_oracle.py` is an explicit maintenance/export command, not part of
routine validation and never an automatic dependency upgrade.

Official AP2 archive: commit `b4587ac1d055888a73b4b21750973cffba961793`,
1776074 bytes, SHA256 `526288baea429019db3f4a7ccb8aad15d629b8c760db3f30ec511d32a53b0d47`.
Download is bounded to 4 MiB, hash-checked before safe tar extraction, no fallback
revision. Source lives in the build image, not vendored into the repository.
The oracle checks fixture manifest hashes before loading the architecture-specific
release `.so`. Official SDK/cryptography and separate quantity enumeration do not
use Ruby verdicts to calculate expected cryptographic facts. Shared profile and
fixture design still limit independence; this is not security certification.

## FFI hardening and memory tools

The only production change is early rejection of overlapping address ranges and
size/address arithmetic overflow, before creating a slice or copying. Valid caller
behavior and all crypto/acceptance rules are unchanged. This cannot validate a
pointer's allocation extent, detect general dangling memory or sandbox native code.

Five library tests cover caller-owned buffers, guard bytes, every capacity from
zero through required+2, unchanged insufficient output/input, immediate allocation/
drop/reuse, overlapping ranges, argument bounds, input exactly at 131072 bytes and
a real private-helper panic caught before output writes. The private verifier
parameter exists for injection in unit tests; production ABI exposes no panic hook.
Panic/ABI failure returns zero; missing result never authorizes.

Date-pinned official nightly `2025-09-18` (rustc 1.92.0-nightly
`4645a7988`) supplies Miri/rust-src. Rustup self-update is disabled in its Docker
recipe. Strict-provenance Miri runs the five library memory tests. ASan instruments
Rust application/dependency crates and runs all library/integration tests, with
leak detection and halt-on-error. The prebuilt standard library is not rebuilt
with ASan; this is not a complete-process sanitizer audit. No sanitizer binaries
are distributed. Deterministic smoke includes 2048 malformed byte inputs and 16
single-byte mutations per retained non-reject request, with result/Evidence
invariants; this is not a coverage-guided or sustained fuzzing claim.

## Evidence

Clean build/test source: `6819e63a23b1a5330c10027628c41e07216b1669`.
Clean safety source: `b15b5d8cf4070c309d09b7fec700f1d7a67d1143`.
Rust, tests, dependencies and safety script are byte-identical between these two
commits; the latter-to-former change selects Ruby child manifests in the clean
script. Subsequent commits add documentation/header comments and these logs only.
PR #1's original evidence and records remain unchanged.

| Observed check | Result |
| --- | --- |
| Linux/arm64 clean Rust debug/release, fmt/Clippy | PASS: five library + five integration tests per mode; retained 61 cases inside integration test |
| Linux/amd64 clean Rust debug/release, fmt/Clippy | Same PASS under Docker emulation; actual rustc host x86_64-unknown-linux-gnu confirmed |
| MRI 3.4.12 and 4.0.9, each architecture | Each 85 FFI checks + 200 calls from 8 threads + three String override regressions PASS |
| README Ruby→Rust release-library example, both MRIs/architectures | verified_research/scoped_research_verified; evidence=true |
| Newly provisioned independent oracle, each architecture | Fixture hashes + 47 basic + 40 causal signature/binding + four exact-quantity checks PASS |
| Strict-provenance Miri Linux/arm64 | Five library tests PASS, 49.15s interpreted test phase |
| ASan Linux/arm64, leak detection enabled | Five library + five integration tests PASS, no reported sanitizer issue |

Logs: [clean arm64](validation-v2/clean-arm64.log),
[clean amd64](validation-v2/clean-amd64.log),
[clean Miri/ASan](validation-v2/clean-safety.log).
Each starts with its source SHA and ends in an explicit PASS only after all steps.

Both MRIs use architecture-specific child manifests of the same pinned official
indexes: 3.4.12 index `8853c6f24866429483fce3a869e61a6a13916beb2720468c6d95c78f41c0f564`,
4.0.9 index `470eb587e87c42ad9b0062ba9bc362d90d94fb67b09fecd6052b381d897d0777`.
An initial amd64 run passed Rust tests but Docker refused to overwrite a shared
multi-arch digest when starting Ruby. The script was corrected to use those
indexes' separate fixed child digests, then both clean paths passed end to end.
Existing images were not removed, nor dependency versions changed to work around
the failure. This was an environment collision, not a permission denial.

Remaining NOT_RUN: native x86 hardware, macOS/Windows/musl, declared MSRV 1.85,
other sanitizers, sustained coverage-guided fuzzing/load, numeric RSS/time bounds
and binary reproducibility/distribution. Miri covered the FFI library tests, not
the whole cryptographic conformance suite; ASan covered Rust tests, not the Ruby
runtime. Docker x86 emulation is functional evidence, not hardware certification.
No current blocker prevents merging this scoped work. Publication still requires
separate Owner decision and the candidate criteria above.

All implementation, tests, adapters and records stay in ap2-verifier-core-rust.
The source Gem repository remains read-only, including code/docs/branches/PRs/
tags/releases/settings; the local source checkout remains clean at
`9b1aa41c6de6ef57311fe4dc620bd7c259f2d140`. No source-Gem runtime is needed for
these clean tests. Prior provenance is retained, not reissued or rewritten.
