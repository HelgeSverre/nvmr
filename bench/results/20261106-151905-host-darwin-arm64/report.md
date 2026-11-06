# Benchmark report (host Darwin arm64, 2026-11-06T15:19:36+0100)

- git `v0.1.0-1-g107b1be-dirty`, seed 20261022, 3 rounds
- Apple M2 Max, 12 CPUs, macOS-15.6-arm64-arm-64bit-Mach-O
- cgroup: none
- hyperfine 1.20.0; bash `GNU bash, version 3.2.57(1)-release (arm64-apple-darwin24)`; `zsh 5.9 (arm64-apple-darwin24.0)`; fnm `fnm 1.39.0`; nvm v0.40.8; nvmr `0.40.8`
- load average before/after: [48.2, 37.54, 35.82] / [14.41, 25.67, 29.8]  **NOISY: load exceeded ncpu/4; trust min and p10, not medians**

Per-operation numbers are loop totals minus the same shell's startup, divided by the operation count; `±` is the summed MAD of both, per operation: differences below it are not resolved. `min` is the min over every run of every round; `median` is the median of the per-round medians. Times in ms.

## Shell startup overhead (config minus bare shell)

`nvm-cd`/`nvmr-cd` add upstream's README auto-switch recipe; `nvmr-autocd` uses nvmr's one-process hook; `fnm` is `fnm env --use-on-cd`.

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `startup/bash/nvmr-autocd` | – | – | – | 5.24 | 9.07 | 900 | 68% |
| `startup/bash/fnm` | – | – | – | 5.67 | 7.62 | 900 | 60% |

## `ls` with 102 versions installed

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `ls-many/bash/nvmr` | 4.42 | 5.18 | 1.13 | 49.56 | 61.19 | 900 | 71% |
| `ls-many/bash/fnm` | 3.21 | 3.64 | 0.75 | 37.97 | 47.53 | 900 | 55% |

## `ls --no-alias` with 102 versions installed

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `ls-many-noalias/bash/nvmr` | 2.11 | 2.65 | 0.71 | 26.39 | 35.85 | 900 | 68% |

## `cd` between directories without a version file

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `cd-nochange/bash/nvmr-native` | 2.09 | 2.88 | 0.82 | 214 | 297 | 900 | 75% |
| `cd-nochange/bash/fnm` | 0.02 | 0.02 | 0.03 | 7.76 | 9.98 | 900 | 78% |
