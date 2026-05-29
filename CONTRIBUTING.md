# Contributing to STEPS

Thank you for your interest in contributing to STEPS.

## Prerequisites

- [Rust and Cargo](https://www.rust-lang.org/tools/install) (installed via rustup)

## Building

```bash
cargo build --release
```

For native CPU optimizations:

```bash
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

## Running Tests

```bash
cargo test
```

## Submitting Changes

1. Fork the repository
2. Create a branch for your change
3. Make your changes and ensure tests pass
4. Open a pull request against `main`

For larger changes, please open an issue first to discuss the approach.

## License

By contributing, you agree that your contributions will be licensed under the
[GPL-3.0 License](LICENSE).
