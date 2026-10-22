# fnm e2e-derived cases (basic, latest, latest-lts, nvmrc-lts, aliases, exec, current, uninstall, system-node)
# against the fake binary mirror from scenario12.pre. Expected behaviour is always upstream nvm's.
t() { printf '=== nvm %s\n' "$*"; nvm "$@" 2>&1; echo "rc=$?"; }
rm -rf /tmp/p12 && mkdir -p /tmp/p12 /tmp/sysbin && cd /tmp/p12
printf '#!/bin/sh\necho custom\n' > /tmp/sysbin/node; chmod +x /tmp/sysbin/node
# current with nothing installed / no node on PATH
t current
t ls
t ls-remote
t ls-remote --lts
t ls-remote 20
t ls-remote v8
t version-remote lts/*
t version-remote --lts
t version-remote lts/iron
t version-remote lts/-1
t version-remote lts/jod
t version-remote 20.10
t version-remote 99
t version-remote node
# latest-lts / latest / partial / exact
t install --lts
t alias
t install node
t install 20
t install v20.10
t install 8.11.3
t install 6
t install 99
t install lts/hydrogen
t install lts/iron
t install lts/-2
t install lts/nope
t ls
t current
# nvmrc-lts: .nvmrc with an LTS definition
echo lts/hydrogen > .nvmrc
t use
t install
t which
t exec node -v
rm .nvmrc
# basic.test .nvmrc with v prefix (exact, installed)
echo v8.11.3 > .nvmrc
t install
t use
t current
rm .nvmrc
# aliases.test "aliasing versions"
t alias oldie 8.11
t alias older 6
t alias default older
t ls
t use older
t current
t use oldie
t current
t use default
t current
t unalias oldie
t use oldie
t unalias lts
t unalias oldie
t alias version8 8.11.3
t alias version8
t unalias version8
t alias version8
# exec.test: version file exec, explicit version
echo v8.11.3 > .nvmrc
t exec -- node -v
t exec node -v
t exec 6 node -v
t exec 20.10 node -v
t exec --silent 22 node -v
t run --silent 6 --version
rm .nvmrc
# system node (system-node.test / aliases "can alias the system node")
PATH=/tmp/sysbin:$PATH
t use 20
t use system
t current
t which system
t version system
t alias my_system system
t alias my_system2 my_system
t ls my_system
t use my_system
t current
t use my_system2
t alias default system
t use default
t current
t unalias my_system
t use my_system
t use my_system2
# uninstall.test: uninstall via alias / partial / current
t use 6
t uninstall 6
t alias hello 22.11.0
t uninstall hello
t uninstall 22.11
t uninstall 22.11
t alias
t ls
t uninstall 23
t ls node
t ls
