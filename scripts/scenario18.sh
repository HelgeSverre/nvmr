t() { printf '=== nvm %s\n' "$*"; nvm "$@" 2>&1; echo "rc=$?"; }
case "$(uname -m)" in aarch64|arm64) a=arm64;; x86_64) a=x64;; *) a=$(uname -m);; esac
s=node-v24.1.0-linux-$a
mkdir -p "$NVM_DIR/.cache/bin/$s"
printf broken > "$NVM_DIR/.cache/bin/$s/$s.tar.xz"
t install --no-progress 24.1.0
t uninstall 24.1.0
t install --offline --no-progress 24.1.0
