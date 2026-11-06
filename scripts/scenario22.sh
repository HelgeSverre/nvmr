# Command dispatch and option-boundary cases using only local fake installations.
t() { printf '=== nvm %s\n' "$*"; nvm "$@" 2>&1; echo "rc=$?"; }
t current extra
t version
t version --
t version 22 extra
t version ../22
t which 22 extra
t which --silent 22 extra
t use -- 22
t use --silent -- 22
t use 20 22
t exec -- 22 node x
t exec --silent -- 22 node x
t exec --bogus 22 node x
t run --silent 22 x
t run -- 22 x
t cache dir extra
t cache clear extra
t clear-cache extra
t __auto
t __auto bogus
t __auto use ignored
t __auto cd ignored
t __auto install ignored
t use 20 --bogus
t use --bogus 20
t use -w 20
t which -- 22
t which --silent -- 22
t exec 22
t exec 22 --
t exec 22 -- --version
t exec 99 node
t run 22
t run 22 --
t run 22 --version
t run 99
t current --silent
t deactivate bogus
(NVM_DIR=/tmp/nvm-empty22; rm -rf "$NVM_DIR"; mkdir -p "$NVM_DIR"; t ls; t ls node; t ls iojs; t ls v; t ls 99; t ls foo; t ls default; t ls stable; t ls system; t ls current)
