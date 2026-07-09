use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::contains;
use std::path::PathBuf;

fn bin() -> Command {
    Command::cargo_bin("rustmetrics").unwrap()
}

#[test]
fn test_function_size_only() {
    let mut cmd = bin();
    let fixture = PathBuf::from("tests/fixtures/minimal_crate");
    cmd.args([
        "--no-coverage",
        "--no-dead-code",
        "--format",
        "json",
        &fixture.to_string_lossy(),
    ]);
    cmd.assert()
        .success()
        .stdout(contains("\"name\"").and(contains("\"large_function\"")))
        .stdout(contains("\"name\"").and(contains("\"unused_helper\"")));
}

#[test]
fn test_help() {
    let mut cmd = bin();
    cmd.arg("--help");
    cmd.assert().success().stdout(contains("rustmetrics"));
}
