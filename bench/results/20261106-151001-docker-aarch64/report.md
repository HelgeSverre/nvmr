# Benchmark report (docker cpuset=2,3 mem=2g, 2026-11-06T14:10:06+0000)

- git `v0.1.0-1-g107b1be-dirty`, seed 20261022, 3 rounds
- aarch64, 12 CPUs, Linux-7.2.12-orbstack-00394-gbecdd6130068-aarch64-with-glibc2.36
- cgroup: {'/sys/fs/cgroup/cpu.max': 'max 100000', '/sys/fs/cgroup/cpuset.cpus.effective': '2-3', '/sys/fs/cgroup/memory.max': '2147483648'}
- hyperfine 1.20.0; bash `GNU bash, version 5.2.15(1)-release (aarch64-unknown-linux-gnu)`; `zsh 5.9 (aarch64-unknown-linux-gnu)`; fnm `fnm 1.39.0`; nvm v0.40.8; nvmr `0.40.8`
- load average before/after: [2.19, 1.95, 1.41] / [1.42, 1.74, 1.43]

Per-operation numbers are loop totals minus the same shell's startup, divided by the operation count; `±` is the summed MAD of both, per operation: differences below it are not resolved. `min` is the min over every run of every round; `median` is the median of the per-round medians. Times in ms.

## Shell startup overhead (config minus bare shell)

`nvm-cd`/`nvmr-cd` add upstream's README auto-switch recipe; `nvmr-autocd` uses nvmr's one-process hook; `fnm` is `fnm env --use-on-cd`.

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `startup/bash/nvmr-autocd` | – | – | – | 1.08 | 4.33 | 900 | 35% |
| `startup/bash/fnm` | – | – | – | 0.85 | 2.70 | 900 | 3% |

## `ls` with 102 versions installed

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `ls-many/bash/nvmr` | 1.12 | 3.14 | 0.63 | 12.47 | 36.58 | 900 | 17% |
| `ls-many/bash/fnm` | 0.48 | 1.55 | 0.35 | 5.69 | 17.92 | 900 | 9% |

## `ls --no-alias` with 102 versions installed

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `ls-many-noalias/bash/nvmr` | 0.26 | 0.81 | 0.39 | 3.86 | 13.24 | 900 | 22% |

## `cd` between directories without a version file

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `cd-nochange/bash/nvmr-native` | 0.21 | 0.79 | 0.13 | 22.57 | 83.24 | 900 | 13% |
| `cd-nochange/bash/fnm` | 0.01 | 0.02 | 0.02 | 1.71 | 5.02 | 900 | 10% |
