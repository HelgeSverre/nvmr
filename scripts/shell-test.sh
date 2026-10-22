#!/bin/bash
# Inside container: compare upstream vs native nvm.sh in bash and zsh, incl. README auto-switch recipes.
export TERM=dumb NVM_NO_COLORS=--no-colors
mkfake() { rm -rf /tmp/sh; mkdir -p /tmp/sh/proj/a /tmp/sh/proj/b /tmp/sh/alias
  for v in v20.20.2 v22.12.0; do d=/tmp/sh/versions/node/$v/bin; mkdir -p $d; printf '#!/bin/sh\necho %s\n' $v > $d/node; chmod +x $d/node; done
  echo 22 > /tmp/sh/alias/default; echo 20 > /tmp/sh/proj/a/.nvmrc; echo 22.12.0 > /tmp/sh/proj/b/.nvmrc; }
body_common='
cd /tmp/sh/proj
echo "start: node=$(command -v node) v=$(node -v 2>&1) bin=${NVM_BIN-} cur=$(nvm current)"
nvm use 20 >/dev/null; echo "use20: v=$(node -v) first=${PATH%%:*} bin=$NVM_BIN inc=$NVM_INC"
nvm use default >/dev/null; echo "usedef: v=$(node -v)"
nvm use --silent 20; echo "silent: v=$(node -v) cur=$(nvm current)"
nvm deactivate >/dev/null 2>&1; echo "deact: node=$(command -v node || echo none) bin=${NVM_BIN-unset}"
nvm use 22.12.0 >/dev/null; echo "again: v=$(node -v) cur=$(nvm current) which=$(nvm which 22)"
echo "findup=$(cd /tmp/sh/proj/a && nvm_find_up .nvmrc) nvmrc=$(cd /tmp/sh/proj/a && nvm_find_nvmrc) ver=$(nvm_version 20) cur=$(nvm_ls_current) ls=$(nvm_ls | tr "\n" " ")"
'
bash_recipe='
cdnvm() {
  command cd "$@" || return $?
  nvm_path="$(nvm_find_up .nvmrc | command tail -1)"
  if [[ ! $nvm_path = *[^[:space:]]* ]]; then
    declare default_version; default_version="$(nvm version default)"
    if [[ $default_version == "N/A" ]]; then nvm alias default node; default_version=$(nvm version default); fi
    if [[ $(nvm current) != "$default_version" ]]; then nvm use default; fi
  elif [[ -s "$nvm_path/.nvmrc" && -r "$nvm_path/.nvmrc" ]]; then
    declare nvm_version; nvm_version=$(<"$nvm_path"/.nvmrc)
    declare locally_resolved_nvm_version
    locally_resolved_nvm_version=$(nvm ls --no-colors "$nvm_version" | command tail -1 | command tr -d "\->*" | command tr -d "[:space:]")
    if [[ "$locally_resolved_nvm_version" == "N/A" ]]; then nvm install "$nvm_version"; elif [[ $(nvm current) != "$locally_resolved_nvm_version" ]]; then nvm use "$nvm_version"; fi
  fi
}
cdnvm /tmp/sh/proj/a; echo "cd a: v=$(node -v)"
cdnvm /tmp/sh/proj/b; echo "cd b: v=$(node -v)"
cdnvm /tmp; echo "cd /tmp: v=$(node -v)"
'
zsh_recipe='
autoload -U add-zsh-hook
load-nvmrc() {
  local nvmrc_path; nvmrc_path="$(nvm_find_nvmrc)"
  if [ -n "$nvmrc_path" ]; then
    local nvmrc_node_version; nvmrc_node_version=$(nvm version "$(cat "${nvmrc_path}")")
    if [ "$nvmrc_node_version" = "N/A" ]; then nvm install
    elif [ "$nvmrc_node_version" != "$(nvm version)" ]; then nvm use; fi
  elif [ -n "$(PWD=$OLDPWD nvm_find_nvmrc)" ] && [ "$(nvm version)" != "$(nvm version default)" ]; then
    echo "Reverting to nvm default version"; nvm use default
  fi
}
add-zsh-hook chpwd load-nvmrc
cd /tmp/sh/proj/a; echo "cd a: v=$(node -v)"
cd /tmp/sh/proj/b; echo "cd b: v=$(node -v)"
cd /tmp; echo "cd /tmp: v=$(node -v)"
'
tail_common='
nvm unload; echo "unload: fn=$(type nvm 2>&1 | head -c 60) dir=${NVM_DIR-unset}"
'
# zsh-opts: options and global aliases from popular zsh setups (oh-my-zsh, yadr) that break naive shell code;
# upstream's test/sourcing covers the aliases and extendedglob (with a comment in the default alias).
# nounset is left out: upstream's `nvm version` without arguments trips over it in zsh (`1: parameter not set`).
zsh_opts='alias -g R="| less -S" G="| grep" C="| wc -l" H="| head" L="| less" N="| /dev/null" S="| sort"
setopt extendedglob pipefail nonomatch; echo "22.12.0 # comment" > /tmp/sh/alias/default;'
for variant in bash zsh zsh-opts; do
  sh=${variant%-opts}; pre=; [ "$variant" = zsh-opts ] && pre=$zsh_opts
  for who in up nat; do
    mkfake
    if [ $who = up ]; then init=". /opt/nvm-upstream/nvm.sh"; else init="NVM_CORE=/src/target/release/nvm-core; . /src/nvm.sh"; fi
    recipe=$bash_recipe; [ $sh = zsh ] && recipe=$zsh_recipe
    env -i HOME=/root TERM=dumb PATH=/usr/local/bin:/usr/bin:/bin NVM_DIR=/tmp/sh $sh -c "$pre $init; $body_common $recipe $tail_common" > /tmp/sh-$variant-$who.txt 2>&1
  done
  echo "##### $variant"; diff /tmp/sh-$variant-up.txt /tmp/sh-$variant-nat.txt && echo "IDENTICAL ($variant)"; head -"${SHOW:-0}" /tmp/sh-$variant-nat.txt
done
