# nvm (native) for fish. Install as ~/.config/fish/conf.d/nvm.fish (or source it from config.fish).
# NVM_LAZY=1 skips the startup auto-use (run `nvm use` yourself); the cd hook below still applies.
set -q NVM_DIR; or set -gx NVM_DIR $HOME/.nvm

function __nvm_core
    if test -n "$NVM_CORE"
        echo $NVM_CORE
    else
        echo $NVM_DIR/nvm-core
    end
end

function nvm
    if test (count $argv) -lt 1
        set argv --help
    end
    set -l f $NVM_DIR/.cache/eval.$fish_pid
    set -l core (__nvm_core)
    env NVM_EVAL_FILE=$f NVM_SHELL=fish $core $argv
    set -l rc $status
    if test -s $f
        source $f
        : >$f
    end
    return $rc
end

function nvm_find_up
    set -l d $PWD
    while test -n "$d"; and not test -f "$d/$argv[1]"
        set d (string replace -r '/[^/]*$' '' -- $d)
    end
    echo $d
end

function nvm_find_nvmrc
    set -l d (nvm_find_up .nvmrc)
    test -e "$d/.nvmrc"; and echo "$d/.nvmrc"
end

function nvm_ls_current
    set -l c (__nvm_core)
    command $c current
end
function nvm_version
    set -l c (__nvm_core)
    command $c version $argv
end

# auto-switch on directory change (no process is spawned when nothing changes beyond nvm-core itself)
function __nvm_cd --on-variable PWD
    nvm __auto cd >/dev/null 2>&1
end

if not set -q NVM_LAZY
    nvm __auto use >/dev/null 2>&1
end
