# nvmr

A native drop-in replacement for nvm (parity target: upstream nvm v0.40.8). The behaviour contract is
`nvm-compat-spec.md`; §12 lists upstream quirks reproduced on purpose and §15 the known, deliberate differences.

## Layout

- One crate, everything under `src/`. Keep it monolithic: do not add workspace members or split crates unless
  there is a concrete reason that a module cannot solve. The two binaries exist for startup latency only:
  `nvmr` (all logic, links libc only) and `src/bin/nvmr-fetch.rs` (HTTPS, checksum, extraction).
- `nvm.sh` / `nvm.fish` are thin shell integrations; `install.sh` installs a bundle built by `scripts/bundle.sh`.
- `scripts/` holds the parity harness; `bench/` holds the benchmark and profiling harness.

## Running things

- Never run nvm, nvmr, node installs or the parity harness on the host. Everything runs in Docker via
  `scripts/d.sh <cmd>` (dev image `nvmr-dev`, built from `docker/Dockerfile`, with upstream nvm at
  `/opt/nvm-upstream`). The only exception is `bench/host.sh`, which works in an isolated sandbox.
- Before `cargo build` in the container, `touch src/*.rs src/bin/*.rs`: the bind mount sometimes hides edits from
  cargo's mtime check. Parallel workers use separate build volumes: `NVMR_TARGET_VOL=<name> scripts/d.sh ...`.
- Gate: `scripts/verify.sh` (clean build, rustfmt, clippy `-D warnings`, unit tests, every scenario diffed against
  upstream, colour, bash/zsh/fish, read-only, Ctrl-C, portability) must end `ALL GREEN` before you report a change as
  done. A behaviour change comes with a scenario (`scripts/scenario*.sh`, compared by `scripts/compare.sh`) that
  shows it matches upstream. Scenario helper servers must be started as `cmd & SRV_PID=$!`.

## Code

- rustfmt and `cargo clippy --all-targets -- -D warnings` clean. Idiomatic Rust: `let ... else`, `?`, iterators,
  no panicking `unwrap()` outside tests and documented invariants.
- Exit codes and messages mirror upstream exactly; numeric return codes are the error type on purpose.

## Benchmarking and profiling

Every performance number in this repository (docs, spec, README, commit messages, PR descriptions) must come from
the harness in `bench/` and cite the results directory it came from. No ad-hoc `time` or one-off hyperfine numbers.

1. **Harness only.** `bench/docker.sh` is the reference run: a pinned image (`bench/Dockerfile`), no network,
   pinned CPU set (`BENCH_CPUS`), memory cap, tmpfs work dir, fixed environment (`HOME`, `PATH`, `LC_ALL=C`,
   `TERM=dumb`, no colours). `bench/host.sh` is the real-world run on the developer machine, in a sandbox under
   `bench/.sandbox` that never touches the user's own nvm/fnm setup or shell profiles. Report both when you claim
   a speed-up; never mix numbers from the two in one comparison.
2. **Pinned inputs.** Tool versions, base image digest, Debian snapshot, node versions and checksums live in
   `bench/pins.env`. Change pins only in a commit of their own that says why. Results are comparable only between
   runs with the same pins and the same `bench.py`.
3. **Measure the right thing.** Each benchmark has a correctness check that runs first (for example, after the
   tool's init `node -v` must print the pinned default). A failing check aborts the run. Add a check to every new
   benchmark.
4. **Fair comparisons.** Same node versions and the same on-disk state for every tool (the "many" state has 102
   versions); each tool's documented init line; upstream's README auto-switch recipes for nvm and nvmr alike;
   nvmr's own one-process hook is a separate, labelled configuration.
5. **Statistics.** `hyperfine -N` with warmups; at least 50 runs per ms-scale benchmark (300 for numbers you
   publish), at least 3 rounds in a seeded shuffled order. Report min, p10, the median of the per-round medians,
   MAD and the spread between rounds. Never report means. Sub-millisecond in-shell operations are measured as loops
   with the same shell's startup subtracted, divided by the operation count.
6. **Noise.** The harness records load averages per round and marks a run noisy when the 1-minute load exceeds
   a quarter of the CPUs it is averaged over (the whole machine or Docker VM, not the pinned CPU set). From a noisy
   run only min and p10 may be quoted, labelled as such. Rerun when the round spread exceeds 10 %. Close other heavy work before a run you intend to publish; on a laptop, stay on mains power.
7. **Results.** Each run writes `bench/results/<stamp>-<where>/` with `report.md`, `summary.json`, `meta.json`
   (environment, versions, pins, seed, load) and `profile.md`. Commit those four files when you cite the run;
   `raw/` (hyperfine JSON) stays local. A quick run (`--quick`) is for development and is never cited.
8. **Profiling.** `bench.py profile` (run by both wrappers) records binary sizes and linkage, process and syscall
   counts per operation (`strace -f`, Linux), `perf stat` where the kernel allows it, and peak RSS. For deeper
   work, profile a release build with symbols (`CARGO_PROFILE_RELEASE_DEBUG=true CARGO_PROFILE_RELEASE_STRIP=false`)
   using `perf record` in the container or `samply record` on macOS, and keep the command line and output next to
   the results. `NVM_TIMINGS=1` makes `nvmr` print its in-process setup/command/eval-write split to
   stderr; use it to locate a regression, not as a published number.
9. **Budgets.** `nvm-compat-spec.md` §13.2 lists the performance budgets. A change that moves a hot path is
   compared against the previous committed result with the same pins.
