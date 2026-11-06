# Benchmark image: every tool and input pinned (bench/pins.env), no network at run time.
# Built by bench/docker.sh, which first puts the nvmr release bundle in bench/.build/nvmr.
# Keep the pin values in sync with pins.env (docker.sh passes them as build args from that file).
ARG DEBIAN_IMAGE
FROM ${DEBIAN_IMAGE} AS fetch
ARG DEBIAN_SNAPSHOT TARGETARCH NVM_VERSION NVM_COMMIT FNM_VERSION HYPERFINE_VERSION NODE_A NODE_B \
    FNM_SHA256_linux_arm64 FNM_SHA256_linux_x64 MISE_VERSION MISE_SHA256_linux_arm64 MISE_SHA256_linux_x64 \
    PROTO_VERSION PROTO_SHA256_linux_arm64 PROTO_SHA256_linux_x64 PROTO_NODE_PLUGIN_SHA256 PROTO_SCHEMA_PLUGIN_SHA256 \
    HYPERFINE_SHA256_linux_arm64 HYPERFINE_SHA256_linux_x64 \
    NODE_A_SHA256_linux_arm64 NODE_A_SHA256_linux_x64 NODE_B_SHA256_linux_arm64 NODE_B_SHA256_linux_x64
SHELL ["/bin/bash", "-euo", "pipefail", "-c"]
RUN printf 'deb [check-valid-until=no] http://snapshot.debian.org/archive/%s/%s %s main\n' \
      debian "$DEBIAN_SNAPSHOT" bookworm debian "$DEBIAN_SNAPSHOT" bookworm-updates debian-security "$DEBIAN_SNAPSHOT" bookworm-security \
      > /etc/apt/sources.list \
 && rm -f /etc/apt/sources.list.d/* \
 && apt-get -o Acquire::Retries=5 update && apt-get -y dist-upgrade && apt-get -o Acquire::Retries=5 install -y --no-install-recommends ca-certificates curl git unzip xz-utils \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /fetch
RUN case "$TARGETARCH" in arm64) a=arm64; ha=aarch64; fz=fnm-arm64.zip ;; amd64) a=x64; ha=x86_64; fz=fnm-linux.zip ;; *) exit 1 ;; esac \
 && fnm_sha="FNM_SHA256_linux_$a"; hf_sha="HYPERFINE_SHA256_linux_$a" \
 && curl -fsSL -o fnm.zip "https://github.com/Schniz/fnm/releases/download/$FNM_VERSION/$fz" \
 && echo "${!fnm_sha}  fnm.zip" | sha256sum -c - && unzip -q fnm.zip && chmod +x fnm \
 && mise_sha="MISE_SHA256_linux_$a" && proto_sha="PROTO_SHA256_linux_$a" \
 && curl -fsSL -o mise "https://github.com/jdx/mise/releases/download/$MISE_VERSION/mise-$MISE_VERSION-linux-$a" \
 && echo "${!mise_sha}  mise" | sha256sum -c - && chmod +x mise \
 && curl -fsSL -o proto.tar.xz "https://github.com/moonrepo/proto/releases/download/$PROTO_VERSION/proto_cli-$ha-unknown-linux-gnu.tar.xz" \
 && echo "${!proto_sha}  proto.tar.xz" | sha256sum -c - \
 && mkdir proto-dist && tar -xJf proto.tar.xz -C proto-dist \
 && curl -fsSL -o hf.tgz "https://github.com/sharkdp/hyperfine/releases/download/$HYPERFINE_VERSION/hyperfine-$HYPERFINE_VERSION-$ha-unknown-linux-gnu.tar.gz" \
 && echo "${!hf_sha}  hf.tgz" | sha256sum -c - && tar -xzf hf.tgz && mv hyperfine-*/hyperfine . \
 && mkdir dist && for k in NODE_A NODE_B; do v="${!k}"; s="${k}_SHA256_linux_$a"; f="node-$v-linux-$a.tar.xz"; \
      curl -fsSL -o "dist/$f" "https://nodejs.org/dist/$v/$f" && echo "${!s}  dist/$f" | sha256sum -c -; done \
 && git clone -q https://github.com/nvm-sh/nvm.git nvm && git -C nvm checkout -q "$NVM_COMMIT" \
 && test "$(git -C nvm describe --tags)" = "$NVM_VERSION" && rm -rf nvm/.git
RUN case "$TARGETARCH" in arm64) ha=aarch64 ;; amd64) ha=x86_64 ;; *) exit 1 ;; esac \
 && export PROTO_HOME=/fetch/proto-seed HOME=/fetch/proto-home \
 && proto="/fetch/proto-dist/proto_cli-$ha-unknown-linux-gnu/proto" \
 && export PATH="/fetch/proto-dist/proto_cli-$ha-unknown-linux-gnu:$PATH" \
 && "$proto" install node "${NODE_A#v}" >/dev/null \
 && "$proto" install node "${NODE_B#v}" >/dev/null \
 && printf 'node = "%s"\n' "${NODE_A#v}" > "$PROTO_HOME/.prototools" \
 && eval "$("$proto" activate bash --no-shim --config-mode all)" \
 && node_plugin=("$PROTO_HOME"/plugins/node-*.wasm) \
 && schema_plugin=("$PROTO_HOME"/plugins/internal-schema-*.wasm) \
 && echo "$PROTO_NODE_PLUGIN_SHA256  ${node_plugin[0]}" | sha256sum -c - \
 && echo "$PROTO_SCHEMA_PLUGIN_SHA256  ${schema_plugin[0]}" | sha256sum -c - \
 && rm -rf "$PROTO_HOME/tools/node/${NODE_A#v}" "$PROTO_HOME/tools/node/${NODE_B#v}"

FROM ${DEBIAN_IMAGE}
ARG DEBIAN_SNAPSHOT
ARG TARGETARCH
# The snapshot CDN intermittently 504s on this archived file. Preseed apt's cache with
# the same versioned Debian archive; apt still validates it against the pinned index.
ADD https://deb.debian.org/debian/pool/main/e/elfutils/libdw1_0.188-2.1_${TARGETARCH}.deb /tmp/libdw1.deb
RUN printf 'deb [check-valid-until=no] http://snapshot.debian.org/archive/%s/%s %s main\n' \
      debian "$DEBIAN_SNAPSHOT" bookworm debian "$DEBIAN_SNAPSHOT" bookworm-updates debian-security "$DEBIAN_SNAPSHOT" bookworm-security \
      > /etc/apt/sources.list \
 && rm -f /etc/apt/sources.list.d/* \
 && apt-get -o Acquire::Retries=5 update && apt-get -y dist-upgrade \
 && cp /tmp/libdw1.deb "/var/cache/apt/archives/libdw1_0.188-2.1_${TARGETARCH}.deb" \
 && apt-get -o Acquire::Retries=5 install -y --no-install-recommends bash zsh python3 procps strace time file curl xz-utils linux-perf ca-certificates \
 && rm -rf /var/lib/apt/lists/*
COPY --from=fetch /fetch/hyperfine /usr/local/bin/hyperfine
COPY --from=fetch /fetch/fnm /opt/bench/fnm
COPY --from=fetch /fetch/mise /opt/bench/mise
COPY --from=fetch /fetch/proto-dist /opt/bench/proto-dist
COPY --from=fetch /fetch/proto-seed /opt/bench/proto-seed
COPY --from=fetch /fetch/dist /opt/bench/dist
COPY --from=fetch /fetch/nvm /opt/bench/nvm
COPY .build/nvmr /opt/bench/nvmr
COPY pins.env bench.py /opt/bench/
ENV BENCH_WHERE=docker LANG=C LC_ALL=C
WORKDIR /opt/bench
