use std::{process::{exit, Command}, env, error::Error};

fn build_kernels_inner() -> Result<bool, Box<dyn Error>> {
    let target_features = env::var("CARGO_CFG_TARGET_FEATURE").unwrap();
    let avx2_available = target_features.contains("avx2");

    Ok(Command::new("make")
        // Run from the kernels directory
        .current_dir("src/sim/C_kernels/")
        // -B forces make to rebuild, since cargo has already decided a rebuild is needed,
        // we don't need to let make decide whether to rebuild or not
        .args(&["-B", "kernels"])
        // Tell make whether to use AVX2
        .env("KERNELS_USE_AVX2", format!("{}", avx2_available as u8))
        // Provide SLEEF directory
        .env("SLEEF_HEADER_DIR", sleef_shim::get_header_dir())
        .spawn()?
        .wait()?
        .success())
}

fn build_kernels() -> String {
    match build_kernels_inner() {
        Err(_) | Ok(false) => {
            eprintln!("Failed to build RELLTEE C kernels");
            exit(1);
        }
        _ => ()
    }

    env::var("OUT_DIR").unwrap()
}

fn main() {
    // Rerun if the kernels or Makefile are changed
    println!("cargo:rerun-if-changed=src/sim/C_kernels/Makefile");
    println!("cargo:rerun-if-changed=src/sim/C_kernels/kernels.c");

    // Build the kernels with make
    let relltee_lib_dir = build_kernels();
    
    // Link the kernels
    println!("cargo:rustc-link-search=native={}", relltee_lib_dir);
    println!("cargo:rustc-link-lib=static=rellteekernels");
    // Link SLEEF
    println!("cargo:rustc-link-search=native={}", sleef_shim::get_lib_dir().display());
    println!("cargo:rustc-link-lib=static=sleef");
}
