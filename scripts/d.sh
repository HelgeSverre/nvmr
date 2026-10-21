#!/bin/sh
# Run a command inside the isolated dev container. Nothing executes on the host.
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
exec docker run --rm -i -v "$ROOT":/src -v nvmfast-target:/src/target -v nvmfast-cargo:/usr/local/cargo/registry \
  -e CARGO_TERM_COLOR=never nvmfast-dev "$@"
