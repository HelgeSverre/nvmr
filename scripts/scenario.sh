# Sourced after `nvm` is defined. Every step prints a header, then output+rc (stdout and stderr interleaved).
t() { printf '=== nvm %s\n' "$*"; nvm "$@" 2>&1; echo "rc=$?"; }
t --version
t ls-remote --no-colors 22
t ls-remote --lts --no-colors
t ls-remote --lts=iron --no-colors
t ls-remote --no-colors 99
t ls-remote --no-colors
t ls-remote --no-colors 22.12
t ls-remote --no-colors node
t ls-remote --lts 22
t version-remote 22
t version-remote --lts
t version-remote lts/iron
t version-remote lts/-1
t version-remote lts/Iron
t ls
t current
t install --no-progress v22.12.0
t current
t ls
t ls 22
t version 22
t version v22.12.0
t which 22
t alias
t alias foo 22
t alias foo
t alias lts/iron
t unalias foo
t unalias foo
t use 22
t use 99
t use nope
t use --silent 22
t install --no-progress 22.12.0
t install --no-progress --lts=argon
t install --no-progress 99999
t alias myver 22.12
t ls
t exec 22 node -e 'console.log(process.version)'
t exec --silent 22 node -e 'console.log(1)'
t run 22 -e 'console.log(2)'
t uninstall 22
t uninstall v22.12.0
t uninstall v22.12.0
t ls
t alias
t deactivate
t cache dir
t bogus-command
