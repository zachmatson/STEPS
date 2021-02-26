use std::process::Command;
use std::env;

fn main() {
    println!("cargo:rerun-if-changed=src/sim/C_kernels/Makefile");
    println!("cargo:rerun-if-changed=src/sim/C_kernels/kernels.c");
    let out_dir = env::var("OUT_DIR").unwrap();
    Command::new("make")
        .current_dir("src/sim/C_kernels/")
        .args(&["kernels"])
        .spawn()
        .expect("Failed to build C kernels");
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=rellteekernels");
}