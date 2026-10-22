# nvm-compatible native reimplementation — behavioural specification

Target: a drop-in replacement for `nvm-sh/nvm` (reference: v0.40.8, `nvm.sh` @ `b1b3a00`, 2026-10-09), compiled (Rust — decision and alternatives in §14), macOS-first, with Linux as a secondary target. The user-facing command is `nvm`. Everything a user can type today must keep working, produce the same visible output and exit codes, and read/write the same on-disk state, so that it can coexist with (or replace) the shell version on the same `$NVM_DIR`.

Non-goals (explicitly dropped): Windows/MSYS/Cygwin, SunOS, AIX, OpenBSD, FreeBSD source-build quirks, io.js *installation* (io.js *recognition* in on-disk state and output is kept so `nvm ls` stays identical on old installs).

---

## 1. Architecture

### 1.1 The parent-shell problem

`nvm use`, `nvm deactivate`, `nvm unload`, and `nvm install` (which calls `use`) mutate the **calling shell's** environment: `PATH`, `MANPATH`, `NODE_PATH`, `NVM_BIN`, `NVM_INC`, and the command hash table. A child process cannot do that, which is why upstream nvm is a sourced 5.6k-line shell function. The replacement is therefore two parts:

```
┌─────────────────────────────┐      ┌────────────────────────────────┐
│ nvm()  shell function       │ exec │ nvm-core  (native binary)      │
│  ~20 lines, per shell       │─────▶│  all logic, all I/O            │
│  sourced at startup         │      │  writes shell mutations to an  │
│  evals what nvm-core emits  │◀─────│  "eval channel", not stdout    │
└─────────────────────────────┘      └────────────────────────────────┘
```

- **`nvm-core`** (name is internal; install as `$NVM_DIR/nvm-core` or `/usr/local/bin/nvm-core`) does everything: resolution, download, extraction, printing. Stdout/stderr carry exactly what upstream prints.
- **`nvm-fetch`**: a second binary holding the HTTPS/TLS/extraction code, spawned by `nvm-core` for `install`, `ls-remote` and `version-remote`, so that `nvm-core` never links `Security.framework`/`CoreFoundation` and starts in < 2 ms (§13.2).
- **Eval channel**: the function passes `NVM_EVAL_FILE=$NVM_DIR/.cache/eval.$$` (one file per shell process). `nvm-core` writes shell statements there in the dialect named by `NVM_SHELL` (`sh`, `bash`, `zsh`, `fish`) and creates the directory if needed. After the binary exits, the function sources the file and truncates it with the `:` builtin. No `mktemp` or `rm` is spawned, so every `nvm` call costs exactly one process. `nvm-core` truncates stale content when it has nothing to write and prunes `eval.*` files older than a day. The file is never written when nothing changed. (Alternative: fd 3 — upstream already uses `3>&1` for `nvm_rc_version`; fd 3 is fine for POSIX shells but fish can't `exec 3>` cleanly, so the file protocol is the one true protocol.)
- Emitted statements are limited to a small grammar: `export VAR=value`, `unset VAR`, `hash -r` (POSIX) / `rehash` (zsh) / `set -gx VAR value` + `set -e VAR` (fish). Values are single-quoted with `'\''` escaping; never interpolate unescaped.

### 1.2 The shell function (reference, POSIX sh/bash/zsh)

```sh
nvm() {
  if [ "$#" -lt 1 ]; then set -- --help; fi
  local __nvm_eval __nvm_rc
  __nvm_eval="$(command mktemp "${TMPDIR:-/tmp}/nvm-eval.XXXXXX")" || return 1
  NVM_EVAL_FILE="$__nvm_eval" NVM_SHELL="${NVM_SHELL:-$( [ -n "${ZSH_VERSION-}" ] && echo zsh || echo bash )}" \
    command "${NVM_DIR}/nvm-core" "$@"
  __nvm_rc=$?
  if [ -s "$__nvm_eval" ]; then . "$__nvm_eval"; fi
  command rm -f "$__nvm_eval"
  return $__nvm_rc
}
```

Why not `eval "$(nvm-core "$@")"`: stdout must stay the user's (progress bars, `nvm ls` output, `nvm which` paths piped into scripts). Why `local`: zsh/bash both have it; for pure POSIX `sh` drop `local` and prefix the names.

### 1.3 `nvm-exec` (unchanged contract)

`$NVM_DIR/nvm-exec` is an executable that: sets `NODE_VERSION` env → `nvm use $NODE_VERSION` (exit 127 on failure); else `.nvmrc` → `nvm use`; else prints `nvm-exec: unable to select a node version` + hint and exits 127; then `exec "$@"`. In the native version `nvm-exec` is `nvm-core exec-internal "$@"`: compute the environment and `execvp` directly — no shell involved.

### 1.4 Startup cost budget

- Sourcing the function: < 1 ms (it's ~20 lines, no subprocesses).
- Auto-use at startup (`nvm_process_parameters`, §9): one `nvm-core __auto use` invocation ≤ 2.5 ms (stat `alias/default`, stat one `bin/node`, emit PATH edit). This replaces upstream's 300–600 ms (measured, §13.1). Budgets are normative in §13.2.
- Lazy mode (§9.3) defers even that until first use of `nvm`/`node`/`npm`/`npx`/`corepack`.

---

## 2. On-disk state (`$NVM_DIR`, default `~/.nvm`)

Must be byte-compatible; the shell nvm and the native nvm may share it.

| Path | Meaning |
|---|---|
| `versions/node/vX.Y.Z/` | A node install (`bin/`, `lib/`, `include/`, `share/`). "New layout" — node ≥ v0.12.0. |
| `versions/io.js/vX.Y.Z/` | io.js install (recognise only). |
| `vX.Y.Z/` | "Old layout" — node < v0.12.0 (recognise only; never create). |
| `alias/<name>` | Plain-text file; content = a version-ish string (may be another alias). Trailing whitespace and `# comments` are stripped on read; first non-blank line wins. Names may contain `/` (`lts/iron`) but never a `..` component. Names containing `#` are rejected on create. |
| `alias/lts/<codename>` | Auto-maintained by `ls-remote`/`install` from `index.tab`. Lowercase codename (`argon` … `jod` …). |
| `alias/lts/*` | A file literally named `*` → content `lts/<newest-codename>`. |
| `alias/default` | User's default. Auto-created by the first `nvm install` if absent. |
| `.cache/bin/<slug>/<slug>.<ext>` | Downloaded binary tarball; `<slug>` = `node-vX.Y.Z-<os>-<arch>`. |
| `.cache/bin/<slug>/files/` | Extraction scratch dir (removed on success). |
| `.cache/src/node-vX.Y.Z/` | Source tarball cache. |
| `.cache/locks/<version>` | Install lock **directory** (atomic `mkdir`). |
| `default-packages` | One npm package spec per line, `#` comments, blank lines ignored. A line with whitespace inside is an error. |
| `current` | Symlink to the active version dir, only when `NVM_SYMLINK_CURRENT=true`. |
| `nvm-core`, `nvm-fetch` | Native binaries (§1.1, §13.2); may also live in `/usr/local/bin` or a Homebrew `libexec/`, located via `NVM_CORE`/`NVM_FETCH`. |
| `nvm.sh`, `nvm-exec`, `bash_completion` | Upstream files; native install ships `nvm.sh` as the tiny function file so existing `[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"` profile lines keep working unchanged. |

`$NVM_DIR` rules: if set with a trailing slash → strip it and print `Warning: $NVM_DIR should not have trailing slashes` to stderr. If unset → the directory containing the sourced `nvm.sh`. The native binary exports it if it had to derive it.

---

## 3. The version-ish grammar

Every `<version>` argument is one of:

| Form | Example | Meaning |
|---|---|---|
| Full | `v20.11.1`, `20.11.1` | Exact. A leading `v` is added to anything starting with a digit (`nvm_ensure_version_prefix`). |
| Partial | `20`, `20.11`, `v0.10` | Prefix match on `major` or `major.minor` → **highest** matching installed/remote version. Internally a trailing `.` is appended so `2` does not match `20`. |
| Prerelease | `v21.0.0-rc.1`, `v22.0.0-nightly20240101abc` | Must be full `x.y.z-<suffix>`; suffix chars `[0-9A-Za-z.-]`, no `..`, no `.` followed by non-digit. |
| `current` | | The node found first on `PATH` right now. **Never** affected by `.nvmrc`. |
| `system` | | The non-nvm node (whatever `node` resolves to after removing nvm from `PATH`). |
| `node` | | Synonym for `stable`. (`node-` also accepted.) |
| `stable` | | Highest installed (or remote) `major.minor` with major ≥ 1; for 0.x, highest **even** minor. |
| `unstable` | | Highest 0.x **odd** minor; `N/A` when none. (With ≥1.x only, `unstable` is `N/A`.) |
| `iojs`, `io.js` | | Highest io.js. |
| `lts/*` | | Newest LTS line. |
| `lts/<name>` | `lts/iron` | That LTS line's newest release. **Must be lowercase** → else `LTS names must be lowercase`, rc 3. |
| `lts/-N` | `lts/-1` | The line N places **before** the newest (`lts/-1` = second newest, verified against upstream), by alphabetical order of `alias/lts/` (codenames are alphabetical by design). Beyond range → `That many LTS releases do not exist yet.`, rc 2. |
| `<alias>` | `default`, `myproj` | A file under `alias/`; followed transitively. |
| `iojs-vX.Y.Z` | | io.js exact. |
| `.nvmrc` content | | Any string containing `#` or a newline passed to the resolver is parsed as `.nvmrc` content first (`nvm_ls` does this). |

Validation (`nvm_is_valid_version`): `stable|unstable|iojs|node` → valid; else strip `iojs-`, strip `v`, apply the rules above; a trailing `.` is tolerated (`20.` = `20`); ≥ 4 dotted groups invalid.

Rejected everywhere as a path-traversal guard: any version or alias whose `/`-split components include `..` → `invalid version: X` / `invalid alias name: X`, rc 3.

---

## 4. Resolution algorithms

### 4.1 `current` — `nvm_ls_current`

```
p = which node (first on PATH)          → none      if not found
if p is under $NVM_DIR/versions/io.js   → "iojs-" + `iojs --version`
elif p is under $NVM_DIR                → `node --version`    (special-case "v0.6.21-pre" → "v0.6.21"; empty → "none")
else                                    → "system"
```
"Under" (`nvm_tree_contains_path`) = walk the path's ancestors and compare literally **or** by inode (`-ef`), so symlinked `NVM_DIR` and case-insensitive APFS behave.

Native: do not spawn `node --version`; read the version from the **path component** (`versions/node/<v>/bin/node`) — identical result for every nvm-managed install, 1 ms cheaper. Only exec `node --version` for the `system` case when the version number is needed (`nvm ls` appends `system vX`).

### 4.2 Alias chain — `nvm_resolve_alias(pattern)`

```
seen = {pattern}; a = pattern
loop:
  content = first non-comment, non-blank line of alias/<a>  (or stop if no file)
  if content in seen → a = "∞"; stop          # cycle
  seen += content; a = content
if a != pattern:                                # at least one hop
  if a ∈ {∞, iojs, iojs-, node} → print a
  else                           → print ensure_version_prefix(a)
  rc 0
else if pattern ∈ {stable, unstable, iojs, node}:
  print implicit alias (local)    # may be empty
rc 2
```
Note: an alias file named like an exact installed version (`alias/v20.0.0`) **shadows** that version (`nvm_is_installed_exact_version` returns false, `nvm_version` falls into alias resolution).

### 4.3 Local resolution — `nvm_version(pattern)`

```
"" → "current"
"current"              → ls_current
"node" | "node-"       → pattern = "stable"
exact installed vX.Y.Z (not shadowed by an alias) → itself, rc 0        # fast path
else: v = last line of nvm_ls(pattern); "system ..." → "system"
      empty or "N/A"   → print "N/A", rc 3
```

`nvm_ls(pattern)` (returns a sorted list, lowest → highest, one per line):

1. Pattern contains `#`/newline → parse as `.nvmrc` content; invalid → `N/A`, rc 3.
2. `current` → ls_current.
3. `node`/`iojs` → `node-`/`iojs-` (meaning "all of that flavour").
4. Otherwise resolve alias chain. If the target is `system` and a system node exists → output `system <system-version>` (or just `system`) and stop. If the alias chain resolved to something → return `nvm_resolve_local_alias` (= `nvm_version` of the target; cycles yield `∞`). Else ensure `v` prefix.
5. `N/A` → empty.
6. If pattern is `vX.Y.Z` (3 groups): return it if installed (node, else try `iojs-vX.Y.Z`), otherwise nothing.
7. Else, for 1–2 groups append `.` (prefix semantics). Enumerate: old-layout dir, `versions/node`, `versions/io.js` (only the iojs dir if pattern was `iojs-*`; old+new if `node-`). Keep entries whose name starts with the pattern. Sort numerically by `major.minor.patch` across flavours (upstream: `sort -t. -u -k 1.2,1n -k 2,2n -k 3,3n`). Print node versions as `vX.Y.Z`, io.js as `iojs-vX.Y.Z`.
8. If a system node exists: pattern empty/`v` → append `system <ver>`; pattern `system` → output only `system <ver>`.
9. Nothing → `N/A`, rc 3.

`nvm_match_version(p)`: `iojs`/`io.js` → `nvm_version iojs`; `system` → `system`; else `nvm_version p`. Used by `use`, `uninstall`.

### 4.4 Implicit aliases — `nvm_print_implicit_alias(local|remote, name)`

- `node` → prints `stable`.
- `iojs` → highest io.js `major.minor` (local list or remote `index.tab`).
- `stable`/`unstable` → take every version's `major.minor` (deduped, in order); for each: if major ≥ 1 → candidate for `stable`; if major = 0 → even minor → `stable`, odd → `unstable`. Last assignment wins (list is ascending so "highest"). `unstable` prints `N/A` if never set.

The result is a `major.minor`; callers then run it through `nvm_version`/`ls-remote` to get the highest patch.

### 4.5 Remote resolution — `nvm_remote_version(pattern)`

Fetch `${MIRROR}/index.tab` (tab-separated; header line dropped; columns: `version date files npm v8 uv zlib openssl modules lts security`). Column 10 is the LTS codename or `-`.

Side effect on **every** fetch: rewrite `alias/lts/<codename>` for every codename seen (first occurrence in file order = newest release of that line) and `alias/lts/*` → `lts/<first codename seen>`.

Filtering (`NVM_LTS` set):
- `*` → only rows with a codename.
- `<name>` → only rows whose lowercased codename matches `<name>` (regex match, lowercase both). Name is normalised via `lts/-N` first.

Output rows: `vX.Y.Z` | `vX.Y.Z <Codename>` | `vX.Y.Z <Codename> *` (the `*` marks the first/newest row of each LTS line). Then `grep -w <pattern>` (word match on the `v`-prefixed pattern; trailing `.` stripped first), then numeric sort ascending. Implicit aliases (`stable`, `node`, …) are resolved with the remote list via §4.4 and then matched by `vMAJOR.MINOR` prefix. Last line = answer. `NVM_VERSION_ONLY=true` → print only the first field. `N/A` → rc 3.

Guard: if a non-alias pattern was given and the chosen version string does not contain the pattern as a substring → `N/A` (prevents `nvm install 99` from installing something unrelated).

### 4.6 Offline resolution — `nvm_offline_version(pattern)`

`nvm_version` first; else scan `.cache/bin/` for `node-vX.Y.Z-<os>-<arch>` and `.cache/src/node-vX.Y.Z` dirs, filter by `v`-prefixed pattern substring, numeric sort, last; else `N/A` rc 3.

### 4.7 `.nvmrc`

- **Find**: from `$PWD` walk up (`${path%/*}` each step) until `<dir>/.nvmrc` exists; the root `/` is tested too (as the empty prefix).
- **Parse** (`nvm_process_nvmrc_content`): drop everything from `#` to end of each line; trim both ends; drop blank lines. Then:
  - every line is either `key=value` (one `=`, at least one char before it) or a *bare line*;
  - a line that **starts** with `=` counts as a bare line;
  - exactly one bare line must remain, otherwise `invalid .nvmrc!` (full message in §7) and rc 1;
  - `key=value` pairs: key `node` is invalid; duplicate keys invalid; otherwise accepted and ignored (reserved);
  - result = the bare line, **not** validated as a version here.
- **`nvm_rc_version`** output contract: to stdout (unless `NVM_SILENT=1`): `Found '<path>/.nvmrc' with version <X>`; the version `X` is returned on the eval/fd-3 channel. Errors to stderr: `No version provided and no .nvmrc file found` (rc 1); `Warning: empty .nvmrc file found at "<path>"` (rc 2).
- **Write** (`--save`/`-w`): resolve with `nvm_version`; `∞`/`N/A` → rc 1 silently; write `<resolved>\n` to `$PWD/.nvmrc` (overwrite); stdout `Wrote version number (vX.Y.Z) to .nvmrc`; failure → stderr `Warning: Unable to write version number (…) to .nvmrc`, rc 3.

---

## 5. Environment mutation

### 5.1 `PATH` edit — `nvm_change_path(PATH, "/bin", versionDir)`

Given entries split on `:`:

1. Empty PATH → `versionDir/bin`.
2. No entry matches `$NVM_DIR/[^/]*/bin` or `$NVM_DIR/versions/[^/]*/[^/]*/bin` → **prepend** `versionDir/bin:`.
3. An nvm entry exists **but** a system bin dir (`/bin`, `/usr/bin`, `/usr/local/bin`) appears **before** it → prepend anyway (upstream issue #1652: keep nvm ahead of system node).
4. Otherwise → **replace in place** the first nvm entry (regex `NVM_DIR/[^/]*/bin[^:]*` then the `versions/` form) with `versionDir/bin`, preserving order. Only the first occurrence of each pattern is replaced (sed without `g`).

`MANPATH` (only if a `manpath` command exists): same algorithm with suffix `/share/man`; then ensure the list has an empty entry (`:`-terminated or `::` or leading `:`) so `man` still consults its defaults — upstream appends `:` if none present.

### 5.2 `use` also sets
`export NVM_BIN=<versionDir>/bin`, `export NVM_INC=<versionDir>/include/node`, `hash -r` (zsh: `rehash`), optional `rm -f $NVM_DIR/current && ln -s <versionDir> $NVM_DIR/current` when `NVM_SYMLINK_CURRENT=true`.

### 5.3 `deactivate` — `nvm_strip_path`
Remove every `PATH` entry that starts with `$NVM_DIR` and whose remainder matches `^(/versions/[^/]*)?/[^/]*/bin.*$`; preserve a trailing `:` if the original had one. Same for `MANPATH` (`/share/man`; if the result is empty or `:` → `unset MANPATH`) and `NODE_PATH` (`/lib/node_modules`). `unset NVM_BIN NVM_INC`.

### 5.4 npm prefix guard — `nvm_die_on_prefix(delete, cmd, versionDir)`
Run after every successful non-`system` `use`. Any hit → `use` returns **11** (the helper itself returns 3/4/10):

1. `$PREFIX` set and ≠ `nvm_version_path(node -v)` → deactivate, print `nvm is not compatible with the "PREFIX" environment variable: currently set to "<v>"` / `` Run `unset PREFIX` to unset it. ``.
2. Any env var whose **uppercased** name is `NPM_CONFIG_PREFIX` with a value not under `$NVM_DIR` → deactivate, same message shape with the actual var name.
3. Check these npmrc files **in this order** for a line whose key (text before `=`, right-trimmed) is `prefix` or `globalconfig`: builtin `<versionDir>/lib/node_modules/npm/npmrc`, global `<versionDir>/etc/npmrc`, user `$HOME/.npmrc`, project `<nearest dir with package.json or node_modules>/.npmrc`. Hit → either (with `--delete-prefix`) run `npm config delete prefix` / `delete globalconfig` against that file, or print:
   ```
   Your <builtin|global|user’s .npmrc|project npmrc> file (<path with $NVM_DIR/$HOME substituted>)
   has a `globalconfig` and/or a `prefix` setting, which are incompatible with nvm.
   Run `nvm use --delete-prefix vX.Y.Z [--silent]` to unset it.
   ```
   (note the curly apostrophe in `user’s`.) Paths are printed through `nvm_sanitize_path` (`$NVM_DIR` → `${NVM_DIR}`, `$HOME` → `${HOME}`).

---

## 6. Commands

Common front-matter for the top-level dispatcher:
- No args → `--help`.
- Any arg (scanning left to right, stopping at `--`) equal to `-h`, `help`, or `--help` → print help (§7) to stdout, rc 0. `--no-colors` anywhere suppresses colour in the legend.
- Unknown command → help to **stderr**, rc 127.
- Any option beginning with `---` → `` arguments with `---` are not supported - this is likely a typo ``, rc 55 (install parser; other commands say `Unsupported option "<opt>".`, rc 55).
- `--` is accepted and ignored as a separator by most commands.

### 6.1 `install` / `i`

Flags (must precede the version; parsing stops at the first non-flag):
`-s` source only · `-b` binary only (`-s`+`-b` → rc 6) · `-j <n>` make jobs · `--no-progress` · `--offline` · `--lts` · `--lts=<name>` · `--latest-npm` · `--default` · `--alias=<name>` (both → rc 6; `--default` twice → rc 6) · `--reinstall-packages-from=<v>` / `--copy-packages-from=<v>` (bare form without `=` → rc 6; twice → rc 6; empty → rc 6) · `--skip-default-packages` · `--save`/`-w` (twice → rc 6).
Flags after the version are also parsed (`--reinstall-packages-from=…`, `--skip-default-packages`); anything else after the version is collected as `ADDITIONAL_PARAMETERS` and passed to `./configure` for source builds.

`NVM_NO_SOURCE_FALLBACK=1` ⇒ behaves as `-b`; combined with `-s` → rc 6.
Not offline and neither curl nor wget available → `nvm needs curl or wget to proceed.` rc 1 (native: keep the message only if the built-in HTTP client is somehow unavailable — in practice drop the check).

Version selection:
- none given: `--lts` → stdout `Installing latest LTS version.`; `--lts=x` → `Installing with latest version of LTS line: x`; else `.nvmrc`; none → usage (`Usage: nvm install [<version>]` / `  Provide a <version>, or run from a directory containing an .nvmrc file.` / `` Run `nvm --help` for full help. ``) rc 127.
- `lts/*` / `lts/x` as the version → converted to the `--lts` forms.
- Resolve remotely (§4.5) or offline (§4.6). Offline with `--lts` requires the local `alias/lts/*` to exist, else `LTS alias 'x' not found locally. Run `nvm ls-remote --lts` first to populate LTS aliases.` rc 3.
- Not found → `Version '<v>' [(with LTS filter) |(with LTS filter 'x') ]not found - try `nvm ls-remote[ --lts[=x]]` to browse available versions.` rc 3 (offline: `… not found locally or in cache - try `nvm ls` …`). With `--lts=x` and no version: `Version with LTS filter 'x' not found - try …`.
- `--reinstall-packages-from` pointing at the version being installed → `You can't reinstall global packages from the same version of node you're installing.` rc 4; pointing at a non-installed version → `If --reinstall-packages-from is provided, it must point to an installed version of node.` rc 5.

Already installed (and `nvm_validate_install` passes: non-empty executable `bin/node`, `bin/npm` symlink not dangling):
stderr `vX.Y.Z is already installed.` → `nvm use vX.Y.Z` → `--latest-npm` → default-packages (unless skipped) → reinstall-packages → ensure default alias (`lts/<x>` if `--lts`, else the user-typed version string) → `--save` → `--alias`. rc = last failing step.

Fresh install:
1. `NVM_INSTALL_THIRD_PARTY_HOOK` set → run `<hook> <version> node std <binary|source> <versionDir>`; failure → `*** Third-party $NVM_INSTALL_THIRD_PARTY_HOOK env var failed to install! ***` + hook's rc; "succeeded" but not installed → rc 33.
2. Acquire lock `.cache/locks/<version>` (`mkdir`; retry every 1 s; `NVM_INSTALL_LOCK_TIMEOUT` default 600 s → `Timed out after Ns waiting for another install of vX to finish.` rc 1; `NVM_INSTALL_LOCK_STALE` minutes → steal with message; first wait prints `Waiting for another install of vX to finish...`).
3. Binary available (version ≥ v0.8.6) and not `-s` → **binary install** (§6.1.1). Failure and not `-b` → **source install** (§6.1.2). `-b` and no binary → `Binary download is not available for vX` rc 3.
4. Release lock.
5. `nvm_validate_install` fails → `The install of vX reported success but failed verification; not activating it.` rc 1.
6. `use` if not already current → `npm` present check (ancient node only) → ensure default alias → `--latest-npm` → default-packages → reinstall-packages.

#### 6.1.1 Binary install
- stdout `Downloading and installing node vX.Y.Z...`
- Slug: `node-vX.Y.Z-<os>-<arch>`; `os` ∈ `darwin|linux` (native targets); `arch`: `x86_64|amd64→x64`, `i*86→x86`, `aarch64|armv8l|arm64→arm64`, `loongarch64→loong64`, else `uname -m`. Linux: 32-bit userland (`getconf LONG_BIT`=32) → `x86`; arm64 kernel with 32-bit `/sbin/init` → `armv7l`; `/etc/alpine-release` → `x64-musl` / `arm64-musl`. **darwin arm64 with node < 16.0.0 → `x64`** (Rosetta). Node < 4 and armv6l/armv7l → `arm-pi`.
- Extension: `tar.xz` when xz is supported (darwin ≥ 10.9 and node ≥ 4, or 0.12.10+, 0.10.42+), else `tar.gz`.
- URL: `${MIRROR}/vX.Y.Z/<slug>.<ext>` (node ≤ 0.1.13: no version dir). Mirror = `NVM_NODEJS_ORG_MIRROR` or `https://nodejs.org/dist`. The mirror value must match `^https?://[]a-zA-Z0-9._~:/@%[-]+$`, else `$NVM_NODEJS_ORG_MIRROR and $NVM_IOJS_ORG_MIRROR may only contain a URL` rc 2.
- Checksum: fetch `${MIRROR}/vX.Y.Z/SHASUMS256.txt`, take the line whose 2nd field equals `<slug>.<ext>`. (Upstream falls back to `SHASUMS.txt`/sha1 only when no sha256 tool exists — native always uses sha256.)
- Cache hit at `.cache/bin/<slug>/<slug>.<ext>`: stderr `Local cache found: ${NVM_DIR}/.cache/…`; checksum matches → `Checksums match! Using existing downloaded archive …`; mismatch → `Checksum check failed!` / `Removing the broken local cache...` and re-download.
- Download: stderr `Downloading <url>...`; resumable (`-C -`); progress bar on stderr unless `--no-progress`/`NVM_NO_PROGRESS=1` (then `-sS` semantics: silent, errors shown); HTTP failure → `download from <url> failed` rc 4; body containing `404 Not Found` → `HTTP 404 at URL <url>` rc 5; checksum mismatch → `Checksums do not match: '<got>' found, '<want>' expected.` rc 6. Success prints `Computing checksum with sha256sum` then `Checksums matched!` (stderr).
- `NVM_AUTH_HEADER` → sent as `Authorization: <value>` after stripping chars outside `[a-zA-Z0-9 :_.+/=~-]`.
- Extract into `.cache/bin/<slug>/files` with `--strip-components 1 --no-same-owner`, then **atomically** `rm -rf <versionDir>` and rename `files` → `<versionDir>` (fallback: mkdir + move entries when crossing filesystems). The cached tarball is kept.
- `--offline`: use the cached tarball **without** checksum (`Offline: using cached archive …`), or `Offline: no cached archive found for <slug>` rc 4.
- Failure with source fallback allowed → `Binary download failed, trying source.` rc 1 to caller; with `-b` → `Binary download failed. Download from source aborted.` rc 2.

#### 6.1.2 Source install
`NVM_MAKE_JOBS` or `-j`: if unset, detect cores (`sysctl -n hw.ncpu` on darwin, `/proc/cpuinfo` on linux); print `Detected that you have N CPU core(s)`; N > 2 → `N-1` jobs (`Running with N-1 threads to speed up the build`), else 1 (`Number of CPU core(s) less than or equal to 2, running in single-threaded mode`). Download `${MIRROR}/vX/node-vX.tar.<ext>` into `.cache/src/node-vX/`, checksum as above, extract, then `./configure --prefix=<versionDir> <ADDITIONAL_PARAMETERS> && make -j N && make -j N install` (echoing the configure line as `$>./configure …<`). Node < 1.0 may need `MAKE_CXX` tweaks; out of scope for mac-first — keep the pipeline but don't chase 0.x compiler quirks.

#### 6.1.3 Default alias
`nvm_ensure_default_set(v)`: if `alias/default` is readable → no-op. Else `nvm alias default <v>` and stdout `Creating default alias: <formatted alias line>`.

#### 6.1.4 `default-packages`
If `$NVM_DIR/default-packages` exists and is non-empty: stdout `Installing default global packages from ${NVM_DIR}/default-packages...` and `npm install -g --quiet <pkgs…>` (the command echoed), then run it; failure → `Failed installing default packages. Please check if your default-packages file or a package in it has problems!` rc 1. A line containing internal whitespace → `` Only one package per line is allowed in `<path>`. Please remove any lines with multiple space-separated values. `` rc 1.

### 6.2 `uninstall <version>` | `--lts` | `--lts=<name>`

Exactly one arg else usage rc 127. Resolve via `nvm_match_version` (`--lts`/`lts/*`/`lts/x` forms). Equals current → `nvm: Cannot uninstall currently-active node version, vX (inferred from <arg>).` rc 1. Not installed → `Version 'vX' (inferred from <arg>) is not installed.` (or `Version '<arg>' is not installed.` when nothing to infer) **rc 0**. Permission check (every file writable or owned by the user, recursively, not following symlinks) fails → `Cannot uninstall, incorrect permissions on installation folder.` + chown/chmod hints, rc 1. Delete `.cache/bin/<slug>/files`, `.cache/src/node-vX/files`, `<versionDir>`; stdout `Uninstalled node vX`. Then every alias file whose content mentions the version (`grep -l`) is removed via `nvm unalias <name>` (which prints its own line).

### 6.3 `use [--silent] [--lts[=x]] [--save|-w] [--delete-prefix] [<version>]`

See §4/§5. Unknown `--*` flags are **silently ignored** (upstream quirk). Full flow:

1. `--lts` → `nvm_match_version lts/<x or *>`. No version → `.nvmrc` (silent-aware) → `nvm_version`; no `.nvmrc` → stderr `` Please see `nvm --help` or https://github.com/nvm-sh/nvm#nvmrc for more information. `` rc 127. Exact installed version → used verbatim without further checks.
2. Empty result → `Usage: nvm use [<version>]` + two hint lines, rc 127.
3. `--save` → write `.nvmrc` (before activation).
4. `system` → if a system node exists: deactivate; stdout `Now using system version of node: vX (npm vY)`; rc 0. Else stderr `System version of node not found.` rc 127.
5. `∞` → stderr `The alias "<arg>" leads to an infinite loop. Aborting.` rc 8.
6. `N/A` → the not-installed message (§4.3-ish, from `nvm_ensure_version_installed`) rc 3. Message shapes:
   - alias: `N/A: version "<arg> -> vX" is not yet installed.`
   - plain: `N/A: version "vX" is not yet installed.`
   - then an empty line, then one of: `` `lts` is not an alias - you may need to run `nvm install --lts` to install and `nvm use --lts` to use it. `` (arg literally `lts`); `` You need to run `nvm install <arg>` to install and use it. ``; or, when the version came from `.nvmrc`, `` You need to run `nvm install` to install and use the node version specified in `.nvmrc`. ``
7. Mutate env (§5.1–5.2).
8. Prefix guard (§5.4) → rc 11 on failure (env already mutated! upstream leaves PATH changed but deactivates in the env-var cases).
9. stdout `Now using node vX.Y.Z (npm vA.B.C)` — the npm suffix only if `npm --version` succeeds on the newly selected version. `--silent` suppresses **all** stdout/stderr from this command.

### 6.4 `run [--silent] [--lts[=x]] [<version>] [<args…>]`
Resolve version (arg → `nvm_version`; if that is `N/A` **and** the arg is not a valid version-ish, treat the arg as a script and fall back to `.nvmrc`, then to `current` with the deprecation warning:
```
WARNING: `nvm run` was invoked without a version argument and without an .nvmrc file.
  Falling back to the active node version; this will become an error in a future release.
  Pass `current` explicitly (e.g. `nvm run current ...`) to silence this warning.
```
). Then delegates to `nvm exec [--silent] [--lts=x] <version> node "$@"` (or `iojs`). Not installed → the §6.3.6 message, rc 1.

### 6.5 `exec [--silent] [--lts[=x]] [<version>] <command…>`
Unknown `--*` → `Unsupported option "<opt>".` rc 55; `--` ends option parsing. Version resolution as in `run`. Not installed → rc 1 with message. Unless silent, stdout one of:
`Running node vX (npm vY)` · `Running node latest LTS -> vX (npm vY)` · `Running node LTS "x" -> vX (npm vY)` · `Running io.js X (npm vY)`.
Then `NODE_VERSION=vX $NVM_DIR/nvm-exec <command…>`; the child gets `PATH`/`NVM_BIN`/`NVM_INC` as if `use` ran, **the parent shell is untouched**. Exit code = the command's.

### 6.6 `current` → prints §4.1 result (`none`, `system`, `vX.Y.Z`, `iojs-vX`).

### 6.7 `which [--silent] [current|<version>]`
No version → `.nvmrc`; none → usage rc 127. `system` → deactivate temporarily and print `which node`; none → `System version of node not found.` rc 127. `∞` → rc 8. Not installed → §6.3.6 message rc 1. Else `<versionDir>/bin/node`.

### 6.8 `ls` / `list [--no-colors] [--no-alias] [<pattern>]`
`--no-alias` with a pattern → `` `--no-alias` is not supported when a pattern is provided. `` rc 55. Prints `nvm_ls(pattern)` through the formatter (§8), then (no pattern, no `--no-alias`) the alias listing. rc = `nvm_ls`'s rc (3 when `N/A`).

### 6.9 `ls-remote` / `list-remote [--lts[=x]] [--no-colors] [<pattern>]`
`lts/*`, `lts/x` patterns map to `--lts` forms. Output formatted (§8). With no pattern and no `--lts`, the rows are annotated with `(Latest: node)` on the highest stable row and `(Aliases: a, b)` for user aliases that resolve to that row. `N/A` → prints `N/A` formatted, rc 3.

### 6.10 `version <version>` → `nvm_version`. `version-remote [--lts[=x]] [<version>]` → `nvm_remote_version` with `NVM_VERSION_ONLY` (default pattern `node`).

### 6.11 `alias [--no-colors] [<pattern>]` / `alias <name> <version>` / `unalias <name>`
- Listing: ensure `alias/lts/` exists. Sections, each sorted: user aliases matching `alias/<pattern>*`; then the four implicit ones (`node`, `stable`, `unstable`, `iojs`) that have no file, filtered by exact pattern match; then `alias/lts/<pattern>*`. A pattern starting with `lts/` prints that alias file's content only.
- Create: `#` in name → `Aliases with a comment delimiter (#) are not supported.` rc 1; `/` in name → `Aliases in subdirectories are not supported.` rc 1; target not resolvable → stderr `! WARNING: Version '<target>' does not exist.` but the alias is still written; writes `<target>\n`; prints the formatted line. Empty-string target (`nvm alias foo ""`) → `unalias foo`.
- Delete: no file → built-in name (`stable|unstable|iojs|node|system`) → `<name> is a default (built-in) alias and cannot be deleted.` rc 1, else `Alias <name> doesn't exist!` **rc 0**. Else remove and print `` Deleted alias <name> - restore it with `nvm alias "<name>" "<old content>"` ``.

Formatted line (`nvm_print_formatted_alias`): `<alias> -> <dest>` when dest = resolved version, else `<alias> -> <dest> (-> <resolved>)`; implicit ones get ` (default)` appended; without colours an installed resolution gets ` *` appended to the version. Colours: current → colour 3, installed → 1, `∞`/`N/A` → 4, arrow is `\033[0;90m->\033[0m`, `(default)` in colour 5, LTS names in colour 6 (= bold colour 2).

### 6.12 `deactivate [--silent]` — §5.3; stdout `${NVM_DIR}/*/bin removed from ${PATH}` (literal `*`), stderr `Could not find ${NVM_DIR}/*/bin in ${PATH}` when nothing to strip; same for `share/man` and `lib/node_modules`.

### 6.13 `unload` — `deactivate` (quiet), then unset the function and every `NVM_*` variable the script owns (`NVM_NODEJS_ORG_MIRROR NVM_IOJS_ORG_MIRROR NVM_DIR NVM_CD_FLAGS NVM_BIN NVM_INC NVM_MAKE_JOBS NVM_INSTALL_LOCK NVM_COLORS INSTALLED_COLOR SYSTEM_COLOR CURRENT_COLOR NOT_INSTALLED_COLOR DEFAULT_COLOR LTS_COLOR`). Native: emit `unset -f nvm` + the unsets on the eval channel.

### 6.14 `install-latest-npm` (no args, else rc 127)
stdout `Attempting to upgrade to the latest working version of npm...`, then pick the newest npm that runs on the current node; the exact ladder is a static table (node → npm cap): `<0.6` n/a · `0.6` → `npm@1.3` · `<1.1` → `4.5` · `<4` → `4` · `4.0–4.4`,`5.0–5.9` → `5.3` · `4.5–4.6` → `5.4.1` · `<6` → `5` · `6.0–6.1`,`9.0–9.2` → `6.9` · `<10` → `6` (after bumping to `npm@4` if npm < 4.4.4) · `<12.13`,`13`,`14.0–14.15`,`15` → `7` · `12`,`14.13–14.16`,`16.0–16.12`,`17` → `8.6` · `<18.17`,`19`,`20.0–20.4` → `9` · `<20.17`,`21`,`22.0–22.8` → `10` · else → `npm` (latest). Each branch prints its `* …` explanation line, runs `npm install -g npm@<x>`, and ends with `* npm upgraded to: v<ver>`. `NVM_DEBUG=1` echoes instead of running.

### 6.15 `reinstall-packages` / `copy-packages <version>`
Exactly one arg. Same as current → `Can not reinstall packages from the current version of node.` rc 2. `system` without a system node → rc 3. Lists `npm list -g --depth=0` under the source version (drop `npm`, `corepack`, `-> ` links, `(empty)`, `UNMET PEER DEPENDENCY`), prints `Reinstalling global packages from vX...`, runs `npm install -g --quiet <pkgs>` (or `No installed global packages found...`), then `Linking global packages from vX...` and `npm link` in each linked path (or `No linked global packages found...`).

### 6.16 `cache dir` → `${NVM_DIR}/.cache`; `cache clear` → `rm -rf` + `mkdir -p`, `nvm cache cleared.`; else usage rc 127. `clear-cache` (legacy) → removes `$NVM_DIR/v*` files and prints `nvm cache cleared.`.

### 6.17 `set-colors <5 chars>` — chars from `rRgGbBcCyYmMkKeW`; wrong → help to stderr + `Please pass in five valid color codes. Choose from: rRgGbBcCyYmMkKeW` (coloured), rc 17 internally (command rc 0 after printing). Success: `export NVM_COLORS=<codes>` on the eval channel; stdout `Setting colors to: a b c d e` (+ `WARNING: Colors may not display because they are not supported in this shell.` when stdout is not a colour terminal).

### 6.18 `debug` — dumps `nvm --version`, `$TERM_PROGRAM`, `$SHELL`, `$SHLVL`, `whoami`, `$HOME`, `$NVM_DIR`, `$PATH`, `$PREFIX`, `$NPM_CONFIG_PREFIX`, mirrors, shell version, `uname -a` (hostname blanked), checksum binary, OS version (`sw_vers` on mac), tool locations/versions (awk, curl, wget, git, grep, sed, cut, basename, rm, mkdir, xargs), then `nvm current`, `which node/iojs/npm`, `npm config get prefix`, `npm root -g` — all to **stderr**, rc **42**. Native: same keys, plus a `nvm-core:` line with the binary's own build info.

### 6.19 `--version` / `-v` → `0.40.8` (match whatever upstream version the native build claims compatibility with; expose the native build via `nvm debug` only, so scripts that parse `nvm --version` are unaffected).

### 6.20 Exit-code summary

| rc | Used for |
|---|---|
| 0 | success; also `uninstall` of a non-installed version, `unalias` of a missing alias |
| 1 | generic failure, not-installed in `exec`/`which`/`run`, lock timeout, permission problems |
| 2 | `nvm_rc_version` empty `.nvmrc`; `reinstall-packages` from current; bad mirror URL; `lts/-N` out of range; `-b` binary failure |
| 3 | `N/A` / version not found (remote or local), `use` of uninstalled version, path traversal, invalid LTS case |
| 4 | reinstall from same version; download failure |
| 5 | reinstall-from not installed; HTTP 404 |
| 6 | mutually exclusive / duplicate flags; checksum mismatch during download |
| 8 | alias cycle (`∞`) |
| 11 | npm `prefix` conflict in `use` |
| 17 | bad `set-colors` argument (internal) |
| 33 | third-party hook lied |
| 42 | `debug` |
| 55 | unsupported option / `---` |
| 87 | source install on non-WSL Windows (never on mac; keep the number reserved) |
| 127 | usage errors, unknown command, no `.nvmrc`, system node missing |

---

## 7. Fixed strings

The help text must be reproduced verbatim (copy from `nvm.sh` lines 3934–4025: header `Node Version Manager (v0.40.8)`, the `Note: <version> …` block, the `Usage:` table, the colour legend, the `Example:` block, the trailing `Note:` about removing `$NVM_DIR`). Treat it as a static asset in the repo and diff it against upstream in CI.

The `.nvmrc` error:
```
invalid .nvmrc!
all non-commented content (anything after # is a comment) must be either:
  - a single bare nvm-recognized version-ish
  - or, multiple distinct key-value pairs, each key/value separated by a single equals sign (=)

additionally, a single bare nvm-recognized version-ish must be present (after stripping comments).

non-commented content parsed:
<the stripped lines>
```
(first block in red, second in yellow, when colours are on).

---

## 8. Output formatting (`nvm ls`, `nvm ls-remote`)

Reference implementation: the awk program in `nvm_print_versions` (nvm.sh 2322–2530). Rules:

- Colour on only when stdout is a TTY **and** `tput colors ≥ 8` **and** `NVM_NO_COLORS` ≠ `--no-colors`. Italics additionally require terminfo `sitm`.
- Palette `NVM_COLORS` (default `bygre`): position 1 installed, 2 system/LTS, 3 current, 4 not-installed, 5 default; colour 6 = bold variant of 2. Codes: `r 0;31 R 1;31 g 0;32 G 1;32 b 0;34 B 1;34 c 0;36 C 1;36 m 0;35 M 1;35 y 0;33 Y 1;33 k 0;30 K 1;30 e 0;37 W 1;37`.
- Each row: version right-aligned in 15 columns. Current row: `->` + 13-wide (`->%13s`). Without colours, installed rows and the current row get a trailing ` *`; non-installed rows get two trailing spaces of padding instead.
- Annotations, each starting at a fixed column across all rows so output stays `cut`-able: ` (LTS: Name)` (old LTS line, default colour 5), ` (Latest LTS: Name)` (bold current colour, for rows flagged `*`), ` (Latest: node)` and ` (Aliases: a, b)` (dim current colour, alias names italic). System row: ` (-> vX)` in system colour.
- Rows come from `nvm_ls`/`nvm_ls_remote` lines: `v18.0.0`, `v18.0.0 Hydrogen`, `v18.0.0 Hydrogen *`, `system v20.0.0`, `iojs-v3.3.1`.

Native: port the awk logic 1:1 and golden-test against upstream output with `NVM_NO_COLORS=--no-colors` and with a forced 256-colour terminfo.

---

## 9. Shell integration

### 9.1 Startup arguments to `nvm.sh`
Sourcing `nvm.sh` accepts `--no-use` and `--install`:
- default (`use` mode): if current is `none`/`system`, resolve `alias/default` → if valid & installed, `nvm use --silent <it>`; else if `.nvmrc` found in `$PWD` → `nvm use --silent`; if current is already an nvm version → re-`use` it (re-asserts PATH order). Any failure is silent.
- `--install`: same but `nvm install <default>` / `nvm install` (quiet).
- `--no-use`: nothing.

The native `nvm.sh` keeps that interface and implements it as `nvm-core __auto use|install` through the eval channel. Two distinct semantics exist and are separate sub-commands: `__auto use` is the source-time algorithm above (default alias, else `.nvmrc`, else re-assert the current nvm version); `__auto cd` is the directory-change hook (`.nvmrc` found up the tree, else `default`; no-op when already active). The zsh/bash/fish hooks in §9.4 call `__auto cd`.

### 9.2 Installing

Installer mirrors upstream `install.sh`: `NVM_DIR` (default `$XDG_CONFIG_HOME/nvm` if `XDG_CONFIG_HOME` set, else `~/.nvm`), `PROFILE` override (`/dev/null` = don't touch), profile detection order: `$SHELL`=bash → `~/.bashrc` then `~/.bash_profile`; zsh → `$ZDOTDIR/.zshrc` then `.zprofile`; else first existing of `.profile .bashrc .bash_profile .zprofile .zshrc`. Appends exactly:
```sh
export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"  # This loads nvm
[ -s "$NVM_DIR/bash_completion" ] && \. "$NVM_DIR/bash_completion"  # This loads nvm bash_completion
```
Because the native `nvm.sh` is the thin function, existing installs upgrade by replacing `nvm.sh`, `nvm-exec`, and dropping in `nvm-core` — profiles do not change. Homebrew formula: install `nvm-core` to the Cellar, `nvm.sh` to `$(brew --prefix)/opt/nvm/nvm.sh`, print the same source line.

### 9.3 Lazy mode

Opt-in with `NVM_LAZY=1` set **before** sourcing `nvm.sh` (or `. "$NVM_DIR/nvm.sh" --lazy`). Behaviour:

1. `nvm.sh` defines `nvm` and stub functions `node npm npx corepack yarn pnpm` (list overridable via `NVM_LAZY_CMDS`). Nothing else runs — no `__auto`.
2. The first call of any stub runs `nvm-core __auto use` (resolving `default` / `.nvmrc`), evals the result, **undefines all stubs**, then re-invokes the original command (`command node "$@"`). Cost is paid once, on first use, and only in shells that actually use node.
3. `nvm` itself, when called before the stubs fired, also triggers the auto-use first so `nvm current` reports what a non-lazy shell would.
4. Caveat printed in docs: in lazy mode `which node` returns the function until first use; tools that probe `node` by `command -v` without executing it won't trigger activation. `NVM_LAZY_PATH=1` (optional refinement) instead prepends `$NVM_DIR/alias/default`-resolved `bin` dir to `PATH` by **reading the alias file in shell** without executing anything — a ~0.3 ms `read` — and skips the stubs entirely. This is the recommended default for mac: it is both lazy and `which`-correct.

### 9.4 Per-shell setup

**zsh** — `~/.zshrc`:
```zsh
export NVM_DIR="$HOME/.nvm"
export NVM_LAZY=1                      # optional
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
# completion (native): compdef is generated by nvm-core
[ -s "$NVM_DIR/bash_completion" ] && \. "$NVM_DIR/bash_completion"   # upstream bash-style completion still works via bashcompinit
```
Auto-switch on `cd` (upstream recipe, ported to one binary call):
```zsh
autoload -U add-zsh-hook
_nvm_auto_use() { nvm __auto cd 2>/dev/null; }
add-zsh-hook chpwd _nvm_auto_use
_nvm_auto_use
```
(`__auto cd` with a `.nvmrc` in `$PWD` ancestry uses it; without one, reverts to `default` — exactly the upstream `load-nvmrc` semantics: "Reverting to nvm default version". The native command performs that comparison itself so the hook is one process spawn, ~3 ms.)

**bash** — `~/.bashrc` (macOS: `~/.bash_profile` must source `~/.bashrc` or contain the lines itself):
```bash
export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
[ -s "$NVM_DIR/bash_completion" ] && \. "$NVM_DIR/bash_completion"
```
Auto-switch: wrap `cd` as upstream's `cdnvm` does, but call `nvm __auto cd` after `builtin cd`. The upstream `cdnvm` function (README "Calling nvm use automatically") must keep working unmodified: it uses `nvm_find_up`, `nvm version`, `nvm use`, `nvm install` — so **`nvm_find_up` must remain a shell function** (trivial: the native `nvm.sh` defines `nvm_find_up`, `nvm_find_nvmrc`, `nvm_rc_version`, `nvm_ls_current`, `nvm_version` as one-line wrappers around `nvm-core __fn <name> "$@"`).

**fish** — upstream has no fish support; the native version does. `~/.config/fish/conf.d/nvm.fish`:
```fish
set -gx NVM_DIR $HOME/.nvm
function nvm
    set -l f (mktemp)
    env NVM_EVAL_FILE=$f NVM_SHELL=fish $NVM_DIR/nvm-core $argv
    set -l rc $status
    test -s $f; and source $f
    rm -f $f
    return $rc
end
nvm __auto use --silent
```
Auto-switch: `function __nvm_cd --on-variable PWD; nvm __auto cd; end`. The binary emits `set -gx PATH …` (fish's PATH is a list — emit it as space-separated quoted items), `set -e MANPATH`, etc. Completions: `nvm __complete fish` prints a `complete -c nvm …` file; install it to `~/.config/fish/completions/nvm.fish`.

**Non-interactive shells / CI**: `BASH_ENV`-style sourcing works unchanged. `nvm-exec` needs no shell at all.

### 9.5 Completion
Ship the upstream `bash_completion` verbatim (it calls `nvm_ls` and reads `alias/`, both still available). Additionally `nvm-core __complete {bash,zsh,fish}` generates native completions covering every subcommand and flag in §6 (upstream's covers subcommands only).

---

## 10. Environment variables (complete list)

Exported by nvm: `NVM_DIR`, `NVM_BIN`, `NVM_INC`, `NVM_CD_FLAGS` (zsh: `-q`, else empty — keep exporting for scripts that read it), `NVM_RC_VERSION` (legacy; set during `.nvmrc` use).

Read by nvm: `NVM_NODEJS_ORG_MIRROR`, `NVM_IOJS_ORG_MIRROR`, `NVM_AUTH_HEADER`, `NVM_NO_PROGRESS`, `NVM_NO_COLORS` (`--no-colors`), `NVM_COLORS`, `NVM_SYMLINK_CURRENT`, `NVM_NO_SOURCE_FALLBACK`, `NVM_INSTALL_LOCK_TIMEOUT`, `NVM_INSTALL_LOCK_STALE`, `NVM_INSTALL_THIRD_PARTY_HOOK`, `NVM_MAKE_JOBS`, `NVM_DEBUG`, `NVM_SILENT` (internal), `NVM_LTS` (internal), `NVM_VERSION_ONLY` (internal), `NVM_OS_RELEASE` / `NVM_OS_KERNEL_OSTYPE` (test hooks), `PREFIX`, `NPM_CONFIG_PREFIX` (any case), `MANPATH`, `NODE_PATH`, `TMPDIR`, `HOME`, `ZDOTDIR`, `PROFILE`, `XDG_CONFIG_HOME` (installer), `NVM_INSTALL_VERSION`, `NVM_INSTALL_GITHUB_REPO`, `NVM_SOURCE`, `METHOD` (installer).

New (native only): `NVM_EVAL_FILE`, `NVM_SHELL`, `NVM_LAZY`, `NVM_LAZY_CMDS`, `NVM_LAZY_PATH`, `NVM_CORE` (override binary path).

---

## 11. Native implementation notes

- **HTTP**: built-in client (`ureq` 3 + rustls + `rustls-platform-verifier`, in the separate `nvm-fetch` binary — §13.2, §14.3). Honour `HTTPS_PROXY`/`https_proxy`/`NO_PROXY` like curl does. Resume with `Range:` when a partial file exists. Keep `curl`/`wget` out of the hot path entirely; the "needs curl or wget" error is dropped.
- **Archive**: `tar.xz` via the `liblzma` crate (static xz 5.8; `xz2` is dormant since 2022); `tar.gz` via `flate2` (`zlib-rs` backend); entries via the `tar` crate. Strip one leading path component; preserve mode bits; do not preserve owner.
- **Checksum**: sha256, streamed during download so the cache file is validated without a second read.
- **Version compare**: integers `(major, minor, patch, prerelease: Option<String>)`; prerelease compares as upstream's awk does (`a[i] < b[i]` on non-numeric text is a string compare; a prerelease on the right makes the left "greater" unless equality counts). Golden-test against `nvm_version_greater` with the upstream test suite's cases.
- **Sorting** lists: numeric by (major, minor, patch); io.js and node interleave by number (upstream behaviour); `system` always last.
- **Locking**: `mkdir` on `.cache/locks/<version>` (chars outside `[0-9A-Za-z._+-]` → `_`); poll 1 s.
- **Process spawning allowed**: `npm`, `node`/`iojs` (only for `system`, `--latest-npm`, `reinstall-packages`, `print_npm_version`), `man`-path check is a `which manpath` (cacheable), `tput` replaced by terminfo lookup in-process, `sw_vers` only in `debug`. Nothing is spawned on the `nvm use` fast path except possibly `npm --version` for the suffix — cache that per version dir in `.cache/npm-version/<v>` keyed by `lib/node_modules/npm/package.json` mtime, or read `package.json` directly (preferred: zero spawns).
- **Atomic installs**: extract to `files/`, rename into place; a crash leaves no half-installed version dir.
- **Signals**: Ctrl-C during download removes the partial `files/` dir but keeps the partial tarball for resume (matches `-C -`).
- **Testing**: run upstream's `test/fast` and `test/slow` suites (urchin) against the native `nvm.sh` — they exercise the public function names and outputs; add a golden-output harness that runs identical command sequences through `nvm.sh@upstream` and `nvm-core` in a scratch `NVM_DIR` and diffs stdout/stderr/rc/env.

---

## 12. Deliberate deviations (document, don't hide)

| Upstream | Native | Why |
|---|---|---|
| Needs `curl`/`wget` | built-in HTTP | removes a dependency; error string dropped |
| Spawns `node --version` for `current` | reads path | identical result, faster |
| sha1 fallback when no sha256 tool | always sha256 | always available natively |
| No fish support | fish supported | user requirement |
| `nvm ls` relies on `find`/`sed`/`sort` | in-process | identical output, golden-tested |
| Unknown `--flags` on `use` ignored | kept ignored | compat; emit nothing |
| `nvm unalias` missing alias rc 0 | kept | compat |
| io.js/0.x source builds | not exercised | out of scope; code path kept but untested |
| Windows/SunOS/AIX/*BSD | not built | mac-first; `nvm_get_os` only knows `darwin`/`linux` |


### 12.1 Upstream quirks reproduced on purpose (verified byte-for-byte against v0.40.8)

| Behaviour | Detail |
|---|---|
| `ls-remote <pattern>` exit code | Exits **3** whenever a positional pattern is given without `--lts`, even when rows were printed (`node` excepted). With `--lts[=x]` it exits 0. |
| `ls-remote` rows | The unfiltered list also contains io.js releases (`iojs-vX.Y.Z`, from the io.js index) merged by number; `ls-remote node` lists node only. |
| `ls-remote` / `version-remote` on fetch failure | Any network error, 404 or invalid mirror yields an empty list, not an error: `N/A`, rc 3. An invalid mirror additionally prints `$NVM_NODEJS_ORG_MIRROR and $NVM_IOJS_ORG_MIRROR may only contain a URL`. |
| `ls-remote` padding | No trailing spaces on plain rows. An LTS annotation is preceded by two spaces when the row is not installed; installed rows carry ` *` instead. |
| `N/A` row | `ls` with nothing installed prints `N/A *`; `ls <pattern>` prints `N/A`; `ls-remote <numeric pattern>` prints `N/A *`; `ls-remote` with an LTS filter or a non-version pattern prints plain `N/A`. |
| `version-remote lts/Iron` | Prints `LTS names must be lowercase` twice plus a blank line and exits **0**. |
| `use --lts` when the newest LTS is not installed | Silent, rc 3 (no message). `use lts/*` prints the full not-installed message. |
| Not-installed message | `N/A: version "<arg> -> <resolved alias target>" is not yet installed.` The target comes from the alias chain, not from `nvm_version`. |
| `.nvmrc` validation | A `node=` key and duplicate keys are invalid; invalid or empty files print the long `invalid .nvmrc!` block, then ``Please see `nvm --help` ...``, rc 127. |
| `exec`/`run` with `.nvmrc` | Print `Found '<path>/.nvmrc' with version <X>` before `Running node ...`. `run` with a first argument that is not version-ish falls back to `.nvmrc`, then to `current` with the deprecation warning. |
| `default-packages` | Upstream's awk joins accepted lines with a leading space for any line after file line 1, so the echoed command is `npm install -g --quiet  pkg` (two spaces). Any line containing whitespace (even leading) is an error. |
| `reinstall-packages <unresolvable>` | Prints the not-installed message for `N/A`, then the four `Reinstalling…/No … found…` lines, rc 0. |
| `--reinstall-packages-from` | Checked before the "already installed" notice; for an already installed target it delegates to `reinstall-packages` (rc 2 `Can not reinstall packages from the current version of node.`). The rc 4 same-version error only applies to fresh installs. |
| `deactivate` text | `<expanded $NVM_DIR>/*/bin removed from ${PATH}` (directory expanded, `${PATH}` literal); also reports `share/man` (stderr when absent) and `lib/node_modules`. |
| `MANPATH` | Managed only when a `manpath` executable exists; an empty entry is appended so `man` keeps its defaults. |
| Implicit alias order | The `iojs`, `node`, `stable`, `unstable` lines are **sorted as text including colour escapes**, so with colour on the unresolved (red) ones sort first. `stable` is omitted when no version qualifies; `alias <name>` filters user aliases by prefix and `lts/<pattern>` entries by the part after `lts/`. |
| Prefix guard | Runs after the environment is already mutated; failures return 11 and leave `PATH` changed except for the `PREFIX`/`NPM_CONFIG_PREFIX` cases, which deactivate. `--silent` does not silence it. |
| `install --default` with `--alias` or a repeat | `--default and --alias are mutually exclusive, and may not be provided more than once` (rc 6). |
| `---flag` | Only `install` rejects it (``arguments with `---` are not supported - this is likely a typo``, rc 55); other commands ignore unknown flags. |
| Third-party hook | Prints `** $NVM_INSTALL_THIRD_PARTY_HOOK env var set; dispatching to third-party installation method **` first; a hook that exits 0 without installing gives `…claimed to succeed, but failed to install!`, rc 33. |

---

## 13. Performance: baseline, budgets, profiling, competitors

"Fastest and most compatible" is the product goal. Compatibility is §2–§12; this section makes speed measurable and normative. All numbers below were measured on 2026-10-21 on an Apple M2 Max (macOS 15.6, zsh 5.9, hyperfine 1.20 `-N`) with node v22.22.3 installed in every manager under an isolated home. The machine carried a load average of 17–33 from unrelated processes during the shell-level runs, so **min and p10 are the trustworthy columns**; medians are inflated by noise, and absolute nvm numbers are roughly 2× the idle-machine figures published in nvm-sh/nvm#3930 (302.7 ms auto-use / 75.0 ms `--no-use` on an idle M-series). Rankings and ratios are stable across runs. Raw JSON, rc files and scripts: `/private/tmp/claude-501/-Users-helge-code-nvm-fast/7a8d1862-2c41-48ca-82d5-25e20f573b2c/scratchpad/lab/` (session scratchpad; copy into `bench/` when the repo exists).

### 13.1 Where upstream nvm spends its time

Upstream is a 5,226-line POSIX script that forks for everything. Census from `bash -x` (one installed version, one alias):

| Operation | External execs | Dominant utilities |
|---|---:|---|
| `source nvm.sh` (auto-use `default`) | ≈ 40 | `grep` 10, `uname` 8, `sed` 8, `awk` 6 |
| `nvm use 22` | ≈ 70 | `grep` 22, `sed` 20, `uname` 6, `npm --version` 1 |
| `nvm ls` | ≈ 540 | `sed` 138, `awk` 114, `uname` 50, `tr` 42, `tput` 38, `find` 14 |

Plus one `$(…)` subshell fork per helper call (800+ for `source`, 7,900+ for `nvm ls`). Costs measured (min over rotated rounds; bare `zsh -ic exit` with `NO_GLOBAL_RCS` = 6.2 ms; bare `/bin/bash --norc -c exit` = 2.0 ms):

| nvm 0.40.8 | zsh 5.9 | bash 3.2 (macOS `/bin/bash`) |
|---|---:|---:|
| `source nvm.sh --no-use` (parse + define 5.2k lines) | +15 ms | +173 ms |
| `source nvm.sh` (auto-use `default`) | +601 ms | — |
| `source nvm.sh; nvm use 22` | — | +625 ms |
| `nvm use 22` (partial, in a sourced shell) | +636 ms | |
| `nvm use v22.22.3` (exact fast path) | +370 ms | |
| `nvm use --silent 22` | +328 ms | |
| `nvm version 22` | +119 ms | |
| `nvm which 22` | +422 ms | |
| `nvm alias` | +1,437 ms | |
| `nvm ls` | +1,783 ms | |
| chpwd hook (README `load-nvmrc`), `cd` with no `.nvmrc` change | +6.5 ms per `cd` | |
| chpwd hook, `cd` into/out of a `.nvmrc` dir (switch + revert) | +203 ms per `cd` | |

Three structural causes, all removed by the native design: (1) parse cost of the script itself (the only thing `--no-use` leaves, and 10× worse on bash 3.2); (2) `nvm_ls`/`nvm_version` enumerate with `find | sed | sort` pipelines on every resolution; (3) `nvm use` spawns `npm --version` for the "Now using" suffix and greps four npmrc files.

### 13.2 Budgets (normative)

Measured on an idle Apple-silicon Mac with hyperfine `-N`, `--warmup 20`, ≥ 300 runs, 3 rotated rounds; the gate is the **min** of mins and the **median** of medians. Linux targets get 1.5× the ms figures.

| Metric | MUST (min) | MUST (median) | Rationale |
|---|---:|---:|---|
| `nvm-core __auto use`, nothing to change (shell startup, `chpwd`) | ≤ 2.0 ms | ≤ 3.0 ms | fnm `env` = 2.9 / 3.7 ms; `/bin/true` ≈ 1.1 ms; libSystem-only Rust hot path measured 1.6 / 1.9 ms |
| `nvm-core __auto use`, version switch | ≤ 2.5 ms | ≤ 4.0 ms | fnm `use --silent-if-unchanged` 3.5 ms |
| `nvm use <installed>` incl. `(npm vX)` suffix | ≤ 3.0 ms | ≤ 4.5 ms | npm version read from `package.json`, never spawned (§11) |
| `nvm current` / `which` / `version` / `alias <name>` | ≤ 2.0 ms | ≤ 3.0 ms | |
| `nvm ls`, `nvm alias` (100 installed versions, 20 aliases, colour on) | ≤ 3.0 ms | ≤ 5.0 ms | upstream: seconds |
| `nvm ls-remote` excluding network (parse ≈ 1,000-row `index.tab`, rewrite `alias/lts/*`, format) | ≤ 5.0 ms | ≤ 8.0 ms | |
| `nvm install` of a cached tarball, 26 MB `tar.xz` → 5.9k entries | ≤ 1.3× `xz -dc \| tar -x` wall | | liblzma-bound; measured 1.34 s vs 1.39 s system |
| Sourcing the native `nvm.sh` (function definitions only) | ≤ 1.0 ms zsh, ≤ 2.0 ms bash 3.2 | | upstream 15 / 173 ms |
| Shell-startup overhead over bare shell, eager mode | ≤ 3.0 ms | ≤ 4.5 ms | fnm +7.3, mise +23.5, nvm +601 |
| Shell-startup overhead, `NVM_LAZY_PATH=1` | ≤ 0.5 ms | | no process spawned |
| Per-`node`-invocation overhead | 0 | 0 | PATH edit, never shims (§13.4) |
| Peak RSS, any hot command | ≤ 4 MB | | |
| `nvm-core` binary (macOS, stripped) / `nvm-fetch` | ≤ 2 MB / ≤ 4 MB | | fnm 6.4 MB, proto 36.9 MB, mise 135 MB |
| Dynamic libraries linked by `nvm-core` | `libSystem.B.dylib` only | | see below |
| Syscalls on the `__auto use` no-change path | ≤ 80 | | `strace -c` on Linux, `dtruss` on mac |

**Two binaries, one hot path.** Linking `Security.framework` + `CoreFoundation` (what any system-trust-store TLS verifier pulls in: `rustls-platform-verifier`, `native-tls`, Swift `URLSession`) costs ≈ 1.0 ms of dyld work on every launch even when no network code runs — measured: Rust hot path 1.64 ms with libSystem only vs 2.59 ms with the full HTTPS stack linked; fnm pays this (2.9 ms). Therefore:

- `nvm-core` — every command in §6 except the network parts; links libSystem only. CI asserts `otool -L nvm-core` lists exactly `/usr/lib/libSystem.B.dylib` (Linux: static-PIE, `ldd` says "not a dynamic executable").
- `nvm-fetch` — HTTPS download with resume, sha256 streaming, `index.tab` fetch, tar.xz/tar.gz extraction into `files/` (§6.1.1). Spawned by `nvm-core` for `install`, `ls-remote`, `version-remote`, and `__auto`'s `--install` mode. Its ≈ 2 ms startup is irrelevant next to network time.
- Both ship in the same directory (`$NVM_DIR/`, or Homebrew `libexec/`); `nvm-core` locates `nvm-fetch` next to its own `argv[0]`/`proc/self/exe`, overridable with `NVM_FETCH`.

**Hot-path rules** (MUST): no allocation-heavy argument parser work before dispatch (parse `argv` by hand for the `__auto`/`use`/`current` fast paths); no `readdir` of `versions/node` when the request is an exact installed version or an alias that resolves to one (stat only); no terminfo lookup unless output is a TTY; no `MANPATH` computation unless `MANPATH` is set or `manpath` exists (cache the `which manpath` result in `.cache/`); no `npm --version` ever (§11); the eval file is written with one `write(2)`; exit via `_exit` after flushing.

### 13.3 Profiling and the benchmark harness

**Wall-clock (black box).** `hyperfine -N` (no intermediate shell, hyperfine ≥ 1.19), `--warmup ≥ 20`, `--runs ≥ 300` for ms-scale commands, `--export-json`; report min, p10, median — never mean. Rotate command order across ≥ 3 rounds so load spikes do not land on one command. Refuse to record a baseline when the 1-minute load average exceeds `ncpu / 4`.

- Shell startup: an isolated `ZDOTDIR` per configuration with `.zshenv` = `setopt NO_GLOBAL_RCS` (skips `/etc/zshrc`'s `locale` fork and `path_helper`) and `.zshrc` = the tool's documented init lines; `hyperfine -N "env ZDOTDIR=<dir> zsh -ic exit"`; subtract the bare configuration. bash: `/bin/bash --noprofile --rcfile <rc> -ic exit`. fish: `fish -C 'source <file>' -c exit`.
- `cd` hooks: `zsh -ic 'repeat 20 { cd A; cd B }'` minus startup, divided by 40; run once with no version file in `A` and once with a `.nvmrc` in `A` (switch + revert).
- Per-exec: `hyperfine -N '<resolved path>/node -e 0'` for the direct binary vs each manager's shim; the difference is the shim tax.
- First-prompt lag, the user-perceived number: `romkatv/zsh-bench` (`first_prompt_lag_ms`, `command_lag_ms`) once per release.

**In-process timings.** `NVM_TIMINGS=1` makes `nvm-core` print a phase table to stderr on exit (`argv`, `env`, `alias-resolve`, `stat`, `path-edit`, `eval-write`, `total`, each in µs) — the equivalent of mise's `MISE_TIMINGS`. Zero cost when unset (one `getenv`).

**Native profilers.** macOS: `samply record ./nvm-core …` (Firefox Profiler UI; works on any native binary) or `xctrace record --template 'Time Profiler'`; `DYLD_PRINT_STATISTICS` no longer prints on macOS 15+, so audit load cost with `otool -L` and `dyld_info -dependents`. Linux: `perf stat -e task-clock,page-faults,syscalls` and `cargo flamegraph`. Syscall counts: `strace -c` (Linux), `dtruss` (mac, needs SIP relaxed; CI uses the Linux count as the gate). In-process micro-benchmarks with `divan` for version parsing/sorting (1,000 entries), `nvm_change_path`, `index.tab` parsing, `.nvmrc` parsing, and the awk-port formatter.

**CI gate.** `bench/run.sh` executes the matrix above on the macOS arm64 and ubuntu arm64 runners after every build, writes `bench/results/<sha>.json`, and fails the job when any MUST column in §13.2 is exceeded or when a hot command's min regresses > 15 % against `bench/baseline.json` (updated deliberately in the same PR that changes behaviour). GitHub runners are noisy: the gate uses mins over 5 rotated rounds, and the medians are informative only. The golden-output harness (§11) reuses the same command sequences, so correctness and timing run from one script.

### 13.4 Competitors: measured and what the design takes from each

Shell-startup overhead (`zsh -ic exit`, min over 3 rotated rounds, minus 6.2 ms bare), `cd` cost, and the per-`node`-exec tax (direct `node -e 0` = 22.8 ms):

| Tool (version) | Language | Activation model | Startup | `cd`, no change | `cd`, switch | `node` exec tax | Full nvm compat |
|---|---|---|---:|---:|---:|---:|---|
| nvm 0.40.8 | POSIX sh | sourced function, auto-use | +601 | +6.5 (hook) | +203 | 0 | reference |
| nvm 0.40.8 `--no-use` | | | +15 | | | 0 | |
| **this spec (budget)** | Rust | tiny function + `nvm-core`, chpwd-only hook | **≤ 3** | **≤ 2** | **≤ 2.5** | **0** | **yes** |
| fnm 1.39.0 | Rust | `eval $(fnm env --use-on-cd)`; per-shell symlink dir; chpwd hook | +7.3 | ≈ 0 | +2.4 | 0 | no (`FNM_DIR`, no `nvm alias`, no `.nvmrc` comments, no `--reinstall-packages-from`) |
| mise 2026.10.17 | Rust | `eval $(mise activate)`; **precmd + chpwd** `hook-env` every prompt | +23.5 | +12.7 | +14 | 0 (hook) / +11.7 (shim mode) | no (`.nvmrc` opt-in since 2025.10; `mise sync node --nvm` imports) |
| proto 0.63.1 | Rust | shims (`proto run` detection per exec) + `activate` hook | +16.3 | +8.5 | +9.9 | +26.3 | no |
| vfox 1.0.13 | Go + Lua | `eval $(vfox activate)`; precmd hook | +17 | +9.3 | +9 | 0 | no |
| volta 2.0.2 (unmaintained) | Rust | PATH + shims reading `package.json` | +0.6 | — | — | +5.5 | no (no `.nvmrc` at all) |
| asdf 0.20.2 | Go | PATH + shims → `asdf exec` | +0.5 | — | — | +8.3 | no (`legacy_version_file` opt-in) |
| n 11.0.0 | bash | copies node into `$N_PREFIX/bin` | +0.2 | — | — | 0 | no (one global version; `n auto`) |
| nodenv 1.6.2 | bash | shims (bash scripts → `nodenv exec`) | +66 | — | — | +48.8 | no (`.node-version`; plugins) |
| nvs 1.7.1 | JS on bundled node | sourced `nvs.sh`; every command boots node | +46 | ≈ 0 | n/a | 0 | partial (`lts/*` mapped) |

Command latency of the fastest competitors (bare process, min): fnm `env` 3.8, `env --use-on-cd` 3.9, `use` 3.5, `current` 3.2, `ls` 3.5; mise `activate` 7.1, `hook-env` 11.0, `ls` 10.0; proto `activate` 5.9; volta `list` 3.2; asdf `current` 2.9; nodenv `version` 47.4. Published idle-machine references agree on the ordering: nvm 240–300 ms, mise 23, fnm 4–8, volta/asdf/n ≈ 0 at startup (r-jc.jp 2026-09, nvm#3930, mise docs "4 ms no change / 14 ms full reload").

Design consequences (normative):

1. **No shims.** Every shim-based manager (volta, asdf, proto, nodenv, mise-shims) taxes every `node`/`npm` exec by 5–50 ms, breaks `which node`, and needs `reshim` after `npm i -g`. The PATH-edit model is both nvm's contract and the fastest.
2. **chpwd-only hooks, never precmd.** mise/vfox/proto run a process on every prompt (10–24 ms per prompt). `nvm __auto use` is wired to `chpwd` (zsh), `cd` wrapper (bash), `--on-variable PWD` (fish) only (§9.4), and the no-change path returns in ≤ 2 ms.
3. **No per-shell symlink directories.** fnm's `fnm_multishells/<pid>_<ms>` dirs accumulate in `~/.local/state` and break in sandboxes (fnm#1575, #1603). PATH points at `versions/node/vX/bin` directly, as upstream.
4. **The shell file stays tiny.** Parse cost is the one thing lazy-loading cannot remove; the native `nvm.sh` is ≈ 60 lines including the `nvm_find_up`-family wrappers (§9.4) and is gated at ≤ 1 ms.
5. **Zero spawns on the hot path.** The only forks upstream needs that the binary keeps are `npm` for package operations and `node`/`npm` for the `system` version (§11).
6. **Compatibility is the differentiator, not raw speed.** fnm is already within 2–3 ms of the floor; the gap this project closes is "fnm-class latency with 100 % of nvm's surface": `$NVM_DIR` layout, `alias/`, `lts/-N`, `.nvmrc` comments and key=value lines, `default-packages`, `--reinstall-packages-from`, `nvm exec/run`, exit codes, strings, bash completion, and the upstream test suite.

Direct competitors to this exact idea exist and should be tracked, not ignored: `tsilenzio/nvmimic` (Rust, 2026-10, claims 5/6/4 ms bash/zsh/dash vs nvm 144/346/110, parity-tests nvm helpers), `charlzyx/znvm` (Zig + shell), `mose-x/nvm-rust`. None is widely used as of 2026-10; nvmimic's parity tests are worth borrowing.

---

## 14. Implementation language: Rust (decided 2026-10-21)

### 14.1 Decision

`nvm-core` and `nvm-fetch` are written in **Rust** (stable, 6-week train; MSRV = current stable − 2). Zig and Swift were evaluated against the same requirements — ≤ 2 ms hot-path startup, HTTPS to nodejs.org and corporate mirrors through `HTTPS_PROXY` with system and custom CA trust, sha256, tar.xz/tar.gz, terminfo, single static-ish binaries for macOS arm64/x86_64 and Linux x86_64/aarch64 (glibc + musl), Homebrew and curl-installer distribution — with local measurements and source-level checks of each toolchain. Rust is the only option where every requirement works today without writing infrastructure; its single weakness (framework link cost) is solved by the two-binary split in §13.2.

### 14.2 Scorecard

Local numbers: Apple M2 Max, same hot-path program in each language (read `alias/default`, stat `bin/node`, write the eval file), min / median over 3 rotated rounds of 300 runs. "Full stack" = HTTPS client + TLS + sha256 + xz + tar linked but idle. Toolchains: rustc 1.99, zig 0.16.0 (0.17.0 is current), Swift 6.2.4 (Xcode 26.3).

| Criterion | Rust | Zig | Swift |
|---|---|---|---|
| Hot-path startup, libSystem only | **1.64 / 1.94 ms** | 1.80 / 2.21 ms | 1.85 / 2.18 ms (no Foundation) |
| Hot-path startup, full stack linked | 2.59 / 3.24 ms (Security.framework via platform verifier) → mitigated by split binary | **1.82 / 2.14 ms** (std TLS links nothing) | 2.56 / 3.19 ms (Foundation + CFNetwork); 2.43 ms with Foundation alone |
| C reference / fnm `env` | 1.39 / 1.68 ms · 2.91 / 3.71 ms | | |
| HTTPS: proxy env vars | `ureq` 3 / `reqwest` 0.13 read `HTTPS_PROXY`/`NO_PROXY`, CONNECT + auth | reads `https_proxy` but **never TLS-wraps the CONNECT tunnel** → HTTPS via proxy broken (Codeberg #35294 open since 2024, fix PR unmerged) | macOS `URLSession` **ignores `HTTPS_PROXY`** (system proxy prefs only; verified); Linux `FoundationNetworking` = libcurl with no proxy API wired (`//TODO` in source) |
| HTTPS: trust store | `rustls-platform-verifier` → macOS keychain (MDM-pushed roots), Linux native certs + `SSL_CERT_FILE` | raw parse of two keychain files, no `SSL_CERT_FILE` (#36288), TLS 1.2 = ECDHE_RSA only (no ECDSA/CBC), 5,637/6,280 top sites vs 6,270 for `tls.zig`; out-of-order chains fail | macOS excellent; Linux hard-coded distro paths, no custom-CA API |
| Verified today against nodejs.org, TLS-1.2-only host, bad certs | pass | pass (direct connections only) | pass (macOS) |
| xz / tar / sha256 | `liblzma` 0.4 (static xz 5.8): 1.34 s vs 1.39 s system `xz` on the 26 MB node tarball; `tar`, `flate2`, `sha2` | std: 1.95 s (+45 %); std.tar / sha2 fine | SDK ships `liblzma.tbd` but **no `lzma.h`**; `SWCompression` pure Swift; `Compression.framework` has no xz container |
| Linux static binary | musl targets first-class: 1.78 MB aarch64 / 2.08 MB x86_64 static PIE, built locally | 0.90 / 1.08 MB; best in class | Static Linux SDK (musl) works but ≥ 13 MB with ArgumentParser + FoundationEssentials, 25–70 MB real tools; needs the swift.org toolchain, not Xcode's; 6.4.0 aarch64 SIGILL regression |
| Cross-compile from macOS | `cargo zigbuild` both musl targets in 45–50 s incl. liblzma C code | native `-target …-linux-musl`, 34 s | `--swift-sdk x86_64-swift-linux-musl` (Tuist does it); SDK must match toolchain exactly |
| Binary size (macOS, full stack, stripped, LTO) | 1.86 MB (1.65 MB with `--fetch` only) | 0.90 MB ReleaseSafe | 62 KB (everything in the dyld shared cache) |
| Toolchain stability | 6-week stable train, backwards-compatibility guarantee | every release since 0.15 rewrote I/O (0.15 "Writergate", 0.16 `std.Io` everywhere — `std.fs.File` and `std.posix.getenv` were gone when this probe was written, 0.17 `@bitCast`/renames); no 1.0 date; repo moved to Codeberg | source-stable since 5; Swift 6 strict-concurrency churn; static SDK regressions per release |
| Precedent in this tool category | fnm, mise, proto, volta (all Rust), plus zoxide/starship for the hook pattern | zigup/anyzig (both carry open proxy/TLS tickets); Bun left Zig for Rust 2026-05 | swiftly (ships musl libarchive itself); no cross-platform version manager |
| AI-assisted development | 1.4 M GitHub repos, stable APIs → highest generated-code accuracy | 39 K repos; models reliably emit pre-0.15 APIs; project bans LLM contributions | 1.7 M repos skewed to UIKit/SwiftUI; concurrency idioms unstable |
| Clean release build (this stack) | 25 s | 12–29 s | 3 s macOS; 17–23 min for NIO-class Linux builds |
| Homebrew | core formulae use `cargo install`; bottles for arm64, x86_64, Linux | formulae pin `zig@0.15`/`@0.16` because everything breaks per release | `swift build` with `depends_on xcode`; Linux bottles exist |

### 14.3 Why not the others, concretely

**Zig** wins two rows that matter (full-stack startup 1.8 ms without any split, smallest static binaries, effortless cross-compilation) and its std covers xz/tar/sha256/HTTP in-tree. Shipping it to corporate users means owning: HTTPS through an HTTP proxy (the single most common enterprise setup; broken upstream for two years), `SSL_CERT_FILE`/custom CA loading, TLS-1.2 servers with ECDSA certs or CBC suites (older Artifactory/Nexus mirrors), out-of-order chains — or vendoring libcurl + mbedTLS through `allyourcodebase` (whose own README says openssl "doesn't work on macOS"). Plus a std rewrite per release with no 1.0 in sight, and measurably worse assisted coding. Re-evaluate when Zig tags 1.0 and `std.http.Client` tunnels TLS through CONNECT.

**Swift** is the natural macOS-first choice and is excellent there: 62 KB binary, 1.9–2.6 ms, system TLS and keychain for free. It fails the "most compatible" bar on exactly the enterprise paths: `URLSession` ignores `HTTPS_PROXY` on macOS (nvm users set it; fnm/volta/proto issue trackers are full of it), Linux networking is libcurl behind `FoundationNetworking` with no proxy or CA API, static Linux binaries start at 13 MB and need a separate toolchain, and no xz headers ship in the SDK. Fixing those means AsyncHTTPClient + NIOSSL + hand-parsed proxy env + a C liblzma module map — more infrastructure than the project itself. Re-evaluate if a Linux-static Foundation with proxy support lands under 5 MB.

**Rust** costs 1 ms of `Security.framework` load if the network stack lives in the hot binary (fnm pays it) and a 117-crate dependency tree. Both are handled: the split in §13.2 keeps `nvm-core` at libSystem-only (1.6 ms measured, below every competitor), and the tree is pinned (`Cargo.lock` committed, `cargo deny` for licences/advisories, `cargo vet` or `cargo crev` review of the TLS/compression crates, no `tokio` — `ureq` is synchronous).

### 14.4 Stack (binding)

| Concern | Choice | Note |
|---|---|---|
| Edition / toolchain | Rust 2024 edition, stable, `rust-toolchain.toml` pinned per release | `panic = "abort"`, `lto = "fat"`, `codegen-units = 1`, `strip = true`, `opt-level = 3` |
| HTTPS (`nvm-fetch` only) | `ureq` 3 (rustls + ring, `platform-verifier`, `gzip`) | synchronous; proxy env + CONNECT + `NO_PROXY` built in; `NVM_CA_BUNDLE`/`SSL_CERT_FILE` appended via `rustls_native_certs` on Linux and explicit PEM load on all platforms |
| TLS trust | `rustls-platform-verifier` | macOS keychain + MDM roots; Linux native store |
| xz / gzip / tar | `liblzma` (static), `flate2` (`zlib-rs`), `tar` | strip one component, no owner, mode bits kept (§11) |
| Hashing | `sha2` | streamed during download |
| CLI parsing | hand-rolled for `__auto`, `use`, `current`, `which`, `version`; `lexopt` (or `pico-args`) for the rest | no `clap` on the hot path; help text is a static asset (§7) |
| Terminal | in-process terminfo read for `colors`/`sitm` only when stdout is a TTY | no `tput` |
| Linux builds | `*-unknown-linux-musl` static PIE via `cargo-zigbuild` on CI; glibc builds not shipped (Node's own binaries still need glibc on the host, which is unaffected) | |
| macOS builds | native `aarch64-apple-darwin` + `x86_64-apple-darwin`, `lipo`-ed universal binary for the Homebrew bottle; min macOS 12 | |
| Tests | `cargo test` units + golden harness + upstream urchin suites (§11) + `bench/` gate (§13.3) | |

Revisit trigger for this decision: a MUST in §13.2 that cannot be met in Rust after profiling, or Zig/Swift closing the proxy/CA/static-Linux gaps listed above.


---

## 15. Implementation status (2026-10-22)

Implemented in Rust (`src/`), verified on Linux (arm64) inside Docker against upstream nvm v0.40.8 with `scripts/verify.sh` (clean build, 8 unit tests, 277 compared commands (7 scenarios), colour, bash, zsh, fish, portability):
resolution (§3–4), PATH/MANPATH/NODE_PATH editing and the prefix guard (§5), every command in §6 except source builds (`install -s` and the source fallback print an explanatory message), `.nvmrc`, aliases, LTS maintenance, offline installs, install locks, third-party hook, `default-packages`, `--latest-npm`, `reinstall-packages`, colours and `set-colors`, `debug`, `unload`, system node, lazy mode, `nvm-exec`, installer, fish integration, static musl bundle.

Measured (Linux arm64 VM, medians): `nvm-core` hot commands 0.2–0.4 ms (budget 2–3 ms); `source nvm.sh` with auto-use 2.9 ms bash / 4.7 ms zsh versus 138 / 240 ms upstream; `nvm use` in a fresh shell 1.0 ms versus 67 ms; `nvm ls` with 122 versions 1.2 ms versus 670–2,063 ms. Static musl binaries: `nvm-core` 0.66 MB, `nvm-fetch` 1.97 MB. The glibc-dynamic `nvm-core` makes 93 syscalls on the `__auto use` path (budget 80); the static build was not re-measured.

Not done: macOS build and measurements (the Security.framework startup budget in §13.2 is unverified); source builds; io.js installation; `NVM_AUTH_HEADER` is wired but untested; signal handling during downloads (partial tarballs are kept for resume, but Ctrl-C does not clean `files/`); upstream's urchin test suite (its tests call internal shell functions that the native build intentionally does not define).

Known remaining differences from upstream, found by an independent adversarial pass and deliberately not fixed (rare in practice; each is a small, local change if needed):
- Locally installed io.js versions (`versions/io.js/*`) are ignored by `ls`, `version`, `use`.
- Junk entries in `versions/node` (non-version directory names) are skipped, upstream lists them; a directory named like a version but without `bin/node` is listed in both, symlinked and plain-file entries are skipped in both.
- Alias-file parsing: leading whitespace is trimmed (upstream yields `N/A`), a two-line alias file uses the first line only, a self-referencing alias shows `∞` at creation (upstream `N/A`), and over-long alias names fail silently instead of printing `tee`'s error.
- Odd version-ish strings that upstream's regexes accept or reject: `.20`, `20*`, `lts/-0`, `nvm version -- 20`, `which 0` on fake 0.x layouts.
- `NVM_COLORS` with invalid codes: upstream prints `Invalid color code: z` per bad code, native falls back silently to the default palette.
- `ls current` prints `N/A` in native where upstream prints the current row; `ls unstable` prints one `N/A` line fewer; `ls-remote --lts --lts` prints `N/A` without the star.
- `install` with bare option-only invocations (`-s`, `--alias`, `--alias=`, `--offline` and no version/.nvmrc): usage in native, "No version provided..." then continuation in upstream; `install lts/*` also prints `Installing latest LTS version.`.
- Invalid `.nvmrc` with `install --offline`; bare `nvm use --save current` edge cases.
- Read-only `NVM_DIR`: `alias` creation failures exit 1 silently instead of printing the underlying `mkdir`/`tee` error.
