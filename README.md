<!-- Hallmark · pre-emit critique: P5 H5 E4 S5 R5 V4 -->
![nvmr — Switch Node. Keep your flow.](website/assets/brand/readme-banner.svg)

# nvmr

**A native Node.js version manager, written in Rust.**

Keep your nvm commands, installed versions, aliases, and `.nvmrc` files. nvmr runs the version-management logic in a native process, with a thin shell function to update your environment.

[Install](#install) · [Quick start](#quick-start) · [Compatibility](#compatibility) · [Benchmarks](#benchmarks) · [One-page website](website/index.html) · [Brand guide](website/BRAND.md)

> **Project name: `nvmr`. Shell command: `nvm`.** The existing command name keeps scripts and shell integrations compatible. Prefer the branded command? Add `alias nvmr='nvm'` after loading nvm in your profile.

## Install

**Linux, from source.** Run these commands from a checkout on a Linux host with Docker running. The bundle is built for the container’s architecture. macOS is a target but has not yet been built or tested.

```sh
docker build -t nvmr-dev docker
scripts/d.sh sh /src/scripts/bundle.sh

NVMR_BUNDLE="dist/nvmr-linux-$(docker run --rm nvmr-dev uname -m)" sh install.sh
```

Open a new terminal. For a default installation at `~/.nvm`, you can also load the integration in the current bash/zsh session:

```sh
export NVM_DIR="$HOME/.nvm"
. "$NVM_DIR/nvm.sh"
```

If you set `NVM_DIR` or `XDG_CONFIG_HOME`, use the installation directory printed by the installer instead.

The installer copies `nvm.sh`, `nvmr`, `nvmr-fetch`, `nvm-exec`, `bash_completion`, and `nvm.fish` into `$NVM_DIR`. Existing Node versions and aliases are shared with upstream nvm. Profile lines are added when needed; fish integration is installed into `conf.d` when fish is available.

The installer replaces the integration files in that directory. Keep a copy of your previous integration files if you want an easy rollback.

<details>
<summary>Installer options and existing bundles</summary>


| Option | Purpose |
| --- | --- |
| `NVMR_BUNDLE` | Install from a local directory containing the six bundle files. |
| `NVMR_URL` | Download `nvmr-<os>-<arch>.tar.gz` from a base URL you provide. |
| `NVM_DIR` | Choose the installation directory; defaults to `$XDG_CONFIG_HOME/nvm` when set, otherwise `~/.nvm`. |
| `PROFILE` | Choose a shell profile; `PROFILE=/dev/null` skips POSIX profile edits. Fish integration is still installed when fish is available. |

Example with an existing Linux arm64 bundle:

```sh
NVMR_BUNDLE=dist/nvmr-linux-aarch64 sh install.sh
```

See [install.sh](install.sh) for the full installer. There is no default hosted download endpoint configured.

</details>

## Quick start

```sh
nvm install --lts          # Install and use the latest LTS
nvm use 22                 # Switch to an installed Node 22
nvm alias default 22       # Set the default for new shells
nvm current                # Show the active version
nvm ls                     # List installed versions and aliases
nvm exec 22 node app.js    # Run a command using Node 22
```

Let a project choose its version:

```sh
echo "22" > .nvmrc
nvm install
nvm use
```

For the branded command name, add this after nvm loads in your shell profile. bash, zsh, and fish support this alias syntax:

```sh
alias nvmr='nvm'
```

Then use `nvmr install --lts`, `nvmr use`, or any other nvm command. Existing `nvm` calls keep working.

Run `nvm --help` for all commands, or read the [command reference](src/help.txt).

## Compatibility

nvmr targets the behavior of [nvm-sh/nvm v0.40.8](https://github.com/nvm-sh/nvm/tree/v0.40.8): commands, output, exit codes, and the `$NVM_DIR` layout. It is an independent implementation. The [behavioral specification](nvm-compat-spec.md) records the contracts and upstream quirks reproduced on purpose. Some edge cases still differ; see the [known differences in §15](nvm-compat-spec.md#15-implementation-status-2026-10-22).

| Area | Current status |
| --- | --- |
| Linux | Built and verified in Docker. |
| macOS | Target platform; not yet built or tested. |
| Shells | bash, zsh, dash/POSIX sh, and fish. |
| Existing installs | Node versions, aliases, and `.nvmrc` files use the upstream layout. |
| Legacy installs | Existing io.js and pre-0.12 Node installs can be recognized, listed, resolved, used, and uninstalled. Installing io.js is unsupported. |
| Other platforms | Windows/MSYS, SunOS, AIX, and BSD are outside the current scope. |

### Startup options

Set these before loading `nvm.sh`:

```sh
export NVM_LAZY=1
. "$NVM_DIR/nvm.sh"
```

- `NVM_LAZY=1` defers automatic version selection until the first `nvm`, `node`, `npm`, `npx`, `corepack`, `yarn`, or `pnpm` call. In fish it skips startup auto-use; run `nvm use` yourself.
- `NVM_LAZY_PATH=1`, together with lazy mode, can set the path directly when the default alias contains an installed exact version.
- `. "$NVM_DIR/nvm.sh" --no-use` loads the POSIX integration without selecting a version.

## Benchmarks

The [benchmark harness](bench/bench.py) measures shell startup, commands, and project switching against pinned inputs. [bench/docker.sh](bench/docker.sh) provides the controlled reference run; [bench/host.sh](bench/host.sh) measures the developer machine in an isolated sandbox.

Publish performance figures only with their saved `bench/results/<run>/` report and environment details. There are no saved results included with this page yet. See [specification §13](nvm-compat-spec.md#13-performance-baseline-budgets-profiling-competitors) for performance requirements and methodology.

## How it works

```text
Your shell                     Native processes
───────────────────────────    ──────────────────────────────
nvm() shell function     ────► nvmr
  applies environment    ◄────   resolves versions and PATH
  changes to this shell          │
                                 └── nvmr-fetch
                                       HTTPS · checksums
                                       download · extraction
```

A child process cannot change its parent’s environment. The shell function runs `nvmr`, then sources the environment changes it queued. Keeping HTTPS and extraction in `nvmr-fetch` keeps that machinery out of the everyday command path. `nvm-exec` can launch a command with the selected environment directly.

The project name is `nvmr`; `nvmr`, `nvmr-fetch`, `nvm.sh`, and bundle names remain implementation interfaces.

## Development

Rust builds and parity checks run in Docker. The dev image includes Rust, bash, zsh, fish, hyperfine, and upstream nvm v0.40.8 at `/opt/nvm-upstream`.

```sh
docker build -t nvmr-dev docker
scripts/d.sh cargo build --release
scripts/verify.sh
scripts/d.sh sh /src/scripts/bundle.sh
```

[scripts/verify.sh](scripts/verify.sh) uses a fresh Cargo target volume and runs [scripts/all.sh](scripts/all.sh). The gate compares stdout, stderr, exit codes, and resulting shell environments against upstream, including real downloads:

- Rust formatting, Clippy with warnings denied, and unit tests.
- Version resolution, installs, aliases, `.nvmrc`, LTS, source builds, mirrors, offline behavior, locks, hooks, global packages, and legacy layouts.
- Interrupted downloads, extraction, and installs.
- Colored output, bash/zsh/fish integration, POSIX portability, lazy modes, and completion.

### Website and brand

The [one-page website](website/index.html) is buildless HTML, CSS, and JavaScript, with local fonts. All website files, brand assets, and design records live in `website/`. Open `website/index.html` directly, or serve the repository root with a static HTTP server and visit `/website/` to enable clipboard buttons.

For publication, copy `website/` and its linked project resources into a dedicated public directory, preserving their relative locations: `README.md`, `install.sh`, `nvm-compat-spec.md`, `src/help.txt`, and `bench/bench.py` sit outside `website/`. Serve the public directory and visit `/website/`; no build step is needed.

The [brand guide](website/BRAND.md) documents naming, artwork, color, typography, and voice. All web design values live in [website/tokens.css](website/tokens.css).
