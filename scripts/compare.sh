#!/bin/bash
# Runs inside the container: same scenario through upstream nvm and the native build, then diffs.

export NVM_DIR=/tmp/nvmtest NVM_NO_COLORS=--no-colors NVM_NO_PROGRESS=1 TERM=dumb
CORE=/src/target/release/nvm-core
run() { # $1=label ; rest = shell snippet that defines nvm
  rm -rf "$NVM_DIR" /tmp/home4 /tmp/proj4 /tmp/proj5 /tmp/sysbin /tmp/hook.sh; mkdir -p "$NVM_DIR"
  ( export PATH=/usr/local/bin:/usr/bin:/bin; eval "$2"; cd /tmp; . /src/scripts/${SC:-scenario.sh} ) > /tmp/out-$1.txt 2>&1
}
cp /opt/nvm-upstream/nvm-exec /tmp/nvm-exec.up; run up  'cp /tmp/nvm-exec.up $NVM_DIR/nvm-exec; ln -sf /opt/nvm-upstream/nvm.sh $NVM_DIR/nvm.sh; . /opt/nvm-upstream/nvm.sh --no-use'
run nat "NVM_CORE=$CORE; . /src/nvm.sh --no-use"
# normalise volatile bits
for f in up nat; do sed -E -e 's/\x1b\[[0-9;]*m//g' -e 's/[0-9.]+(%| [kMG]B)//g' -e '/^npm notice/d' -e 's/ in [0-9.]+m?s//' /tmp/out-$f.txt > /tmp/n-$f.txt; done
if diff -u /tmp/n-up.txt /tmp/n-nat.txt > /tmp/diff.txt; then echo "IDENTICAL ($(grep -c '^=== ' /tmp/n-up.txt) commands)"; else echo "DIFFERENCES:"; head -${DIFFMAX:-150} /tmp/diff.txt; fi
