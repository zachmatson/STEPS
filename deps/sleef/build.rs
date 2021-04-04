use std::env;

fn main() {
    let build_sleef = env::var("CARGO_FEATURE_BUILD_SLEEF").is_ok();
    if build_sleef {
        cmake::Config::new("sleef")
            .define("BUILD_TESTS", "FALSE")
            .define("BUILD_SHARED_LIBS", "FALSE")
            .define("BUILD_GNUABI_LIBS", "FALSE")
            .define("CMAKE_INSTALL_LIBDIR", "lib")
            .profile("Release")
            .build();

        println!("cargo:rerun-if-changed=sleef");
    }
}
