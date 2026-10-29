#!/bin/sh
# Inside the container: build static musl release binaries for this arch and assemble dist/nvmr-linux-<arch>/.
set -eu
arch="$(uname -m)"
target="${arch}-unknown-linux-musl"
cd /src
var="CC_$(echo "$target" | tr '-' '_')"
env "$var=musl-gcc" cargo build --release --target "$target" 2>&1 | tail -3
out=/src/dist/nvmr-linux-$arch
rm -rf "$out"; mkdir -p "$out"
cp "target/$target/release/nvmr" "target/$target/release/nvmr-fetch" nvm.sh nvm-exec bash_completion nvm.fish "$out/"
( cd /src/dist && tar -czf "nvmr-linux-$arch.tar.gz" -C "nvmr-linux-$arch" . )
ls -l "$out" | awk '{print $5, $9}'
file "$out/nvmr" 2>/dev/null || true
