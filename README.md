### Instructions to build and run:
1. Install Rust and Cargo using [rustup](https://www.rust-lang.org/tools/install)
2. Navigate to the root directory of the project
3. Compile as described below
4. Run `./target/release/steps help` or `./target/debug/steps help` for usage instructions


### Compilation Options
- Basic compilation command is `cargo build` for debug or `cargo build --release` for release (optimized) mode
- To target the native CPU and allow better optimization, use `RUSTFLAGS="-C target-cpu=native" cargo build [...]`
    - A specific architecture like `skylake` can be specified instead of `native`
    - Make sure the target CPU selected will not cause issues for any computers you use to run the code
- Release mode and target specification are highly recommended
- On the MSU HPCC, use `RUSTFLAGS="-C target-cpu=skylake" cargo build --release`
- On a single personal computer, use `RUSTFLAGS="-C target-cpu=native" cargo build --release`
- For more portable libraries when compiling for Linux, the `crt-static` feature can be used
    - `RUSTFLAGS="[...] -C target-feature=+crt-static" cargo build [...] --target=x86_64-unknown-linux-gnu`

