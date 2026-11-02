# Experimental release decision and recovery procedure

**No release is authorized or performed.** Local 0.1.0-alpha.1 is a candidate,
`publish = false` remains in both source and normalized Cargo manifest. Source Gem
code/docs/branches/PRs/tags/releases/settings are permanently outside this work.

## Concrete options

1. **Recommended now: keep all artifacts private and HOLD publication.** Close
   RUSTSEC-2026-0387 with a maintained compatible patched RustCrypto chain and
   repeat conformance/oracle/MSRV/memory/package/advisory gates. Choose initial
   Linux arm64-only or arm64 + explicitly experimental emulated-x86 scope.
2. After that gate and independent review, request **source crate only** approval
   for an exact experimental version/name/registry/source SHA/package digest.
   Native binary uploads remain unapproved, reducing distribution obligations.
3. After that gate and per-target distribution review, request source plus exact
   Linux native bundle approval. x86 needs an explicit decision about emulated
   evidence versus additional native-hardware testing. macOS/Windows/musl stay out.

This is a choice of future bounded release scope, not a request to publish a
currently flagged artifact. No waiver/ignore rule is used to turn the audit green.
A scoped non-applicability disposition would need independent evidence and an
explicit reviewed decision; verification-only use is not itself that proof.

## Before any approval request

- Re-check registry name/ownership/availability read-only; no credentials creation
  or ownership/security changes. Registry name is not reserved by local packaging.
- Record reviewed source SHA, version, target/runtime limits, exact source/native
  bytes+SHA256, Cargo.lock, SBOM/licenses/toolchain/system dependencies, advisory
  DB/date/results, known limitations and rollback/withdrawal constraints.
- Rebuild from exact source package, test extracted native archives with both MRIs
  and independent oracle. Required checks/review threads/protections satisfied;
  latest-HEAD independent review is not a production security certification.
- Separate Owner approval must name publication actions: registry/version, exact
  artifacts, tag and GitHub Release if any. A merge or general continuation does
  not authorize any of those actions. Never submit an Owner approval review.

## Future publish gate change

`publish = false` prevents registry publishing. If explicitly approved later,
change it in a reviewed PR to the approved registry allowlist (for example
`publish = ["crates-io"]`), or separately authorize its removal. That changes the
normalized package and checksum. Build/test the **new exact package** and present
its fresh digest before publication approval; do not claim today's gated archive
is byte-identical to a future publishable artifact. Re-check advisory snapshot
and all scope decisions. Do not silently add cargo packaging/publish overrides.

## Recovery if a future approved publication fails or is uncertain

Stop and read the registry/remote state before retrying. Do not overwrite,
re-tag or delete/recreate an existing immutable version, create credentials or
bypass denied security controls. Verify artifact digest and installation behavior
read-only. If a bad artifact was published, notify Owner with exact version/hash
and impact; any yank/withdrawal/release deletion/tag correction needs its own
explicit authorization. A yank does not erase downloads or fix existing locks;
plan a separately reviewed new version rather than imply full rollback.

Current local archive failure/recovery: scripts preserve per-source-SHA output
folders and refuse overwriting a native bundle. Use a new source commit/output
folder for a changed candidate; retain prior evidence, inspect partial files and
never call a failed check PASS. Temporary build trees are removed on exit; local
cache images/artifacts remain. Nothing is uploaded by these scripts.
