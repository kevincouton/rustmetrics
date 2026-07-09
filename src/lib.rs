pub mod cli;
pub mod config;
pub mod model;
pub mod providers;
pub mod reporter;
pub mod runner;

pub use cli::Args;
pub use config::Config;
pub use model::*;
pub use providers::{MetricError, MetricProvider, ProviderOutput};
pub use runner::Runner;

use crate::reporter::reporter_for;
use anyhow::Result;

pub fn run(args: Args) -> Result<()> {
    let config = Config::load(args.config.as_deref())?;
    let runner = Runner::new(config, args);
    let report = runner.run()?;

    let reporter = reporter_for(runner.format());
    println!("{}", reporter.render(&report));

    if report.threshold_violations.is_empty() {
        Ok(())
    } else {
        std::process::exit(1);
    }
}
