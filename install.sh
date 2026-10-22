#!/bin/sh
# Installs native nvm into $NVM_DIR (default ~/.nvm, or $XDG_CONFIG_HOME/nvm when XDG_CONFIG_HOME is set)
# and adds the same profile lines upstream's installer adds, so existing setups keep working.
#
#   NVM_FAST_BUNDLE=<dir>   directory containing nvm.sh nvm-core nvm-fetch nvm-exec bash_completion nvm.fish
#   NVM_FAST_URL=<url>      or a base URL serving nvm-fast-<os>-<arch>.tar.gz
#   NVM_DIR, PROFILE        same meaning as upstream install.sh (PROFILE=/dev/null leaves profiles alone)
set -eu

nvm_default_dir() {
  if [ -n "${XDG_CONFIG_HOME-}" ]; then printf %s "${XDG_CONFIG_HOME}/nvm"; else printf %s "${HOME}/.nvm"; fi
}
NVM_DIR="${NVM_DIR:-$(nvm_default_dir)}"
mkdir -p "$NVM_DIR"

os="$(uname -s | tr '[:upper:]' '[:lower:]')"
arch="$(uname -m)"
case "$arch" in x86_64 | amd64) arch=x86_64 ;; aarch64 | arm64) arch=aarch64 ;; esac

if [ -n "${NVM_FAST_BUNDLE-}" ]; then
  src="$NVM_FAST_BUNDLE"
elif [ -n "${NVM_FAST_URL-}" ]; then
  src="$(mktemp -d)"
  trap 'rm -rf "$src"' EXIT
  curl -fsSL "${NVM_FAST_URL}/nvm-fast-${os}-${arch}.tar.gz" | tar -xz -C "$src"
else
  echo "install.sh: set NVM_FAST_BUNDLE or NVM_FAST_URL" >&2
  exit 1
fi

files="nvm.sh nvm-core nvm-fetch nvm-exec bash_completion nvm.fish"
# Check the whole bundle first so a broken one never leaves a half-updated install.
for f in $files; do
  [ -e "$src/$f" ] || { echo "install.sh: missing $src/$f" >&2; exit 1; }
done
for f in $files; do
  cp "$src/$f" "$NVM_DIR/$f.new" && mv "$NVM_DIR/$f.new" "$NVM_DIR/$f"
done
chmod +x "$NVM_DIR/nvm-core" "$NVM_DIR/nvm-fetch" "$NVM_DIR/nvm-exec"

# Profile detection (same order as upstream)
profile="${PROFILE-}"
if [ -z "$profile" ]; then
  case "${SHELL-}" in
    */bash) for p in "$HOME/.bashrc" "$HOME/.bash_profile"; do [ -f "$p" ] && profile="$p" && break; done ;;
    */zsh) for p in "${ZDOTDIR:-$HOME}/.zshrc" "${ZDOTDIR:-$HOME}/.zprofile"; do [ -f "$p" ] && profile="$p" && break; done ;;
  esac
  if [ -z "$profile" ]; then
    for p in .profile .bashrc .bash_profile .zprofile .zshrc; do [ -f "$HOME/$p" ] && profile="$HOME/$p" && break; done
  fi
fi

case "$NVM_DIR" in "$HOME"/*) export_dir="\$HOME/${NVM_DIR#"$HOME"/}" ;; *) export_dir="$NVM_DIR" ;; esac
if [ -n "$profile" ] && [ "$profile" != /dev/null ]; then
  if ! grep -qs 'nvm.sh' "$profile"; then
    {
      printf '\n'
      printf 'export NVM_DIR="%s"\n' "$export_dir"
      printf '[ -s "$NVM_DIR/nvm.sh" ] && \\. "$NVM_DIR/nvm.sh"  # This loads nvm\n'
      printf '[ -s "$NVM_DIR/bash_completion" ] && \\. "$NVM_DIR/bash_completion"  # This loads nvm bash_completion\n'
    } >>"$profile"
    echo "=> nvm source string appended to $profile"
  else
    echo "=> nvm source string already in $profile"
  fi
elif [ "$profile" != /dev/null ]; then
  echo "=> Profile not found. Add this to your shell profile:"
  printf '   export NVM_DIR="%s"\n   [ -s "$NVM_DIR/nvm.sh" ] && \\. "$NVM_DIR/nvm.sh"\n' "$export_dir"
fi

# fish: conf.d integration
if command -v fish >/dev/null 2>&1; then
  fdir="${XDG_CONFIG_HOME:-$HOME/.config}/fish/conf.d"
  mkdir -p "$fdir" && cp "$NVM_DIR/nvm.fish" "$fdir/nvm.fish" && echo "=> fish integration installed to $fdir/nvm.fish"
fi
echo "=> nvm installed in $NVM_DIR. Open a new terminal or run: . \"$NVM_DIR/nvm.sh\""
