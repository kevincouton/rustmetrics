use crate::model::CoverageSummary;
use crate::providers::{MetricError, MetricProvider, ProviderOutput};
use serde::Deserialize;
use std::path::Path;
use std::process::Command;

pub struct CoverageProvider {
    command: String,
}

impl CoverageProvider {
    pub fn new(command: String) -> Self {
        Self { command }
    }

    pub fn parse_json(&self, text: &str) -> Result<CoverageSummary, MetricError> {
        let report: LlvmCovReport =
            serde_json::from_str(text).map_err(|e| MetricError::ParseError {
                provider: "coverage".to_string(),
                source: e.into(),
            })?;

        let data = report
            .data
            .into_iter()
            .next()
            .ok_or_else(|| MetricError::ParseError {
                provider: "coverage".to_string(),
                source: anyhow::anyhow!("no data section in coverage report"),
            })?;

        let lines = data.totals.lines;
        let line_rate = if lines.count == 0 {
            1.0
        } else {
            lines.percent / 100.0
        };

        Ok(CoverageSummary {
            line_rate,
            branch_rate: data.totals.branches.map(|b| b.percent / 100.0),
            function_rate: data.totals.functions.map(|f| f.percent / 100.0),
            lines_covered: lines.covered,
            lines_total: lines.count,
        })
    }
}

#[derive(Debug, Deserialize)]
struct LlvmCovReport {
    data: Vec<LlvmCovData>,
}

#[derive(Debug, Deserialize)]
struct LlvmCovData {
    totals: LlvmCovTotals,
}

#[derive(Debug, Deserialize)]
struct LlvmCovTotals {
    lines: LlvmCovStat,
    #[serde(default)]
    functions: Option<LlvmCovStat>,
    #[serde(default)]
    branches: Option<LlvmCovStat>,
}

#[derive(Debug, Deserialize)]
struct LlvmCovStat {
    count: u64,
    covered: u64,
    percent: f64,
}

impl MetricProvider for CoverageProvider {
    fn name(&self) -> &'static str {
        "coverage"
    }

    fn collect(&self, crate_root: &Path) -> Result<ProviderOutput, MetricError> {
        let output = if self.command == "cargo-llvm-cov" {
            Command::new("cargo")
                .args(["llvm-cov", "--json"])
                .current_dir(crate_root)
                .output()
        } else {
            Command::new(&self.command).current_dir(crate_root).output()
        }
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                MetricError::ToolNotFound {
                    tool: self.command.clone(),
                }
            } else {
                MetricError::CommandFailed {
                    command: self.command.clone(),
                    source: e,
                }
            }
        })?;

        if !output.status.success() {
            return Err(MetricError::CommandExit {
                command: self.command.clone(),
                status: output.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let summary = self.parse_json(&stdout)?;
        eprintln!(
            "coverage: {:.1}% lines covered ({} / {})",
            summary.line_rate * 100.0,
            summary.lines_covered,
            summary.lines_total
        );
        Ok(ProviderOutput::Coverage(summary))
    }
}
