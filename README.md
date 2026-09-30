# template-rust

Minimal Rust template (edition 2024), hooks via [prek](https://prek.j178.dev).

```
src/lib.rs      library
src/main.rs     binary
tests/          integration tests
```

```sh
prek install                         # install pre-commit + pre-push hooks
cargo run
cargo test
prek run -a                          # pre-commit hooks
prek run -a --hook-stage pre-push    # pre-push hooks
```

- pre-commit: `cargo fmt`, `cargo clippy --fix`, `cargo clippy -D warnings`
- pre-push: `cargo clippy -D warnings`, `cargo test`

Opinionated defaults:

- Toolchain pinned in `rust-toolchain.toml`; rustup downloads it automatically.
- Clippy `pedantic`, `nursery` and `cargo` groups, `unsafe_code` forbidden.
- `-C target-cpu=native` for all builds (`.cargo/config.toml`).
- Release profile: fat LTO, 1 codegen unit, stripped, `panic = "abort"`.

Coverage ([cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov), install with `cargo install --locked cargo-llvm-cov`):

```sh
cargo llvm-cov --all-targets          # terminal summary
cargo llvm-cov --all-targets --html   # target/llvm-cov/html
cargo llvm-cov --all-targets --lcov --output-path lcov.info
```
