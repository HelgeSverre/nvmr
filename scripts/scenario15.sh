t() { printf '=== nvm %s\n' "$*"; nvm "$@" 2>&1; echo "rc=$?"; }
t ls-remote --no-colors node
t ls-remote --no-colors --lts
t version-remote --lts
t alias lts/bad
export NVM_IOJS_ORG_MIRROR=not-a-url
t ls-remote --no-colors 22
t ls-remote --no-colors node
export NVM_NODEJS_ORG_MIRROR=not-a-url NVM_IOJS_ORG_MIRROR=http://127.0.0.1:8115/iojs
t ls-remote --no-colors 22
t ls-remote --no-colors
