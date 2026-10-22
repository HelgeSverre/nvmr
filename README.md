# nvm-fast

A native, drop-in replacement for [nvm](https://github.com/nvm-sh/nvm) (compatible with v0.40.8).
Same commands, same output, same exit codes, same `$NVM_DIR` layout, so it can replace the shell version in place,
but a command takes about a millisecond instead of up to seconds.

| | nvm 0.40.8 (bash 5, Linux arm64 VM) | nvm-fast |
|---|---:|---:|
| `source nvm.sh` with default auto-use (zsh) | 240 ms | 4.7 ms |
| `nvm use 22` | 68 ms | 1.0 ms |
| `nvm ls` (122 versions) | 2,063 ms | 1.2 ms |
| `nvm current` | 8.7 ms | 1.3 ms |

(Container numbers from `scripts/perf.sh`; medians on a busy machine. macOS numbers were not measured in this session.
The design and budgets are in `nvm-compat-spec.md` §13.)

## How it works

`nvm` is still a tiny shell function (`nvm.sh`), because a child process cannot change your shell's `PATH`.
The function runs `nvm-core` (one process, all logic) and sources the few `export`/`unset` statements it queued.
HTTPS download, checksum and extraction live in a second binary, `nvm-fetch`, so `nvm-core` links only libc and starts fast.

## Install

```sh
NVM_FAST_BUNDLE=dist/nvm-fast-linux-aarch64 sh install.sh   # or NVM_FAST_URL=<base url>
```

This copies `nvm.sh nvm-core nvm-fetch nvm-exec bash_completion nvm.fish` into `~/.nvm` and appends the same three
profile lines upstream's installer adds, so existing setups need no changes. It is safe to install over an existing
upstream `~/.nvm`: versions and aliases are shared byte for byte.

Shell support: bash, zsh, dash/POSIX sh, and fish (`nvm.fish`, installed to `conf.d` automatically). Extras over upstream:
`NVM_LAZY=1` (stub `node`/`npm`/... until first use), `NVM_LAZY_PATH=1`, `nvm-exec` without a shell, fish support.

## Not supported

Installing io.js (existing io.js installs under `versions/io.js` and pre-0.12 node installs in the old `$NVM_DIR/vX`
layout are listed, resolved, used and uninstalled like upstream), Windows/MSYS, SunOS/AIX/BSD. Linux and macOS are the
targets. **macOS has not been built or tested yet**: everything here was developed and verified on Linux inside Docker.

## Development (everything runs in Docker, nothing on the host)

```sh
scripts/d.sh cargo build --release        # build in the dev container
scripts/verify.sh                         # clean-room gate: fresh build + every parity check below
scripts/d.sh sh /src/scripts/bundle.sh    # static musl release bundle into dist/
```

`docker/Dockerfile` builds the dev image (`docker build -t nvmfast-dev docker`): Rust, bash, zsh, fish, hyperfine,
and upstream nvm v0.40.8 at `/opt/nvm-upstream`.

The gate (`scripts/all.sh`) runs the same command sequences through upstream nvm and `nvm-core`, with real downloads, and
diffs stdout, stderr, exit codes and the resulting shell environment:

- `cargo fmt --check`, `cargo clippy -- -D warnings` and the unit tests
- `scenario*.sh`: 377 commands (resolution, install/uninstall, aliases, `.nvmrc`, `lts/*`, npm prefix guard, system node,
  offline and mirror errors, locks, hooks, global package reinstall, source builds against a fake mirror, io.js and
  old-layout installs, ...)
- `sigint-test.sh`: Ctrl-C mid-download (partial tarball kept and resumed), mid-extraction (no half-extracted tree), and
  during `nvm install` (rc 130, install lock released)
- `colour-cmp.sh`: coloured output on a pseudo-terminal
- `shell-test.sh`, `fish-test.sh`: bash/zsh/fish sessions including the README `cdnvm`/`chpwd` auto-switch recipes
- `portability.sh`: dash, bash `--posix`, `set -eu`, lazy modes, completion

Upstream quirks that the native build reproduces on purpose are listed in `nvm-compat-spec.md` §12.
