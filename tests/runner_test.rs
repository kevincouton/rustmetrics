use clap::Parser;
use rustmetrics::cli::Args;
use rustmetrics::config::Config;
use rustmetrics::model::{CoverageSummary, DeadCodeItem, FunctionMetric};
use rustmetrics::providers::ProviderOutput;
use rustmetrics::providers::{MetricError, MetricProvider};
use rustmetrics::runner::Runner;
use std::path::{Path, PathBuf};
use std::sync::Arc;

struct MockCoverageProvider {
    line_rate: f64,
}

impl MetricProvider for MockCoverageProvider {
    fn name(&self) -> &'static str {
        "mock-coverage"
    }

    fn collect(&self, _crate_root: &Path) -> Result<ProviderOutput, MetricError> {
        Ok(ProviderOutput::Coverage(CoverageSummary {
            line_rate: self.line_rate,
            branch_rate: None,
            function_rate: None,
            lines_covered: 50,
            lines_total: 100,
        }))
    }
}

struct MockDeadCodeProvider {
    items: Vec<DeadCodeItem>,
}

impl MetricProvider for MockDeadCodeProvider {
    fn name(&self) -> &'static str {
        "mock-dead-code"
    }

    fn collect(&self, _crate_root: &Path) -> Result<ProviderOutput, MetricError> {
        Ok(ProviderOutput::DeadCode(self.items.clone()))
    }
}

struct MockFunctionSizeProvider {
    metrics: Vec<FunctionMetric>,
}

impl MetricProvider for MockFunctionSizeProvider {
    fn name(&self) -> &'static str {
        "mock-function-size"
    }

    fn collect(&self, _crate_root: &Path) -> Result<ProviderOutput, MetricError> {
        Ok(ProviderOutput::FunctionSizes(self.metrics.clone()))
    }
}

struct FailingProvider;

impl MetricProvider for FailingProvider {
    fn name(&self) -> &'static str {
        "failing-provider"
    }

    fn collect(&self, _crate_root: &Path) -> Result<ProviderOutput, MetricError> {
        Err(MetricError::ToolNotFound {
            tool: "mock-tool".into(),
        })
    }
}

#[test]
fn test_runner_respects_cli_flags() {
    let config = Config::load(None).unwrap();
    let args = Args::parse_from(["rustmetrics", "--no-coverage", "--no-dead-code", "."]);
    let runner = Runner::new(config, args);
    assert_eq!(runner.providers.len(), 1); // only function_size
}

#[test]
fn test_runner_format_is_stored() {
    let config = Config::load(None).unwrap();
    let args = Args::parse_from(["rustmetrics", "--format", "json", "."]);
    let runner = Runner::new(config, args);
    assert_eq!(runner.format(), "json");
}

#[test]
fn test_runner_run_populates_report_from_providers() {
    let mut config = Config::load(None).unwrap();
    // Disable thresholds so only provider output is asserted.
    config.thresholds.min_line_coverage = None;
    config.thresholds.max_function_lines = None;
    config.thresholds.max_dead_code_items = None;

    let args = Args::parse_from(["rustmetrics", "."]);
    let mut runner = Runner::new(config, args);

    let dead_code = vec![DeadCodeItem {
        file: PathBuf::from("src/lib.rs"),
        name: "unused_fn".into(),
        item_kind: "fn".into(),
        line: 5,
        column: 1,
    }];

    let function_sizes = vec![FunctionMetric {
        file: PathBuf::from("src/lib.rs"),
        name: "small_fn".into(),
        line_start: 1,
        line_end: 5,
        line_count: 5,
        statement_count: 3,
    }];

    runner.providers = vec![
        Arc::new(MockCoverageProvider { line_rate: 0.75 }),
        Arc::new(MockDeadCodeProvider {
            items: dead_code.clone(),
        }),
        Arc::new(MockFunctionSizeProvider {
            metrics: function_sizes.clone(),
        }),
    ];

    let report = runner.run().expect("run should succeed");

    assert!(report.coverage.is_some());
    assert!((report.coverage.unwrap().line_rate - 0.75).abs() < f64::EPSILON);

    assert_eq!(report.dead_code.len(), 1);
    assert_eq!(report.dead_code[0].name, "unused_fn");
    assert_eq!(report.dead_code[0].file, PathBuf::from("src/lib.rs"));

    assert_eq!(report.function_sizes.len(), 1);
    assert_eq!(report.function_sizes[0].name, "small_fn");
    assert_eq!(report.function_sizes[0].line_count, 5);
}

#[test]
fn test_runner_run_emits_all_threshold_violations() {
    let mut config = Config::load(None).unwrap();
    config.thresholds.min_line_coverage = Some(0.9);
    config.thresholds.max_function_lines = Some(10);
    config.thresholds.max_dead_code_items = Some(1);

    let args = Args::parse_from(["rustmetrics", "."]);
    let mut runner = Runner::new(config, args);

    runner.providers = vec![
        Arc::new(MockCoverageProvider { line_rate: 0.5 }),
        Arc::new(MockDeadCodeProvider {
            items: vec![
                DeadCodeItem {
                    file: PathBuf::from("src/a.rs"),
                    name: "d1".into(),
                    item_kind: "fn".into(),
                    line: 1,
                    column: 1,
                },
                DeadCodeItem {
                    file: PathBuf::from("src/b.rs"),
                    name: "d2".into(),
                    item_kind: "fn".into(),
                    line: 2,
                    column: 1,
                },
            ],
        }),
        Arc::new(MockFunctionSizeProvider {
            metrics: vec![FunctionMetric {
                file: PathBuf::from("src/lib.rs"),
                name: "big_fn".into(),
                line_start: 1,
                line_end: 12,
                line_count: 11,
                statement_count: 8,
            }],
        }),
    ];

    let report = runner.run().expect("run should succeed");

    let threshold_names: Vec<&str> = report
        .threshold_violations
        .iter()
        .map(|v| v.threshold.as_str())
        .collect();

    assert!(threshold_names.contains(&"min_line_coverage"));
    assert!(threshold_names.contains(&"max_function_lines"));
    assert!(threshold_names.contains(&"max_dead_code_items"));

    let coverage_violation = report
        .threshold_violations
        .iter()
        .find(|v| v.threshold == "min_line_coverage")
        .unwrap();
    assert!((coverage_violation.actual - 0.5).abs() < f64::EPSILON);
    assert!((coverage_violation.limit - 0.9).abs() < f64::EPSILON);

    let function_violation = report
        .threshold_violations
        .iter()
        .find(|v| v.threshold == "max_function_lines")
        .unwrap();
    assert!((function_violation.actual - 11.0).abs() < f64::EPSILON);
    assert!((function_violation.limit - 10.0).abs() < f64::EPSILON);

    let dead_code_violation = report
        .threshold_violations
        .iter()
        .find(|v| v.threshold == "max_dead_code_items")
        .unwrap();
    assert!((dead_code_violation.actual - 2.0).abs() < f64::EPSILON);
    assert!((dead_code_violation.limit - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_runner_run_propagates_provider_error_with_context() {
    let config = Config::load(None).unwrap();
    let args = Args::parse_from(["rustmetrics", "."]);
    let mut runner = Runner::new(config, args);

    runner.providers = vec![Arc::new(FailingProvider)];

    let err = runner.run().expect_err("run should fail");
    let msg = err.to_string();

    assert!(
        msg.contains("failing-provider"),
        "error should name the failing provider: {msg}"
    );
    assert!(
        msg.contains("mock-tool"),
        "error should include the underlying error: {msg}"
    );
}
