//! nvm-fetch: HTTPS download, checksum, extraction (spec §6.1.1, §13.2).
//! Protocol with nvm-core:
//!   nvm-fetch get <url>                              body -> stdout; rc 4 on failure, 5 on 404
//!   nvm-fetch install <mirror> <vX.Y.Z> <nvm_dir> [--no-progress] [--offline]
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::exit;

fn agent() -> ureq::Agent {
    ureq::Agent::new_with_config(
        ureq::config::Config::builder()
            .http_status_as_error(false)
            .timeout_global(Some(std::time::Duration::from_secs(900)))
            .build(),
    )
}

fn arch() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        "x86" => "x86",
        "loongarch64" => "loong64",
        o => o,
    }
}

fn slug(ver: &str) -> String {
    let os = if cfg!(target_os = "macos") {
        "darwin"
    } else {
        "linux"
    };
    let mut a = arch().to_string();
    if os == "darwin" && a == "arm64" && major(ver) < 16 {
        a = "x64".into();
    }
    if os == "linux" && Path::new("/etc/alpine-release").exists() {
        a.push_str("-musl");
    }
    format!("node-{ver}-{os}-{a}")
}

fn major(v: &str) -> u64 {
    v.trim_start_matches('v')
        .split('.')
        .next()
        .and_then(|x| x.parse().ok())
        .unwrap_or(0)
}

fn http_get(url: &str) -> Result<Vec<u8>, i32> {
    let mut r = agent().get(url).call().map_err(|_| 4)?;
    let st = r.status().as_u16();
    if st == 404 {
        eprintln!("HTTP 404 at URL {url}");
        return Err(5);
    }
    if st >= 400 {
        eprintln!("download from {url} failed");
        return Err(4);
    }
    let mut b = Vec::new();
    r.body_mut()
        .as_reader()
        .read_to_end(&mut b)
        .map_err(|_| 4)?;
    Ok(b)
}

fn download(url: &str, dest: &Path, progress: bool) -> Result<String, i32> {
    let mut have = fs::metadata(dest).map(|m| m.len()).unwrap_or(0);
    let mut req = agent().get(url);
    if have > 0 {
        req = req.header("Range", format!("bytes={have}-"));
    }
    if let Ok(h) = std::env::var("NVM_AUTH_HEADER") {
        let clean: String = h
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || " :_.+/=~-".contains(*c))
            .collect();
        req = req.header("Authorization", clean);
    }
    let mut resp = req.call().map_err(|_| {
        eprintln!("download from {url} failed");
        4
    })?;
    let st = resp.status().as_u16();
    if st == 404 {
        eprintln!("HTTP 404 at URL {url}");
        return Err(5);
    }
    if st == 416 {
        have = 0;
        let _ = fs::remove_file(dest);
        return download(url, dest, progress);
    }
    if st >= 400 {
        eprintln!("download from {url} failed");
        return Err(4);
    }
    let resumed = st == 206;
    let mut f = if resumed {
        fs::OpenOptions::new()
            .append(true)
            .open(dest)
            .map_err(|_| 4)?
    } else {
        have = 0;
        File::create(dest).map_err(|_| 4)?
    };
    let total = resp.body().content_length().map(|l| l + have);
    let mut rd = resp.body_mut().as_reader();
    let mut buf = vec![0u8; 256 * 1024];
    let mut done = have;
    let mut last = 0u64;
    loop {
        let n = rd.read(&mut buf).map_err(|_| 4)?;
        if n == 0 {
            break;
        }
        f.write_all(&buf[..n]).map_err(|_| 4)?;
        done += n as u64;
        if progress && done - last > 512 * 1024 {
            last = done;
            if let Some(t) = total {
                eprint!("\r{:5.1}%", done as f64 * 100.0 / t as f64);
            }
        }
    }
    if progress {
        eprintln!("\r100.0%");
    }
    drop(f);
    sha_file(dest).map_err(|_| 4)
}

fn sha_file(p: &Path) -> std::io::Result<String> {
    let mut f = File::open(p)?;
    let mut h = Sha256::new();
    let mut b = vec![0u8; 256 * 1024];
    loop {
        let n = f.read(&mut b)?;
        if n == 0 {
            return Ok(h.finalize().iter().map(|x| format!("{x:02x}")).collect());
        }
        h.update(&b[..n]);
    }
}

fn extract(tarball: &Path, into: &Path) -> Result<(), i32> {
    let f = File::open(tarball).map_err(|_| 1)?;
    let dec: Box<dyn Read> = if tarball.to_string_lossy().ends_with(".xz") {
        Box::new(liblzma::read::XzDecoder::new(f))
    } else {
        Box::new(flate2::read::GzDecoder::new(f))
    };
    let mut ar = tar::Archive::new(dec);
    ar.set_preserve_permissions(true);
    ar.set_unpack_xattrs(false);
    ar.set_preserve_ownerships(false);
    for e in ar.entries().map_err(|_| 1)? {
        let mut e = e.map_err(|_| 1)?;
        let p = e.path().map_err(|_| 1)?.into_owned();
        let stripped: PathBuf = p.components().skip(1).collect();
        if stripped.as_os_str().is_empty() || stripped.components().any(|c| c.as_os_str() == "..") {
            continue;
        }
        let out = into.join(stripped);
        if let Some(d) = out.parent() {
            fs::create_dir_all(d).map_err(|_| 1)?;
        }
        e.unpack(&out).map_err(|_| 1)?;
    }
    Ok(())
}

fn install(args: &[String]) -> i32 {
    let (mirror, ver, nvm_dir) = (&args[0], &args[1], &args[2]);
    let progress = !args.iter().any(|a| a == "--no-progress")
        && std::env::var("NVM_NO_PROGRESS").as_deref() != Ok("1");
    let offline = args.iter().any(|a| a == "--offline");
    let slug = slug(ver);
    let file = format!("{slug}.tar.xz");
    let cdir = Path::new(nvm_dir).join(".cache/bin").join(&slug);
    let tarball = cdir.join(&file);
    let _ = fs::create_dir_all(&cdir);
    let url = format!("{mirror}/{ver}/{file}");

    let want = if offline {
        None
    } else {
        let sums = match http_get(&format!("{mirror}/{ver}/SHASUMS256.txt")) {
            Ok(b) => String::from_utf8_lossy(&b).into_owned(),
            Err(c) => return c,
        };
        sums.lines().find_map(|l| {
            let mut it = l.split_whitespace();
            let (h, n) = (it.next()?, it.next()?);
            (n == file).then(|| h.to_string())
        })
    };

    let mut have_cache = false;
    if tarball.exists() {
        eprintln!("Local cache found: {}", tarball.display());
        if offline {
            eprintln!("Offline: using cached archive {}", tarball.display());
            have_cache = true;
        } else if sha_file(&tarball).ok().as_deref() == want.as_deref() && want.is_some() {
            eprintln!(
                "Checksums match! Using existing downloaded archive {}",
                tarball.display()
            );
            have_cache = true;
        } else {
            eprintln!("Checksum check failed!\nRemoving the broken local cache...");
            let _ = fs::remove_file(&tarball);
        }
    } else if offline {
        eprintln!("Offline: no cached archive found for {slug}");
        return 4;
    }
    if !have_cache {
        eprintln!("Downloading {url}...");
        let got = match download(&url, &tarball, progress) {
            Ok(g) => g,
            Err(c) => return c,
        };
        eprintln!("Computing checksum with sha256sum");
        match want {
            Some(w) if w == got => eprintln!("Checksums matched!"),
            Some(w) => {
                eprintln!("Checksums do not match: '{got}' found, '{w}' expected.");
                return 6;
            }
            None => {
                eprintln!("Checksums do not match: '{got}' found, '' expected.");
                return 6;
            }
        }
    }
    let files = cdir.join("files");
    let _ = fs::remove_dir_all(&files);
    if fs::create_dir_all(&files).is_err() || extract(&tarball, &files).is_err() {
        let _ = fs::remove_dir_all(&files);
        return 1;
    }
    let vdir = Path::new(nvm_dir).join("versions/node").join(ver);
    let _ = fs::create_dir_all(vdir.parent().unwrap());
    let _ = fs::remove_dir_all(&vdir);
    if fs::rename(&files, &vdir).is_err() {
        return 1;
    }
    0
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let rc = match a.first().map(String::as_str) {
        Some("get") if a.len() >= 2 => match http_get(&a[1]) {
            Ok(b) => {
                let _ = std::io::stdout().write_all(&b);
                0
            }
            Err(c) => c,
        },
        Some("install") if a.len() >= 4 => install(&a[1..]),
        _ => {
            eprintln!("usage: nvm-fetch get <url> | install <mirror> <version> <nvm_dir>");
            2
        }
    };
    exit(rc);
}
