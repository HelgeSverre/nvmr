//! Colour palette (§8): NVM_COLORS positions 1 installed, 2 system/LTS, 3 current, 4 missing, 5 default; 6 = bold of 2.
use std::{env, io::IsTerminal};

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
    match env::var("TERM") {
        Ok(t) if !t.is_empty() && t != "dumb" => {}
        _ => return None,
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
