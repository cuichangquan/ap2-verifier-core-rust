#!/bin/sh
set -eu
# Local Rust-only reproducible validation; no Actions execution or publication.
docker build -f Dockerfile.validation -t ap2-core-rust-validation:local .
docker run --rm --network none --cpus=2 --memory=2g -v "$PWD:/work" -w /work \
 ap2-core-rust-validation:local sh -c 'cargo fmt --check && cargo clippy --locked --offline --all-targets -- -D warnings && cargo test --locked --offline && cargo test --release --locked --offline && cargo build --release --locked --offline'
for version in 3.4.12 4.0.9; do
 docker run --rm --network none --cpus=2 --memory=1g -v "$PWD:/work:ro" -w /work \
  -e AP2_CORE_LIBRARY=/work/target/release/libap2_verifier_core.so \
  "ruby:$version-bookworm" ruby tests/ruby_ffi.rb
done
