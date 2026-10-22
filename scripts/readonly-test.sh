#!/bin/bash
# Read-only NVM_DIR as a non-root user: use/current/ls must still work (eval file falls back to mktemp).
id u >/dev/null 2>&1 || useradd -m u
rm -rf /tmp/ro; mkdir -p /tmp/ro/versions/node/v20.11.0/bin /tmp/ro/alias
printf '#!/bin/sh\necho v20.11.0\n' > /tmp/ro/versions/node/v20.11.0/bin/node; chmod +x /tmp/ro/versions/node/v20.11.0/bin/node
echo 20 > /tmp/ro/alias/default; chmod -R a-w /tmp/ro; chmod a+rx /tmp/ro
cp /src/nvm.sh /tmp/nvm-ro.sh; cp /src/target/release/nvm-core /tmp/nvm-core-ro; chmod a+rx /tmp/nvm-ro.sh /tmp/nvm-core-ro
su u -c 'export NVM_DIR=/tmp/ro NVM_CORE=/tmp/nvm-core-ro; . /tmp/nvm-ro.sh; echo "auto: $(command -v node)"; nvm use 20; echo "after use: $(command -v node) bin=$NVM_BIN"; nvm deactivate >/dev/null; echo "deact: $(command -v node || echo none)"; nvm cache clear; echo rc=$?; ls -a /tmp/ro | tr "\n" " "; echo' 2>&1
