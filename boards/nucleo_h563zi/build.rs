use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=memory.x");
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo sets OUT_DIR"));
    fs::copy("memory.x", out.join("memory.x")).expect("copy board linker script");
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rustc-link-arg=-Tmemory.x");
}
