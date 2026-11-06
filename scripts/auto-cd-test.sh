#!/bin/bash
# Exercise the shell fast path and its fallbacks against the native resolver.
set -eu
export NVM_DIR=/tmp/nvmr-auto-cd-test NVMR_EXE=/src/target/release/nvmr
rm -rf "$NVM_DIR" /tmp/nvmr-auto-cd-project
mkdir -p "$NVM_DIR/versions/node/v22.1.0/bin" "$NVM_DIR/versions/node/v20.1.0/bin" \
  "$NVM_DIR/alias" "$NVM_DIR/.cache" /tmp/nvmr-auto-cd-project/child
for v in 22.1.0 20.1.0; do
  printf '#!/bin/sh\necho v%s\n' "$v" > "$NVM_DIR/versions/node/v$v/bin/node"
  chmod +x "$NVM_DIR/versions/node/v$v/bin/node"
done
. /src/nvm.sh --no-use
cd /tmp/nvmr-auto-cd-project/child

# No default and no .nvmrc: the hook must not need the executable.
NVMR_EXE=/nonexistent nvm_auto_cd
printf 'v22.1.0\n' > "$NVM_DIR/alias/default"
nvm use v22.1.0 --silent >/dev/null
NVMR_EXE=/nonexistent nvm_auto_cd
[ "$(nvm current)" = v22.1.0 ]

# A major-only default with one matching version also stays in the shell.
printf '22\n' > "$NVM_DIR/alias/default"
NVMR_EXE=/nonexistent nvm_auto_cd
mkdir -p "$NVM_DIR/versions/node/v22.2.0/bin"
printf '#!/bin/sh\necho v22.2.0\n' > "$NVM_DIR/versions/node/v22.2.0/bin/node"
chmod +x "$NVM_DIR/versions/node/v22.2.0/bin/node"
nvm_auto_cd
[ "$(nvm current)" = v22.2.0 ]

# A partial default with another active major, a changed PATH, and an .nvmrc reach the resolver.
printf '20\n' > "$NVM_DIR/alias/default"
nvm_auto_cd
[ "$(nvm current)" = v20.1.0 ]
printf 'v22.1.0\n' > "$NVM_DIR/alias/default"
PATH=/usr/local/bin:/usr/bin:/bin nvm_auto_cd
[ "$(nvm current)" = v22.1.0 ]
printf '20\n' > /tmp/nvmr-auto-cd-project/.nvmrc
nvm_auto_cd
[ "$(nvm current)" = v20.1.0 ]
rm /tmp/nvmr-auto-cd-project/.nvmrc
nvm_auto_cd
[ "$(nvm current)" = v22.1.0 ]
nvm unload >/dev/null
! type nvm_auto_cd >/dev/null 2>&1
echo 'auto-cd fast path and fallbacks: PASS'
