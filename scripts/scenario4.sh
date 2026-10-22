t() { printf '=== nvm %s\n' "$*"; nvm "$@" 2>&1; echo "rc=$?"; }
e() { echo "PATHhead=${PATH%%:*} NVM_BIN=${NVM_BIN-unset}"; }
mkdir -p /tmp/home4 /tmp/proj4; export HOME=/tmp/home4; cd /tmp/proj4
t install --no-progress 22.12.0
V=$NVM_DIR/versions/node/v22.12.0
PREFIX=/opt/foo nvm use 22 2>&1; echo "rc=$?"; e
PREFIX=$V nvm use 22 2>&1; echo "rc=$?"; e
NPM_CONFIG_PREFIX=/opt/bar nvm use 22 2>&1; echo "rc=$?"; e
npm_config_prefix=/opt/baz nvm use 22 2>&1; echo "rc=$?"; e
NPM_CONFIG_PREFIX=$NVM_DIR/x nvm use 22 2>&1; echo "rc=$?"; e
echo "prefix=/opt/home" > $HOME/.npmrc
t use 22
e
t use --silent 22
t use --delete-prefix 22
cat $HOME/.npmrc; echo "---"
printf 'prefix = /opt/p\nglobalconfig=/x\n' > $HOME/.npmrc
t use --delete-prefix 22
cat $HOME/.npmrc; echo "---"; rm -f $HOME/.npmrc
mkdir -p $V/etc; echo "prefix=/opt/g" > $V/etc/npmrc
t use 22
t use --delete-prefix 22
cat $V/etc/npmrc; rm -f $V/etc/npmrc
echo "prefix=/opt/b" > $V/lib/node_modules/npm/npmrc
t use 22
cp $V/lib/node_modules/npm/npmrc /tmp/npmrc.bak; rm $V/lib/node_modules/npm/npmrc
echo '{}' > package.json; echo "prefix=/opt/proj" > .npmrc
t use 22
t use --delete-prefix 22
cat .npmrc; rm -f .npmrc package.json
echo "# prefix=/commented" > .npmrc; echo '{}' > package.json
t use 22
rm -f .npmrc package.json
t use 22
