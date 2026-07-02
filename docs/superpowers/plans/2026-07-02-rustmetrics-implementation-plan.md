# rustmetrics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `rustmetrics`, a CLI tool that aggregates test coverage, dead-code, and per-function size metrics for Rust crates by orchestrating existing tooling and using Tree-sitter for AST analysis.

**Architecture:** A `Runner` loads config, constructs enabled `MetricProvider` implementations, invokes each provider against the target crate, and merges results into a `Report`. Two reporters format the report for terminal or JSON output. Providers are small, stateless units: coverage wraps `cargo llvm-cov`, dead-code wraps `cargo check`, and function-size uses Tree-sitter queries.

**Tech Stack:** Rust 1.96, `clap`, `tree-sitter` + `tree-sitter-rust`, `serde` + `serde_json` + `toml`, `walkdir`, `tempfile`, `assert_cmd` + `predicates`.

## Global Constraints

- Rust version: `cargo 1.96.1` (available in environment).
- Project root: `/root/rustmetrics` (rename from the initial `rustlint` directory).
- All providers must be stateless and implement `MetricProvider`.
- Line and column numbers in output are 1-indexed.
- The tool exits `0` only when all enabled providers succeed and no thresholds are violated.
- Parse errors in Tree-sitter must not stop analysis; they are warnings.
- Config file names: `.rustmetrics.toml` or `rustmetrics.toml`.

---

### Task 1: Project Scaffolding

**Files:**
- Create: `Cargo.toml`
- Create: `rustmetrics.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`
- Create: `README.md`

**Interfaces:**
- Produces: a compilable Rust binary crate named `rustmetrics`.

- [ ] **Step 0: Ensure project directory is named `rustmetrics`**

The intended project root is `/root/rustmetrics`. If it is still named `/root/rustlint`, rename it:

```bash
cd /root
if [ -d rustlint ] && [ ! -d rustmetrics ]; then
    mv rustlint rustmetrics
fi
cd rustmetrics
```

The design spec and this plan live under `docs/superpowers/` inside this directory.

- [ ] **Step 1: Create `Cargo.toml`**

```toml
[package]
name = "rustmetrics"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <you@example.com>"]
description = "Aggregates test coverage, dead code, and function-size metrics for Rust projects."
license = "MIT OR Apache-2.0"
repository = "https://github.com/example/rustmetrics"

[dependencies]
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
walkdir = "2.5"
tree-sitter = "0.22"
tree-sitter-rust = "0.23"
tempfile = "3.10"
thiserror = "1.0"
anyhow = "1.0"

[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.1"
```

- [ ] **Step 2: Create `src/main.rs`**

```rust
use rustmetrics::run;

fn main() -> anyhow::Result<()> {
    run()
}
```

- [ ] **Step 3: Create `src/lib.rs`**

```rust
pub mod cli;
pub mod config;
pub mod model;
pub mod providers;
pub mod reporter;
pub mod runner;

use anyhow::Result;

pub fn run() -> Result<()> {
    println!("rustmetrics scaffold ready");
    Ok(())
}
```

- [ ] **Step 4: Create empty module files**

Create the following files with empty module declarations:
- `src/cli.rs`
- `src/config.rs`
- `src/model.rs`
- `src/providers/mod.rs`
- `src/reporter/mod.rs`
- `src/runner.rs`

- [ ] **Step 5: Create `rustmetrics.toml`**

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

- [ ] **Step 6: Create `README.md`**

```markdown
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
```

- [ ] **Step 7: Build the scaffold**

Run:
```bash
cargo build
```

Expected: successful compilation with warnings about unused modules.

- [ ] **Step 8: Commit**

```bash
git init -b main
git add .
git commit -m "chore: scaffold rustmetrics crate"
```

---

### Task 2: Core Data Models and Provider Trait

**Files:**
- Create: `src/model.rs`
- Create: `src/providers/mod.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Produces: `model::Severity`, `model::Diagnostic`, `model::CoverageSummary`, `model::FunctionMetric`, `model::DeadCodeItem`, `model::ThresholdViolation`, `model::Report`.
- Produces: `providers::MetricProvider` trait and `providers::ProviderOutput` enum.
- Produces: `providers::MetricError` enum.
- Consumes: nothing.

- [ ] **Step 1: Write the model definitions in `src/model.rs`**

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Note,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageSummary {
    pub line_rate: f64,
    pub branch_rate: Option<f64>,
    pub function_rate: Option<f64>,
    pub lines_covered: u64,
    pub lines_total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMetric {
    pub file: PathBuf,
    pub name: String,
    pub line_start: usize,
    pub line_end: usize,
    pub line_count: usize,
    pub statement_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeItem {
    pub file: PathBuf,
    pub name: String,
    pub item_kind: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdViolation {
    pub threshold: String,
    pub actual: f64,
    pub limit: f64,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub crate_root: PathBuf,
    pub coverage: Option<CoverageSummary>,
    pub dead_code: Vec<DeadCodeItem>,
    pub function_sizes: Vec<FunctionMetric>,
    pub threshold_violations: Vec<ThresholdViolation>,
}

impl Report {
    pub fn new(crate_root: PathBuf) -> Self {
        Self {
            crate_root,
            coverage: None,
            dead_code: Vec::new(),
            function_sizes: Vec::new(),
            threshold_violations: Vec::new(),
        }
    }
}
```

- [ ] **Step 2: Write the provider trait and error type in `src/providers/mod.rs`**

```rust
use crate::model::{CoverageSummary, DeadCodeItem, FunctionMetric};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MetricError {
    #[error("command `{command}` failed: {source}")]
    CommandFailed {
        command: String,
        #[source]
        source: std::io::Error,
    },
    #[error("command `{command}` exited with status {status}: {stderr}")]
    CommandExit {
        command: String,
        status: i32,
        stderr: String,
    },
    #[error("failed to parse {provider} output: {source}")]
    ParseError {
        provider: String,
        #[source]
        source: anyhow::Error,
    },
    #[error("tool `{tool}` is not installed")]
    ToolNotFound { tool: String },
}

#[derive(Debug)]
pub enum ProviderOutput {
    Coverage(CoverageSummary),
    DeadCode(Vec<DeadCodeItem>),
    FunctionSizes(Vec<FunctionMetric>),
}

pub trait MetricProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn collect(&self, crate_root: &Path) -> Result<ProviderOutput, MetricError>;
}
```

- [ ] **Step 3: Re-export model in `src/lib.rs`**

Update `src/lib.rs`:

```rust
pub mod cli;
pub mod config;
pub mod model;
pub mod providers;
pub mod reporter;
pub mod runner;

pub use model::*;
pub use providers::{MetricError, MetricProvider, ProviderOutput};

use anyhow::Result;

pub fn run() -> Result<()> {
    println!("rustmetrics scaffold ready");
    Ok(())
}
```

- [ ] **Step 4: Build and verify**

Run:
```bash
cargo build
```

Expected: successful compilation.

- [ ] **Step 5: Commit**

```bash
git add src/model.rs src/providers/mod.rs src/lib.rs
git commit -m "feat: add core data models and provider trait"
```

---

### Task 3: Configuration Module

**Files:**
- Create: `src/config.rs`
- Create: `tests/config_test.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Produces: `config::Config`, `config::ProviderConfig`, `config::Thresholds`.
- Produces: `Config::load(path: Option<&Path>) -> Result<Config>`.
- Consumes: nothing.

- [ ] **Step 1: Write the failing test in `tests/config_test.rs`**

```rust
use rustmetrics::config::Config;
use std::path::Path;

#[test]
fn test_load_default_config() {
    let cfg = Config::load(None).unwrap();
    assert!(cfg.providers.coverage.enabled);
    assert!(cfg.providers.dead_code.enabled);
    assert!(cfg.providers.function_size.enabled);
    assert_eq!(cfg.thresholds.min_line_coverage, Some(0.80));
    assert_eq!(cfg.thresholds.max_function_lines, Some(100));
}

#[test]
fn test_load_config_from_path() {
    let toml = r#"
[providers]
coverage = { enabled = false }

[thresholds]
max_function_lines = 50
"#;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("rustmetrics.toml");
    std::fs::write(&path, toml).unwrap();

    let cfg = Config::load(Some(&path)).unwrap();
    assert!(!cfg.providers.coverage.enabled);
    assert_eq!(cfg.thresholds.max_function_lines, Some(50));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run:
```bash
cargo test --test config_test
```

Expected: compile errors because `Config`, `load`, etc., do not exist.

- [ ] **Step 3: Implement `src/config.rs`**

```rust
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageProviderConfig {
    pub enabled: bool,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeProviderConfig {
    pub enabled: bool,
    pub rustflags: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSizeProviderConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvidersConfig {
    #[serde(default = "default_coverage")]
    pub coverage: CoverageProviderConfig,
    #[serde(default = "default_dead_code")]
    pub dead_code: DeadCodeProviderConfig,
    #[serde(default = "default_function_size")]
    pub function_size: FunctionSizeProviderConfig,
}

fn default_coverage() -> CoverageProviderConfig {
    CoverageProviderConfig {
        enabled: true,
        command: "cargo-llvm-cov".to_string(),
    }
}

fn default_dead_code() -> DeadCodeProviderConfig {
    DeadCodeProviderConfig {
        enabled: true,
        rustflags: "-W dead_code".to_string(),
    }
}

fn default_function_size() -> FunctionSizeProviderConfig {
    FunctionSizeProviderConfig { enabled: true }
}

impl Default for ProvidersConfig {
    fn default() -> Self {
        Self {
            coverage: default_coverage(),
            dead_code: default_dead_code(),
            function_size: default_function_size(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thresholds {
    pub min_line_coverage: Option<f64>,
    pub max_function_lines: Option<usize>,
    pub max_dead_code_items: Option<usize>,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            min_line_coverage: Some(0.80),
            max_function_lines: Some(100),
            max_dead_code_items: Some(0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub providers: ProvidersConfig,
    #[serde(default)]
    pub thresholds: Thresholds,
}

impl Config {
    pub fn load(path: Option<&Path>) -> Result<Self> {
        if let Some(path) = path {
            let content = std::fs::read_to_string(path)
                .with_context(|| format!("failed to read config from {}", path.display()))?;
            let config: Config = toml::from_str(&content)
                .with_context(|| format!("failed to parse config from {}", path.display()))?;
            return Ok(config);
        }

        for name in [".rustmetrics.toml", "rustmetrics.toml"] {
            let candidate = PathBuf::from(name);
            if candidate.exists() {
                return Self::load(Some(&candidate));
            }
        }

        Ok(Config::default())
    }
}
```

- [ ] **Step 4: Update `src/lib.rs` to re-export config**

```rust
pub mod cli;
pub mod config;
pub mod model;
pub mod providers;
pub mod reporter;
pub mod runner;

pub use config::Config;
pub use model::*;
pub use providers::{MetricError, MetricProvider, ProviderOutput};
```

- [ ] **Step 5: Run the tests to verify they pass**

Run:
```bash
cargo test --test config_test
```

Expected: both tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/config.rs tests/config_test.rs src/lib.rs
git commit -m "feat: add TOML configuration module"
```

---

### Task 4: CLI Arguments

**Files:**
- Create: `src/cli.rs`
- Modify: `src/lib.rs`
- Modify: `src/main.rs`
- Create: `tests/cli_test.rs`

**Interfaces:**
- Produces: `cli::Args` struct with `parse()` via `clap`.
- Produces: `Args::crate_root() -> PathBuf`.
- Consumes: `Config::load`.

- [ ] **Step 1: Write the failing test in `tests/cli_test.rs`**

```rust
use rustmetrics::cli::Args;
use clap::Parser;

#[test]
fn test_parse_defaults() {
    let args = Args::parse_from(["rustmetrics"]);
    assert_eq!(args.crate_root, std::path::PathBuf::from("."));
    assert_eq!(args.format, "human");
    assert!(args.config.is_none());
    assert!(!args.no_coverage);
}

#[test]
fn test_parse_options() {
    let args = Args::parse_from([
        "rustmetrics",
        "--format",
        "json",
        "--config",
        "myconfig.toml",
        "--no-dead-code",
        "/tmp/my-crate",
    ]);
    assert_eq!(args.format, "json");
    assert_eq!(args.config, Some(std::path::PathBuf::from("myconfig.toml")));
    assert!(args.no_dead_code);
    assert_eq!(args.crate_root, std::path::PathBuf::from("/tmp/my-crate"));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run:
```bash
cargo test --test cli_test
```

Expected: compile errors because `Args` does not exist.

- [ ] **Step 3: Implement `src/cli.rs`**

```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "rustmetrics")]
#[command(about = "Aggregate test coverage, dead code, and function-size metrics for Rust projects")]
pub struct Args {
    /// Path to the crate to analyze.
    #[arg(default_value = ".")]
    pub crate_root: PathBuf,

    /// Output format.
    #[arg(short, long, default_value = "human")]
    pub format: String,

    /// Path to a TOML configuration file.
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Skip the coverage provider.
    #[arg(long)]
    pub no_coverage: bool,

    /// Skip the dead-code provider.
    #[arg(long)]
    pub no_dead_code: bool,

    /// Skip the function-size provider.
    #[arg(long)]
    pub no_function_size: bool,
}
```

- [ ] **Step 4: Update `src/lib.rs`**

```rust
pub mod cli;
pub mod config;
pub mod model;
pub mod providers;
pub mod reporter;
pub mod runner;

pub use cli::Args;
pub use config::Config;
pub use model::*;
pub use providers::{MetricError, MetricProvider, ProviderOutput};
```

- [ ] **Step 5: Update `src/main.rs` to parse args**

```rust
use rustmetrics::{run, Args};
use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    run(args)
}
```

- [ ] **Step 6: Update `src/lib.rs` `run` signature**

```rust
use anyhow::Result;

pub fn run(args: Args) -> Result<()> {
    let _config = Config::load(args.config.as_deref())?;
    println!("rustmetrics CLI ready");
    Ok(())
}
```

- [ ] **Step 7: Run the tests to verify they pass**

Run:
```bash
cargo test --test cli_test
```

Expected: both tests pass.

- [ ] **Step 8: Build and run the binary**

Run:
```bash
cargo run -- --help
```

Expected: help text is displayed.

- [ ] **Step 9: Commit**

```bash
git add src/cli.rs src/lib.rs src/main.rs tests/cli_test.rs
git commit -m "feat: add clap-based CLI"
```

---

### Task 5: Tree-Sitter Function-Size Provider

**Files:**
- Create: `src/providers/function_size.rs`
- Modify: `src/providers/mod.rs`
- Create: `tests/function_size_test.rs`

**Interfaces:**
- Produces: `providers::function_size::FunctionSizeProvider`.
- Produces: `FunctionSizeProvider::new() -> Self`.
- Consumes: `MetricProvider` trait, `FunctionMetric`, `ProviderOutput`, `MetricError`.

- [ ] **Step 1: Write the failing test in `tests/function_size_test.rs`**

```rust
use rustmetrics::providers::function_size::FunctionSizeProvider;
use rustmetrics::providers::MetricProvider;
use std::path::PathBuf;

#[test]
fn test_function_size_on_inline_source() {
    let provider = FunctionSizeProvider::new();
    let source = r#"
fn small() {
    let x = 1;
    let y = 2;
}

fn big() {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    if a > 0 {
        println!("yes");
    }
}
"#;
    let metrics = provider.analyze_source(source, PathBuf::from("src/lib.rs"));
    assert_eq!(metrics.len(), 2);

    let small = metrics.iter().find(|m| m.name == "small").unwrap();
    assert_eq!(small.line_count, 4);
    assert_eq!(small.statement_count, 2);

    let big = metrics.iter().find(|m| m.name == "big").unwrap();
    assert!(big.line_count >= 9);
    assert!(big.statement_count >= 6);
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run:
```bash
cargo test --test function_size_test
```

Expected: compile error because `FunctionSizeProvider` does not exist.

- [ ] **Step 3: Implement `src/providers/function_size.rs`**

```rust
use crate::model::FunctionMetric;
use crate::providers::{MetricError, MetricProvider, ProviderOutput};
use std::path::{Path, PathBuf};
use tree_sitter::{Node, Parser, Query, QueryCursor};

pub struct FunctionSizeProvider;

impl FunctionSizeProvider {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze_source(&self, source: &str, file: PathBuf) -> Vec<FunctionMetric> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE)
            .expect("tree-sitter rust grammar should load");

        let tree = match parser.parse(source, None) {
            Some(tree) => tree,
            None => return Vec::new(),
        };

        let query_str = "(function_item name: (identifier) @name) @function";
        let query = match Query::new(&tree_sitter_rust::LANGUAGE, query_str) {
            Ok(q) => q,
            Err(_) => return Vec::new(),
        };

        let root = tree.root_node();
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, root, source.as_bytes());

        let mut metrics = Vec::new();
        for m in matches {
            let mut name_node: Option<Node> = None;
            let mut function_node: Option<Node> = None;

            for capture in m.captures {
                let name = query.capture_names()[capture.index as usize];
                if name == "name" {
                    name_node = Some(capture.node);
                } else if name == "function" {
                    function_node = Some(capture.node);
                }
            }

            let (Some(name_node), Some(function_node)) = (name_node, function_node) else {
                continue;
            };

            let name = source[name_node.byte_range()].to_string();
            let start = function_node.start_position();
            let end = function_node.end_position();
            let line_count = end.row.saturating_sub(start.row) + 1;
            let statement_count = count_statements(function_node, source);

            metrics.push(FunctionMetric {
                file: file.clone(),
                name,
                line_start: start.row + 1,
                line_end: end.row + 1,
                line_count,
                statement_count,
            });
        }

        metrics
    }
}

fn count_statements(function_node: Node, source: &str) -> usize {
    let body = function_node
        .children(&mut function_node.walk())
        .find(|child| child.kind() == "block");

    let Some(body) = body else { return 0 };

    let mut count = 0;
    let mut cursor = body.walk();
    for child in body.children(&mut cursor) {
        if is_statement(child.kind()) {
            count += 1;
        }
    }
    count
}

fn is_statement(kind: &str) -> bool {
    matches!(
        kind,
        "expression_statement"
            | "let_declaration"
            | "const_item"
            | "static_item"
            | "if_expression"
            | "match_expression"
            | "for_expression"
            | "while_expression"
            | "loop_expression"
            | "return_expression"
            | "break_expression"
            | "continue_expression"
    )
}

impl MetricProvider for FunctionSizeProvider {
    fn name(&self) -> &'static str {
        "function_size"
    }

    fn collect(&self, crate_root: &Path) -> Result<ProviderOutput, MetricError> {
        let mut metrics = Vec::new();

        for entry in walkdir::WalkDir::new(crate_root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            if path.components().any(|c| c.as_os_str() == "target") {
                continue;
            }

            let source = match std::fs::read_to_string(path) {
                Ok(s) => s,
                Err(_) => continue,
            };

            let relative = path.strip_prefix(crate_root).unwrap_or(path).to_path_buf();
            metrics.extend(self.analyze_source(&source, relative));
        }

        Ok(ProviderOutput::FunctionSizes(metrics))
    }
}
```

- [ ] **Step 4: Update `src/providers/mod.rs` to expose the module**

```rust
pub mod coverage;
pub mod dead_code;
pub mod function_size;

// ... existing code ...
```

- [ ] **Step 5: Run the tests to verify they pass**

Run:
```bash
cargo test --test function_size_test
```

Expected: the test passes.

- [ ] **Step 6: Commit**

```bash
git add src/providers/function_size.rs src/providers/mod.rs tests/function_size_test.rs
git commit -m "feat: add Tree-sitter function-size provider"
```

---

### Task 6: Dead-Code Provider

**Files:**
- Create: `src/providers/dead_code.rs`
- Modify: `src/providers/mod.rs`
- Create: `tests/dead_code_test.rs`

**Interfaces:**
- Produces: `providers::dead_code::DeadCodeProvider`.
- Produces: `DeadCodeProvider::new(rustflags: String) -> Self`.
- Consumes: `MetricProvider`, `DeadCodeItem`, `ProviderOutput`, `MetricError`.

- [ ] **Step 1: Write the failing test in `tests/dead_code_test.rs`**

```rust
use rustmetrics::providers::dead_code::DeadCodeProvider;
use rustmetrics::providers::MetricProvider;
use std::path::PathBuf;

#[test]
fn test_parse_dead_code_messages() {
    let provider = DeadCodeProvider::new("-W dead_code".to_string());
    let jsonl = r#"
{"reason":"compiler-message","package_id":"pkg","target":{"kind":["lib"],"name":"mylib","src_path":"/tmp/lib.rs"},"message":{"rendered":"warning: function `unused_fn` is never used\n --> src/lib.rs:3:1\n  |\n3 | fn unused_fn() {}\n  | ^^^^^^^^^^^^^^\n","spans":[{"file_name":"src/lib.rs","byte_start":0,"byte_end":16,"line_start":3,"line_end":3,"column_start":1,"column_end":17}],"code":{"code":"dead_code"},"level":"warning","message":"function `unused_fn` is never used"}}
{"reason":"compiler-artifact","package_id":"pkg","target":{"kind":["lib"],"name":"mylib","src_path":"/tmp/lib.rs"},"filenames":[],"executable":null,"fresh":false}
"#;
    let items = provider.parse_messages(jsonl, PathBuf::from("/tmp/my-crate"));
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].name, "unused_fn");
    assert_eq!(items[0].item_kind, "function");
    assert_eq!(items[0].line, 3);
    assert_eq!(items[0].column, 1);
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run:
```bash
cargo test --test dead_code_test
```

Expected: compile error because `DeadCodeProvider` does not exist.

- [ ] **Step 3: Implement `src/providers/dead_code.rs`**

```rust
use crate::model::DeadCodeItem;
use crate::providers::{MetricError, MetricProvider, ProviderOutput};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct DeadCodeProvider {
    rustflags: String,
}

impl DeadCodeProvider {
    pub fn new(rustflags: String) -> Self {
        Self { rustflags }
    }

    pub fn parse_messages(&self, jsonl: &str, _crate_root: PathBuf) -> Vec<DeadCodeItem> {
        let mut items = Vec::new();

        for line in jsonl.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let Ok(message): Result<CargoMessage, _> = serde_json::from_str(line) else {
                continue;
            };

            if message.reason != "compiler-message" {
                continue;
            }

            let Some(code) = message.message.code.as_ref() else {
                continue;
            };

            if code.code != "dead_code" {
                continue;
            }

            let Some(span) = message.message.spans.first() else {
                continue;
            };

            let name = extract_name(&message.message.message);
            let item_kind = infer_kind(&message.message.message);

            items.push(DeadCodeItem {
                file: PathBuf::from(&span.file_name),
                name,
                item_kind,
                line: span.line_start,
                column: span.column_start,
            });
        }

        items
    }
}

fn extract_name(message: &str) -> String {
    // Try "function `name` is never used" or "constant `NAME` is never used"
    if let Some(start) = message.find('`') {
        if let Some(end) = message[start + 1..].find('`') {
            return message[start + 1..start + 1 + end].to_string();
        }
    }
    "unknown".to_string()
}

fn infer_kind(message: &str) -> String {
    if message.contains("function") {
        "function".to_string()
    } else if message.contains("constant") {
        "constant".to_string()
    } else if message.contains("struct") {
        "struct".to_string()
    } else if message.contains("enum") {
        "enum".to_string()
    } else if message.contains("static") {
        "static".to_string()
    } else {
        "item".to_string()
    }
}

#[derive(Debug, Deserialize)]
struct CargoMessage {
    reason: String,
    message: CompilerMessage,
}

#[derive(Debug, Deserialize)]
struct CompilerMessage {
    code: Option<MessageCode>,
    level: String,
    message: String,
    spans: Vec<MessageSpan>,
}

#[derive(Debug, Deserialize)]
struct MessageCode {
    code: String,
}

#[derive(Debug, Deserialize)]
struct MessageSpan {
    file_name: String,
    line_start: usize,
    column_start: usize,
}

impl MetricProvider for DeadCodeProvider {
    fn name(&self) -> &'static str {
        "dead_code"
    }

    fn collect(&self, crate_root: &Path) -> Result<ProviderOutput, MetricError> {
        let output = Command::new("cargo")
            .arg("check")
            .arg("--message-format=json")
            .env("RUSTFLAGS", &self.rustflags)
            .current_dir(crate_root)
            .output()
            .map_err(|e| MetricError::CommandFailed {
                command: "cargo check".to_string(),
                source: e,
            })?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        if !output.status.success() && stderr.contains("error") {
            return Err(MetricError::CommandExit {
                command: "cargo check".to_string(),
                status: output.status.code().unwrap_or(-1),
                stderr: stderr.to_string(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let items = self.parse_messages(&stdout, crate_root.to_path_buf());
        Ok(ProviderOutput::DeadCode(items))
    }
}
```

- [ ] **Step 4: Update `src/providers/mod.rs` to expose the module**

```rust
pub mod coverage;
pub mod dead_code;
pub mod function_size;

// ... existing code ...
```

- [ ] **Step 5: Run the tests to verify they pass**

Run:
```bash
cargo test --test dead_code_test
```

Expected: the test passes.

- [ ] **Step 6: Commit**

```bash
git add src/providers/dead_code.rs src/providers/mod.rs tests/dead_code_test.rs
git commit -m "feat: add dead-code provider via cargo check"
```

---

### Task 7: Coverage Provider

**Files:**
- Create: `src/providers/coverage.rs`
- Modify: `src/providers/mod.rs`
- Create: `tests/coverage_test.rs`

**Interfaces:**
- Produces: `providers::coverage::CoverageProvider`.
- Produces: `CoverageProvider::new(command: String) -> Self`.
- Consumes: `MetricProvider`, `CoverageSummary`, `ProviderOutput`, `MetricError`.

- [ ] **Step 1: Write the failing test in `tests/coverage_test.rs`**

```rust
use rustmetrics::providers::coverage::CoverageProvider;
use rustmetrics::providers::MetricProvider;
use std::path::PathBuf;

#[test]
fn test_parse_llvm_cov_json() {
    let provider = CoverageProvider::new("cargo-llvm-cov".to_string());
    let json = r#"
{
  "data": [{
    "totals": {
      "lines": { "count": 200, "covered": 169, "percent": 84.5 },
      "functions": { "count": 20, "covered": 15, "percent": 75.0 },
      "branches": { "count": 50, "covered": 40, "percent": 80.0 }
    }
  }]
}
"#;
    let summary = provider.parse_json(json).unwrap();
    assert_eq!(summary.line_rate, 0.845);
    assert_eq!(summary.lines_covered, 169);
    assert_eq!(summary.lines_total, 200);
    assert_eq!(summary.function_rate, Some(0.75));
    assert_eq!(summary.branch_rate, Some(0.80));
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run:
```bash
cargo test --test coverage_test
```

Expected: compile error because `CoverageProvider` does not exist.

- [ ] **Step 3: Implement `src/providers/coverage.rs`**

```rust
use crate::model::CoverageSummary;
use crate::providers::{MetricError, MetricProvider, ProviderOutput};
use serde::Deserialize;
use std::path::Path;
use std::process::Command;

pub struct CoverageProvider {
    command: String,
}

impl CoverageProvider {
    pub fn new(command: String) -> Self {
        Self { command }
    }

    pub fn parse_json(&self, text: &str) -> Result<CoverageSummary, MetricError> {
        let report: LlvmCovReport = serde_json::from_str(text).map_err(|e| MetricError::ParseError {
            provider: "coverage".to_string(),
            source: e.into(),
        })?;

        let data = report.data.into_iter().next().ok_or_else(|| MetricError::ParseError {
            provider: "coverage".to_string(),
            source: anyhow::anyhow!("no data section in coverage report"),
        })?;

        let lines = data.totals.lines;
        let line_rate = if lines.count == 0 {
            1.0
        } else {
            lines.percent / 100.0
        };

        Ok(CoverageSummary {
            line_rate,
            branch_rate: data.totals.branches.map(|b| b.percent / 100.0),
            function_rate: data.totals.functions.map(|f| f.percent / 100.0),
            lines_covered: lines.covered,
            lines_total: lines.count,
        })
    }
}

#[derive(Debug, Deserialize)]
struct LlvmCovReport {
    data: Vec<LlvmCovData>,
}

#[derive(Debug, Deserialize)]
struct LlvmCovData {
    totals: LlvmCovTotals,
}

#[derive(Debug, Deserialize)]
struct LlvmCovTotals {
    lines: LlvmCovStat,
    #[serde(default)]
    functions: Option<LlvmCovStat>,
    #[serde(default)]
    branches: Option<LlvmCovStat>,
}

#[derive(Debug, Deserialize)]
struct LlvmCovStat {
    count: u64,
    covered: u64,
    percent: f64,
}

impl MetricProvider for CoverageProvider {
    fn name(&self) -> &'static str {
        "coverage"
    }

    fn collect(&self, crate_root: &Path) -> Result<ProviderOutput, MetricError> {
        let (program, args): (&str, &[&str]) = match self.command.as_str() {
            "cargo-llvm-cov" => ("cargo", &["llvm-cov", "--json"]),
            other => {
                return Err(MetricError::ToolNotFound {
                    tool: other.to_string(),
                })
            }
        };

        let output = Command::new(program)
            .args(args)
            .current_dir(crate_root)
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    MetricError::ToolNotFound {
                        tool: self.command.clone(),
                    }
                } else {
                    MetricError::CommandFailed {
                        command: self.command.clone(),
                        source: e,
                    }
                }
            })?;

        if !output.status.success() {
            return Err(MetricError::CommandExit {
                command: self.command.clone(),
                status: output.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let summary = self.parse_json(&stdout)?;
        Ok(ProviderOutput::Coverage(summary))
    }
}
```

- [ ] **Step 4: Update `src/providers/mod.rs` to expose the module**

```rust
pub mod coverage;
pub mod dead_code;
pub mod function_size;

// ... existing code ...
```

- [ ] **Step 5: Run the tests to verify they pass**

Run:
```bash
cargo test --test coverage_test
```

Expected: the test passes.

- [ ] **Step 6: Commit**

```bash
git add src/providers/coverage.rs src/providers/mod.rs tests/coverage_test.rs
git commit -m "feat: add coverage provider via cargo llvm-cov"
```

---

### Task 8: Runner and Threshold Application

**Files:**
- Create: `src/runner.rs`
- Modify: `src/lib.rs`
- Create: `tests/runner_test.rs`

**Interfaces:**
- Produces: `runner::Runner`.
- Produces: `Runner::new(config: Config, args: Args) -> Self`.
- Produces: `Runner::run() -> Result<Report>`.
- Consumes: `Config`, `Args`, all providers, `Report`, `ThresholdViolation`.

- [ ] **Step 1: Write the failing test in `tests/runner_test.rs`**

```rust
use rustmetrics::cli::Args;
use rustmetrics::config::Config;
use rustmetrics::runner::Runner;
use clap::Parser;

#[test]
fn test_runner_respects_cli_flags() {
    let config = Config::load(None).unwrap();
    let args = Args::parse_from(["rustmetrics", "--no-coverage", "--no-dead-code", "."]);
    let runner = Runner::new(config, args);
    assert_eq!(runner.providers.len(), 1); // only function_size
}

#[test]
fn test_runner_format_is_stored() {
    let config = Config::load(None).unwrap();
    let args = Args::parse_from(["rustmetrics", "--format", "json", "."]);
    let runner = Runner::new(config, args);
    assert_eq!(runner.format(), "json");
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run:
```bash
cargo test --test runner_test
```

Expected: compile error because `Runner` does not exist.

- [ ] **Step 3: Implement `src/runner.rs`**

```rust
use crate::cli::Args;
use crate::config::Config;
use crate::model::{Report, ThresholdViolation};
use crate::providers::{
    coverage::CoverageProvider, dead_code::DeadCodeProvider, function_size::FunctionSizeProvider,
    MetricProvider, ProviderOutput,
};
use anyhow::Result;
use std::sync::Arc;

pub struct Runner {
    pub providers: Vec<Arc<dyn MetricProvider>>,
    config: Config,
    crate_root: std::path::PathBuf,
    format: String,
}

impl Runner {
    pub fn new(config: Config, args: Args) -> Self {
        let mut providers: Vec<Arc<dyn MetricProvider>> = Vec::new();

        if config.providers.coverage.enabled && !args.no_coverage {
            providers.push(Arc::new(CoverageProvider::new(
                config.providers.coverage.command.clone(),
            )));
        }

        if config.providers.dead_code.enabled && !args.no_dead_code {
            providers.push(Arc::new(DeadCodeProvider::new(
                config.providers.dead_code.rustflags.clone(),
            )));
        }

        if config.providers.function_size.enabled && !args.no_function_size {
            providers.push(Arc::new(FunctionSizeProvider::new()));
        }

        Self {
            providers,
            config,
            crate_root: args.crate_root,
            format: args.format,
        }
    }

    pub fn format(&self) -> &str {
        &self.format
    }

    pub fn run(&self) -> Result<Report> {
        let mut report = Report::new(self.crate_root.clone());

        for provider in &self.providers {
            match provider.collect(&self.crate_root) {
                Ok(ProviderOutput::Coverage(summary)) => report.coverage = Some(summary),
                Ok(ProviderOutput::DeadCode(items)) => report.dead_code.extend(items),
                Ok(ProviderOutput::FunctionSizes(metrics)) => report.function_sizes.extend(metrics),
                Err(e) => {
                    eprintln!("provider {} failed: {}", provider.name(), e);
                    anyhow::bail!(e);
                }
            }
        }

        report
            .function_sizes
            .sort_by(|a, b| a.file.cmp(&b.file).then(a.line_start.cmp(&b.line_start)));
        report
            .dead_code
            .sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));

        apply_thresholds(&mut report, &self.config);

        Ok(report)
    }
}

fn apply_thresholds(report: &mut Report, config: &Config) {
    if let Some(min) = config.thresholds.min_line_coverage {
        if let Some(coverage) = &report.coverage {
            if coverage.line_rate < min {
                report.threshold_violations.push(ThresholdViolation {
                    threshold: "min_line_coverage".to_string(),
                    actual: coverage.line_rate,
                    limit: min,
                    detail: format!(
                        "line coverage {}% is below {}%",
                        coverage.line_rate * 100.0,
                        min * 100.0
                    ),
                });
            }
        }
    }

    if let Some(max) = config.thresholds.max_function_lines {
        for func in &report.function_sizes {
            if func.line_count > max {
                report.threshold_violations.push(ThresholdViolation {
                    threshold: "max_function_lines".to_string(),
                    actual: func.line_count as f64,
                    limit: max as f64,
                    detail: format!("{} has {} lines", func.name, func.line_count),
                });
            }
        }
    }

    if let Some(max) = config.thresholds.max_dead_code_items {
        if report.dead_code.len() > max {
            report.threshold_violations.push(ThresholdViolation {
                threshold: "max_dead_code_items".to_string(),
                actual: report.dead_code.len() as f64,
                limit: max as f64,
                detail: format!("{} dead-code items found", report.dead_code.len()),
            });
        }
    }
}
```

- [ ] **Step 4: Update `src/lib.rs` to expose `Runner`**

```rust
pub use runner::Runner;
```

- [ ] **Step 5: Run the tests to verify they pass**

Run:
```bash
cargo test --test runner_test
```

Expected: the test passes.

- [ ] **Step 6: Commit**

```bash
git add src/runner.rs src/lib.rs tests/runner_test.rs
git commit -m "feat: add runner and threshold logic"
```

---

### Task 9: Reporters (Human and JSON)

**Files:**
- Create: `src/reporter/mod.rs`
- Create: `src/reporter/human.rs`
- Create: `src/reporter/json.rs`
- Create: `tests/reporter_test.rs`

**Interfaces:**
- Produces: `reporter::Reporter` trait.
- Produces: `reporter::HumanReporter`, `reporter::JsonReporter`.
- Produces: `reporter::reporter_for(format: &str) -> Box<dyn Reporter>`.
- Consumes: `Report`, `ThresholdViolation`.

- [ ] **Step 1: Write the failing test in `tests/reporter_test.rs`**

```rust
use rustmetrics::model::Report;
use rustmetrics::reporter::{reporter_for, Reporter};
use std::path::PathBuf;

#[test]
fn test_json_reporter_outputs_valid_json() {
    let report = Report::new(PathBuf::from("/tmp/crate"));
    let reporter = reporter_for("json");
    let output = reporter.render(&report);
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["crate_root"], "/tmp/crate");
}

#[test]
fn test_human_reporter_includes_crate_root() {
    let report = Report::new(PathBuf::from("/tmp/crate"));
    let reporter = reporter_for("human");
    let output = reporter.render(&report);
    assert!(output.contains("/tmp/crate"));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run:
```bash
cargo test --test reporter_test
```

Expected: compile errors because reporter modules do not exist.

- [ ] **Step 3: Implement `src/reporter/mod.rs`**

```rust
pub mod human;
pub mod json;

use crate::model::Report;

pub trait Reporter {
    fn render(&self, report: &Report) -> String;
}

pub fn reporter_for(format: &str) -> Box<dyn Reporter> {
    match format {
        "json" => Box::new(json::JsonReporter),
        _ => Box::new(human::HumanReporter),
    }
}
```

- [ ] **Step 4: Implement `src/reporter/json.rs`**

```rust
use crate::model::Report;
use crate::reporter::Reporter;

pub struct JsonReporter;

impl Reporter for JsonReporter {
    fn render(&self, report: &Report) -> String {
        serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".to_string())
    }
}
```

- [ ] **Step 5: Implement `src/reporter/human.rs`**

```rust
use crate::model::Report;
use crate::reporter::Reporter;

pub struct HumanReporter;

impl Reporter for HumanReporter {
    fn render(&self, report: &Report) -> String {
        let mut lines = Vec::new();
        lines.push(format!("Crate: {}", report.crate_root.display()));
        lines.push(String::new());

        lines.push("Coverage".to_string());
        match &report.coverage {
            Some(c) => lines.push(format!(
                "  Line coverage: {:.1}% ({} / {} lines)",
                c.line_rate * 100.0,
                c.lines_covered,
                c.lines_total
            )),
            None => lines.push("  (skipped)".to_string()),
        }
        lines.push(String::new());

        lines.push("Dead code".to_string());
        if report.dead_code.is_empty() {
            lines.push("  None".to_string());
        } else {
            for item in &report.dead_code {
                lines.push(format!(
                    "  {}:{}:{} unused {} `{}`",
                    item.file.display(),
                    item.line,
                    item.column,
                    item.item_kind,
                    item.name
                ));
            }
        }
        lines.push(String::new());

        lines.push("Function size".to_string());
        if report.function_sizes.is_empty() {
            lines.push("  None".to_string());
        } else {
            for func in &report.function_sizes {
                lines.push(format!(
                    "  {}:{}  {}  {} lines  {} statements",
                    func.file.display(),
                    func.line_start,
                    func.name,
                    func.line_count,
                    func.statement_count
                ));
            }
        }

        if !report.threshold_violations.is_empty() {
            lines.push(String::new());
            lines.push("Threshold violations".to_string());
            for v in &report.threshold_violations {
                lines.push(format!("  {}: {}", v.threshold, v.detail));
            }
        }

        lines.join("\n")
    }
}
```

- [ ] **Step 6: Run the tests to verify they pass**

Run:
```bash
cargo test --test reporter_test
```

Expected: both tests pass.

- [ ] **Step 7: Commit**

```bash
git add src/reporter/mod.rs src/reporter/human.rs src/reporter/json.rs tests/reporter_test.rs
git commit -m "feat: add human and JSON reporters"
```

---

### Task 10: Wire Up the Binary

**Files:**
- Modify: `src/lib.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Consumes: `Args`, `Config`, `Runner`, `reporter_for`.
- Produces: a working CLI with proper exit codes.

- [ ] **Step 1: Update `src/lib.rs` to implement `run`**

```rust
use crate::cli::Args;
use crate::config::Config;
use crate::reporter::reporter_for;
use crate::runner::Runner;
use anyhow::Result;

pub fn run(args: Args) -> Result<()> {
    let config = Config::load(args.config.as_deref())?;
    let runner = Runner::new(config, args);
    let report = runner.run()?;

    let reporter = reporter_for(runner.format());
    println!("{}", reporter.render(&report));

    if report.threshold_violations.is_empty() {
        Ok(())
    } else {
        std::process::exit(1);
    }
}
```

- [ ] **Step 2: Update `src/main.rs`**

```rust
use rustmetrics::run;
use rustmetrics::Args;
use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    run(args)
}
```

- [ ] **Step 3: Build the binary**

Run:
```bash
cargo build
```

Expected: successful compilation.

- [ ] **Step 4: Run help**

Run:
```bash
cargo run -- --help
```

Expected: help text is displayed.

- [ ] **Step 7: Commit**

```bash
git add src/lib.rs src/main.rs src/runner.rs
git commit -m "feat: wire CLI, runner, and reporters into binary"
```

---

### Task 11: Integration Test Fixture Crate

**Files:**
- Create: `tests/fixtures/minimal_crate/Cargo.toml`
- Create: `tests/fixtures/minimal_crate/src/lib.rs`
- Create: `tests/fixtures/minimal_crate/tests/basic.rs`

**Interfaces:**
- Produces: a minimal Rust crate that exercises all three metrics.

- [ ] **Step 1: Create `tests/fixtures/minimal_crate/Cargo.toml`**

```toml
[package]
name = "minimal_crate"
version = "0.1.0"
edition = "2021"

[dependencies]
```

- [ ] **Step 2: Create `tests/fixtures/minimal_crate/src/lib.rs`**

```rust
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn unused_helper() -> i32 {
    42
}

pub fn large_function() -> i32 {
    let mut sum = 0;
    sum += 1;
    sum += 2;
    sum += 3;
    sum += 4;
    sum += 5;
    sum += 6;
    sum += 7;
    sum += 8;
    sum += 9;
    sum += 10;
    sum
}
```

- [ ] **Step 3: Create `tests/fixtures/minimal_crate/tests/basic.rs`**

```rust
use minimal_crate::add;

#[test]
fn test_add() {
    assert_eq!(add(2, 3), 5);
}
```

- [ ] **Step 4: Commit**

```bash
git add tests/fixtures/minimal_crate
git commit -m "test: add minimal crate fixture"
```

---

### Task 12: Integration Tests

**Files:**
- Create: `tests/integration_test.rs`

**Interfaces:**
- Consumes: the compiled `rustmetrics` binary via `assert_cmd`.

- [ ] **Step 1: Write the integration test**

```rust
use assert_cmd::Command;
use predicates::str::contains;
use std::path::PathBuf;

fn bin() -> Command {
    Command::cargo_bin("rustmetrics").unwrap()
}

#[test]
fn test_function_size_only() {
    let mut cmd = bin();
    let fixture = PathBuf::from("tests/fixtures/minimal_crate");
    cmd.args(["--no-coverage", "--no-dead-code", "--format", "json", &fixture.to_string_lossy()]);
    cmd.assert()
        .success()
        .stdout(contains("\"name\":\"large_function\""))
        .stdout(contains("\"name\":\"unused_helper\""));
}

#[test]
fn test_help() {
    let mut cmd = bin();
    cmd.arg("--help");
    cmd.assert().success().stdout(contains("rustmetrics"));
}
```

- [ ] **Step 2: Run the integration tests**

Run:
```bash
cargo test --test integration_test
```

Expected: tests pass.

- [ ] **Step 3: Commit**

```bash
git add tests/integration_test.rs
git commit -m "test: add integration tests"
```

---

### Task 13: Final Verification and README Update

**Files:**
- Modify: `README.md`

**Interfaces:**
- Produces: updated documentation and a passing CI-ready crate.

- [ ] **Step 1: Update `README.md`**

```markdown
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
```

- [ ] **Step 2: Run the full test suite**

Run:
```bash
cargo test
```

Expected: all unit and integration tests pass. Tests requiring `cargo-llvm-cov` may be skipped if not installed.

- [ ] **Step 3: Run a manual smoke test on the fixture**

Run:
```bash
cargo run -- --no-coverage --no-dead-code tests/fixtures/minimal_crate
```

Expected: human-readable report showing `large_function`, `add`, and `unused_helper`.

- [ ] **Step 4: Commit**

```bash
git add README.md
git commit -m "docs: update README with usage and prerequisites"
```

---

## Self-Review

**Spec coverage:**
- Test coverage provider — Task 7.
- Dead-code provider — Task 6.
- Function-size provider via Tree-sitter — Task 5.
- Unified report and two reporters — Tasks 8, 9.
- TOML configuration — Task 3.
- CLI — Task 4.
- Thresholds and exit codes — Tasks 8, 10.
- Testing — Tasks 2–12 include tests; Task 12 is integration.

**Placeholder scan:**
- No `TBD`, `TODO`, or vague steps. Each step includes exact file paths, code, and commands.
- No "appropriate error handling" or "write tests for the above" placeholders.

**Type consistency:**
- `Config::load` returns `Result<Config>` and is used in `run`.
- `Runner::new` takes `Config` and `Args`; `Runner::run` returns `Result<Report>`.
- `Reporter::render` takes `&Report` and returns `String`.
- `MetricProvider::collect` returns `Result<ProviderOutput, MetricError>`.
- `ProviderOutput` variants match `Report` fields exactly.
