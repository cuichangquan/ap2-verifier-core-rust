#!/bin/sh
set -eu
# Linux/arm64 scoped; use a fresh temporary source/target tree, no host tools.
source_sha=$(git rev-parse HEAD)
task_tree=$(mktemp -d "${TMPDIR:-/tmp}/ap2-core-safety.XXXXXX")
trap 'rm -rf "$task_tree"' EXIT HUP INT TERM
git archive "$source_sha" | tar -x -C "$task_tree"
printf 'source_commit=%s platform=linux/arm64 clean_tree=true\n' "$source_sha"
docker build --platform linux/arm64 -f "$task_tree/Dockerfile.safety" -t ap2-core-safety-arm64:local "$task_tree"
docker run --rm --platform linux/arm64 --network none --cpus=2 --memory=2g \
 -v "$task_tree:/work" -w /work -e CARGO_TARGET_DIR=/work/target/miri \
 -e MIRIFLAGS=-Zmiri-strict-provenance ap2-core-safety-arm64:local \
 cargo +nightly-2025-09-18 miri test --locked --offline --lib
docker run --rm --platform linux/arm64 --network none --cpus=2 --memory=2g \
 -v "$task_tree:/work" -w /work -e CARGO_TARGET_DIR=/work/target/asan \
 -e RUSTFLAGS=-Zsanitizer=address -e ASAN_OPTIONS=detect_leaks=1:halt_on_error=1 ap2-core-safety-arm64:local \
 cargo +nightly-2025-09-18 test --locked --offline --target aarch64-unknown-linux-gnu
printf 'PASS source_commit=%s Miri=strict-provenance ASan=all-tests\n' "$source_sha"
