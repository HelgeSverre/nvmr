# Benchmark report (docker cpuset=2,3 mem=2g, 2026-11-06T15:42:28+0000)

- git `v0.1.0-5-gea45aba-dirty`, seed 20261022, 3 rounds
- aarch64, 12 CPUs, Linux-7.2.12-orbstack-00394-gbecdd6130068-aarch64-with-glibc2.36
- cgroup: {'/sys/fs/cgroup/cpu.max': 'max 100000', '/sys/fs/cgroup/cpuset.cpus.effective': '2-3', '/sys/fs/cgroup/memory.max': '2147483648'}
- hyperfine 1.20.0; bash `GNU bash, version 5.2.15(1)-release (aarch64-unknown-linux-gnu)`; `zsh 5.9 (aarch64-unknown-linux-gnu)`; fnm `fnm 1.39.0`; nvm v0.40.8; nvmr `0.40.8`; mise `2026.11.3 linux-arm64 (2026-11-06)`; proto `proto 0.63.2`
- load average before/after: [2.53, 1.95, 1.4] / [1.23, 1.32, 1.34]

Per-operation numbers are loop totals minus the same shell's startup, divided by the operation count; `±` is the summed MAD of both, per operation: differences below it are not resolved. `min` is the min over every run of every round; `median` is the median of the per-round medians. Times in ms.

## Shell startup overhead (config minus bare shell)

`nvm-cd`/`nvmr-cd` add upstream's README auto-switch recipe; `nvmr-autocd` uses nvmr's one-process hook; `fnm` is `fnm env --use-on-cd`; `mise` uses `mise activate` with documented `.nvmrc` support; `proto-activate` uses `proto activate --no-shim --config-mode all`; `proto-shim` adds its shim directory.

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `startup/bash/bare` | – | – | – | 0.42 | 1.03 | 900 | 41% |
| `startup/bash/nvmr-autocd` | 0.83 | 3.80 | 2.01 | 1.24 | 4.83 | 900 | 83% |
| `startup/bash/fnm` | 0.50 | 1.38 | 0.65 | 0.91 | 2.41 | 900 | 12% |
| `startup/bash/fnm-recursive` | 0.48 | 1.83 | 0.87 | 0.90 | 2.86 | 900 | 37% |

## `cd` into/out of a project with `.nvmrc` (switches each time)

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `cd-switch/bash/nvmr-native` | 0.74 | 3.00 | 0.93 | 74.86 | 305 | 900 | 79% |
| `cd-switch/bash/fnm` | 0.38 | 1.27 | 0.20 | 39.09 | 130 | 900 | 25% |
| `cd-switch/bash/fnm-recursive` | 0.41 | 1.12 | 0.18 | 41.89 | 115 | 900 | 15% |

## `cd` between directories without a version file

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `cd-nochange/bash/nvmr-native` | 0.08 | 0.16 | 0.05 | 9.30 | 20.51 | 900 | 17% |
| `cd-nochange/bash/fnm` | 0.01 | 0.02 | 0.01 | 1.62 | 4.71 | 900 | 16% |
| `cd-nochange/bash/fnm-recursive` | 0.53 | 1.35 | 0.22 | 54.25 | 138 | 900 | 30% |
