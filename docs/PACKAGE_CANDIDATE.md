# Unpublished candidate scope

Candidate version: **0.1.0-alpha.1**, explicitly experimental, no stable SDK or
source-Gem replacement promise. Cargo publishing remains disabled. Local source
archive and native bundles only; any registry/tag/Release/artifact upload requires
separate Owner approval. The source Gem remains entirely read-only.

Intended initial target set: aarch64-unknown-linux-gnu, plus explicitly labeled
x86_64-unknown-linux-gnu tested under Docker emulation. No macOS/Windows/musl or
native x86 hardware certification. C ABI v1 and request/result/Evidence contracts
remain as documented in CONTRACT.md, including caller memory ownership, no Host
state/authorization and the 131072-byte whole-request limit. Initial release may
choose arm64 only if native x86 hardware evidence is required.

This file defines candidate scope. Exact package bytes, licenses, toolchains,
checksums and observed results are retained separately in the candidate report;
no public distribution has been performed.
