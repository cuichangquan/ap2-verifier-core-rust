# Security scope

Experimental unpublished library. Do not send real Mandates, keys, personal data,
nonces or payment information in issues. Report suspected vulnerabilities privately
to the repository owner through their available private contact.

This core checks only the documented fixed profiles. It does not establish actual
issuer provenance, implement revocation services, consume nonces, authenticate
saved JSON or authorize purchases. Host safety and production go-live remain
separate decisions. Never treat missing/unknown/error/partial results as permission.

C callers must uphold valid non-overlapping buffer contracts. Rust panic containment
does not protect against caller UB, abort/OOM or hostile native code in-process.
See docs/CONTRACT.md and docs/COVERAGE.md for supported grammar and test limits.
