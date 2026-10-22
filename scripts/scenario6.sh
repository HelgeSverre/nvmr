t() { printf '=== nvm %s\n' "$*"; nvm "$@" 2>&1; echo "rc=$?"; }
t install --no-progress 22.12.0
t uninstall 22.12.0
# lock: pre-held lock with short timeout
mkdir -p $NVM_DIR/.cache/locks/v22.12.0
NVM_INSTALL_LOCK_TIMEOUT=2 nvm install --no-progress 22.12.0 2>&1 | sed "s#$NVM_DIR#NVMDIR#g"; echo "rc=${PIPESTATUS[0]}"
touch -d '2 hours ago' $NVM_DIR/.cache/locks/v22.12.0
NVM_INSTALL_LOCK_STALE=5 nvm install --no-progress 22.12.0 2>&1 | head -5; echo "rc=${PIPESTATUS[0]}"
ls $NVM_DIR/.cache/locks | wc -l
# third-party hook
cat > /tmp/hook.sh <<'HOOK'
#!/bin/sh
echo "hook args: $1 $2 $3 $4"
case "$HOOKMODE" in fail) exit 9 ;; lie) exit 0 ;; esac
mkdir -p "$5/bin"; printf '#!/bin/sh\necho %s\n' "$1" > "$5/bin/node"; chmod +x "$5/bin/node"
HOOK
chmod +x /tmp/hook.sh
t uninstall 22.12.0
NVM_INSTALL_THIRD_PARTY_HOOK=/tmp/hook.sh HOOKMODE=fail nvm install 20.0.0 2>&1 | sed "s#$NVM_DIR#NVMDIR#g"; echo "rc=${PIPESTATUS[0]}"
NVM_INSTALL_THIRD_PARTY_HOOK=/tmp/hook.sh HOOKMODE=lie nvm install 20.0.0 2>&1 | sed "s#$NVM_DIR#NVMDIR#g"; echo "rc=${PIPESTATUS[0]}"
t ls
