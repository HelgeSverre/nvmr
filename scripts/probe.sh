export NVM_DIR=/tmp/p; rm -rf $NVM_DIR; mkdir -p $NVM_DIR; . /opt/nvm-upstream/nvm.sh --no-use
nvm ls-remote --no-colors > /tmp/a.txt 2>&1; nvm ls-remote --no-colors node > /tmp/b.txt 2>&1
diff /tmp/a.txt /tmp/b.txt | head -8; diff /tmp/a.txt /tmp/b.txt | grep -c '^<'; head -2 /tmp/b.txt
