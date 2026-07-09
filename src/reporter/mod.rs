pub mod human;
pub mod json;

use crate::model::Report;

pub trait Reporter {
    fn render(&self, report: &Report) -> String;
}

pub fn reporter_for(format: &str) -> Box<dyn Reporter> {
    match format {
        "json" => Box::new(json::JsonReporter),
        _ => Box::new(human::HumanReporter),
    }
}
