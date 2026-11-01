# Frozen synthetic conformance suite v1

Manifest binds exact file bytes and upstream source/official AP2 commit. No real
identities or private keys. Historical source wires are unchanged; additional
cases use one-time in-memory generated keys, with public trust and fixed clock.
Both export scripts are provenance records, not a routine regeneration step;
new random signatures require a new reviewed fixture version/manifest.

`expected_status` / `expected_code` record the source Gem verdict. Rust compares
status for every case and positive/partial codes; reject code parity is not a
requirement. Failures must have false verified and null Evidence. Native transport
has its own bound; oversized requests reject before semantic processing.
The independent oracle verifies official SDK and cryptographic/binding facts
without using those source verdicts to calculate its expected cryptographic facts.
See root notices and docs/COVERAGE.md for distinctions and remaining gaps.
