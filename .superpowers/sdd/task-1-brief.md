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

