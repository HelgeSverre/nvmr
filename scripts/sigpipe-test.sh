#!/bin/bash
# A closed stdout must end nvmr quietly through SIGPIPE (rc 141), not a "failed printing to stdout" panic.
BIN="${NVMR:-/src/target/release/nvmr}"
D="$(mktemp -d)"; trap 'rm -rf "$D"' EXIT
for i in $(seq 1 400); do mkdir -p "$D/versions/node/v1.$i.0/bin"; done
fails=0
for _ in 1 2 3 4 5; do
  NVM_DIR="$D" NVM_NO_COLORS=1 "$BIN" ls 2>"$D/err" | true
  rc=${PIPESTATUS[0]}
  if [ "$rc" != 141 ] && [ "$rc" != 0 ] || [ -s "$D/err" ]; then fails=1; echo "rc=$rc stderr: $(head -c 200 "$D/err")"; fi
done
[ $fails = 0 ] && echo "closed pipe: quiet"
