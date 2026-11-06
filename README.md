# nvmr

[![Release](https://img.shields.io/github/v/release/HelgeSverre/nvmr?style=flat-square)](https://github.com/HelgeSverre/nvmr/releases)
[![Release bundles](https://github.com/HelgeSverre/nvmr/actions/workflows/release-bundles.yml/badge.svg)](https://github.com/HelgeSverre/nvmr/actions/workflows/release-bundles.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)

A native Node.js version manager written in Rust. It keeps the `nvm` command, your installed versions and aliases, and your `.nvmrc` files. A small shell function applies environment changes from the native binary to your current shell.

**Experimental.** nvmr targets [nvm v0.40.8](https://github.com/nvm-sh/nvm/tree/v0.40.8). Linux is checked against upstream in Docker. macOS bundles receive an install smoke test, but the full parity suite has not run on macOS. See [compatibility](#compatibility) before replacing an existing nvm setup.

## Installation

### Homebrew

```sh
brew install HelgeSverre/tap/nvmr
```

Homebrew installs the binaries and shell files without editing your profile. Add the following to `~/.zshrc` or `~/.bashrc`, replacing `/opt/homebrew` with the output of `brew --prefix` on your machine:

```sh
export NVM_DIR="$HOME/.nvm"
export NVMR_EXE="/opt/homebrew/opt/nvmr/bin/nvmr"
. "/opt/homebrew/opt/nvmr/share/nvmr/nvm.sh"
```

For fish, put this in `~/.config/fish/config.fish` with the same prefix adjustment:

```fish
set -gx NVM_DIR $HOME/.nvm
set -gx NVMR_EXE /opt/homebrew/opt/nvmr/bin/nvmr
source /opt/homebrew/opt/nvmr/share/nvmr/nvm.fish
```

The `opt/nvmr` symlink follows Homebrew upgrades. Set a different `NVM_DIR` in these lines if you already use one.

### Release bundle

Release bundles cover macOS and Linux on arm64 and x86_64. Download the installer from the tagged release, then point it at that release's bundles:

```sh
curl -fsSLO https://raw.githubusercontent.com/HelgeSverre/nvmr/v0.1.0/install.sh
NVMR_URL=https://github.com/HelgeSverre/nvmr/releases/download/v0.1.0 sh install.sh
```

The installer chooses `$XDG_CONFIG_HOME/nvm` when XDG is set, otherwise `~/.nvm`. It adds the shell startup lines to a detected profile and installs fish integration when fish is available. Open a new shell, or load bash/zsh immediately:

```sh
export NVM_DIR="$HOME/.nvm" # Use the directory printed by the installer if different.
. "$NVM_DIR/nvm.sh"
```

An existing nvm installation shares its Node versions and aliases with nvmr. The installer replaces the integration files in `NVM_DIR`; keep a copy of them if you want an easy rollback. Set `PROFILE=/dev/null` to skip POSIX profile edits. See [install.sh](install.sh) for `NVMR_BUNDLE`, `NVMR_URL`, and other options.

### Build a Linux bundle from source

From a checkout on a Linux host with Docker:

```sh
docker build -t nvmr-dev docker
scripts/d.sh sh -c 'touch src/*.rs src/bin/*.rs && sh /src/scripts/bundle.sh'
NVMR_BUNDLE="dist/nvmr-linux-$(docker run --rm nvmr-dev uname -m)" sh install.sh
```

## Quick start

The shell command is `nvm`; it works with existing nvm commands and project files:

```sh
nvm install --lts          # Install and use the latest LTS release.
nvm use 22                 # Switch to an installed Node 22.
nvm alias default 22       # Select the version for new shells.
nvm current                # Show the active version.
nvm ls                     # List installed versions and aliases.
nvm exec 22 node app.js    # Run a command with Node 22.
```

For a project-specific version:

```sh
echo 22 > .nvmrc
nvm install
nvm use
```

If you prefer the project name as a command, add `alias nvmr='nvm'` after the integration loads. Run `nvm --help` or read the [command reference](src/help.txt) for the complete command list.

### Switch when changing directories

After sourcing `nvm.sh`, add a hook to your shell profile to select the nearest `.nvmrc` or return to your default version. For zsh:

```zsh
autoload -U add-zsh-hook
add-zsh-hook chpwd nvm_auto_cd
```

For bash:

```bash
cd() { builtin cd "$@" && nvm_auto_cd; }
```

The hook handles common no-change directory moves in the shell and calls nvmr when it needs to resolve or switch a version.

### Startup options

Set these before sourcing `nvm.sh`:

| Option | Effect |
| --- | --- |
| `NVM_LAZY=1` | Defer automatic selection until the first `nvm`, `node`, `npm`, `npx`, `corepack`, `yarn`, or `pnpm` call. Fish skips startup auto-use; run `nvm use` yourself. |
| `NVM_LAZY_PATH=1` | With lazy mode, set `PATH` directly when the default alias names an installed exact version. |
| `. "$NVM_DIR/nvm.sh" --no-use` | Load the POSIX integration without selecting a version. |

For an exact default version, lazy path avoids a manager process at startup. The
[pinned comparison](docs/performance/lazy-startup-2026-11-06.md) measures it both with and without
the directory hook. It is opt-in because it defers part of eager `nvm use` setup and does not
select an initial project's `.nvmrc` over the exact default.

## Compatibility

The [behavioral specification](nvm-compat-spec.md) records the upstream contract, intentional quirks, and [known differences](nvm-compat-spec.md#15-implementation-status-2026-11-06). nvmr is an independent implementation, not a copy of nvm's shell internals.

| Area | Status |
| --- | --- |
| Linux arm64 and x86_64 | Release bundles; the full parity gate has passed on arm64 in Docker. |
| macOS arm64 and x86_64 | Native release bundles with install smoke checks; full parity remains untested. |
| Shells | bash, zsh, POSIX sh, and fish. |
| Existing installs | Shares Node versions, aliases, and `.nvmrc` files in the upstream layout. |
| Legacy versions | Recognizes existing io.js and pre-0.12 Node installs; installing io.js is unsupported. |

## Development

Build and run parity checks in Docker:

```sh
docker build -t nvmr-dev docker
scripts/verify.sh
```

The gate runs Rust formatting, Clippy, unit tests, and 753 command comparisons against upstream nvm v0.40.8. It also checks color, bash/zsh/fish integration, read-only installs, interrupts, and portability. See [scripts/verify.sh](scripts/verify.sh) for the complete checks and [performance](docs/performance/README.md) for the pinned Docker comparisons with nvm, fnm, mise, and proto.

## License

[MIT](LICENSE).
