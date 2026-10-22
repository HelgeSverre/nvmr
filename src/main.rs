mod pathedit;
mod remote;
mod store;

use std::{env, fs, io::Write, path::PathBuf, process::exit};
use store::{ensure_v, find_nvmrc, parse_nvmrc, Store};

/// Collects shell statements for the eval channel (§1.1).
struct Eval {
    shell: String,
    buf: String,
}

fn q(v: &str) -> String {
    format!("'{}'", v.replace('\'', "'\\''"))
}

impl Eval {
    fn export(&mut self, k: &str, v: &str) {
        if self.shell == "fish" {
            let val = if k.ends_with("PATH") {
                v.split(':').map(|p| format!("'{}'", p.replace('\\', "\\\\").replace('\'', "\\'"))).collect::<Vec<_>>().join(" ")
            } else {
                format!("'{}'", v.replace('\\', "\\\\").replace('\'', "\\'"))
            };
            self.buf += &format!("set -gx {k} {val}\n");
        } else {
            self.buf += &format!("export {k}={}\n", q(v));
        }
    }
    fn unset(&mut self, k: &str) {
        self.buf += &if self.shell == "fish" { format!("set -e {k}\n") } else { format!("unset {k}\n") };
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
    st: Store,
    ev: Eval,
    nvm_dir: String,
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let nvm_dir = env::var("NVM_DIR").map(|d| d.trim_end_matches('/').to_string()).unwrap_or_else(|_| {
        format!("{}/.nvm", env::var("HOME").unwrap_or_default())
    });
    let mut cx = Ctx {
        st: Store { dir: PathBuf::from(&nvm_dir) },
        ev: Eval { shell: env::var("NVM_SHELL").unwrap_or_else(|_| "bash".into()), buf: String::new() },
        nvm_dir,
    };
    let rc = run(&mut cx, &args);
    if !cx.ev.buf.is_empty() {
        if let Ok(f) = env::var("NVM_EVAL_FILE") {
            let _ = fs::write(f, &cx.ev.buf);
        }
    }
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
            println!("{}", cx.st.ls_current(&env::var("PATH").unwrap_or_default()));
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
                let _ = fs::remove_dir_all(&c);
                let _ = fs::create_dir_all(&c);
                println!("nvm cache cleared.");
                0
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
        "__auto" => cmd_auto(cx, rest),
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
    let pat = rest.iter().find(|a| *a != "--").map(String::as_str).unwrap_or("current");
    if !guard(pat) {
        println!("N/A");
        return 3;
    }
    if pat == "current" {
        println!("{}", cx.st.ls_current(&env::var("PATH").unwrap_or_default()));
        return 0;
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

/// Version arg or `.nvmrc`. Returns (pattern, from_rc) or exit code.
fn pick_pattern(rest: &[String], silent: bool, which: bool) -> Result<(String, bool), i32> {
    if let Some(v) = rest.iter().find(|a| !a.starts_with("--") && !matches!(a.as_str(), "-w")) {
        return Ok((v.clone(), false));
    }
    let cwd = env::current_dir().unwrap_or_default();
    let Some(rc) = find_nvmrc(&cwd) else {
        if which {
            eprintln!("No version provided and no .nvmrc file found\nUsage: nvm which [current | <version>]\n  Provide a <version>, or run from a directory containing an .nvmrc file.\n  {HELP_HINT}");
        } else {
            eprintln!("No version provided and no .nvmrc file found\nPlease see `nvm --help` or https://github.com/nvm-sh/nvm#nvmrc for more information.");
        }
        return Err(127);
    };
    let content = fs::read_to_string(&rc).unwrap_or_default();
    match parse_nvmrc(&content) {
        Ok(v) => {
            if !silent {
                println!("Found '{}' with version <{v}>", rc.display());
            }
            Ok((v, true))
        }
        Err(lines) => {
            eprintln!("invalid .nvmrc!\nall non-commented content (anything after # is a comment) must be either:\n  - a single bare nvm-recognized version-ish\n  - or, multiple distinct key-value pairs, each key/value separated by a single equals sign (=)\n\nadditionally, a single bare nvm-recognized version-ish must be present (after stripping comments).\n\nnon-commented content parsed:");
            if !lines.is_empty() {
                eprintln!("{lines}");
            }
            eprintln!("Please see `nvm --help` or https://github.com/nvm-sh/nvm#nvmrc for more information.");
            Err(127)
        }
    }
}
fn not_installed(cx: &Ctx, arg: &str, from_rc: bool) {
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
    let (pat, rc) = match pick_pattern(rest, silent, true) {
        Ok(x) => x,
        Err(c) => return c,
    };
    if pat == "system" {
        return match system_node(cx) {
            Some((p, _)) => {
                println!("{p}");
                0
            }
            None => {
                eprintln!("System version of node not found.");
                127
            }
        };
    }
    let cur = pat == "current";
    let r = if cur { Ok(cx.st.ls_current(&env::var("PATH").unwrap_or_default())) } else { cx.st.version(&pat) };
    match r {
        Err("∞") => {
            eprintln!("The alias \"{pat}\" leads to an infinite loop. Aborting.");
            8
        }
        Ok(v) if cx.st.is_installed(&v) => {
            println!("{}/bin/node", cx.st.version_dir(&v).display());
            0
        }
        _ => {
            not_installed(cx, &pat, rc);
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
            let v = Command::new(&n).arg("--version").output().ok().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
            return Some((n.to_string_lossy().into_owned(), v));
        }
    }
    None
}
fn cmd_use(cx: &mut Ctx, rest: &[String]) -> i32 {
    let silent = rest.iter().any(|a| a == "--silent");
    let save = rest.iter().any(|a| a == "--save" || a == "-w");
    let mut lts: Option<String> = None;
    for a in rest {
        if a == "--lts" {
            lts = Some("*".into());
        } else if let Some(n) = a.strip_prefix("--lts=") {
            lts = Some(n.to_string());
        }
    }
    let positional = rest.iter().any(|a| !a.starts_with('-'));
    let lts_only = lts.is_some() && !positional;
    let (pat, from_rc) = if lts_only {
        (format!("lts/{}", lts.clone().unwrap()), false)
    } else {
        match pick_pattern(rest, silent, false) {
            Ok(x) => x,
            Err(c) => return c,
        }
    };
    if !guard(&pat) {
        eprintln!("invalid version: {pat}");
        return 3;
    }
    let ver = if pat == "system" { Ok("system".to_string()) } else { cx.st.version(&pat) };
    if save {
        if let Ok(v) = &ver {
            let target = env::current_dir().unwrap_or_default().join(".nvmrc");
            match fs::write(&target, format!("{v}\n")) {
                Ok(_) => println!("Wrote version number ({v}) to .nvmrc"),
                Err(_) => {
                    eprintln!("Warning: Unable to write version number ({v}) to .nvmrc");
                    return 3;
                }
            }
        }
    }
    match ver {
        Err("∞") => {
            if !silent {
                eprintln!("The alias \"{pat}\" leads to an infinite loop. Aborting.");
            }
            8
        }
        Ok(ref v) if v == "system" => {
            let Some((_, sv)) = system_node(cx) else {
                if !silent {
                    eprintln!("System version of node not found.");
                }
                return 127;
            };
            let path = env::var("PATH").unwrap_or_default();
            let np = pathedit::strip_path(&path, "/bin", &cx.nvm_dir);
            cx.ev.export("PATH", &np);
            cx.ev.unset("NVM_BIN");
            cx.ev.unset("NVM_INC");
            cx.ev.rehash();
            if !silent {
                let npm = Command::new("npm").arg("--version").env("PATH", &np).output().ok().filter(|o| o.status.success()).map(|o| format!(" (npm v{})", String::from_utf8_lossy(&o.stdout).trim())).unwrap_or_default();
                println!("Now using system version of node: {sv}{npm}");
            }
            0
        }
        Ok(v) if cx.st.is_installed(&v) => {
            let vd = cx.st.version_dir(&v).to_string_lossy().into_owned();
            let path = env::var("PATH").unwrap_or_default();
            cx.ev.export("PATH", &pathedit::change_path(&path, "/bin", &vd, &cx.nvm_dir));
            if let Ok(m) = env::var("MANPATH") {
                cx.ev.export("MANPATH", &pathedit::change_path(&m, "/share/man", &vd, &cx.nvm_dir));
            }
            cx.ev.export("NVM_BIN", &format!("{vd}/bin"));
            cx.ev.export("NVM_INC", &format!("{vd}/include/node"));
            if env::var("NVM_SYMLINK_CURRENT").as_deref() == Ok("true") {
                let cur = cx.st.dir.join("current");
                let _ = fs::remove_file(&cur);
                let _ = std::os::unix::fs::symlink(&vd, &cur);
            }
            cx.ev.rehash();
            if !silent {
                match cx.st.npm_version(&v) {
                    Some(n) => println!("Now using node {v} (npm v{n})"),
                    None => println!("Now using node {v}"),
                }
            }
            0
        }
        _ => {
            // Upstream quirk: `use --lts` with nothing installed for that line is silent.
            if !silent && !lts_only {
                not_installed(cx, &pat, from_rc);
            }
            3
        }
    }
}
fn cmd_deactivate(cx: &mut Ctx, rest: &[String]) -> i32 {
    let silent = rest.iter().any(|a| a == "--silent");
    let path = env::var("PATH").unwrap_or_default();
    let np = pathedit::strip_path(&path, "/bin", &cx.nvm_dir);
    if np == path {
        if !silent {
            eprintln!("Could not find {}/*/bin in ${{PATH}}", cx.nvm_dir);
        }
    } else {
        cx.ev.export("PATH", &np);
        if !silent {
            println!("{}/*/bin removed from ${{PATH}}", cx.nvm_dir);
        }
    }
    if let Ok(m) = env::var("MANPATH") {
        let nm = pathedit::strip_path(&m, "/share/man", &cx.nvm_dir);
        if nm != m {
            if nm.is_empty() || nm == ":" {
                cx.ev.unset("MANPATH");
            } else {
                cx.ev.export("MANPATH", &nm);
            }
        }
    }
    cx.ev.unset("NVM_BIN");
    cx.ev.unset("NVM_INC");
    cx.ev.rehash();
    0
}

fn builtin(n: &str) -> bool {
    matches!(n, "stable" | "unstable" | "iojs" | "node" | "system")
}

fn cmd_alias(cx: &mut Ctx, rest: &[String]) -> i32 {
    let a: Vec<&String> = rest.iter().filter(|a| !a.starts_with("--")).collect();
    match a.as_slice() {
        [] => {
            list_aliases(cx, None);
            0
        }
        [pat] if pat.starts_with("lts/") => match cx.st.read_alias(pat) {
            Some(t) => {
                println!("{t}");
                0
            }
            None => {
                eprintln!("Alias {pat} doesn't exist!");
                3
            }
        },
        [pat] => {
            list_aliases(cx, Some(pat));
            0
        }
        [name, target] => {
            if target.is_empty() {
                return cmd_unalias(cx, &[name.to_string()]);
            }
            if name.contains('#') {
                eprintln!("Aliases with a comment delimiter (#) are not supported.");
                return 1;
            }
            if name.contains('/') {
                eprintln!("Aliases in subdirectories are not supported.");
                return 1;
            }
            if !guard(name) {
                eprintln!("invalid alias name: {name}");
                return 3;
            }
            if cx.st.version(target).is_err() {
                eprintln!("! WARNING: Version '{target}' does not exist.");
            }
            let _ = fs::create_dir_all(cx.st.dir.join("alias"));
            if fs::write(cx.st.alias_path(name), format!("{target}\n")).is_err() {
                return 1;
            }
            print_alias(cx, name, target);
            0
        }
        _ => {
            eprintln!("Usage: nvm alias [<pattern>] | nvm alias <name> <version>");
            127
        }
    }
}

fn cmd_unalias(cx: &mut Ctx, rest: &[String]) -> i32 {
    let Some(name) = rest.first() else {
        eprintln!("Usage: nvm unalias <name>");
        return 127;
    };
    if !guard(name) {
        eprintln!("invalid alias name: {name}");
        return 3;
    }
    let p = cx.st.alias_path(name);
    let Some(old) = cx.st.read_alias(name) else {
        if builtin(name) {
            eprintln!("{name} is a default (built-in) alias and cannot be deleted.");
            return 1;
        }
        eprintln!("Alias {name} doesn't exist!");
        return 0;
    };
    let _ = fs::remove_file(p);
    println!("Deleted alias {name} - restore it with `nvm alias \"{name}\" \"{old}\"`");
    0
}

fn print_alias(cx: &Ctx, name: &str, dest: &str) {
    let resolved = cx.st.version(dest).unwrap_or_else(|e| e.to_string());
    let star = if cx.st.is_installed(&resolved) { " *" } else { "" };
    if resolved == dest {
        println!("{name} -> {dest}{star}");
    } else {
        println!("{name} -> {dest} (-> {resolved}{star})");
    }
}

fn walk_aliases(dir: &std::path::Path, prefix: &str, out: &mut Vec<String>) {
    for e in fs::read_dir(dir).into_iter().flatten().flatten() {
        let n = e.file_name().to_string_lossy().into_owned();
        let full = if prefix.is_empty() { n.clone() } else { format!("{prefix}/{n}") };
        if e.path().is_dir() {
            walk_aliases(&e.path(), &full, out);
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
    let lts: Vec<String> = lts.into_iter().filter(|n| n.starts_with(p)).collect();
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
    let implicit = |name: &str, val: Option<String>| {
        if has(name) || (pat.is_some() && name != p) {
            return;
        }
        match val {
            None => println!("{name} -> N/A (default)"),
            Some(d) => {
                let r = cx.st.version(&d).unwrap_or_else(|e| e.to_string());
                let star = if cx.st.is_installed(&r) { " *" } else { "" };
                if r == d {
                    println!("{name} -> {d}{star} (default)");
                } else {
                    println!("{name} -> {d} (-> {r}{star}) (default)");
                }
            }
        }
    };
    implicit("iojs", None);
    implicit("node", Some("stable".into()));
    if stable.is_some() {
        implicit("stable", stable);
    }
    implicit("unstable", unstable);
    for n in &lts {
        if let Some(t) = cx.st.read_alias(n) {
            print_alias(cx, n, &t);
        }
    }
}

/// Plain (no colour) `nvm ls`: §8 without annotations other than ` *`.
fn cmd_ls(cx: &mut Ctx, rest: &[String]) -> i32 {
    let no_alias = rest.iter().any(|a| a == "--no-alias");
    let pat = rest.iter().find(|a| !a.starts_with("--"));
    let cur = cx.st.ls_current(&env::var("PATH").unwrap_or_default());
    let list: Vec<String> = match pat {
        Some(p) => {
            if no_alias {
                eprintln!("`--no-alias` is not supported when a pattern is provided.");
                return 55;
            }
            let numeric = p.trim_start_matches('v').starts_with(|c: char| c.is_ascii_digit());
            if !numeric {
                match cx.st.version(p) {
                    Ok(v) if cx.st.is_installed(&v) => vec![v],
                    _ => vec![],
                }
            } else {
                let pfx = format!("{}.", ensure_v(p).trim_end_matches('.'));
                cx.st.installed().into_iter().map(|x| x.1).filter(|n| format!("{n}.").starts_with(&pfx) || *n == ensure_v(p)).collect()
            }
        }
        None => cx.st.installed().into_iter().map(|x| x.1).collect(),
    };
    if list.is_empty() {
        println!("{:>15}{}", "N/A", if pat.is_none() { " *" } else { "" });
        if pat.is_none() && !no_alias {
            list_aliases(cx, None);
        }
        return 3;
    }
    for v in &list {
        if *v == cur {
            println!("->{v:>13} *");
        } else {
            println!("{v:>15} *");
        }
    }
    if pat.is_none() && !no_alias {
        list_aliases(cx, None);
    }
    0
}

/// §9.1 `__auto use`: `.nvmrc` in ancestry, else `default`; no-op when already right.
fn cmd_auto(cx: &mut Ctx, rest: &[String]) -> i32 {
    if rest.first().map(String::as_str) != Some("use") {
        return 0;
    }
    let cwd = env::current_dir().unwrap_or_default();
    let want = find_nvmrc(&cwd)
        .and_then(|f| fs::read_to_string(f).ok())
        .and_then(|c| parse_nvmrc(&c).ok())
        .unwrap_or_else(|| "default".into());
    let Ok(target) = cx.st.version(&want) else { return 0 };
    if !cx.st.is_installed(&target) {
        return 0;
    }
    let path = env::var("PATH").unwrap_or_default();
    if cx.st.ls_current(&path) == target && env::var("NVM_BIN").is_ok() {
        return 0;
    }
    cmd_use(cx, &[target, "--silent".into()])
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
    if pat.is_none() && lts.is_none() {
        if let Ok(mut io) = remote::iojs_rows() {
            rows.append(&mut io);
            remote::sort_rows(&mut rows);
        }
    }
    let cur = cx.st.ls_current(&env::var("PATH").unwrap_or_default());
    let node_alias = pat.as_deref() == Some("node");
    let sel = remote::filter(&rows, lts.as_deref(), if node_alias { None } else { pat.as_deref() });

    if sel.is_empty() {
        println!("{:>15} *", "N/A");
        return 3;
    }
    let mut out = String::new();
    for r in sel {
        out += &remote::format_row(r, cx.st.is_installed(&r.ver), r.ver == cur, true);
        out.push('\n');
    }
    print!("{out}");
    // Upstream quirk (v0.40.8): a positional pattern without --lts exits 3 even on success.
    if pat.is_some() && lts.is_none() && !node_alias {
        3
    } else {
        0
    }
}

fn cmd_version_remote(cx: &mut Ctx, rest: &[String]) -> i32 {
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
    match remote::resolve(&cx.st, &rows, pat.as_deref().unwrap_or("node"), lts.as_deref()) {
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
    cx.st.npm_version(v).map(|n| format!(" (npm v{n})")).unwrap_or_default()
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
    for a in rest {
        match a.as_str() {
            "--lts" if version.is_none() => flags_lts = Some("*".into()),
            "--default" => set_default = true,
            "--save" | "-w" => save = true,
            "--skip-default-packages" => skip_pkgs = true,
            "--no-progress" => passthru.push(a.clone()),
            "--offline" => passthru.push(a.clone()),
            "-b" => bin_only = true,
            "-s" => src_only = true,
            "--latest-npm" => {}
            x if x.starts_with("--lts=") => flags_lts = Some(x[6..].to_string()),
            x if x.starts_with("--alias=") => alias = Some(x[8..].to_string()),
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
        eprintln!("--default and --alias are mutually exclusive, and may not be provided more than once");
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
                match rcf.clone().and_then(|f| fs::read_to_string(f).ok()).map(|c| parse_nvmrc(&c)) {
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
                return rc;
            }
        },
        o => o.clone(),
    };
    let offline = passthru.iter().any(|a| a == "--offline");
    let chosen: Option<String> = if offline {
        let pat = version.clone().unwrap_or_else(|| "lts/*".into());
        match cx.st.version(&pat) {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    } else {
        let rows = match remote::index(&cx.st) {
            Ok(r) => r,
            Err(c) => return c,
        };
        remote::resolve(&cx.st, &rows, version.as_deref().unwrap_or("node"), lts_for_resolve.as_deref()).map(|r| r.ver.clone())
    };
    let Some(ver) = chosen else {
        let (f1, f2) = match &flags_lts {
            None => ("".to_string(), "".to_string()),
            Some(n) if n == "*" => (" (with LTS filter) ".into(), " --lts".into()),
            Some(n) => (format!(" (with LTS filter '{n}') "), format!(" --lts={n}")),
        };
        match &version {
            Some(v) => eprintln!("Version '{v}' {}not found - try `nvm ls-remote{f2}` to browse available versions.", f1.trim_start()),
            None => eprintln!("Version with LTS filter '{}' not found - try `nvm ls-remote{f2}` to browse available versions.", flags_lts.clone().unwrap_or_default()),
        }
        return 3;
    };
    let typed = version.clone().unwrap_or_else(|| if flags_lts.as_deref() == Some("*") { "lts/*".into() } else { format!("lts/{}", flags_lts.clone().unwrap_or_default()) });

    let valid = |cx: &Ctx| cx.st.is_installed(&ver);
    let mut rc = 0;
    if valid(cx) {
        eprintln!("{ver} is already installed.");
    } else {
        println!("Downloading and installing node {ver}...");
        let _ = std::fs::create_dir_all(cx.st.dir.join(".cache/locks"));
        let mut cmd = Command::new(remote::fetch_bin());
        cmd.args(["install", &remote::mirror(), &ver, &cx.nvm_dir]).args(&passthru);
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
        if !valid(cx) {
            eprintln!("The install of {ver} reported success but failed verification; not activating it.");
            return 1;
        }
    }
    let r = cmd_use(cx, &[ver.clone()]);
    if r != 0 {
        rc = r;
    }
    // default alias
    if cx.st.read_alias("default").is_none() {
        let _ = fs::create_dir_all(cx.st.dir.join("alias"));
        let target = typed.clone();
        if fs::write(cx.st.alias_path("default"), format!("{target}\n")).is_ok() {
            print!("Creating default alias: ");
            print_alias(cx, "default", &target);
        }
    }
    if set_default {
        let _ = fs::write(cx.st.alias_path("default"), format!("{typed}\n"));
        print_alias(cx, "default", &typed);
    }
    if let Some(a) = alias {
        let _ = cmd_alias(cx, &[a, typed.clone()]);
    }
    if save {
        let _ = fs::write(env::current_dir().unwrap_or_default().join(".nvmrc"), format!("{ver}\n"));
        println!("Wrote version number ({ver}) to .nvmrc");
    }
    if !skip_pkgs {
        if let Some(c) = default_packages(cx, &ver) {
            if c != 0 {
                rc = c;
            }
        }
    }
    rc
}

fn default_packages(cx: &Ctx, ver: &str) -> Option<i32> {
    let f = cx.st.dir.join("default-packages");
    let txt = fs::read_to_string(&f).ok()?;
    let mut pkgs = vec![];
    for l in txt.lines() {
        let l = l.split('#').next().unwrap_or("").trim();
        if l.is_empty() {
            continue;
        }
        if l.split_whitespace().count() > 1 {
            eprintln!("Only one package per line is allowed in `{}`. Please remove any lines with multiple space-separated values.", f.display());
            return Some(1);
        }
        pkgs.push(l.to_string());
    }
    if pkgs.is_empty() {
        return None;
    }
    println!("Installing default global packages from ${{NVM_DIR}}/default-packages...");
    println!("npm install -g --quiet {}", pkgs.join(" "));
    let bin = cx.st.version_dir(ver).join("bin");
    let path = format!("{}:{}", bin.display(), env::var("PATH").unwrap_or_default());
    let ok = Command::new(bin.join("npm")).args(["install", "-g", "--quiet"]).args(&pkgs).env("PATH", path).status().map(|s| s.success()).unwrap_or(false);
    if !ok {
        eprintln!("Failed installing default packages. Please check if your default-packages file or a package in it has problems!");
        return Some(1);
    }
    Some(0)
}

fn cmd_uninstall(cx: &mut Ctx, rest: &[String]) -> i32 {
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
    let arg = if lts.is_some() && pat.is_none() { arg } else { pat.clone().unwrap_or(arg) };
    let shown = if pat.is_none() { rest.iter().find(|a| a.starts_with("--lts")).cloned().unwrap_or_else(|| arg.clone()) } else { arg.clone() };
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
        eprintln!("nvm: Cannot uninstall currently-active node version, {ver} (inferred from {arg}).");
        return 1;
    }
    let _ = fs::remove_dir_all(cx.st.version_dir(&ver));
    for d in [format!(".cache/src/node-{ver}/files")] {
        let _ = fs::remove_dir_all(cx.st.dir.join(d));
    }
    if let Ok(rd) = fs::read_dir(cx.st.dir.join(".cache/bin")) {
        for e in rd.flatten() {
            if e.file_name().to_string_lossy().starts_with(&format!("node-{ver}-")) {
                let _ = fs::remove_dir_all(e.path().join("files"));
            }
        }
    }
    println!("Uninstalled node {ver}");
    let mut names = vec![];
    walk_aliases(&cx.st.dir.join("alias"), "", &mut names);
    names.sort();
    for n in names {
        if fs::read_to_string(cx.st.alias_path(&n)).map(|c| c.contains(&ver)).unwrap_or(false) {
            cmd_unalias(cx, &[n]);
        }
    }
    0
}

/// Resolve `[--silent] [--lts[=x]] [version]` for exec/run. Returns (version, silent, rest args).
fn valid_versionish(s: &str) -> bool {
    if matches!(s, "stable" | "unstable" | "iojs" | "node") {
        return true;
    }
    let s = s.strip_prefix("iojs-").unwrap_or(s);
    let s = s.strip_prefix('v').unwrap_or(s).trim_end_matches('.');
    let parts: Vec<&str> = s.split('.').collect();
    !s.is_empty() && parts.len() <= 3 && parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

/// Resolve `[--silent] [--lts[=x]] [version]` for exec/run. Returns (version, silent, rest args).
fn exec_prelude(cx: &mut Ctx, rest: &[String], run: bool) -> Result<(String, bool, Vec<String>, String), i32> {
    let mut silent = false;
    let mut lts: Option<String> = None;
    let mut i = 0;
    while i < rest.len() {
        let a = &rest[i];
        if a == "--silent" {
            silent = true;
        } else if a == "--lts" {
            lts = Some("*".into());
        } else if let Some(n) = a.strip_prefix("--lts=") {
            lts = Some(n.to_string());
        } else if a == "--" {
            i += 1;
            break;
        } else if a.starts_with("--") {
            eprintln!("Unsupported option \"{a}\".");
            return Err(55);
        } else {
            break;
        }
        i += 1;
    }
    let mut tail: Vec<String> = rest[i..].to_vec();
    let mut label = String::new();
    let mut from_rc = false;
    let pat = if let Some(n) = &lts {
        label = n.clone();
        format!("lts/{n}")
    } else if let Some(first) = tail.first().cloned().filter(|f| !run || cx.st.version(f).is_ok() || valid_versionish(f)) {
        tail.remove(0);
        first
    } else {
        let cwd = env::current_dir().unwrap_or_default();
        match find_nvmrc(&cwd) {
            Some(f) => match parse_nvmrc(&fs::read_to_string(&f).unwrap_or_default()) {
                Ok(v) => {
                    if !silent {
                        println!("Found '{}' with version <{v}>", f.display());
                    }
                    from_rc = true;
                    v
                }
                Err(_) => {
                    eprintln!("invalid .nvmrc!");
                    return Err(1);
                }
            },
            None if run => {
                eprintln!("No version provided and no .nvmrc file found");
                eprintln!("WARNING: `nvm run` was invoked without a version argument and without an .nvmrc file.\n  Falling back to the active node version; this will become an error in a future release.\n  Pass `current` explicitly (e.g. `nvm run current ...`) to silence this warning.");
                "current".into()
            }
            None => {
                eprintln!("No version provided and no .nvmrc file found\nPlease see `nvm --help` or https://github.com/nvm-sh/nvm#nvmrc for more information.");
                return Err(127);
            }
        }
    };
    let cur = cx.st.ls_current(&env::var("PATH").unwrap_or_default());
    let v = if pat == "current" { Ok(cur) } else { cx.st.version(&pat) };
    match v {
        Ok(v) if cx.st.is_installed(&v) => {
            if !silent {
                let npm = npm_ver_suffix(cx, &v);
                if lts.as_deref() == Some("*") {
                    println!("Running node latest LTS -> {v}{npm}");
                } else if lts.is_some() {
                    println!("Running node LTS \"{label}\" -> {v}{npm}");
                } else {
                    println!("Running node {v}{npm}");
                }
            }
            Ok((v, silent, tail, pat))
        }
        _ => {
            not_installed(cx, &pat, from_rc);
            Err(1)
        }
    }
}
fn run_with(cx: &Ctx, ver: &str, cmd: &[String]) -> i32 {
    use std::os::unix::process::CommandExt;
    if cmd.is_empty() {
        return 0;
    }
    let vd = cx.st.version_dir(ver).to_string_lossy().into_owned();
    let path = pathedit::change_path(&env::var("PATH").unwrap_or_default(), "/bin", &vd, &cx.nvm_dir);
    let e = Command::new(&cmd[0]).args(&cmd[1..]).env("PATH", path).env("NVM_BIN", format!("{vd}/bin")).env("NVM_INC", format!("{vd}/include/node")).env("NODE_VERSION", ver).exec();
    eprintln!("{}: {e}", cmd[0]);
    127
}

use std::process::Command;

fn cmd_exec(cx: &mut Ctx, rest: &[String]) -> i32 {
    match exec_prelude(cx, rest, false) {
        Ok((v, _, tail, _)) => run_with(cx, &v, &tail),
        Err(c) => c,
    }
}

fn cmd_run(cx: &mut Ctx, rest: &[String]) -> i32 {
    match exec_prelude(cx, rest, true) {
        Ok((v, _, tail, _)) => {
            let mut c = vec!["node".to_string()];
            c.extend(tail);
            run_with(cx, &v, &c)
        }
        Err(code) => code,
    }
}
