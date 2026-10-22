#!/bin/sh
# Real-world benchmark on this machine (macOS arm64 or Linux), in an isolated sandbox: its own HOME, NVM_DIR
# and FNM_DIR under bench/.sandbox, pinned nvm/fnm/node downloads verified by checksum, nvm-fast built from
# this checkout with `cargo build --release`. Your own ~/.nvm, fnm setup and shell profiles are never touched.
# Results land in bench/results/<date>-host-<os>-<arch>/.
#
#   bench/host.sh [bench.py run args...]          e.g. bench/host.sh --quick
set -eu
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
B="$ROOT/bench"
S="$B/.sandbox/host"
# shellcheck disable=SC1091
. "$B/pins.env"

case "$(uname -s)-$(uname -m)" in
  Darwin-arm64) plat=darwin-arm64; fz=fnm-macos.zip; fnm_sha=$FNM_SHA256_darwin_arm64 ;;
  Linux-aarch64) plat=linux-arm64; fz=fnm-arm64.zip; fnm_sha=$FNM_SHA256_linux_arm64 ;;
  Linux-x86_64) plat=linux-x64; fz=fnm-linux.zip; fnm_sha=$FNM_SHA256_linux_x64 ;;
  *) echo "unsupported host" >&2; exit 1 ;;
esac
sha256() { if command -v sha256sum >/dev/null; then sha256sum "$1"; else shasum -a 256 "$1"; fi | cut -d' ' -f1; }
fetch() { # url dest sha
  [ -f "$2" ] && [ "$(sha256 "$2")" = "$3" ] && return 0
  curl -fsSL -o "$2.part" "$1"
  [ "$(sha256 "$2.part")" = "$3" ] || { echo "checksum mismatch: $1" >&2; exit 1; }
  mv "$2.part" "$2"
}

hv="$(hyperfine --version 2>/dev/null | cut -d' ' -f2)"
[ "v$hv" = "$HYPERFINE_VERSION" ] || { echo "need hyperfine $HYPERFINE_VERSION on PATH (found: ${hv:-none})" >&2; exit 1; }

mkdir -p "$S/dist" "$S/fnm-bin"
for k in A B; do
  eval "v=\$NODE_$k sha=\$NODE_${k}_SHA256_$(echo $plat | tr - _)"
  fetch "https://nodejs.org/dist/$v/node-$v-$plat.tar.xz" "$S/dist/node-$v-$plat.tar.xz" "$sha"
done
fetch "https://github.com/Schniz/fnm/releases/download/$FNM_VERSION/$fz" "$S/fnm.zip" "$fnm_sha"
(cd "$S/fnm-bin" && unzip -oq ../fnm.zip && chmod +x fnm)
if [ ! -d "$S/nvm" ]; then
  git clone -q https://github.com/nvm-sh/nvm.git "$S/nvm"
fi
git -C "$S/nvm" checkout -q "$NVM_COMMIT"

# nvm-fast from this checkout, laid out like an installed bundle.
CARGO_TARGET_DIR="$S/target" cargo build --release --quiet --manifest-path "$ROOT/Cargo.toml"
rm -rf "$S/nvm-fast" && mkdir -p "$S/nvm-fast"
cp "$S/target/release/nvm-core" "$S/target/release/nvm-fetch" "$ROOT/nvm.sh" "$ROOT/nvm-exec" \
  "$ROOT/bash_completion" "$ROOT/nvm.fish" "$S/nvm-fast/"

out="$B/results/$(date +%Y%m%d-%H%M%S)-host-$plat"
mkdir -p "$out"
cd "$B"
python3 bench.py setup --root "$S/root" --nvm-src "$S/nvm" --fast-src "$S/nvm-fast" --fnm "$S/fnm-bin/fnm" \
  --dist "$S/dist" >/dev/null
BENCH_GIT="$(git -C "$ROOT" describe --always --dirty)" BENCH_WHERE="host $(uname -sm)" \
  python3 bench.py run --root "$S/root" --out "$out" --bash /bin/bash --zsh /bin/zsh "$@"
python3 bench.py profile --root "$S/root" --out "$out" >/dev/null
echo "results: $out"
