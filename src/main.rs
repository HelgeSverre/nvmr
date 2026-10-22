mod colors;
mod eval;
mod install;
mod npm;
mod pathedit;
mod remote;
mod store;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};
use std::{env, fs};

use eval::{flush_eval, Eval};
use install::{cmd_install, cmd_uninstall};
use store::{ensure_v, find_nvmrc, parse_nvmrc, Store};

struct Ctx {
    pal: Option<colors::Pal>,
    st: Store,
    ev: Eval,
    nvm_dir: String,
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let nvm_dir = env::var("NVM_DIR").map_or_else(
        |_| format!("{}/.nvm", env::var("HOME").unwrap_or_default()),
        |d| d.trim_end_matches('/').to_string(),
    );
    let mut cx = Ctx {
        pal: colors::detect(&args),
        st: Store {
            dir: PathBuf::from(&nvm_dir),
        },
        ev: Eval::new(env::var("NVM_SHELL").unwrap_or_else(|_| "bash".into())),
        nvm_dir,
    };
    let rc = run(&mut cx, &args);
    flush_eval(&cx.ev.buf);
    let _ = std::io::stdout().flush();
    exit(rc);
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
            let a = rest.first().map_or("", String::as_str);
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
    let pat = rest.first().map_or("", String::as_str);
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
        "io.js" => cx.st.version("iojs"),
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
    if let Ok(v) = rc_version(silent) {
        Ok((v, true))
    } else {
        if which {
            eprintln!("Usage: nvm which [current | <version>]\n  Provide a <version>, or run from a directory containing an .nvmrc file.\n  {HELP_HINT}");
        } else {
            eprintln!("Please see `nvm --help` or https://github.com/nvm-sh/nvm#nvmrc for more information.");
        }
        Err(127)
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
        Ok(v) if v == "system" => {
            if let Some((p, _)) = system_node(cx) {
                println!("{p}");
                0
            } else {
                eprintln!("System version of node not found.");
                127
            }
        }
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
        let n = Path::new(d).join("node");
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
        if let Ok(v) = ver.clone() {
            if let Ok(w) = cx.st.version(&v) {
                let target = env::current_dir().unwrap_or_default().join(".nvmrc");
                if fs::write(&target, format!("{w}\n")).is_ok() {
                    if !silent {
                        println!("Wrote version number ({w}) to .nvmrc");
                    }
                } else {
                    if !silent {
                        eprintln!("Warning: Unable to write version number ({w}) to .nvmrc");
                    }
                    return 3;
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
            if die_on_prefix(cx, delete, &ucmd, &vd).is_err() {
                return 11;
            }
            if !silent {
                let what = match v.strip_prefix("iojs-") {
                    Some(b) => format!("io.js {b}"),
                    None => format!("node {v}"),
                };
                match cx.st.npm_version(&v) {
                    Some(n) => println!("Now using {what} (npm v{n})"),
                    None => println!("Now using {what}"),
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

/// Exact port of upstream's `grep -Ee '^(prefix|globalconfig) *='`, run over raw bytes so that
/// files containing non-UTF-8 bytes behave like upstream's grep: the key must start the line and
/// only spaces may separate it from the `=` (tabs do not count, matching the regex).
fn npmrc_line_bad(l: &[u8]) -> bool {
    for key in ["prefix", "globalconfig"] {
        if let Some(rest) = l.strip_prefix(key.as_bytes()) {
            let sp = rest.iter().take_while(|&&b| b == b' ').count();
            return rest[sp..].first() == Some(&b'=');
        }
    }
    false
}

fn npmrc_bad(p: &Path) -> bool {
    fs::read(p).is_ok_and(|t| t.split(|&b| b == b'\n').any(npmrc_line_bad))
}

/// §5.4. Err(rc) is the helper's code (3, 4, 10); `use` turns any failure into 11.
/// Which npm config layer `nvm use --delete-prefix` edits to clear an npmrc setting.
enum NpmrcScope {
    /// `npm config delete --userconfig=<file>`
    UserConfig,
    /// `npm config --global delete`
    Global,
    /// `npm config delete`, which finds the project `.npmrc` from the working directory
    Project,
}

fn die_on_prefix(cx: &mut Ctx, delete: bool, cmd: &str, vd: &str) -> Result<(), i32> {
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
                project = PathBuf::new();
                break;
            }
        }
    }
    let project_rc = format!("{}/.npmrc", project.display());
    let checks = [
        (
            "builtin npmrc",
            format!("{vd}/lib/node_modules/npm/npmrc"),
            NpmrcScope::UserConfig,
        ),
        (
            "global npmrc",
            format!("{vd}/etc/npmrc"),
            NpmrcScope::Global,
        ),
        ("user’s .npmrc", home_rc, NpmrcScope::UserConfig),
        ("project npmrc", project_rc, NpmrcScope::Project),
    ];
    for (label, file, scope) in &checks {
        if !npmrc_bad(Path::new(file)) {
            continue;
        }
        if !delete {
            eprintln!("Your {label} file ({})\nhas a `globalconfig` and/or a `prefix` setting, which are incompatible with nvm.\nRun `{cmd}` to unset it.", sanitize(cx, file));
            return Err(10);
        }
        let userconfig = format!("--userconfig={file}");
        for key in ["prefix", "globalconfig"] {
            let mut args = vec!["config"];
            if matches!(scope, NpmrcScope::Global) {
                args.push("--global");
            }
            args.extend(["--loglevel=warn", "delete", key]);
            if matches!(scope, NpmrcScope::UserConfig) {
                args.push(&userconfig);
            }
            run_npm(&args);
        }
    }
    Ok(())
}

fn has_manpath() -> bool {
    env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .any(|d| Path::new(d).join("manpath").is_file())
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
    if alias.as_deref().is_some_and(|a| a.contains('#')) {
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
        let pre = cx.st.version(&t);
        if pre == Err("N/A") {
            eprintln!("! WARNING: Version '{t}' does not exist.");
        }
        let _ = fs::create_dir_all(cx.st.dir.join("alias"));
        if fs::write(cx.st.alias_path(&name), format!("{t}\n")).is_err() {
            return 1;
        }
        // upstream formats with the resolution computed *before* the alias was written
        println!(
            "{}",
            alias_line_with(
                cx,
                &name,
                &t,
                false,
                Some(pre.unwrap_or_else(std::string::ToString::to_string))
            )
        );
        return 0;
    }
    match alias.filter(|a| !a.is_empty()) {
        None => {
            list_aliases(cx, None);
            0
        }
        Some(pat) if pat.starts_with("lts/") => {
            if let Some(t) = cx.st.read_alias(&pat) {
                println!("{t}");
                0
            } else {
                eprintln!("Alias does not exist.");
                2
            }
        }
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
    alias_line_with(cx, name, dest, implicit, None)
}

fn alias_line_with(
    cx: &Ctx,
    name: &str,
    dest: &str,
    implicit: bool,
    pre: Option<String>,
) -> String {
    let path = env::var("PATH").unwrap_or_default();
    let cur = cx.st.ls_current(&path);
    let resolved = pre.unwrap_or_else(|| {
        cx.st
            .version(dest)
            .unwrap_or_else(std::string::ToString::to_string)
    });
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
fn walk_aliases(dir: &Path, prefix: &str, out: &mut Vec<String>) {
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
    let inst = cx.st.installed_flavour(true, false);
    let iojs = cx
        .st
        .installed_flavour(false, true)
        .pop()
        .map(|((maj, min, _), _)| format!("iojs-v{maj}.{min}"));
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
    implicit("iojs", iojs);
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
            let numeric = store::bare(p)
                .trim_start_matches('v')
                .starts_with(|c: char| c.is_ascii_digit());
            match p.as_str() {
                "node" => cx
                    .st
                    .installed_flavour(true, false)
                    .into_iter()
                    .map(|x| x.1)
                    .collect(),
                "iojs" => cx
                    .st
                    .installed_flavour(false, true)
                    .into_iter()
                    .map(|x| x.1)
                    .collect(),
                "v" => cx.st.installed().into_iter().map(|x| x.1).collect(),
                _ if numeric => cx.st.matching(p),
                _ => match cx.st.version(p) {
                    Ok(v) if cx.st.is_installed(&v) => vec![v],
                    _ => vec![],
                },
            }
        }
        None => cx.st.installed().into_iter().map(|x| x.1).collect(),
    };
    let want_system = pat.is_none_or(|p| p == "system") && cx.st.system_node().is_some();
    let list: Vec<String> = if pat.is_some_and(|p| p == "system") {
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
                            .is_some_and(|c| c.is_ascii_digit() || c == 'v') =>
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
        if pat
            .as_ref()
            .is_none_or(|p| format!("{n}.").starts_with(&format!("{}.", p.trim_end_matches('.'))))
        {
            println!("{n}");
        }
    }
    0
}

/// `nvm-exec` replacement: NODE_VERSION or .nvmrc, then exec the command with the selected node first on PATH.
fn cmd_exec_internal(cx: &mut Ctx, rest: &[String]) -> i32 {
    let sel = if let Some(v) = env::var("NODE_VERSION").ok().filter(|v| !v.is_empty()) {
        match cx.st.version(&v) {
            Ok(r) if cx.st.is_installed(&r) => Ok(r),
            _ => {
                not_installed(cx, &v, false);
                return 127;
            }
        }
    } else {
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
    };
    if let Ok(v) = sel {
        run_with(cx, &v, rest)
    } else {
        eprintln!("nvm-exec: unable to select a node version\n  Set `NODE_VERSION` (e.g. `NODE_VERSION=default`), or add an `.nvmrc` file.");
        127
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
        } else if !a.starts_with('-') && a != "--" {
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
            cx.pal.is_none() && lts.is_none() && pat.as_deref().is_some_and(valid_versionish);
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
    if node_empty || (pat.is_some() && lts.is_none() && !node_alias) {
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
    if let Some(r) = remote::resolve(
        &cx.st,
        &rows,
        pat.as_deref().unwrap_or("node"),
        lts.as_deref(),
    ) {
        println!("{}", r.ver);
        0
    } else {
        println!("N/A");
        3
    }
}

fn npm_ver_suffix(cx: &Ctx, v: &str) -> String {
    cx.st
        .npm_version(v)
        .map(|n| format!(" (npm v{n})"))
        .unwrap_or_default()
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
    while args.first().is_some_and(std::string::String::is_empty) {
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
        version = cx
            .st
            .version(&provided)
            .unwrap_or_else(std::string::ToString::to_string);
        if version == "N/A" && !valid_versionish(&provided) {
            provided = rc_version(silent).unwrap_or_default();
            version = cx
                .st
                .version(&provided)
                .unwrap_or_else(std::string::ToString::to_string);
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
    let resolved = cx
        .st
        .version(&version)
        .unwrap_or_else(std::string::ToString::to_string);
    if !silent {
        let npm = npm_ver_suffix(cx, &resolved);
        match &lts {
            Some(n) if n == "*" => println!("Running node latest LTS -> {resolved}{npm}"),
            Some(n) => println!("Running node LTS \"{n}\" -> {resolved}{npm}"),
            None => match resolved.strip_prefix("iojs-") {
                Some(b) => println!("Running io.js {b}{npm}"),
                None => println!("Running node {resolved}{npm}"),
            },
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
    if tail.first().is_some_and(|a| a == "--") {
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
    while tail.first().is_some_and(std::string::String::is_empty) {
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
            version = cx
                .st
                .version(v)
                .unwrap_or_else(std::string::ToString::to_string);
        }
        if version.is_empty() || version == "N/A" {
            eprintln!("Usage: nvm run [<version>] [<args>]\n  Provide a <version>, or run from a directory containing an .nvmrc file.\n  {HELP_HINT}");
            return 127;
        }
    }
    if lts.is_none() {
        provided = tail.first().cloned().unwrap_or_default();
        if !provided.is_empty() {
            version = cx
                .st
                .version(&provided)
                .unwrap_or_else(std::string::ToString::to_string);
            if version == "N/A" && !valid_versionish(&provided) {
                if !checked_rc {
                    rc_ver = rc_version(silent).ok();
                }
                let rv = if let Some(v) = rc_ver {
                    v
                } else {
                    if !silent {
                        warn_fallback("run");
                    }
                    cx.st.version("current").unwrap_or_default()
                };
                provided = rv.clone();
                from_rc = true;
                version = cx
                    .st
                    .version(&rv)
                    .unwrap_or_else(std::string::ToString::to_string);
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
        args.push(version.clone());
    }
    args.push(
        if version.starts_with("iojs-") {
            "iojs"
        } else {
            "node"
        }
        .into(),
    );
    args.extend(tail);
    exec_core(cx, silent, lts, args)
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
            .map(|d| Path::new(d).join(tool))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn npmrc_line_matches_upstream_grep() {
        for bad in [
            "prefix=x",
            "prefix = x",
            "globalconfig  =y",
            "prefix=",
            "prefix=\r",
        ] {
            assert!(
                npmrc_line_bad(bad.as_bytes()),
                "{bad:?} should trigger the prefix guard"
            );
        }
        for ok in [
            "",
            " prefix=x",
            "prefixs=x",
            "prefix\t=x",
            "PREFIX=x",
            "myprefix=x",
            "global  = x",
            "globalconfig\\=x",
        ] {
            assert!(
                !npmrc_line_bad(ok.as_bytes()),
                "{ok:?} should not trigger the prefix guard"
            );
        }
    }

    #[test]
    fn npmrc_guard_reads_non_utf8() {
        let p = std::env::temp_dir().join(format!("nvmcore-npmrc-{}", std::process::id()));
        let _ = fs::remove_file(&p);
        fs::write(&p, b"prefix=\xff\xfe").unwrap();
        assert!(npmrc_bad(&p));
        fs::write(&p, b"ok=\xff\xfe\n").unwrap();
        assert!(!npmrc_bad(&p));
        let _ = fs::remove_file(&p);
    }
}
