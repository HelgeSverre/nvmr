# Benchmark report (host Darwin arm64, 2026-11-06T19:07:37+0100)

- git `v0.1.0-7-g6a225a1-dirty`, seed 20261022, 3 rounds
- Apple M2 Max, 12 CPUs, macOS-15.6-arm64-arm-64bit-Mach-O
- cgroup: none
- hyperfine 1.20.0; bash `GNU bash, version 3.2.57(1)-release (arm64-apple-darwin24)`; `zsh 5.9 (arm64-apple-darwin24.0)`; fnm `fnm 1.39.0`; nvm v0.40.8; nvmr `0.40.8`
- load average before/after: [12.81, 12.64, 12.02] / [6.23, 7.55, 9.56]  **NOISY: load exceeded ncpu/4; trust min and p10, not medians**

Per-operation numbers are loop totals minus the same shell's startup, divided by the operation count; `±` is the summed MAD of both, per operation: differences below it are not resolved. `min` is the min over every run of every round; `median` is the median of the per-round medians. Times in ms.

## Shell startup overhead (config minus bare shell)

`nvm-cd`/`nvmr-cd` add upstream's README auto-switch recipe; `nvmr-autocd` uses nvmr's one-process hook; `fnm` uses the default local `fnm env --use-on-cd`; `fnm-recursive` enables parent lookup; `mise` uses `mise activate` with documented `.nvmrc` support; `proto-activate` uses `proto activate --no-shim --config-mode all`; `proto-shim` adds its shim directory.

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `startup/bash/bare` | – | – | – | 1.80 | 2.20 | 900 | 13% |
| `startup/bash/bare-node` | -0.00 | -0.03 | 0.27 | 1.80 | 2.17 | 900 | 10% |
| `startup/bash/nvmr-lazypath` | 0.55 | 0.41 | 0.24 | 2.35 | 2.61 | 900 | 5% |
| `startup/bash/nvmr-lazypath-autocd` | 0.61 | 0.51 | 0.27 | 2.41 | 2.72 | 900 | 6% |
| `startup/bash/fnm` | 3.69 | 4.32 | 0.56 | 5.49 | 6.52 | 900 | 11% |
| `startup/bash/fnm-recursive` | 4.02 | 4.32 | 0.44 | 5.82 | 6.52 | 900 | 4% |

## direct `node -e 0` execution (shim cost included)

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `node-exec/bash/bare-node` | 21.28 | 23.94 | 1.16 | 44.36 | 50.04 | 900 | 9% |
| `node-exec/bash/nvmr-lazypath-autocd` | 20.86 | 23.85 | 1.45 | 44.12 | 50.41 | 900 | 13% |
| `node-exec/bash/fnm` | 21.53 | 24.04 | 1.14 | 48.56 | 54.60 | 900 | 4% |

## `cd` into/out of a project with `.nvmrc` (switches each time)

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `cd-switch/bash/nvmr-lazypath` | 2.47 | 3.42 | 0.47 | 249 | 344 | 900 | 27% |
| `cd-switch/bash/fnm` | 3.42 | 3.70 | 0.20 | 348 | 376 | 22 | 19% |
| `cd-switch/bash/fnm-recursive` | 3.57 | 3.70 | 0.21 | 362 | 377 | 21 | 13% |

## `cd` between directories without a version file

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `cd-nochange/bash/nvmr-lazypath` | 0.16 | 0.17 | 0.01 | 18.77 | 19.86 | 900 | 1% |
| `cd-nochange/bash/fnm` | 0.02 | 0.02 | 0.01 | 7.42 | 8.28 | 900 | 18% |
| `cd-nochange/bash/fnm-recursive` | 3.41 | 3.61 | 0.17 | 346 | 368 | 23 | 16% |
