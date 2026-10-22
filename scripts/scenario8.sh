t() { printf '=== nvm %s\n' "$*"; nvm "$@" 2>&1; echo "rc=$?"; }
mkfake() { for v in "$@"; do d=$NVM_DIR/versions/node/$v/bin; mkdir -p $d; printf '#!/bin/sh\necho %s\n' $v > $d/node; chmod +x $d/node; done; }
mkfake v20.11.0 v22.12.0 v0.12.18 v9.11.2 v10.24.1
mkdir -p $NVM_DIR/alias /tmp/p8 && cd /tmp/p8
echo 22 > $NVM_DIR/alias/default
nvm use 22 >/dev/null
# current / unstable / odd patterns
t ls current
t ls stable
t ls node
t ls system
t ls v
t ls 0
t ls 0.12
t ls 10
t ls 9
t ls v20
t ls node
t ls v
t which node
t use node
t current
t version stable
t version 0
t version 9
t version 10
t version .20
t version -- 20
t version lts/-1
# ordering with 0.x, 9, 10
t ls
# alias file oddities
printf '20 # c\n' > $NVM_DIR/alias/cm
printf '20' > $NVM_DIR/alias/nonl
printf '20\r\n' > $NVM_DIR/alias/crlf
printf '# only comment\n' > $NVM_DIR/alias/onlycm
: > $NVM_DIR/alias/empty
t alias
t version cm
t version nonl
t version crlf
t version onlycm
t version empty
t alias selfa selfa
t alias selfb selfa
# colours
t set-colors cgYmW
# ls-remote variants
t ls-remote --lts --lts
t ls-remote --lts=iron 20
