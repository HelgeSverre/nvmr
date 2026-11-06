# Exact-default lazy startup

Nvmr offers an opt-in startup path that reads an exact installed version from `alias/default`
and puts its Node `bin` directory on `PATH` without launching a manager process. Set
`NVM_LAZY=1 NVM_LAZY_PATH=1` before sourcing `nvm.sh`:

```sh
export NVM_LAZY=1 NVM_LAZY_PATH=1
. "$NVM_DIR/nvm.sh"
```

The benchmark harness records this as `nvmr-lazypath`. It also records
`nvmr-lazypath-autocd`, which adds nvmr's native directory hook. Each startup and Node
execution case checks the pinned `node -v` before timing; the directory cases check the
selected version in the project and in a directory without a version file. The comparisons
use the same installed Node versions and on-disk state as fnm.

## Measured result

The [Docker comparison](../../bench/results/20261106-190156-docker-aarch64/report.md) and a
[focused startup rerun](../../bench/results/20261106-191739-docker-aarch64/report.md) measured
300 executions per round in three seeded, shuffled rounds. The table shows **minimum observed
milliseconds**, with each startup value subtracting the bare shell and each `cd` value dividing
by the number of operations. The startup row comes from the rerun; the other rows come from
the comparison. Round spread exceeded 10% for the fnm startup and most other cases, so these
minima are observations rather than dependable typical latencies.

| Bash operation | nvmr lazy path | nvmr lazy path + hook | fnm local hook | fnm recursive hook |
| --- | ---: | ---: | ---: | ---: |
| Shell startup overhead | 0.32 | 0.35 | 0.47 | 0.46 |
| Run `node -e 0` after startup | — | 9.68 | 9.51 | — |
| `cd` into/out of a project with `.nvmrc` | — | 0.29 | 0.37 | 0.36 |
| `cd` between directories without a version file | — | 0.06 | 0.01 | 0.38 |

The no-hook startup case is a useful baseline for fixed-version shells; the hook-enabled
cases are the more relevant comparison for automatic project switching. Node execution is
effectively tied. Fnm's local hook is faster for a `cd` that does not change versions; nvmr's
parent lookup and default restoration require more work there. The project-switch minima favor
nvmr, but their high round spread prevents a stable speed ranking.

The [isolated macOS run](../../bench/results/20261106-190708-host-darwin-arm64/report.md)
used the same configurations and 300 executions in each of three shuffled rounds for the
startup and Node execution cases. The host load exceeded the harness threshold, so these are
**noisy-run minima only**:

| Bash operation | nvmr lazy path | nvmr lazy path + hook | fnm local hook | fnm recursive hook |
| --- | ---: | ---: | ---: | ---: |
| Shell startup overhead | 0.55 | 0.61 | 3.69 | 4.02 |
| Run `node -e 0` after startup | — | 20.86 | 21.53 | — |

The macOS directory-switch cases did not meet the publish run-count requirement for every
tool and are omitted here. Do not combine macOS and Docker numbers into a speedup ratio.

## Compatibility tradeoff

This mode is useful when a shell normally starts with a fixed default Node. It does not run
the full source-time `nvm use` logic: an exact default is selected directly even if the initial
working directory has a different `.nvmrc`, and some environment setup performed by eager
`nvm use` is deferred. An alias such as `22` is not an exact version and takes the lazy-stub
path instead. Use eager mode when its full nvm startup behavior matters. With lazy path, add
the directory hook if the shell should switch versions after `cd`.
