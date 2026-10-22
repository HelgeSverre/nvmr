#!/bin/sh
# Run a command inside the isolated dev container. Nothing executes on the host.
# NVMFAST_TARGET_VOL selects the cargo target volume, so parallel workers don't share build output.
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
exec docker run --rm -i -v "$ROOT":/src -v "${NVMFAST_TARGET_VOL:-nvmfast-target}":/src/target -v nvmfast-cargo:/usr/local/cargo/registry \
  -e CARGO_TERM_COLOR=never nvmfast-dev "$@"
