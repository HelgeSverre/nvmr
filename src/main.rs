mod colors;
mod npm;
mod pathedit;
mod remote;
mod store;

use std::{env, fs, io::Write, path::PathBuf, process::exit};
use store::{ensure_v, find_nvmrc, parse_nvmrc, Store};

/// Collects shell statements for the eval channel (§1.1).
struct Eval {
    shell: String,
    buf: String,
    /// Values queued for export, so later steps in the same invocation see them (None = queued unset).
    vars: std::collections::HashMap<String, Option<String>>,
}

fn q(v: &str) -> String {
    format!("'{}'", v.replace('\'', "'\\''"))
}

impl Eval {
    /// Current value as the shell will see it after the queued statements run.
    fn get(&self, k: &str) -> Option<String> {
        match self.vars.get(k) {
            Some(v) => v.clone(),
            None => env::var(k).ok(),
        }
    }
    fn export(&mut self, k: &str, v: &str) {
        self.vars.insert(k.to_string(), Some(v.to_string()));
        if self.shell == "fish" {
            let val = if k.ends_with("PATH") {
                v.split(':')
                    .map(|p| format!("'{}'", p.replace('\\', "\\\\").replace('\'', "\\'")))
                    .collect::<Vec<_>>()
                    .join(" ")
            } else {
                format!("'{}'", v.replace('\\', "\\\\").replace('\'', "\\'"))
            };
            self.buf += &format!("set -gx {k} {val}\n");
        } else {
            self.buf += &format!("export {k}={}\n", q(v));
        }
    }
    fn unset(&mut self, k: &str) {
        self.vars.insert(k.to_string(), None);
        self.buf += &if self.shell == "fish" {
            format!("set -e {k}\n")
        } else {
            format!("unset {k}\n")
        };
    }
    fn rehash(&mut self) {
        self.buf += match self.shell.as_str() {
            "zsh" => "rehash\n",
            "fish" => "",
            _ => "hash -r\n",
        };
    }
}

struct Ctx {
    pal: Option<colors::Pal>,
    st: Store,
    ev: Eval,
    nvm_dir: String,
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let nvm_dir = env::var("NVM_DIR")
        .map(|d| d.trim_end_matches('/').to_string())
        .unwrap_or_else(|_| format!("{}/.nvm", env::var("HOME").unwrap_or_default()));
    let mut cx = Ctx {
        pal: colors::detect(&args),
        st: Store {
            dir: PathBuf::from(&nvm_dir),
        },
        ev: Eval {
            shell: env::var("NVM_SHELL").unwrap_or_else(|_| "bash".into()),
            buf: String::new(),
            vars: Default::default(),
        },
        nvm_dir,
    };
    let rc = run(&mut cx, &args);
    flush_eval(&cx.ev.buf);
    let _ = std::io::stdout().flush();
    exit(rc);
}

/// Writes queued shell statements to `$NVM_EVAL_FILE` (creating its directory), clears stale content when
/// there is nothing to write, and prunes eval files of long-dead shells from `$NVM_DIR/.cache`.
fn flush_eval(buf: &str) {
    let Ok(f) = env::var("NVM_EVAL_FILE") else {
        return;
    };
    let path = std::path::Path::new(&f);
    if buf.is_empty() {
        if fs::metadata(path).map(|m| m.len() > 0).unwrap_or(false) {
            let _ = fs::write(path, "");
        }
        return;
    }
    if let Some(d) = path.parent() {
        let _ = fs::create_dir_all(d);
        if d.file_name().map_or(false, |n| n == ".cache") {
            let day = std::time::Duration::from_secs(86_400);
            for e in fs::read_dir(d).into_iter().flatten().flatten() {
                let stale = e.file_name().to_string_lossy().starts_with("eval.")
                    && e.metadata()
                        .and_then(|m| m.modified())
                        .ok()
                        .and_then(|t| t.elapsed().ok())
                        .map_or(false, |a| a > day);
                if stale {
                    let _ = fs::remove_file(e.path());
                }
            }
        }
    }
    let _ = fs::write(path, buf);
}

const HELP: &str = include_str!("help.txt");
const HELP_HINT: &str = "Run `nvm --help` for full help.";

fn run(cx: &mut Ctx, args: &[String]) -> i32 {
    if args.is_empty() {
        print!("{HELP}");
        return 0;
    }
    for a in args.iter().take_while(|a| *a != "--") {
        if matches!(a.as_str(), "-h" | "help" | "--help") {
            print!("{HELP}");
            return 0;
        }
    }
    let rest = &args[1..];
    match args[0].as_str() {
        "--version" | "-v" => {
            println!("0.40.8");
            0
        }
        "current" => {
            println!(
                "{}",
                cx.st.ls_current(&env::var("PATH").unwrap_or_default())
            );
            0
        }
        "version" => cmd_version(cx, rest),
        "which" => cmd_which(cx, rest),
        "use" => cmd_use(cx, rest),
        "deactivate" => cmd_deactivate(cx, rest),
        "alias" => cmd_alias(cx, rest),
        "unalias" => cmd_unalias(cx, rest),
        "ls" | "list" => cmd_ls(cx, rest),
        "cache" => match rest.first().map(String::as_str) {
            Some("dir") => {
                println!("{}/.cache", cx.nvm_dir);
                0
            }
            Some("clear") => {
                let c = cx.st.dir.join(".cache");
                let ok = (!c.exists() || fs::remove_dir_all(&c).is_ok())
                    && fs::create_dir_all(&c).is_ok();
                if ok {
                    println!("nvm cache cleared.");
                    0
                } else {
                    eprintln!("Unable to clear nvm cache: {}", c.display());
                    1
                }
            }
            _ => usage_cache(),
        },
        "clear-cache" => {
            if let Ok(rd) = fs::read_dir(&cx.st.dir) {
                for e in rd.flatten() {
                    let n = e.file_name().to_string_lossy().into_owned();
                    if n.starts_with('v') && e.path().is_file() {
                        let _ = fs::remove_file(e.path());
                    }
                }
            }
            println!("nvm cache cleared.");
            0
        }
        "ls-remote" | "list-remote" => cmd_ls_remote(cx, rest),
        "version-remote" => cmd_version_remote(cx, rest),
        "install" | "i" => cmd_install(cx, rest),
        "uninstall" => cmd_uninstall(cx, rest),
        "exec" => cmd_exec(cx, rest),
        "run" => cmd_run(cx, rest),
        "set-colors" => {
            let a = rest.first().map(String::as_str).unwrap_or("");
            if !colors::valid_codes(a) {
                eprint!("{HELP}");
                if env::var("NVM_NO_COLORS").as_deref() == Ok("--no-colors")
                    || args.iter().any(|x| x == "--no-colors")
                {
                    eprintln!(
                        "\nPlease pass in five valid color codes. Choose from: rRgGbBcCyYmMkKeW"
                    );
                } else {
                    eprintln!("\x1b[1;37mPlease pass in five \x1b[1;31mvalid color codes\x1b[1;37m. Choose from: rRgGbBcCyYmMkKeW\x1b[0m");
                }
                return 0;
            }
            let c5: String = a.to_string();
            cx.ev.export("NVM_COLORS", &c5);
            println!(
                "Setting colors to: {}",
                c5.chars().map(String::from).collect::<Vec<_>>().join(" ")
            );
            if cx.pal.is_none() {
                println!(
                    "WARNING: Colors may not display because they are not supported in this shell."
                );
            }
            0
        }
        "unload" => {
            let _ = cmd_deactivate(cx, &["--silent".to_string()]);
            cx.ev.buf += &if cx.ev.shell == "fish" {
                "functions -e nvm\n".to_string()
            } else {
                "unset -f nvm nvm_find_up nvm_find_nvmrc nvm_ls_current nvm_version nvm_rc_version 2>/dev/null\n".to_string()
            };
            for v in [
                "NVM_NODEJS_ORG_MIRROR",
                "NVM_IOJS_ORG_MIRROR",
                "NVM_DIR",
                "NVM_CD_FLAGS",
                "NVM_BIN",
                "NVM_INC",
                "NVM_MAKE_JOBS",
                "NVM_INSTALL_LOCK",
                "NVM_COLORS",
                "NVM_CORE",
            ] {
                cx.ev.unset(v);
            }
            0
        }
        "install-latest-npm" => {
            if !rest.is_empty() {
                eprintln!("Usage: nvm install-latest-npm\n  {HELP_HINT}");
                return 127;
            }
            let cur = cx.st.ls_current(&env::var("PATH").unwrap_or_default());
            let nv = if cur == "system" {
                system_node(cx).map(|x| x.1).unwrap_or_default()
            } else {
                cur.clone()
            };
            if cur == "none" {
                println!("Attempting to upgrade to the latest working version of npm...");
                println!("Detected node version none, npm version v");
                eprintln!("Unable to obtain node version.");
                return 1;
            }
            npm::install_latest(&nv, None)
        }
        "reinstall-packages" | "copy-packages" => {
            if rest.len() != 1 {
                eprintln!("Usage: nvm {} <version>\n  {HELP_HINT}", args[0]);
                return 127;
            }
            let cur = cx.st.ls_current(&env::var("PATH").unwrap_or_default());
            if rest[0] == cur || cx.st.version(&rest[0]).as_deref() == Ok(cur.as_str()) {
                eprintln!("Can not reinstall packages from the current version of node.");
                return 2;
            }
            match cx.st.version(&rest[0]) {
                Ok(v) if cx.st.is_installed(&v) => {
                    npm::reinstall(&v, &cx.st.version_dir(&v).join("bin"), None);
                    0
                }
                _ if rest[0] == "system" => {
                    eprintln!("No system version of node or io.js detected.");
                    3
                }
                _ => {
                    // Upstream quirk: resolution failure falls through with the literal "N/A".
                    not_installed(cx, "N/A", false);
                    println!("Reinstalling global packages from N/A...\nNo installed global packages found...\nLinking global packages from N/A...\nNo linked global packages found...");
                    0
                }
            }
        }
        "debug" => cmd_debug(cx),
        "__auto" => cmd_auto(cx, rest),
        "__ls" => cmd_raw_ls(cx, rest),
        "exec-internal" => cmd_exec_internal(cx, rest),
        "__rcversion" => {
            let Some(f) = find_nvmrc(&env::current_dir().unwrap_or_default()) else {
                return 1;
            };
            match parse_nvmrc(&fs::read_to_string(&f).unwrap_or_default()) {
                Ok(v) => {
                    println!("{}\n{v}", f.display());
                    0
                }
                Err(_) => 2,
            }
        }
        other => {
            let _ = other;
            eprint!("{HELP}");
            127
        }
    }
}

fn usage_cache() -> i32 {
    eprintln!("Usage: nvm cache dir\n       nvm cache clear\n  {HELP_HINT}");
    127
}

fn guard(s: &str) -> bool {
    !s.split('/').any(|c| c == "..")
}

fn cmd_version(cx: &mut Ctx, rest: &[String]) -> i32 {
    let pat = rest.first().map(String::as_str).unwrap_or("");
    if !guard(pat) {
        println!("N/A");
        return 3;
    }
    match cx.st.version(pat) {
        Ok(v) => {
            println!("{v}");
            0
        }
        Err(e) => {
            println!("{e}");
            3
        }
    }
}

/// `nvm_rc_version`: nearest .nvmrc -> Ok(version). Prints errors (and "Found ...") like upstream; `silent` mutes
/// the "No version provided" and "Found" lines. Err(()) when no usable version.
fn rc_version(silent: bool) -> Result<String, ()> {
    let cwd = env::current_dir().unwrap_or_default();
    let Some(f) = find_nvmrc(&cwd) else {
        if !silent {
            eprintln!("No version provided and no .nvmrc file found");
        }
        return Err(());
    };
    match parse_nvmrc(&fs::read_to_string(&f).unwrap_or_default()) {
        Ok(v) => {
            if !silent {
                println!("Found '{}' with version <{v}>", f.display());
            }
            Ok(v)
        }
        Err(lines) => {
            eprintln!("invalid .nvmrc!\nall non-commented content (anything after # is a comment) must be either:\n  - a single bare nvm-recognized version-ish\n  - or, multiple distinct key-value pairs, each key/value separated by a single equals sign (=)\n\nadditionally, a single bare nvm-recognized version-ish must be present (after stripping comments).\n\nnon-commented content parsed:");
            if !lines.is_empty() {
                eprintln!("{lines}");
            }
            Err(())
        }
    }
}

/// `nvm_match_version`.
fn match_version(cx: &Ctx, p: &str) -> Result<String, &'static str> {
    match p {
        "system" => Ok("system".into()),
        _ => cx.st.version(p),
    }
}

/// `nvm_ensure_version_installed`: true when usable; otherwise prints upstream's message and returns false.
fn ensure_installed(cx: &Ctx, provided: &str, from_rc: bool) -> bool {
    if provided == "system" {
        if cx.st.system_node().is_some() {
            return true;
        }
        eprintln!("N/A: no system version of node/io.js is installed.");
        return false;
    }
    if let Ok(v) = cx.st.version(provided) {
        if cx.st.is_installed(&v) {
            return true;
        }
    }
    not_installed(cx, provided, from_rc);
    false
}
/// Version arg or `.nvmrc`. Returns (pattern, from_rc) or exit code.
fn pick_pattern(rest: &[String], silent: bool, which: bool) -> Result<(String, bool), i32> {
    // use: last non-empty non-`--*` argument wins; which: last argument wins (even empty), `--silent` and `--` skipped
    let mut provided: Option<String> = None;
    for a in rest {
        if which {
            if a != "--silent" && a != "--" {
                provided = Some(a.clone());
            }
        } else if !a.starts_with("--") && a != "-w" && !a.is_empty() {
            provided = Some(a.clone());
        }
    }
    if let Some(v) = provided.filter(|v| !v.is_empty()) {
        return Ok((v, false));
    }
    match rc_version(silent) {
        Ok(v) => Ok((v, true)),
        Err(()) => {
            if which {
                eprintln!("Usage: nvm which [current | <version>]\n  Provide a <version>, or run from a directory containing an .nvmrc file.\n  {HELP_HINT}");
            } else {
                eprintln!("Please see `nvm --help` or https://github.com/nvm-sh/nvm#nvmrc for more information.");
            }
            Err(127)
        }
    }
}
fn not_installed(cx: &Ctx, arg: &str, from_rc: bool) {
    if arg == "system" {
        eprintln!("N/A: no system version of node/io.js is installed.");
        return;
    }
    let orig = arg;
    match cx.st.resolve_alias(arg) {
        Some(r) => eprintln!("N/A: version \"{arg} -> {r}\" is not yet installed.\n"),
        None => eprintln!("N/A: version \"{}\" is not yet installed.\n", ensure_v(arg)),
    }
    if orig == "lts" {
        eprintln!("`lts` is not an alias - you may need to run `nvm install --lts` to install and `nvm use --lts` to use it.");
    } else if from_rc {
        eprintln!("You need to run `nvm install` to install and use the node version specified in `.nvmrc`.");
    } else {
        eprintln!("You need to run `nvm install {orig}` to install and use it.");
    }
}
fn cmd_which(cx: &mut Ctx, rest: &[String]) -> i32 {
    let silent = rest.iter().any(|a| a == "--silent");
    let (pat, from_rc) = match pick_pattern(rest, silent, true) {
        Ok(x) => x,
        Err(c) => return c,
    };
    // literal `system` is special; a `system` read from .nvmrc goes through version() like any other pattern
    let ver: Result<String, &str> = if pat == "system" && !from_rc {
        Ok("system".into())
    } else {
        cx.st.version(&pat)
    };
    match ver {
        Ok(v) if v == "system" => match system_node(cx) {
            Some((p, _)) => {
                println!("{p}");
                0
            }
            None => {
                eprintln!("System version of node not found.");
                127
            }
        },
        Err("∞") => {
            eprintln!("The alias \"{pat}\" leads to an infinite loop. Aborting.");
            8
        }
        Ok(v) => {
            if ensure_installed(cx, &pat, false) {
                println!("{}/bin/node", cx.st.version_dir(&v).display());
                0
            } else {
                1
            }
        }
        Err(_) => {
            ensure_installed(cx, &pat, false);
            1
        }
    }
}
/// First `node` on PATH outside `$NVM_DIR`; returns (path, version string).
fn system_node(cx: &Ctx) -> Option<(String, String)> {
    let path = env::var("PATH").unwrap_or_default();
    let clean = pathedit::strip_path(&path, "/bin", &cx.nvm_dir);
    for d in clean.split(':') {
        let n = std::path::Path::new(d).join("node");
        if n.is_file() {
            let v = Command::new(&n)
                .arg("--version")
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_default();
            return Some((n.to_string_lossy().into_owned(), v));
        }
    }
    None
}
fn cmd_use(cx: &mut Ctx, rest: &[String]) -> i32 {
    let mut silent = false;
    let mut save = false;
    let mut delete = false;
    let mut lts: Option<String> = None;
    for a in rest {
        match a.as_str() {
            "--silent" => silent = true,
            "--delete-prefix" => delete = true,
            "--lts" => lts = Some("*".into()),
            "--save" | "-w" => {
                if save {
                    eprintln!("--save and -w may only be provided once");
                    return 6;
                }
                save = true;
            }
            x => {
                if let Some(n) = x.strip_prefix("--lts=") {
                    lts = Some(n.to_string());
                }
            }
        }
    }
    let positional = rest
        .iter()
        .any(|a| !a.starts_with("--") && a != "-w" && !a.is_empty());
    // VERSION and the string the user provided (for messages)
    let (pat, from_rc, ver): (String, bool, Result<String, &'static str>) =
        if let Some(n) = lts.clone() {
            let p = format!("lts/{n}");
            let v = match_version(cx, &p);
            (p, false, v)
        } else {
            let (p, rc) = match pick_pattern(rest, silent, false) {
                Ok(x) => x,
                Err(c) => return c,
            };
            let v = if rc {
                cx.st.version(&p)
            } else {
                match_version(cx, &p)
            };
            (p, rc, v)
        };
    if !guard(&pat) {
        eprintln!("invalid version: {pat}");
        return 3;
    }
    if save {
        if let Ok(v) = ver.as_ref().map(|v| v.clone()) {
            if let Ok(w) = cx.st.version(&v) {
                let target = env::current_dir().unwrap_or_default().join(".nvmrc");
                match fs::write(&target, format!("{w}\n")) {
                    Ok(_) => {
                        if !silent {
                            println!("Wrote version number ({w}) to .nvmrc");
                        }
                    }
                    Err(_) => {
                        if !silent {
                            eprintln!("Warning: Unable to write version number ({w}) to .nvmrc");
                        }
                        return 3;
                    }
                }
            }
        }
    }
    match ver {
        Ok(ref v) if v == "system" && !from_rc || matches!(&ver, Ok(v) if v == "system") => {
            if system_node(cx).is_none() {
                if !silent {
                    eprintln!("System version of node not found.");
                }
                return 127;
            }
            let sv = system_node(cx).map(|x| x.1).unwrap_or_default();
            let path = cx.ev.get("PATH").unwrap_or_default();
            let np = pathedit::strip_path(&path, "/bin", &cx.nvm_dir);
            let _ = cmd_deactivate(cx, &["--silent".to_string()]);
            if !silent {
                let npm = Command::new("npm")
                    .arg("--version")
                    .env("PATH", &np)
                    .output()
                    .ok()
                    .filter(|o| o.status.success())
                    .map(|o| format!(" (npm v{})", String::from_utf8_lossy(&o.stdout).trim()))
                    .unwrap_or_default();
                println!("Now using system version of node: {sv}{npm}");
            }
            0
        }
        Err("∞") => {
            if !silent {
                eprintln!("The alias \"{pat}\" leads to an infinite loop. Aborting.");
            }
            8
        }
        Err(_) => {
            // Upstream quirk: `use --lts` (no positional) passes an empty PROVIDED_VERSION to the check, so it stays silent.
            if !silent && !(lts.is_some() && !positional) {
                ensure_installed(cx, &pat, from_rc);
            }
            3
        }
        Ok(v) => {
            if !cx.st.is_installed(&v) {
                // upstream: `! nvm_ensure_version_installed` then `return $?` leaves rc 0
                if !silent {
                    ensure_installed(cx, &v, from_rc);
                }
                return 0;
            }
            let vd = cx.st.version_dir(&v).to_string_lossy().into_owned();
            let path = cx.ev.get("PATH").unwrap_or_default();
            cx.ev.export(
                "PATH",
                &pathedit::change_path(&path, "/bin", &vd, &cx.nvm_dir),
            );
            if has_manpath() {
                let mut m = pathedit::change_path(
                    &cx.ev.get("MANPATH").unwrap_or_default(),
                    "/share/man",
                    &vd,
                    &cx.nvm_dir,
                );
                if !(m.starts_with(':') || m.contains("::") || m.ends_with(':')) {
                    m.push(':');
                }
                cx.ev.export("MANPATH", &m);
            }
            cx.ev.export("NVM_BIN", &format!("{vd}/bin"));
            cx.ev.export("NVM_INC", &format!("{vd}/include/node"));
            if env::var("NVM_SYMLINK_CURRENT").as_deref() == Ok("true") {
                let cur = cx.st.dir.join("current");
                let _ = fs::remove_file(&cur);
                let _ = std::os::unix::fs::symlink(&vd, &cur);
            }
            cx.ev.rehash();
            let mut ucmd = String::from("nvm use --delete-prefix");
            if positional {
                ucmd += &format!(" {v}");
            }
            if silent {
                ucmd += " --silent";
            }
            if die_on_prefix(cx, delete, &ucmd, &vd, silent).is_err() {
                return 11;
            }
            if !silent {
                match cx.st.npm_version(&v) {
                    Some(n) => println!("Now using node {v} (npm v{n})"),
                    None => println!("Now using node {v}"),
                }
            }
            0
        }
    }
}
fn cmd_deactivate(cx: &mut Ctx, rest: &[String]) -> i32 {
    let silent = rest.iter().any(|a| a == "--silent");
    let path = cx.ev.get("PATH").unwrap_or_default();
    let np = pathedit::strip_path(&path, "/bin", &cx.nvm_dir);
    if np == path {
        if !silent {
            eprintln!("Could not find {}/*/bin in ${{PATH}}", cx.nvm_dir);
        }
    } else {
        cx.ev.export("PATH", &np);
        cx.ev.rehash();
        if !silent {
            println!("{}/*/bin removed from ${{PATH}}", cx.nvm_dir);
        }
    }
    if let Some(m) = cx.ev.get("MANPATH").filter(|m| !m.is_empty()) {
        let nm = pathedit::strip_path(&m, "/share/man", &cx.nvm_dir);
        if nm == m {
            if !silent {
                eprintln!("Could not find {}/*/share/man in ${{MANPATH}}", cx.nvm_dir);
            }
        } else {
            if nm.is_empty() || nm == ":" {
                cx.ev.unset("MANPATH");
            } else {
                cx.ev.export("MANPATH", &nm);
            }
            if !silent {
                println!("{}/*/share/man removed from ${{MANPATH}}", cx.nvm_dir);
            }
        }
    }
    if let Some(n) = cx.ev.get("NODE_PATH").filter(|m| !m.is_empty()) {
        let nn = pathedit::strip_path(&n, "/lib/node_modules", &cx.nvm_dir);
        if nn != n {
            cx.ev.export("NODE_PATH", &nn);
            if !silent {
                println!(
                    "{}/*/lib/node_modules removed from ${{NODE_PATH}}",
                    cx.nvm_dir
                );
            }
        }
    }
    cx.ev.unset("NVM_BIN");
    cx.ev.unset("NVM_INC");
    0
}

fn sanitize(cx: &Ctx, p: &str) -> String {
    let mut o = p.to_string();
    if o != cx.nvm_dir {
        o = o.replace(&cx.nvm_dir, "${NVM_DIR}");
    }
    let home = env::var("HOME").unwrap_or_default();
    if !home.is_empty() && o != home {
        o = o.replace(&home, "${HOME}");
    }
    o
}

fn npmrc_bad(p: &std::path::Path) -> bool {
    fs::read_to_string(p)
        .map(|t| {
            t.lines().any(|l| {
                let k = l.split('=').next().unwrap_or("");
                (l.contains('='))
                    && matches!(k.trim_end_matches(' '), "prefix" | "globalconfig")
                    && l.starts_with(k)
            })
        })
        .unwrap_or(false)
}

/// §5.4. Err(rc) is the helper's code (3, 4, 10); `use` turns any failure into 11.
fn die_on_prefix(
    cx: &mut Ctx,
    delete: bool,
    cmd: &str,
    vd: &str,
    silent_use: bool,
) -> Result<(), i32> {
    let _ = silent_use;
    if let Ok(pre) = env::var("PREFIX") {
        if !pre.is_empty() && pre != vd {
            let _ = cmd_deactivate(cx, &["--silent".to_string()]);
            eprintln!("nvm is not compatible with the \"PREFIX\" environment variable: currently set to \"{pre}\"\nRun `unset PREFIX` to unset it.");
            return Err(3);
        }
    }
    if let Some((name, val)) = env::vars().find(|(k, _)| k.to_uppercase() == "NPM_CONFIG_PREFIX") {
        let under = val == cx.nvm_dir || val.starts_with(&format!("{}/", cx.nvm_dir));
        if !val.is_empty() && !under {
            let _ = cmd_deactivate(cx, &["--silent".to_string()]);
            eprintln!("nvm is not compatible with the \"{name}\" environment variable: currently set to \"{val}\"\nRun `unset {name}` to unset it.");
            return Err(4);
        }
    }
    let npm = format!("{vd}/bin/npm");
    let path = format!("{vd}/bin:{}", env::var("PATH").unwrap_or_default());
    let run_npm = |args: &[&str]| {
        let _ = Command::new(&npm).args(args).env("PATH", &path).status();
    };
    let home_rc = format!("{}/.npmrc", env::var("HOME").unwrap_or_default());
    let mut project = env::current_dir().unwrap_or_default();
    while project.as_os_str() != ""
        && project.to_string_lossy() != "."
        && !project.join("package.json").is_file()
        && !project.join("node_modules").is_dir()
    {
        match project.parent() {
            Some(p) if p != project => project = p.to_path_buf(),
            _ => {
                project = std::path::PathBuf::new();
                break;
            }
        }
    }
    let project_rc = format!("{}/.npmrc", project.display());
    let checks: [(&str, String, Vec<Vec<String>>); 4] = [
        (
            "builtin npmrc",
            format!("{vd}/lib/node_modules/npm/npmrc"),
            vec![],
        ),
        ("global npmrc", format!("{vd}/etc/npmrc"), vec![]),
        ("user’s .npmrc", home_rc, vec![]),
        ("project npmrc", project_rc, vec![]),
    ];
    for (i, (label, file, _)) in checks.iter().enumerate() {
        if !npmrc_bad(std::path::Path::new(file)) {
            continue;
        }
        if delete {
            match i {
                0 | 2 => {
                    let uc = format!("--userconfig={file}");
                    run_npm(&["config", "--loglevel=warn", "delete", "prefix", &uc]);
                    run_npm(&["config", "--loglevel=warn", "delete", "globalconfig", &uc]);
                }
                1 => {
                    run_npm(&["config", "--global", "--loglevel=warn", "delete", "prefix"]);
                    run_npm(&[
                        "config",
                        "--global",
                        "--loglevel=warn",
                        "delete",
                        "globalconfig",
                    ]);
                }
                _ => {
                    run_npm(&["config", "--loglevel=warn", "delete", "prefix"]);
                    run_npm(&["config", "--loglevel=warn", "delete", "globalconfig"]);
                }
            }
        } else {
            eprintln!("Your {label} file ({})\nhas a `globalconfig` and/or a `prefix` setting, which are incompatible with nvm.\nRun `{cmd}` to unset it.", sanitize(cx, file));
            return Err(10);
        }
    }
    Ok(())
}

fn has_manpath() -> bool {
    env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .any(|d| std::path::Path::new(d).join("manpath").is_file())
}
fn builtin(n: &str) -> bool {
    matches!(n, "stable" | "unstable" | "iojs" | "node" | "system")
}

fn cmd_alias(cx: &mut Ctx, rest: &[String]) -> i32 {
    let mut alias: Option<String> = None;
    let mut target: Option<String> = None;
    for a in rest {
        match a.as_str() {
            "--" | "--no-colors" => {}
            x if x.starts_with("--") => {
                eprintln!("Unsupported option \"{x}\".");
                return 55;
            }
            x => {
                if alias.is_none() {
                    alias = Some(x.to_string());
                } else if target.is_none() {
                    target = Some(x.to_string());
                }
            }
        }
    }
    if target.as_deref() == Some("") {
        return cmd_unalias(cx, &[alias.unwrap_or_default()]);
    }
    if alias.as_deref().map_or(false, |a| a.contains('#')) {
        eprintln!("Aliases with a comment delimiter (#) are not supported.");
        return 1;
    }
    if let Some(t) = target {
        let name = alias.unwrap_or_default();
        if name.contains('/') {
            eprintln!("Aliases in subdirectories are not supported.");
            return 1;
        }
        if !guard(&name) {
            eprintln!("invalid alias name: {name}");
            return 3;
        }
        if cx.st.version(&t).is_err() {
            eprintln!("! WARNING: Version '{t}' does not exist.");
        }
        let _ = fs::create_dir_all(cx.st.dir.join("alias"));
        if fs::write(cx.st.alias_path(&name), format!("{t}\n")).is_err() {
            return 1;
        }
        print_alias(cx, &name, &t);
        return 0;
    }
    match alias.filter(|a| !a.is_empty()) {
        None => {
            list_aliases(cx, None);
            0
        }
        Some(pat) if pat.starts_with("lts/") => match cx.st.read_alias(&pat) {
            Some(t) => {
                println!("{t}");
                0
            }
            None => {
                eprintln!("Alias does not exist.");
                2
            }
        },
        Some(pat) => {
            list_aliases(cx, Some(&pat));
            0
        }
    }
}
fn cmd_unalias(cx: &mut Ctx, rest: &[String]) -> i32 {
    if rest.len() != 1 {
        eprintln!("Usage: nvm unalias <name>\n  {HELP_HINT}");
        return 127;
    }
    let name = &rest[0];
    if name.contains('/') {
        eprintln!("Aliases in subdirectories are not supported.");
        return 1;
    }
    if !guard(name) {
        eprintln!("invalid alias name: {name}");
        return 3;
    }
    let p = cx.st.alias_path(name);
    if !p.is_file() {
        if builtin(name) {
            eprintln!("{name} is a default (built-in) alias and cannot be deleted.");
            return 1;
        }
        eprintln!("Alias {name} doesn't exist!");
        return 0;
    }
    let old = cx.st.read_alias(name).unwrap_or_default();
    if fs::remove_file(&p).is_err() {
        return 1;
    }
    println!("Deleted alias {name} - restore it with `nvm alias \"{name}\" \"{old}\"`");
    0
}
fn alias_line(cx: &Ctx, name: &str, dest: &str, implicit: bool) -> String {
    let path = env::var("PATH").unwrap_or_default();
    let cur = cx.st.ls_current(&path);
    let resolved = cx.st.version(dest).unwrap_or_else(|e| e.to_string());
    let status = |v: &str| -> usize {
        if v == "N/A" || v == "∞" || !cx.st.is_installed(v) {
            4
        } else if v == cur {
            3
        } else {
            1
        }
    };
    let colour = cx.pal.is_some();
    let arrow = colors::paint(
        &cx.pal.as_ref().map(|_| {
            colors::Pal([
                "0;90".into(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            ])
        }),
        1,
        "->",
    );
    let arrow = if colour { arrow } else { "->".into() };
    let name_c = if name.starts_with("lts/") {
        6
    } else {
        status(&resolved)
    };
    let dest_c = if dest.starts_with("lts/") {
        6
    } else {
        status(&resolved)
    };
    let star = if !colour && cx.st.is_installed(&resolved) {
        " *"
    } else {
        ""
    };
    let mut out = format!(
        "{} {arrow} {}",
        colors::paint(&cx.pal, name_c, name),
        colors::paint(&cx.pal, dest_c, dest)
    );
    if resolved == dest {
        out += star;
    } else {
        out += &format!(
            " ({arrow} {}{star})",
            colors::paint(&cx.pal, status(&resolved), &resolved)
        );
    }
    if implicit {
        out += &if colour {
            format!(" {}", colors::paint(&cx.pal, 5, "(default)"))
        } else {
            " (default)".to_string()
        };
    }
    out
}

fn print_alias(cx: &Ctx, name: &str, dest: &str) {
    println!("{}", alias_line(cx, name, dest, false));
}
fn walk_aliases(dir: &std::path::Path, prefix: &str, out: &mut Vec<String>) {
    for e in fs::read_dir(dir).into_iter().flatten().flatten() {
        let n = e.file_name().to_string_lossy().into_owned();
        let full = if prefix.is_empty() {
            n.clone()
        } else {
            format!("{prefix}/{n}")
        };
        if e.path().is_dir() {
            // upstream lists only top-level files and the `lts/` directory
            if prefix.is_empty() && n == "lts" {
                walk_aliases(&e.path(), &full, out);
            }
        } else {
            out.push(full);
        }
    }
}

fn list_aliases(cx: &Ctx, pat: Option<&str>) {
    let mut names = vec![];
    walk_aliases(&cx.st.dir.join("alias"), "", &mut names);
    names.sort();
    let p = pat.unwrap_or("");
    let (lts, user): (Vec<_>, Vec<_>) = names.into_iter().partition(|n| n.starts_with("lts/"));
    let user: Vec<String> = user.into_iter().filter(|n| n.starts_with(p)).collect();
    let lts: Vec<String> = lts
        .into_iter()
        .filter(|n| n["lts/".len()..].starts_with(p))
        .collect();
    for n in &user {
        if let Some(t) = cx.st.read_alias(n) {
            print_alias(cx, n, &t);
        }
    }
    let has = |n: &str| user.iter().any(|u| u == n);
    let inst = cx.st.installed();
    let mut stable = None;
    let mut unstable = None;
    for ((maj, min, _), _) in &inst {
        if *maj >= 1 || min % 2 == 0 {
            stable = Some(format!("{maj}.{min}"));
        } else {
            unstable = Some(format!("{maj}.{min}"));
        }
    }
    let lines = std::cell::RefCell::new(Vec::<String>::new());
    let implicit = |name: &str, val: Option<String>| {
        if has(name) || (pat.is_some() && name != p) {
            return;
        }
        match val {
            None => {
                let c = cx.pal.is_some();
                let arrow = if c {
                    colors::paint(
                        &Some(colors::Pal([
                            "0;90".into(),
                            String::new(),
                            String::new(),
                            String::new(),
                            String::new(),
                            String::new(),
                        ])),
                        1,
                        "->",
                    )
                } else {
                    "->".into()
                };
                let tail = if c {
                    colors::paint(&cx.pal, 5, "(default)")
                } else {
                    "(default)".into()
                };
                lines.borrow_mut().push(format!(
                    "{} {arrow} {} {tail}",
                    colors::paint(&cx.pal, 4, name),
                    colors::paint(&cx.pal, 4, "N/A")
                ));
            }
            Some(d) => lines.borrow_mut().push(alias_line(cx, name, &d, true)),
        }
    };
    implicit("iojs", None);
    implicit("node", Some("stable".into()));
    if stable.is_some() {
        implicit("stable", stable);
    }
    implicit("unstable", unstable);
    let mut ls = lines.into_inner();
    ls.sort();
    for l in ls {
        println!("{l}");
    }
    for n in &lts {
        if let Some(t) = cx.st.read_alias(n) {
            print_alias(cx, n, &t);
        }
    }
}

/// Plain (no colour) `nvm ls`: §8 without annotations other than ` *`.
fn cmd_ls(cx: &mut Ctx, rest: &[String]) -> i32 {
    for a in rest {
        if a.starts_with("--") && !matches!(a.as_str(), "--" | "--no-colors" | "--no-alias") {
            eprintln!("Unsupported option \"{a}\".");
            return 55;
        }
    }
    let no_alias = rest.iter().any(|a| a == "--no-alias");
    let pat = rest.iter().find(|a| !a.starts_with("--") && !a.is_empty());
    let cur = cx.st.ls_current(&env::var("PATH").unwrap_or_default());
    let list: Vec<String> = match pat {
        Some(p) => {
            if no_alias {
                eprintln!("`--no-alias` is not supported when a pattern is provided.");
                return 55;
            }
            let numeric = p
                .trim_start_matches('v')
                .starts_with(|c: char| c.is_ascii_digit());
            if !numeric {
                match cx.st.version(p) {
                    Ok(v) if cx.st.is_installed(&v) => vec![v],
                    _ => vec![],
                }
            } else {
                let pfx = format!("{}.", ensure_v(p).trim_end_matches('.'));
                cx.st
                    .installed()
                    .into_iter()
                    .map(|x| x.1)
                    .filter(|n| format!("{n}.").starts_with(&pfx) || *n == ensure_v(p))
                    .collect()
            }
        }
        None => cx.st.installed().into_iter().map(|x| x.1).collect(),
    };
    let want_system = pat.map_or(true, |p| p == "system") && cx.st.system_node().is_some();
    let list: Vec<String> = if pat.map_or(false, |p| p == "system") {
        vec![]
    } else {
        list
    };
    let sys_row = |cx: &Ctx| {
        let sv = cx.st.system_version().unwrap_or_default();
        let txt = if cur == "system" {
            format!("->{:>13}", "system")
        } else {
            format!("{:>15}", "system")
        };
        if cx.pal.is_some() {
            format!(
                "{} ({} {})",
                colors::paint(&cx.pal, if cur == "system" { 3 } else { 2 }, &txt),
                colors::paint(
                    &Some(colors::Pal([
                        "0;90".into(),
                        String::new(),
                        String::new(),
                        String::new(),
                        String::new(),
                        String::new()
                    ])),
                    1,
                    "->"
                ),
                colors::paint(&cx.pal, 2, &sv)
            )
        } else {
            format!("{txt} * (-> {sv})")
        }
    };
    if list.is_empty() && want_system {
        if pat.is_none() {
            println!();
        }
        println!("{}", sys_row(cx));
        if pat.is_none() && !no_alias {
            list_aliases(cx, None);
        }
        return 0;
    }
    if list.is_empty() {
        let na = format!("{:>15}", "N/A");
        println!("{na}{}", if pat.is_none() { " *" } else { "" });
        if pat.is_none() && !no_alias {
            list_aliases(cx, None);
        }
        return 3;
    }
    for v in &list {
        let (txt, ci) = if *v == cur {
            (format!("->{v:>13}"), 3)
        } else {
            (format!("{v:>15}"), 1)
        };
        if cx.pal.is_some() {
            println!("{}", colors::paint(&cx.pal, ci, &txt));
        } else {
            println!("{txt} *");
        }
    }
    if want_system {
        println!("{}", sys_row(cx));
    }
    if pat.is_none() && !no_alias {
        list_aliases(cx, None);
    }
    0
}

/// §9.1 `__auto use`: `.nvmrc` in ancestry, else `default`; no-op when already right.
/// `__auto <use|cd|install>`: `use` = upstream source-time semantics (§9.1); `cd` = chpwd semantics (.nvmrc else default).
fn cmd_auto(cx: &mut Ctx, rest: &[String]) -> i32 {
    let path = env::var("PATH").unwrap_or_default();
    let cur = cx.st.ls_current(&path);
    let nvmrc = |_cx: &Ctx| {
        find_nvmrc(&env::current_dir().unwrap_or_default())
            .and_then(|f| fs::read_to_string(f).ok())
            .and_then(|c| parse_nvmrc(&c).ok())
    };
    match rest.first().map(String::as_str) {
        Some("use") => {
            if cur == "none" || cur == "system" {
                match cx.st.read_alias("default") {
                    Some(_) => match cx.st.version("default") {
                        Ok(v) if cx.st.is_installed(&v) => cmd_use(cx, &[v, "--silent".into()]),
                        _ => 0,
                    },
                    None => {
                        if nvmrc(cx).is_some() {
                            cmd_use(cx, &["--silent".to_string()])
                        } else {
                            0
                        }
                    }
                }
            } else {
                cmd_use(cx, &[cur, "--silent".into()])
            }
        }
        Some("cd") => {
            let want = nvmrc(cx).unwrap_or_else(|| "default".into());
            let Ok(target) = cx.st.version(&want) else {
                return 0;
            };
            if !cx.st.is_installed(&target) {
                return 0;
            }
            if cur == target && env::var("NVM_BIN").is_ok() {
                return 0;
            }
            cmd_use(cx, &[target, "--silent".into()])
        }
        Some("install") => {
            let def = cx.st.read_alias("default");
            match def {
                Some(d)
                    if cx.st.version(&d).is_ok()
                        || d.chars()
                            .next()
                            .map_or(false, |c| c.is_ascii_digit() || c == 'v') =>
                {
                    cmd_install(cx, &[d])
                }
                _ => {
                    if nvmrc(cx).is_some() {
                        cmd_install(cx, &[])
                    } else {
                        0
                    }
                }
            }
        }
        _ => {
            eprintln!("Invalid auto mode supplied.");
            1
        }
    }
}

/// Raw installed versions for shell helpers (`nvm_ls`).
fn cmd_raw_ls(cx: &mut Ctx, rest: &[String]) -> i32 {
    let pat = rest.first().map(|p| ensure_v(p));
    for (_, n) in cx.st.installed() {
        if pat.as_ref().map_or(true, |p| {
            format!("{n}.").starts_with(&format!("{}.", p.trim_end_matches('.')))
        }) {
            println!("{n}");
        }
    }
    0
}

/// `nvm-exec` replacement: NODE_VERSION or .nvmrc, then exec the command with the selected node first on PATH.
fn cmd_exec_internal(cx: &mut Ctx, rest: &[String]) -> i32 {
    let sel = match env::var("NODE_VERSION").ok().filter(|v| !v.is_empty()) {
        Some(v) => match cx.st.version(&v) {
            Ok(r) if cx.st.is_installed(&r) => Ok(r),
            _ => {
                not_installed(cx, &v, false);
                return 127;
            }
        },
        None => {
            let rc = find_nvmrc(&env::current_dir().unwrap_or_default())
                .and_then(|f| fs::read_to_string(f).ok())
                .and_then(|c| parse_nvmrc(&c).ok());
            match rc {
                Some(v) => match cx.st.version(&v) {
                    Ok(r) if cx.st.is_installed(&r) => Ok(r),
                    _ => {
                        not_installed(cx, &v, true);
                        Err(())
                    }
                },
                None => Err(()),
            }
        }
    };
    match sel {
        Ok(v) => run_with(cx, &v, rest),
        Err(()) => {
            eprintln!("nvm-exec: unable to select a node version\n  Set `NODE_VERSION` (e.g. `NODE_VERSION=default`), or add an `.nvmrc` file.");
            127
        }
    }
}
// ---- remote commands ----

/// Split `--lts`, `--lts=x`, `lts/x` forms. Returns (lts filter, remaining pattern).
fn lts_filter(cx: &Ctx, args: &[String]) -> Result<(Option<String>, Option<String>), i32> {
    let mut lts = None;
    let mut pat = None;
    for a in args {
        if a == "--lts" {
            lts = Some("*".to_string());
        } else if let Some(n) = a.strip_prefix("--lts=") {
            lts = Some(n.to_string());
        } else if a == "lts/*" {
            lts = Some("*".into());
        } else if let Some(n) = a.strip_prefix("lts/") {
            lts = Some(n.to_string());
        } else if !a.starts_with("-") && a != "--" {
            pat = Some(a.clone());
        }
    }
    if let Some(n) = lts.clone().filter(|n| n != "*") {
        match remote::lts_name(&cx.st, &n) {
            Ok(x) => lts = Some(x),
            Err((m, rc)) => {
                eprintln!("{m}");
                return Err(rc);
            }
        }
    }
    Ok((lts, pat))
}

fn cmd_ls_remote(cx: &mut Ctx, rest: &[String]) -> i32 {
    let (lts, pat) = match lts_filter(cx, rest) {
        Ok(x) => x,
        Err(c) => return c,
    };
    let mut rows = match remote::index(&cx.st) {
        Ok(r) => r,
        Err(c) => return c,
    };
    let node_empty = rows.is_empty();
    if pat.is_none() && lts.is_none() {
        if let Ok(mut io) = remote::iojs_rows() {
            rows.append(&mut io);
            remote::sort_rows(&mut rows);
        }
    }
    let cur = cx.st.ls_current(&env::var("PATH").unwrap_or_default());
    let node_alias = pat.as_deref() == Some("node");
    let sel = remote::filter(
        &rows,
        lts.as_deref(),
        if node_alias { None } else { pat.as_deref() },
    );

    if sel.is_empty() {
        let star =
            cx.pal.is_none() && lts.is_none() && pat.as_deref().map_or(false, valid_versionish);
        println!("{:>15}{}", "N/A", if star { " *" } else { "" });
        return 3;
    }
    let mut out = String::new();
    for r in sel {
        out += &remote::format_row(r, cx.st.is_installed(&r.ver), r.ver == cur, &cx.pal);
        out.push('\n');
    }
    print!("{out}");
    // Upstream quirk (v0.40.8): a positional pattern without --lts exits 3 even on success.
    if node_empty {
        3
    } else if pat.is_some() && lts.is_none() && !node_alias {
        3
    } else {
        0
    }
}

fn cmd_version_remote(cx: &mut Ctx, rest: &[String]) -> i32 {
    for a in rest {
        if a.starts_with("--") && a != "--" && a != "--lts" && !a.starts_with("--lts=") {
            eprintln!("Unsupported option \"{a}\".");
            return 55;
        }
    }
    let (lts, pat) = match lts_filter(cx, rest) {
        Ok(x) => x,
        // Upstream quirk: prints the message twice, then a blank line, and exits 0.
        Err(3) => {
            eprintln!("LTS names must be lowercase\n");
            return 0;
        }
        Err(c) => return c,
    };
    let rows = match remote::index(&cx.st) {
        Ok(r) => r,
        Err(c) => return c,
    };
    match remote::resolve(
        &cx.st,
        &rows,
        pat.as_deref().unwrap_or("node"),
        lts.as_deref(),
    ) {
        Some(r) => {
            println!("{}", r.ver);
            0
        }
        None => {
            println!("N/A");
            3
        }
    }
}

fn npm_ver_suffix(cx: &Ctx, v: &str) -> String {
    cx.st
        .npm_version(v)
        .map(|n| format!(" (npm v{n})"))
        .unwrap_or_default()
}

fn cmd_install(cx: &mut Ctx, rest: &[String]) -> i32 {
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
    for a in rest {
        match a.as_str() {
            "--reinstall-packages-from" | "--copy-packages-from" => {
                eprintln!(
                    "If {a} is provided, it must point to an installed version of node using `=`."
                );
                return 6;
            }
            x if x.starts_with("--reinstall-packages-from=")
                || x.starts_with("--copy-packages-from=") =>
            {
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
                let val = x.split_once('=').unwrap().1.to_string();
                if val.is_empty() {
                    eprintln!(
                        "If {} is provided, it must point to an installed version of node.",
                        x.split_once('=').unwrap().0
                    );
                    return 6;
                }
                reinstall_from = Some(val);
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
                flags_lts = Some(x[6..].to_string()).filter(|n| !n.is_empty())
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
            x if x.starts_with('-') => {}
            x if version.is_none() => version = Some(x.to_string()),
            _ => {}
        }
    }
    if bin_only && src_only {
        eprintln!("-s and -b cannot be set together since they would skip install from both binary and source");
        return 6;
    }
    if set_default && alias.is_some() {
        eprintln!(
            "--default and --alias are mutually exclusive, and may not be provided more than once"
        );
        return 6;
    }
    let mut from_rc = false;
    if let Some(v) = version.clone() {
        if v == "lts/*" {
            flags_lts = Some("*".into());
            version = None;
        } else if let Some(n) = v.strip_prefix("lts/") {
            flags_lts = Some(n.to_string());
            version = None;
        }
    }
    if version.is_none() {
        match &flags_lts {
            Some(n) if n == "*" => println!("Installing latest LTS version."),
            Some(n) => println!("Installing with latest version of LTS line: {n}"),
            None => {
                let cwd = env::current_dir().unwrap_or_default();
                let rcf = find_nvmrc(&cwd);
                match rcf
                    .clone()
                    .and_then(|f| fs::read_to_string(f).ok())
                    .map(|c| parse_nvmrc(&c))
                {
                    Some(Ok(v)) => {
                        println!("Found '{}' with version <{v}>", rcf.unwrap().display());
                        version = Some(v);
                        from_rc = true;
                    }
                    Some(Err(_)) | None => {
                        eprintln!("Usage: nvm install [<version>]\n  Provide a <version>, or run from a directory containing an .nvmrc file.\n{HELP_HINT}");
                        return 127;
                    }
                }
            }
        }
    }
    let _ = from_rc;
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
        match cx.st.version(&pat) {
            Ok(v) => Some(v),
            Err(_) => {
                let p = ensure_v(&pat);
                let pfx = format!("{}.", p.trim_end_matches('.'));
                cx.st
                    .cached()
                    .into_iter()
                    .rev()
                    .map(|x| x.1)
                    .find(|n| format!("{n}.").starts_with(&pfx))
            }
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
            None => ("".to_string(), "".to_string()),
            Some(n) if n == "*" => (" (with LTS filter) ".into(), " --lts".into()),
            Some(n) => (format!(" (with LTS filter '{n}') "), format!(" --lts={n}")),
        };
        match &version {
            Some(v) if offline => eprintln!("Version '{v}' {}not found locally or in cache - try `nvm ls{f2}` to browse available versions.", f1.trim_start()),
            Some(v) => eprintln!("Version '{v}' {}not found - try `nvm ls-remote{f2}` to browse available versions.", f1.trim_start()),
            None => eprintln!("Version with LTS filter '{}' not found - try `nvm ls-remote{f2}` to browse available versions.", flags_lts.clone().unwrap_or_default()),
        }
        return 3;
    };
    let typed = version.clone().unwrap_or_else(|| {
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
            println!("Downloading and installing node {ver}...");
            let mut cmd = Command::new(remote::fetch_bin());
            cmd.args(["install", &remote::mirror(), &ver, &cx.nvm_dir])
                .args(&passthru);
            let st = cmd.status();
            let code = st.ok().and_then(|s| s.code()).unwrap_or(1);
            if code != 0 {
                if bin_only {
                    eprintln!("Binary download failed. Download from source aborted.");
                    return 2;
                }
                eprintln!("Binary download failed, trying source.");
                eprintln!("Source builds are not supported by this build of nvm.");
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
    let r = cmd_use(cx, &[ver.clone()]);
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

fn cmd_uninstall(cx: &mut Ctx, rest: &[String]) -> i32 {
    if rest.len() != 1 {
        eprintln!("Usage: nvm uninstall <version>\n       nvm uninstall --lts\n       nvm uninstall --lts=<LTS name>\n  {HELP_HINT}");
        return 127;
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
    let ver = match cx.st.version(&arg) {
        Ok(v) => v,
        Err(_) => {
            println!("Version '{shown}' is not installed.");
            return 0;
        }
    };
    if !cx.st.is_installed(&ver) {
        println!("Version '{ver}' (inferred from {arg}) is not installed.");
        return 0;
    }
    if cx.st.ls_current(&env::var("PATH").unwrap_or_default()) == ver {
        eprintln!(
            "nvm: Cannot uninstall currently-active node version, {ver} (inferred from {arg})."
        );
        return 1;
    }
    let _ = fs::remove_dir_all(cx.st.version_dir(&ver));
    for d in [format!(".cache/src/node-{ver}/files")] {
        let _ = fs::remove_dir_all(cx.st.dir.join(d));
    }
    if let Ok(rd) = fs::read_dir(cx.st.dir.join(".cache/bin")) {
        for e in rd.flatten() {
            if e.file_name()
                .to_string_lossy()
                .starts_with(&format!("node-{ver}-"))
            {
                let _ = fs::remove_dir_all(e.path().join("files"));
            }
        }
    }
    println!("Uninstalled node {ver}");
    let mut names = vec![];
    walk_aliases(&cx.st.dir.join("alias"), "", &mut names);
    names.sort();
    for n in names {
        if fs::read_to_string(cx.st.alias_path(&n))
            .map(|c| c.contains(&ver))
            .unwrap_or(false)
        {
            cmd_unalias(cx, &[n]);
        }
    }
    0
}

/// Resolve `[--silent] [--lts[=x]] [version]` for exec/run. Returns (version, silent, rest args).
fn valid_versionish(s: &str) -> bool {
    if matches!(s, "stable" | "unstable" | "iojs" | "node" | "current") {
        return true;
    }
    let s = s.strip_prefix("iojs-").unwrap_or(s);
    let s = s.strip_prefix('v').unwrap_or(s).trim_end_matches('.');
    let parts: Vec<&str> = s.split('.').collect();
    !s.is_empty()
        && parts.len() <= 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

fn warn_fallback(cmd: &str) {
    eprintln!("WARNING: `nvm {cmd}` was invoked without a version argument and without an .nvmrc file.\n  Falling back to the active node version; this will become an error in a future release.\n  Pass `current` explicitly (e.g. `nvm {cmd} current ...`) to silence this warning.");
}

/// Shared `exec` algorithm (§6.5). `args` = everything after the flags (first non-empty element is the version slot).
fn exec_core(cx: &mut Ctx, silent: bool, lts: Option<String>, mut args: Vec<String>) -> i32 {
    while args.first().map_or(false, |a| a.is_empty()) {
        args.remove(0);
    }
    let mut provided = args.first().cloned().unwrap_or_default();
    let mut version: String;
    let mut source = "";
    if let Some(n) = &lts {
        provided = format!("lts/{n}");
        version = provided.clone();
        source = "lts";
    } else if !provided.is_empty() {
        version = cx.st.version(&provided).unwrap_or_else(|e| e.to_string());
        if version == "N/A" && !valid_versionish(&provided) {
            provided = rc_version(silent).unwrap_or_default();
            version = cx.st.version(&provided).unwrap_or_else(|e| e.to_string());
            if !provided.is_empty() {
                source = "nvmrc";
            }
        } else {
            source = "arg";
            args.remove(0);
        }
    } else {
        version = String::new();
    }
    if source.is_empty() {
        if !silent {
            warn_fallback("exec");
        }
        provided = "current".into();
        version = cx.st.version("current").unwrap_or_default();
    }
    if !ensure_installed(cx, &provided, false) {
        return 1;
    }
    let resolved = cx.st.version(&version).unwrap_or_else(|e| e.to_string());
    if !silent {
        let npm = npm_ver_suffix(cx, &resolved);
        match &lts {
            Some(n) if n == "*" => println!("Running node latest LTS -> {resolved}{npm}"),
            Some(n) => println!("Running node LTS \"{n}\" -> {resolved}{npm}"),
            None => println!("Running node {resolved}{npm}"),
        }
    }
    run_with(cx, &resolved, &args)
}

fn parse_exec_flags(rest: &[String]) -> Result<(bool, Option<String>, Vec<String>), i32> {
    let mut silent = false;
    let mut lts = None;
    let mut i = 0;
    while i < rest.len() {
        let a = &rest[i];
        if a == "--silent" {
            silent = true;
        } else if a == "--lts" {
            lts = Some("*".to_string());
        } else if let Some(n) = a.strip_prefix("--lts=") {
            lts = Some(n.to_string());
        } else if a == "--" {
            break;
        } else if a.starts_with("--") {
            eprintln!("Unsupported option \"{a}\".");
            return Err(55);
        } else if a.is_empty() {
        } else {
            break;
        }
        i += 1;
    }
    let mut tail = rest[i..].to_vec();
    if tail.first().map_or(false, |a| a == "--") {
        tail.remove(0);
        // `--` ends option parsing; the next token is still the version slot
    }
    Ok((silent, lts, tail))
}

fn run_with(cx: &Ctx, ver: &str, cmd: &[String]) -> i32 {
    use std::os::unix::process::CommandExt;
    if cmd.is_empty() {
        return 0;
    }
    let vd = cx.st.version_dir(ver).to_string_lossy().into_owned();
    let path = pathedit::change_path(
        &env::var("PATH").unwrap_or_default(),
        "/bin",
        &vd,
        &cx.nvm_dir,
    );
    let e = Command::new(&cmd[0])
        .args(&cmd[1..])
        .env("PATH", path)
        .env("NVM_BIN", format!("{vd}/bin"))
        .env("NVM_INC", format!("{vd}/include/node"))
        .env("NODE_VERSION", ver)
        .exec();
    eprintln!("{}: {e}", cmd[0]);
    127
}

use std::process::Command;

fn cmd_exec(cx: &mut Ctx, rest: &[String]) -> i32 {
    match parse_exec_flags(rest) {
        Ok((silent, lts, tail)) => exec_core(cx, silent, lts, tail),
        Err(c) => c,
    }
}
fn cmd_run(cx: &mut Ctx, rest: &[String]) -> i32 {
    let (silent, lts, mut tail) = match parse_exec_flags(rest) {
        Ok(x) => x,
        Err(c) => return c,
    };
    while tail.first().map_or(false, |a| a.is_empty()) {
        tail.remove(0);
    }
    let mut version = String::new();
    let mut provided = String::new();
    let mut from_rc = false;
    let mut checked_rc = false;
    let mut rc_ver: Option<String> = None;
    if tail.is_empty() && lts.is_none() {
        rc_ver = rc_version(silent).ok();
        checked_rc = true;
        if let Some(v) = &rc_ver {
            version = cx.st.version(v).unwrap_or_else(|e| e.to_string());
        }
        if version.is_empty() || version == "N/A" {
            eprintln!("Usage: nvm run [<version>] [<args>]\n  Provide a <version>, or run from a directory containing an .nvmrc file.\n  {HELP_HINT}");
            return 127;
        }
    }
    if lts.is_none() {
        provided = tail.first().cloned().unwrap_or_default();
        if !provided.is_empty() {
            version = cx.st.version(&provided).unwrap_or_else(|e| e.to_string());
            if version == "N/A" && !valid_versionish(&provided) {
                if !checked_rc {
                    rc_ver = rc_version(silent).ok();
                }
                let rv = match rc_ver.clone() {
                    Some(v) => v,
                    None => {
                        if !silent {
                            warn_fallback("run");
                        }
                        cx.st.version("current").unwrap_or_default()
                    }
                };
                provided = rv.clone();
                from_rc = true;
                version = cx.st.version(&rv).unwrap_or_else(|e| e.to_string());
            } else {
                tail.remove(0);
            }
        }
    }
    if lts.is_some() {
        version = String::new();
    }
    if version == "N/A" {
        ensure_installed(cx, &provided, from_rc);
        return 1;
    }
    let mut args: Vec<String> = vec![];
    if lts.is_none() {
        args.push(version);
    }
    args.push("node".into());
    args.extend(tail);
    exec_core(cx, silent, lts, args)
}
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
                    .map_or(false, |a| a.as_secs() > stale * 60);
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

fn cmd_debug(cx: &mut Ctx) -> i32 {
    let run = |c: &str, a: &[&str]| {
        Command::new(c)
            .args(a)
            .output()
            .ok()
            .map(|o| {
                String::from_utf8_lossy(&[o.stdout, o.stderr].concat())
                    .trim()
                    .to_string()
            })
            .unwrap_or_default()
    };
    eprintln!("nvm --version: v0.40.8");
    eprintln!(
        "nvm-core: {} ({}-{})",
        env!("CARGO_PKG_VERSION"),
        env::consts::OS,
        env::consts::ARCH
    );
    if let Ok(t) = env::var("TERM_PROGRAM") {
        eprintln!("$TERM_PROGRAM: {t}");
    }
    let shell = env::var("SHELL").unwrap_or_default();
    eprintln!("$SHELL: {shell}");
    eprintln!("$SHLVL: {}", env::var("SHLVL").unwrap_or_default());
    eprintln!("whoami: '{}'", run("whoami", &[]));
    eprintln!("${{HOME}}: {}", env::var("HOME").unwrap_or_default());
    eprintln!("${{NVM_DIR}}: '{}'", sanitize(cx, &cx.nvm_dir));
    eprintln!(
        "${{PATH}}: {}",
        sanitize(cx, &env::var("PATH").unwrap_or_default())
    );
    eprintln!(
        "$PREFIX: '{}'",
        sanitize(cx, &env::var("PREFIX").unwrap_or_default())
    );
    eprintln!(
        "${{NPM_CONFIG_PREFIX}}: '{}'",
        sanitize(cx, &env::var("NPM_CONFIG_PREFIX").unwrap_or_default())
    );
    eprintln!(
        "$NVM_NODEJS_ORG_MIRROR: '{}'",
        env::var("NVM_NODEJS_ORG_MIRROR").unwrap_or_default()
    );
    eprintln!(
        "$NVM_IOJS_ORG_MIRROR: '{}'",
        env::var("NVM_IOJS_ORG_MIRROR").unwrap_or_default()
    );
    if !shell.is_empty() {
        eprintln!(
            "shell version: '{}'",
            run(&shell, &["--version"]).lines().next().unwrap_or("")
        );
    }
    let un = run("uname", &["-a"]);
    let un: Vec<&str> = un
        .split_whitespace()
        .enumerate()
        .filter(|(i, _)| *i != 1)
        .map(|(_, w)| w)
        .collect();
    eprintln!("uname -a: '{}'", un.join(" "));
    eprintln!("checksum binary: 'built-in sha256'");
    let osv = if cfg!(target_os = "macos") {
        run("sw_vers", &[])
            .lines()
            .filter_map(|l| l.split_whitespace().nth(1))
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        fs::read_to_string("/etc/os-release")
            .ok()
            .and_then(|t| {
                t.lines().find_map(|l| {
                    l.strip_prefix("PRETTY_NAME=")
                        .map(|v| v.trim_matches('"').to_string())
                })
            })
            .unwrap_or_default()
    };
    if !osv.is_empty() {
        eprintln!("OS version: {osv}");
    }
    let cur = cx.st.ls_current(&env::var("PATH").unwrap_or_default());
    eprintln!("nvm current: {cur}");
    for (label, tool) in [
        ("which node", "node"),
        ("which iojs", "iojs"),
        ("which npm", "npm"),
    ] {
        let p = env::var("PATH")
            .unwrap_or_default()
            .split(':')
            .map(|d| std::path::Path::new(d).join(tool))
            .find(|f| f.is_file())
            .map(|f| f.display().to_string())
            .unwrap_or_default();
        eprintln!("{label}: {}", sanitize(cx, &p));
    }
    eprintln!(
        "npm config get prefix: {}",
        sanitize(cx, &run("npm", &["config", "get", "prefix"]))
    );
    eprintln!(
        "npm root -g: {}",
        sanitize(cx, &run("npm", &["root", "-g"]))
    );
    42
}
