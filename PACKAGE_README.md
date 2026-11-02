# ap2-verifier-core 0.1.0-alpha.1 — unpublished experimental candidate

Reusable fixed-profile Rust verification with a local C ABI and Ruby Fiddle
adapter. This is not a replacement for the stable ap2-mandate-verifier Ruby SDK,
not general AP2 compliance and not purchase/payment authorization.

Source package includes core, lockfile, public C header, Ruby adapter, bounded
conformance tests and safe synthetic fixtures. No private keys or real Mandates.
Linux GNU arm64 and Docker-emulated x86_64 are the limited observed targets;
macOS/Windows/musl and native x86 hardware are not certified.

From the extracted source package, using Docker (preparation may fetch official
locked crates; no host Rust/Ruby installation):

```sh
docker run --rm --cpus=2 --memory=2g -v "$PWD:/work" -w /work \
 rust:1.90.0-bookworm@sha256:3914072ca0c3b8aad871db9169a651ccfce30cf58303e5d6f2db16d1d8a7e58f \
 sh -c 'cargo test --locked && cargo build --release --locked'
```

Use the image architecture matching the library. The local native bundle places
that target's library at `lib/libap2_verifier_core.so`; source builds place it in
`target/release/`. Supply its absolute trusted path, never a token-selected path:

```ruby
require_relative "adapters/ruby/lib/ap2_verifier_core"
core = AP2VerifierCore::Native.new(library_path: "/absolute/path/libap2_verifier_core.so")
result = core.verify(operation: "verify_checkout", arguments: independent_seven_inputs)
raise "Stop" unless result["status"] == "verified_research"
```

`examples/verify.rb` uses fixed synthetic time and can take `AP2_CORE_LIBRARY` to
select the bundle library. It performs no challenge consumption, order or payment.
[ABI/ownership contract](docs/CONTRACT.md), [candidate scope](docs/PACKAGE_CANDIDATE.md),
[MIT/source notices](THIRD_PARTY_NOTICES.md). Full dated evidence is in the
[repository](https://github.com/cuichangquan/ap2-verifier-core-rust).

Unknown/malformed/unavailable/partial results never authorize. Host supplies
independently authenticated current keys, trust/profile, order/challenge and clock;
it owns revocation, durable atomic replay/order state, final authorization and
execution. Serialized checked facts are not authenticated credentials. Whole JSON
requests are limited to 131072 bytes, including escaping. ABI v1 requires valid
caller buffers and does not catch pointer UB, OOM or abort.

The source Gem is unchanged. This local candidate is unpublished, registry
publishing disabled; public registry/binary/tag/Release needs separate approval.
