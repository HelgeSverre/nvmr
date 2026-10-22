#!/bin/bash
# Runs inside the dev container: build, unit tests, every parity harness. Exit non-zero on any failure.
cd /src || exit 1
fail=0
step() { printf '\n##### %s\n' "$1"; }
chk() { if "$@"; then echo "PASS"; else echo "FAIL: $*"; fail=1; fi; }
step "rustfmt";     cargo fmt --check >/dev/null 2>&1 && echo PASS || { echo "FAIL: cargo fmt --check"; fail=1; }
step "clippy";      cargo clippy --release --all-targets -- -D warnings 2>&1 | tail -3; [ "${PIPESTATUS[0]}" = 0 ] || fail=1
step "cargo test";  cargo test --release 2>&1 | grep "test result"; [ "${PIPESTATUS[0]}" = 0 ] || fail=1
step "cargo build"; cargo build --release 2>&1 | tail -1; [ "${PIPESTATUS[0]}" = 0 ] || fail=1
for s in scenario scenario2 scenario3 scenario4 scenario5 scenario6 scenario7 scenario8 scenario9 scenario10 scenario11 scenario11b scenario12 scenario13; do
  step "$s vs upstream"; out="$(SC=$s.sh DIFFMAX=60 /src/scripts/compare.sh 2>&1)"; echo "$out" | head -70; echo "$out" | grep -q '^IDENTICAL' || fail=1
done
step "colour (pty)";  out="$(/src/scripts/colour-cmp.sh 2>&1)"; echo "$out" | tail -20; echo "$out" | grep -q 'COLOUR-IDENTICAL' || fail=1
step "colour ls-remote (pty)"; out="$(DIFFMAX=40 /src/scripts/colour-cmp-remote.sh 2>&1)"; echo "$out" | tail -8; echo "$out" | grep -q 'REMOTE-COLOUR-DONE' && ! echo "$out" | grep -q '^[-+][^-+]' || fail=1
step "bash+zsh";      out="$(/src/scripts/shell-test.sh 2>&1)"; echo "$out" | grep -E '^(#####|IDENTICAL|[<>])'; [ "$(echo "$out" | grep -c '^IDENTICAL')" = 3 ] || fail=1
step "fish";          out="$(/src/scripts/fish-test.sh 2>&1)"; echo "$out"; echo "$out" | grep -q 'unload: gone' && echo "$out" | grep -q 'cd a: v=v20.20.2' || fail=1
step "read-only NVM_DIR"; out="$(/src/scripts/readonly-test.sh 2>&1)"; echo "$out"; echo "$out" | grep -q 'after use: /tmp/ro/versions/node/v20.11.0/bin/node bin=/tmp/ro/versions' || fail=1
step "Ctrl-C";        out="$(/src/scripts/sigint-test.sh 2>&1)"; echo "$out"; for w in 'download interrupted rc=130' 'resume rc=0' 'extract interrupted rc=130' 'files dir present after interrupt: no; version dir: no' 'nvm install rc=130' 'lock left behind: $'; do echo "$out" | grep -q "$w" || { echo "FAIL: missing '$w'"; fail=1; }; done
step "portability";   /src/scripts/portability.sh 2>&1 | tee /tmp/port.txt; grep -qE 'error|not found' /tmp/port.txt && fail=1
echo; [ $fail = 0 ] && echo "ALL GREEN" || echo "FAILURES PRESENT"
exit $fail
