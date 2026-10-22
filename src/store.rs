//! On-disk state under `$NVM_DIR`: installed versions and aliases.
use std::fs;
use std::path::{Path, PathBuf};

pub type Ver = (u64, u64, u64);

pub fn parse_ver(s: &str) -> Option<Ver> {
    let s = s.strip_prefix('v').unwrap_or(s);
    let mut it = s.splitn(3, '.');
    let a = it.next()?.parse().ok()?;
    let b = it.next()?.parse().ok()?;
    let c = it.next()?.parse().ok()?;
    Some((a, b, c))
}

/// Version without the `iojs-` flavour prefix.
pub fn bare(s: &str) -> &str {
    s.strip_prefix("iojs-").unwrap_or(s)
}

pub fn ensure_v(s: &str) -> String {
    if s.starts_with(|c: char| c.is_ascii_digit()) {
        format!("v{s}")
    } else {
        s.to_string()
    }
}

pub struct Store {
    pub dir: PathBuf,
}

impl Store {
    /// `nvm_version_path`: io.js under `versions/io.js`, node < 0.12 in the old layout (`$NVM_DIR/vX`).
    pub fn version_dir(&self, v: &str) -> PathBuf {
        if let Some(r) = v.strip_prefix("iojs-") {
            return self.dir.join("versions/io.js").join(r);
        }
        if parse_ver(v).is_some_and(|x| x < (0, 12, 0)) {
            return self.dir.join(v);
        }
        self.dir.join("versions/node").join(v)
    }

    /// Installed versions of every flavour, ascending (io.js entries are named `iojs-vX.Y.Z`).
    pub fn installed(&self) -> Vec<(Ver, String)> {
        self.installed_flavour(true, true)
    }

    /// Upstream's `nvm_ls` enumeration: node = old layout + `versions/node`, io.js = `versions/io.js`.
    /// Sorted numerically across flavours; one entry per number (`sort -u`, node wins).
    pub fn installed_flavour(&self, node: bool, iojs: bool) -> Vec<(Ver, String)> {
        let scan = |dir: PathBuf, prefix: &str, out: &mut Vec<(Ver, String)>| {
            for e in fs::read_dir(dir).into_iter().flatten().flatten() {
                let Ok(n) = e.file_name().into_string() else {
                    continue;
                };
                if !e.file_type().is_ok_and(|t| t.is_dir()) {
                    continue; // upstream's `find -type d` ignores files and symlinks
                }
                if let Some(v) = parse_ver(&n).filter(|_| n.starts_with('v')) {
                    out.push((v, format!("{prefix}{n}")));
                }
            }
        };
        let mut out = vec![];
        if node {
            scan(self.dir.clone(), "", &mut out);
            scan(self.dir.join("versions/node"), "", &mut out);
        }
        if iojs {
            scan(self.dir.join("versions/io.js"), "iojs-", &mut out);
        }
        out.sort_by_key(|(v, n)| (*v, n.starts_with("iojs-")));
        out.dedup_by_key(|x| x.0);
        out
    }

    /// Path of the first `node` on PATH that is not managed by nvm.
    pub fn system_node(&self) -> Option<PathBuf> {
        let root = self.dir.clone();
        let path = std::env::var("PATH").unwrap_or_default();
        path.split(':')
            .filter(|d| !Path::new(d).starts_with(&root))
            .map(|d| Path::new(d).join("node"))
            .find(|n| n.is_file())
    }

    pub fn system_version(&self) -> Option<String> {
        let n = self.system_node()?;
        let o = std::process::Command::new(n)
            .arg("--version")
            .output()
            .ok()?;
        Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
    }

    /// Versions with a cached tarball (`.cache/bin/node-vX-<os>-<arch>`), ascending.
    pub fn cached(&self) -> Vec<(Ver, String)> {
        let mut out: Vec<(Ver, String)> = fs::read_dir(self.dir.join(".cache/bin"))
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| {
                let n = e.file_name().into_string().ok()?;
                let v = n.strip_prefix("node-")?.split('-').next()?.to_string();
                Some((parse_ver(&v)?, v))
            })
            .collect();
        out.sort();
        out.dedup();
        out
    }

    pub fn is_installed(&self, v: &str) -> bool {
        if v == "system" {
            return self.system_node().is_some();
        }
        let p = self.version_dir(v).join("bin/node");
        fs::metadata(p).is_ok_and(|m| m.is_file())
    }

    pub fn alias_path(&self, name: &str) -> PathBuf {
        self.dir.join("alias").join(name)
    }

    /// First non-blank, comment-stripped line of an alias file.
    pub fn read_alias(&self, name: &str) -> Option<String> {
        let s = fs::read_to_string(self.alias_path(name)).ok()?;
        s.lines()
            .map(|l| l.split('#').next().unwrap_or("").trim())
            .find(|l| !l.is_empty())
            .map(String::from)
    }

    /// §4.2. Returns Some(target) after >=1 hop (`∞` on cycle), None when no alias.
    pub fn lts_minus(&self, n: usize) -> Option<String> {
        let mut names: Vec<String> = fs::read_dir(self.dir.join("alias/lts"))
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|n| n != "*")
            .collect();
        names.sort();
        names
            .len()
            .checked_sub(n + 1)
            .and_then(|i| names.get(i).cloned())
    }

    pub fn resolve_alias(&self, pattern: &str) -> Option<String> {
        if let Some(n) = pattern
            .strip_prefix("lts/-")
            .and_then(|n| n.parse::<usize>().ok())
        {
            let name = self.lts_minus(n)?;
            return self
                .read_alias(&format!("lts/{name}"))
                .map(|v| ensure_v(&v));
        }
        let mut seen = vec![pattern.to_string()];
        let mut cur = pattern.to_string();
        while let Some(next) = self.read_alias(&cur) {
            if seen.contains(&next) {
                return Some("∞".into());
            }
            seen.push(next.clone());
            cur = next;
        }
        if cur == pattern {
            return None;
        }
        Some(if matches!(cur.as_str(), "iojs" | "iojs-" | "node") {
            cur
        } else {
            ensure_v(&cur)
        })
    }

    /// Installed versions matching a (partial) pattern like `20`, `v20.11`, `iojs-v3`, ascending.
    /// A bare number searches every flavour; an `iojs-` pattern only io.js.
    pub fn matching(&self, pat: &str) -> Vec<String> {
        let (iojs_only, pat) = match pat.strip_prefix("iojs-") {
            Some(r) => (true, r),
            None => (false, pat),
        };
        let pat = pat.strip_prefix('v').unwrap_or(pat).trim_end_matches('.');
        let pfx = format!("{pat}.");
        self.installed_flavour(!iojs_only, true)
            .into_iter()
            .map(|x| x.1)
            .filter(|n| {
                let b = &bare(n)[1..];
                pat.is_empty() || b.starts_with(&pfx) || b == pat
            })
            .collect()
    }

    /// Highest installed version matching a (partial) pattern.
    pub fn match_partial(&self, pat: &str) -> Option<String> {
        self.matching(pat).pop()
    }

    /// §4.3 `nvm_version`: Ok(resolved) or Err("N/A" | "∞").
    pub fn version(&self, pattern: &str) -> Result<String, &'static str> {
        if pattern.is_empty() || pattern == "current" {
            return Ok(self.ls_current(&std::env::var("PATH").unwrap_or_default()));
        }
        let pattern = match pattern {
            // upstream's `nvm_version` maps `node` to `stable` before any alias lookup
            "node" | "node-" | "stable" => return self.stable_or_unstable(true),
            "unstable" => return self.stable_or_unstable(false),
            "iojs" | "iojs-" => {
                return self
                    .installed_flavour(false, true)
                    .pop()
                    .map(|x| x.1)
                    .ok_or("N/A")
            }
            p => p,
        };
        if pattern.contains("..") {
            return Err("N/A");
        }
        if pattern == "system" {
            return if self.system_node().is_some() {
                Ok("system".into())
            } else {
                Err("N/A")
            };
        }
        let p = ensure_v(pattern);
        if pattern.starts_with("lts/-") {
            return match self.resolve_alias(pattern) {
                Some(t) => self.version(&t),
                None => Err("N/A"),
            };
        }
        if self.alias_path(pattern).is_file() {
            return self.version_via_alias(pattern);
        }
        if parse_ver(&p).is_some() {
            // an exact `vX.Y.Z` is node first, else the io.js build of that number
            if self.is_installed(&p) {
                return Ok(p);
            }
            let io = format!("iojs-{p}");
            if self.is_installed(&io) {
                return Ok(io);
            }
        }
        self.match_partial(&p).ok_or("N/A")
    }

    fn version_via_alias(&self, name: &str) -> Result<String, &'static str> {
        match self.resolve_alias(name) {
            Some(t) if t == "∞" => Err("∞"),
            Some(t) if t == "system" => {
                if self.system_node().is_some() {
                    Ok(t)
                } else {
                    Err("N/A")
                }
            }
            Some(t) => self.version(&t),
            None => Err("N/A"),
        }
    }

    /// `major.minor` of the implicit `stable` (newest line with an even 0.x minor or major >= 1) or
    /// `unstable` (newest odd 0.x minor) alias, over installed node versions.
    pub fn implicit_minor(&self, stable: bool) -> Option<(u64, u64)> {
        self.installed_flavour(true, false)
            .into_iter()
            .rev()
            .map(|((maj, min, _), _)| (maj, min))
            .find(|&(maj, min)| (maj >= 1 || min % 2 == 0) == stable)
    }

    /// `stable` / `unstable`: a user alias file of that name wins, else the implicit alias.
    fn stable_or_unstable(&self, stable: bool) -> Result<String, &'static str> {
        let name = if stable { "stable" } else { "unstable" };
        if self.alias_path(name).is_file() {
            return self.version_via_alias(name);
        }
        let (maj, min) = self.implicit_minor(stable).ok_or("N/A")?;
        self.match_partial(&format!("{maj}.{min}")).ok_or("N/A")
    }

    pub fn ls_current(&self, path_env: &str) -> String {
        for d in path_env.split(':') {
            let node = Path::new(d).join("node");
            if node.is_file() {
                let Ok(rest) = Path::new(d).strip_prefix(&self.dir) else {
                    return "system".into();
                };
                let parts: Vec<String> = rest
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect();
                return match parts.iter().map(String::as_str).collect::<Vec<_>>()[..] {
                    ["versions", "io.js", v, ..] => format!("iojs-{v}"),
                    ["versions", _, v, ..] => v.to_string(),
                    [v, ..] => v.to_string(),
                    [] => "system".into(),
                };
            }
        }
        "none".into()
    }

    pub fn npm_version(&self, v: &str) -> Option<String> {
        let s = fs::read_to_string(
            self.version_dir(v)
                .join("lib/node_modules/npm/package.json"),
        )
        .ok()?;
        let i = s.find("\"version\"")?;
        let rest = &s[i + 9..];
        let a = rest.find('"')? + 1;
        let b = rest[a..].find('"')?;
        Some(rest[a..a + b].to_string())
    }
}

/// §4.7 parse `.nvmrc` content: the single bare line, or Err(stripped lines).
pub fn parse_nvmrc(content: &str) -> Result<String, String> {
    let lines: Vec<&str> = content
        .lines()
        .map(|l| l.split('#').next().unwrap_or("").trim())
        .filter(|l| !l.is_empty())
        .collect();
    let mut bare = vec![];
    let mut keys: Vec<&str> = vec![];
    for l in &lines {
        match l.split_once('=') {
            Some((k, _)) if !k.is_empty() => {
                if *k == *"node" || keys.contains(&k) {
                    return Err(lines.join("\n"));
                }
                keys.push(k);
            }
            _ => bare.push(*l),
        }
    }
    if bare.len() == 1 {
        Ok(bare[0].to_string())
    } else {
        Err(lines.join("\n"))
    }
}

pub fn find_nvmrc(start: &Path) -> Option<PathBuf> {
    let mut d = Some(start);
    while let Some(p) = d {
        let f = p.join(".nvmrc");
        if f.is_file() {
            return Some(f);
        }
        d = p.parent();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> Store {
        let d = std::env::temp_dir().join(format!("nvmcore-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(d.join("alias/lts")).unwrap();
        for v in ["v18.20.8", "v20.1.0", "v20.11.1", "v22.12.0", "v22.2.0"] {
            let b = d.join("versions/node").join(v).join("bin");
            fs::create_dir_all(&b).unwrap();
            fs::write(b.join("node"), "").unwrap();
        }
        Store { dir: d }
    }

    #[test]
    fn nvmrc_parsing() {
        assert_eq!(parse_nvmrc("20\n").unwrap(), "20");
        assert_eq!(parse_nvmrc("# c\nlts/iron # t\n").unwrap(), "lts/iron");
        assert_eq!(parse_nvmrc("foo=bar\n20\n").unwrap(), "20");
        assert!(parse_nvmrc("node=1\n20\n").is_err());
        assert!(parse_nvmrc("a=1\na=2\n20\n").is_err());
        assert!(parse_nvmrc("20\n22\n").is_err());
        assert!(parse_nvmrc("").is_err());
        assert_eq!(parse_nvmrc("=weird\n").unwrap(), "=weird");
    }

    #[test]
    fn partial_matching_is_numeric_and_prefix_safe() {
        let st = tmp("partial");
        assert_eq!(st.match_partial("2"), None, "2 must not match 20 or 22");
        assert_eq!(st.match_partial("20").as_deref(), Some("v20.11.1"));
        assert_eq!(st.match_partial("v22").as_deref(), Some("v22.12.0"));
        assert_eq!(st.match_partial("22.2").as_deref(), Some("v22.2.0"));
        assert_eq!(st.match_partial("20.1").as_deref(), Some("v20.1.0"));
        assert_eq!(st.version("node").as_deref(), Ok("v22.12.0"));
        assert_eq!(st.version("9"), Err("N/A"));
        let _ = fs::remove_dir_all(&st.dir);
    }

    #[test]
    fn alias_chains_and_cycles() {
        let st = tmp("alias");
        fs::write(st.dir.join("alias/a"), "b\n").unwrap();
        fs::write(st.dir.join("alias/b"), "20 # note\n").unwrap();
        fs::write(st.dir.join("alias/x"), "y\n").unwrap();
        fs::write(st.dir.join("alias/y"), "x\n").unwrap();
        assert_eq!(st.resolve_alias("a").as_deref(), Some("v20"));
        assert_eq!(st.version("a").as_deref(), Ok("v20.11.1"));
        assert_eq!(st.resolve_alias("x").as_deref(), Some("∞"));
        assert_eq!(st.version("x"), Err("∞"));
        assert_eq!(st.resolve_alias("nothing"), None);
        fs::write(st.dir.join("alias/lts/hydrogen"), "v18.20.8\n").unwrap();
        fs::write(st.dir.join("alias/lts/iron"), "v20.11.1\n").unwrap();
        assert_eq!(st.resolve_alias("lts/-1").as_deref(), Some("v18.20.8"));
        assert_eq!(st.version("lts/-1").as_deref(), Ok("v18.20.8"));
        let _ = fs::remove_dir_all(&st.dir);
    }

    #[test]
    fn version_parsing() {
        assert_eq!(parse_ver("v1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_ver("1.2"), None);
        assert!(parse_ver("v10.0.0") > parse_ver("v9.9.9"));
        assert_eq!(ensure_v("20"), "v20");
        assert_eq!(ensure_v("lts/iron"), "lts/iron");
    }
}
