#!/bin/sh
set -eu
# Run from repository root. Uses only committed files in an empty temporary tree.
platform=${1:-linux/arm64}
case "$platform" in
 linux/arm64) arch=arm64 ;;
 linux/amd64) arch=amd64 ;;
 *) printf 'Only linux/arm64 and linux/amd64 are scoped\n' >&2; exit 2 ;;
esac
source_sha=$(git rev-parse HEAD)
task_tree=$(mktemp -d "${TMPDIR:-/tmp}/ap2-core-clean.XXXXXX")
trap 'rm -rf "$task_tree"' EXIT HUP INT TERM
git archive "$source_sha" | tar -x -C "$task_tree"
printf 'source_commit=%s platform=%s clean_tree=true\n' "$source_sha" "$platform"
rust_image="ap2-core-validation-$arch:local"
oracle_image="ap2-core-oracle-$arch:local"
docker build --platform "$platform" -f "$task_tree/Dockerfile.validation" -t "$rust_image" "$task_tree"
docker build --platform "$platform" -f "$task_tree/Dockerfile.oracle" -t "$oracle_image" "$task_tree"
docker run --rm --platform "$platform" --network none --cpus=2 --memory=2g \
 -v "$task_tree:/work" -w /work "$rust_image" \
 sh -c 'cargo fmt --check && cargo clippy --locked --offline --all-targets -- -D warnings && cargo test --locked --offline && cargo test --release --locked --offline && cargo build --release --locked --offline'
# Same source tree contains only this architecture's newly compiled library.
for image in \
 'ruby:3.4.12-bookworm@sha256:8853c6f24866429483fce3a869e61a6a13916beb2720468c6d95c78f41c0f564' \
 'ruby:4.0.9-bookworm@sha256:470eb587e87c42ad9b0062ba9bc362d90d94fb67b09fecd6052b381d897d0777'; do
 docker run --rm --platform "$platform" --network none --cpus=2 --memory=1g \
  -v "$task_tree:/work:ro" -w /work -e AP2_CORE_LIBRARY=/work/target/release/libap2_verifier_core.so \
  "$image" ruby tests/ruby_ffi.rb
 docker run --rm --platform "$platform" --network none --cpus=1 --memory=512m \
  -v "$task_tree:/work:ro" -w /work "$image" ruby examples/verify.rb
done
docker run --rm --platform "$platform" --network none --cpus=2 --memory=1g \
 -v "$task_tree:/work:ro" -w /work -e AP2_CORE_LIBRARY=/work/target/release/libap2_verifier_core.so \
 "$oracle_image" python tests/independent_oracle.py
printf 'PASS source_commit=%s platform=%s\n' "$source_sha" "$platform"
