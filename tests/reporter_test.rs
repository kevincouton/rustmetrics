use rustmetrics::model::{
    CoverageSummary, DeadCodeItem, FunctionMetric, Report, ThresholdViolation,
};
use rustmetrics::reporter::reporter_for;
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

#[test]
fn test_human_reporter_with_coverage() {
    let mut report = Report::new(PathBuf::from("/tmp/crate"));
    report.coverage = Some(CoverageSummary {
        line_rate: 0.85,
        branch_rate: Some(0.75),
        function_rate: Some(0.90),
        lines_covered: 85,
        lines_total: 100,
    });
    let reporter = reporter_for("human");
    let output = reporter.render(&report);
    assert!(output.contains("85.0%"));
    assert!(output.contains("85 / 100 lines"));
}

#[test]
fn test_human_reporter_with_dead_code() {
    let mut report = Report::new(PathBuf::from("/tmp/crate"));
    report.dead_code.push(DeadCodeItem {
        file: PathBuf::from("src/lib.rs"),
        name: "unused_fn".into(),
        item_kind: "fn".into(),
        line: 7,
        column: 4,
    });
    let reporter = reporter_for("human");
    let output = reporter.render(&report);
    assert!(output.contains("unused_fn"));
    assert!(output.contains("src/lib.rs:7:4"));
}

#[test]
fn test_human_reporter_with_function_sizes() {
    let mut report = Report::new(PathBuf::from("/tmp/crate"));
    report.function_sizes.push(FunctionMetric {
        file: PathBuf::from("src/lib.rs"),
        name: "big_fn".into(),
        line_start: 10,
        line_end: 30,
        line_count: 21,
        statement_count: 12,
    });
    let reporter = reporter_for("human");
    let output = reporter.render(&report);
    assert!(output.contains("big_fn"));
    assert!(output.contains("21 lines"));
    assert!(output.contains("12 statements"));
}

#[test]
fn test_human_reporter_with_threshold_violations() {
    let mut report = Report::new(PathBuf::from("/tmp/crate"));
    report.threshold_violations.push(ThresholdViolation {
        threshold: "max_function_lines".into(),
        actual: 120.0,
        limit: 100.0,
        detail: "big_fn has 120 lines".into(),
    });
    let reporter = reporter_for("human");
    let output = reporter.render(&report);
    assert!(output.contains("Threshold violations"));
    assert!(output.contains("big_fn has 120 lines"));
}
