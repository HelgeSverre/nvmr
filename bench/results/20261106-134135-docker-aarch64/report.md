# Benchmark report (docker cpuset=2,3 mem=2g, 2026-11-06T12:41:39+0000)

- git `v0.1.0-dirty`, seed 20261022, 3 rounds
- aarch64, 12 CPUs, Linux-7.2.12-orbstack-00394-gbecdd6130068-aarch64-with-glibc2.36
- cgroup: {'/sys/fs/cgroup/cpu.max': 'max 100000', '/sys/fs/cgroup/cpuset.cpus.effective': '2-3', '/sys/fs/cgroup/memory.max': '2147483648'}
- hyperfine 1.20.0; bash `GNU bash, version 5.2.15(1)-release (aarch64-unknown-linux-gnu)`; `zsh 5.9 (aarch64-unknown-linux-gnu)`; fnm `fnm 1.39.0`; nvm v0.40.8; nvmr `0.40.8`
- load average before/after: [1.32, 2.12, 2.58] / [1.04, 1.73, 2.36]

Per-operation numbers are loop totals minus the same shell's startup, divided by the operation count; `±` is the summed MAD of both, per operation: differences below it are not resolved. `min` is the min over every run of every round; `median` is the median of the per-round medians. Times in ms.

## Shell startup overhead (config minus bare shell)

`nvm-cd`/`nvmr-cd` add upstream's README auto-switch recipe; `nvmr-autocd` uses nvmr's one-process hook; `fnm` is `fnm env --use-on-cd`.

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `startup/bash/bare` | – | – | – | 0.32 | 0.37 | 900 | 8% |
| `startup/bash/nvmr` | 0.84 | 1.10 | 0.16 | 1.17 | 1.46 | 900 | 34% |
| `startup/bash/nvmr-nouse` | 0.22 | 0.25 | 0.04 | 0.54 | 0.62 | 900 | 4% |

## `use` (switch between two installed versions)

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `use/bash/nvmr` | 0.36 | 1.11 | 0.44 | 36.77 | 113 | 900 | 99% |

## `current`

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `current/bash/nvmr` | 0.28 | 0.33 | 0.02 | 29.55 | 34.86 | 900 | 4% |

## `ls` with 102 versions installed

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `ls-many/bash/nvmr` | 1.16 | 1.20 | 0.07 | 12.67 | 13.47 | 900 | 5% |

## Bare process latency

| benchmark | min | p10 | median | MAD | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|
| `process/nvmr-current` | 0.07 | 0.09 | 0.10 | 0.01 | 900 | 11% |
