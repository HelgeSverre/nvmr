# npm upgrade ladders for old Node releases, using fake installed node/npm binaries.
t() { printf '=== nvm %s\n' "$*"; nvm "$@" 2>&1; echo "rc=$?"; }
mkold() {
  v=$1
  case "$v" in
    0.*) d="$NVM_DIR/v$v/bin" ;;
    *) d="$NVM_DIR/versions/node/v$v/bin" ;;
  esac
  mkdir -p "$d"
  printf '#!/bin/sh\necho v%s\n' "$v" > "$d/node"
  printf '#!/bin/sh\ncase "$1" in --version) echo 1.0.0 ;; *) exit 0 ;; esac\n' > "$d/npm"
  chmod +x "$d/node" "$d/npm"
}
mkold 0.6.0
mkold 0.8.0
mkold 0.9.0
mkold 9.3.0
export NVM_DEBUG=1
nvm use 0.6.0 >/dev/null
t install-latest-npm
nvm use 0.8.0 >/dev/null
t install-latest-npm
nvm use 0.9.0 >/dev/null
t install-latest-npm
nvm use 9.3.0 >/dev/null
t install-latest-npm
