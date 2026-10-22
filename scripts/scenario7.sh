t() { printf '=== nvm %s\n' "$*"; nvm "$@" 2>&1; echo "rc=$?"; }
mkfake() { for v in "$@"; do d=$NVM_DIR/versions/node/$v/bin; mkdir -p $d; printf '#!/bin/sh\necho %s\n' $v > $d/node; chmod +x $d/node; done; }
mkfake v20.11.0 v22.12.0
mkdir -p $NVM_DIR/alias /tmp/p7/sub && cd /tmp/p7
echo 22 > $NVM_DIR/alias/default
nvm use 22 >/dev/null
# --- run/exec current & bare forms
echo 20 > .nvmrc
t run current -v
t run
t exec
t exec --silent
t exec current node -v
rm .nvmrc
t run
t exec
t exec --silent
t run current -v
t exec current node -v
t use --silent
t which --silent
t exec --silent 22 node -v
# --- not-installed hints with .nvmrc
echo lts/iron > .nvmrc;  t use; t which; t run -v
echo 99 > .nvmrc;        t use; t which; t run -v; t exec 99 node -v
echo system > .nvmrc;    t use; t which; t run -v
echo node=20 > .nvmrc;   t use; t which; t run -v
rm .nvmrc
t use sys
echo system > $NVM_DIR/alias/sys; t use sys; t which sys
# --- argument handling
t use ""
t which ""
t version ""
t ls ""
t alias ""
t use 22 20
t which 22 20
t use --save system
t use --save current
cat .nvmrc 2>/dev/null; rm -f .nvmrc
t ls --lts
t alias --lts
t uninstall 22 20
t reinstall-packages current
t reinstall-packages ""
t set-colors RRRRRR
# --- alias edge cases
t alias a b c
t alias lts/missing
t alias lts/
mkdir -p $NVM_DIR/alias/lts; echo v20.11.0 > $NVM_DIR/alias/lts/iron
t unalias lts/iron
t unalias a b
t alias x 20 --silent
mkdir -p $NVM_DIR/alias/sub; echo 22 > $NVM_DIR/alias/sub/x
t alias
# --- install argument handling (no network needed)
t install --default --default 22.12.0
t install --alias=a --alias=b 22.12.0
t install --offline --lts
t install --lts= 22
# --- PATH ordering
V22=$NVM_DIR/versions/node/v22.12.0/bin; V20=$NVM_DIR/versions/node/v20.11.0/bin
PATH=/usr/bin:$V22 nvm use 22 >/dev/null; echo "p1=${PATH}"
export PATH=/usr/bin:$V22:/bin; nvm use 22 >/dev/null; nvm use 20 >/dev/null; echo "p2=${PATH//$NVM_DIR/NVM}"
export PATH=/usr/bin:/opt/x; nvm use 22 >/dev/null; nvm use 20 >/dev/null; nvm use 22 >/dev/null; echo "p3=${PATH//$NVM_DIR/NVM}"
# --- symlinked version dir / odd entries
ln -s $NVM_DIR/versions/node/v22.12.0 $NVM_DIR/versions/node/v25.0.0
touch $NVM_DIR/versions/node/v23.0.0
t ls
t version node
