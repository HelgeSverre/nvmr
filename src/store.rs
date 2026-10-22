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
    pub fn version_dir(&self, v: &str) -> PathBuf {
        self.dir.join("versions/node").join(v)
    }

    /// Installed versions, ascending.
    pub fn installed(&self) -> Vec<(Ver, String)> {
        let mut out: Vec<(Ver, String)> = fs::read_dir(self.dir.join("versions/node"))
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| {
                let n = e.file_name().into_string().ok()?;
                if !e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    return None; // upstream's `find -type d` ignores files and symlinks
                }
                Some((parse_ver(&n).filter(|_| n.starts_with('v'))?, n))
            })
            .collect();
        out.sort();
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
        fs::metadata(p).map(|m| m.is_file()).unwrap_or(false)
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

    /// Highest installed version matching a (partial) pattern like `20`, `v20.11`.
    pub fn match_partial(&self, pat: &str) -> Option<String> {
        let pat = pat.strip_prefix('v').unwrap_or(pat).trim_end_matches('.');
        if pat.is_empty() {
            return self.installed().pop().map(|x| x.1);
        }
        let pfx = format!("{pat}.");
        self.installed()
            .into_iter()
            .rev()
            .find(|(_, n)| n[1..].starts_with(&pfx) || n[1..] == *pat)
            .map(|x| x.1)
    }

    /// §4.3 `nvm_version`: Ok(resolved) or Err("N/A" | "∞").
    pub fn version(&self, pattern: &str) -> Result<String, &'static str> {
        if pattern.is_empty() || pattern == "current" {
            return Ok(self.ls_current(&std::env::var("PATH").unwrap_or_default()));
        }
        let pattern = match pattern {
            "node" | "node-" | "stable" => return self.match_partial("").ok_or("N/A"),
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
            return match self.resolve_alias(pattern) {
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
            };
        }
        if parse_ver(&p).is_some() && self.is_installed(&p) {
            return Ok(p);
        }
        self.match_partial(&p).ok_or("N/A")
    }

    pub fn ls_current(&self, path_env: &str) -> String {
        let root = self.dir.join("versions/node");
        for d in path_env.split(':') {
            let node = Path::new(d).join("node");
            if node.is_file() {
                return match Path::new(d).strip_prefix(&root) {
                    Ok(rest) => rest
                        .components()
                        .next()
                        .map_or("system".into(), |c| c.as_os_str().to_string_lossy().into()),
                    Err(_) => "system".into(),
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
