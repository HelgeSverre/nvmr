t() { printf '=== nvm %s\n' "$*"; nvm "$@" 2>&1; echo "rc=$?"; }
export NVM_NODEJS_ORG_MIRROR=http://127.0.0.1:8099/dist
t install --no-progress 22.99.0
t ls
t uninstall 22.99.0
t install --no-progress -j 2 22.99.0 --without-foo --bar=1
t uninstall 22.99.0
t install --no-progress -s -j 3 22.99.0
t uninstall 22.99.0
t install --no-progress -j abc 22.97.0
t install --no-progress -b 22.96.0
NVM_NO_SOURCE_FALLBACK=1 nvm install --no-progress 22.96.0 2>&1; echo "rc=$?"
NVM_MAKE_JOBS=5 nvm install --no-progress 22.96.0 2>&1; echo "rc=$?"
t install --no-progress 22.98.0
t ls
ls $NVM_DIR/.cache/src/ | tr '\n' ' '; echo
t install --no-progress -s -b 22.97.0
t cache clear
