pub mod dead_code;
pub mod function_size;

use crate::model::{CoverageSummary, DeadCodeItem, FunctionMetric};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MetricError {
    #[error("command `{command}` failed: {source}")]
    CommandFailed {
        command: String,
        #[source]
        source: std::io::Error,
    },
    #[error("command `{command}` exited with status {status}: {stderr}")]
    CommandExit {
        command: String,
        status: i32,
        stderr: String,
    },
    #[error("failed to parse {provider} output: {source}")]
    ParseError {
        provider: String,
        #[source]
        source: anyhow::Error,
    },
    #[error("tool `{tool}` is not installed")]
    ToolNotFound { tool: String },
}

#[derive(Debug)]
pub enum ProviderOutput {
    Coverage(CoverageSummary),
    DeadCode(Vec<DeadCodeItem>),
    FunctionSizes(Vec<FunctionMetric>),
}

pub trait MetricProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn collect(&self, crate_root: &Path) -> Result<ProviderOutput, MetricError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    struct DummyProvider;

    impl MetricProvider for DummyProvider {
        fn name(&self) -> &'static str {
            "dummy"
        }

        fn collect(&self, _crate_root: &Path) -> Result<ProviderOutput, MetricError> {
            Ok(ProviderOutput::Coverage(CoverageSummary {
                line_rate: 1.0,
                branch_rate: None,
                function_rate: None,
                lines_covered: 1,
                lines_total: 1,
            }))
        }
    }

    #[test]
    fn metric_provider_trait_can_be_implemented() {
        let provider = DummyProvider;
        assert_eq!(provider.name(), "dummy");

        let output = provider.collect(Path::new("/tmp/dummy")).unwrap();
        match output {
            ProviderOutput::Coverage(summary) => {
                assert!((summary.line_rate - 1.0).abs() < f64::EPSILON);
            }
            _ => panic!("expected coverage output"),
        }
    }

    #[test]
    fn command_failed_error_formats_correctly() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "no such file");
        let err = MetricError::CommandFailed {
            command: "cargo".into(),
            source: io_err,
        };
        let msg = err.to_string();
        assert!(msg.contains("command `cargo` failed"));
        assert!(msg.contains("no such file"));
    }

    #[test]
    fn command_exit_error_formats_correctly() {
        let err = MetricError::CommandExit {
            command: "cargo".into(),
            status: 101,
            stderr: "build failed".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("command `cargo` exited with status 101"));
        assert!(msg.contains("build failed"));
    }

    #[test]
    fn parse_error_formats_correctly() {
        let err = MetricError::ParseError {
            provider: "coverage".into(),
            source: anyhow::anyhow!("invalid JSON"),
        };
        let msg = err.to_string();
        assert!(msg.contains("failed to parse coverage output"));
        assert!(msg.contains("invalid JSON"));
    }

    #[test]
    fn tool_not_found_error_formats_correctly() {
        let err = MetricError::ToolNotFound {
            tool: "llvm-cov".into(),
        };
        assert_eq!(err.to_string(), "tool `llvm-cov` is not installed");
    }

    #[test]
    fn provider_output_coverage_round_trips_via_model() {
        let summary = CoverageSummary {
            line_rate: 0.5,
            branch_rate: Some(0.25),
            function_rate: Some(0.75),
            lines_covered: 10,
            lines_total: 20,
        };
        let output = ProviderOutput::Coverage(summary.clone());

        match output {
            ProviderOutput::Coverage(parsed) => {
                assert!((parsed.line_rate - summary.line_rate).abs() < f64::EPSILON);
                assert_eq!(parsed.branch_rate, summary.branch_rate);
            }
            _ => panic!("expected coverage"),
        }
    }

    #[test]
    fn provider_output_dead_code_round_trips_via_model() {
        let items = vec![DeadCodeItem {
            file: PathBuf::from("src/lib.rs"),
            name: "dead".into(),
            item_kind: "fn".into(),
            line: 3,
            column: 1,
        }];
        let output = ProviderOutput::DeadCode(items.clone());

        match output {
            ProviderOutput::DeadCode(parsed) => {
                assert_eq!(parsed.len(), 1);
                assert_eq!(parsed[0].name, "dead");
            }
            _ => panic!("expected dead code"),
        }
    }

    #[test]
    fn provider_output_function_sizes_round_trips_via_model() {
        let metrics = vec![FunctionMetric {
            file: PathBuf::from("src/lib.rs"),
            name: "big".into(),
            line_start: 1,
            line_end: 10,
            line_count: 10,
            statement_count: 7,
        }];
        let output = ProviderOutput::FunctionSizes(metrics.clone());

        match output {
            ProviderOutput::FunctionSizes(parsed) => {
                assert_eq!(parsed.len(), 1);
                assert_eq!(parsed[0].name, "big");
            }
            _ => panic!("expected function sizes"),
        }
    }
}
