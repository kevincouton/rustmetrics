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

## Local CI

You can run the GitHub Actions workflow locally with [act](https://github.com/nektos/act):

```bash
act
```

The repository includes an `.actrc` with sensible defaults.

## Reporting Issues

Use the issue templates for bug reports and feature requests.
