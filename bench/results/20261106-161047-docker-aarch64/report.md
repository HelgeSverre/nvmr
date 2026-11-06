# Benchmark report (docker cpuset=2,3 mem=2g, 2026-11-06T15:10:55+0000)

- git `v0.1.0-4-g8f47a5e-dirty`, seed 20261022, 3 rounds
- aarch64, 12 CPUs, Linux-7.2.12-orbstack-00394-gbecdd6130068-aarch64-with-glibc2.36
- cgroup: {'/sys/fs/cgroup/cpu.max': 'max 100000', '/sys/fs/cgroup/cpuset.cpus.effective': '2-3', '/sys/fs/cgroup/memory.max': '2147483648'}
- hyperfine 1.20.0; bash `GNU bash, version 5.2.15(1)-release (aarch64-unknown-linux-gnu)`; `zsh 5.9 (aarch64-unknown-linux-gnu)`; fnm `fnm 1.39.0`; nvm v0.40.8; nvmr `0.40.8`; mise `2026.11.3 linux-arm64 (2026-11-06)`; proto `proto 0.63.2`
- load average before/after: [1.37, 1.01, 0.68] / [1.28, 1.33, 1.08]

Per-operation numbers are loop totals minus the same shell's startup, divided by the operation count; `±` is the summed MAD of both, per operation: differences below it are not resolved. `min` is the min over every run of every round; `median` is the median of the per-round medians. Times in ms.

## Shell startup overhead (config minus bare shell)

`nvm-cd`/`nvmr-cd` add upstream's README auto-switch recipe; `nvmr-autocd` uses nvmr's one-process hook; `fnm` is `fnm env --use-on-cd`; `mise` uses `mise activate` with documented `.nvmrc` support; `proto-activate` uses `proto activate --no-shim --config-mode all`; `proto-shim` adds its shim directory.

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `startup/bash/bare` | – | – | – | 0.35 | 0.44 | 900 | 86% |
| `startup/bash/bare-node` | 0.00 | -0.03 | 0.08 | 0.35 | 0.41 | 900 | 11% |
| `startup/bash/nvmr` | 0.63 | 2.62 | 1.65 | 0.98 | 3.06 | 900 | 128% |
| `startup/bash/nvmr-autocd` | 1.08 | 2.11 | 0.54 | 1.42 | 2.56 | 900 | 56% |
| `startup/bash/fnm` | 0.46 | 0.62 | 0.17 | 0.81 | 1.06 | 900 | 5% |
| `startup/bash/mise` | 5.46 | 6.16 | 0.48 | 5.81 | 6.61 | 900 | 9% |
| `startup/bash/proto-activate` | 28.68 | 32.63 | 1.52 | 29.03 | 33.08 | 900 | 7% |
| `startup/bash/proto-shim` | -0.00 | -0.00 | 0.09 | 0.35 | 0.44 | 900 | 82% |

## `ls` with 102 versions installed

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `ls-many/bash/nvmr` | 0.43 | 0.26 | 0.23 | 5.44 | 6.65 | 900 | 14% |
| `ls-many/bash/fnm` | 0.42 | 0.49 | 0.09 | 5.13 | 5.96 | 900 | 152% |
| `ls-many/bash/mise` | 4.33 | 4.75 | 1.03 | 49.76 | 58.38 | 900 | 3% |

## direct `node -e 0` execution (shim cost included)

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `node-exec/bash/bare-node` | 8.86 | 11.49 | 1.51 | 18.08 | 23.39 | 900 | 108% |
| `node-exec/bash/nvmr` | 9.38 | 11.31 | 1.60 | 19.73 | 25.69 | 900 | 6% |
| `node-exec/bash/fnm` | 9.07 | 11.87 | 2.14 | 18.96 | 24.81 | 900 | 92% |
| `node-exec/bash/mise` | 9.76 | 12.86 | 5.63 | 25.32 | 32.33 | 900 | 76% |
| `node-exec/bash/proto-activate` | 10.27 | 12.62 | 5.52 | 49.57 | 58.31 | 900 | 74% |
| `node-exec/bash/proto-shim` | 36.40 | 43.87 | 4.66 | 73.14 | 88.18 | 900 | 76% |

## `cd` into/out of a project with `.nvmrc` (switches each time)

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `cd-switch/bash/nvmr-native` | 0.44 | 1.07 | 0.56 | 45.38 | 109 | 900 | 113% |
| `cd-switch/bash/fnm` | 0.35 | 0.47 | 0.07 | 36.29 | 47.87 | 900 | 100% |
| `cd-switch/bash/mise` | 4.06 | 5.20 | 0.57 | 412 | 527 | 16 | 26% |

## `cd` between directories without a version file

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `cd-nochange/bash/nvmr-native` | 0.08 | 0.15 | 0.04 | 9.27 | 17.67 | 900 | 55% |
| `cd-nochange/bash/fnm` | 0.01 | 0.01 | 0.00 | 1.57 | 1.82 | 900 | 3% |
| `cd-nochange/bash/mise` | 3.77 | 8.68 | 2.84 | 383 | 874 | 19 | 57% |
