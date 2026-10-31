fn main() {
    const LINEAR_MEMORY_BYTES: u64 = 256 * 1024 * 1024;
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-env=CUBE_WASM_LINEAR_MEMORY_BYTES={LINEAR_MEMORY_BYTES}");
    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32") {
        println!("cargo:rustc-link-arg=--max-memory={LINEAR_MEMORY_BYTES}");
    }
}
