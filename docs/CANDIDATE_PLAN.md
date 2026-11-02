# Bounded unpublished candidate preparation

Owner authorizes MSRV validation, local package/binary build checks, dependency
license/SBOM/advisory audit and release decision materials. No source-Gem changes,
other-language adapters, paid services, host tools, credentials/security changes,
registry publication, public binary upload, tags or GitHub Releases. Keep
`publish = false`; changing that gate requires a later explicit scoped decision.

Choose experimental `0.1.0-alpha.1`; limited Linux targets, no Ruby SDK replacement.
Validate exact source archive extraction and local native bundles using independent
fresh directories, all core/Ruby tests, official oracle, license notices and source/
artifact SHA256. Verify declared Rust 1.85.0 or report a concrete failure/correct
metadata with evidence. Preserve every prior record rather than rewriting history.

Pin official tool images/dependencies and cargo-audit 0.22.2 + RustSec DB commit.
Local runtime runs offline, <=2 CPUs /2 GiB; compiler build jobs 2. A scan only
reports known advisories in that dated snapshot; no blanket security claim.
Generate conservative SPDX dependency inventory and retain original license texts
for compiled dependencies in local native bundles. No proprietary code/keys/data.

Finish with reviewed/merged source and exact private local candidate materials,
clear remaining gates, a small set of limited release choices and recovery steps.
Do not extend into indefinite hardening or silently publish. No new Actions CI.
