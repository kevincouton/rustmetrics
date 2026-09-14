# Security Policy

## Scope

rustmetrics is a CLI tool that aggregates test coverage, dead-code, and
function-size metrics for Rust projects. In scope for security reports:

- Vulnerabilities in rustmetrics itself (e.g. unsafe handling of untrusted
  project paths or TOML configuration, code execution via wrapped cargo
  subcommands beyond their documented purpose).
- Vulnerabilities in dependencies that rustmetrics pulls in and exposes to
  users.
- Leaked credentials or sensitive data in this repository.

Out of scope: issues in the Rust projects you analyze *with* rustmetrics, and
vulnerabilities already publicly disclosed and tracked in the RustSec advisory
database.

## Reporting a Vulnerability

Please report vulnerabilities **privately** via GitHub:

- Use [GitHub private vulnerability reporting](https://github.com/kevincouton/rustmetrics/security/advisories/new)
  to open a private security advisory.

Do **not** open a public issue for undisclosed vulnerabilities.

## Response Expectations

- Acknowledgement within 7 days.
- An initial assessment and, if accepted, a remediation plan within 30 days.
- Reporters will be credited in the fix commit/release notes unless they
  prefer to remain anonymous.

This project is maintained by a single maintainer on a best-effort basis;
thank you for your patience and responsible disclosure.
