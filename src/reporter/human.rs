use crate::model::Report;
use crate::reporter::Reporter;

pub struct HumanReporter;

impl Reporter for HumanReporter {
    fn render(&self, report: &Report) -> String {
        let mut lines = Vec::new();
        lines.push(format!("Crate: {}", report.crate_root.display()));
        lines.push(String::new());

        lines.push("Coverage".to_string());
        match &report.coverage {
            Some(c) => lines.push(format!(
                "  Line coverage: {:.1}% ({} / {} lines)",
                c.line_rate * 100.0,
                c.lines_covered,
                c.lines_total
            )),
            None => lines.push("  (skipped)".to_string()),
        }
        lines.push(String::new());

        lines.push("Dead code".to_string());
        if report.dead_code.is_empty() {
            lines.push("  None".to_string());
        } else {
            for item in &report.dead_code {
                lines.push(format!(
                    "  {}:{}:{} unused {} `{}`",
                    item.file.display(),
                    item.line,
                    item.column,
                    item.item_kind,
                    item.name
                ));
            }
        }
        lines.push(String::new());

        lines.push("Function size".to_string());
        if report.function_sizes.is_empty() {
            lines.push("  None".to_string());
        } else {
            for func in &report.function_sizes {
                lines.push(format!(
                    "  {}:{}  {}  {} lines  {} statements",
                    func.file.display(),
                    func.line_start,
                    func.name,
                    func.line_count,
                    func.statement_count
                ));
            }
        }

        if !report.threshold_violations.is_empty() {
            lines.push(String::new());
            lines.push("Threshold violations".to_string());
            for v in &report.threshold_violations {
                lines.push(format!("  {}: {}", v.threshold, v.detail));
            }
        }

        lines.join("\n")
    }
}
