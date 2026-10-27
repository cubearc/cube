fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    // The complete Clap command tree exceeds Windows' default 1 MiB stack in
    // debug builds. Reserve a bounded 8 MiB for the CLI entry point.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
            println!("cargo:rustc-link-arg-bin=cube=/STACK:8388608");
        } else {
            println!("cargo:rustc-link-arg-bin=cube=-Wl,--stack,8388608");
        }
    }
}
