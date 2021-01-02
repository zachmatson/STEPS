### Instructions to build and run:
1. Install Rust and Cargo using [rustup](https://www.rust-lang.org/tools/install)
2. Navigate to the root directory of the project
3. Run `cargo build --release` to build the release version or `cargo build` to build the debug version
4. Run `./target/release/relltee help` or `./target/debug/relltee help` for usage instructions


### Compilation Options
- To target the native CPU and allow better optimization, use `RUSTFLAGS="-C target-cpu=native" cargo build --release`
    - A specific architecture like `skylake` can be specified instead of `native`
    - Make sure the target CPU selected will not cause issues for any computers you use to run the code
- On 64-bit x86 systems with AVX2 (including Skylake CPUs), the `sleef` feature can be enabled
    - Sleef is a vectorized math library and offers considerable performance improvements
    - Requires nightly Rust, and requires dynamic Clang libraries to be available
    - Must specify the `target-cpu` as mentioned above
    - Compile with `[...] cargo build --release --features sleef`
- For more portable libraries when compiling for Linux, the `crt-static` feature can be used
    - `RUSTFLAGS="[...] -C target-feature=+crt-static" cargo build --release [...] --target=x86_64-unknown-linux-gnu`
- The recommended settings when compiling *for* the HPCC are then `RUSTFLAGS="-C target-cpu=skylake -C target-feature=+crt-static" cargo build --release --features sleef --target=x86_64-unknown-linux-gnu`
  - Currently, the `sleef` feature cannot be enabled when compiling *on* the HPCC because of the Clang installation. Working on a way around this. Compiling with these settings on another machine would produce an executable that can run well on all HPCC nodes.