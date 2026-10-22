#!/bin/sh
# Inside the container: build static musl release binaries for this arch and assemble dist/nvm-fast-linux-<arch>/.
set -eu
arch="$(uname -m)"
target="${arch}-unknown-linux-musl"
cd /src
var="CC_$(echo "$target" | tr '-' '_')"
env "$var=musl-gcc" cargo build --release --target "$target" 2>&1 | tail -3
out=/src/dist/nvm-fast-linux-$arch
rm -rf "$out"; mkdir -p "$out"
cp "target/$target/release/nvm-core" "target/$target/release/nvm-fetch" nvm.sh nvm-exec bash_completion nvm.fish "$out/"
( cd /src/dist && tar -czf "nvm-fast-linux-$arch.tar.gz" -C "nvm-fast-linux-$arch" . )
ls -l "$out" | awk '{print $5, $9}'
file "$out/nvm-core" 2>/dev/null || true
