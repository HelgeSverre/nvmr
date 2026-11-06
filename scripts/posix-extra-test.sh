#!/bin/bash
# Exercise native lazy loading under a caller-supplied IFS in POSIX shells.
set -euo pipefail

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
mkdir -p "$tmp/versions/node/v20.1.0/bin" "$tmp/alias"
printf '#!/bin/sh\necho v20.1.0\n' >"$tmp/versions/node/v20.1.0/bin/node"
chmod +x "$tmp/versions/node/v20.1.0/bin/node"
printf '20\n' >"$tmp/alias/default"

for shell in dash bash zsh; do
  env -i HOME="$tmp" PATH=/usr/local/bin:/usr/bin:/bin NVM_DIR="$tmp" \
    "$shell" -c 'IFS=:
      NVM_LAZY=1 NVMR_EXE=/src/target/release/nvmr
      . /src/nvm.sh
      test "$IFS" = :
      test "$(node -v)" = v20.1.0
      test "$IFS" = :
      test "$(nvm current)" = v20.1.0'
  env -i HOME="$tmp" PATH=/usr/local/bin:/usr/bin:/bin NVM_DIR="$tmp" \
    "$shell" -c 'set -u
      unset IFS
      NVM_LAZY=1 NVMR_EXE=/src/target/release/nvmr
      . /src/nvm.sh
      test "${IFS+x}" != x
      test "$(node -v)" = v20.1.0
      test "${IFS+x}" != x'
done
echo 'POSIX-LAZY-IFS-PASS'
