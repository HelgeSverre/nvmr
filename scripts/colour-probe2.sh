export NVM_DIR=/tmp/c; rm -rf $NVM_DIR; mkdir -p $NVM_DIR/versions/node/v22.12.0/bin $NVM_DIR/versions/node/v20.20.2/bin
for v in v22.12.0 v20.20.2; do touch $NVM_DIR/versions/node/$v/bin/node; chmod +x $NVM_DIR/versions/node/$v/bin/node; done
export TERM=xterm-256color
for c in "ls-remote 22.1" "ls-remote --lts=iron" "ls-remote 99"; do echo "== $c"; script -qec "bash -c '. /opt/nvm-upstream/nvm.sh --no-use; nvm $c'" /dev/null | cat -v | sed -n '1,3p;$p;/v22.12.0/p;/v20.20.2/p' | head -8; done
echo "== no-colors flag in tty"; script -qec "bash -c '. /opt/nvm-upstream/nvm.sh --no-use; nvm ls --no-colors'" /dev/null | cat -v | head -3
echo "== NVM_COLORS"; script -qec "bash -c 'export NVM_COLORS=cgYmW; . /opt/nvm-upstream/nvm.sh --no-use; nvm ls 22'" /dev/null | cat -v | head -3
echo "== set-colors"; bash -c '. /opt/nvm-upstream/nvm.sh --no-use; nvm set-colors cgYmW; echo rc=$?; nvm set-colors zzzzz; echo rc=$?; nvm set-colors abc; echo rc=$?' 2>&1 | head -20 | cat -v
