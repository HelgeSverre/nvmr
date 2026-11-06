# Sourced after nvm is defined. Compare exact environment transitions.
t() { printf '=== nvm %s\n' "$*"; nvm "$@" 2>&1; echo "rc=$?"; }
show() { printf 'ENV path=<%s> man=<%s> nodepath=<%s> bin=<%s> inc=<%s>\n' "${PATH-}" "${MANPATH-}" "${NODE_PATH-}" "${NVM_BIN-}" "${NVM_INC-}"; }
PATH=/tmp/nvmr-path/versions/node/v20.1.0/bin:/usr/bin:/tail MANPATH=/tmp/nvmr-path/versions/node/v20.1.0/share/man:/man NODE_PATH=/tmp/nvmr-path/versions/node/v20.1.0/lib/node_modules:/modules
export PATH MANPATH NODE_PATH
t use 22.2.0
show
t deactivate
show
PATH=/tmp/nvmr-path/versions/node/v20.1.0/bin:/usr/bin:/tail
MANPATH=/tmp/nvmr-path/versions/node/v20.1.0/share/man:/man
NODE_PATH=/tmp/nvmr-path/versions/node/v20.1.0/lib/node_modules:/modules
PREFIX=/outside
export PATH MANPATH NODE_PATH PREFIX
t use 22.2.0
show
unset PREFIX
PATH=/tmp/nvmr-path/versions/node/v20.1.0/bin-old:/usr/bin:/tail
MANPATH=/tmp/nvmr-path/versions/node/v20.1.0/share/man-old:/man
NODE_PATH=/tmp/nvmr-path/versions/node/v20.1.0/lib/node_modules-old:/modules
export PATH MANPATH NODE_PATH
t use 22.2.0
show
t deactivate
show
PATH=/usr/bin:/tmp/nvmr-path/versions/node/v20.1.0/bin:/tail:
MANPATH=:/tmp/nvmr-path/versions/node/v20.1.0/share/man:/man
NODE_PATH=/tmp/nvmr-path/versions/node/v20.1.0/lib/node_modules:
export PATH MANPATH NODE_PATH
t use 22.2.0
show
t deactivate
show
