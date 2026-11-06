# nvmr

[![Latest release](https://img.shields.io/github/v/release/HelgeSverre/nvmr?style=flat-square)](https://github.com/HelgeSverre/nvmr/releases)
[![MIT license](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)

nvmr implements [nvm](https://github.com/nvm-sh/nvm) in Rust. It keeps the `nvm` shell command and uses your existing `$NVM_DIR`, installed Node versions, aliases, and `.nvmrc` files.

**Experimental.** It targets nvm v0.40.8; see [compatibility](#compatibility) for test coverage and known differences.

## Install

Bundles support Linux and macOS on arm64 and x86_64. Shell integration supports bash, zsh, POSIX sh, and fish.

### Homebrew

```sh
brew install HelgeSverre/tap/nvmr
```

Homebrew leaves your shell profile untouched. Add this to `~/.zshrc` or `~/.bashrc`, replacing `/opt/homebrew` with the output of `brew --prefix` if different:

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

The `opt/nvmr` path follows Homebrew upgrades. If you use nvm already, keep its `NVM_DIR`.

### Release bundle

Install v0.1.1:

```sh
curl -fsSLO https://raw.githubusercontent.com/HelgeSverre/nvmr/v0.1.1/install.sh
NVMR_URL=https://github.com/HelgeSverre/nvmr/releases/download/v0.1.1 sh install.sh
```

The installer uses `NVM_DIR` if set. Otherwise it chooses `$XDG_CONFIG_HOME/nvm` when `XDG_CONFIG_HOME` is set, or `~/.nvm`. It updates a detected shell profile; set `PROFILE=/dev/null` to skip that step.

It replaces the integration files in `NVM_DIR`. Back up `$NVM_DIR/nvm.sh` before installing over an existing nvm setup. See [install.sh](install.sh) for local-bundle and profile options.

Open a new shell after installation. To activate it immediately in bash or zsh:

```sh
export NVM_DIR="$HOME/.nvm" # Use the directory printed by the installer if different.
. "$NVM_DIR/nvm.sh"
```

## Use

Install Node 22 and make it the default for new shells:

```sh
nvm install 22
nvm alias default 22
nvm current
nvm ls
```

In a project, `nvm install` reads `.nvmrc` and selects the requested version:

```sh
echo 22 > .nvmrc
nvm install
```

Use the `nvm` shell function for commands that change `PATH`; the `nvmr` executable cannot modify its parent shell. To use the name `nvmr`, add `alias nvmr='nvm'` after loading the integration.

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
| `nvm --help` | Show usage and common flags. |
| `nvm --version` | Show the upstream compatibility version. |

Use `--lts` with `install` or `use`, and `--save` to write `.nvmrc`. For installation, `--offline`, `-s` (source only), and `-b` (binary only) control how Node is obtained. See the [CLI help](src/help.txt) for flag placement and other options.

### Switch after `cd`

Add a directory hook after sourcing `nvm.sh` to select the nearest `.nvmrc` on `cd` and restore your default when you leave a project.

For zsh:

```zsh
autoload -U add-zsh-hook
add-zsh-hook chpwd nvm_auto_cd
```

For bash:

```bash
cd() { builtin cd "$@" && nvm_auto_cd; }
```

`nvm.fish` installs the fish directory hook when sourced.

### Startup options

Set these before sourcing `nvm.sh`:

| Setting | Effect |
| --- | --- |
| `NVM_LAZY=1` | Defer selection until the first `nvm` or Node-related command. In fish, skip startup selection and run `nvm use` yourself. |
| `NVM_LAZY=1 NVM_LAZY_PATH=1` | Put an installed exact `alias/default` version on `PATH` without launching nvmr at startup. |
| `. "$NVM_DIR/nvm.sh" --no-use` | Load the POSIX integration without selecting a version. |

Lazy path starts with the exact default even when the initial directory has a different `.nvmrc`, and defers some `nvm use` setup. See its [behavior and measurements](docs/performance/lazy-startup-2026-11-06.md).

## Compatibility

nvmr recognizes older io.js and pre-0.12 Node installations, but cannot install io.js.

The parity gate passed against nvm v0.40.8 on Linux arm64 in Docker. macOS bundles pass build and installation smoke tests; the parity gate has not run on macOS. The [specification](nvm-compat-spec.md) records the behavior contract and [known differences](nvm-compat-spec.md#15-implementation-status-2026-11-06). See [performance results](docs/performance/README.md) for comparisons with nvm, fnm, mise, and proto.

`nvm --version` reports the nvm compatibility version, `0.40.8`. See [releases](https://github.com/HelgeSverre/nvmr/releases) for the nvmr package version.

## Development

Build the dev image and run the verification gate from a checkout:

```sh
docker build -t nvmr-dev docker
scripts/verify.sh
```

The gate checks the Rust build, formatting, Clippy, unit tests, upstream parity, shell behavior, read-only installs, interrupts, and portability. To build a Linux release bundle locally:

```sh
scripts/d.sh sh -c 'touch src/*.rs src/bin/*.rs && sh /src/scripts/bundle.sh'
```

See [scripts/verify.sh](scripts/verify.sh) and [the benchmark harness](bench/docker.sh) for details.

## License

[MIT](LICENSE).
