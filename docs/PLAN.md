# Implementation plan and boundaries

Owner instruction (2026-11-02 JST): start Rust project autonomously, reuse completed
Ruby verifier, merge PRs without separate approval; **library releases need approval**.
README's historical planning/no-code restriction is superseded for this work.
No source Gem changes, deployment, paid infrastructure, real wallets/issuers/payments,
credentials, security/protection changes, publication, release tags or release assets.

Security proposition: independent Host-selected P-256 keys and original signed bytes
must constrain the supported Checkout profiles; missing/unknown data cannot mint
successful Evidence. Ruby calls actual Rust through a bounded local C ABI.

1. Pin baseline, extract versioned synthetic conformance inputs/expected results.
2. Implement bounded strict parser, ES256, disclosure closure and exact byte hashes.
3. Port both Open→Closed constraint profiles, trust/Host bindings and validity;
   Direct, Checkout JWT and Receipt verification. Keep issuance in Ruby baseline.
4. Add caller-owned ABI buffers, Ruby Fiddle adapter, independent oracle, signed
   negatives, boundary/concurrency tests and platform/build documentation.
5. Source audit, latest-HEAD independent review, fix substantive findings, inspect
   checks/protections/threads, merge. Stop before any release/publication.

Validation budget: local Docker only, official Rust image and official crates,
reuse existing Ruby/Python images, 2 CPUs/2 GiB per Rust run and 1 GiB oracle runs.
No automatic CI introduced; avoid consuming Actions budget. Fetch dependency cache
once into a local validation image, then run offline. No host tool installation.
Do not run unrelated historical SQL/Host labs or production/Phase 2-B work.

STOP: permission denial, cost/security change, real data, unpinned oracle, material
signature/schema ambiguity, unresolved substantive review, failed required checks.
Continue unaffected work and report exact blocked action. Never bypass protections.
