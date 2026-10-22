//! Remote index (`index.tab`), LTS alias maintenance, remote version resolution (§4.5).
use crate::store::{parse_ver, Store, Ver};
use std::{env, fs, path::PathBuf, process::Command};

#[derive(Clone)]
pub struct Row {
    pub ver: String,
    pub lts: Option<String>,
    /// First (newest) row of its LTS line.
    pub latest_of_line: bool,
}

pub fn mirror() -> String {
    env::var("NVM_NODEJS_ORG_MIRROR")
        .unwrap_or_else(|_| "https://nodejs.org/dist".into())
        .trim_end_matches('/')
        .to_string()
}

/// `^https?://[]a-zA-Z0-9._~:/@%[-]+$`
pub fn mirror_ok(m: &str) -> bool {
    let rest = m
        .strip_prefix("https://")
        .or_else(|| m.strip_prefix("http://"));
    rest.map_or(false, |r| {
        !r.is_empty()
            && r.chars()
                .all(|c| c.is_ascii_alphanumeric() || "]._~:/@%[-".contains(c))
    })
}

pub fn fetch_bin() -> PathBuf {
    if let Ok(p) = env::var("NVM_FETCH") {
        return p.into();
    }
    env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|d| d.join("nvm-fetch")))
        .unwrap_or_else(|| "nvm-fetch".into())
}

pub fn http_get(url: &str) -> Result<String, i32> {
    let out = Command::new(fetch_bin())
        .args(["get", url])
        .stderr(std::process::Stdio::null())
        .output()
        .map_err(|_| {
            eprintln!("nvm-fetch not found");
            1
        })?;
    if !out.status.success() {
        return Err(out.status.code().unwrap_or(4));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn key(v: &str) -> Ver {
    parse_ver(v.strip_prefix("iojs-").unwrap_or(v)).unwrap_or((0, 0, 0))
}

/// io.js rows (recognition only): `iojs-vX.Y.Z`, no LTS data.
pub fn iojs_rows() -> Result<Vec<Row>, i32> {
    let m = env::var("NVM_IOJS_ORG_MIRROR").unwrap_or_else(|_| "https://iojs.org/dist".into());
    let body = http_get(&format!("{}/index.tab", m.trim_end_matches('/')))?;
    Ok(body
        .lines()
        .skip(1)
        .filter_map(|l| l.split('\t').next())
        .filter(|v| v.starts_with('v'))
        .map(|v| Row {
            ver: format!("iojs-{v}"),
            lts: None,
            latest_of_line: false,
        })
        .collect())
}

/// Fetch index.tab, rewrite alias/lts/*, return rows ascending by version.
pub fn index(st: &Store) -> Result<Vec<Row>, i32> {
    if !mirror_ok(&mirror()) {
        eprintln!("$NVM_NODEJS_ORG_MIRROR and $NVM_IOJS_ORG_MIRROR may only contain a URL");
        return Ok(vec![]);
    }
    // Upstream treats any fetch failure as "no versions".
    let Ok(body) = http_get(&format!("{}/index.tab", mirror())) else {
        return Ok(vec![]);
    };
    let mut rows = vec![];
    let mut seen: Vec<String> = vec![];
    for l in body.lines().skip(1) {
        let c: Vec<&str> = l.split('\t').collect();
        if c.len() < 10 || !c[0].starts_with('v') {
            continue;
        }
        let lts = (c[9] != "-").then(|| c[9].to_string());
        let mut latest = false;
        if let Some(n) = &lts {
            let low = n.to_lowercase();
            if !seen.contains(&low) {
                seen.push(low.clone());
                latest = true;
                let dir = st.dir.join("alias/lts");
                let _ = fs::create_dir_all(&dir);
                let _ = fs::write(dir.join(&low), format!("{}\n", c[0]));
                if seen.len() == 1 {
                    let _ = fs::write(dir.join("*"), format!("lts/{low}\n"));
                }
            }
        }
        rows.push(Row {
            ver: c[0].into(),
            lts,
            latest_of_line: latest,
        });
    }
    rows.sort_by_key(|r| key(&r.ver));
    Ok(rows)
}

/// `lts/-N` -> codename; Err(msg, rc).
pub fn lts_name(st: &Store, arg: &str) -> Result<String, (String, i32)> {
    if let Some(n) = arg.strip_prefix('-').and_then(|n| n.parse::<usize>().ok()) {
        let mut names: Vec<String> = fs::read_dir(st.dir.join("alias/lts"))
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|n| n != "*")
            .collect();
        names.sort();
        return names
            .len()
            .checked_sub(n + 1)
            .and_then(|i| names.get(i).cloned())
            .ok_or(("That many LTS releases do not exist yet.".into(), 2));
    }
    if arg != arg.to_lowercase() {
        return Err(("LTS names must be lowercase".into(), 3));
    }
    Ok(arg.to_string())
}

pub fn sort_rows(rows: &mut [Row]) {
    rows.sort_by_key(|r| key(&r.ver));
}

pub fn filter<'a>(rows: &'a [Row], lts: Option<&str>, pattern: Option<&str>) -> Vec<&'a Row> {
    rows.iter()
        .filter(|r| match lts {
            None => true,
            Some("*") => r.lts.is_some(),
            Some(n) => r
                .lts
                .as_ref()
                .map(|l| l.to_lowercase() == n.to_lowercase())
                .unwrap_or(false),
        })
        .filter(|r| match pattern {
            None => true,
            Some(p) => {
                let p = p.trim_end_matches('.');
                let p = p.strip_prefix('v').unwrap_or(p);
                let v = &r.ver[1..];
                v == p || (v.starts_with(p) && v[p.len()..].starts_with('.'))
            }
        })
        .collect()
}

/// Resolve a pattern against the remote list. Returns the chosen row.
pub fn resolve<'a>(
    st: &Store,
    rows: &'a [Row],
    pattern: &str,
    lts: Option<&str>,
) -> Option<&'a Row> {
    let mut pat = pattern.to_string();
    if let Some(t) = st.resolve_alias(&pat) {
        pat = t;
    }
    if matches!(pat.as_str(), "" | "node" | "stable") {
        return filter(rows, lts, None)
            .into_iter()
            .filter(|r| parse_ver(&r.ver).map(|v: Ver| v.0 >= 1).unwrap_or(false))
            .last();
    }
    filter(rows, lts, Some(&pat)).into_iter().last()
}

pub fn format_row(
    r: &Row,
    installed: bool,
    current: bool,
    pal: &Option<crate::colors::Pal>,
) -> String {
    use crate::colors::{bold_current, paint};
    let base = if current {
        format!("->{:>13}", r.ver)
    } else {
        format!("{:>15}", r.ver)
    };
    let Some(p) = pal else {
        let mut s = base;
        if installed || current {
            s += " *";
        }
        if let Some(n) = &r.lts {
            if !(installed || current) {
                s += "  ";
            }
            s += &if r.latest_of_line {
                format!(" (Latest LTS: {n})")
            } else {
                format!(" (LTS: {n})")
            };
        }
        return s;
    };
    let mut s = if current {
        paint(pal, 3, &base)
    } else if installed {
        paint(pal, 1, &base)
    } else {
        base
    };
    if let Some(n) = &r.lts {
        s += "  ";
        if r.latest_of_line {
            s += &format!("\x1b[{}m (Latest LTS: {n})\x1b[0m", bold_current(p));
        } else {
            s += &paint(pal, 5, &format!(" (LTS: {n})"));
        }
    }
    s
}
