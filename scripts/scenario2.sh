t() { printf '=== nvm %s\n' "$*"; nvm "$@" 2>&1; echo "rc=$?"; }
e() { printf '=== env\n'; echo "NVM_BIN=${NVM_BIN-} NVM_INC=${NVM_INC-}"; echo "$PATH" | tr ':' '\n' | head -2; echo "node=$(command -v node)"; }
mkdir -p /tmp/proj/sub && cd /tmp/proj
t install --no-progress 22.12.0
t install --no-progress --lts=iron
e
t alias default
t alias default 22.12
t alias
t use iron
e
t current
t use --lts
t use lts/*
t use lts/iron
t use lts/-1
t use lts/-9
t use lts/IRON
t use default
t use node
t use stable
t use system
e
t deactivate
e
t version node
t version stable
t version lts/*
t version lts/iron
t version ../x
t version 20
t version v20
t version 22.12.0
t version 2
t which iron
t which 20
t which node
t which system
t ls --no-alias
t ls 20
t ls lts/iron
t ls 9
t ls --no-alias 20
# .nvmrc
echo "20" > .nvmrc
t use
e
t which
t version
t exec 20 node -e 'console.log(process.version)'
t run -e 'console.log(1)'
cd sub
t use
t current
cd ..
printf '# comment\nlts/iron # trailing\n' > .nvmrc
t use
printf 'node=1\n20\n' > .nvmrc
t use
printf '20\n22\n' > .nvmrc
t use
printf '' > .nvmrc
t use
printf '99\n' > .nvmrc
t use
t install --no-progress
printf '22.12.0\n' > .nvmrc
t install --no-progress
t use --silent
t use --save 20
cat .nvmrc
rm .nvmrc
t use
t which
t exec 22 node -v
t run -e 'console.log(3)'
# misc flags
t install --no-progress --default 20
t alias default
t install --no-progress --alias=work 22.12.0
t alias work
t install --no-progress --save 20
cat .nvmrc
t install --no-progress -b -s 20
t install --no-progress --default --alias=x 20
t install --no-progress ---bad 20
t use ---bad
t alias a#b 20
t alias a/b 20
t alias nope 9
t alias nope ""
t unalias stable
t unalias node
t cache dir
t cache clear
t clear-cache
t cache bad
t uninstall
t uninstall 99
t uninstall --lts=argon
t uninstall iron
t ls
t alias
