# rustmetrics

A CLI metrics aggregator for Rust projects.

## Metrics

- Test coverage via `cargo llvm-cov`
- Dead code via `cargo check`
- Function size via Tree-sitter

## Usage

```bash
cargo run -- [CRATE_ROOT]
```
