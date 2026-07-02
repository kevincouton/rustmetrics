# rustmetrics Design Specification

## 1. Overview

`rustmetrics` is a command-line metrics aggregator for Rust projects. Rather than re-implementing a test runner, coverage engine, or dead-code analyzer, it orchestrates existing Rust tooling and parses the results into a single, consistent report. It also performs its own lightweight AST analysis using **Tree-sitter** for metrics that existing tools do not provide at the desired granularity, starting with per-function size.

### Success criteria
- Extract **test coverage** by invoking `cargo llvm-cov` and parsing its output.
- Extract **dead code** by invoking `cargo check` with the `dead_code` lint and parsing JSON messages.
- Extract **per-function size metrics** by parsing Rust source with Tree-sitter.
- Emit a unified human-readable and JSON report.
- Support configurable thresholds that can fail CI when crossed.
- Include unit and integration tests covering all three metric providers.

## 2. Scope

### In scope (MVP)
- CLI argument parsing.
- Coverage provider backed by `cargo llvm-cov`.
- Dead-code provider backed by `cargo check` with `dead_code` lint.
- Function-size provider backed by Tree-sitter AST analysis.
- Unified report model and two reporters (human, JSON).
- TOML configuration for tool paths, thresholds, and enable/disable flags.
- Unit tests for Tree-sitter queries and integration tests for the full CLI.

### Out of scope (MVP)
- Re-implementing coverage instrumentation from scratch.
- Full inter-crate dead-code analysis beyond what `rustc` reports.
- Cyclomatic complexity or Halstead metrics.
- Historical trending or delta reports.
- IDE/Language Server Protocol integration.

## 3. Architecture

```text
┌─────────────┐     ┌─────────────────────┐
│ CLI (clap)  │────▶│ Metrics runner      │
└─────────────┘     │                     │
                    │ - load config       │
                    │ - run providers     │
                    │ - aggregate results │
                    └─────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        ▼                     ▼                     ▼
┌───────────────┐    ┌─────────────────┐    ┌──────────────────┐
│ Coverage      │    │ Dead code       │    │ Function size    │
│ provider      │    │ provider        │    │ provider         │
│               │    │                 │    │                  │
│ cargo llvm-cov│    │ cargo check     │    │ tree-sitter-rust │
└───────────────┘    └─────────────────┘    └──────────────────┘
        │                     │                     │
        └─────────────────────┼─────────────────────┘
                              ▼
                    ┌─────────────────┐
                    │ Report          │
                    │ (human / JSON)  │
                    └─────────────────┘
```

### Components

| Component | Responsibility |
|-----------|----------------|
| `cli` | Parse command-line arguments. |
| `config` | Load and merge TOML configuration. |
| `runner` | Orchestrate providers, collect metric results, apply thresholds. |
| `providers/coverage` | Run `cargo llvm-cov` and parse coverage data. |
| `providers/dead_code` | Run `cargo check` and parse `dead_code` warnings. |
| `providers/function_size` | Use Tree-sitter to find functions and measure size. |
| `model` | Shared data structures for metrics and reports. |
| `reporter` | Format and emit the final report. |

## 4. Data Flow

1. CLI parses arguments and determines the crate root (default: current directory).
2. `Config::load(path)` merges the user config with built-in defaults.
3. `Runner::new(config)` constructs the enabled providers.
4. Each provider runs independently:
   - **Coverage provider**: Executes `cargo llvm-cov --json` (or equivalent) and parses the JSON coverage report.
   - **Dead-code provider**: Executes `cargo check --message-format=json` with `RUSTFLAGS="-W dead_code"` and parses `dead_code` warnings.
   - **Function-size provider**: Walks `.rs` files with `walkdir`, parses each with Tree-sitter, and runs a query to find `function_item` nodes.
5. The runner aggregates all results into a `Report`.
6. The selected reporter prints the report.
7. The process exits `1` if any configured threshold is violated or a provider fails; otherwise `0`.

## 5. Data Models

### `CoverageSummary`
```rust
pub struct CoverageSummary {
    pub line_rate: f64,       // 0.0 .. 1.0
    pub branch_rate: Option<f64>,
    pub function_rate: Option<f64>,
    pub lines_covered: u64,
    pub lines_total: u64,
}
```

### `FunctionMetric`
```rust
pub struct FunctionMetric {
    pub file: PathBuf,
    pub name: String,
    pub line_start: usize,    // 1-indexed
    pub line_end: usize,      // 1-indexed, inclusive
    pub line_count: usize,
    pub statement_count: usize,
}
```

### `DeadCodeItem`
```rust
pub struct DeadCodeItem {
    pub file: PathBuf,
    pub name: String,
    pub item_kind: String,    // e.g., "function", "struct", "constant"
    pub line: usize,          // 1-indexed
    pub column: usize,        // 1-indexed
}
```

### `Report`
```rust
pub struct Report {
    pub crate_root: PathBuf,
    pub coverage: Option<CoverageSummary>,
    pub dead_code: Vec<DeadCodeItem>,
    pub function_sizes: Vec<FunctionMetric>,
    pub threshold_violations: Vec<ThresholdViolation>,
}
```

### `MetricProvider` trait
```rust
pub trait MetricProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn collect(&self, crate_root: &Path) -> Result<ProviderOutput, MetricError>;
}

pub enum ProviderOutput {
    Coverage(CoverageSummary),
    DeadCode(Vec<DeadCodeItem>),
    FunctionSizes(Vec<FunctionMetric>),
}
```

Providers are stateless and constructed from config. Each provider returns only the metric type it produces.

## 6. Providers

### 6.1 Coverage provider
- **Tool:** `cargo llvm-cov` (preferred) with fallback to `cargo tarpaulin` if configured.
- **Command:** `cargo llvm-cov --json` (output captured from stdout).
- **Output parsed:** JSON coverage summary and per-file rates. The provider also accepts an optional `--output-path` for generated reports when configured.
- **Failure mode:** If the tool is missing or returns an error, the provider returns `MetricError` and the runner prints the error. The other providers still run.

### 6.2 Dead-code provider
- **Tool:** `cargo check`.
- **Command:** `RUSTFLAGS="-W dead_code" cargo check --message-format=json`.
- **Output parsed:** `reason: "compiler-message"` entries with `code: "dead_code"`.
- **Extraction:** `file`, `line`, `column`, item name, and item kind from the rendered message or span.

### 6.3 Function-size provider
- **Tool:** Tree-sitter.
- **Parser:** `tree-sitter` + `tree-sitter-rust`.
- **Query:** `(function_item name: (identifier) @name) @function`.
- **Metrics computed:**
  - `line_count`: `line_end - line_start + 1` from the Tree-sitter node range.
  - `statement_count`: count of statement-level descendants inside the function body (e.g., `expression_statement`, `let_declaration`, `return_expression`, `if_expression`, `loop_expression`).
- **Failure mode:** Parse errors emit a warning and continue; the file is skipped.

## 7. CLI

```bash
rustmetrics [OPTIONS] [CRATE_ROOT]
```

### Arguments
- `CRATE_ROOT`: path to the Rust crate to analyze. Defaults to the current directory.

### Options
- `-f, --format <FORMAT>`: Output format, `human` or `json`. Default: `human`.
- `-c, --config <PATH>`: Path to a TOML configuration file.
- `--no-coverage`: Skip the coverage provider.
- `--no-dead-code`: Skip the dead-code provider.
- `--no-function-size`: Skip the function-size provider.
- `-h, --help`: Show help.
- `-V, --version`: Show version.

### Exit codes
- `0`: All providers succeeded and no thresholds violated.
- `1`: A provider failed, a threshold was violated, or a required tool was missing.

## 8. Configuration

Config files are TOML. The tool searches for `.rustmetrics.toml` or `rustmetrics.toml` in the crate root when `--config` is not provided.

```toml
[providers]
coverage = { enabled = true, command = "cargo llvm-cov" }
dead_code = { enabled = true, rustflags = "-W dead_code" }
function_size = { enabled = true }

[thresholds]
min_line_coverage = 0.80
max_function_lines = 100
max_dead_code_items = 0
```

### Semantics
- All providers are enabled by default.
- `providers.<name>.enabled` toggles a provider.
- `providers.coverage.command` selects the coverage backend (`cargo-llvm-cov` or `cargo-tarpaulin`).
- Thresholds are optional. If present and violated, the final exit code is `1`.

## 9. Output Formats

### Human format
```text
Crate: /home/user/my-crate

Coverage (cargo llvm-cov)
  Line coverage: 84.5% (169 / 200 lines)

Dead code (cargo check)
  src/lib.rs:42:1 unused function `old_helper`

Function size (tree-sitter)
  src/lib.rs:10  process_data  142 lines  38 statements

Threshold violations
  max_function_lines exceeded: process_data (142 lines)
```

### JSON format
```json
{
  "crate_root": "/home/user/my-crate",
  "coverage": {
    "line_rate": 0.845,
    "lines_covered": 169,
    "lines_total": 200
  },
  "dead_code": [
    {
      "file": "src/lib.rs",
      "name": "old_helper",
      "item_kind": "function",
      "line": 42,
      "column": 1
    }
  ],
  "function_sizes": [
    {
      "file": "src/lib.rs",
      "name": "process_data",
      "line_start": 10,
      "line_end": 151,
      "line_count": 142,
      "statement_count": 38
    }
  ],
  "threshold_violations": [
    {
      "threshold": "max_function_lines",
      "actual": 142,
      "limit": 100,
      "function": "process_data"
    }
  ]
}
```

## 10. Error Handling

- **Provider not installed:** Print a clear error (e.g., "`cargo-llvm-cov` not found") and exit `1`.
- **Coverage command failure:** If tests fail or coverage cannot be generated, the provider returns an error; the runner may still run other providers unless configured to fail fast.
- **Cargo check failure:** Dead-code provider parses messages even when the build has warnings. Hard build errors are surfaced as errors.
- **Tree-sitter parse errors:** Emit a warning and skip the file. Analysis continues.
- **Config errors:** Invalid TOML or unknown values cause immediate exit `1` with a descriptive message.
- **Internal errors:** The runner must not panic on malformed input; errors are collected and reported.

## 11. Testing Strategy

### Unit tests
- Tree-sitter query tests on inline Rust snippets asserting correct function detection and line/statement counts.
- Coverage JSON parsing tests using sample `cargo llvm-cov` output fixtures.
- Dead-code message parsing tests using sample `cargo check --message-format=json` output.

### Integration tests
- The `tests/integration_tests.rs` harness runs the CLI against sample crates in `tests/fixtures/`.
- Fixtures:
  - `tests/fixtures/minimal_crate/`: a small crate with one large function, one unused function, and tests.
  - Asserts on JSON report structure and exit code.

## 12. Project Structure

```text
rustmetrics/
├── Cargo.toml
├── README.md
├── rustmetrics.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── cli.rs
│   ├── config.rs
│   ├── runner.rs
│   ├── model.rs
│   ├── reporter/
│   │   ├── mod.rs
│   │   ├── human.rs
│   │   └── json.rs
│   └── providers/
│       ├── mod.rs
│       ├── coverage.rs
│       ├── dead_code.rs
│       └── function_size.rs
└── tests/
    ├── fixtures/
    │   └── minimal_crate/
    │       ├── Cargo.toml
    │       ├── src/
    │       │   └── lib.rs
    │       └── tests/
    │           └── basic.rs
    └── integration_tests.rs
```

## 13. Dependencies

| Crate | Purpose |
|-------|---------|
| `tree-sitter` | Generic Tree-sitter parser runtime. |
| `tree-sitter-rust` | Rust grammar for Tree-sitter. |
| `clap` | CLI parsing (derive API). |
| `serde` + `serde_json` + `toml` | JSON/TOML parsing and serialization. |
| `walkdir` | Directory traversal for `.rs` files. |
| `assert_cmd` + `predicates` | Integration test harness. |
| `tempfile` | Temporary files for provider output. |

## 14. Future Extensions

Deferred but architecturally supported:
- Additional metric providers (cyclomatic complexity, unsafe usage via `cargo-geiger`).
- Historical trending and delta reports.
- SARIF output format.
- Thresholds per file or module.
- Autofix suggestions for dead code.

## 15. Decisions and Trade-offs

- **Tree-sitter vs. `syn`**: Tree-sitter handles incomplete or malformed code more gracefully and uses a language-agnostic query API. This keeps the door open for analyzing other languages later. The downside is slightly more setup than `syn`.
- **Tool orchestration vs. re-implementation**: Coverage and dead-code detection are delegated to `cargo llvm-cov` and `cargo check` because re-implementing them would require compiler internals. Function size is self-implemented because existing tools do not expose it reliably.
- **Human + JSON output first**: These two formats cover terminal usage and CI ingestion. SARIF can be added later.
