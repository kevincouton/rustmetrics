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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_serializes_to_lowercase() {
        assert_eq!(
            serde_json::to_string(&Severity::Error).unwrap(),
            "\"error\""
        );
        assert_eq!(
            serde_json::to_string(&Severity::Warning).unwrap(),
            "\"warning\""
        );
        assert_eq!(serde_json::to_string(&Severity::Note).unwrap(), "\"note\"");
    }

    #[test]
    fn severity_deserializes_from_lowercase() {
        assert_eq!(
            serde_json::from_str::<Severity>("\"error\"").unwrap(),
            Severity::Error
        );
        assert_eq!(
            serde_json::from_str::<Severity>("\"warning\"").unwrap(),
            Severity::Warning
        );
        assert_eq!(
            serde_json::from_str::<Severity>("\"note\"").unwrap(),
            Severity::Note
        );
    }

    #[test]
    fn report_new_initializes_empty() {
        let root = PathBuf::from("/tmp/test-crate");
        let report = Report::new(root.clone());

        assert_eq!(report.crate_root, root);
        assert!(report.coverage.is_none());
        assert!(report.dead_code.is_empty());
        assert!(report.function_sizes.is_empty());
        assert!(report.threshold_violations.is_empty());
    }

    #[test]
    fn coverage_summary_round_trips_json() {
        let summary = CoverageSummary {
            line_rate: 0.85,
            branch_rate: Some(0.75),
            function_rate: None,
            lines_covered: 42,
            lines_total: 50,
        };

        let json = serde_json::to_string(&summary).unwrap();
        let parsed: CoverageSummary = serde_json::from_str(&json).unwrap();

        assert!((parsed.line_rate - summary.line_rate).abs() < f64::EPSILON);
        assert_eq!(parsed.branch_rate, summary.branch_rate);
        assert_eq!(parsed.function_rate, summary.function_rate);
        assert_eq!(parsed.lines_covered, summary.lines_covered);
        assert_eq!(parsed.lines_total, summary.lines_total);
    }

    #[test]
    fn function_metric_round_trips_json() {
        let metric = FunctionMetric {
            file: PathBuf::from("src/lib.rs"),
            name: "foo".into(),
            line_start: 10,
            line_end: 25,
            line_count: 16,
            statement_count: 5,
        };

        let json = serde_json::to_string(&metric).unwrap();
        let parsed: FunctionMetric = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.file, metric.file);
        assert_eq!(parsed.name, metric.name);
        assert_eq!(parsed.line_start, metric.line_start);
        assert_eq!(parsed.line_end, metric.line_end);
        assert_eq!(parsed.line_count, metric.line_count);
        assert_eq!(parsed.statement_count, metric.statement_count);
    }

    #[test]
    fn dead_code_item_round_trips_json() {
        let item = DeadCodeItem {
            file: PathBuf::from("src/lib.rs"),
            name: "unused_fn".into(),
            item_kind: "fn".into(),
            line: 7,
            column: 4,
        };

        let json = serde_json::to_string(&item).unwrap();
        let parsed: DeadCodeItem = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.file, item.file);
        assert_eq!(parsed.name, item.name);
        assert_eq!(parsed.item_kind, item.item_kind);
        assert_eq!(parsed.line, item.line);
        assert_eq!(parsed.column, item.column);
    }

    #[test]
    fn threshold_violation_round_trips_json() {
        let violation = ThresholdViolation {
            threshold: "function-size".into(),
            actual: 120.0,
            limit: 100.0,
            detail: "function foo is too large".into(),
        };

        let json = serde_json::to_string(&violation).unwrap();
        let parsed: ThresholdViolation = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.threshold, violation.threshold);
        assert!((parsed.actual - violation.actual).abs() < f64::EPSILON);
        assert!((parsed.limit - violation.limit).abs() < f64::EPSILON);
        assert_eq!(parsed.detail, violation.detail);
    }
}
