#!/bin/sh
# Syntax + sourcing checks of nvm.sh under dash, bash (--posix too), zsh, busybox-style sh.
export NVM_DIR=/tmp/pt NVM_CORE=/src/target/release/nvm-core
rm -rf /tmp/pt; mkdir -p /tmp/pt/versions/node/v20.1.0/bin /tmp/pt/alias; printf '#!/bin/sh\necho v20.1.0\n' > /tmp/pt/versions/node/v20.1.0/bin/node; chmod +x /tmp/pt/versions/node/v20.1.0/bin/node; echo 20 > /tmp/pt/alias/default
for sh in dash bash zsh; do
  printf '%-6s syntax: ' $sh; $sh -n /src/nvm.sh && echo ok
  printf '%-6s source+use: ' $sh; $sh -c '. /src/nvm.sh; nvm use 20 >/dev/null; node -v; nvm current' 2>&1 | tr '\n' ' '; echo
done
printf 'bash --posix: '; bash --posix -c '. /src/nvm.sh; node -v' 2>&1 | tr '\n' ' '; echo
printf 'bash lazy: '; bash -c 'NVM_LAZY=1; . /src/nvm.sh; type node | head -1; node -v; type node | head -1' 2>&1 | tr '\n' ' '; echo
printf 'bash lazy-path: '; echo v20.1.0 > /tmp/pt/alias/default; bash -c 'NVM_LAZY=1 NVM_LAZY_PATH=1; . /src/nvm.sh; command -v node; node -v' 2>&1 | tr '\n' ' '; echo; echo 20 > /tmp/pt/alias/default
printf 'zsh lazy: '; zsh -c 'NVM_LAZY=1; . /src/nvm.sh; node -v; nvm current' 2>&1 | tr '\n' ' '; echo
printf 'unset NVM_DIR derive: '; (unset NVM_DIR; cp /src/nvm.sh /tmp/pt/nvm.sh; bash -c '. /tmp/pt/nvm.sh --no-use; echo $NVM_DIR') 2>&1
printf 'trailing slash: '; NVM_DIR=/tmp/pt/ bash -c '. /src/nvm.sh --no-use; echo $NVM_DIR' 2>&1 | tr '\n' ' '; echo
printf 'set -eu shells: '; bash -c 'set -eu; . /src/nvm.sh; nvm use 20 >/dev/null; node -v' 2>&1 | tr '\n' ' '; echo
printf 'completion loads: '; bash -c '. /src/nvm.sh --no-use; . /src/bash_completion; complete -p nvm' 2>&1 | head -2 | tr '\n' ' '; echo
