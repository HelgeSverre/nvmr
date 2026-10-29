# Benchmark image: every tool and input pinned (bench/pins.env), no network at run time.
# Built by bench/docker.sh, which first puts the nvmr release bundle in bench/.build/nvmr.
# Keep the pin values in sync with pins.env (docker.sh passes them as build args from that file).
ARG DEBIAN_IMAGE
FROM ${DEBIAN_IMAGE} AS fetch
ARG DEBIAN_SNAPSHOT TARGETARCH NVM_VERSION NVM_COMMIT FNM_VERSION HYPERFINE_VERSION NODE_A NODE_B \
    FNM_SHA256_linux_arm64 FNM_SHA256_linux_x64 HYPERFINE_SHA256_linux_arm64 HYPERFINE_SHA256_linux_x64 \
    NODE_A_SHA256_linux_arm64 NODE_A_SHA256_linux_x64 NODE_B_SHA256_linux_arm64 NODE_B_SHA256_linux_x64
SHELL ["/bin/bash", "-euo", "pipefail", "-c"]
RUN printf 'deb [check-valid-until=no] http://snapshot.debian.org/archive/%s/%s %s main\n' \
      debian "$DEBIAN_SNAPSHOT" bookworm debian "$DEBIAN_SNAPSHOT" bookworm-updates debian-security "$DEBIAN_SNAPSHOT" bookworm-security \
      > /etc/apt/sources.list \
 && rm -f /etc/apt/sources.list.d/* \
 && apt-get -o Acquire::Retries=5 update && apt-get -y dist-upgrade && apt-get install -y --no-install-recommends ca-certificates curl git unzip xz-utils \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /fetch
RUN case "$TARGETARCH" in arm64) a=arm64; ha=aarch64; fz=fnm-arm64.zip ;; amd64) a=x64; ha=x86_64; fz=fnm-linux.zip ;; *) exit 1 ;; esac \
 && fnm_sha="FNM_SHA256_linux_$a"; hf_sha="HYPERFINE_SHA256_linux_$a" \
 && curl -fsSL -o fnm.zip "https://github.com/Schniz/fnm/releases/download/$FNM_VERSION/$fz" \
 && echo "${!fnm_sha}  fnm.zip" | sha256sum -c - && unzip -q fnm.zip && chmod +x fnm \
 && curl -fsSL -o hf.tgz "https://github.com/sharkdp/hyperfine/releases/download/$HYPERFINE_VERSION/hyperfine-$HYPERFINE_VERSION-$ha-unknown-linux-gnu.tar.gz" \
 && echo "${!hf_sha}  hf.tgz" | sha256sum -c - && tar -xzf hf.tgz && mv hyperfine-*/hyperfine . \
 && mkdir dist && for k in NODE_A NODE_B; do v="${!k}"; s="${k}_SHA256_linux_$a"; f="node-$v-linux-$a.tar.xz"; \
      curl -fsSL -o "dist/$f" "https://nodejs.org/dist/$v/$f" && echo "${!s}  dist/$f" | sha256sum -c -; done \
 && git clone -q https://github.com/nvm-sh/nvm.git nvm && git -C nvm checkout -q "$NVM_COMMIT" \
 && test "$(git -C nvm describe --tags)" = "$NVM_VERSION" && rm -rf nvm/.git

FROM ${DEBIAN_IMAGE}
ARG DEBIAN_SNAPSHOT
RUN printf 'deb [check-valid-until=no] http://snapshot.debian.org/archive/%s/%s %s main\n' \
      debian "$DEBIAN_SNAPSHOT" bookworm debian "$DEBIAN_SNAPSHOT" bookworm-updates debian-security "$DEBIAN_SNAPSHOT" bookworm-security \
      > /etc/apt/sources.list \
 && rm -f /etc/apt/sources.list.d/* \
 && apt-get -o Acquire::Retries=5 update && apt-get -y dist-upgrade \
 && apt-get install -y --no-install-recommends bash zsh python3 procps strace time file curl xz-utils linux-perf \
 && rm -rf /var/lib/apt/lists/*
COPY --from=fetch /fetch/hyperfine /usr/local/bin/hyperfine
COPY --from=fetch /fetch/fnm /opt/bench/fnm
COPY --from=fetch /fetch/dist /opt/bench/dist
COPY --from=fetch /fetch/nvm /opt/bench/nvm
COPY .build/nvmr /opt/bench/nvmr
COPY pins.env bench.py /opt/bench/
ENV BENCH_WHERE=docker LANG=C LC_ALL=C
WORKDIR /opt/bench
