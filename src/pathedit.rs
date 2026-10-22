//! §5.1 PATH-like list editing and §5.3 stripping.
const SYSTEM: [&str; 3] = ["/bin", "/usr/bin", "/usr/local/bin"];

fn is_nvm_entry(e: &str, dir: &str, suffix: &str) -> bool {
    let Some(rest) = e.strip_prefix(dir).and_then(|r| r.strip_prefix('/')) else { return false };
    let parts: Vec<&str> = rest.split('/').collect();
    let sfx: Vec<&str> = suffix.trim_start_matches('/').split('/').collect();
    let shaped = |n: usize| parts.len() >= n + sfx.len() && parts[n..n + sfx.len()] == sfx[..];
    (parts.first() != Some(&"versions") && shaped(1)) || (parts.first() == Some(&"versions") && shaped(3))
}

pub fn change_path(path: &str, suffix: &str, version_dir: &str, nvm_dir: &str) -> String {
    let new = format!("{version_dir}{suffix}");
    if path.is_empty() {
        return new;
    }
    let entries: Vec<&str> = path.split(':').collect();
    let Some(idx) = entries.iter().position(|e| is_nvm_entry(e, nvm_dir, suffix)) else {
        return format!("{new}:{path}");
    };
    // upstream: a system bin dir that has *any* nvm entry after it (not just the first) means prepend
    let last_nvm = entries.iter().rposition(|e| is_nvm_entry(e, nvm_dir, suffix)).unwrap_or(idx);
    if entries[..last_nvm].iter().any(|e| SYSTEM.contains(e)) {
        return format!("{new}:{path}");
    }
    let mut out = entries.clone();
    out[idx] = &new;
    out.join(":")
}

pub fn strip_path(path: &str, suffix: &str, nvm_dir: &str) -> String {
    let kept: Vec<&str> = path.split(':').filter(|e| !is_nvm_entry(e, nvm_dir, suffix)).collect();
    let mut s = kept.join(":");
    if path.ends_with(':') && !s.ends_with(':') && !s.is_empty() {
        s.push(':');
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    const D: &str = "/h/.nvm";
    #[test]
    fn prepend_when_absent() {
        assert_eq!(change_path("/usr/bin", "/bin", "/h/.nvm/versions/node/v20.0.0", D), "/h/.nvm/versions/node/v20.0.0/bin:/usr/bin");
    }
    #[test]
    fn replace_in_place() {
        let p = "/x:/h/.nvm/versions/node/v18.0.0/bin:/usr/bin";
        assert_eq!(change_path(p, "/bin", "/h/.nvm/versions/node/v20.0.0", D), "/x:/h/.nvm/versions/node/v20.0.0/bin:/usr/bin");
    }
    #[test]
    fn prepend_when_system_first() {
        let p = "/usr/bin:/h/.nvm/versions/node/v18.0.0/bin";
        assert!(change_path(p, "/bin", "/h/.nvm/versions/node/v20.0.0", D).starts_with("/h/.nvm/versions/node/v20.0.0/bin:/usr/bin"));
    }
    #[test]
    fn prepend_when_system_precedes_a_later_nvm_entry() {
        let p = "/h/.nvm/versions/node/v22.0.0/bin:/usr/bin:/h/.nvm/versions/node/v22.0.0/bin";
        assert_eq!(change_path(p, "/bin", "/h/.nvm/versions/node/v20.0.0", D), format!("/h/.nvm/versions/node/v20.0.0/bin:{p}"));
    }
    #[test]
    fn strip() {
        assert_eq!(strip_path("/h/.nvm/versions/node/v18.0.0/bin:/usr/bin", "/bin", D), "/usr/bin");
    }
}
