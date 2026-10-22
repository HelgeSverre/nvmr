export NVM_DIR=/tmp/c; mkdir -p $NVM_DIR; . /opt/nvm-upstream/nvm.sh --no-use
for a in "zzzzz" "abc" "" "cgYmW extra"; do echo "== set-colors [$a]"; nvm set-colors $a >/tmp/so.txt 2>/tmp/se.txt; echo "rc=$? stdout=$(wc -l </tmp/so.txt) stderr=$(wc -l </tmp/se.txt)"; cat /tmp/so.txt; tail -3 /tmp/se.txt | cat -v; done
echo "== env after valid"; nvm set-colors cgYmW >/dev/null; echo "NVM_COLORS=$NVM_COLORS"
echo "== unload"; nvm unload; echo "rc=$? type=$(type nvm 2>&1 | head -1) NVM_DIR=${NVM_DIR-unset}"
