#!/usr/bin/env python3
"""Reproducible benchmark harness: upstream nvm vs nvm-fast vs fnm.

Runs identically inside the pinned Docker image (bench/docker.sh) and on a host sandbox (bench/host.sh).
Every input is pinned in bench/pins.env; every run writes raw hyperfine JSON, the environment it ran in,
and a Markdown report. See AGENTS.md ("Benchmarking and profiling") for the rules.

  bench.py setup  --root R --nvm-src DIR --fast-src DIR --fnm BIN --dist DIR
  bench.py run    --root R --out DIR [--rounds 3] [--quick] [--filter REGEX] [--seed N]
  bench.py report DIR
  bench.py profile --root R --out DIR          (Linux: strace/perf; macOS: linkage and RSS)
"""
from __future__ import annotations

import argparse
import json
import os
import platform
import random
import re
import shlex
import shutil
import statistics
import subprocess
import sys
import tarfile
import time
from dataclasses import asdict, dataclass, field
from pathlib import Path

HERE = Path(__file__).resolve().parent
FAKE_VERSIONS = 100  # extra fake installs in the "many" state, for listing and resolution scaling
FAKE_ALIASES = 20

# --------------------------------------------------------------------------------------------- pins


def pins() -> dict[str, str]:
    out = {}
    for line in (HERE / "pins.env").read_text().splitlines():
        line = line.strip()
        if line and not line.startswith("#") and "=" in line:
            k, v = line.split("=", 1)
            out[k] = v
    return out


def node_platform() -> str:
    os_ = {"Linux": "linux", "Darwin": "darwin"}[platform.system()]
    arch = {"x86_64": "x64", "amd64": "x64", "aarch64": "arm64", "arm64": "arm64"}[platform.machine().lower()]
    return f"{os_}-{arch}"


# --------------------------------------------------------------------------------------------- setup


def sh(cmd: list[str] | str, **kw) -> subprocess.CompletedProcess:
    return subprocess.run(cmd, shell=isinstance(cmd, str), check=True, text=True, **kw)


def copy_tree(src: Path, dst: Path) -> None:
    """Cheap copy of the node installs: hard links on Linux, APFS clones on macOS."""
    dst.parent.mkdir(parents=True, exist_ok=True)
    if platform.system() == "Darwin":
        sh(["cp", "-Rc", str(src), str(dst)])
    else:
        sh(["cp", "-al", str(src), str(dst)])


def fake_node(bindir: Path, version: str) -> None:
    bindir.mkdir(parents=True, exist_ok=True)
    node = bindir / "node"
    node.write_text(f"#!/bin/sh\necho {version}\n")
    node.chmod(0o755)


def cmd_setup(a: argparse.Namespace) -> None:
    p = pins()
    root = Path(a.root).resolve()
    if root.exists():
        shutil.rmtree(root)
    (root / "home").mkdir(parents=True)
    plat = node_platform()

    # Extract each pinned node once, then share it between the managers.
    extracted = root / "_node"
    for key in ("NODE_A", "NODE_B"):
        ver = p[key]
        tarball = Path(a.dist) / f"node-{ver}-{plat}.tar.xz"
        with tarfile.open(tarball) as tf:
            tf.extractall(extracted, filter="tar") if sys.version_info >= (3, 12) else tf.extractall(extracted)
        (extracted / f"node-{ver}-{plat}").rename(extracted / ver)

    def nvm_state(dirname: str, src: Path, many: bool) -> None:
        d = root / dirname
        shutil.copytree(src, d, symlinks=True, ignore=shutil.ignore_patterns(".git", "test"))
        for key in ("NODE_A", "NODE_B"):
            copy_tree(extracted / p[key], d / "versions/node" / p[key])
        (d / "alias").mkdir(exist_ok=True)
        (d / "alias/default").write_text(p["NODE_A"].lstrip("v").split(".")[0] + "\n")
        if many:
            for i in range(1, FAKE_VERSIONS + 1):
                fake_node(d / f"versions/node/v18.{i}.0/bin", f"v18.{i}.0")
            for i in range(1, FAKE_ALIASES + 1):
                (d / f"alias/a{i}").write_text(f"18.{i}\n")

    def fnm_state(dirname: str, many: bool) -> None:
        d = root / dirname
        for key in ("NODE_A", "NODE_B"):
            copy_tree(extracted / p[key], d / "node-versions" / p[key] / "installation")
        (d / "aliases").mkdir(parents=True)
        os.symlink(d / "node-versions" / p["NODE_A"] / "installation", d / "aliases/default")
        if many:
            for i in range(1, FAKE_VERSIONS + 1):
                fake_node(d / f"node-versions/v18.{i}.0/installation/bin", f"v18.{i}.0")

    for many in (False, True):
        sfx = "-many" if many else ""
        nvm_state(f"nvm{sfx}", Path(a.nvm_src), many)
        nvm_state(f"fast{sfx}", Path(a.fast_src), many)
        fnm_state(f"fnm{sfx}", many)
    shutil.rmtree(extracted)

    bindir = root / "bin"
    bindir.mkdir()
    shutil.copy2(a.fnm, bindir / "fnm")

    # Projects for the cd benchmarks: a/b switch versions, plain1/plain2 have no version file.
    proj = root / "proj"
    for name, content in (("a", p["NODE_B"].lstrip("v").split(".")[0]), ("b", p["NODE_A"].lstrip("v").split(".")[0])):
        (proj / name).mkdir(parents=True)
        (proj / name / ".nvmrc").write_text(content + "\n")
    for name in ("plain1", "plain2"):
        (proj / name).mkdir(parents=True)

    # Local mirror for the install benchmark: index.tab, index.json, SHASUMS256.txt, NODE_B tarball.
    mirror = root / "mirror"
    vdir = mirror / p["NODE_B"]
    vdir.mkdir(parents=True)
    tb = f"node-{p['NODE_B']}-{plat}.tar.xz"
    shutil.copy2(Path(a.dist) / tb, vdir / tb)
    sha = p[f"NODE_B_SHA256_{plat.replace('-', '_')}"]
    (vdir / "SHASUMS256.txt").write_text(f"{sha}  {tb}\n")
    (mirror / "index.tab").write_text(
        "version\tdate\tfiles\tnpm\tv8\tuv\tzlib\topenssl\tmodules\tlts\tsecurity\n"
        f"{p['NODE_B']}\t2026-01-01\t{plat}\t10.0.0\t-\t-\t-\t-\t-\tIron\t-\n"
    )
    (mirror / "index.json").write_text(json.dumps([
        {"version": p["NODE_B"], "date": "2026-01-01", "files": [plat], "npm": "10.0.0", "lts": "Iron"}
    ]))
    write_rcfiles(root, p)
    print(f"setup complete: {root}")


# --------------------------------------------------------------------------------------------- rc files

# The README auto-switch recipes from upstream nvm, used unchanged for nvm and nvm-fast (drop-in).
BASH_CDNVM = r'''
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
    locally_resolved_nvm_version=$(nvm ls --no-colors "$nvm_version" | command tail -1 | command tr -d '\->*' | command tr -d '[:space:]')
    if [[ "$locally_resolved_nvm_version" == "N/A" ]]; then nvm install "$nvm_version";
    elif [[ $(nvm current) != "$locally_resolved_nvm_version" ]]; then nvm use "$nvm_version"; fi
  fi
}
alias cd='cdnvm'
cdnvm "$PWD" || exit
'''
ZSH_LOAD_NVMRC = r'''
autoload -U add-zsh-hook
load-nvmrc() {
  local nvmrc_path
  nvmrc_path="$(nvm_find_nvmrc)"
  if [ -n "$nvmrc_path" ]; then
    local nvmrc_node_version
    nvmrc_node_version=$(nvm version "$(cat "${nvmrc_path}")")
    if [ "$nvmrc_node_version" = "N/A" ]; then
      nvm install
    elif [ "$nvmrc_node_version" != "$(nvm version)" ]; then
      nvm use
    fi
  elif [ -n "$(PWD=$OLDPWD nvm_find_nvmrc)" ] && [ "$(nvm version)" != "$(nvm version default)" ]; then
    echo "Reverting to nvm default version"
    nvm use default
  fi
}
add-zsh-hook chpwd load-nvmrc
load-nvmrc
'''
# nvm-fast's own hook (what nvm.fish uses): one process per cd, no recipe logic in the shell.
NATIVE_CD = {
    "bash": 'cd() { builtin cd "$@" && nvm __auto cd >/dev/null; }\n',
    "zsh": 'autoload -U add-zsh-hook\n__nvm_cd() { nvm __auto cd >/dev/null; }\nadd-zsh-hook chpwd __nvm_cd\n',
}

# config name -> (state dir, init snippet builder)
CONFIGS = ("bare", "nvm", "nvm-nouse", "nvm-cd", "fast", "fast-nouse", "fast-cd", "fast-autocd", "fnm",
           "nvm-many", "fast-many", "fnm-many")


def init_lines(cfg: str, shell: str, root: Path) -> str:
    if cfg == "bare":
        return ""
    base, _, variant = cfg.partition("-")
    many = variant == "many"
    state = root / (base + ("-many" if many else ""))
    if base == "fnm":
        return (f'export FNM_DIR="{state}"\nexport PATH="{root}/bin:$PATH"\n'
                f'eval "$(fnm env --use-on-cd --shell {shell})"\n')
    lines = f'export NVM_DIR="{state}"\n. "$NVM_DIR/nvm.sh"' + (" --no-use" if variant == "nouse" else "") + "\n"
    if variant == "cd":
        lines += BASH_CDNVM if shell == "bash" else ZSH_LOAD_NVMRC
    elif variant == "autocd":
        lines += NATIVE_CD[shell]
    return lines


def write_rcfiles(root: Path, p: dict[str, str]) -> None:
    for shell in ("bash", "zsh"):
        for cfg in CONFIGS:
            d = root / "rc" / shell / cfg
            d.mkdir(parents=True, exist_ok=True)
            body = init_lines(cfg, shell, root)
            if shell == "bash":
                (d / "bashrc").write_text(body)
            else:
                # NO_GLOBAL_RCS skips /etc/zshrc & co. (macOS forks `locale` and `path_helper` there).
                (d / ".zshenv").write_text("setopt NO_GLOBAL_RCS\n")
                (d / ".zshrc").write_text(body)


# --------------------------------------------------------------------------------------------- matrix


@dataclass
class Bench:
    id: str
    group: str
    tool: str  # nvm | fast | fnm | bare
    cmd: list[str]
    ops: int = 0  # >0: in-shell loop doing `ops` operations; per-op cost is derived against `baseline`
    baseline: str = ""
    check: str = ""  # expected substring in the output of the check run
    check_cmd: list[str] = field(default_factory=list)
    prepare: str = ""
    slow: bool = False  # fewer runs; set automatically when one run takes longer than SLOW_RUN_S


def shell_cmd(shell: str, cfg: str, root: Path, script: str) -> list[str]:
    rc = root / "rc" / shell / cfg
    if shell == "bash":
        return [SHELLS["bash"], "--noprofile", "--rcfile", str(rc / "bashrc"), "-i", "-c", script]
    return ["env", f"ZDOTDIR={rc}", SHELLS["zsh"], "-i", "-c", script]


SHELLS = {"bash": "bash", "zsh": "zsh"}
HYPERFINE = shutil.which("hyperfine") or "hyperfine"  # resolved with the caller's PATH, run with the fixed one


def matrix(root: Path, p: dict[str, str], quick: bool) -> list[Bench]:
    A, B = p["NODE_A"], p["NODE_B"]
    amaj, bmaj = A.lstrip("v").split(".")[0], B.lstrip("v").split(".")[0]
    out: list[Bench] = []
    shells = ("bash", "zsh")
    proj = root / "proj"

    # 1. Shell startup with each tool's documented init (overhead = config - bare).
    for s in shells:
        for cfg in ("bare", "nvm", "nvm-nouse", "nvm-cd", "fast", "fast-nouse", "fast-cd", "fast-autocd", "fnm"):
            tool = cfg.split("-")[0]
            expect = "" if cfg in ("bare", "nvm-nouse", "fast-nouse") else A
            out.append(Bench(
                id=f"startup/{s}/{cfg}", group="startup", tool=tool,
                cmd=shell_cmd(s, cfg, root, "exit"),
                check=expect, check_cmd=shell_cmd(s, cfg, root, "node -v 2>/dev/null; true"),
                baseline=f"startup/{s}/bare" if cfg != "bare" else ""))

    # 2. In-shell operations, measured as loops so ms-scale costs rise above process-start noise.
    def loop(n: int, body: str) -> str:
        return f"i=0; while [ $i -lt {n} ]; do {body}; i=$((i+1)); done"

    # Iterations per tool and operation, sized so the loop takes well over the startup's run-to-run spread
    # (otherwise the startup subtraction is noise). Upstream nvm is ~100x slower, so it gets fewer iterations.
    scale = 1 if quick else 2
    iters = {"nvm": {"use": 4, "current": 10, "version": 5},
             "fast": {"use": 25, "current": 50, "version": 50},
             "fnm": {"use": 25, "current": 50}}
    for s in shells:
        for tool, it in iters.items():
            cfg = tool
            n = it["use"] * scale
            use = "fnm use" if tool == "fnm" else "nvm use"
            cur = "fnm current" if tool == "fnm" else "nvm current"
            base = f"startup/{s}/{cfg}"
            out.append(Bench(f"use/{s}/{tool}", "use", tool,
                             shell_cmd(s, cfg, root, loop(n, f"{use} {bmaj} >/dev/null; {use} {amaj} >/dev/null")),
                             ops=2 * n, baseline=base,
                             check=B, check_cmd=shell_cmd(s, cfg, root, f"{use} {bmaj} >/dev/null; node -v")))
            n = it["current"] * scale
            out.append(Bench(f"current/{s}/{tool}", "current", tool,
                             shell_cmd(s, cfg, root, loop(n, f"{cur} >/dev/null")),
                             ops=n, baseline=base, check=A, check_cmd=shell_cmd(s, cfg, root, cur)))
            # listing with 102 installed versions (+20 aliases for nvm)
            mcfg = f"{cfg}-many"
            ls = "fnm ls" if tool == "fnm" else "nvm ls"
            ln = 1 if tool == "nvm" else 5 * scale
            out.append(Bench(f"ls-many/{s}/{tool}", "ls-many", tool,
                             shell_cmd(s, mcfg, root, loop(ln, f"{ls} >/dev/null")),
                             ops=ln, baseline=f"startup/{s}/{mcfg}",
                             check="v18.100.0", check_cmd=shell_cmd(s, mcfg, root, ls)))
            if tool != "fnm":  # resolution only; fnm has no `version`
                n = it["version"] * scale
                out.append(Bench(f"version/{s}/{tool}", "version", tool,
                                 shell_cmd(s, cfg, root, loop(n, f"nvm version {bmaj} >/dev/null")),
                                 ops=n, baseline=base, check=B,
                                 check_cmd=shell_cmd(s, cfg, root, f"nvm version {bmaj}")))
        # startup of the "many" states, as baselines for ls-many
        for cfg in ("nvm-many", "fast-many", "fnm-many"):
            out.append(Bench(f"startup/{s}/{cfg}", "startup-many", cfg.split("-")[0], shell_cmd(s, cfg, root, "exit"),
                             baseline=f"startup/{s}/bare"))

    # 3. cd with auto-switch: into a project with .nvmrc and back (2 switches per iteration),
    #    and between directories without a version file (2 no-op hooks per iteration).
    cd_cfgs = {  # label -> (config, iterations for cd-switch, iterations for cd-nochange)
        "nvm": ("nvm-cd", 2, 10), "fast": ("fast-cd", 10, 20),
        "fast-native": ("fast-autocd", 25, 25), "fnm": ("fnm", 25, 25)}
    for s in shells:
        for label, (cfg, n_switch, n_same) in cd_cfgs.items():
            n = n_switch * scale
            tool = label.split("-")[0]
            # enter the project root without firing the hook, so only the loop's cds are hooked
            pre = f"builtin cd {proj}; " if s == "bash" else f"cd -q {proj}; "
            out.append(Bench(f"cd-switch/{s}/{label}", "cd-switch", tool,
                             shell_cmd(s, cfg, root, pre + loop(n, f"cd {proj}/a >/dev/null; cd {proj}/b >/dev/null")),
                             ops=2 * n, baseline=f"startup/{s}/{cfg}",
                             check=B, check_cmd=shell_cmd(s, cfg, root, f"cd {proj}/a >/dev/null; node -v")))
            n = n_same * scale
            out.append(Bench(f"cd-nochange/{s}/{label}", "cd-nochange", tool,
                             shell_cmd(s, cfg, root, pre + loop(n, f"cd {proj}/plain1; cd {proj}/plain2")),
                             ops=2 * n, baseline=f"startup/{s}/{cfg}",
                             check=A, check_cmd=shell_cmd(s, cfg, root, f"cd {proj}/plain1; node -v")))

    # 4. Bare process latency of the hot binaries (no shell).
    env_fast = root / "fast"
    out += [
        Bench("process/nvm-core-current", "process", "fast", [str(env_fast / "nvm-core"), "current"]),
        Bench("process/nvm-core-auto-use", "process", "fast", [str(env_fast / "nvm-core"), "__auto", "use"]),
        Bench("process/nvm-core-version", "process", "fast", [str(env_fast / "nvm-core"), "version", bmaj]),
        Bench("process/fnm-env", "process", "fnm", [str(root / "bin/fnm"), "env", "--shell", "bash"]),
        # `fnm current` needs the per-shell env from `fnm env`; `fnm ls` is the comparable read-only command
        Bench("process/fnm-ls", "process", "fnm", [str(root / "bin/fnm"), "ls"]),
        Bench("process/nvm-core-ls", "process", "fast", [str(env_fast / "nvm-core"), "ls"]),
        Bench("process/true", "process", "bare", ["true"]),
    ]

    # 5. Install from a loopback mirror (download + verify + extract), clean each run.
    if not quick:
        for s in ("bash",):
            for tool, cfg in (("nvm", "nvm-nouse"), ("fast", "fast-nouse"), ("fnm", "fnm")):
                state = root / ("fnm" if tool == "fnm" else tool)
                if tool == "fnm":
                    script = f"FNM_NODE_DIST_MIRROR=http://127.0.0.1:{MIRROR_PORT} fnm install {B} >/dev/null 2>&1"
                    prep = f"rm -rf {state}/node-versions/{B}"
                else:
                    script = f"NVM_NODEJS_ORG_MIRROR=http://127.0.0.1:{MIRROR_PORT} nvm install {B} >/dev/null 2>&1"
                    prep = f"rm -rf {state}/versions/node/{B} {state}/.cache"
                out.append(Bench(f"install/{s}/{tool}", "install", tool, shell_cmd(s, cfg, root, script),
                                 prepare=prep, slow=True,
                                 check=B, check_cmd=shell_cmd(s, cfg, root, script + f"; {'fnm exec --using=' + B if tool == 'fnm' else 'nvm exec --silent ' + B} node -v")))
    return out


MIRROR_PORT = 8765
SLOW_RUN_S = 0.3  # above this per run, a benchmark gets 5-20 runs instead of 50-300

# --------------------------------------------------------------------------------------------- run


def loadavg() -> list[float]:
    try:
        return [round(x, 2) for x in os.getloadavg()]
    except OSError:
        return []


def capture(cmd: list[str] | str) -> str:
    try:
        r = subprocess.run(cmd, shell=isinstance(cmd, str), capture_output=True, text=True, timeout=30)
        return (r.stdout + r.stderr).strip()
    except (OSError, subprocess.TimeoutExpired) as e:
        return f"<{e}>"


def cpu_model() -> str:
    if platform.system() == "Darwin":
        return capture(["sysctl", "-n", "machdep.cpu.brand_string"])
    for line in Path("/proc/cpuinfo").read_text().splitlines():
        if line.lower().startswith(("model name", "cpu model")):
            return line.split(":", 1)[1].strip()
    model = capture("lscpu | sed -n 's/^Model name: *//p' | head -1")
    return model if model and model != "-" else platform.machine()


def environment(root: Path, a: argparse.Namespace) -> dict:
    p = pins()
    cg = {}
    for f in ("/sys/fs/cgroup/cpu.max", "/sys/fs/cgroup/cpuset.cpus.effective", "/sys/fs/cgroup/memory.max"):
        if Path(f).exists():
            cg[f] = Path(f).read_text().strip()
    return {
        "started": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "where": os.environ.get("BENCH_WHERE", "host"),
        "git": os.environ.get("BENCH_GIT", capture(["git", "-C", str(HERE.parent), "describe", "--always", "--dirty"])),
        "pins": p,
        "seed": a.seed, "rounds": a.rounds, "quick": a.quick, "filter": a.filter,
        "platform": platform.platform(), "uname": capture(["uname", "-a"]), "cpu": cpu_model(),
        "ncpu": os.cpu_count(), "cgroup": cg,
        "versions": {
            "hyperfine": capture([HYPERFINE, "--version"]),
            "bash": capture([SHELLS["bash"], "--version"]).splitlines()[0],
            "zsh": capture([SHELLS["zsh"], "--version"]),
            "fnm": capture([str(root / "bin/fnm"), "--version"]),
            "nvm-fast": capture([str(root / "fast/nvm-core"), "--version"]),
            "python": platform.python_version(),
        },
    }


def noisy(meta: dict) -> bool:
    """The 1-minute load exceeded a quarter of the CPUs it is averaged over (the whole machine or VM, even when
    the benchmark is pinned to fewer CPUs) before the run or any round."""
    loads = meta["load_before"][:1] + [r["load_before"][0] for r in meta.get("rounds_log", []) if r["load_before"]]
    return bool(loads) and max(loads) > meta["ncpu"] / 4


def check(b: Bench, env: dict) -> None:
    if not b.check_cmd:  # no output to verify: the command must at least succeed, as hyperfine requires
        r = subprocess.run(b.cmd, capture_output=True, text=True, env=env, timeout=600)
        if r.returncode != 0:
            raise SystemExit(f"check failed for {b.id}: exit {r.returncode}\ncmd: {shlex.join(b.cmd)}\n"
                             f"stderr: {r.stderr[-2000:]}")
        return
    if b.prepare:
        subprocess.run(b.prepare, shell=True, env=env)
    r = subprocess.run(b.check_cmd, capture_output=True, text=True, env=env, timeout=600)
    if b.check and b.check not in r.stdout:
        raise SystemExit(f"check failed for {b.id}: expected {b.check!r} in stdout\n"
                         f"cmd: {shlex.join(b.check_cmd)}\nstdout: {r.stdout[-2000:]}\nstderr: {r.stderr[-2000:]}")


def run_env(root: Path) -> dict:
    """The fixed environment every benchmarked process starts with."""
    return {
        # No Homebrew or user dirs: a stray system `node` would change what the managers do.
        "HOME": str(root / "home"), "PATH": "/usr/bin:/bin:/usr/sbin:/sbin" if platform.system() == "Darwin" else "/usr/local/bin:/usr/bin:/bin",
        "LANG": "C", "LC_ALL": "C", "TERM": "dumb", "NVM_NO_COLORS": "--no-colors", "NVM_NO_PROGRESS": "1",
        "TMPDIR": str(root / "home"), "SHELL": "/bin/sh",
        # process benchmarks read these directly
        "NVM_DIR": str(root / "fast"), "FNM_DIR": str(root / "fnm"),
    }


def cmd_run(a: argparse.Namespace) -> None:
    root = Path(a.root).resolve()
    out = Path(a.out).resolve()
    (out / "raw").mkdir(parents=True, exist_ok=True)
    p = pins()
    benches = matrix(root, p, a.quick)
    if a.filter:
        rx = re.compile(a.filter)
        keep = {b.id for b in benches if rx.search(b.id)}
        keep |= {b.baseline for b in benches if b.id in keep and b.baseline}  # derived numbers need baselines
        benches = [b for b in benches if b.id in keep]
    env = run_env(root)
    meta = environment(root, a)
    meta["load_before"] = loadavg()
    meta["ncpu_usable"] = len(os.sched_getaffinity(0)) if hasattr(os, "sched_getaffinity") else meta["ncpu"]
    if noisy(meta):
        print(f"WARNING: 1-min load {meta['load_before'][0]} > ncpu/4 ({meta['ncpu'] / 4}); results will be marked noisy",
              file=sys.stderr)

    server = None
    if any(b.group == "install" for b in benches):
        server = subprocess.Popen([sys.executable, "-m", "http.server", str(MIRROR_PORT), "--bind", "127.0.0.1",
                                   "-d", str(root / "mirror")], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        time.sleep(1)
    try:
        print(f"checking {len(benches)} benchmarks measure what they claim ...")
        for b in benches:
            check(b, env)
            if b.prepare:
                subprocess.run(b.prepare, shell=True, env=env)
            t0 = time.monotonic()
            subprocess.run(b.cmd, env=env, capture_output=True)
            b.slow = b.slow or time.monotonic() - t0 > SLOW_RUN_S
        meta["benchmarks"] = [asdict(b) for b in benches]
        rng = random.Random(a.seed)
        meta["rounds_log"] = []
        for r in range(a.rounds):
            order = benches[:]
            rng.shuffle(order)  # rotate order so a load spike never lands on the same benchmark every round
            meta["rounds_log"].append({"round": r, "load_before": loadavg(), "order": [b.id for b in order]})
            for i, b in enumerate(order, 1):
                print(f"[round {r + 1}/{a.rounds} {i}/{len(order)}] {b.id}", flush=True)
                dest = out / "raw" / f"r{r}" / (b.id.replace("/", "__") + ".json")
                dest.parent.mkdir(parents=True, exist_ok=True)
                hf = [HYPERFINE, "-N", "--style", "none", "--export-json", str(dest), "-n", b.id]
                if b.slow:
                    hf += ["--warmup", "1", "--min-runs", "3" if a.quick else "5", "--max-runs", "10" if a.quick else "20"]
                else:
                    hf += ["--warmup", "5" if a.quick else "10", "--min-runs", "20" if a.quick else "50",
                           "--max-runs", "60" if a.quick else "300"]
                if b.prepare:
                    hf += ["--prepare", b.prepare]
                hf.append(shlex.join(b.cmd))
                subprocess.run(hf, env=env, check=True, stdout=subprocess.DEVNULL)
            meta["rounds_log"][-1]["load_after"] = loadavg()
    finally:
        if server:
            server.terminate()
    meta["finished"] = time.strftime("%Y-%m-%dT%H:%M:%S%z")
    meta["load_after"] = loadavg()
    meta["noisy"] = noisy(meta)
    (out / "meta.json").write_text(json.dumps(meta, indent=2))
    report(out)


# --------------------------------------------------------------------------------------------- report


def pct(xs: list[float], q: float) -> float:
    xs = sorted(xs)
    return xs[min(len(xs) - 1, int(q * len(xs)))]


def summarize(out: Path) -> dict[str, dict]:
    stats: dict[str, dict] = {}
    for f in sorted((out / "raw").glob("r*/*.json")):
        for res in json.loads(f.read_text())["results"]:
            s = stats.setdefault(res["command"], {"times": [], "medians": []})
            s["times"] += res["times"]
            s["medians"].append(res["median"])
    for s in stats.values():
        t = s["times"]
        med = statistics.median(t)
        s.update(n=len(t), min=min(t), p10=pct(t, 0.10), median=med,
                 mad=statistics.median([abs(x - med) for x in t]),
                 median_of_rounds=statistics.median(s["medians"]),
                 round_spread=(max(s["medians"]) - min(s["medians"])) / med if med else 0.0)
        del s["times"]
    return stats


def cmd_report(a: argparse.Namespace) -> None:
    report(Path(a.out))


def report(out: Path) -> None:
    meta = json.loads((out / "meta.json").read_text())
    stats = summarize(out)
    benches = {b["id"]: b for b in meta["benchmarks"]}
    derived: dict[str, dict] = {}
    for bid, b in benches.items():
        s = stats.get(bid)
        if not s:
            continue
        base = stats.get(b["baseline"]) if b["baseline"] else None
        if base:
            ops = b["ops"] or 1
            derived[bid] = {"min": (s["min"] - base["min"]) / ops,
                            "median": (s["median_of_rounds"] - base["median_of_rounds"]) / ops,
                            # run-to-run spread of the difference: what the per-op median cannot resolve
                            "noise": (s["mad"] + base["mad"]) / ops}
    (out / "summary.json").write_text(json.dumps({"stats": stats, "derived": derived}, indent=2))

    def ms(x: float | None) -> str:
        if x is None:
            return "–"
        return f"{x * 1000:.2f}" if abs(x) < 0.1 else f"{x * 1000:.0f}"
    L = [f"# Benchmark report ({meta['where']}, {meta['started']})", "",
         f"- git `{meta['git']}`, seed {meta['seed']}, {meta['rounds']} rounds{' (quick)' if meta['quick'] else ''}",
         f"- {meta['cpu']}, {meta['ncpu']} CPUs, {meta['platform']}",
         f"- cgroup: {meta['cgroup'] or 'none'}",
         f"- {meta['versions']['hyperfine']}; bash `{meta['versions']['bash']}`; `{meta['versions']['zsh']}`; "
         f"fnm `{meta['versions']['fnm']}`; nvm {meta['pins']['NVM_VERSION']}; nvm-fast `{meta['versions']['nvm-fast']}`",
         f"- load average before/after: {meta['load_before']} / {meta['load_after']}"
         + ("  **NOISY: load exceeded ncpu/4; trust min and p10, not medians**" if noisy(meta) else ""), "",
         "Per-operation numbers are loop totals minus the same shell's startup, divided by the operation count; "
         "`±` is the summed MAD of both, per operation: differences below it are not resolved. "
         "`min` is the min over every run of every round; `median` is the median of the per-round medians. "
         "Times in ms.", ""]

    def table(title: str, ids: list[str], use_derived: bool, note: str = "") -> None:
        rows = [i for i in ids if i in stats]
        if not rows:
            return
        L.extend([f"## {title}", ""] + ([note, ""] if note else []))
        if use_derived:
            L.append("| benchmark | per-op min | per-op median | ± | total min | total median | runs | round spread |")
            L.append("|---|---:|---:|---:|---:|---:|---:|---:|")
            for i in rows:
                s, d = stats[i], derived.get(i, {})
                L.append(f"| `{i}` | {ms(d.get('min'))} | {ms(d.get('median'))} | {ms(d.get('noise'))} "
                         f"| {ms(s['min'])} | {ms(s['median_of_rounds'])} | {s['n']} | {s['round_spread'] * 100:.0f}% |")
        else:
            L.append("| benchmark | min | p10 | median | MAD | runs | round spread |")
            L.append("|---|---:|---:|---:|---:|---:|---:|")
            for i in rows:
                s = stats[i]
                L.append(f"| `{i}` | {ms(s['min'])} | {ms(s['p10'])} | {ms(s['median_of_rounds'])} | {ms(s['mad'])} "
                         f"| {s['n']} | {s['round_spread'] * 100:.0f}% |")
        L.append("")

    by = lambda g: [i for i, b in benches.items() if b["group"] == g]
    table("Shell startup overhead (config minus bare shell)", by("startup"), True,
          "`nvm-cd`/`fast-cd` add upstream's README auto-switch recipe; `fast-autocd` uses nvm-fast's one-process "
          "hook; `fnm` is `fnm env --use-on-cd`.")
    for g, title in (("use", "`use` (switch between two installed versions)"), ("current", "`current`"),
                     ("version", "`version <major>` (resolution)"), ("ls-many", "`ls` with 102 versions installed"),
                     ("cd-switch", "`cd` into/out of a project with `.nvmrc` (switches each time)"),
                     ("cd-nochange", "`cd` between directories without a version file"),
                     ("install", "install from a loopback mirror (download + verify + extract)")):
        table(title, by(g), g != "install")
    table("Bare process latency", by("process"), False)
    (out / "report.md").write_text("\n".join(L) + "\n")
    print("\n".join(L))


# --------------------------------------------------------------------------------------------- profile


def cmd_profile(a: argparse.Namespace) -> None:
    root = Path(a.root).resolve()
    out = Path(a.out).resolve()
    out.mkdir(parents=True, exist_ok=True)
    env = run_env(root)
    p = pins()
    bmaj = p["NODE_B"].lstrip("v").split(".")[0]
    L = ["# Profile", ""]
    core = root / "fast/nvm-core"
    L += ["## Binaries", "", "```", capture(["ls", "-l", str(core), str(root / "fast/nvm-fetch"), str(root / "bin/fnm")])]
    if platform.system() == "Darwin":
        L += [capture(["otool", "-L", str(core)]), capture(["otool", "-L", str(root / "bin/fnm")])]
    else:
        L += [capture(["file", str(core)]), capture(["ldd", str(core)])]
    L += ["```", ""]

    cases = {
        "nvm: source + use": shell_cmd("bash", "nvm-nouse", root, f"nvm use {bmaj} >/dev/null"),
        "nvm: source + ls (102 versions)": shell_cmd("bash", "nvm-many", root, "nvm ls >/dev/null"),
        "nvm: startup with auto-use": shell_cmd("bash", "nvm", root, "exit"),
        "fast: source + use": shell_cmd("bash", "fast-nouse", root, f"nvm use {bmaj} >/dev/null"),
        "fast: source + ls (102 versions)": shell_cmd("bash", "fast-many", root, "nvm ls >/dev/null"),
        "fast: startup with auto-use": shell_cmd("bash", "fast", root, "exit"),
        "fnm: startup (env --use-on-cd)": shell_cmd("bash", "fnm", root, "exit"),
        "fnm: startup + use": shell_cmd("bash", "fnm", root, f"fnm use {bmaj} >/dev/null"),
        "bare bash": shell_cmd("bash", "bare", root, "exit"),
    }
    if platform.system() == "Linux" and shutil.which("strace"):
        L += ["## Processes and syscalls (strace -f, bash)", "",
              "| case | execve | forks | syscalls |", "|---|---:|---:|---:|"]
        for name, cmd in cases.items():
            log = out / "strace.tmp"
            subprocess.run(["strace", "-f", "-qq", "-o", str(log), *cmd], env=env, capture_output=True)
            text = log.read_text()
            execs = len(re.findall(r"\bexecve\(.*= 0$", text, re.M))
            forks = len(re.findall(r"\b(clone3?|fork|vfork)\(.*= [1-9]", text, re.M))
            L.append(f"| {name} | {execs} | {forks} | {len(text.splitlines())} |")
            log.unlink()
        L.append("")
        exe = out / "exec.tmp"
        subprocess.run(["strace", "-f", "-qq", "-e", "trace=execve", "-o", str(exe),
                        *cases["nvm: source + use"]], env=env, capture_output=True)
        progs: dict[str, int] = {}
        for m in re.finditer(r'execve\("([^"]+)"', exe.read_text()):
            k = Path(m.group(1)).name
            progs[k] = progs.get(k, 0) + 1
        exe.unlink()
        L += ["Programs executed by upstream `nvm use`: " +
              ", ".join(f"{k} {v}" for k, v in sorted(progs.items(), key=lambda x: -x[1])), ""]
        c = subprocess.run(["strace", "-c", "-f", str(core), "__auto", "use"], env=env, capture_output=True, text=True)
        L += ["## `nvm-core __auto use` syscall summary", "", "```", c.stderr.strip()[-3000:], "```", ""]
    if platform.system() == "Linux" and shutil.which("perf"):
        r = subprocess.run(["perf", "stat", "-r", "50", "-e", "task-clock,context-switches,page-faults",
                            str(core), "current"], env=env, capture_output=True, text=True)
        L += ["## perf stat (`nvm-core current`, 50 runs)", "", "```", (r.stderr or "perf unavailable").strip(), "```", ""]
    # Peak RSS
    tbin = "/usr/bin/time"
    if Path(tbin).exists():
        flag = "-l" if platform.system() == "Darwin" else "-v"
        L += ["## Peak RSS", "", "| command | max RSS |", "|---|---:|"]
        for name, cmd in (("nvm-core current", [str(core), "current"]),
                          ("nvm-core ls (102)", [str(root / "fast-many/nvm-core"), "ls"]),
                          ("fnm ls", [str(root / "bin/fnm"), "ls"]),
                          ("fnm ls (102)", [str(root / "bin/fnm"), "ls", "--fnm-dir", str(root / "fnm-many")])):
            r = subprocess.run([tbin, flag, *cmd], env={**env, "NVM_DIR": str(Path(cmd[0]).parent)},
                               capture_output=True, text=True)
            m = re.search(r"(\d+)\s+maximum resident set size", r.stderr) or re.search(
                r"Maximum resident set size \(kbytes\): (\d+)", r.stderr)
            kb = int(m.group(1)) // (1024 if platform.system() == "Darwin" else 1) if m else -1
            L.append(f"| {name} | {kb} KB |")
        L.append("")
    (out / "profile.md").write_text("\n".join(L) + "\n")
    print("\n".join(L))


# --------------------------------------------------------------------------------------------- main


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    s = sub.add_parser("setup")
    s.add_argument("--root", required=True)
    s.add_argument("--nvm-src", required=True)
    s.add_argument("--fast-src", required=True)
    s.add_argument("--fnm", required=True)
    s.add_argument("--dist", required=True)
    r = sub.add_parser("run")
    r.add_argument("--root", required=True)
    r.add_argument("--out", required=True)
    r.add_argument("--rounds", type=int, default=3)
    r.add_argument("--quick", action="store_true")
    r.add_argument("--filter", default="")
    r.add_argument("--seed", type=int, default=20261022)
    r.add_argument("--bash", default="bash")
    r.add_argument("--zsh", default="zsh")
    rp = sub.add_parser("report")
    rp.add_argument("out")
    pr = sub.add_parser("profile")
    pr.add_argument("--root", required=True)
    pr.add_argument("--out", required=True)
    a = ap.parse_args()
    if a.cmd == "run":
        SHELLS.update(bash=a.bash, zsh=a.zsh)
    {"setup": cmd_setup, "run": cmd_run, "report": cmd_report, "profile": cmd_profile}[a.cmd](a)


if __name__ == "__main__":
    main()
