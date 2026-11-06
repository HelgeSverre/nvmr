# Benchmark report (host Darwin arm64, 2026-11-06T15:29:42+0100)

- git `v0.1.0-1-g107b1be-dirty`, seed 20261022, 3 rounds
- Apple M2 Max, 12 CPUs, macOS-15.6-arm64-arm-64bit-Mach-O
- cgroup: none
- hyperfine 1.20.0; bash `GNU bash, version 3.2.57(1)-release (arm64-apple-darwin24)`; `zsh 5.9 (arm64-apple-darwin24.0)`; fnm `fnm 1.39.0`; nvm v0.40.8; nvmr `0.40.8`
- load average before/after: [28.71, 27.82, 30.36] / [11.84, 19.87, 26.34]  **NOISY: load exceeded ncpu/4; trust min and p10, not medians**

Per-operation numbers are loop totals minus the same shell's startup, divided by the operation count; `±` is the summed MAD of both, per operation: differences below it are not resolved. `min` is the min over every run of every round; `median` is the median of the per-round medians. Times in ms.

## Shell startup overhead (config minus bare shell)

`nvm-cd`/`nvmr-cd` add upstream's README auto-switch recipe; `nvmr-autocd` uses nvmr's one-process hook; `fnm` is `fnm env --use-on-cd`.

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `startup/bash/nvmr-autocd` | – | – | – | 5.17 | 10.90 | 900 | 51% |
| `startup/bash/fnm` | – | – | – | 6.01 | 8.22 | 900 | 29% |

## `ls` with 102 versions installed

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `ls-many/bash/nvmr` | 2.72 | 3.13 | 0.80 | 32.94 | 42.20 | 900 | 47% |
| `ls-many/bash/fnm` | 3.38 | 4.02 | 0.29 | 39.59 | 47.14 | 900 | 6% |

## `ls --no-alias` with 102 versions installed

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `ls-many-noalias/bash/nvmr` | 2.26 | 2.53 | 0.56 | 28.37 | 36.24 | 900 | 13% |

## `cd` between directories without a version file

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `cd-nochange/bash/nvmr-native` | 0.19 | 0.21 | 0.08 | 24.53 | 32.00 | 900 | 68% |
| `cd-nochange/bash/fnm` | 0.02 | 0.03 | 0.02 | 7.75 | 10.92 | 900 | 60% |
