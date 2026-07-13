# Project Hardening, CI, and Examples Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add CI, open-source hygiene, runnable examples, and agent-facing documentation to rustmetrics.

**Architecture:** Standard GitHub Actions workflow for Rust; markdown documentation and TOML examples committed to the repository; SDD scratch files removed from tracking.

**Tech Stack:** GitHub Actions, Markdown, TOML, Rust toolchain.

## Global Constraints

- Rust version: `cargo 1.96.1` (available in environment).
- Project root: `/root/rustmetrics`.
- All existing tests, formatting, and clippy checks must remain clean.
- License in `Cargo.toml` is `MIT OR Apache-2.0`.
- SDD scratch files must not be published.

---

### Task 1: Remove SDD Scratch Files and Update .gitignore

**Files:**
- Delete: `.superpowers/sdd/*`
- Modify: `.gitignore`

**Interfaces:**
- Produces: clean repository with ignored scratch directory.

- [ ] **Step 1: Inspect current .gitignore**

Current `.gitignore`:
```
/target
```

- [ ] **Step 2: Update .gitignore**

Replace `.gitignore` contents with:
```
/target
Cargo.lock
.superpowers/
tests/fixtures/*/target/
tests/fixtures/*/Cargo.lock
```

- [ ] **Step 3: Remove SDD scratch files**

Run:
```bash
rm -rf .superpowers/sdd
```

- [ ] **Step 4: Verify only intended files remain**

Run:
```bash
git status --short
```

Expected: deleted `.superpowers/sdd/...` files and modified `.gitignore`.

- [ ] **Step 5: Commit**

```bash
git add .gitignore
git commit -m "chore: remove SDD scratch files and update .gitignore"
```

---

### Task 2: Add LICENSE and CODE_OF_CONDUCT

**Files:**
- Create: `LICENSE`
- Create: `CODE_OF_CONDUCT.md`

**Interfaces:**
- Produces: dual-license file and code of conduct.

- [ ] **Step 1: Create LICENSE**

Write `LICENSE`:
```
MIT OR Apache-2.0

This project is dual-licensed under the MIT license and the Apache License, Version 2.0.
You may use, modify, and distribute this software under either license.

---

MIT License

Copyright (c) 2026 rustmetrics contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

---

Apache License, Version 2.0

Copyright 2026 rustmetrics contributors

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```

- [ ] **Step 2: Create CODE_OF_CONDUCT.md**

Write `CODE_OF_CONDUCT.md`:
```markdown
# Code of Conduct

## Our Pledge

We pledge to make participation in our project a harassment-free experience for everyone, regardless of age, body size, visible or invisible disability, ethnicity, sex characteristics, gender identity and expression, level of experience, education, socio-economic status, nationality, personal appearance, race, religion, or sexual identity and orientation.

## Our Standards

Examples of behavior that contributes to a positive environment:
- Using welcoming and inclusive language
- Being respectful of differing viewpoints and experiences
- Gracefully accepting constructive criticism
- Focusing on what is best for the community

Examples of unacceptable behavior:
- Trolling, insulting/derogatory comments, and personal or political attacks
- Public or private harassment
- Publishing others' private information without permission

## Enforcement

Instances of abusive, harassing, or otherwise unacceptable behavior may be reported to the project maintainers. All complaints will be reviewed and investigated promptly and fairly.
```

- [ ] **Step 3: Commit**

```bash
git add LICENSE CODE_OF_CONDUCT.md
git commit -m "docs: add LICENSE and CODE_OF_CONDUCT"
```

---

### Task 3: Add CONTRIBUTING.md and Issue Templates

**Files:**
- Create: `CONTRIBUTING.md`
- Create: `.github/ISSUE_TEMPLATE/bug_report.md`
- Create: `.github/ISSUE_TEMPLATE/feature_request.md`

**Interfaces:**
- Produces: contribution guidelines and issue templates.

- [ ] **Step 1: Create CONTRIBUTING.md**

Write `CONTRIBUTING.md`:
```markdown
# Contributing to rustmetrics

Thank you for your interest in contributing!

## Getting Started

```bash
git clone https://github.com/kevincouton/rustmetrics.git
cd rustmetrics
cargo test
```

## Development Workflow

1. Create a feature branch: `git checkout -b feature/my-change`
2. Make your changes
3. Ensure checks pass:
   ```bash
   cargo fmt --check
   cargo clippy -- -D warnings
   cargo test
   ```
4. Commit with a descriptive message
5. Open a pull request against `main`

## Code Style

- Run `cargo fmt` before committing
- Keep `cargo clippy -- -D warnings` clean
- Write tests for new behavior
- Update documentation and examples as needed

## Reporting Issues

Use the issue templates for bug reports and feature requests.
```

- [ ] **Step 2: Create bug report template**

Write `.github/ISSUE_TEMPLATE/bug_report.md`:
```markdown
---
name: Bug report
about: Report a problem with rustmetrics
title: ''
labels: bug
assignees: ''
---

## Description

A clear description of the bug.

## Steps to Reproduce

1. Run `rustmetrics ...`
2. Observe ...

## Expected Behavior

What you expected to happen.

## Actual Behavior

What actually happened, including output or error messages.

## Environment

- rustmetrics version:
- Rust version:
- OS:
```

- [ ] **Step 3: Create feature request template**

Write `.github/ISSUE_TEMPLATE/feature_request.md`:
```markdown
---
name: Feature request
about: Suggest an idea for rustmetrics
title: ''
labels: enhancement
assignees: ''
---

## Description

What problem are you trying to solve?

## Proposed Solution

How should rustmetrics address it?

## Alternatives

What other approaches have you considered?

## Additional Context

Any other context or examples.
```

- [ ] **Step 4: Commit**

```bash
git add CONTRIBUTING.md .github/ISSUE_TEMPLATE/
git commit -m "docs: add CONTRIBUTING guide and issue templates"
```

---

### Task 4: Add GitHub Actions CI Workflow

**Files:**
- Create: `.github/workflows/ci.yml`

**Interfaces:**
- Produces: CI workflow that runs on push/PR to main.

- [ ] **Step 1: Create workflow directory and file**

Write `.github/workflows/ci.yml`:
```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Check formatting
        run: cargo fmt --check

      - name: Run clippy
        run: cargo clippy -- -D warnings

      - name: Run tests
        run: cargo test

      - name: Build release
        run: cargo build --release
```

- [ ] **Step 2: Validate workflow syntax**

No direct validation tool is required; inspect indentation. Optional: if `actionlint` is available, run it.

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: add GitHub Actions workflow"
```

---

### Task 5: Add Example Configurations

**Files:**
- Create: `examples/configs/strict.toml`
- Create: `examples/configs/relaxed.toml`
- Create: `examples/configs/function_size_only.toml`

**Interfaces:**
- Produces: example TOML configuration files.

- [ ] **Step 1: Create examples directory and strict config**

Write `examples/configs/strict.toml`:
```toml
[providers]
coverage = { enabled = true, command = "cargo-llvm-cov" }
dead_code = { enabled = true, rustflags = "-W dead_code" }
function_size = { enabled = true }

[thresholds]
min_line_coverage = 0.90
max_function_lines = 50
max_dead_code_items = 0
```

- [ ] **Step 2: Create relaxed config**

Write `examples/configs/relaxed.toml`:
```toml
[providers]
coverage = { enabled = true, command = "cargo-llvm-cov" }
dead_code = { enabled = true, rustflags = "-W dead_code" }
function_size = { enabled = true }

[thresholds]
min_line_coverage = 0.60
max_function_lines = 200
max_dead_code_items = 10
```

- [ ] **Step 3: Create function-size-only config**

Write `examples/configs/function_size_only.toml`:
```toml
[providers]
coverage = { enabled = false, command = "cargo-llvm-cov" }
dead_code = { enabled = false, rustflags = "-W dead_code" }
function_size = { enabled = true }

[thresholds]
max_function_lines = 100
```

- [ ] **Step 4: Commit**

```bash
git add examples/configs/
git commit -m "docs: add example configurations"
```

---

### Task 6: Add Example Outputs

**Files:**
- Create: `examples/output/human.md`
- Create: `examples/output/report.json`

**Interfaces:**
- Produces: sample output files generated from the fixture crate.

- [ ] **Step 1: Generate sample human report**

Run:
```bash
cargo run -- --no-coverage --no-dead-code tests/fixtures/minimal_crate > examples/output/human.md 2>&1
```

- [ ] **Step 2: Generate sample JSON report**

Run:
```bash
cargo run -- --no-coverage --no-dead-code --format json tests/fixtures/minimal_crate > examples/output/report.json 2>&1
```

- [ ] **Step 3: Verify outputs are sensible**

`examples/output/human.md` should contain `Crate:`, `Function size`, and function names.
`examples/output/report.json` should be valid JSON.

- [ ] **Step 4: Commit**

```bash
git add examples/output/
git commit -m "docs: add example outputs from fixture crate"
```

---

### Task 7: Add AGENTS.md

**Files:**
- Create: `AGENTS.md`

**Interfaces:**
- Produces: agent-facing project guide.

- [ ] **Step 1: Create AGENTS.md**

Write `AGENTS.md`:
```markdown
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
```

- [ ] **Step 2: Commit**

```bash
git add AGENTS.md
git commit -m "docs: add AGENTS.md for agent discoverability"
```

---

### Task 8: Final Verification

**Files:**
- Verify: all of the above

**Interfaces:**
- Produces: passing CI-ready repository.

- [ ] **Step 1: Run formatting check**

```bash
cargo fmt --check
```

Expected: exit 0.

- [ ] **Step 2: Run clippy**

```bash
cargo clippy -- -D warnings
```

Expected: exit 0.

- [ ] **Step 3: Run tests**

```bash
cargo test
```

Expected: all tests pass.

- [ ] **Step 4: Verify git status is clean**

```bash
git status --short
```

Expected: no uncommitted changes (or only ignored files).

- [ ] **Step 5: Commit**

If verification surfaced no new changes, no additional commit is needed.
