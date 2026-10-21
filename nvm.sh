# Native nvm: thin shell function around nvm-core (spec §1.2).
# Sourcing this file defines `nvm`, then auto-uses the default version unless --no-use is given.
if [ -z "${NVM_DIR-}" ]; then
  NVM_DIR="$(cd "$(dirname "${BASH_SOURCE:-${(%):-%x}}")" 2>/dev/null && pwd)"
  export NVM_DIR
fi
nvm() {
  if [ "$#" -lt 1 ]; then set -- --help; fi
  __nvm_eval="$(command mktemp "${TMPDIR:-/tmp}/nvm-eval.XXXXXX")" || return 1
  NVM_EVAL_FILE="$__nvm_eval" NVM_SHELL="${NVM_SHELL:-$([ -n "${ZSH_VERSION-}" ] && echo zsh || echo bash)}" \
    command "${NVM_CORE:-$NVM_DIR/nvm-core}" "$@"
  __nvm_rc=$?
  if [ -s "$__nvm_eval" ]; then . "$__nvm_eval"; fi
  command rm -f "$__nvm_eval"
  __nvm_eval=
  return $__nvm_rc
}
case " $* " in
  *" --no-use "*) ;;
  *) nvm __auto use ;;
esac
