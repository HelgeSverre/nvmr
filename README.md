# nvmr

[![Latest release](https://img.shields.io/github/v/release/HelgeSverre/nvmr?style=flat-square)](https://github.com/HelgeSverre/nvmr/releases)
[![MIT license](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)

nvmr runs the `nvm` command with a Rust executable and a small shell function for environment changes. It uses the same `$NVM_DIR` layout, installed Node versions, aliases, and `.nvmrc` files as [nvm](https://github.com/nvm-sh/nvm).

**Experimental.** The compatibility target is nvm v0.40.8. See [compatibility](#compatibility) for test coverage and known differences.

## Install

Release bundles are available for Linux and macOS on arm64 and x86_64. Shell integration supports bash, zsh, POSIX sh, and fish.

### Homebrew

```sh
brew install HelgeSverre/tap/nvmr
```

Homebrew installs the files but does not edit your shell profile. Add this to `~/.zshrc` or `~/.bashrc`. Replace `/opt/homebrew` with your `brew --prefix` output if it differs:

```sh
export NVM_DIR="$HOME/.nvm"
export NVMR_EXE="/opt/homebrew/opt/nvmr/bin/nvmr"
. "/opt/homebrew/opt/nvmr/share/nvmr/nvm.sh"
```

For fish, add this to `~/.config/fish/config.fish` with the same prefix adjustment:

```fish
set -gx NVM_DIR $HOME/.nvm
set -gx NVMR_EXE /opt/homebrew/opt/nvmr/bin/nvmr
source /opt/homebrew/opt/nvmr/share/nvmr/nvm.fish
```

The `opt/nvmr` path follows Homebrew upgrades. Keep your existing `NVM_DIR` if you already use nvm.

### Release bundle

Download the installer from the release you want and point it at that release's bundles:

```sh
curl -fsSLO https://raw.githubusercontent.com/HelgeSverre/nvmr/v0.1.1/install.sh
NVMR_URL=https://github.com/HelgeSverre/nvmr/releases/download/v0.1.1 sh install.sh
```

The installer chooses `$XDG_CONFIG_HOME/nvm` when `XDG_CONFIG_HOME` is set, otherwise `~/.nvm`. It edits a detected shell profile; set `PROFILE=/dev/null` to skip that step. It replaces the integration files in `NVM_DIR`, so save a copy of an existing `nvm.sh` if you want a simple rollback. See [install.sh](install.sh) for local-bundle and profile options.

Open a new shell after installation. To activate it immediately in bash or zsh:

```sh
export NVM_DIR="$HOME/.nvm" # Use the directory printed by the installer if different.
. "$NVM_DIR/nvm.sh"
```

## Use

The shell command is `nvm`. Use it as you would upstream nvm:

```sh
nvm install 22             # Install and use the latest Node 22 release.
nvm alias default 22       # Use Node 22 in new shells.
nvm use 22                 # Switch this shell to an installed Node 22.
nvm current                # Show the active version.
nvm ls                     # Show installed versions and aliases.
```

For a project, put a version in `.nvmrc`:

```sh
echo 22 > .nvmrc
nvm install                # Install the version requested by .nvmrc.
nvm use                    # Select it in this shell.
```

Call `nvm` through the shell integration when you need to change the current shell's `PATH`. The standalone `nvmr` binary cannot change its parent shell. If you want a command named `nvmr`, define `alias nvmr='nvm'` after loading the integration.

### Commands

| Command | Purpose |
| --- | --- |
| `nvm install [version]` | Install and use a version; without an argument, read `.nvmrc`. |
| `nvm uninstall <version>` | Remove an installed version. |
| `nvm use [version]` | Select a version in the current shell; without an argument, read `.nvmrc`. |
| `nvm current` | Show the version active on `PATH`. |
| `nvm ls [version]` | List installed versions and aliases, optionally filtered. |
| `nvm ls-remote [version]` | List available remote versions. |
| `nvm version <version>` | Resolve a local version or alias. |
| `nvm version-remote <version>` | Resolve a remote version or alias. |
| `nvm which [version]` | Print the installed Node executable path. |
| `nvm alias [name] [version]` | List aliases or set an alias. |
| `nvm unalias <name>` | Remove an alias. |
| `nvm exec <version> <command>` | Run a command with that version on `PATH`. |
| `nvm run <version> [args]` | Run Node with that version. |
| `nvm reinstall-packages <version>` | Reinstall global npm packages from another installed version. |
| `nvm install-latest-npm` | Upgrade npm for the current Node version. |
| `nvm deactivate` | Remove the selected Node version from this shell. |
| `nvm unload` | Remove the nvm shell integration. |
| `nvm cache dir` / `nvm cache clear` | Show or clear the download cache. |
| `nvm set-colors [codes]` | Set the five output colors. |
| `nvm debug` | Print environment and installation diagnostics. |
| `nvm --help` | Show all commands and flags. |
| `nvm --version` | Show the upstream compatibility version. |

Common flags include `--lts`, `--save`, `--offline`, `-s` (source only), and `-b` (binary only). See [the full command reference](src/help.txt) for flag placement and additional install options. `nvm --version` reports the upstream compatibility version, `0.40.8`; the [releases page](https://github.com/HelgeSverre/nvmr/releases) shows the nvmr package version.

### Switch on directory changes

The optional hook selects the nearest `.nvmrc` when you change directories and restores your default version when you leave a project. Add it **after** sourcing `nvm.sh`.

For zsh:

```zsh
autoload -U add-zsh-hook
add-zsh-hook chpwd nvm_auto_cd
```

For bash:

```bash
cd() { builtin cd "$@" && nvm_auto_cd; }
```

Fish installs its directory hook with `nvm.fish`. The bash and zsh hook handles common no-change moves in the shell and calls nvmr when version resolution or switching is needed.

### Startup options

Set these before sourcing `nvm.sh`:

| Setting | Effect |
| --- | --- |
| `NVM_LAZY=1` | Defer selection until the first `nvm` or Node-related command. In fish, skip startup selection and run `nvm use` yourself. |
| `NVM_LAZY=1 NVM_LAZY_PATH=1` | Put an installed exact `alias/default` version on `PATH` without launching nvmr at startup. |
| `. "$NVM_DIR/nvm.sh" --no-use` | Load the POSIX integration without selecting a version. |

Lazy path is opt-in: it does not select an initial project's `.nvmrc` over the exact default and defers some eager `nvm use` setup. See the [measured comparison](docs/performance/lazy-startup-2026-11-06.md).

## Compatibility

nvmr targets nvm v0.40.8, including its on-disk version and alias layout. An existing nvm installation can share `NVM_DIR` with nvmr. It recognizes installed io.js and pre-0.12 Node versions; installing io.js is unsupported.

The [specification](nvm-compat-spec.md) records the behavior contract and [known differences](nvm-compat-spec.md#15-implementation-status-2026-11-06). Linux arm64 has passed the full Docker parity gate against upstream; the macOS bundles have build and installation smoke coverage, but not the full parity suite. See [performance results](docs/performance/README.md) for pinned comparisons with nvm, fnm, mise, and proto.

## Development

Build the dev image and run the verification gate from a checkout:

```sh
docker build -t nvmr-dev docker
scripts/verify.sh
```

The gate runs a clean Rust build, formatting, Clippy, unit tests, upstream command comparisons, shell and color checks, read-only installation, interrupts, and portability checks. To build a Linux release bundle locally:

```sh
scripts/d.sh sh -c 'touch src/*.rs src/bin/*.rs && sh /src/scripts/bundle.sh'
```

See [scripts/verify.sh](scripts/verify.sh) and [the benchmark harness](bench/docker.sh) for details.

## License

[MIT](LICENSE).
