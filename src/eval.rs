//! The eval channel (§1.1): shell statements queued by `nvmr` and sourced by the `nvm` function.
use std::collections::HashMap;
use std::{env, fs};

/// Collects shell statements for the eval channel (§1.1).
#[derive(Debug, Default)]
pub struct Eval {
    pub shell: String,
    pub buf: String,
    /// Values queued for export, so later steps in the same invocation see them (None = queued unset).
    vars: HashMap<String, Option<String>>,
}

/// Single-quotes `v` for POSIX shells.
pub fn q(v: &str) -> String {
    format!("'{}'", v.replace('\'', "'\\''"))
}

impl Eval {
    pub fn new(shell: String) -> Self {
        Self {
            shell,
            ..Self::default()
        }
    }

    /// Current value as the shell will see it after the queued statements run.
    pub fn get(&self, k: &str) -> Option<String> {
        match self.vars.get(k) {
            Some(v) => v.clone(),
            None => env::var(k).ok(),
        }
    }
    pub fn export(&mut self, k: &str, v: &str) {
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
    pub fn unset(&mut self, k: &str) {
        self.vars.insert(k.to_string(), None);
        self.buf += &if self.shell == "fish" {
            format!("set -e {k}\n")
        } else {
            format!("unset {k}\n")
        };
    }
    pub fn rehash(&mut self) {
        self.buf += match self.shell.as_str() {
            "zsh" => "rehash\n",
            "fish" => "",
            _ => "hash -r\n",
        };
    }
}

/// Writes queued shell statements to `$NVM_EVAL_FILE` (creating its directory), clears stale content when
/// there is nothing to write, and prunes eval files of long-dead shells from `$NVM_DIR/.cache`.
pub fn flush_eval(buf: &str) {
    let Ok(f) = env::var("NVM_EVAL_FILE") else {
        return;
    };
    let path = std::path::Path::new(&f);
    if buf.is_empty() {
        if fs::metadata(path).is_ok_and(|m| m.len() > 0) {
            let _ = fs::write(path, "");
        }
        return;
    }
    if let Some(d) = path.parent() {
        let _ = fs::create_dir_all(d);
        if d.file_name().is_some_and(|n| n == ".cache") {
            let day = std::time::Duration::from_secs(86_400);
            for e in fs::read_dir(d).into_iter().flatten().flatten() {
                let stale = e.file_name().to_string_lossy().starts_with("eval.")
                    && e.metadata()
                        .and_then(|m| m.modified())
                        .ok()
                        .and_then(|t| t.elapsed().ok())
                        .is_some_and(|a| a > day);
                if stale {
                    let _ = fs::remove_file(e.path());
                }
            }
        }
    }
    let _ = fs::write(path, buf);
}
