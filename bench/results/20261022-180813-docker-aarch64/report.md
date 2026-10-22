# Benchmark report (docker cpuset=2,3 mem=2g, 2026-10-22T16:08:24+0000)

- git `85808c9-dirty`, seed 20261022, 3 rounds
- aarch64, 12 CPUs, Linux-7.2.9-orbstack-00391-g2b298d05fbd4-aarch64-with-glibc2.36
- cgroup: {'/sys/fs/cgroup/cpu.max': 'max 100000', '/sys/fs/cgroup/cpuset.cpus.effective': '2-3', '/sys/fs/cgroup/memory.max': '2147483648'}
- hyperfine 1.20.0; bash `GNU bash, version 5.2.15(1)-release (aarch64-unknown-linux-gnu)`; `zsh 5.9 (aarch64-unknown-linux-gnu)`; fnm `fnm 1.39.0`; nvm v0.40.8; nvm-fast `0.40.8`
- load average before/after: [4.26, 1.56, 0.56] / [4.88, 4.14, 3.45]  **NOISY: load exceeded ncpu/4; trust min and p10, not medians**

Per-operation numbers are loop totals minus the same shell's startup, divided by the operation count; `±` is the summed MAD of both, per operation: differences below it are not resolved. `min` is the min over every run of every round; `median` is the median of the per-round medians. Times in ms.

## Shell startup overhead (config minus bare shell)

`nvm-cd`/`fast-cd` add upstream's README auto-switch recipe; `fast-autocd` uses nvm-fast's one-process hook; `fnm` is `fnm env --use-on-cd`.

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `startup/bash/bare` | – | – | – | 0.40 | 0.65 | 900 | 29% |
| `startup/bash/nvm` | 52.87 | 81.40 | 9.19 | 53.27 | 82.04 | 150 | 12% |
| `startup/bash/nvm-nouse` | 5.85 | 6.89 | 0.80 | 6.25 | 7.54 | 900 | 20% |
| `startup/bash/nvm-cd` | 190 | 255 | 28.69 | 191 | 256 | 150 | 25% |
| `startup/bash/fast` | 0.66 | 1.48 | 0.86 | 1.06 | 2.13 | 900 | 84% |
| `startup/bash/fast-nouse` | 0.16 | 0.14 | 0.22 | 0.56 | 0.79 | 900 | 25% |
| `startup/bash/fast-cd` | 3.85 | 5.65 | 0.89 | 4.25 | 6.30 | 900 | 11% |
| `startup/bash/fast-autocd` | 1.11 | 2.25 | 0.88 | 1.50 | 2.90 | 900 | 69% |
| `startup/bash/fnm` | 0.57 | 0.93 | 0.42 | 0.97 | 1.58 | 900 | 23% |
| `startup/zsh/bare` | – | – | – | 0.51 | 0.65 | 900 | 23% |
| `startup/zsh/nvm` | 187 | 271 | 35.82 | 188 | 272 | 150 | 11% |
| `startup/zsh/nvm-nouse` | 75.25 | 83.94 | 4.01 | 75.76 | 84.59 | 150 | 6% |
| `startup/zsh/nvm-cd` | 185 | 252 | 31.24 | 186 | 253 | 150 | 29% |
| `startup/zsh/fast` | 0.91 | 1.54 | 0.74 | 1.42 | 2.19 | 900 | 64% |
| `startup/zsh/fast-nouse` | 0.25 | 0.36 | 0.20 | 0.75 | 1.01 | 900 | 12% |
| `startup/zsh/fast-cd` | 2.51 | 4.80 | 1.11 | 3.02 | 5.45 | 900 | 48% |
| `startup/zsh/fast-autocd` | 1.26 | 2.14 | 0.72 | 1.77 | 2.79 | 900 | 75% |
| `startup/zsh/fnm` | 0.87 | 1.25 | 0.35 | 1.38 | 1.90 | 900 | 18% |

## `use` (switch between two installed versions)

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `use/bash/nvm` | 108 | 142 | 15.36 | 1783 | 2348 | 15 | 3% |
| `use/bash/fast` | 0.70 | 1.91 | 0.36 | 71.41 | 193 | 150 | 66% |
| `use/bash/fnm` | 0.46 | 0.73 | 0.09 | 46.54 | 74.24 | 150 | 10% |
| `use/zsh/nvm` | 145 | 155 | 12.82 | 2501 | 2752 | 15 | 5% |
| `use/zsh/fast` | 1.28 | 1.89 | 0.53 | 130 | 192 | 150 | 68% |
| `use/zsh/fnm` | 0.51 | 0.87 | 0.20 | 52.70 | 88.86 | 150 | 42% |

## `current`

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `current/bash/nvm` | 5.70 | 5.83 | 1.18 | 167 | 199 | 39 | 15% |
| `current/bash/fast` | 0.32 | 0.71 | 0.20 | 33.48 | 73.27 | 183 | 40% |
| `current/bash/fnm` | 0.38 | 0.66 | 0.14 | 39.01 | 67.34 | 178 | 74% |
| `current/zsh/nvm` | 4.16 | 6.52 | 4.67 | 271 | 402 | 150 | 28% |
| `current/zsh/fast` | 0.30 | 0.48 | 0.10 | 31.57 | 50.41 | 184 | 52% |
| `current/zsh/fnm` | 0.47 | 0.75 | 0.17 | 47.91 | 77.07 | 150 | 97% |

## `version <major>` (resolution)

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `version/bash/nvm` | 24.94 | 37.45 | 12.53 | 303 | 457 | 20 | 227% |
| `version/bash/fast` | 0.39 | 0.68 | 0.17 | 39.85 | 69.88 | 164 | 57% |
| `version/zsh/nvm` | 37.34 | 40.99 | 11.08 | 561 | 681 | 15 | 24% |
| `version/zsh/fast` | 0.37 | 0.67 | 0.12 | 38.85 | 69.19 | 166 | 35% |

## `ls` with 102 versions installed

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `ls-many/bash/nvm` | 772 | 946 | 102 | 824 | 1046 | 15 | 16% |
| `ls-many/bash/fast` | 1.19 | 1.69 | 0.41 | 13.60 | 19.65 | 520 | 38% |
| `ls-many/bash/fnm` | 0.59 | 0.80 | 0.20 | 6.86 | 10.09 | 705 | 79% |
| `ls-many/zsh/nvm` | 888 | 1142 | 162 | 1059 | 1373 | 15 | 22% |
| `ls-many/zsh/fast` | 1.29 | 1.89 | 0.33 | 14.55 | 21.62 | 452 | 34% |
| `ls-many/zsh/fnm` | 0.49 | 0.85 | 0.21 | 6.30 | 10.73 | 854 | 18% |

## `cd` into/out of a project with `.nvmrc` (switches each time)

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `cd-switch/bash/nvm` | 181 | 219 | 33.91 | 1642 | 2010 | 15 | 22% |
| `cd-switch/bash/fast` | 2.91 | 4.37 | 0.75 | 120 | 181 | 150 | 36% |
| `cd-switch/bash/fast-native` | 0.74 | 1.95 | 0.77 | 75.42 | 198 | 150 | 90% |
| `cd-switch/bash/fnm` | 0.38 | 0.64 | 0.12 | 39.16 | 65.69 | 150 | 47% |
| `cd-switch/zsh/nvm` | 185 | 259 | 38.31 | 1665 | 2324 | 15 | 30% |
| `cd-switch/zsh/fast` | 3.91 | 6.47 | 1.26 | 159 | 264 | 150 | 53% |
| `cd-switch/zsh/fast-native` | 0.62 | 1.75 | 1.05 | 63.82 | 178 | 150 | 137% |
| `cd-switch/zsh/fnm` | 0.59 | 0.86 | 0.11 | 60.42 | 87.96 | 150 | 20% |

## `cd` between directories without a version file

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `cd-nochange/bash/nvm` | 44.89 | 73.50 | 8.54 | 1986 | 3196 | 15 | 16% |
| `cd-nochange/bash/fast` | 1.49 | 2.78 | 0.31 | 124 | 228 | 150 | 29% |
| `cd-nochange/bash/fast-native` | 0.38 | 0.70 | 0.11 | 39.17 | 73.28 | 176 | 24% |
| `cd-nochange/bash/fnm` | 0.01 | 0.01 | 0.01 | 1.75 | 2.30 | 900 | 4% |
| `cd-nochange/zsh/nvm` | 1.97 | 4.46 | 2.57 | 265 | 431 | 23 | 20% |
| `cd-nochange/zsh/fast` | 1.41 | 2.88 | 0.64 | 116 | 236 | 150 | 38% |
| `cd-nochange/zsh/fast-native` | 0.41 | 0.51 | 0.11 | 43.25 | 54.22 | 168 | 45% |
| `cd-nochange/zsh/fnm` | 0.01 | 0.01 | 0.01 | 2.16 | 2.81 | 900 | 16% |

## install from a loopback mirror (download + verify + extract)

| benchmark | min | p10 | median | MAD | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|
| `install/bash/nvm` | 2161 | 2164 | 2436 | 214 | 15 | 9% |
| `install/bash/fast` | 1482 | 1517 | 1798 | 153 | 15 | 15% |
| `install/bash/fnm` | 1479 | 1496 | 1807 | 34.19 | 15 | 1% |

## Bare process latency

| benchmark | min | p10 | median | MAD | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|
| `process/nvm-core-current` | 0.06 | 0.12 | 0.16 | 0.05 | 900 | 149% |
| `process/nvm-core-auto-use` | 0.11 | 0.14 | 0.19 | 0.03 | 900 | 28% |
| `process/nvm-core-version` | 0.07 | 0.13 | 0.17 | 0.05 | 900 | 99% |
| `process/fnm-env` | 0.31 | 0.35 | 0.49 | 0.16 | 900 | 178% |
| `process/fnm-ls` | 0.33 | 0.39 | 0.50 | 0.07 | 900 | 29% |
| `process/nvm-core-ls` | 0.21 | 0.28 | 0.49 | 0.15 | 900 | 30% |
| `process/true` | 0.11 | 0.15 | 0.23 | 0.05 | 900 | 55% |

