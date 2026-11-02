# AP2 Verifier Core (Rust)

Experimental native verification core with a local Ruby FFI adapter. **Unpublished;
not a replacement for the completed Ruby Gem and not production authorization.**

Ports the scoped verification rules from [ap2-mandate-verifier 1.0.0](https://github.com/cuichangquan/ap2-mandate-verifier/tree/v1.0.0),
with strict JSON, ES256, exact original-wire hashing, SD-JWT disclosure closure,
Open→Agent Closed delegation/constraints, independent Host trust/order/challenge
bindings, Direct, Checkout JWT and Checkout Receipt verification.
The first adapter uses Ruby's standard Fiddle to execute Rust in-process, without
a verifier server, subprocess or source Gem at runtime.

```text
Ruby Fiddle → bounded C ABI → Rust verification → structured result
Host supplies trust, order, challenge and clock; Host owns state and execution.
```

## Build and try (Docker)

Requires Docker; adds no Rust/Ruby tooling to the host. The official Rust image is
pinned by digest, Cargo dependencies by Cargo.lock. Build preparation downloads
only official Rust components and crates. No Actions CI or paid services.

```sh
docker build -f Dockerfile.validation -t ap2-core-rust-validation:local .
docker run --rm --network none --cpus=2 --memory=2g \
  -v "$PWD:/work" -w /work ap2-core-rust-validation:local \
  sh -c 'cargo test --locked --offline && cargo build --release --locked --offline'
docker run --rm --network none -v "$PWD:/work:ro" -w /work \
  ruby:3.4.12-bookworm ruby examples/verify.rb
```

The example uses retained synthetic inputs and a fixed clock; it does not contact
issuers, issue or consume a challenge, execute an order or make a payment.

```ruby
require_relative "adapters/ruby/lib/ap2_verifier_core"
core = AP2VerifierCore::Native.new(
  library_path: "/work/target/release/libap2_verifier_core.so")
# seven_inputs must come from independently authenticated Host records;
# preserve all signed strings and embedded JSON bodies byte-for-byte.
result = core.verify(operation: "verify_checkout", arguments: seven_inputs)
raise "Stop" unless result["status"] == "verified_research"
facts = result.fetch("evidence") # checked facts; Host must still authorize
```

Full request/response and buffer ownership: [contract](docs/CONTRACT.md).
Current coverage, differences, tests and NOT_RUN platforms: [coverage](docs/COVERAGE.md).
Baseline/provenance/licensing: [notices](THIRD_PARTY_NOTICES.md) and
[conformance manifest](conformance/v1/manifest.json).

## Security and scope

Only the fixed profiles `ap2.checkout.single_merchant_sku` v1,
`ap2.checkout.single_merchant_multi_item` v1 and Direct
`ap2.checkout.direct.single_merchant` are intended. Unknown inputs fail closed.
All candidates remain disclosed; no generic SD-JWT/JSON Schema/AP2 implementation.
Partial checks return `unverified` and no successful Evidence. An authentic Error
Receipt is a verified receipt with an unsuccessful purchase outcome.

Trust is never obtained from token keys. Host supplies current enrolled keys,
profile/policy, expected order/challenge and clock; it owns authentication,
revocation, durable replay state, atomic nonce/order CAS, final authorization and
payment/execution. Serialized Evidence is not an authenticated credential.
No real Issuer/Host production proof or cross-platform certification is claimed.

Issuance, input-building and signing remain in the source Ruby SDK. Its published
APIs and defaults are unchanged. Python/TypeScript/Go adapters are future work.
No cargo publication, public binary release, release tags or GitHub Release is
performed; each needs separate Owner approval. See [implementation plan](docs/PLAN.md).

MIT; adapted source notices are retained.

## Reproduce without previous task images

From a committed checkout with Docker, `./scripts/validate-clean.sh linux/arm64`
creates an empty temporary source/target tree, builds the pinned oracle environment,
then runs Rust debug/release, both MRIs and the independent official SDK checks.
`linux/amd64` selects Docker x86_64 execution (emulated on the Apple Silicon test
host). `./scripts/validate-safety.sh` runs date-pinned strict-provenance Miri and
AddressSanitizer on Linux/arm64. Preparation downloads are from official registries;
validation runs offline. Temporary source/build trees are removed, local Docker
cache images remain. No CI service, host tool installation or publication.
See [results and limits](docs/REPRODUCIBILITY.md) and
[release decision criteria](docs/RELEASE_READINESS.md).

## Local packaging candidate

The unpublished experimental candidate is **0.1.0-alpha.1**, MSRV **1.88** after
updating the RustCrypto stack to address the initial dependency advisory. Exact
local source/native archives, SBOM/licenses, checksums and platform limits are in
[the candidate report](docs/CANDIDATE_REPORT.md). `publish = false` remains set;
[release scope and recovery](docs/RELEASE_PROCEDURE.md) require a separate decision.
The stable source Ruby Gem is unchanged.
