# Benchmark report (docker cpuset=2,3 mem=2g, 2026-11-06T14:13:34+0000)

- git `v0.1.0-1-g107b1be-dirty`, seed 20261022, 3 rounds
- aarch64, 12 CPUs, Linux-7.2.12-orbstack-00394-gbecdd6130068-aarch64-with-glibc2.36
- cgroup: {'/sys/fs/cgroup/cpu.max': 'max 100000', '/sys/fs/cgroup/cpuset.cpus.effective': '2-3', '/sys/fs/cgroup/memory.max': '2147483648'}
- hyperfine 1.20.0; bash `GNU bash, version 5.2.15(1)-release (aarch64-unknown-linux-gnu)`; `zsh 5.9 (aarch64-unknown-linux-gnu)`; fnm `fnm 1.39.0`; nvm v0.40.8; nvmr `0.40.8`
- load average before/after: [1.91, 1.82, 1.47] / [1.83, 1.81, 1.48]

Per-operation numbers are loop totals minus the same shell's startup, divided by the operation count; `±` is the summed MAD of both, per operation: differences below it are not resolved. `min` is the min over every run of every round; `median` is the median of the per-round medians. Times in ms.

## Shell startup overhead (config minus bare shell)

`nvm-cd`/`nvmr-cd` add upstream's README auto-switch recipe; `nvmr-autocd` uses nvmr's one-process hook; `fnm` is `fnm env --use-on-cd`.

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `startup/bash/nvmr-autocd` | – | – | – | 1.18 | 1.58 | 900 | 19% |
| `startup/bash/fnm` | – | – | – | 0.81 | 0.99 | 900 | 22% |

## `ls` with 102 versions installed

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `ls-many/bash/nvmr` | 0.44 | 0.53 | 0.07 | 5.73 | 7.13 | 900 | 11% |
| `ls-many/bash/fnm` | 0.42 | 0.55 | 0.08 | 5.02 | 6.57 | 900 | 18% |

## `ls --no-alias` with 102 versions installed

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `ls-many-noalias/bash/nvmr` | 0.22 | 0.27 | 0.08 | 3.46 | 4.50 | 900 | 17% |

## `cd` between directories without a version file

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `cd-nochange/bash/nvmr-native` | 0.07 | 0.08 | 0.01 | 8.23 | 9.35 | 900 | 5% |
| `cd-nochange/bash/fnm` | 0.01 | 0.01 | 0.00 | 1.56 | 1.83 | 900 | 8% |
