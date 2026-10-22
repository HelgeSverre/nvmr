# Ported from upstream nvm's test/sourcing and test/slow suites, plus shell-option edge cases other managers
# test (set -u, set -e, IFS, CDPATH, spaces in NVM_DIR). Fake installs; `--install` uses scenario13.pre's mirror.
t() { printf '=== nvm %s\n' "$*"; nvm "$@" 2>&1; echo "rc=$?"; }
src() { printf '=== . nvm.sh %s\n' "$*"; . "$SRC" "$@" 2>&1; echo "rc=$?"; }
mkn() { for v in "$@"; do d="$NVM_DIR/versions/node/$v/bin"; mkdir -p "$d"
  printf '#!/bin/sh\necho %s\n' "$v" > "$d/node"; chmod +x "$d/node"; done; }
SRC=/opt/nvm-upstream/nvm.sh; [ -n "${NVM_CORE-}" ] && SRC=/src/nvm.sh
rm -rf /tmp/p13 "/tmp/nvm 13" && mkdir -p /tmp/p13/rc "$NVM_DIR/alias" && cd /tmp/p13
mkn v20.10.0 v22.11.0 v22.12.0
D=$NVM_DIR

# --- sourcing with no default alias returns 0, even under `set -e`
(set -e; . "$SRC"; echo "sourced under set -e: rc=$? current=$(nvm current)")
# --- sourcing uses the default when no nvm node is active; a comment in the alias file is ignored
echo '22.11.0 # comment' > "$NVM_DIR/alias/default"
src
t ls current
t alias default
echo "node=$(node -v)"
# --- re-sourcing keeps an active version instead of switching back to the default
t use 20
src
t current
# --- --no-use after deactivate leaves nothing active
t deactivate
src --no-use
t current
# --- --install installs the .nvmrc version, then the default when there is no .nvmrc
echo 20.11 > /tmp/p13/rc/.nvmrc
(cd /tmp/p13/rc && rm -f "$NVM_DIR/alias/default" && src --install; t current; t ls)
echo 18 > "$NVM_DIR/alias/default"
(src --install; t current; t alias default)
echo 22.11.0 > "$NVM_DIR/alias/default"
# --- invalid versions fail nicely
t install invalid.invalid
t use invalid.invalid
t uninstall invalid.invalid

# --- test/slow `nvm use`: partial versions, explicit v prefix, already-active versions
t use 22
t use v22.11
t use 22.12.0
t use 22.12.0
echo "node=$(node -v) bin=${NVM_BIN#$D/}"
# --- test/slow `nvm exec` / `nvm run` with partial versions and exit codes
t exec 20 node
t exec --silent 22.11 node
t run 20 --version
t exec 20 sh -c 'exit 7'
t run 99 --version
# --- test/slow `nvm uninstall`: the active version cannot be uninstalled; partial versions resolve
t use 22.12.0
t uninstall 22.12.0
t uninstall 20.10
t ls

# --- set -u: sourcing and everyday commands must not trip over unset variables
(set -u; . "$SRC" --no-use; nvm use 22 >/dev/null; echo "set -u: use rc=$? node=$(node -v)"
  nvm current; nvm ls >/dev/null; echo "ls rc=$?"; nvm version 22; nvm which 22 | sed "s|$D/||"
  cd /tmp/p13/rc; nvm use >/dev/null 2>&1; echo "nvmrc use rc=$?")
# --- set -e: a failing nvm command fails the caller; a successful one does not
(set -e; nvm use 22 >/dev/null; echo "set -e: ok after use"; nvm use 99 >/dev/null 2>&1; echo "unreachable")
echo "set -e subshell rc=$?"
# --- a custom IFS and CDPATH in the caller's shell do not change behaviour
(IFS=:; CDPATH=/tmp; nvm use 22.11 >/dev/null; echo "IFS/CDPATH: node=$(node -v)"; nvm ls | head -3)
(CDPATH=/tmp/p13; cd / && cd rc >/dev/null; nvm use >/dev/null 2>&1; echo "CDPATH: rc=$? node=$(node -v) pwd=$PWD")
# --- NVM_DIR containing a space
(NVM_DIR="/tmp/nvm 13"; mkn v22.12.0; mkdir "$NVM_DIR/alias"; echo 22 > "$NVM_DIR/alias/default"
  if [ -n "${NVM_CORE-}" ]; then cp /src/nvm-exec "$NVM_CORE" "$NVM_DIR/"
  else cp /opt/nvm-upstream/nvm-exec "$NVM_DIR/"; ln -s "$SRC" "$NVM_DIR/nvm.sh"; fi
  nvm deactivate >/dev/null; . "$SRC"; echo "space: rc=$? node=$(node -v)"
  nvm current; nvm which 22; nvm alias default; nvm ls --no-alias; nvm use 22; nvm exec 22 node)

# --- version-file parsing edge cases (from fnm, n, nvs and nodenv's .nvmrc tests)
mkdir -p /tmp/p13/vf && cd /tmp/p13/vf
for c in '20' 'v20' '  22.11  ' "$(printf '\t22\t')" 'lts/*' 'lts/iron' 'lts/IRON' 'node' 'stable' 'system' \
  '' '# just a comment' "$(printf '\n\n20\n\n')" "$(printf '\357\273\27720')" '20.11.1 extra' 'version=20' \
  "$(printf 'node=20\n22')" '>=20' '^20' '20.x' 'latest' 'lts' 'v'; do
  printf '%s' "$c" > .nvmrc
  printf '=== .nvmrc [%s]\n' "$(printf '%s' "$c" | od -An -c | tr -s ' ' | tr -d '\n')"
  nvm use 2>&1; echo "rc=$?"; w=$(nvm which 2>&1); r=$?; printf "%s\n" "$w" | sed "s|$D/||"; echo "rc=$r"
done
