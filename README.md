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

Optional [rustc instrumentation PGO](https://doc.rust-lang.org/rustc/profile-guided-optimization.html)
works with the pinned toolchain and its `llvm-tools-preview` component. Run this
POSIX-shell recipe from the project root, with that rustup toolchain's `cargo` and
`rustc` on `PATH` (the Nix shell supplies rustup):

```sh
(
pgo_host=$(rustc -vV | sed -n 's/^host: //p')
pgo_profdata="$(rustc --print sysroot)/lib/rustlib/$pgo_host/bin/llvm-profdata"
# RUSTFLAGS overrides .cargo/config.toml: keep its native CPU flag explicitly.
pgo_flags='-Ctarget-cpu=native'
unset CARGO_ENCODED_RUSTFLAGS
RUSTFLAGS="$pgo_flags" cargo build --locked --release --target "$pgo_host" --target-dir target/pgo-baseline
rm -rf target/pgo-data
mkdir -p target/pgo-data/raw
RUSTFLAGS="$pgo_flags -Cprofile-generate=$PWD/target/pgo-data/raw" \
    cargo build --locked --release --target "$pgo_host" --target-dir target/pgo-generate
./target/pgo-generate/"$pgo_host"/release/template
# Repeat the preceding command with representative application workloads.
"$pgo_profdata" merge target/pgo-data/raw -o target/pgo-data/merged.profdata
# Cargo does not track profile contents: clear only the previous PGO-use build.
rm -rf target/pgo-use
RUSTFLAGS="$pgo_flags -Cprofile-use=$PWD/target/pgo-data/merged.profdata -Cllvm-args=-pgo-warn-missing-function" \
    cargo build --locked --release --target "$pgo_host" --target-dir target/pgo-use
./target/pgo-use/"$pgo_host"/release/template
hyperfine --warmup 3 "./target/pgo-baseline/$pgo_host/release/template" \
    "./target/pgo-use/$pgo_host/release/template"
)
```

`hyperfine` is optional. The explicit host target keeps PGO flags away from build
scripts and proc macros. All builds retain the release profile's fat LTO, strip,
codegen-unit and panic settings. If you customize Rust flags, put the same flags in
`pgo_flags` for all three builds. These `RUSTFLAGS` commands require a project path
without spaces. The subshell keeps these flags and environment changes local to
the recipe; normal builds retain their settings.

The greeting only smoke-tests the pipeline. Train your real executable on typical
inputs, hot paths and concurrency; let it exit normally (an abort/crash does not
flush profiles). Test the final binary and compare baseline versus optimized
release performance with repeated runs of a separate representative workload.
Keep source, toolchain, target and dependencies fixed; clear and regenerate the
profiles after changes. Use the matching rustup `llvm-profdata`, without `-sparse`,
and never substitute cargo-llvm-cov profiles. Missing-function warnings need review.

Stable rustc's documented workflow is instrumentation-based. This template offers
no sampling/AutoFDO or Propeller/relinking workflow: an LLVM backend alone does not
provide stable rustc options, linker integration or a validated profile converter
for those modes.
