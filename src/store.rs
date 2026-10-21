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
                Some((parse_ver(&n).filter(|_| n.starts_with('v'))?, n))
            })
            .collect();
        out.sort();
        out
    }

    pub fn is_installed(&self, v: &str) -> bool {
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
    pub fn resolve_alias(&self, pattern: &str) -> Option<String> {
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
        Some(if matches!(cur.as_str(), "iojs" | "iojs-" | "node") { cur } else { ensure_v(&cur) })
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
        let pattern = match pattern {
            "node" | "node-" | "stable" => return self.match_partial("").ok_or("N/A"),
            p => p,
        };
        if pattern.contains("..") {
            return Err("N/A");
        }
        let p = ensure_v(pattern);
        if self.alias_path(pattern).is_file() {
            return match self.resolve_alias(pattern) {
                Some(t) if t == "∞" => Err("∞"),
                Some(t) if t == "system" => Ok(t),
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
                    Ok(rest) => rest.components().next().map_or("system".into(), |c| c.as_os_str().to_string_lossy().into()),
                    Err(_) => "system".into(),
                };
            }
        }
        "none".into()
    }

    pub fn npm_version(&self, v: &str) -> Option<String> {
        let s = fs::read_to_string(self.version_dir(v).join("lib/node_modules/npm/package.json")).ok()?;
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
    let bare: Vec<&&str> = lines.iter().filter(|l| l.starts_with('=') || !l.contains('=')).collect();
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
