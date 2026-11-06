# Benchmark report (docker cpuset=2,3 mem=2g, 2026-11-06T12:50:05+0000)

- git `v0.1.0-dirty`, seed 20261022, 3 rounds
- aarch64, 12 CPUs, Linux-7.2.12-orbstack-00394-gbecdd6130068-aarch64-with-glibc2.36
- cgroup: {'/sys/fs/cgroup/cpu.max': 'max 100000', '/sys/fs/cgroup/cpuset.cpus.effective': '2-3', '/sys/fs/cgroup/memory.max': '2147483648'}
- hyperfine 1.20.0; bash `GNU bash, version 5.2.15(1)-release (aarch64-unknown-linux-gnu)`; `zsh 5.9 (aarch64-unknown-linux-gnu)`; fnm `fnm 1.39.0`; nvm v0.40.8; nvmr `0.40.8`
- load average before/after: [1.36, 1.47, 2.01] / [1.36, 1.56, 1.7]

Per-operation numbers are loop totals minus the same shell's startup, divided by the operation count; `±` is the summed MAD of both, per operation: differences below it are not resolved. `min` is the min over every run of every round; `median` is the median of the per-round medians. Times in ms.

## Shell startup overhead (config minus bare shell)

`nvm-cd`/`nvmr-cd` add upstream's README auto-switch recipe; `nvmr-autocd` uses nvmr's one-process hook; `fnm` is `fnm env --use-on-cd`.

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `startup/bash/bare` | – | – | – | 0.32 | 0.37 | 900 | 9% |
| `startup/bash/nvm` | 48.16 | 56.41 | 2.85 | 48.49 | 56.79 | 900 | 4% |
| `startup/bash/nvm-cd` | 123 | 150 | 14.40 | 123 | 150 | 900 | 19% |
| `startup/bash/nvmr` | 0.98 | 1.68 | 0.54 | 1.30 | 2.06 | 900 | 59% |
| `startup/bash/nvmr-autocd` | 0.69 | 0.76 | 0.11 | 1.02 | 1.14 | 900 | 35% |
| `startup/bash/fnm` | 0.49 | 0.67 | 0.10 | 0.82 | 1.04 | 900 | 6% |
| `startup/zsh/bare` | – | – | – | 0.50 | 0.61 | 900 | 9% |
| `startup/zsh/nvm` | 140 | 162 | 15.68 | 140 | 163 | 900 | 64% |
| `startup/zsh/nvmr` | 0.64 | 0.96 | 0.57 | 1.14 | 1.57 | 900 | 66% |
| `startup/zsh/fnm` | 0.66 | 0.91 | 0.18 | 1.16 | 1.51 | 900 | 16% |

## `use` (switch between two installed versions)

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `use/bash/nvm` | 86.46 | 100 | 12.29 | 1432 | 1664 | 15 | 41% |
| `use/bash/nvmr` | 0.22 | 0.47 | 0.16 | 23.69 | 49.47 | 900 | 82% |
| `use/bash/fnm` | 0.34 | 0.39 | 0.03 | 34.83 | 40.27 | 900 | 12% |

## `current`

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `current/bash/nvm` | 3.39 | 3.93 | 0.68 | 116 | 135 | 900 | 6% |
| `current/bash/nvmr` | 0.15 | 0.17 | 0.02 | 16.58 | 19.44 | 900 | 88% |
| `current/bash/fnm` | 0.34 | 0.38 | 0.02 | 34.36 | 39.32 | 900 | 11% |

## `version <major>` (resolution)

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `version/bash/nvm` | 17.35 | 21.52 | 3.38 | 222 | 272 | 34 | 24% |
| `version/bash/nvmr` | 0.16 | 0.19 | 0.02 | 17.14 | 21.33 | 900 | 9% |

## `ls` with 102 versions installed

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `ls-many/bash/nvm` | 540 | 679 | 104 | 588 | 733 | 15 | 17% |
| `ls-many/bash/nvmr` | 1.04 | 1.13 | 0.10 | 11.48 | 13.05 | 900 | 4% |
| `ls-many/bash/fnm` | 0.40 | 0.47 | 0.04 | 4.90 | 5.84 | 900 | 9% |

## `cd` into/out of a project with `.nvmrc` (switches each time)

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `cd-switch/bash/nvm` | 126 | 156 | 15.49 | 1135 | 1395 | 15 | 15% |
| `cd-switch/bash/nvmr-native` | 0.40 | 0.74 | 0.24 | 40.73 | 74.79 | 900 | 115% |
| `cd-switch/bash/fnm` | 0.35 | 0.39 | 0.03 | 35.53 | 40.48 | 900 | 9% |

## `cd` between directories without a version file

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `cd-nochange/bash/nvm` | 31.82 | 34.54 | 4.97 | 1396 | 1532 | 15 | 75% |
| `cd-nochange/bash/nvmr-native` | 0.16 | 0.21 | 0.01 | 17.43 | 21.97 | 900 | 7% |
| `cd-nochange/bash/fnm` | 0.01 | 0.01 | 0.00 | 1.57 | 1.80 | 900 | 9% |

## install from a loopback mirror (download + verify + extract)

| benchmark | min | p10 | median | MAD | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|
| `install/bash/nvm` | 1576 | 1591 | 1692 | 88.99 | 15 | 20% |
| `install/bash/nvmr` | 1373 | 1373 | 1410 | 27.85 | 15 | 1% |
| `install/bash/fnm` | 1310 | 1310 | 1373 | 19.41 | 15 | 5% |

## Bare process latency

| benchmark | min | p10 | median | MAD | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|
| `process/nvmr-current` | 0.07 | 0.09 | 0.10 | 0.01 | 900 | 7% |
| `process/fnm-ls` | 0.25 | 0.28 | 0.31 | 0.02 | 900 | 6% |
| `process/nvmr-ls` | 0.15 | 0.17 | 0.19 | 0.01 | 900 | 4% |
