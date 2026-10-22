#!/bin/sh
# Benchmark inside the pinned image: build the nvm-fast release bundle (static musl, in the dev image),
# build bench/Dockerfile with every pin from bench/pins.env, run setup + run + profile with no network,
# pinned CPUs, a memory cap and a tmpfs work dir. Results land in bench/results/<date>-docker-<arch>/.
#
#   bench/docker.sh [bench.py run args...]        e.g. bench/docker.sh --quick --filter 'use|startup'
#   BENCH_CPUS=2,3 BENCH_MEM=2g                   CPU set and memory for the benchmark container
set -eu
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
B="$ROOT/bench"
cd "$ROOT"

# 1. nvm-fast exactly as shipped: the static release bundle.
scripts/d.sh sh -c 'touch src/*.rs src/bin/*.rs; sh /src/scripts/bundle.sh' </dev/null >/dev/null
arch="$(scripts/d.sh uname -m </dev/null)"
rm -rf "$B/.build" && mkdir -p "$B/.build"
cp -R "dist/nvm-fast-linux-$arch" "$B/.build/nvm-fast"

# 2. The image, with every pin passed as a build arg.
args=""
while IFS='=' read -r k v; do
  case "$k" in '' | \#*) continue ;; esac
  args="$args --build-arg $k=$v"
done <"$B/pins.env"
git_desc="$(git describe --always --dirty)"
tag="nvmfast-bench:$(git rev-parse --short HEAD)"
# shellcheck disable=SC2086
docker build -q $args -t "$tag" "$B" >/dev/null

# 3. Run.
out="$B/results/$(date +%Y%m%d-%H%M%S)-docker-$arch"
mkdir -p "$out"
docker run --rm --network none --cpuset-cpus "${BENCH_CPUS:-2,3}" --memory "${BENCH_MEM:-2g}" \
  --tmpfs /work:rw,exec,size=3g --security-opt seccomp=unconfined --cap-add SYS_PTRACE \
  -e BENCH_GIT="$git_desc" -e BENCH_WHERE="docker cpuset=${BENCH_CPUS:-2,3} mem=${BENCH_MEM:-2g}" \
  -v "$out":/out "$tag" sh -c '
    set -e
    python3 bench.py setup --root /work/root --nvm-src nvm --fast-src nvm-fast --fnm fnm --dist dist >/dev/null
    python3 bench.py run --root /work/root --out /out "$@"
    python3 bench.py profile --root /work/root --out /out >/dev/null' sh "$@"
echo "results: $out"
