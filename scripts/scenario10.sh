# Old layout (node < 0.12 in $NVM_DIR/vX) and io.js installs (versions/io.js/vX): recognition only.
t() { printf '=== nvm %s\n' "$*"; nvm "$@" 2>&1; echo "rc=$?"; }
mk() { # $1=dir $2=version-string $3..=binaries
  d=$1/bin; v=$2; shift 2; mkdir -p $d
  for b in "$@"; do printf '#!/bin/sh\n[ "$1" = --version ] && exec echo %s\necho %s "$@"\n' $v $v > $d/$b; chmod +x $d/$b; done
}
for v in v0.10.48 v0.11.16; do mk $NVM_DIR/$v $v node; done
for v in v20.11.0 v22.12.0; do mk $NVM_DIR/versions/node/$v $v node; done
for v in v1.8.4 v3.3.1; do mk $NVM_DIR/versions/io.js/$v $v node iojs; done
mkdir -p $NVM_DIR/alias /tmp/p8 && cd /tmp/p8
t ls
t ls node
t ls iojs
t ls v
t ls 3
t ls iojs-v3
t ls 0
t ls 0.10
t version iojs
t version io.js
t version 3
t version iojs-v1
t version iojs-v3.3.1
t version v3.3.1
t version node
t version stable
t version unstable
t version 0.10
t alias
t alias iojs
t ls stable
t ls unstable
t use io.js
t use iojs
t use --silent unstable
t current
t use iojs-v3
t current
t ls
t which current
t which iojs-v1
t uninstall iojs-v3.3.1
t run iojs-v1 --version
t exec iojs-v3 node -v
t use 0.10
t current
t which 0.10
t ls 0.1
t use 0.11.16
t uninstall 0.11
t alias old 0.10
t alias io iojs-v1
t alias
t use 22
t uninstall iojs-v3.3.1
t uninstall 0.10
t uninstall io.js
t ls
t alias
