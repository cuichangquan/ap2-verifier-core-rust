#!/bin/sh
set -eu
# Nonzero means a finding or operational failure; never suppress for publication.
output=${1:-.cache/candidate-audit}
mkdir -p "$output"
docker build -f Dockerfile.audit -t ap2-core-audit:local .
status=0
docker run --rm --network none --cpus=2 --memory=2g -v "$PWD:/work:ro" -w /work \
 ap2-core-audit:local cargo audit --no-fetch --db /advisory-db --json > "$output/advisories.json" 2> "$output/advisories.stderr" || status=$?
lock_sha=$(shasum -a 256 Cargo.lock | cut -d ' ' -f 1)
checked_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)
printf '{"schema":"ap2.audit-context.1","cargo_lock_sha256":"%s","exit_code":%s,"checked_at":"%s","cargo_audit":"0.22.2","rustsec_db":"2295466c4e9df26647fd67d8fa1c705ac2fc7f81"}\n' "$lock_sha" "$status" "$checked_at" > "$output/audit-context.json"
exit "$status"
