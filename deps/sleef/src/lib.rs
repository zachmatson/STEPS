use std::path::{Path, PathBuf};

const BASE_DIR: &'static str = env!("OUT_DIR");

fn warn_nobuild() -> ! {
    panic!(
        "sleef-shim was compiled without the \"build-sleef\" option. To use the SLEEF library, compile with this option."
    )
}

pub fn get_lib_dir() -> PathBuf {
    if cfg!(feature = "build-sleef") {
        Path::new(BASE_DIR).join("lib")
    } else {
        warn_nobuild()
    }
}

pub fn get_header_dir() -> PathBuf {
    if cfg!(feature = "build-sleef") {
        Path::new(BASE_DIR).join("include")
    } else {
        warn_nobuild()
    }
}
