t() { printf '=== nvm %s\n' "$*"; nvm "$@" 2>&1; echo "rc=$?"; }
e() { echo "PATHhead=${PATH%%:*} NVM_BIN=${NVM_BIN-unset} node=$(command -v node || echo none)"; }
mkdir -p /tmp/proj5 && cd /tmp/proj5
t use system
t which system
t version system
t current
t ls
t alias
mkdir -p /tmp/sysbin; printf '#!/bin/sh\necho v18.19.0\n' > /tmp/sysbin/node; chmod +x /tmp/sysbin/node
export PATH="$PATH:/tmp/sysbin"
t current
t ls
t ls system
t version system
t which system
t use system
e
t alias
t alias foo system
t use foo
t install --no-progress 22.12.0
t ls
t current
t use system
e
t current
t ls
t version
t version system
t use default
e
t deactivate
e
export NVM_SYMLINK_CURRENT=true
t use 22
ls -l $NVM_DIR/current | sed 's/.*current/current/'
unset NVM_SYMLINK_CURRENT
t uninstall 22
t install --offline v22.12.0
t install --offline 99
t install --offline --no-progress 22
t uninstall 22.12.0
t install --no-progress 22.12.0
t uninstall 22.12.0
t install --offline --no-progress 22.12.0
export NVM_NODEJS_ORG_MIRROR=ftp://x
t install 22.12.0
t ls-remote
export NVM_NODEJS_ORG_MIRROR=http://127.0.0.1:1
t install --no-progress 22.12.5
t ls-remote
t version-remote 22
export NVM_NODEJS_ORG_MIRROR=https://nodejs.org/dist/nonexistent
t install --no-progress 22.12.6
unset NVM_NODEJS_ORG_MIRROR
t install --no-progress -b 99999.0.0
t install --no-progress v22.12.0.1
t install --no-progress notaversion
t install --no-progress --lts=Iron
t install --no-progress --lts=nonexistent
t ls-remote --lts=nonexistent
t ls-remote foo
