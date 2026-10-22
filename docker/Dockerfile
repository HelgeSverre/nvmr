FROM rust:1-bookworm
RUN apt-get update && apt-get install -y --no-install-recommends zsh fish hyperfine strace xz-utils curl git ca-certificates procps ncurses-bin python3 \
 && rm -rf /var/lib/apt/lists/*
RUN git clone -q --depth 1 --branch v0.40.8 https://github.com/nvm-sh/nvm.git /opt/nvm-upstream
WORKDIR /src
