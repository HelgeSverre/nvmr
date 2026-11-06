# Benchmark report (docker cpuset=2,3 mem=2g, 2026-11-06T13:10:53+0000)

- git `v0.1.0-dirty`, seed 20261022, 3 rounds
- aarch64, 12 CPUs, Linux-7.2.12-orbstack-00394-gbecdd6130068-aarch64-with-glibc2.36
- cgroup: {'/sys/fs/cgroup/cpu.max': 'max 100000', '/sys/fs/cgroup/cpuset.cpus.effective': '2-3', '/sys/fs/cgroup/memory.max': '2147483648'}
- hyperfine 1.20.0; bash `GNU bash, version 5.2.15(1)-release (aarch64-unknown-linux-gnu)`; `zsh 5.9 (aarch64-unknown-linux-gnu)`; fnm `fnm 1.39.0`; nvm v0.40.8; nvmr `0.40.8`
- load average before/after: [1.2, 1.55, 1.69] / [1.52, 1.47, 1.61]

Per-operation numbers are loop totals minus the same shell's startup, divided by the operation count; `±` is the summed MAD of both, per operation: differences below it are not resolved. `min` is the min over every run of every round; `median` is the median of the per-round medians. Times in ms.

## Shell startup overhead (config minus bare shell)

`nvm-cd`/`nvmr-cd` add upstream's README auto-switch recipe; `nvmr-autocd` uses nvmr's one-process hook; `fnm` is `fnm env --use-on-cd`.

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `startup/bash/bare` | – | – | – | 0.34 | 0.37 | 900 | 9% |
| `startup/bash/nvmr` | 0.66 | 1.38 | 0.66 | 1.00 | 1.75 | 900 | 87% |
| `startup/bash/nvmr-autocd` | 0.79 | 2.15 | 0.30 | 1.13 | 2.52 | 900 | 62% |

## `use` (switch between two installed versions)

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `use/bash/nvmr` | 0.62 | 1.28 | 0.55 | 62.57 | 129 | 900 | 91% |

## `current`

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `current/bash/nvmr` | 0.17 | 0.20 | 0.02 | 18.03 | 21.37 | 900 | 9% |

## `ls` with 102 versions installed

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `ls-many/bash/nvm` | 506 | 663 | 77.68 | 554 | 718 | 15 | 22% |
| `ls-many/bash/nvmr` | 1.01 | 1.03 | 0.11 | 11.13 | 11.95 | 900 | 16% |

## `cd` into/out of a project with `.nvmrc` (switches each time)

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `cd-switch/bash/nvmr-native` | 0.24 | 1.13 | 0.64 | 24.88 | 116 | 900 | 126% |
