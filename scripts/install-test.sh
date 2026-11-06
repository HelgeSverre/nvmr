#!/bin/bash
# Exercise the installed files in a fresh XDG-configured fish home.
set -euo pipefail

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
mkdir -p "$tmp/bundle" "$tmp/home"
cp /src/target/release/nvmr /src/target/release/nvmr-fetch \
  /src/nvm.sh /src/nvm-exec /src/bash_completion /src/nvm.fish "$tmp/bundle/"

export HOME="$tmp/home" XDG_CONFIG_HOME="$tmp/home/.config" PROFILE=/dev/null
NVMR_BUNDLE="$tmp/bundle" sh /src/install.sh >/dev/null
test -x "$XDG_CONFIG_HOME/nvm/nvmr"
fish -c 'test "$NVM_DIR" = "$XDG_CONFIG_HOME/nvm"; and nvm --version' | grep -qx '0.40.8'
custom_dir="$tmp/home/a custom/nvm"
NVM_DIR="$custom_dir" NVMR_BUNDLE="$tmp/bundle" sh /src/install.sh >/dev/null
env -u NVM_DIR fish -c 'test "$NVM_DIR" = "$argv[1]"; and nvm --version' "$custom_dir" | grep -qx '0.40.8'
NVM_DIR="$custom_dir" NVMR_BUNDLE="$tmp/bundle" sh /src/install.sh >/dev/null
test "$(grep -c '^set -gx NVM_DIR ' "$XDG_CONFIG_HOME/fish/conf.d/nvm.fish")" -eq 1
profile="$tmp/home/profile"
: >"$profile"
NVM_DIR="$custom_dir" PROFILE="$profile" NVMR_BUNDLE="$tmp/bundle" sh /src/install.sh >/dev/null
NVM_DIR="$custom_dir" PROFILE="$profile" NVMR_BUNDLE="$tmp/bundle" sh /src/install.sh >/dev/null
test "$(grep -c '^export NVM_DIR=' "$profile")" -eq 1
bash --noprofile --rcfile "$profile" -ic 'nvm --version' 2>/dev/null | grep -qx '0.40.8'
archive="nvmr-$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m).tar.gz"
tar -czf "$tmp/$archive" -C "$tmp/bundle" .
url_home="$tmp/url-home"
mkdir -p "$url_home"
HOME="$url_home" XDG_CONFIG_HOME= PROFILE=/dev/null NVMR_URL="file://$tmp" sh /src/install.sh >/dev/null
HOME="$url_home" bash -c '. "$HOME/.nvm/nvm.sh" --no-use; nvm --version' | grep -qx '0.40.8'
broken="$tmp/broken"
cp -a "$tmp/bundle" "$broken"
rm "$broken/nvmr"
mkdir "$broken/nvmr"
printf '# broken bundle\n' >"$broken/nvm.sh"
before="$(sha256sum "$custom_dir/nvm.sh")"
if NVM_DIR="$custom_dir" PROFILE=/dev/null NVMR_BUNDLE="$broken" sh /src/install.sh >/dev/null 2>&1; then
  exit 1
fi
test "$before" = "$(sha256sum "$custom_dir/nvm.sh")"
echo 'INSTALL-XDG-FISH-PASS'
