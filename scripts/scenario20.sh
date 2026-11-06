# A pseudo-terminal with vt100 terminfo has no colour capability. Upstream
# warns after set-colors; keep this separate from compare.sh's TERM=dumb shell.
printf '%s\n' '=== terminfo vt100 set-colors'
if [ -n "${NVMR_EXE-}" ]; then
  script -qec "env TERM=vt100 $NVMR_EXE set-colors cgYmW" /dev/null | tr -d '\r'
else
  script -qec "env TERM=vt100 NVM_DIR=$NVM_DIR bash -c '. \"$NVM_DIR/nvm.sh\" --no-use; nvm set-colors cgYmW'" /dev/null | tr -d '\r'
fi
echo 'rc=0'

t() { printf '=== nvm %s\n' "$*"; nvm "$@" 2>&1; echo "rc=$?"; }
t help
t --help
t help extra
