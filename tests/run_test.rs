use clap::Parser;
use rustmetrics::cli::Args;
use rustmetrics::run;
use rustmetrics::RunStatus;

#[test]
fn test_run_returns_ok_when_no_threshold_violations() {
    let args = Args::parse_from([
        "rustmetrics",
        "--no-coverage",
        "--no-dead-code",
        "tests/fixtures/minimal_crate",
    ]);
    let status = run(args).unwrap();
    assert_eq!(status, RunStatus::Ok);
}

#[test]
fn test_run_returns_threshold_violations_when_coverage_below_min() {
    let args = Args::parse_from([
        "rustmetrics",
        "--no-dead-code",
        "--format",
        "json",
        "tests/fixtures/minimal_crate",
    ]);
    let status = run(args).unwrap();
    assert_eq!(status, RunStatus::ThresholdViolations);
}
