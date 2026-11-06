# Benchmark report (docker cpuset=2,3 mem=2g, 2026-11-06T16:39:37+0000)

- git `v0.1.0-6-g22e5c55`, seed 20261022, 3 rounds
- aarch64, 12 CPUs, Linux-7.2.12-orbstack-00394-gbecdd6130068-aarch64-with-glibc2.36
- cgroup: {'/sys/fs/cgroup/cpu.max': 'max 100000', '/sys/fs/cgroup/cpuset.cpus.effective': '2-3', '/sys/fs/cgroup/memory.max': '2147483648'}
- hyperfine 1.20.0; bash `GNU bash, version 5.2.15(1)-release (aarch64-unknown-linux-gnu)`; `zsh 5.9 (aarch64-unknown-linux-gnu)`; fnm `fnm 1.39.0`; nvm v0.40.8; nvmr `0.40.8`; mise `2026.11.3 linux-arm64 (2026-11-06)`; proto `proto 0.63.2`
- load average before/after: [2.66, 2.29, 1.89] / [1.99, 2.14, 1.88]

Per-operation numbers are loop totals minus the same shell's startup, divided by the operation count; `±` is the summed MAD of both, per operation: differences below it are not resolved. `min` is the min over every run of every round; `median` is the median of the per-round medians. Times in ms.

## Shell startup overhead (config minus bare shell)

`nvm-cd`/`nvmr-cd` add upstream's README auto-switch recipe; `nvmr-autocd` uses nvmr's one-process hook; `fnm` uses the default local `fnm env --use-on-cd`; `fnm-recursive` enables parent lookup; `mise` uses `mise activate` with documented `.nvmrc` support; `proto-activate` uses `proto activate --no-shim --config-mode all`; `proto-shim` adds its shim directory.

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `startup/bash/bare` | – | – | – | 0.33 | 0.73 | 900 | 98% |
| `startup/bash/nvmr-autocd` | 0.76 | 1.63 | 0.46 | 1.09 | 2.37 | 900 | 33% |
| `startup/bash/fnm` | 0.44 | 1.18 | 0.31 | 0.77 | 1.92 | 900 | 142% |
| `startup/bash/fnm-recursive` | 0.47 | 1.19 | 0.24 | 0.80 | 1.92 | 900 | 110% |

## `cd` between directories without a version file

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `cd-nochange/bash/nvmr-native` | 0.07 | 0.14 | 0.01 | 8.47 | 16.28 | 900 | 84% |
| `cd-nochange/bash/fnm` | 0.01 | 0.00 | 0.00 | 1.54 | 1.93 | 900 | 21% |
| `cd-nochange/bash/fnm-recursive` | 0.35 | 0.39 | 0.08 | 36.22 | 41.22 | 900 | 157% |
