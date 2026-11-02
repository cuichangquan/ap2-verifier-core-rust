#!/bin/sh
set -eu
# Nonzero means a finding or operational failure; never suppress for publication.
output=${1:-.cache/candidate-audit}
mkdir -p "$output"
docker build -f Dockerfile.audit -t ap2-core-audit:local .
docker run --rm --network none --cpus=2 --memory=2g -v "$PWD:/work:ro" -w /work \
 ap2-core-audit:local cargo audit --no-fetch --db /advisory-db --json > "$output/advisories.json" 2> "$output/advisories.stderr"
