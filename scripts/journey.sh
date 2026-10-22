#!/bin/sh
# Runs inside a CLEAN image (debian:bookworm-slim + bash + ca-certificates only): the end-user journey.
set -u
useradd -m -s /bin/bash u 2>/dev/null
cat > /tmp/j.sh <<'J'
set -u
cd $HOME; touch .bashrc
NVM_FAST_BUNDLE=/bundle sh /install.sh
echo "--- profile"; tail -4 .bashrc
echo "--- fresh interactive shell: install LTS"
bash -ic 'nvm --version; nvm install --lts 2>&1 | tail -6; node -v; npm -v; command -v node'
echo "--- another fresh shell (auto-use default)"
bash -ic 'node -v; nvm current; nvm ls --no-colors; nvm alias --no-colors | head -4'
echo "--- project with .nvmrc and cdnvm-style"
mkdir -p proj && echo 20 > proj/.nvmrc && cd proj
bash -ic 'nvm install 2>&1 | tail -3; node -v; nvm which; nvm run -e "console.log(process.version)"; nvm exec 20 node -v'
echo "--- nvm-exec"
NODE_VERSION=default $HOME/.nvm/nvm-exec node -e 'console.log("nvm-exec", process.version)'
$HOME/.nvm/nvm-exec node -e 'console.log("nvm-exec nvmrc", process.version)'
echo "--- uninstall"
bash -ic 'nvm uninstall 20 2>&1 | tail -2; nvm ls --no-colors | head -3'
echo "--- sizes"; ls -l $HOME/.nvm | awk '{print $5, $9}' | tail -8
J
su u -c 'sh /tmp/j.sh' 2>&1
