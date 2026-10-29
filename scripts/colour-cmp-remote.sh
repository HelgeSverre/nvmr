#!/bin/bash
# Coloured ls-remote on a pty, upstream vs native, with fake installed versions.
export TERM=xterm-256color NVM_DIR=/tmp/c
mk() { rm -rf $NVM_DIR; mkdir -p $NVM_DIR/versions/node/v22.12.0/bin $NVM_DIR/versions/node/v20.20.2/bin
  for v in v22.12.0 v20.20.2; do printf '#!/bin/sh\necho %s\n' $v > $NVM_DIR/versions/node/$v/bin/node; chmod +x $NVM_DIR/versions/node/$v/bin/node; done; }
cmds=("ls-remote 22.1" "ls-remote --lts=iron" "ls-remote 99" "ls-remote --lts" "ls-remote --no-colors 22.12")
for who in up nat; do
  mk; : > /tmp/rc-$who.txt
  for c in "${cmds[@]}"; do
    if [ $who = up ]; then init=". /opt/nvm-upstream/nvm.sh --no-use"; else init="NVMR_EXE=/src/target/release/nvmr; . /src/nvm.sh --no-use"; fi
    { echo "== $c"; script -qec "bash -c '$init; PATH=$NVM_DIR/versions/node/v22.12.0/bin:\$PATH; nvm $c; echo rc=\$?'" /dev/null | cat -v; } >> /tmp/rc-$who.txt
  done
done
diff -u /tmp/rc-up.txt /tmp/rc-nat.txt | head -${DIFFMAX:-60} && echo "REMOTE-COLOUR-DONE"; grep -c . /tmp/rc-up.txt
