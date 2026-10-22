fn main() {
    // On macOS every framework a dependency asks for (Foundation, Security, ...) is recorded as a load command
    // in each binary of the package, used or not; dyld then pays for loading them on every `nvm-core` start.
    // Drop the unused ones so `nvm-core` links libSystem only (nvm-compat-spec.md §13.2).
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rustc-link-arg-bin=nvm-core=-Wl,-dead_strip_dylibs");
    }
}
