# Benchmark report (docker cpuset=2,3 mem=2g, 2026-11-06T13:20:36+0000)

- git `v0.1.0-dirty`, seed 20261022, 3 rounds
- aarch64, 12 CPUs, Linux-7.2.12-orbstack-00394-gbecdd6130068-aarch64-with-glibc2.36
- cgroup: {'/sys/fs/cgroup/cpu.max': 'max 100000', '/sys/fs/cgroup/cpuset.cpus.effective': '2-3', '/sys/fs/cgroup/memory.max': '2147483648'}
- hyperfine 1.20.0; bash `GNU bash, version 5.2.15(1)-release (aarch64-unknown-linux-gnu)`; `zsh 5.9 (aarch64-unknown-linux-gnu)`; fnm `fnm 1.39.0`; nvm v0.40.8; nvmr `0.40.8`
- load average before/after: [0.34, 0.72, 1.25] / [1.2, 0.89, 1.27]

Per-operation numbers are loop totals minus the same shell's startup, divided by the operation count; `±` is the summed MAD of both, per operation: differences below it are not resolved. `min` is the min over every run of every round; `median` is the median of the per-round medians. Times in ms.

## Shell startup overhead (config minus bare shell)

`nvm-cd`/`nvmr-cd` add upstream's README auto-switch recipe; `nvmr-autocd` uses nvmr's one-process hook; `fnm` is `fnm env --use-on-cd`.

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `startup/bash/fnm` | – | – | – | 0.78 | 0.89 | 900 | 7% |

## `current`

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `current/bash/fnm` | 0.31 | 0.35 | 0.01 | 32.14 | 35.54 | 900 | 7% |

## install from a loopback mirror (download + verify + extract)

| benchmark | min | p10 | median | MAD | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|
| `install/bash/nvm` | 1545 | 1552 | 1569 | 14.81 | 15 | 1% |
