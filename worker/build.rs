use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {

    let out = &PathBuf::from(
        env::var_os("OUT_DIR").expect("OUT_DIR environment variable must be set by Cargo"),
    );
    File::create(out.join("memory.x"))
        .expect("failed to create memory.x in OUT_DIR")
        .write_all(include_bytes!("memory.x"))
        .expect("failed to write memory.x to OUT_DIR");
    println!("cargo:rustc-link-search={}", out.display());

    println!("cargo:rerun-if-changed=memory.x");

    let target = env::var("TARGET").unwrap_or_default();
    if target.contains("thumb") || target.contains("none") {
        println!("cargo:rustc-link-arg-bins=--nmagic");
        println!("cargo:rustc-link-arg-bins=-Tlink.x");
        println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
    }
}
