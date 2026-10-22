//! `install` and `uninstall` (§6.1, §6.2): binary and source installs, install locks, interrupts.
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{env, fs};

use crate::store::{ensure_v, parse_ver};
use crate::{
    cmd_alias, cmd_unalias, cmd_use, lts_filter, npm, print_alias, rc_version, remote,
    walk_aliases, Ctx, HELP_HINT,
};

pub fn cmd_install(cx: &mut Ctx, rest: &[String]) -> i32 {
    let mut flags_lts: Option<String> = None;
    let mut set_default = false;
    let mut alias: Option<String> = None;
    let mut save = false;
    let mut passthru: Vec<String> = vec![];
    let mut skip_pkgs = false;
    let mut version: Option<String> = None;
    let mut bin_only = false;
    let mut src_only = false;
    let mut latest_npm = false;
    let mut reinstall_from: Option<String> = None;
    let mut jobs: Option<usize> = env::var("NVM_MAKE_JOBS").ok().and_then(|v| v.parse().ok());
    let mut extra: Vec<String> = vec![];
    let mut it = rest.iter().peekable();
    while let Some(a) = it.next() {
        match a.as_str() {
            "-j" => {
                let n = it.next().cloned().unwrap_or_default();
                jobs = make_jobs(Some(&n)).or(jobs);
                if let Some(j) = jobs {
                    cx.ev.export("NVM_MAKE_JOBS", &j.to_string());
                }
            }
            "--reinstall-packages-from" | "--copy-packages-from" => {
                eprintln!(
                    "If {a} is provided, it must point to an installed version of node using `=`."
                );
                return 6;
            }
            x if x.starts_with("--reinstall-packages-from=")
                || x.starts_with("--copy-packages-from=") =>
            {
                let (flag, val) = x.split_once('=').expect("guard matched a `--flag=` prefix");
                if reinstall_from.is_some() {
                    eprintln!(
                        "--reinstall-packages-from may not be provided more than once{}",
                        if x.starts_with("--copy") {
                            ", or combined with `--copy-packages-from`"
                        } else {
                            ""
                        }
                    );
                    return 6;
                }
                if val.is_empty() {
                    eprintln!(
                        "If {flag} is provided, it must point to an installed version of node."
                    );
                    return 6;
                }
                reinstall_from = Some(val.to_string());
            }
            "--latest-npm" => latest_npm = true,
            "--lts" if version.is_none() => flags_lts = Some("*".into()),
            "--default" => {
                if set_default || alias.is_some() {
                    eprintln!("--default and --alias are mutually exclusive, and may not be provided more than once");
                    return 6;
                }
                set_default = true;
            }
            "--save" | "-w" => save = true,
            "--skip-default-packages" => skip_pkgs = true,
            "--no-progress" => passthru.push(a.clone()),
            "--offline" => passthru.push(a.clone()),
            "-b" => bin_only = true,
            "-s" => src_only = true,
            x if x.starts_with("--lts=") => {
                flags_lts = Some(x[6..].to_string()).filter(|n| !n.is_empty());
            }
            x if x.starts_with("--alias=") => {
                if alias.is_some() || set_default {
                    eprintln!("--default and --alias are mutually exclusive, and may not be provided more than once");
                    return 6;
                }
                alias = Some(x[8..].to_string());
            }
            x if x.starts_with("---") => {
                eprintln!("arguments with `---` are not supported - this is likely a typo");
                return 55;
            }
            x if x.starts_with('-') && version.is_none() => {}
            x if version.is_none() => version = Some(x.to_string()),
            x => extra.push(x.to_string()),
        }
    }
    if bin_only && src_only {
        eprintln!("-s and -b cannot be set together since they would skip install from both binary and source");
        return 6;
    }
    // NVM_NO_SOURCE_FALLBACK=1 (exactly) behaves like `-b` on every install.
    if !bin_only && env::var("NVM_NO_SOURCE_FALLBACK").as_deref() == Ok("1") {
        if src_only {
            eprintln!("-s cannot be combined with NVM_NO_SOURCE_FALLBACK=1 since that would skip install from both binary and source");
            return 6;
        }
        bin_only = true;
    }
    if set_default && alias.is_some() {
        eprintln!(
            "--default and --alias are mutually exclusive, and may not be provided more than once"
        );
        return 6;
    }
    if version.is_none() {
        match &flags_lts {
            Some(n) if n == "*" => println!("Installing latest LTS version."),
            Some(n) => println!("Installing with latest version of LTS line: {n}"),
            None => match rc_version(false) {
                Ok(v) => version = Some(v),
                Err(()) => {
                    eprintln!("Usage: nvm install [<version>]\n  Provide a <version>, or run from a directory containing an .nvmrc file.\n  {HELP_HINT}");
                    return 127;
                }
            },
        }
    }
    // upstream turns an `lts/<name>` version (argument or .nvmrc) into an LTS filter only after the messages above
    if let Some(n) = version.as_deref().and_then(|v| v.strip_prefix("lts/")) {
        flags_lts = Some(n.to_string());
        version = None;
    }
    let lts_for_resolve = match &flags_lts {
        Some(n) if n != "*" => match remote::lts_name(&cx.st, n) {
            Ok(x) => Some(x),
            Err((m, rc)) => {
                eprintln!("{m}");
                if rc == 3 {
                    let n = flags_lts.clone().unwrap_or_default();
                    eprintln!("Version with LTS filter '{n}' not found - try `nvm ls-remote --lts={n}` to browse available versions.");
                }
                return rc;
            }
        },
        o => o.clone(),
    };
    let offline = passthru.iter().any(|a| a == "--offline");
    if offline && version.is_none() {
        if let Some(n) = &flags_lts {
            if cx.st.read_alias(&format!("lts/{n}")).is_none() {
                eprintln!("LTS alias '{n}' not found locally. Run `nvm ls-remote --lts` first to populate LTS aliases.");
                return 3;
            }
        }
    }
    let chosen: Option<String> = if offline {
        let pat = version.clone().unwrap_or_else(|| "lts/*".into());
        if let Ok(v) = cx.st.version(&pat) {
            Some(v)
        } else {
            let p = ensure_v(&pat);
            let pfx = format!("{}.", p.trim_end_matches('.'));
            cx.st
                .cached()
                .into_iter()
                .rev()
                .map(|x| x.1)
                .find(|n| format!("{n}.").starts_with(&pfx))
        }
    } else {
        let rows = match remote::index(&cx.st) {
            Ok(r) => r,
            Err(c) => return c,
        };
        remote::resolve(
            &cx.st,
            &rows,
            version.as_deref().unwrap_or("node"),
            lts_for_resolve.as_deref(),
        )
        .map(|r| r.ver.clone())
    };
    let Some(ver) = chosen else {
        let (f1, f2) = match &flags_lts {
            None => (String::new(), String::new()),
            Some(n) if n == "*" => (" (with LTS filter) ".into(), " --lts".into()),
            Some(n) => (format!(" (with LTS filter '{n}') "), format!(" --lts={n}")),
        };
        match &version {
            Some(v) if offline => eprintln!("Version '{v}' {}not found locally or in cache - try `nvm ls{f2}` to browse available versions.", f1.trim_start()),
            Some(v) => eprintln!("Version '{v}' {}not found - try `nvm ls-remote{f2}` to browse available versions.", f1.trim_start()),
            None => eprintln!("Version with LTS filter '{}' not found - try `nvm ls-remote{f2}` to browse available versions.", flags_lts.unwrap_or_default()),
        }
        return 3;
    };
    let typed = version.unwrap_or_else(|| {
        if flags_lts.as_deref() == Some("*") {
            "lts/*".into()
        } else {
            format!("lts/{}", flags_lts.clone().unwrap_or_default())
        }
    });

    if let Some(p) = &reinstall_from {
        match cx.st.version(p) {
            Ok(v) if cx.st.is_installed(&v) => {
                if v == ver && !cx.st.is_installed(&ver) {
                    eprintln!("You can't reinstall global packages from the same version of node you're installing.");
                    return 4;
                }
            }
            _ => {
                eprintln!("If --reinstall-packages-from is provided, it must point to an installed version of node.");
                return 5;
            }
        }
    }
    let valid = |cx: &Ctx| cx.st.is_installed(&ver);
    let mut rc = 0;
    let was_installed = valid(cx);
    if was_installed {
        eprintln!("{ver} is already installed.");
    } else {
        let mut hooked = false;
        if let Ok(hook) = env::var("NVM_INSTALL_THIRD_PARTY_HOOK") {
            if !hook.is_empty() {
                println!("** $NVM_INSTALL_THIRD_PARTY_HOOK env var set; dispatching to third-party installation method **");
                let vd = cx.st.version_dir(&ver);
                let st = Command::new(&hook)
                    .args([
                        ver.as_str(),
                        "node",
                        "std",
                        if src_only { "source" } else { "binary" },
                    ])
                    .arg(&vd)
                    .status();
                let code = st.ok().and_then(|s| s.code()).unwrap_or(1);
                if code != 0 {
                    eprintln!("*** Third-party $NVM_INSTALL_THIRD_PARTY_HOOK env var failed to install! ***");
                    return code;
                }
                if !cx.st.is_installed(&ver) {
                    eprintln!("*** Third-party $NVM_INSTALL_THIRD_PARTY_HOOK env var claimed to succeed, but failed to install! ***");
                    return 33;
                }
                hooked = true;
            }
        }
        if !hooked {
            let _lock = match InstallLock::acquire(cx, &ver) {
                Ok(l) => l,
                Err(c) => return c,
            };
            // Ctrl-C reaches the whole foreground process group. The children (nvm-fetch, make) clean up and
            // exit; nvm-core survives the signal so the lock above is released instead of going stale.
            let _ = ctrlc::set_handler(|| INTERRUPTED.store(true, Ordering::SeqCst));
            let interrupted = || INTERRUPTED.load(Ordering::SeqCst);
            let binary_ok = parse_ver(&ver).is_some_and(|v| v >= (0, 8, 6));
            let mut code: i32 = -1;
            if !src_only && binary_ok {
                println!("Downloading and installing node {ver}...");
                code = fetch(cx, &ver, Some(&cx.st.version_dir(&ver)), &passthru);
                if interrupted() {
                    return 130;
                }
                if code != 0 {
                    if bin_only {
                        eprintln!("Binary download failed. Download from source aborted.");
                        return 2;
                    }
                    eprintln!("Binary download failed, trying source.");
                }
            } else if bin_only {
                eprintln!("Binary download is not available for {ver}");
                code = 3;
            }
            if code != 0 && !bin_only {
                let j = match jobs {
                    Some(j) => j,
                    None => make_jobs(None).unwrap_or(1),
                };
                // upstream keeps the result in a shell variable, so later installs in this shell skip detection
                cx.ev.export("NVM_MAKE_JOBS", &j.to_string());
                code = build_from_source(cx, &ver, j, &extra, &passthru);
                if interrupted() {
                    return 130;
                }
            }
            if code != 0 {
                return code;
            }
        }
        if !valid(cx) {
            eprintln!(
                "The install of {ver} reported success but failed verification; not activating it."
            );
            return 1;
        }
    }
    let reinstall_src = reinstall_from.as_ref().and_then(|p| cx.st.version(p).ok());
    let r = cmd_use(cx, std::slice::from_ref(&ver));
    if r != 0 {
        rc = r;
    }
    let bindir = cx.st.version_dir(&ver).join("bin");
    let ensure_default = |cx: &mut Ctx| {
        if cx.st.read_alias("default").is_none() {
            let _ = fs::create_dir_all(cx.st.dir.join("alias"));
            if fs::write(cx.st.alias_path("default"), format!("{typed}\n")).is_ok() {
                print!("Creating default alias: ");
                print_alias(cx, "default", &typed);
            }
        }
    };
    if !was_installed {
        ensure_default(cx);
    }
    if latest_npm {
        let c = npm::install_latest(&ver, Some(&bindir));
        if c != 0 {
            rc = c;
        }
    }
    if !skip_pkgs {
        let c = npm::install_default_packages(&cx.st.dir, &bindir);
        if c != 0 {
            rc = c;
        }
    }
    if let Some(src) = reinstall_src {
        if src == ver {
            eprintln!("Can not reinstall packages from the current version of node.");
            rc = 2;
        } else {
            npm::reinstall(&src, &cx.st.version_dir(&src).join("bin"), Some(&bindir));
        }
    }
    if was_installed {
        ensure_default(cx);
    }
    if set_default {
        let _ = fs::write(cx.st.alias_path("default"), format!("{typed}\n"));
        print_alias(cx, "default", &typed);
    }
    if save {
        let _ = fs::write(
            env::current_dir().unwrap_or_default().join(".nvmrc"),
            format!("{ver}\n"),
        );
        println!("Wrote version number ({ver}) to .nvmrc");
    }
    if let Some(a) = alias {
        let _ = cmd_alias(cx, &[a, typed.clone()]);
    }
    rc
}

/// Runs `nvm-fetch install`: download, verify and extract `ver`, then move it to `dest`
/// (binary install) or leave it in the source cache (`dest` = `None`). Returns the exit code.
fn fetch(cx: &Ctx, ver: &str, dest: Option<&Path>, passthru: &[String]) -> i32 {
    let mut cmd = Command::new(remote::fetch_bin());
    cmd.args(["install", &remote::mirror(), ver, &cx.nvm_dir]);
    match dest {
        Some(d) => cmd.arg(format!("--dest={}", d.display())),
        None => cmd.arg("--source"),
    };
    cmd.args(passthru)
        .status()
        .ok()
        .and_then(|s| s.code())
        .unwrap_or(1)
}

pub fn cmd_uninstall(cx: &mut Ctx, rest: &[String]) -> i32 {
    if rest.len() != 1 {
        eprintln!("Usage: nvm uninstall <version>\n       nvm uninstall --lts\n       nvm uninstall --lts=<LTS name>\n  {HELP_HINT}");
        return 127;
    }
    if rest[0] == "--" {
        // upstream resolves `--` like any other pattern and gets an empty version back
        eprintln!("Version '' (inferred from --) is not installed.");
        return 0;
    }
    let (lts, pat) = match lts_filter(cx, rest) {
        Ok(x) => x,
        Err(c) => return c,
    };
    let arg = match (&lts, &pat) {
        (_, Some(p)) => p.clone(),
        (Some(n), None) => format!("lts/{}", if n == "*" { "*" } else { n }),
        _ => {
            eprintln!("Usage: nvm uninstall <version>\n       nvm uninstall --lts\n       nvm uninstall --lts=<LTS name>\n  {HELP_HINT}");
            return 127;
        }
    };
    let arg = if lts.is_some() && pat.is_none() {
        arg
    } else {
        pat.clone().unwrap_or(arg)
    };
    let shown = if pat.is_none() {
        rest.iter()
            .find(|a| a.starts_with("--lts"))
            .cloned()
            .unwrap_or_else(|| arg.clone())
    } else {
        arg.clone()
    };
    let Ok(ver) = cx.st.version(&arg) else {
        eprintln!("Version '{shown}' is not installed.");
        return 0;
    };
    if !cx.st.is_installed(&ver) {
        // as above: upstream echoes the raw argument (`PATTERN`), not the normalized form
        eprintln!("Version '{ver}' (inferred from {shown}) is not installed.");
        return 0;
    }
    if cx.st.ls_current(&env::var("PATH").unwrap_or_default()) == ver {
        // upstream prints the argument as typed here (`PATTERN`), not the normalized `lts/…` form
        let what = if ver.starts_with("iojs-") {
            "io.js"
        } else {
            "node"
        };
        eprintln!(
            "nvm: Cannot uninstall currently-active {what} version, {ver} (inferred from {shown})."
        );
        return 1;
    }
    let _ = fs::remove_dir_all(cx.st.version_dir(&ver));
    let (flavour, num) = match ver.strip_prefix("iojs-") {
        Some(b) => ("iojs", b),
        None => ("node", ver.as_str()),
    };
    let _ = fs::remove_dir_all(cx.st.dir.join(format!(".cache/src/{flavour}-{num}/files")));
    if let Ok(rd) = fs::read_dir(cx.st.dir.join(".cache/bin")) {
        for e in rd.flatten() {
            if e.file_name()
                .to_string_lossy()
                .starts_with(&format!("{flavour}-{num}-"))
            {
                let _ = fs::remove_dir_all(e.path().join("files"));
            }
        }
    }
    match ver.strip_prefix("iojs-") {
        Some(b) => println!("Uninstalled io.js {b}"),
        None => println!("Uninstalled node {ver}"),
    }
    // upstream: `for a in $(nvm_grep -l "${VERSION}" "$(nvm_alias_path)"/*)` — a non-recursive,
    // top-level glob. Alias files under `alias/lts/` sit in a subdirectory and must survive
    // uninstall, even when they name the removed version. Content matching (comments included)
    // is upstream's own substring grep and stays as-is.
    let mut names = vec![];
    walk_aliases(&cx.st.dir.join("alias"), "", &mut names);
    names.sort();
    for n in names {
        if n.contains('/') {
            continue;
        }
        if fs::read_to_string(cx.st.alias_path(&n)).is_ok_and(|c| c.contains(&ver)) {
            cmd_unalias(cx, &[n]);
        }
    }
    0
}

static INTERRUPTED: AtomicBool = AtomicBool::new(false);

/// Per-version advisory install lock (`mkdir` is atomic), removed on drop (§6.1 step 2).
struct InstallLock(Option<PathBuf>);

impl InstallLock {
    fn acquire(cx: &Ctx, ver: &str) -> Result<InstallLock, i32> {
        let root = cx.st.dir.join(".cache/locks");
        if fs::create_dir_all(&root).is_err() {
            return Ok(InstallLock(None));
        }
        let name: String = ver
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || "._+-".contains(c) {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let lock = root.join(name);
        let timeout: u64 = env::var("NVM_INSTALL_LOCK_TIMEOUT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(600);
        let stale: u64 = env::var("NVM_INSTALL_LOCK_STALE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let (mut waited, mut announced) = (0u64, false);
        while fs::create_dir(&lock).is_err() {
            if stale > 0 {
                let old = fs::metadata(&lock)
                    .and_then(|m| m.modified())
                    .ok()
                    .and_then(|t| t.elapsed().ok())
                    .is_some_and(|a| a.as_secs() > stale * 60);
                if old {
                    eprintln!(
                        "Removing stale install lock for {ver} (older than {stale} minute(s))"
                    );
                    let _ = fs::remove_dir_all(&lock);
                    continue;
                }
            }
            if waited >= timeout {
                eprintln!("Timed out after {timeout}s waiting for another install of {ver} to finish.\nIf no other install is running, remove {} and try again.", lock.display());
                return Err(1);
            }
            if !announced {
                eprintln!("Waiting for another install of {ver} to finish...");
                announced = true;
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
            waited += 1;
        }
        Ok(InstallLock(Some(lock)))
    }
}

impl Drop for InstallLock {
    fn drop(&mut self) {
        if let Some(p) = &self.0 {
            let _ = fs::remove_dir(p).or_else(|_| fs::remove_dir_all(p));
        }
    }
}

/// `nvm_get_make_jobs`: explicit count, or detect cores. Prints upstream's messages.
fn make_jobs(arg: Option<&str>) -> Option<usize> {
    if let Some(a) = arg {
        if let Ok(n) = a.parse::<usize>() {
            if n > 0 {
                println!("number of `make` jobs: {n}");
                return Some(n);
            }
        }
        if !a.is_empty() {
            eprintln!("{a} is invalid for number of `make` jobs, must be a natural number");
        }
    }
    let cores: Option<usize> = if cfg!(target_os = "linux") {
        fs::read_to_string("/proc/cpuinfo").ok().map(|t| {
            t.lines()
                .filter(|l| {
                    l.starts_with("processor")
                        && l.split(':')
                            .nth(1)
                            .is_some_and(|v| v.trim().parse::<u32>().is_ok())
                })
                .count()
        })
    } else if cfg!(target_os = "macos") {
        Command::new("sysctl")
            .args(["-n", "hw.ncpu"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse().ok())
    } else {
        std::thread::available_parallelism()
            .ok()
            .map(std::num::NonZero::get)
    };
    match cores.filter(|c| *c > 0) {
        None => {
            eprintln!("Can not determine how many core(s) are available, running in single-threaded mode.\nPlease report an issue on GitHub to help us make nvm run faster on your computer!");
            Some(1)
        }
        Some(c) => {
            println!("Detected that you have {c} CPU core(s)");
            if c > 2 {
                println!("Running with {} threads to speed up the build", c - 1);
                Some(c - 1)
            } else {
                println!("Number of CPU core(s) less than or equal to 2, running in single-threaded mode");
                Some(1)
            }
        }
    }
}

fn clang_version() -> Option<(u64, u64)> {
    let o = Command::new("clang").arg("--version").output().ok()?;
    let t = String::from_utf8_lossy(&o.stdout).into_owned();
    let line = t.lines().next()?;
    let w: Vec<&str> = line.split_whitespace().collect();
    let i = w.iter().position(|x| *x == "version")?;
    let mut p = w.get(i + 1)?.split(['.', '-']);
    Some((
        p.next()?.parse().ok()?,
        p.next().and_then(|x| x.parse().ok()).unwrap_or(0),
    ))
}

/// Source install (§6.1.2): download+verify the source tarball, then configure/make/install. Returns the exit code.
fn build_from_source(
    cx: &mut Ctx,
    ver: &str,
    jobs: usize,
    extra: &[String],
    passthru: &[String],
) -> i32 {
    let extra_s = extra.join(" ");
    if !extra_s.is_empty() {
        // upstream accumulates the parameters with a leading space
        println!("Additional options while compiling:  {extra_s}");
    }
    let mut make = String::from("make");
    if parse_ver(ver).is_some_and(|v| v < (0, 12, 0)) {
        make.push_str(" SHELL=/bin/sh");
    }
    let mut make_cxx = String::new();
    if cfg!(target_os = "macos") {
        make_cxx = format!(
            "CC={} CXX={}",
            env::var("CC").unwrap_or_else(|_| "cc".into()),
            env::var("CXX").unwrap_or_else(|_| "c++".into())
        );
    }
    let has = |t: &str| {
        env::var("PATH")
            .unwrap_or_default()
            .split(':')
            .any(|d| Path::new(d).join(t).is_file())
    };
    if has("clang++")
        && has("clang")
        && clang_version().is_some_and(|v| v >= (3, 5))
        && (env::var("CC").is_err() || env::var("CXX").is_err())
    {
        println!(
            "Clang v3.5+ detected! CC or CXX not specified, will use Clang as C/C++ compiler!"
        );
        make_cxx = format!(
            "CC={} CXX={}",
            env::var("CC").unwrap_or_else(|_| "cc".into()),
            env::var("CXX").unwrap_or_else(|_| "c++".into())
        );
    }
    let c = fetch(cx, ver, None, passthru);
    if c != 0 {
        return c;
    }
    let files = cx
        .st
        .dir
        .join(".cache/src")
        .join(format!("node-{ver}"))
        .join("files");
    let vpath = cx.st.version_dir(ver);
    let _ = fs::create_dir_all(vpath.parent().unwrap_or(Path::new("/")));
    println!("$>./configure --prefix={} {extra_s}<", vpath.display());
    let run = |program: &str, args: Vec<String>| {
        Command::new(program)
            .args(args)
            .current_dir(&files)
            .status()
            .is_ok_and(|s| s.success())
    };
    let mut mk: Vec<String> = make.split_whitespace().map(String::from).collect();
    let mk_prog = mk.remove(0);
    let mut build = mk.clone();
    build.extend(["-j".to_string(), jobs.to_string()]);
    build.extend(make_cxx.split_whitespace().map(String::from));
    let mut install = build.clone();
    install.push("install".into());
    let mut cfg = vec![format!("--prefix={}", vpath.display())];
    cfg.extend(extra.iter().cloned());
    let ok = run("./configure", cfg) && run(&mk_prog, build) && {
        let _ = fs::remove_file(&vpath);
        run(&mk_prog, install)
    };
    if !ok {
        eprintln!("nvm: install {ver} failed!");
        let _ = fs::remove_dir_all(&files);
        return 1;
    }
    0
}
