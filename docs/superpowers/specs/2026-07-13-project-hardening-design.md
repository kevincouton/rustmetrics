# Project Hardening, CI, and Examples Design

## Overview

Add standard open-source hygiene, continuous integration, runnable examples, and agent-facing documentation to the rustmetrics project.

## 1. CI/CD

File: `.github/workflows/ci.yml`

Triggers: push and pull request to `main`.

Jobs:
- `check` job running on `ubuntu-latest`:
  - Checkout code
  - Install Rust stable toolchain
  - Run `cargo fmt --check`
  - Run `cargo clippy -- -D warnings`
  - Run `cargo test`
  - Run `cargo build --release`

Coverage provider tests require `cargo-llvm-cov`; since the project is designed to allow skipping coverage, the CI workflow does not install it. Tests that require it must either be skipped or tolerate its absence. Current tests already pass without it.

## 2. SDD Scratch Cleanup

The Subagent-Driven Development workflow produced scratch files under `.superpowers/sdd/`:
- Task briefs (`task-N-brief.md`)
- Task reports (`task-N-report.md`)
- Review diffs (`review-*.diff`)
- Progress ledger (`progress.md`)

These are internal planning artifacts and should not be in the published repository. Remove them and update `.gitignore` so future scratch files are ignored.

Update `.gitignore` to also ignore:
- `.superpowers/`
- `tests/fixtures/*/target/`
- `tests/fixtures/*/Cargo.lock`

## 3. Project Polish

### LICENSE

Add `LICENSE` file with the MIT OR Apache-2.0 dual license text, matching the `Cargo.toml` license field.

### CODE_OF_CONDUCT.md

Add standard Contributor Covenant code of conduct.

### CONTRIBUTING.md

Add guidelines covering:
- How to build and test
- Code formatting and clippy requirements
- How to open issues and pull requests
- Commit message conventions used in the project

### Issue Templates

Add `.github/ISSUE_TEMPLATE/bug_report.md` and `.github/ISSUE_TEMPLATE/feature_request.md`.

## 4. Examples by Category

Create `examples/` directory with:

### Configuration examples (`examples/configs/`)

- `strict.toml` — low thresholds, all providers enabled
- `relaxed.toml` — generous thresholds for legacy codebases
- `function_size_only.toml` — disable coverage and dead-code providers

### Output examples (`examples/output/`)

- `human.md` — sample human-readable report from the fixture crate
- `report.json` — sample JSON report from the fixture crate

## 5. Agent Friendliness

Add `AGENTS.md` at repository root containing:
- Quick start: build, test, and run commands
- Project architecture overview
- How to add a new provider
- How to add a new reporter
- CI expectations

## Success Criteria

- `cargo test`, `cargo fmt --check`, and `cargo clippy -- -D warnings` remain clean
- GitHub Actions workflow passes
- Repository no longer contains SDD scratch files
- Examples are valid and representative
- `AGENTS.md` is accurate and helpful
