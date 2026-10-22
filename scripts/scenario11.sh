# Edge cases ported from upstream nvm's own test suite (test/fast: Aliases, Listing versions/paths).
t() { printf '=== nvm %s\n' "$*"; nvm "$@" 2>&1; echo "rc=$?"; }
mkn() { # fake node in the layout nvm_version_path picks (old layout below 0.12)
  for v in "$@"; do
    case $v in v0.[0-9].*|v0.1[01].*) d=$NVM_DIR/$v/bin;; *) d=$NVM_DIR/versions/node/$v/bin;; esac
    mkdir -p $d; printf '#!/bin/sh\necho %s\n' $v > $d/node; chmod +x $d/node
  done
}
mki() { for v in "$@"; do d=$NVM_DIR/versions/io.js/$v/bin; mkdir -p $d; for b in node iojs; do printf '#!/bin/sh\necho %s\n' $v > $d/$b; chmod +x $d/$b; done; done; }
A=$NVM_DIR/alias
mkdir -p $A $NVM_DIR/src && cd /tmp
# --- Aliases/setup: test-stable-N, test-unstable-N, test-iojs-N
for i in 1 2 3 4 5 6 7 8 9 10; do
  echo 0.0.$i > $A/test-stable-$i; mkn v0.0.$i
  echo 0.1.$i > $A/test-unstable-$i; mkn v0.1.$i
  echo 0.2.$i > $A/test-iojs-$i; mki v0.2.$i
done
t alias
t alias test-stable-1
t alias test-
t alias test-stable-1 0.0.2
t alias test-stable-1
t alias test-stable-1 0.0.1
t alias test-stable-1
t version stable
t version unstable
t version node
t version iojs
t ls stable
t ls unstable
t ls node
t ls iojs
# hash / slash in alias names
t alias foo#bar baz
t alias foo# baz
t alias '#bar' baz
t alias foo/bar baz
t alias foo/ baz
t alias /bar baz
t unalias foo/bar
t unalias foo/
t unalias /bar
# built-in alias names
for n in node stable unstable iojs system; do t unalias $n; done
t alias node stable
t alias node
t unalias node
t unalias node
# manual aliases overriding implicit ones
mkn v0.8.1 v0.9.1; mki v0.2.1
t alias stable 0.1
t alias unstable 0.0
t alias node stable
t alias iojs unstable
t alias
t version node
t version iojs
t ls stable
t use node
t use iojs
nvm deactivate >/dev/null 2>&1
rm -f $A/stable $A/unstable $A/node $A/iojs
# leading blank lines / multi-line alias files
printf '\n\nv0.0.1\n\n' > $A/test-blank-lines
printf '\n\nv0.0.1\nv0.0.2\nv0.0.3\n\n' > $A/test-multi-lines
t alias test-blank-lines
t version test-blank-lines
t version test-multi-lines
t which test-blank-lines
# edge-case alias files
printf 'v0.0.5\n' > "$A/test edge spaces"
printf 'v0.0.5\r\n' > $A/test-edge-cr
printf 'v0.0.6\r' > $A/test-edge-cr-bare
printf '# this is a comment\n# another comment\n' > $A/test-edge-comments
: > $A/test-edge-empty
printf 'v0.0.5\f\n' > $A/test-edge-ff
printf 'v0.0.6\v\n' > $A/test-edge-vt
printf 'v0.0.5' > $A/test-edge-no-newline
printf 'v0.0.5 #%s\n' "$(printf '%0*d' 10000 0 | tr 0 x)" > $A/test-edge-long
printf '0.0.5   \t \n' > $A/test-edge-trailing-ws
echo 99.99.99 > $A/test-edge-noexist
for n in "test edge spaces" test-edge-cr test-edge-cr-bare test-edge-comments test-edge-empty test-edge-ff test-edge-vt test-edge-no-newline test-edge-long test-edge-trailing-ws test-edge-noexist; do
  t version "$n"; t which "$n"; t use --silent "$n"
done
t alias test-edge-
# circular aliases
echo loopback > $A/loopback; echo two > $A/one; echo three > $A/two; echo one > $A/three; echo two > $A/four
for n in loopback one two three four; do t alias $n; done
t use loopback
t use one
t which loopback
t which four
t version four
t ls loopback
t run loopback --version
t exec one node
echo self > $A/self
echo link2 > $A/link1; echo link3 > $A/link2; echo link1 > $A/link3
echo midway > "$A/foo bar"; echo "foo bar" > $A/midway
t alias self
t alias link1
t version "foo bar"
t use "foo bar"
echo bar > $A/midway; echo 0.0.9 > $A/bar
t version "foo bar"
t alias "foo bar"
echo a.b > $A/axb; echo 0.0.8 > $A/a.b
t version axb
echo axb > $A/a.b
t version axb
t alias axb
echo hop2 > $A/hop1; echo hop3 > $A/hop2; echo hop4 > $A/hop3; echo 0.0.7 > $A/hop4
t version hop1
t alias hop1
t use hop1
nvm deactivate >/dev/null 2>&1
# percent in alias name
echo v0.0.1 > "$A/test-%s-alias"; echo 'test-%s-alias' > $A/test-pct-chain
t version 'test-%s-alias'
t version test-pct-chain
t alias test-pct-chain
t alias 'test-%s-alias'
# uppercase
t alias UPPER_ALIAS v0.0.1
t alias MixedCase v0.0.1
t version UPPER_ALIAS
t alias MixedCase
t version lts/ARGON
ls $A
# LTS alias dir is ensured by `nvm alias`
rm -rf $A/lts
t alias test-stable-10
[ -d $A/lts ] && echo "lts dir exists" || echo "lts dir missing"
# nvm_ensure_default_set via install path is network-only; default alias creation/removal:
t alias default 0.1
t unalias default
t unalias default
t alias default 0.2
t alias default
