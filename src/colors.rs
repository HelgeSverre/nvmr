//! Colour palette (§8): NVM_COLORS positions 1 installed, 2 system/LTS, 3 current, 4 missing, 5 default; 6 = bold of 2.
use std::{env, fs, io::IsTerminal, path::PathBuf};

#[derive(Clone)]
pub struct Pal(pub [String; 6]);

fn code(c: char) -> Option<&'static str> {
    Some(match c {
        'r' => "0;31",
        'R' => "1;31",
        'g' => "0;32",
        'G' => "1;32",
        'b' => "0;34",
        'B' => "1;34",
        'c' => "0;36",
        'C' => "1;36",
        'm' => "0;35",
        'M' => "1;35",
        'y' => "0;33",
        'Y' => "1;33",
        'k' => "0;30",
        'K' => "1;30",
        'e' => "0;37",
        'W' => "1;37",
        _ => return None,
    })
}

pub fn valid_codes(s: &str) -> bool {
    s.chars().count() == 5 && s.chars().all(|c| code(c).is_some())
}

/// Whether the active terminal database advertises at least eight colours.
///
/// This is the small subset of terminfo that nvm needs. Reading it here keeps
/// the normal command path free of a `tput` process.
fn has_colors(term: &str) -> bool {
    if term.is_empty() || term.contains('/') {
        return false;
    }
    terminfo_paths(term).any(|path| {
        let Ok(data) = fs::read(path) else {
            return false;
        };
        terminfo_colors(&data).is_some_and(|colors| colors >= 8)
    })
}

fn terminfo_paths(term: &str) -> impl Iterator<Item = PathBuf> + '_ {
    let mut dirs = env::var_os("TERMINFO")
        .map(PathBuf::from)
        .into_iter()
        .collect::<Vec<_>>();
    if let Some(path) = env::var_os("TERMINFO_DIRS") {
        dirs.extend(env::split_paths(&path));
    }
    dirs.extend([
        "/usr/share/terminfo".into(),
        "/usr/share/lib/terminfo".into(),
        "/usr/lib/terminfo".into(),
        "/lib/terminfo".into(),
        "/etc/terminfo".into(),
    ]);
    let first = term.as_bytes()[0];
    let char_dir = char::from(first).to_string();
    let hex_dir = format!("{first:x}");
    dirs.into_iter().flat_map(move |dir| {
        [
            dir.join(&char_dir).join(term),
            dir.join(&hex_dir).join(term),
        ]
    })
}

/// Extract `max_colors` (numeric capability 13) from a compiled terminfo entry.
fn terminfo_colors(data: &[u8]) -> Option<i32> {
    let header = |offset| {
        data.get(offset..offset + 2)
            .and_then(|bytes| bytes.try_into().ok())
            .map(u16::from_le_bytes)
    };
    let magic = header(0)?;
    let names = usize::from(header(2)?);
    let bools = usize::from(header(4)?);
    let numbers = usize::from(header(6)?);
    if numbers <= 13 {
        return None;
    }
    let offset = 12 + names + bools;
    let offset = offset + offset % 2;
    match magic {
        0x011a => data
            .get(offset + 26..offset + 28)
            .and_then(|bytes| bytes.try_into().ok())
            .map(i16::from_le_bytes)
            .map(i32::from),
        0x021e => data
            .get(offset + 52..offset + 56)
            .and_then(|bytes| bytes.try_into().ok())
            .map(i32::from_le_bytes),
        _ => None,
    }
}

/// None when colour is off: not a TTY, TERM unusable, `--no-colors`, or NVM_NO_COLORS=--no-colors.
pub fn detect(args: &[String]) -> Option<Pal> {
    if args.iter().any(|a| a == "--no-colors")
        || env::var("NVM_NO_COLORS").as_deref() == Ok("--no-colors")
    {
        return None;
    }
    if !std::io::stdout().is_terminal() {
        return None;
    }
    let term = env::var("TERM").ok()?;
    if !has_colors(&term) {
        return None;
    }
    let spec = env::var("NVM_COLORS")
        .ok()
        .filter(|s| valid_codes(s))
        .unwrap_or_else(|| "bygre".into());
    let cs: Vec<&str> = spec.chars().filter_map(code).collect();
    let bold2 = format!("1;{}", cs[1].split(';').nth(1).unwrap_or("33"));
    Some(Pal([
        cs[0].into(),
        cs[1].into(),
        cs[2].into(),
        cs[3].into(),
        cs[4].into(),
        bold2,
    ]))
}

pub fn paint(p: &Option<Pal>, i: usize, s: &str) -> String {
    match p {
        Some(p) => format!("\x1b[{}m{s}\x1b[0m", p.0[i - 1]),
        None => s.to_string(),
    }
}

pub fn bold_current(p: &Pal) -> String {
    format!("1;{}", p.0[2].split(';').nth(1).unwrap_or("32"))
}

#[cfg(test)]
mod tests {
    use super::terminfo_colors;

    fn entry(magic: u16, colors: i32) -> Vec<u8> {
        let width = if magic == 0x021e { 4 } else { 2 };
        let mut data = vec![0; 12 + 14 * width];
        data[..2].copy_from_slice(&magic.to_le_bytes());
        data[6..8].copy_from_slice(&14_u16.to_le_bytes());
        let offset = 12 + 13 * width;
        if width == 2 {
            data[offset..offset + 2].copy_from_slice(&(colors as i16).to_le_bytes());
        } else {
            data[offset..offset + 4].copy_from_slice(&colors.to_le_bytes());
        }
        data
    }

    #[test]
    fn reads_legacy_and_extended_max_colors() {
        assert_eq!(terminfo_colors(&entry(0x011a, -1)), Some(-1));
        assert_eq!(terminfo_colors(&entry(0x021e, 256)), Some(256));
    }
}
