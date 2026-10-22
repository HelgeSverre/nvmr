# Benchmark report (host Darwin arm64, 2026-10-22T15:44:58+0200)

- git `c871fb2-dirty`, seed 20261022, 3 rounds
- Apple M2 Max, 12 CPUs, macOS-15.6-arm64-arm-64bit-Mach-O
- cgroup: none
- hyperfine 1.20.0; bash `GNU bash, version 3.2.57(1)-release (arm64-apple-darwin24)`; `zsh 5.9 (arm64-apple-darwin24.0)`; fnm `fnm 1.39.0`; nvm v0.40.8; nvm-fast `0.40.8`
- load average before/after: [20.76, 18.26, 21.63] / [28.1, 24.62, 25.29]  **NOISY: load exceeded ncpu/4; trust min and p10, not medians**

Per-operation numbers are loop totals minus the same shell's startup, divided by the operation count; `±` is the summed MAD of both, per operation: differences below it are not resolved. `min` is the min over every run of every round; `median` is the median of the per-round medians. Times in ms.

## Shell startup overhead (config minus bare shell)

`nvm-cd`/`fast-cd` add upstream's README auto-switch recipe; `fast-autocd` uses nvm-fast's one-process hook; `fnm` is `fnm env --use-on-cd`.

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `startup/bash/bare` | – | – | – | 2.02 | 3.12 | 900 | 32% |
| `startup/bash/nvm` | 151 | 227 | 36.56 | 153 | 230 | 44 | 22% |
| `startup/bash/nvm-nouse` | 5.85 | 7.54 | 1.88 | 7.87 | 10.66 | 838 | 33% |
| `startup/bash/nvm-cd` | 328 | 394 | 78.32 | 330 | 397 | 18 | 120% |
| `startup/bash/fast` | 4.10 | 10.87 | 5.22 | 6.12 | 13.99 | 685 | 121% |
| `startup/bash/fast-nouse` | 0.20 | 0.17 | 0.95 | 2.22 | 3.29 | 900 | 14% |
| `startup/bash/fast-cd` | 15.45 | 23.07 | 4.30 | 17.47 | 26.20 | 392 | 20% |
| `startup/bash/fast-autocd` | 4.10 | 7.24 | 3.48 | 6.12 | 10.37 | 794 | 134% |
| `startup/bash/fnm` | 4.09 | 5.33 | 1.98 | 6.11 | 8.45 | 625 | 37% |
| `startup/zsh/bare` | – | – | – | 3.59 | 5.12 | 900 | 11% |
| `startup/zsh/nvm` | 507 | 782 | 110 | 510 | 787 | 15 | 42% |
| `startup/zsh/nvm-nouse` | 56.53 | 60.33 | 4.01 | 60.12 | 65.45 | 150 | 33% |
| `startup/zsh/nvm-cd` | 545 | 762 | 184 | 549 | 767 | 15 | 44% |
| `startup/zsh/fast` | 4.23 | 5.25 | 2.45 | 7.82 | 10.37 | 644 | 53% |
| `startup/zsh/fast-nouse` | 0.33 | 0.66 | 2.48 | 3.92 | 5.78 | 897 | 123% |
| `startup/zsh/fast-cd` | 13.73 | 18.33 | 3.31 | 17.32 | 23.45 | 310 | 69% |
| `startup/zsh/fast-autocd` | 5.62 | 9.50 | 3.38 | 9.21 | 14.61 | 707 | 38% |
| `startup/zsh/fnm` | 4.52 | 5.60 | 1.71 | 8.11 | 10.72 | 670 | 8% |

## `use` (switch between two installed versions)

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `use/bash/nvm` | 240 | 296 | 34.53 | 4000 | 4971 | 15 | 27% |
| `use/bash/fast` | 4.43 | 7.12 | 1.52 | 449 | 726 | 15 | 37% |
| `use/bash/fnm` | 3.99 | 5.43 | 0.71 | 405 | 551 | 18 | 27% |
| `use/zsh/nvm` | 359 | 528 | 88.66 | 6250 | 9241 | 15 | 8% |
| `use/zsh/fast` | 5.30 | 9.19 | 2.49 | 538 | 929 | 15 | 58% |
| `use/zsh/fnm` | 3.71 | 6.14 | 1.03 | 379 | 624 | 16 | 41% |

## `current`

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `current/bash/nvm` | 24.31 | 27.88 | 3.95 | 639 | 788 | 15 | 4% |
| `current/bash/fast` | 3.62 | 5.95 | 2.02 | 368 | 609 | 18 | 194% |
| `current/bash/fnm` | 3.77 | 5.68 | 1.03 | 383 | 576 | 18 | 62% |
| `current/zsh/nvm` | 25.46 | 16.69 | 15.60 | 1020 | 1121 | 15 | 30% |
| `current/zsh/fast` | 5.17 | 5.54 | 0.48 | 525 | 565 | 15 | 18% |
| `current/zsh/fnm` | 3.53 | 3.94 | 0.42 | 361 | 405 | 20 | 58% |

## `version <major>` (resolution)

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `version/bash/nvm` | 52.72 | 55.29 | 15.98 | 680 | 783 | 15 | 202% |
| `version/bash/fast` | 4.01 | 4.85 | 0.37 | 407 | 499 | 17 | 16% |
| `version/zsh/nvm` | 113 | 210 | 48.41 | 1639 | 2890 | 15 | 28% |
| `version/zsh/fast` | 4.29 | 7.29 | 1.16 | 437 | 739 | 16 | 41% |

## `ls` with 102 versions installed

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `ls-many/bash/nvm` | 1215 | 1501 | 490 | 1358 | 1736 | 15 | 131% |
| `ls-many/bash/fast` | 6.16 | 8.18 | 1.34 | 68.39 | 92.74 | 150 | 28% |
| `ls-many/bash/fnm` | 4.26 | 6.34 | 1.56 | 48.67 | 71.84 | 153 | 26% |
| `ls-many/zsh/nvm` | 2250 | 4028 | 1260 | 2694 | 4598 | 15 | 45% |
| `ls-many/zsh/fast` | 5.80 | 7.35 | 1.84 | 66.90 | 87.61 | 150 | 110% |
| `ls-many/zsh/fnm` | 4.27 | 5.97 | 1.41 | 50.98 | 73.32 | 150 | 48% |

## `cd` into/out of a project with `.nvmrc` (switches each time)

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `cd-switch/bash/nvm` | 455 | 587 | 116 | 3973 | 5095 | 15 | 20% |
| `cd-switch/bash/fast` | 22.71 | 58.39 | 17.63 | 926 | 2362 | 15 | 85% |
| `cd-switch/bash/fast-native` | 6.28 | 7.95 | 1.78 | 634 | 806 | 15 | 36% |
| `cd-switch/bash/fnm` | 3.95 | 4.72 | 0.28 | 402 | 481 | 16 | 2% |
| `cd-switch/zsh/nvm` | 551 | 595 | 97.37 | 4954 | 5528 | 15 | 197% |
| `cd-switch/zsh/fast` | 27.43 | 38.75 | 4.10 | 1115 | 1574 | 15 | 30% |
| `cd-switch/zsh/fast-native` | 4.40 | 7.82 | 1.16 | 449 | 797 | 15 | 47% |
| `cd-switch/zsh/fnm` | 4.31 | 6.83 | 0.89 | 439 | 693 | 16 | 38% |

## `cd` between directories without a version file

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `cd-nochange/bash/nvm` | 119 | 204 | 31.82 | 5105 | 8575 | 15 | 32% |
| `cd-nochange/bash/fast` | 11.77 | 15.72 | 2.93 | 959 | 1284 | 15 | 28% |
| `cd-nochange/bash/fast-native` | 3.97 | 5.62 | 0.98 | 403 | 572 | 17 | 63% |
| `cd-nochange/bash/fnm` | 0.02 | 0.03 | 0.03 | 8.06 | 10.96 | 607 | 45% |
| `cd-nochange/zsh/nvm` | 5.57 | 6.93 | 10.90 | 771 | 1045 | 15 | 100% |
| `cd-nochange/zsh/fast` | 9.05 | 9.98 | 2.04 | 741 | 822 | 15 | 20% |
| `cd-nochange/zsh/fast-native` | 3.71 | 4.99 | 1.05 | 380 | 513 | 17 | 91% |
| `cd-nochange/zsh/fnm` | 0.01 | 0.03 | 0.03 | 9.40 | 13.99 | 628 | 42% |

## install from a loopback mirror (download + verify + extract)

| benchmark | min | p10 | median | MAD | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|
| `install/bash/nvm` | 4092 | 4161 | 4742 | 786 | 15 | 30% |
| `install/bash/fast` | 2083 | 2131 | 2797 | 420 | 15 | 36% |
| `install/bash/fnm` | 2113 | 2247 | 3235 | 729 | 15 | 73% |

## Bare process latency

| benchmark | min | p10 | median | MAD | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|
| `process/nvm-core-current` | 2.60 | 3.00 | 3.75 | 0.66 | 900 | 24% |
| `process/nvm-core-auto-use` | 2.31 | 2.75 | 3.26 | 0.50 | 900 | 18% |
| `process/nvm-core-version` | 2.32 | 2.58 | 2.95 | 0.44 | 900 | 20% |
| `process/fnm-env` | 2.93 | 3.40 | 4.16 | 0.57 | 900 | 6% |
| `process/fnm-ls` | 2.87 | 3.35 | 4.77 | 0.84 | 900 | 29% |
| `process/nvm-core-ls` | 2.53 | 2.82 | 3.19 | 0.28 | 900 | 10% |
| `process/true` | 1.01 | 1.07 | 1.27 | 0.13 | 900 | 15% |

