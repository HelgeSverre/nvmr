# nvm (native): thin shell integration around nvm-core. Source from your shell profile exactly like upstream nvm.sh.
# Supports: --no-use, --install, NVM_LAZY=1 (stub node/npm/npx/... until first use), NVM_LAZY_PATH=1.
# Works in bash, zsh and any POSIX sh that provides `local`-free function bodies (names are prefixed instead).
if [ -z "${NVM_DIR-}" ]; then
  if [ -n "${BASH_SOURCE-}" ]; then __nvm_src="${BASH_SOURCE}"
  elif [ -n "${ZSH_VERSION-}" ]; then __nvm_src="${(%):-%x}"
  else __nvm_src="$0"; fi
  NVM_DIR="$(cd "$(dirname "${__nvm_src}")" >/dev/null 2>&1 && pwd)"
  export NVM_DIR
  unset __nvm_src
else
  case $NVM_DIR in
    *[!/]*/)
      NVM_DIR="${NVM_DIR%"${NVM_DIR##*[!/]}"}"
      export NVM_DIR
      printf '%s\n' 'Warning: $NVM_DIR should not have trailing slashes' >&2
      ;;
  esac
fi
if [ -z "${NVM_CD_FLAGS-}" ]; then export NVM_CD_FLAGS=''; fi
if [ -n "${ZSH_VERSION-}" ]; then NVM_CD_FLAGS="-q"; fi

# --- the nvm function: run nvm-core, then source whatever shell statements it queued (spec §1.1) ---
nvm() {
  if [ "$#" -lt 1 ]; then set -- --help; fi
  if [ -n "${__NVM_LAZY_PENDING-}" ]; then __nvm_lazy_load; fi
  __nvm_eval="$(command mktemp "${TMPDIR:-/tmp}/nvm-eval.XXXXXX")" || return 1
  NVM_EVAL_FILE="$__nvm_eval" NVM_SHELL="${NVM_SHELL:-$([ -n "${ZSH_VERSION-}" ] && echo zsh || echo bash)}" \
    command "${NVM_CORE:-$NVM_DIR/nvm-core}" "$@"
  __nvm_rc=$?
  if [ -s "$__nvm_eval" ]; then . "$__nvm_eval"; fi
  command rm -f "$__nvm_eval"
  __nvm_eval=
  return $__nvm_rc
}

# --- helpers that upstream recipes (cdnvm, bash_completion, nvm-exec) call ---
nvm_echo() { command printf %s\\n "$*" 2>/dev/null; }
nvm_err() { nvm_echo "$@" >&2; }
nvm_has() { type "${1-}" >/dev/null 2>&1; }
nvm_find_up() {
  __nvm_d="${PWD}"
  while [ "${__nvm_d}" != "" ] && [ ! -f "${__nvm_d}/${1-}" ]; do __nvm_d=${__nvm_d%/*}; done
  nvm_echo "${__nvm_d}"
  unset __nvm_d
}
nvm_find_nvmrc() {
  __nvm_d="$(nvm_find_up '.nvmrc')"
  if [ -e "${__nvm_d}/.nvmrc" ]; then nvm_echo "${__nvm_d}/.nvmrc"; fi
  unset __nvm_d
}
nvm_ls_current() { command "${NVM_CORE:-$NVM_DIR/nvm-core}" current; }
nvm_version() { command "${NVM_CORE:-$NVM_DIR/nvm-core}" version "$@"; }
nvm_ls() { command "${NVM_CORE:-$NVM_DIR/nvm-core}" __ls "$@"; }
nvm_rc_version() {
  __nvm_o="$(command "${NVM_CORE:-$NVM_DIR/nvm-core}" __rcversion)"; __nvm_r=$?
  if [ $__nvm_r -ne 0 ]; then
    if [ $__nvm_r -eq 1 ]; then nvm_err 'No .nvmrc file found'; else nvm_err 'invalid .nvmrc!'; fi
    unset __nvm_o __nvm_r; return 1
  fi
  NVM_RC_VERSION="$(nvm_echo "${__nvm_o}" | command sed -n 2p)"
  if [ -z "${NVM_SILENT-}" ]; then nvm_echo "Found '$(nvm_echo "${__nvm_o}" | command sed -n 1p)' with version <${NVM_RC_VERSION}>"; fi
  nvm_echo "${NVM_RC_VERSION}" >&3 2>/dev/null || true
  unset __nvm_o __nvm_r
}

# --- lazy mode (spec §9.3) ---
__nvm_lazy_load() {
  unset __NVM_LAZY_PENDING
  for __nvm_c in ${NVM_LAZY_CMDS:-node npm npx corepack yarn pnpm}; do unset -f "$__nvm_c" 2>/dev/null; done
  unset __nvm_c
  nvm __auto use >/dev/null 2>&1
}
__nvm_lazy_stub() {
  __nvm_lazy_load
  command "$@"
}

# --- startup ---
__nvm_mode=use
for __nvm_a in "$@"; do
  case "$__nvm_a" in
    --no-use) __nvm_mode=none ;;
    --install) __nvm_mode=install ;;
    --lazy) NVM_LAZY=1 ;;
  esac
done
case "$__nvm_mode" in
  none) ;;
  install) nvm __auto install >/dev/null ;;
  use)
    if [ -n "${NVM_LAZY-}" ]; then
      if [ -n "${NVM_LAZY_PATH-}" ] && [ -r "$NVM_DIR/alias/default" ]; then
        read -r __nvm_v <"$NVM_DIR/alias/default" 2>/dev/null
        case "$__nvm_v" in
          v*.*.*) [ -x "$NVM_DIR/versions/node/$__nvm_v/bin/node" ] && PATH="$NVM_DIR/versions/node/$__nvm_v/bin:$PATH" && export PATH NVM_BIN="$NVM_DIR/versions/node/$__nvm_v/bin" ;;
          *) __NVM_LAZY_PENDING=1 ;;
        esac
        unset __nvm_v
      else
        __NVM_LAZY_PENDING=1
      fi
      if [ -n "${__NVM_LAZY_PENDING-}" ]; then
        for __nvm_c in ${NVM_LAZY_CMDS:-node npm npx corepack yarn pnpm}; do
          eval "${__nvm_c}() { __nvm_lazy_stub ${__nvm_c} \"\$@\"; }"
        done
        unset __nvm_c
      fi
    else
      nvm __auto use >/dev/null
    fi
    ;;
esac
unset __nvm_mode __nvm_a
