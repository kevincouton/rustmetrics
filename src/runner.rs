use crate::cli::Args;
use crate::config::Config;
use crate::model::{Report, ThresholdViolation};
use crate::providers::{
    coverage::CoverageProvider, dead_code::DeadCodeProvider, function_size::FunctionSizeProvider,
    MetricProvider, ProviderOutput,
};
use anyhow::Result;
use std::sync::Arc;

pub struct Runner {
    pub providers: Vec<Arc<dyn MetricProvider>>,
    config: Config,
    crate_root: std::path::PathBuf,
    format: String,
}

impl Runner {
    pub fn new(config: Config, args: Args) -> Self {
        let mut providers: Vec<Arc<dyn MetricProvider>> = Vec::new();

        if config.providers.coverage.enabled && !args.no_coverage {
            providers.push(Arc::new(CoverageProvider::new(
                config.providers.coverage.command.clone(),
            )));
        }

        if config.providers.dead_code.enabled && !args.no_dead_code {
            providers.push(Arc::new(DeadCodeProvider::new(
                config.providers.dead_code.rustflags.clone(),
            )));
        }

        if config.providers.function_size.enabled && !args.no_function_size {
            providers.push(Arc::new(FunctionSizeProvider::new()));
        }

        Self {
            providers,
            config,
            crate_root: args.crate_root,
            format: args.format,
        }
    }

    pub fn format(&self) -> &str {
        &self.format
    }

    pub fn run(&self) -> Result<Report> {
        let mut report = Report::new(self.crate_root.clone());

        for provider in &self.providers {
            match provider.collect(&self.crate_root) {
                Ok(ProviderOutput::Coverage(summary)) => report.coverage = Some(summary),
                Ok(ProviderOutput::DeadCode(items)) => report.dead_code.extend(items),
                Ok(ProviderOutput::FunctionSizes(metrics)) => report.function_sizes.extend(metrics),
                Err(e) => {
                    eprintln!("provider {} failed: {}", provider.name(), e);
                    anyhow::bail!(e);
                }
            }
        }

        report
            .function_sizes
            .sort_by(|a, b| a.file.cmp(&b.file).then(a.line_start.cmp(&b.line_start)));
        report
            .dead_code
            .sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));

        apply_thresholds(&mut report, &self.config);

        Ok(report)
    }
}

fn apply_thresholds(report: &mut Report, config: &Config) {
    if let Some(min) = config.thresholds.min_line_coverage {
        if let Some(coverage) = &report.coverage {
            if coverage.line_rate < min {
                report.threshold_violations.push(ThresholdViolation {
                    threshold: "min_line_coverage".to_string(),
                    actual: coverage.line_rate,
                    limit: min,
                    detail: format!(
                        "line coverage {}% is below {}%",
                        coverage.line_rate * 100.0,
                        min * 100.0
                    ),
                });
            }
        }
    }

    if let Some(max) = config.thresholds.max_function_lines {
        for func in &report.function_sizes {
            if func.line_count > max {
                report.threshold_violations.push(ThresholdViolation {
                    threshold: "max_function_lines".to_string(),
                    actual: func.line_count as f64,
                    limit: max as f64,
                    detail: format!("{} has {} lines", func.name, func.line_count),
                });
            }
        }
    }

    if let Some(max) = config.thresholds.max_dead_code_items {
        if report.dead_code.len() > max {
            report.threshold_violations.push(ThresholdViolation {
                threshold: "max_dead_code_items".to_string(),
                actual: report.dead_code.len() as f64,
                limit: max as f64,
                detail: format!("{} dead-code items found", report.dead_code.len()),
            });
        }
    }
}
