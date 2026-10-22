# Benchmark report (host Darwin arm64, 2026-10-22T16:23:23+0200)

- git `c42f9a4-dirty`, seed 20261022, 3 rounds
- Apple M2 Max, 12 CPUs, macOS-15.6-arm64-arm-64bit-Mach-O
- cgroup: none
- hyperfine 1.20.0; bash `GNU bash, version 3.2.57(1)-release (arm64-apple-darwin24)`; `zsh 5.9 (arm64-apple-darwin24.0)`; fnm `fnm 1.39.0`; nvm v0.40.8; nvm-fast `0.40.8`
- load average before/after: [37.25, 33.89, 29.35] / [18.18, 19.16, 22.18]  **NOISY: load exceeded ncpu/4; trust min and p10, not medians**

Per-operation numbers are loop totals minus the same shell's startup, divided by the operation count; `±` is the summed MAD of both, per operation: differences below it are not resolved. `min` is the min over every run of every round; `median` is the median of the per-round medians. Times in ms.

## Shell startup overhead (config minus bare shell)

`nvm-cd`/`fast-cd` add upstream's README auto-switch recipe; `fast-autocd` uses nvm-fast's one-process hook; `fnm` is `fnm env --use-on-cd`.

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `startup/bash/bare` | – | – | – | 1.89 | 2.73 | 900 | 128% |
| `startup/bash/nvm` | 143 | 238 | 43.16 | 145 | 240 | 150 | 42% |
| `startup/bash/nvm-nouse` | 6.21 | 7.25 | 1.50 | 8.10 | 9.99 | 839 | 7% |
| `startup/bash/nvm-cd` | 244 | 361 | 56.56 | 246 | 364 | 31 | 46% |
| `startup/bash/fast` | 3.08 | 6.92 | 3.09 | 4.97 | 9.65 | 754 | 61% |
| `startup/bash/fast-nouse` | 0.28 | 0.24 | 1.05 | 2.18 | 2.98 | 900 | 20% |
| `startup/bash/fast-cd` | 12.61 | 16.53 | 3.59 | 14.50 | 19.26 | 430 | 52% |
| `startup/bash/fast-autocd` | 3.84 | 6.13 | 1.74 | 5.74 | 8.86 | 900 | 17% |
| `startup/bash/fnm` | 3.83 | 5.17 | 1.91 | 5.72 | 7.90 | 754 | 45% |
| `startup/zsh/bare` | – | – | – | 3.54 | 4.85 | 900 | 23% |
| `startup/zsh/nvm` | 478 | 654 | 111 | 482 | 659 | 15 | 48% |
| `startup/zsh/nvm-nouse` | 56.01 | 58.35 | 2.77 | 59.55 | 63.20 | 150 | 5% |
| `startup/zsh/nvm-cd` | 466 | 584 | 112 | 470 | 589 | 16 | 36% |
| `startup/zsh/fast` | 3.68 | 4.30 | 1.91 | 7.21 | 9.16 | 634 | 17% |
| `startup/zsh/fast-nouse` | 0.45 | 0.62 | 1.27 | 3.99 | 5.48 | 900 | 6% |
| `startup/zsh/fast-cd` | 12.37 | 16.31 | 2.56 | 15.91 | 21.16 | 431 | 13% |
| `startup/zsh/fast-autocd` | 3.43 | 5.90 | 1.87 | 6.96 | 10.76 | 728 | 24% |
| `startup/zsh/fnm` | 4.15 | 4.52 | 1.36 | 7.68 | 9.37 | 844 | 11% |

## `use` (switch between two installed versions)

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `use/bash/nvm` | 201 | 248 | 25.25 | 3357 | 4213 | 15 | 35% |
| `use/bash/fast` | 3.32 | 4.27 | 0.50 | 337 | 436 | 18 | 33% |
| `use/bash/fnm` | 3.41 | 4.09 | 0.35 | 347 | 417 | 18 | 45% |
| `use/zsh/nvm` | 346 | 355 | 24.00 | 6022 | 6344 | 15 | 15% |
| `use/zsh/fast` | 4.41 | 5.31 | 0.75 | 449 | 541 | 16 | 52% |
| `use/zsh/fnm` | 3.49 | 4.53 | 0.68 | 357 | 462 | 18 | 51% |

## `current`

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `current/bash/nvm` | 17.77 | 21.04 | 3.82 | 500 | 661 | 15 | 13% |
| `current/bash/fast` | 2.74 | 3.30 | 0.43 | 279 | 339 | 25 | 11% |
| `current/bash/fnm` | 3.68 | 4.28 | 0.50 | 374 | 436 | 20 | 110% |
| `current/zsh/nvm` | 24.23 | 22.76 | 11.24 | 966 | 1115 | 15 | 20% |
| `current/zsh/fast` | 3.01 | 3.55 | 0.41 | 308 | 364 | 18 | 23% |
| `current/zsh/fnm` | 3.57 | 3.99 | 0.28 | 365 | 409 | 21 | 9% |

## `version <major>` (resolution)

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `version/bash/nvm` | 66.13 | 77.70 | 19.69 | 806 | 1017 | 15 | 84% |
| `version/bash/fast` | 2.79 | 2.98 | 0.28 | 284 | 308 | 25 | 53% |
| `version/zsh/nvm` | 116 | 173 | 41.56 | 1638 | 2393 | 15 | 28% |
| `version/zsh/fast` | 2.89 | 3.62 | 0.49 | 297 | 371 | 22 | 13% |

## `ls` with 102 versions installed

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `ls-many/bash/nvm` | 1361 | 1537 | 150 | 1499 | 1713 | 15 | 8% |
| `ls-many/bash/fast` | 4.86 | 6.34 | 0.91 | 54.40 | 72.49 | 150 | 18% |
| `ls-many/bash/fnm` | 3.95 | 4.95 | 0.98 | 45.28 | 56.84 | 155 | 46% |
| `ls-many/zsh/nvm` | 1789 | 2328 | 343 | 2260 | 2885 | 15 | 24% |
| `ls-many/zsh/fast` | 5.36 | 5.90 | 0.78 | 60.50 | 71.62 | 150 | 23% |
| `ls-many/zsh/fnm` | 3.85 | 4.79 | 0.61 | 46.09 | 56.76 | 159 | 20% |

## `cd` into/out of a project with `.nvmrc` (switches each time)

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `cd-switch/bash/nvm` | 436 | 501 | 59.31 | 3731 | 4370 | 15 | 25% |
| `cd-switch/bash/fast` | 13.31 | 17.31 | 3.32 | 547 | 712 | 15 | 53% |
| `cd-switch/bash/fast-native` | 4.45 | 7.74 | 1.85 | 451 | 783 | 16 | 59% |
| `cd-switch/bash/fnm` | 3.59 | 4.13 | 0.54 | 364 | 421 | 19 | 30% |
| `cd-switch/zsh/nvm` | 502 | 668 | 84.08 | 4482 | 5932 | 15 | 19% |
| `cd-switch/zsh/fast` | 23.86 | 26.56 | 3.33 | 970 | 1083 | 15 | 18% |
| `cd-switch/zsh/fast-native` | 4.47 | 7.03 | 2.27 | 454 | 714 | 16 | 75% |
| `cd-switch/zsh/fnm` | 3.63 | 4.16 | 0.25 | 371 | 425 | 19 | 11% |

## `cd` between directories without a version file

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `cd-nochange/bash/nvm` | 107 | 117 | 8.89 | 4526 | 5034 | 15 | 6% |
| `cd-nochange/bash/fast` | 9.80 | 11.88 | 2.62 | 799 | 970 | 15 | 49% |
| `cd-nochange/bash/fast-native` | 2.74 | 2.97 | 0.25 | 279 | 305 | 24 | 57% |
| `cd-nochange/bash/fnm` | 0.02 | 0.01 | 0.02 | 7.45 | 8.81 | 847 | 7% |
| `cd-nochange/zsh/nvm` | 6.24 | 13.73 | 7.06 | 719 | 1138 | 15 | 43% |
| `cd-nochange/zsh/fast` | 7.61 | 9.01 | 1.32 | 625 | 742 | 15 | 32% |
| `cd-nochange/zsh/fast-native` | 2.86 | 3.23 | 0.37 | 293 | 334 | 23 | 30% |
| `cd-nochange/zsh/fnm` | 0.02 | 0.02 | 0.02 | 9.18 | 11.74 | 652 | 12% |

## install from a loopback mirror (download + verify + extract)

| benchmark | min | p10 | median | MAD | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|
| `install/bash/nvm` | 3961 | 3997 | 5496 | 815 | 15 | 28% |
| `install/bash/fast` | 1873 | 2020 | 2404 | 317 | 15 | 30% |
| `install/bash/fnm` | 2286 | 2410 | 3017 | 484 | 15 | 32% |

## Bare process latency

| benchmark | min | p10 | median | MAD | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|
| `process/nvm-core-current` | 1.59 | 1.72 | 2.22 | 0.79 | 900 | 129% |
| `process/nvm-core-auto-use` | 1.70 | 1.79 | 1.97 | 0.18 | 900 | 15% |
| `process/nvm-core-version` | 1.58 | 1.80 | 2.06 | 0.22 | 900 | 8% |
| `process/fnm-env` | 2.88 | 3.23 | 4.11 | 0.50 | 900 | 20% |
| `process/fnm-ls` | 2.87 | 3.12 | 3.44 | 0.38 | 900 | 21% |
| `process/nvm-core-ls` | 1.90 | 2.04 | 2.23 | 0.14 | 900 | 4% |
| `process/true` | 0.98 | 1.04 | 1.14 | 0.16 | 900 | 92% |

