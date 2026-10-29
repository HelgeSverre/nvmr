# More cases ported from upstream nvm's test/fast suite: focused usage errors, CR in .nvmrc, exec/run
# fallback warnings, the `current` symlink, MANPATH, trailing slashes, deactivate/unload, permissions.
t() { printf '=== nvm %s\n' "$*"; nvm "$@" 2>&1; echo "rc=$?"; }
mkn() { for v in "$@"; do d=$NVM_DIR/versions/node/$v; mkdir -p $d/bin $d/share/man/man1 $d/include/node
  printf '#!/bin/sh\necho %s\n' $v > $d/bin/node; chmod +x $d/bin/node; done; }
SRC=/opt/nvm-upstream/nvm.sh; [ -n "${NVMR_EXE-}" ] && { SRC=/src/nvm.sh; cp /src/nvm-exec "$NVMR_EXE" "$NVM_DIR/"; }
mkn v20.11.0 v22.12.0
mkdir -p $NVM_DIR/alias /tmp/p11b/empty && cd /tmp/p11b/empty
# Subcommands with missing or invalid args show a focused usage error (stderr, rc 127)
t cache bogus
t install
t run
t which
t uninstall
t uninstall a b
t unalias
t unalias a b
t install-latest-npm extra
t reinstall-packages
t copy-packages a b
# exec / run without a resolvable version warn and fall back; --silent mutes the warning
t exec
t exec --silent
t run bogusversion
t run --silent bogusversion
# .nvmrc with CR, without newline
printf '20\r\n' > .nvmrc; t use; t version "$(cat .nvmrc)"
printf '22' > .nvmrc; t use
printf '20\r' > .nvmrc; t use
rm .nvmrc
# the `current` symlink: created with NVM_SYMLINK_CURRENT=true, not for false
NVM_SYMLINK_CURRENT=true nvm use 20 >/dev/null; echo "current -> $(readlink $NVM_DIR/current)"
rm -f $NVM_DIR/current; NVM_SYMLINK_CURRENT=false nvm use 22 >/dev/null; echo "current exists: $([ -e $NVM_DIR/current ] && echo yes || echo no)"
# MANPATH: an unset MANPATH stays unset unless manpath exists; a set one keeps its system entries
unset MANPATH; nvm use 20 >/dev/null; echo "MANPATH=${MANPATH-unset}"
MANPATH=/usr/share/man:/usr/local/share/man; nvm use 22 >/dev/null; echo "MANPATH=$MANPATH"
nvm use 20 >/dev/null; echo "MANPATH=$MANPATH"
# deactivate removes nvm's PATH entries and NVM_BIN/NVM_INC
nvm use 22 >/dev/null; t deactivate; echo "NVM_BIN=${NVM_BIN-unset} NVM_INC=${NVM_INC-unset} node=$(command -v node || echo none)"
t deactivate
# circular alias: use / which
echo loop2 > $NVM_DIR/alias/loop1; echo loop1 > $NVM_DIR/alias/loop2
t use loop1
t which loop1
t version loop1
# which / use of system with no system node
t which system
t use system
# uninstall with incorrect permissions
mkn v18.20.4; chmod 555 $NVM_DIR/versions/node/v18.20.4/bin
if [ "$(id -u)" = 0 ]; then echo "(root: permission case skipped)"; else t uninstall 18; fi
chmod 755 $NVM_DIR/versions/node/v18.20.4/bin
# uninstall: inferred vs requested messages
t uninstall 16
t uninstall v16.0.0
t uninstall 18
t ls
# sourcing must not modify the caller's positional parameters, and strips a trailing slash from NVM_DIR
set -- one two three; . "$SRC" --no-use; echo "args after sourcing: $# $*"
D=$NVM_DIR; NVM_DIR="$D/"; . "$SRC" --no-use 2>&1; echo "NVM_DIR=$NVM_DIR"; NVM_DIR=$D
# nvm-exec shows the required version via NODE_VERSION and .nvmrc
cd /tmp/p11b; echo 20 > .nvmrc
NODE_VERSION=22 "$NVM_DIR/nvm-exec" node 2>&1; echo "rc=$?"
"$NVM_DIR/nvm-exec" node 2>&1; echo "rc=$?"
rm .nvmrc
"$NVM_DIR/nvm-exec" node 2>&1; echo "rc=$?"
printf "foo=\n" > .nvmrc; "$NVM_DIR/nvm-exec" node 2>&1; echo "rc=$?"; rm .nvmrc
# --help mentions current and the .nvmrc fallback
nvm --help | grep -E 'nvm (use|exec|run|which) \[current' ; echo "rc=$?"
