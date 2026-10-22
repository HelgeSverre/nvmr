t() { printf '=== nvm %s\n' "$*"; nvm "$@" 2>&1; echo "rc=$?"; }
mkdir -p /tmp/proj3 && cd /tmp/proj3
printf '# comment line\n\nis-number\n' > "$NVM_DIR/default-packages"
t install --no-progress 22.12.0
ls $NVM_DIR/versions/node/v22.12.0/lib/node_modules
printf 'is-number\nis-odd extra\n' > "$NVM_DIR/default-packages"
t install --no-progress 20.20.2
printf 'is-number\n' > "$NVM_DIR/default-packages"
npm install -g --quiet is-even 2>&1 | tail -1
t use 22
npm install -g --quiet is-even >/dev/null 2>&1
ls $NVM_DIR/versions/node/v22.12.0/lib/node_modules
t use 20
t reinstall-packages 22
t reinstall-packages 20
t reinstall-packages
t reinstall-packages 99
ls $NVM_DIR/versions/node/v20.20.2/lib/node_modules
rm -f "$NVM_DIR/default-packages"
t install --no-progress --reinstall-packages-from=22 20
t install --no-progress --reinstall-packages-from=20 20
t install --no-progress --reinstall-packages-from=99 20
t install --no-progress --reinstall-packages-from 20
t install --no-progress --reinstall-packages-from= 20
t install --no-progress --reinstall-packages-from=22 --copy-packages-from=20 20
export NVM_DEBUG=1
t install-latest-npm
t use 22
t install-latest-npm
t install-latest-npm extra
unset NVM_DEBUG
t set-colors cgYmW
t set-colors zzzzz
t set-colors
echo "NVM_COLORS=${NVM_COLORS-unset}"
t unload
echo "after unload: NVM_DIR=${NVM_DIR-unset} fn=$(type -t nvm || echo gone)"
