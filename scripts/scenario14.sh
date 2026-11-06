t() { printf '=== nvm %s\n' "$*"; nvm "$@" 2>&1; echo "rc=$?"; }
t ls-remote --no-colors node
t version-remote node
t ls-remote --no-colors 27
t version-remote 27
