t() { printf '=== nvm %s\n' "$*"; nvm "$@" 2>&1; echo "rc=$?"; }
t install --save --save 22.99.0
t install -w -w 22.99.0
t install --save -w 22.99.0
