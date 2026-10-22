#!/bin/sh
# Clean-room gate, run from the host: everything executes in Docker with a FRESH cargo target volume.
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
V="nvmfast-verify-$$"
docker volume create "$V-target" >/dev/null
docker run --rm -v "$ROOT":/src -v "$V-target":/src/target -v nvmfast-cargo:/usr/local/cargo/registry nvmfast-dev bash /src/scripts/all.sh
rc=$?
docker volume rm "$V-target" >/dev/null 2>&1
exit $rc
