export NVM_DIR=/tmp/c; rm -rf $NVM_DIR; mkdir -p $NVM_DIR/versions/node/v20.1.0/bin $NVM_DIR/versions/node/v22.2.0/bin $NVM_DIR/alias/lts
for v in v20.1.0 v22.2.0; do touch $NVM_DIR/versions/node/$v/bin/node; chmod +x $NVM_DIR/versions/node/$v/bin/node; done
echo v22.2.0 > $NVM_DIR/alias/lts/iron; echo lts/iron > $NVM_DIR/alias/lts/\*; echo 20 > $NVM_DIR/alias/default; echo 22 > $NVM_DIR/alias/work
export TERM=xterm-256color
. /opt/nvm-upstream/nvm.sh --no-use
nvm use 22.2.0 >/dev/null 2>&1 || true
export PATH=$NVM_DIR/versions/node/v22.2.0/bin:$PATH
for c in "ls" "alias" "ls 20" "alias default"; do echo "== $c"; script -qec "bash -c '. /opt/nvm-upstream/nvm.sh --no-use; export PATH=$PATH; nvm $c'" /dev/null | cat -v; done
