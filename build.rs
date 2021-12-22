use std::env;

fn main() {
    // Rerun if the kernels are changed
    println!("cargo:rerun-if-changed=src/sim/C_kernels/kernels.c");

    // Build the kernels
    let mut builder = cc::Build::new();
    builder.file("src/sim/C_kernels/kernels.c");

    let target_features = env::var("CARGO_CFG_TARGET_FEATURE");
    if let Ok(target_features) = target_features {
        // Use SLEEF
        if target_features.contains("avx2") {
            builder
                .flag("-mavx2")
                .define("KERNELS_USE_AVX2", None)
                .include(sleef_shim::get_header_dir());

            println!(
                "cargo:rustc-link-search=native={}",
                sleef_shim::get_lib_dir().display()
            );
            println!("cargo:rustc-link-lib=static=sleef");
        }
    }

    let target = env::var("CARGO_CFG_TARGET_ARCH");
    if let Ok("wasm32") = &target.as_deref() {
        builder.compiler("clang");
    }

    builder.compile("libstepskernels.a");
}
