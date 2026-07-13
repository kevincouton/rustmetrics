# Agent Guide for rustmetrics

## Quick Start

```bash
cargo build
cargo test
cargo run -- tests/fixtures/minimal_crate
```

## Common Commands

- Format check: `cargo fmt --check`
- Lint: `cargo clippy -- -D warnings`
- Run all tests: `cargo test`
- Run one test: `cargo test --test runner_test`
- Manual smoke test: `cargo run -- --no-coverage --no-dead-code tests/fixtures/minimal_crate`

## Architecture

- `src/cli.rs` — CLI argument definitions (clap)
- `src/config.rs` — TOML configuration loading
- `src/model.rs` — shared data types (`Report`, `ThresholdViolation`, etc.)
- `src/providers/` — metric providers
  - `function_size.rs` — Tree-sitter based analysis
  - `dead_code.rs` — wraps `cargo check`
  - `coverage.rs` — wraps `cargo llvm-cov`
- `src/runner.rs` — orchestrates providers and applies thresholds
- `src/reporter/` — output formatters (human, JSON)
- `src/lib.rs` / `src/main.rs` — binary wiring
- `tests/fixtures/minimal_crate/` — integration test fixture

## Adding a Provider

1. Add a new module under `src/providers/`
2. Implement `MetricProvider` (see `src/providers/mod.rs`)
3. Return the appropriate `ProviderOutput` variant
4. Wire it into `Runner::new` in `src/runner.rs`
5. Add unit/integration tests

## Adding a Reporter

1. Add a module under `src/reporter/`
2. Implement the `Reporter` trait
3. Register it in `src/reporter/mod.rs::reporter_for`

## CI Expectations

The GitHub Actions workflow enforces formatting, clippy, tests, and release build. Keep all four green.
