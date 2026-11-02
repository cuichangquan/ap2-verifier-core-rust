#!/bin/sh
set -eu
# Only local archives. No release/tag/publish/upload commands.
platform=${1:-linux/arm64}
case "$platform" in
 linux/arm64) arch=arm64; triple=aarch64-unknown-linux-gnu; ruby=1b4ee3a2e4d38a7c0d31d8736133ddf76ef3bf0e9a3054c544863cbb199cdf7e; ruby40=cf6a3b57ee0dff4c89b214100910080e202507a83ab9adecfd5499a3cf5f7846 ;;
 linux/amd64) arch=amd64; triple=x86_64-unknown-linux-gnu; ruby=7456f1f4e0231f63aaf91187e396339b6ce2496fead7b2cecda7e6adbfbb6529; ruby40=1da0f57da8eae78e002e81068336dfda15086fd791c3e6328291eb9414602f2f ;;
 *) exit 2 ;;
esac
sha=$(git rev-parse HEAD); epoch=$(git show -s --format=%ct HEAD)
repo=$PWD; output="$repo/.cache/candidate/$sha/$arch"; mkdir -p "$output"
tree=$(mktemp -d "${TMPDIR:-/tmp}/ap2-candidate.XXXXXX")
trap 'rm -rf "$tree"' EXIT HUP INT TERM
mkdir "$tree/source" "$tree/extract" "$tree/bundle-test"
git archive "$sha" | tar -x -C "$tree/source"
image="ap2-core-candidate-$arch:local"
docker build --platform "$platform" -f "$tree/source/Dockerfile.validation" -t "$image" "$tree/source"
docker run --rm --platform "$platform" --network none --cpus=2 --memory=2g -v "$tree/source:/work" -w /work "$image" cargo package --locked --offline --no-verify
crate=ap2-verifier-core-0.1.0-alpha.1.crate
cp "$tree/source/target/package/$crate" "$output/$crate"
crate_sha=$(shasum -a 256 "$output/$crate" | cut -d ' ' -f 1)
tar -xzf "$output/$crate" -C "$tree/extract"
package="$tree/extract/ap2-verifier-core-0.1.0-alpha.1"
printf 'source_commit=%s platform=%s crate_sha256=%s\n' "$sha" "$platform" "$crate_sha"
# Build/tests from the exact extracted archive, with an initially empty target.
if [ "$arch" = arm64 ]; then
 docker build --platform linux/arm64 -f "$tree/source/Dockerfile.msrv" -t ap2-core-msrv:local "$tree/source"
 docker run --rm --platform linux/arm64 --network none --cpus=2 --memory=2g -v "$package:/work" -w /work -e CARGO_TARGET_DIR=/work/target/msrv ap2-core-msrv:local sh -c 'rustc +1.87.0 -vV && cargo +1.87.0 test --locked --offline && cargo +1.87.0 build --release --locked --offline'
fi
docker run --rm --platform "$platform" --network none --cpus=2 --memory=2g -v "$package:/work" -w /work "$image" sh -c 'cargo test --locked --offline && cargo build --release --locked --offline'
# Inventory/tooling lives outside the package, so it cannot add undeclared files.
docker run --rm --platform "$platform" --network none --cpus=2 --memory=1g -v "$package:/work" -v "$output:/out" -v "$tree/source/scripts:/tools:ro" -w /work -e SOURCE_DATE_EPOCH="$epoch" "$image" python3 /tools/candidate_inventory.py /out/inventory
docker run --rm --platform "$platform" --network none --cpus=2 --memory=1g -v "$package:/work:ro" -v "$output:/out" -v "$tree/source/scripts:/tools:ro" -v "$tree/source/docs/candidate:/audit:ro" -w /work -e CANDIDATE_SOURCE_SHA="$sha" -e CANDIDATE_CRATE_SHA="$crate_sha" "$image" python3 /tools/candidate_bundle.py /out "$triple"
# Extract the exact native archive into another empty tree before loading it.
tar -xzf "$output/ap2-verifier-core-0.1.0-alpha.1-$triple.tar.gz" -C "$tree/bundle-test"
bundle="$tree/bundle-test/ap2-verifier-core-0.1.0-alpha.1-$triple"
for ruby_digest in "$ruby" "$ruby40"; do
 docker run --rm --platform "$platform" --network none --cpus=2 --memory=1g -v "$bundle:/work:ro" -w /work -e AP2_CORE_LIBRARY=/work/lib/libap2_verifier_core.so "ruby@sha256:$ruby_digest" ruby tests/ruby_ffi.rb
 docker run --rm --platform "$platform" --network none --cpus=1 --memory=512m -v "$bundle:/work:ro" -w /work -e AP2_CORE_LIBRARY=/work/lib/libap2_verifier_core.so "ruby@sha256:$ruby_digest" ruby examples/verify.rb
done
# The independently provisioned oracle image is reproducible from Dockerfile.oracle.
docker build --platform "$platform" -f "$tree/source/Dockerfile.oracle" -t "ap2-core-oracle-$arch:local" "$tree/source"
docker run --rm --platform "$platform" --network none --cpus=2 --memory=1g -v "$bundle:/work:ro" -w /work -e AP2_CORE_LIBRARY=/work/lib/libap2_verifier_core.so "ap2-core-oracle-$arch:local" python tests/independent_oracle.py
printf 'PASS source_commit=%s target=%s local_artifacts=.cache/candidate/%s/%s\n' "$sha" "$triple" "$sha" "$arch"
