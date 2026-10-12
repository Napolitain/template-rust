# template-rust

Minimal Rust template (edition 2024), hooks via [prek](https://prek.j178.dev).

```
src/lib.rs      library
src/main.rs     binary (keep it thin, excluded from coverage and mutation)
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
- pre-push: `cargo clippy -D warnings`, `cargo llvm-cov` (tests + coverage gate)

Opinionated defaults:

- Toolchain pinned in `rust-toolchain.toml`; rustup downloads it automatically.
- Clippy `pedantic`, `nursery` and `cargo` groups, `unsafe_code` forbidden.
- `-C target-cpu=native` for all builds (`.cargo/config.toml`).
- Release profile: fat LTO, 1 codegen unit, stripped, `panic = "abort"`.

## Optional Nix shell

Installing the tools directly on NixOS (or another OS) remains supported. The
optional shell supports x86_64/aarch64 Linux and Apple Silicon macOS. With Nix
flakes enabled:

```sh
nix develop --command "$SHELL"       # keep your shell, aliases and prompt
# Or: nix develop                   # use Nix's default Bash shell
```

Then run the normal development commands in this README. Each template has its own
`flake.nix` and `flake.lock`, using `github:NixOS/nixpkgs/nixpkgs-unstable`.
To refresh to the latest unstable packages, run `nix flake update nixpkgs`,
then leave and re-enter the shell. Commit the updated lock file with your project.

The shell supplies rustup, cargo-llvm-cov, cargo-mutants and a native linker.
Rustup still installs the compiler and components from `rust-toolchain.toml`.

## Complexity

Clippy's `cognitive_complexity` restriction lint is explicitly enabled and fails
above 15 through the existing `-D warnings` hooks and CI. The threshold lives in
`clippy.toml`. This is Clippy's own control-flow heuristic, not the same algorithm
as Sonar's cognitive complexity; Clippy has no separate cyclomatic-complexity lint.
See the [lint documentation](https://rust-lang.github.io/rust-clippy/master/index.html#cognitive_complexity)
for its limitations. Refactoring is manual.

## Coverage

[cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov) (`cargo install --locked cargo-llvm-cov`) runs the tests on pre-push and in CI, and fails below 80% line coverage (`--fail-under-lines` in `prek.toml`).

```sh
cargo llvm-cov --all-targets          # terminal summary
cargo llvm-cov --all-targets --html   # target/llvm-cov/html
cargo llvm-cov --all-targets --lcov --output-path lcov.info
```

## Property-based testing

[proptest](https://proptest-rs.github.io/proptest/) is a dev-dependency; see `tests/greet.rs`. It runs with `cargo test`. Failing cases are saved to `proptest-regressions/`; commit them so they're always replayed.

```rust
proptest::proptest! {
    #[test]
    fn sort_is_idempotent(mut v in proptest::collection::vec(0i32..100, 0..50)) {
        v.sort();
        let once = v.clone();
        v.sort();
        proptest::prop_assert_eq!(v, once);
    }
}
```

## Mutation testing

[cargo-mutants](https://mutants.rs) (`cargo install --locked cargo-mutants`), on demand and weekly in CI (`.github/workflows/mutation.yml`); fails if any mutant is missed:

```sh
prek run --hook-stage manual cargo-mutants --all-files
cargo mutants --in-diff <(git diff main)   # only mutants in your changes
```

Results are in `mutants.out/` (`missed.txt` lists survivors). Config in `.cargo/mutants.toml`.

## Profile-guided optimization

The pinned rustup toolchain includes `llvm-tools-preview` for
[rustc instrumentation PGO](https://doc.rust-lang.org/rustc/profile-guided-optimization.html).
Use its `cargo` and `rustc` on PATH (the Nix shell supplies rustup).
Create an executable `train.sh` that runs `"$1"` with representative inputs:

```sh
./tools/perf-build baseline
./tools/perf-build pgo -- ./train.sh
# Smoke test only: ./tools/perf-build pgo -- env
```

The helper builds instrumentation, runs training, merges with the matching LLVM
tool, then rebuilds with the profile. Training must exit normally to flush counts.
Both variants retain release **fat LTO**, one codegen unit, strip and panic settings.
Set `PERF_BINARY` when renaming the `template` executable. Native CPU flags
match `.cargo/config.toml` by default; `RUSTFLAGS` / `CARGO_ENCODED_RUSTFLAGS`
are honored for both variants. Keep those defaults aligned if you edit the config.

Outputs: `target/pgo-baseline/<host>/release/template` and
`target/pgo-use/<host>/release/template`. Fresh profiles and input records live
under `target/pgo-data/`; fresh paths prevent stale Cargo profile-use builds.
Normal builds remain independent. Run profiling builds serially and retrain after
source/toolchain/workload changes; coverage profiles cannot be substituted.

Check the optimized executable's correctness and compare both binaries on separate
representative evaluation inputs with repeated runs (e.g. `hyperfine --warmup 3`).
Review missing-function warnings and compare latency, memory and size too; no gain
is guaranteed. The greeting only checks the pipeline.

Stable rustc exposes instrumentation PGO here. AutoFDO and Propeller are not
provided as stable template modes; sharing an LLVM backend is insufficient.
