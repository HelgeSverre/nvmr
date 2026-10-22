#!/bin/bash
export TERM=dumb
rm -rf /tmp/sh; mkdir -p /tmp/sh/proj/a /tmp/sh/proj/b /tmp/sh/alias
for v in v20.20.2 v22.12.0; do d=/tmp/sh/versions/node/$v/bin; mkdir -p $d; printf '#!/bin/sh\necho %s\n' $v > $d/node; chmod +x $d/node; done
echo 22 > /tmp/sh/alias/default; echo 20 > /tmp/sh/proj/a/.nvmrc; echo 22.12.0 > /tmp/sh/proj/b/.nvmrc
cat > /tmp/t.fish <<'FEOF'
set -gx NVM_CORE /src/target/release/nvm-core
source /src/nvm.fish
echo "start: node="(command -v node)" v="(node -v)" cur="(nvm current)" bin=$NVM_BIN"
nvm use 20
echo "use20: v="(node -v)" first=$PATH[1]"
cd /tmp/sh/proj/a; echo "cd a: v="(node -v)
cd /tmp/sh/proj/b; echo "cd b: v="(node -v)
cd /tmp; echo "cd /tmp: v="(node -v)
nvm use 20 >/dev/null; nvm deactivate
echo "deact: node="(command -v node; or echo none)" bin="(set -q NVM_BIN; and echo $NVM_BIN; or echo unset)
nvm use default --silent; echo "default: v="(node -v)
echo "path entries with nvm: "(string match -r '^/tmp/sh/versions' $PATH | count)
nvm unload; echo "unload: "(type -q nvm; and echo present; or echo gone)
FEOF
env -i HOME=/root TERM=dumb PATH=/usr/local/bin:/usr/bin:/bin NVM_DIR=/tmp/sh fish /tmp/t.fish 2>&1
