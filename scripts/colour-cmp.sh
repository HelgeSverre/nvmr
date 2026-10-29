#!/bin/bash
# Compare coloured (pty) output of upstream vs native for local commands. Runs inside the container.
export TERM=xterm-256color NVM_DIR=/tmp/c
mk() { rm -rf $NVM_DIR; mkdir -p $NVM_DIR/versions/node/v20.1.0/bin $NVM_DIR/versions/node/v22.2.0/bin $NVM_DIR/alias/lts
  for v in v20.1.0 v22.2.0; do touch $NVM_DIR/versions/node/$v/bin/node; chmod +x $NVM_DIR/versions/node/$v/bin/node; done
  echo v22.2.0 > $NVM_DIR/alias/lts/iron; echo lts/iron > $NVM_DIR/alias/lts/\*; echo 20 > $NVM_DIR/alias/default; echo 22 > $NVM_DIR/alias/work; echo nope > $NVM_DIR/alias/broken; }
cmds=("ls" "alias" "ls 20" "ls 99" "alias default" "alias lts" "ls --no-colors" "alias --no-colors")
for who in up nat; do
  mk; : > /tmp/col-$who.txt
  for c in "${cmds[@]}"; do
    if [ $who = up ]; then init=". /opt/nvm-upstream/nvm.sh --no-use"; else init="NVMR_EXE=/src/target/release/nvmr; . /src/nvm.sh --no-use"; fi
    { echo "== $c"; script -qec "bash -c '$init; nvm $c; echo rc=\$?'" /dev/null | cat -v; } >> /tmp/col-$who.txt
  done
  NVM_COLORS=cgYmW; export NVM_COLORS
  { echo "== NVM_COLORS"; script -qec "bash -c '$init; nvm ls 22'" /dev/null | cat -v; } >> /tmp/col-$who.txt
  unset NVM_COLORS
done
diff -u /tmp/col-up.txt /tmp/col-nat.txt && echo COLOUR-IDENTICAL
