use std::process::Command;
use std::env;

fn main() {
    // Rerun if the kernels or Makefile are changed
    println!("cargo:rerun-if-changed=src/sim/C_kernels/Makefile");
    println!("cargo:rerun-if-changed=src/sim/C_kernels/kernels.c");

    let out_dir = env::var("OUT_DIR").unwrap();
    let target_features = env::var("CARGO_CFG_TARGET_FEATURE").unwrap();
    let avx2_available = target_features.contains("avx2");

    // Build the kernels with make
    Command::new("make")
        // Run from the kernels directory
        .current_dir("src/sim/C_kernels/")
        // -B forces make to rebuild, since cargo has already decided a rebuild is needed,
        // we don't need to let make decide whether to rebuild or not
        .args(&["-B", "kernels"])
        // Tell make whether to use AVX2
        .env("KERNELS_USE_AVX2", format!("{}", avx2_available as u8))
        .spawn()
        .expect("Failed to build C kernels");
    
    // Link the kernels
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=rellteekernels");
}