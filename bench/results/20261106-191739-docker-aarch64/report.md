# Benchmark report (docker cpuset=2,3 mem=2g, 2026-11-06T18:17:44+0000)

- git `v0.1.0-7-g6a225a1-dirty`, seed 20261022, 3 rounds
- aarch64, 12 CPUs, Linux-7.2.12-orbstack-00394-gbecdd6130068-aarch64-with-glibc2.36
- cgroup: {'/sys/fs/cgroup/cpu.max': 'max 100000', '/sys/fs/cgroup/cpuset.cpus.effective': '2-3', '/sys/fs/cgroup/memory.max': '2147483648'}
- hyperfine 1.20.0; bash `GNU bash, version 5.2.15(1)-release (aarch64-unknown-linux-gnu)`; `zsh 5.9 (aarch64-unknown-linux-gnu)`; fnm `fnm 1.39.0`; nvm v0.40.8; nvmr `0.40.8`; mise `2026.11.3 linux-arm64 (2026-11-06)`; proto `proto 0.63.2`
- load average before/after: [1.04, 0.46, 0.53] / [1.04, 0.47, 0.53]

Per-operation numbers are loop totals minus the same shell's startup, divided by the operation count; `±` is the summed MAD of both, per operation: differences below it are not resolved. `min` is the min over every run of every round; `median` is the median of the per-round medians. Times in ms.

## Shell startup overhead (config minus bare shell)

`nvm-cd`/`nvmr-cd` add upstream's README auto-switch recipe; `nvmr-autocd` uses nvmr's one-process hook; `fnm` uses the default local `fnm env --use-on-cd`; `fnm-recursive` enables parent lookup; `mise` uses `mise activate` with documented `.nvmrc` support; `proto-activate` uses `proto activate --no-shim --config-mode all`; `proto-shim` adds its shim directory.

| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |
|---|---:|---:|---:|---:|---:|---:|---:|
| `startup/bash/bare` | – | – | – | 0.32 | 0.37 | 900 | 10% |
| `startup/bash/nvmr-lazypath` | 0.32 | 0.34 | 0.04 | 0.64 | 0.71 | 900 | 6% |
| `startup/bash/nvmr-lazypath-autocd` | 0.35 | 0.37 | 0.03 | 0.67 | 0.74 | 900 | 1% |
| `startup/bash/fnm` | 0.47 | 0.57 | 0.09 | 0.79 | 0.94 | 900 | 15% |
| `startup/bash/fnm-recursive` | 0.46 | 0.51 | 0.06 | 0.78 | 0.88 | 900 | 13% |
