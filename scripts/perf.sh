#!/bin/bash
# Inside container. Fake installs (no downloads); compares native vs upstream.
export TERM=dumb NVM_NO_COLORS=--no-colors
rm -rf /tmp/pf; mkdir -p /tmp/pf/alias/lts /tmp/pf/proj
for i in $(seq 1 100); do d=/tmp/pf/versions/node/v18.$i.0/bin; mkdir -p $d; printf '#!/bin/sh\necho v18.%s.0\n' $i > $d/node; chmod +x $d/node; done
for v in v22.12.0 v20.20.2; do d=/tmp/pf/versions/node/$v/bin; mkdir -p $d; printf '#!/bin/sh\necho %s\n' $v > $d/node; chmod +x $d/node; done
echo 22 > /tmp/pf/alias/default; for i in $(seq 1 20); do echo 18.$i > /tmp/pf/alias/a$i; done
echo v22.12.0 > /tmp/pf/alias/lts/jod; echo lts/jod > "/tmp/pf/alias/lts/*"
export NVM_DIR=/tmp/pf NVM_EVAL_FILE=/tmp/pf/eval.out
C=/src/target/release/nvm-core
cd /tmp/pf/proj
H="hyperfine -N -i --warmup 20 --runs 300 --export-json /tmp/perf.json"
$H "$C __auto use" "$C __auto cd" "$C current" "$C version 22" "$C which 22" "$C alias default" "$C ls" "$C alias" "$C use 22" "$C use --silent 22.12.0" >/dev/null 2>&1
python3 - <<'PY'
import json
print("native (ms)         median    min    p10")
for r in json.load(open('/tmp/perf.json'))['results']:
    t=sorted(r['times']); print(f"{r['command'][len('/src/target/release/nvm-core '):]:<18} {r['median']*1000:7.2f} {r['min']*1000:6.2f} {t[len(t)//10]*1000:6.2f}")
PY
echo "--- shell source cost (bash, zsh)"
H2="hyperfine -N -i --warmup 5 --runs 40 --export-json /tmp/perf2.json"
$H2 -n bash-bare "bash --norc -c exit" -n bash-nat-nouse "bash --norc -c 'NVM_CORE=$C . /src/nvm.sh --no-use'" -n bash-nat-auto "bash --norc -c 'NVM_CORE=$C . /src/nvm.sh'" -n bash-up-nouse "bash --norc -c '. /opt/nvm-upstream/nvm.sh --no-use'" -n bash-up-auto "bash --norc -c '. /opt/nvm-upstream/nvm.sh'" \
    -n zsh-bare "zsh -f -c exit" -n zsh-nat-nouse "zsh -f -c 'NVM_CORE=$C . /src/nvm.sh --no-use'" -n zsh-nat-auto "zsh -f -c 'NVM_CORE=$C . /src/nvm.sh'" -n zsh-up-nouse "zsh -f -c '. /opt/nvm-upstream/nvm.sh --no-use'" -n zsh-up-auto "zsh -f -c '. /opt/nvm-upstream/nvm.sh'" >/dev/null 2>&1
python3 - <<'PY'
import json
for r in json.load(open('/tmp/perf2.json'))['results']:
    t=sorted(r['times']); print(f"{r['command']:<16} median {r['median']*1000:8.1f} min {r['min']*1000:8.1f}")
PY
echo "--- command latency inside a sourced shell (bash), minus startup"
S=". /src/nvm.sh --no-use"; U=". /opt/nvm-upstream/nvm.sh --no-use"
$H2 -n nat-use "bash --norc -c 'NVM_CORE=$C $S; nvm use 22 >/dev/null'" -n up-use "bash --norc -c '$U; nvm use 22 >/dev/null'" \
    -n nat-ls "bash --norc -c 'NVM_CORE=$C $S; nvm ls >/dev/null'" -n up-ls "bash --norc -c '$U; nvm ls >/dev/null'" \
    -n nat-cur "bash --norc -c 'NVM_CORE=$C $S; nvm current >/dev/null'" -n up-cur "bash --norc -c '$U; nvm current >/dev/null'" >/dev/null 2>&1
python3 - <<'PY'
import json
for r in json.load(open('/tmp/perf2.json'))['results']:
    print(f"{r['command']:<10} median {r['median']*1000:8.1f} min {r['min']*1000:8.1f}")
PY
echo "--- syscalls / linkage"
strace -c -f $C __auto use 2>&1 >/dev/null | tail -4; ldd $C | head -3; ls -l $C /src/target/release/nvm-fetch | awk '{print $5, $9}'
