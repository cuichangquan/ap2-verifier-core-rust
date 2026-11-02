#!/bin/sh
set -eu
# Run from repository root. Uses only committed files in an empty temporary tree.
platform=${1:-linux/arm64}
case "$platform" in
 linux/arm64)
  arch=arm64
  ruby34=1b4ee3a2e4d38a7c0d31d8736133ddf76ef3bf0e9a3054c544863cbb199cdf7e
  ruby40=cf6a3b57ee0dff4c89b214100910080e202507a83ab9adecfd5499a3cf5f7846 ;;
 linux/amd64)
  arch=amd64
  ruby34=7456f1f4e0231f63aaf91187e396339b6ce2496fead7b2cecda7e6adbfbb6529
  ruby40=1da0f57da8eae78e002e81068336dfda15086fd791c3e6328291eb9414602f2f ;;
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
# Child manifests of the fixed official 3.4.12 / 4.0.9 indexes. Separate digest
# identities avoid classic Docker image-store collisions across architectures.
# Same source tree contains only this architecture's newly compiled library.
for image in \
 "ruby@sha256:$ruby34" "ruby@sha256:$ruby40"; do
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
