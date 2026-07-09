use crate::model::Report;
use crate::reporter::Reporter;

pub struct JsonReporter;

impl Reporter for JsonReporter {
    fn render(&self, report: &Report) -> String {
        serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".to_string())
    }
}
