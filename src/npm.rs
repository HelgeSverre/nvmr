//! npm-facing helpers: latest-npm ladder (§6.14), global package reinstall (§6.15), default-packages (§6.1.4).
use crate::store::{parse_ver, Ver};
use std::{env, fs, path::Path, process::Command};

fn npm_cmd(bindir: Option<&Path>) -> Command {
    match bindir {
        Some(b) => {
            let mut c = Command::new(b.join("npm"));
            c.env(
                "PATH",
                format!("{}:{}", b.display(), env::var("PATH").unwrap_or_default()),
            );
            c
        }
        None => Command::new("npm"),
    }
}

fn ge(v: Ver, a: u64, b: u64, c: u64) -> bool {
    v >= (a, b, c)
}

#[derive(Clone, Copy)]
struct UpgradeStep {
    message: &'static str,
    package: Option<&'static str>,
}

impl UpgradeStep {
    const fn install(message: &'static str, package: &'static str) -> Self {
        Self {
            message,
            package: Some(package),
        }
    }

    const fn note(message: &'static str) -> Self {
        Self {
            message,
            package: None,
        }
    }
}

/// The upgrade sequence from upstream's `nvm_install_latest_npm`.
fn plan(v: Ver, npm: Ver) -> Vec<UpgradeStep> {
    let mut steps = vec![];
    let is06 = ge(v, 0, 6, 0) && !ge(v, 0, 7, 0);
    let is09 = ge(v, 0, 9, 0) && !ge(v, 0, 10, 0);

    if !is06 && !is09 {
        if ge(npm, 1, 0, 0) && !ge(npm, 2, 0, 0) {
            steps.push(UpgradeStep::install(
                "* `npm` v1.x needs to first jump to `npm` v1.4.28 to be able to upgrade further",
                "npm@1.4.28",
            ));
        } else if ge(npm, 2, 0, 0) && !ge(npm, 3, 0, 0) {
            steps.push(UpgradeStep::install(
                "* `npm` v2.x needs to first jump to the latest v2 to be able to upgrade further",
                "npm@2",
            ));
        }
    }

    if is06 {
        steps.push(UpgradeStep::install(
            "* `node` v0.6.x can only upgrade to `npm` v1.3.x",
            "npm@1.3",
        ));
        steps.push(UpgradeStep::note(
            "* node v0.6 and v0.9 are unable to upgrade further",
        ));
        return steps;
    }
    if is09 {
        steps.push(UpgradeStep::note(
            "* node v0.6 and v0.9 are unable to upgrade further",
        ));
        return steps;
    }
    if !ge(v, 1, 1, 0) {
        steps.push(UpgradeStep::install(
            "* `npm` v4.5.x is the last version that works on `node` versions < v1.1.0",
            "npm@4.5",
        ));
        return steps;
    }
    if !ge(v, 4, 0, 0) {
        steps.push(UpgradeStep::install(
            "* `npm` v5 and higher do not work on `node` versions below v4.0.0",
            "npm@4",
        ));
        return steps;
    }
    let is44 = !ge(v, 4, 5, 0);
    let is5 = !is44 && ge(v, 5, 0, 0);
    let is6 = is5 && ge(v, 6, 0, 0);
    let is62 = is6 && ge(v, 6, 2, 0);
    let is9 = is62 && ge(v, 9, 0, 0);
    let is93 = is9 && ge(v, 9, 3, 0);
    let is10 = is93 && ge(v, 10, 0, 0);
    let is12 = is10 && ge(v, 12, 13, 0);
    let is13 = is12 && ge(v, 13, 0, 0);
    let is14 = is13 && ge(v, 14, 15, 0);
    let is1417 = is14 && ge(v, 14, 17, 0);
    let is15 = is14 && ge(v, 15, 0, 0);
    let is16 = is15 && ge(v, 16, 0, 0);
    let is16l = is16 && ge(v, 16, 13, 0);
    let is17 = is16l && ge(v, 17, 0, 0);
    let is18 = is17 && ge(v, 18, 0, 0);
    let is1817 = is18 && ge(v, 18, 17, 0);
    let is19 = is1817 && ge(v, 19, 0, 0);
    let is205 = is19 && ge(v, 20, 5, 0);
    let is2017 = is205 && ge(v, 20, 17, 0);
    let is21 = is2017 && ge(v, 21, 0, 0);
    let is229 = is21 && ge(v, 22, 9, 0);
    let final_step = if is44 || (is5 && !ge(v, 5, 10, 0)) {
        UpgradeStep::install("* `npm` `v5.3.x` is the last version that works on `node` 4.x versions below v4.4, or 5.x versions below v5.10, due to `Buffer.alloc`", "npm@5.3")
    } else if !is44 && !ge(v, 4, 7, 0) {
        UpgradeStep::install(
            "* `npm` `v5.4.1` is the last version that works on `node` `v4.5` and `v4.6`",
            "npm@5.4.1",
        )
    } else if !is6 {
        UpgradeStep::install(
            "* `npm` `v5.x` is the last version that works on `node` below `v6.0.0`",
            "npm@5",
        )
    } else if (is6 && !is62) || (is9 && !is93) {
        UpgradeStep::install("* `npm` `v6.9` is the last version that works on `node` `v6.0.x`, `v6.1.x`, `v9.0.x`, `v9.1.x`, or `v9.2.x`", "npm@6.9")
    } else if !is10 {
        if !ge(npm, 4, 4, 4) {
            steps.push(UpgradeStep::install(
                "* `npm` `v4.4.4` or later is required to install npm v6.14.18",
                "npm@4",
            ));
        }
        UpgradeStep::install(
            "* `npm` `v6.x` is the last version that works on `node` below `v10.0.0`",
            "npm@6",
        )
    } else if !is12 || (is13 && !is14) || (is15 && !is16) {
        UpgradeStep::install("* `npm` `v7.x` is the last version that works on `node` `v13`, `v15`, below `v12.13`, or `v14.0` - `v14.15`", "npm@7")
    } else if (is12 && !is13) || (is14 && !is1417) || (is16 && !is16l) || (is17 && !is18) {
        UpgradeStep::install("* `npm` `v8.6` is the last version that works on `node` `v12`, `v14.13` - `v14.16`, or `v16.0` - `v16.12`", "npm@8.6")
    } else if !is1817 || (is19 && !is205) {
        UpgradeStep::install("* `npm` `v9.x` is the last version that works on `node` `< v18.17`, `v19`, or `v20.0` - `v20.4`", "npm@9")
    } else if !is2017 || (is21 && !is229) {
        UpgradeStep::install("* `npm` `v10.x` is the last version that works on `node` `< v20.17`, `v21`, or `v22.0` - `v22.8`", "npm@10")
    } else {
        UpgradeStep::install("* Installing latest `npm`; if this does not work on your node version, please report a bug!", "npm")
    };
    steps.push(final_step);
    steps
}

fn npm_version(bindir: Option<&Path>) -> Option<String> {
    let o = npm_cmd(bindir)
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())?;
    Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
}

/// `node_version` like `v22.12.0`; `bindir` = directory with that node/npm (None => PATH).
pub fn install_latest(node_version: &str, bindir: Option<&Path>) -> i32 {
    println!("Attempting to upgrade to the latest working version of npm...");
    let Some(v) = parse_ver(node_version) else {
        eprintln!("Unable to obtain node version.");
        return 1;
    };
    let Some(npm) = npm_version(bindir) else {
        eprintln!("Unable to obtain npm version.");
        return 2;
    };
    let debug = env::var("NVM_DEBUG").as_deref() == Ok("1");
    if debug {
        println!("Detected node version {node_version}, npm version v{npm}");
    }
    let npm_ver = parse_ver(&npm).unwrap_or_default();
    for step in plan(v, npm_ver) {
        println!("{}", step.message);
        if let Some(pkg) = step.package {
            if debug {
                println!("npm install -g {pkg}");
            } else {
                let _ = npm_cmd(bindir).args(["install", "-g", pkg]).status();
            }
        }
    }
    println!(
        "* npm upgraded to: v{}",
        npm_version(bindir).unwrap_or_default()
    );
    0
}

/// Parses `npm list -g --depth=0` output into (installs, links).
pub fn global_modules(src_bin: &Path) -> (Vec<String>, Vec<String>) {
    let out = npm_cmd(Some(src_bin))
        .args(["list", "-g", "--depth=0"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();
    let lines: Vec<&str> = out
        .lines()
        .skip(1)
        .filter(|l| !l.contains("UNMET PEER DEPENDENCY"))
        .collect();
    let mut installs = vec![];
    for l in &lines {
        if l.contains(" -> ") || l.contains("(empty)") {
            continue;
        }
        if let Some(tok) = l
            .split(' ')
            .rev()
            .find(|t| t.contains('@') && !t.is_empty())
        {
            if !tok.starts_with("npm@") && !tok.starts_with("corepack@") {
                installs.push(tok.to_string());
            }
        }
    }
    let links = lines
        .iter()
        .filter_map(|l| l.split_once(" -> ").map(|x| x.1.to_string()))
        .collect();
    (installs, links)
}

pub fn reinstall(label: &str, src_bin: &Path, target_bin: Option<&Path>) {
    let (installs, links) = global_modules(src_bin);
    println!("Reinstalling global packages from {label}...");
    if installs.is_empty() {
        println!("No installed global packages found...");
    } else {
        let _ = npm_cmd(target_bin)
            .args(["install", "-g", "--quiet"])
            .args(&installs)
            .status();
    }
    println!("Linking global packages from {label}...");
    if links.is_empty() {
        println!("No linked global packages found...");
    } else {
        for l in links.iter().filter(|l| !l.is_empty()) {
            let dir = if l.starts_with('/') {
                l.into()
            } else {
                let root = npm_cmd(target_bin)
                    .args(["root", "-g"])
                    .output()
                    .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                    .unwrap_or_default();
                format!("{root}/../{l}")
            };
            let _ = npm_cmd(target_bin).arg("link").current_dir(dir).status();
        }
    }
}

/// awk-exact loader: skip `#...` and blank lines, error on any line containing whitespace.
pub fn default_packages(nvm_dir: &Path) -> Result<Vec<String>, i32> {
    let f = nvm_dir.join("default-packages");
    let Ok(txt) = fs::read_to_string(&f) else {
        return Ok(vec![]);
    };
    let mut out = vec![];
    for (i, l) in txt.lines().enumerate() {
        if l.trim_start().starts_with('#') || l.trim().is_empty() {
            continue;
        }
        if l.contains(' ') || l.contains('\t') {
            eprintln!("Only one package per line is allowed in `{}`. Please remove any lines with multiple space-separated values.", f.display());
            return Err(1);
        }
        // awk quirk: every accepted line after file line 1 is preceded by a space, so the joined string may start with one.
        out.push(if i > 0 {
            format!(" {l}")
        } else {
            l.to_string()
        });
    }
    Ok(out)
}

pub fn install_default_packages(nvm_dir: &Path, bindir: &Path) -> i32 {
    let pkgs = match default_packages(nvm_dir) {
        Ok(p) => p,
        Err(c) => return c,
    };
    if pkgs.is_empty() {
        return 0;
    }
    println!(
        "Installing default global packages from {}/default-packages...",
        nvm_dir.display()
    );
    println!("npm install -g --quiet {}", pkgs.concat());
    let ok = npm_cmd(Some(bindir))
        .args(["install", "-g", "--quiet"])
        .args(pkgs.iter().map(|p| p.trim()))
        .status()
        .is_ok_and(|s| s.success());
    if !ok {
        eprintln!("Failed installing default packages. Please check if your default-packages file or a package in it has problems!");
        return 1;
    }
    0
}
