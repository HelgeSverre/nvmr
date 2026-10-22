FROM rust:1-bookworm
RUN apt-get update && apt-get install -y --no-install-recommends zsh fish hyperfine strace musl-tools xz-utils curl git ca-certificates procps ncurses-bin python3 \
 && rm -rf /var/lib/apt/lists/*
RUN rustup target add aarch64-unknown-linux-musl x86_64-unknown-linux-musl && rustup component add clippy rustfmt
RUN git clone -q --depth 1 --branch v0.40.8 https://github.com/nvm-sh/nvm.git /opt/nvm-upstream
WORKDIR /src
