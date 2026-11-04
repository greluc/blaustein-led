//! Puts `memory.x` on the linker search path and selects the linker scripts.

use std::path::PathBuf;
use std::{env, fs};

fn main() {
    let out_dir =
        PathBuf::from(env::var_os("OUT_DIR").expect("Cargo sets OUT_DIR for build scripts"));
    fs::write(out_dir.join("memory.x"), include_bytes!("memory.x"))
        .expect("the build script can write to OUT_DIR");

    println!("cargo::rustc-link-search={}", out_dir.display());
    println!("cargo::rerun-if-changed=memory.x");
    println!("cargo::rustc-link-arg-bins=--nmagic");
    println!("cargo::rustc-link-arg-bins=-Tlink.x");
    if env::var_os("CARGO_FEATURE_DEBUG").is_some() {
        println!("cargo::rustc-link-arg-bins=-Tdefmt.x");
    }
}
