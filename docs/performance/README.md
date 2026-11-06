# Performance

See the [listing and directory-hook optimization follow-up](optimization-2026-11-06.md) for
controlled before/after Docker and macOS runs. The measurements below describe the original
implementation before that optimization.

Measured on 2026-11-06 in the pinned Linux arm64 Docker harness. These results describe this
environment, not macOS or an arbitrary developer machine.

## Method

The [benchmark harness](../../bench/docker.sh) builds the release bundle and puts nvmr, upstream
nvm v0.40.8, and fnm v1.39.0 in the same Debian image. Every tool gets Node v22.22.3 as its
default and v20.20.2 as its switch target. The container has no network while measuring, uses
CPUs 2 and 3, a 2 GB memory limit, a tmpfs work directory, and fixed shell environment. Install
tests fetch the same archive from a loopback mirror. Each command passes a correctness check
before timing. Short cases use 300 runs per round, three rounds in shuffled order; install and
other slow cases use 5–20 runs per round. In-shell operation times subtract the corresponding
shell startup and divide by the operation count.

The saved results contain the full commands, versions, pins, load, raw-derived summary, and
profile. Raw hyperfine JSON remains local.

- [B: before the shell change](../../bench/results/20261106-134135-docker-aarch64/report.md)
  ([metadata](../../bench/results/20261106-134135-docker-aarch64/meta.json),
  [profile](../../bench/results/20261106-134135-docker-aarch64/profile.md))
- [P: comparison after the change](../../bench/results/20261106-135000-docker-aarch64/report.md)
  ([metadata](../../bench/results/20261106-135000-docker-aarch64/meta.json),
  [profile](../../bench/results/20261106-135000-docker-aarch64/profile.md))
- [R: focused rerun](../../bench/results/20261106-141047-docker-aarch64/report.md)
  ([metadata](../../bench/results/20261106-141047-docker-aarch64/meta.json))
- [S: fnm current and nvm install rerun](../../bench/results/20261106-142032-docker-aarch64/report.md)
  ([metadata](../../bench/results/20261106-142032-docker-aarch64/meta.json))

All four runs stayed below the harness's load threshold. Some individual commands still varied
substantially between rounds. The table uses medians only for rows with a round spread at or below
10%; otherwise it says **min**. A min is the best observed time, not a typical time. Times are
milliseconds unless marked seconds. B, P, R, and S identify the result directory above.

## Comparison

| Operation | nvm | nvmr | fnm |
| --- | ---: | ---: | ---: |
| Bash startup, eager default | 56.41 P | 0.66 **min** R | 0.67 P |
| Bash `current`, per call | 3.93 P | 0.20 R | 0.35 S |
| Bash `ls`, 102 installed versions, per call | 506 **min** R | 1.13 P | 0.47 P |
| Bash `cd`, no version file, native hooks, per change | — | 0.21 P | 0.01 P |
| Install v20.20.2 from loopback mirror, seconds | 1.569 S | 1.410 P | 1.373 P |

The `ls` comparison is useful as a workflow comparison, but its output differs: nvmr also prints
nvm aliases, while fnm does not. The nvm listing row had more than 10% round spread on the rerun;
the large gap to nvmr is clear, but its displayed minimum should not be read as a typical nvm
time. Eager startup for nvmr, and switching or directory changes for nvmr,
remained unstable even on the focused rerun. The [full report](../../bench/results/20261106-135000-docker-aarch64/report.md)
records their observed ranges without a stable median claim here.

The bare `nvmr ls` process took 0.19 ms median, versus 0.31 ms for `fnm ls` in P. That measures
process invocation with the small installed state, rather than the 102-version shell workflow.
The stripped nvmr core is 791 KB and peaks at 660 KB RSS on the profiled hot commands; fnm is
8.4 MB and peaks at 2,328 KB RSS in the same [profile](../../bench/results/20261106-135000-docker-aarch64/profile.md).

## What changed

The shell wrapper previously used command substitution to choose `bash` or `zsh` on every `nvm`
call. [nvm.sh](../../nvm.sh) now chooses with shell builtins. The before and after profiles show
`source + use` falling from two forks and 3,348 traced syscalls (B) to one fork and 1,908 syscalls
(P). In a stable `current` comparison, the per-call median fell from 0.33 ms (B, 4% round spread)
to 0.20 ms (R, 9% spread). The binary did not change. The full upstream parity gate ended
`ALL GREEN` after the shell edit.

## Next opportunities

1. The native `cd` hook still starts nvmr when moving between directories without a version file.
   fnm's hook does almost no work in that case. A shell-side no-op guard could help, but it must
   preserve parent `.nvmrc` lookup and default reversion; benchmark and parity-test it first.
2. `nvmr ls` is still slower than `fnm ls` with many versions. nvmr also renders aliases. Its
   [listing path](../../src/main.rs) enumerates versions and aliases and resolves implicit aliases;
   caching the installed version list within one command is a plausible next experiment. Measure
   `--no-alias` separately before attributing the whole gap to Rust work.
3. nvmr and fnm installs are close in the loopback test. Extraction dominates, so there is no
   clear reason to change that path from these results.

For a new publishable run, use `bench/docker.sh --publish --filter '<cases>'`. Keep the pins and
CPU set fixed, commit the four generated result files when citing the run, and inspect the load
flag and round spread before comparing medians. `--quick` is for development only.
