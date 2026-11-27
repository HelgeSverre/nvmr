//! Restore the default SIGPIPE disposition.
//!
//! The Rust runtime ignores SIGPIPE, so writing to a closed pipe returns `EPIPE` and `println!`
//! panics on it (`nvm ls | head`). With the default disposition the process ends silently on
//! SIGPIPE, as nvm's shell functions and other command-line tools do.

#[cfg(unix)]
pub fn restore_default() {
    extern "C" {
        fn signal(signum: i32, handler: usize) -> usize;
    }
    // SIGPIPE is 13 and SIG_DFL is 0 on Linux and macOS.
    const SIGPIPE: i32 = 13;
    const SIG_DFL: usize = 0;
    // SAFETY: `signal` with SIG_DFL installs no Rust handler; it runs before any threads start.
    unsafe {
        signal(SIGPIPE, SIG_DFL);
    }
}

#[cfg(not(unix))]
pub fn restore_default() {}
