mod pathedit;
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

const HELP_HINT: &str = "Run `nvm --help` for full help.";

fn run(cx: &mut Ctx, args: &[String]) -> i32 {
    if args.is_empty() {
        println!("Node Version Manager (v0.40.8)\n\nUsage: nvm <command> ... (see upstream help text asset)");
        return 0;
    }
    for a in args.iter().take_while(|a| *a != "--") {
        if matches!(a.as_str(), "-h" | "help" | "--help") {
            println!("Node Version Manager (v0.40.8)\n\nUsage: nvm <command> ... (see upstream help text asset)");
            return 0;
        }
        if a.starts_with("---") {
            eprintln!("Unsupported option \"{a}\".");
            return 55;
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
            _ => usage_cache(),
        },
        "__auto" => cmd_auto(cx, rest),
        other => {
            eprintln!("nvm: command '{other}' is not implemented in this build yet.\n{HELP_HINT}");
            127
        }
    }
}

fn usage_cache() -> i32 {
    eprintln!("Usage: nvm cache dir | nvm cache clear");
    127
}

fn guard(s: &str) -> bool {
    !s.split('/').any(|c| c == "..")
}

fn cmd_version(cx: &mut Ctx, rest: &[String]) -> i32 {
    let pat = rest.iter().find(|a| *a != "--").map(String::as_str).unwrap_or("current");
    if !guard(pat) {
        eprintln!("invalid version: {pat}");
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
fn pick_pattern(rest: &[String], silent: bool) -> Result<(String, bool), i32> {
    if let Some(v) = rest.iter().find(|a| !a.starts_with("--") || *a == "--") {
        if v != "--" {
            return Ok((v.clone(), false));
        }
    }
    let cwd = env::current_dir().unwrap_or_default();
    let Some(rc) = find_nvmrc(&cwd) else {
        eprintln!("No .nvmrc file found\nPlease see `nvm --help` or https://github.com/nvm-sh/nvm#nvmrc for more information.");
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
            eprintln!("invalid .nvmrc!\nall non-commented content (anything after # is a comment) must be either:\n  - a single bare nvm-recognized version-ish\n  - or, multiple distinct key-value pairs, each key/value separated by a single equals sign (=)\n\nadditionally, a single bare nvm-recognized version-ish must be present (after stripping comments).\n\nnon-commented content parsed:\n{lines}");
            Err(1)
        }
    }
}

fn not_installed(arg: &str, resolved: &str, from_rc: bool) {
    let orig = arg;
    let shown = ensure_v(arg);
    let arg = shown.as_str();
    if arg == resolved || resolved == "N/A" {
        eprintln!("N/A: version \"{arg}\" is not yet installed.\n");
    } else {
        eprintln!("N/A: version \"{arg} -> {resolved}\" is not yet installed.\n");
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
    let (pat, rc) = match pick_pattern(rest, silent) {
        Ok(x) => x,
        Err(c) => return c,
    };
    match cx.st.version(&pat) {
        Err("∞") => {
            eprintln!("The alias \"{pat}\" leads to an infinite loop. Aborting.");
            8
        }
        Ok(v) if cx.st.is_installed(&v) => {
            println!("{}/bin/node", cx.st.version_dir(&v).display());
            0
        }
        r => {
            not_installed(&pat, r.as_deref().unwrap_or("N/A"), rc);
            1
        }
    }
}

fn cmd_use(cx: &mut Ctx, rest: &[String]) -> i32 {
    let silent = rest.iter().any(|a| a == "--silent");
    let (pat, from_rc) = match pick_pattern(rest, silent) {
        Ok(x) => x,
        Err(c) => return c,
    };
    if !guard(&pat) {
        eprintln!("invalid version: {pat}");
        return 3;
    }
    let ver = cx.st.version(&pat);
    match ver {
        Err("∞") => {
            if !silent {
                eprintln!("The alias \"{pat}\" leads to an infinite loop. Aborting.");
            }
            8
        }
        Ok(ref v) if v == "system" => {
            let path = env::var("PATH").unwrap_or_default();
            let np = pathedit::strip_path(&path, "/bin", &cx.nvm_dir);
            cx.ev.export("PATH", &np);
            cx.ev.unset("NVM_BIN");
            cx.ev.unset("NVM_INC");
            cx.ev.rehash();
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
        other => {
            if !silent {
                not_installed(&pat, other.as_deref().unwrap_or("N/A"), from_rc);
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
            eprintln!("Could not find {}/*/bin in {}", "${NVM_DIR}", "${PATH}");
        }
    } else {
        cx.ev.export("PATH", &np);
        if !silent {
            println!("${{NVM_DIR}}/*/bin removed from ${{PATH}}");
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
            list_aliases(cx);
            0
        }
        [name] => {
            if let Some(t) = cx.st.read_alias(name) {
                print_alias(cx, name, &t);
                0
            } else {
                eprintln!("Alias {name} doesn't exist!");
                3
            }
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

fn list_aliases(cx: &Ctx) {
    let mut names = vec![];
    walk_aliases(&cx.st.dir.join("alias"), "", &mut names);
    names.sort();
    let (lts, user): (Vec<_>, Vec<_>) = names.into_iter().partition(|n| n.starts_with("lts/"));
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
        if has(name) {
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
    implicit("stable", stable);
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
            let pfx = format!("{}.", ensure_v(p).trim_end_matches('.'));
            cx.st.installed().into_iter().map(|x| x.1).filter(|n| format!("{n}.").starts_with(&pfx) || *n == ensure_v(p)).collect()
        }
        None => cx.st.installed().into_iter().map(|x| x.1).collect(),
    };
    if list.is_empty() {
        println!("{:>15}", "N/A");
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
        list_aliases(cx);
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
