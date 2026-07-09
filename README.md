# rustmetrics

Aggregate test coverage, dead code, and function-size metrics for Rust projects.

## Prerequisites

- Rust toolchain
- `cargo-llvm-cov` installed (`cargo install cargo-llvm-cov`)

## Usage

```bash
rustmetrics [CRATE_ROOT]
rustmetrics --format json /path/to/crate
rustmetrics --no-coverage --no-dead-code /path/to/crate
```

## Metrics

- **Coverage**: wraps `cargo llvm-cov`
- **Dead code**: wraps `cargo check` with `dead_code` lint
- **Function size**: Tree-sitter AST analysis

## Configuration

Create `rustmetrics.toml`:

```toml
[providers]
coverage = { enabled = true, command = "cargo-llvm-cov" }
dead_code = { enabled = true, rustflags = "-W dead_code" }
function_size = { enabled = true }

[thresholds]
min_line_coverage = 0.80
max_function_lines = 100
max_dead_code_items = 0
```

## Development

```bash
cargo test
cargo run -- tests/fixtures/minimal_crate
```
