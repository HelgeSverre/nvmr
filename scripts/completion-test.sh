#!/bin/bash
# Alias completion must list $NVM_DIR/alias only, from any directory, in bash and zsh, without
# changing directory (zsh's `command cd` runs /usr/bin/cd, and a real cd fires chpwd hooks).
SRC="${SRC:-/src}"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
export NVM_DIR="$T/nvm"
mkdir -p "$NVM_DIR/alias/lts" "$T/busy/a/b"
cp "$SRC/nvm.sh" "$SRC/bash_completion" "$NVM_DIR/"
printf 'v20.0.0\n' > "$NVM_DIR/alias/default"
printf 'v20.0.0\n' > "$NVM_DIR/alias/lts/iron"
for i in $(seq 1 50); do : > "$T/busy/a/b/file$i"; done
want='default lts/iron node stable unstable iojs'
norm() { tr ' ' '\n' | sed '/^$/d' | sort | tr '\n' ' '; }
want_n="$(printf %s "$want" | norm)"
fail=0

got="$(cd "$T/busy" && bash -c '. "$NVM_DIR/nvm.sh" --no-use; . "$NVM_DIR/bash_completion"; __nvm_aliases' 2>&1 | norm)"
if [ "$got" = "$want_n" ]; then echo "bash aliases: ok"; else echo "bash aliases: got '$got'"; fail=1; fi

got="$(cd "$T/busy" && zsh -fc '
  . "$NVM_DIR/nvm.sh" --no-use; . "$NVM_DIR/bash_completion"
  hooked=0; chpwd() { hooked=1; }
  __nvm_aliases
  print "hook=$hooked"; print "pwd=${PWD:t}"' 2>&1)"
aliases="$(printf '%s\n' "$got" | sed '/^hook=/d;/^pwd=/d' | norm)"
if [ "$aliases" = "$want_n" ]; then echo "zsh aliases: ok"; else echo "zsh aliases: got '$(printf %s "$aliases" | head -c 300)'"; fail=1; fi
printf '%s\n' "$got" | grep -qx 'hook=0' && echo "zsh chpwd hook: not fired" || { echo "zsh chpwd hook fired"; fail=1; }
printf '%s\n' "$got" | grep -qx 'pwd=busy' && echo "zsh pwd: unchanged" || { echo "zsh pwd changed"; fail=1; }

[ $fail = 0 ] && echo "COMPLETION-OK"
exit $fail
