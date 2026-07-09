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
