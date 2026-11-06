# Benchmark report (docker cpuset=2,3 mem=2g, 2026-11-06T18:02:01+0000)

- git `v0.1.0-7-g6a225a1-dirty`, seed 20261022, 3 rounds
- aarch64, 12 CPUs, Linux-7.2.12-orbstack-00394-gbecdd6130068-aarch64-with-glibc2.36
- cgroup: {'/sys/fs/cgroup/cpu.max': 'max 100000', '/sys/fs/cgroup/cpuset.cpus.effective': '2-3', '/sys/fs/cgroup/memory.max': '2147483648'}
- hyperfine 1.20.0; bash `GNU bash, version 5.2.15(1)-release (aarch64-unknown-linux-gnu)`; `zsh 5.9 (aarch64-unknown-linux-gnu)`; fnm `fnm 1.39.0`; nvm v0.40.8; nvmr `0.40.8`; mise `2026.11.3 linux-arm64 (2026-11-06)`; proto `proto 0.63.2`
- load average before/after: [1.28, 0.81, 0.56] / [1.32, 1.25, 0.84]

Per-operation numbers are loop totals minus the same shell's startup, divided by the operation count; `±` is the summed MAD of both, per operation: differences below it are not resolved. `min` is the min over every run of every round; `median` is the median of the per-round medians. Times in ms.

## Shell startup overhead (config minus bare shell)

`nvm-cd`/`nvmr-cd` add upstream's README auto-switch recipe; `nvmr-autocd` uses nvmr's one-process hook; `fnm` uses the default local `fnm env --use-on-cd`; `fnm-recursive` enables parent lookup; `mise` uses `mise activate` with documented `.nvmrc` support; `proto-activate` uses `proto activate --no-shim --config-mode all`; `proto-shim` adds its shim directory.

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `startup/bash/bare` | – | – | – | 0.33 | 0.40 | 900 | 22% |
| `startup/bash/bare-node` | 0.01 | -0.01 | 0.04 | 0.34 | 0.39 | 900 | 2% |
| `startup/bash/nvmr-lazypath` | 0.32 | 0.38 | 0.09 | 0.65 | 0.78 | 900 | 17% |
| `startup/bash/nvmr-lazypath-autocd` | 0.40 | 0.54 | 0.12 | 0.73 | 0.93 | 900 | 16% |
| `startup/bash/fnm` | 0.55 | 0.81 | 0.16 | 0.88 | 1.21 | 900 | 21% |
| `startup/bash/fnm-recursive` | 0.51 | 0.78 | 0.20 | 0.84 | 1.18 | 900 | 28% |

## direct `node -e 0` execution (shim cost included)

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `node-exec/bash/bare-node` | 10.03 | 11.80 | 1.23 | 20.40 | 23.99 | 900 | 20% |
| `node-exec/bash/nvmr-lazypath-autocd` | 9.68 | 11.36 | 0.79 | 20.09 | 23.65 | 900 | 11% |
| `node-exec/bash/fnm` | 9.51 | 11.12 | 0.81 | 19.91 | 23.46 | 900 | 11% |

## `cd` into/out of a project with `.nvmrc` (switches each time)

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `cd-switch/bash/nvmr-lazypath` | 0.29 | 0.51 | 0.08 | 30.02 | 51.93 | 900 | 34% |
| `cd-switch/bash/fnm` | 0.37 | 0.40 | 0.02 | 37.47 | 41.10 | 900 | 53% |
| `cd-switch/bash/fnm-recursive` | 0.36 | 0.40 | 0.03 | 36.58 | 41.27 | 900 | 31% |

## `cd` between directories without a version file

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `cd-nochange/bash/nvmr-lazypath` | 0.06 | 0.06 | 0.00 | 6.70 | 7.23 | 900 | 9% |
| `cd-nochange/bash/fnm` | 0.01 | 0.01 | 0.00 | 1.56 | 1.77 | 900 | 19% |
| `cd-nochange/bash/fnm-recursive` | 0.38 | 0.44 | 0.03 | 38.87 | 44.78 | 900 | 22% |
