# Listing and directory-hook optimization

The previous implementation at commit `107b1be` and the optimized build were measured with the
same `bench.py` (SHA-256 `aa03bbca0dd1`), pins, correctness checks, and case definitions. Each
short case ran 300 times in each of three shuffled rounds. The saved metadata records full
binary and shell-file hashes, CPU environment, and load. Docker ran on pinned CPUs 2 and 3 with
no network; the macOS runs used the isolated `bench/.sandbox/host` environment. Numbers below
are **minimum per-operation milliseconds** after subtracting the corresponding shell startup.

| Bash operation | Docker before | Docker after | macOS before | macOS after |
| --- | ---: | ---: | ---: | ---: |
| `ls`, 102 installed versions and 20 extra aliases | 1.12 | 0.44 | 4.42 | 2.72 |
| `ls --no-alias`, 102 versions | 0.26 | 0.22 | 2.11 | 2.26 |
| `cd` between directories without `.nvmrc` | 0.21 | 0.07 | 2.09 | 0.19 |

The Docker listing and hook minima were corroborated by a focused rerun: 0.99 to 0.47 ms for
`ls`, 0.24 to 0.23 ms for `ls --no-alias`, and 0.18 to 0.07 ms for `cd`. The full optimized
Docker run measured fnm at 0.42 ms for `ls` and 0.01 ms for this `cd` case. Thus the ordinary
nvmr listing is now close to fnm in this environment, while a no-change directory move still
costs more. `ls --no-alias` showed little change, which points to alias resolution as the main
listing gain.

All Docker runs stayed below the harness load threshold. Listing spread exceeded 10% between
rounds, including on the reruns, so the table uses labelled minima rather than medians. The
first optimized `cd` run was stable (0.08 ms median, 5% spread); the before run had 13% spread.
Both macOS runs exceeded the load threshold and had large round spread. Their minima suggest
the same direction, especially for the hook, but should not be treated as typical host latency.
The two environments are not mixed in any ratio or ranking.

## What changed

- During `ls`, the Rust store reuses installed-version scans while resolving aliases. Alias
  rendering also computes the current version once and avoids repeated installed-file checks.
  The Docker profile's traced syscall count for `source + ls` with 102 versions fell from
  4,298 to 3,715.
- The bash/zsh `nvm_auto_cd` hook checks `.nvmrc` ancestry in the shell. With no `.nvmrc`, it
  skips nvmr when there is no default alias, an exact default is already active, or a major-only
  default has just one matching installed version. Ambiguous defaults, changed `PATH`, and
  project `.nvmrc` files still use the native resolver. The dedicated hook test covers those
  fallbacks; the full upstream gate ended `ALL GREEN`.

## Saved runs

Each directory contains `report.md`, `summary.json`, `meta.json`, and `profile.md`. Raw hyperfine
JSON remains local. The baseline and optimized runs have identical `bench.py` hashes in their
metadata, while their nvmr binary and shell-file hashes identify the builds.

| Environment | Previous implementation | Optimized implementation |
| --- | --- | --- |
| Docker full comparison | [151001](../../bench/results/20261106-151001-docker-aarch64/report.md) | [151330](../../bench/results/20261106-151330-docker-aarch64/report.md) |
| Docker focused rerun | [151515](../../bench/results/20261106-151515-docker-aarch64/report.md) | [151719](../../bench/results/20261106-151719-docker-aarch64/report.md) |
| macOS host | [151905](../../bench/results/20261106-151905-host-darwin-arm64/report.md) | [152916](../../bench/results/20261106-152916-host-darwin-arm64/report.md) |

The remaining directory-hook gap is the cost of checking parent directories for `.nvmrc` and
reading the default alias in the shell. A cache would need to detect new parent `.nvmrc` files
and externally changed aliases or installs to preserve the current behavior.
