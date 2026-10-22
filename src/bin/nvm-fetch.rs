//! nvm-fetch: HTTPS download, checksum, extraction (spec §6.1.1, §13.2).
//! Protocol with nvm-core:
//!   nvm-fetch get <url>                              body -> stdout; rc 4 on failure, 5 on 404
//!   nvm-fetch install <mirror> <vX.Y.Z> <nvm_dir> [--dest=<dir>] [--source] [--offline] [--no-progress]
//!                                                    download + verify + extract; with --dest, move into place
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

/// A GET request carrying `$NVM_AUTH_HEADER` (sanitized like upstream's `nvm_sanitize_auth_header`),
/// which upstream sends with every download: indexes, checksums and tarballs.
fn request(url: &str) -> ureq::RequestBuilder<ureq::typestate::WithoutBody> {
    let req = agent().get(url);
    match std::env::var("NVM_AUTH_HEADER") {
        Ok(h) if !h.is_empty() => {
            let clean: String = h
                .chars()
                .filter(|c| c.is_ascii_alphanumeric() || " :_.+/=~-".contains(*c))
                .collect();
            req.header("Authorization", clean)
        }
        _ => req,
    }
}

fn http_get(url: &str) -> Result<Vec<u8>, i32> {
    let mut r = request(url).call().map_err(|_| 4)?;
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
    let _ = ctrlc::set_handler(|| exit(130)); // partial tarball stays on disk for `Range:` resume
    let mut have = fs::metadata(dest).map_or(0, |m| m.len());
    let mut req = request(url);
    if have > 0 {
        req = req.header("Range", format!("bytes={have}-"));
    }
    let mut resp = req.call().map_err(|_| {
        eprintln!("download from {url} failed");
        4
    })?;
    let st = resp.status().as_u16();
    if st == 416 {
        let _ = fs::remove_file(dest);
        return download(url, dest, progress);
    }
    if st >= 400 {
        // curl --fail prints its own error line first
        eprintln!("curl: (22) The requested URL returned error: {st}");
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

/// `nvm_sanitize_path`: show `$NVM_DIR` and `$HOME` symbolically.
fn sanitize(nvm_dir: &str, p: &Path) -> String {
    let mut o = p.display().to_string();
    if o != nvm_dir {
        o = o.replace(nvm_dir, "${NVM_DIR}");
    }
    if let Ok(h) = std::env::var("HOME") {
        if !h.is_empty() && o != h {
            o = o.replace(&h, "${HOME}");
        }
    }
    o
}

/// Options for `nvm-fetch install`.
struct Install<'a> {
    mirror: &'a str,
    version: &'a str,
    nvm_dir: &'a str,
    /// Version directory to move the extracted tree into; `None` leaves it in the cache (source builds).
    dest: Option<&'a Path>,
    source: bool,
    offline: bool,
    progress: bool,
}

impl<'a> Install<'a> {
    fn parse(mirror: &'a str, version: &'a str, nvm_dir: &'a str, flags: &'a [String]) -> Self {
        let has = |f: &str| flags.iter().any(|a| a == f);
        Install {
            mirror,
            version,
            nvm_dir,
            dest: flags
                .iter()
                .find_map(|a| a.strip_prefix("--dest="))
                .map(Path::new),
            source: has("--source"),
            offline: has("--offline"),
            progress: !has("--no-progress")
                && std::env::var("NVM_NO_PROGRESS").as_deref() != Ok("1"),
        }
    }
}

fn install(opts: &Install) -> i32 {
    let Install {
        mirror,
        version: ver,
        nvm_dir,
        source,
        offline,
        progress,
        ..
    } = *opts;
    let slug = if source {
        format!("node-{ver}")
    } else {
        slug(ver)
    };
    let file = format!("{slug}.tar.xz");
    let cdir = Path::new(nvm_dir)
        .join(if source { ".cache/src" } else { ".cache/bin" })
        .join(&slug);
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
        eprintln!("Local cache found: {}", sanitize(nvm_dir, &tarball));
        if offline {
            eprintln!(
                "Offline: using cached archive {}",
                sanitize(nvm_dir, &tarball)
            );
            have_cache = true;
        } else if sha_file(&tarball).ok().as_deref() == want.as_deref() && want.is_some() {
            eprintln!(
                "Checksums match! Using existing downloaded archive {}",
                sanitize(nvm_dir, &tarball)
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
    {
        // Ctrl-C: drop the half-extracted tree, keep the (verified) tarball for next time.
        let f = files.clone();
        let _ = ctrlc::set_handler(move || {
            let _ = fs::remove_dir_all(&f);
            exit(130);
        });
    }
    let _ = fs::remove_dir_all(&files);
    if fs::create_dir_all(&files).is_err() || extract(&tarball, &files).is_err() {
        let _ = fs::remove_dir_all(&files);
        return 1;
    }
    let Some(vdir) = opts.dest else {
        return 0; // source build: the caller builds in `files/`
    };
    let parent = vdir.parent().unwrap_or_else(|| Path::new("."));
    let _ = fs::create_dir_all(parent);
    // Swap safely: move the previous install aside and restore it if the rename below fails,
    // instead of deleting it before the new files exist. `.old-*` names start with a dot, so
    // version enumeration (`starts_with('v')`) never picks them up.
    let old = parent.join(format!(".old-{slug}"));
    let _ = fs::remove_dir_all(&old);
    if vdir.exists() && fs::rename(vdir, &old).is_err() {
        return 1;
    }
    if fs::rename(&files, vdir).is_err() {
        let _ = fs::rename(&old, vdir);
        return 1;
    }
    let _ = fs::remove_dir_all(&old);
    0
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let rc = match args.as_slice() {
        [cmd, url] if cmd == "get" => match http_get(url) {
            Ok(b) => {
                let _ = std::io::stdout().write_all(&b);
                0
            }
            Err(c) => c,
        },
        [cmd, mirror, version, nvm_dir, flags @ ..] if cmd == "install" => {
            install(&Install::parse(mirror, version, nvm_dir, flags))
        }
        _ => {
            eprintln!("usage: nvm-fetch get <url> | install <mirror> <version> <nvm_dir> [--dest=<dir>] [--source] [--offline] [--no-progress]");
            2
        }
    };
    exit(rc);
}
