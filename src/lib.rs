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

use anyhow::Result;

pub fn run(args: Args) -> Result<()> {
    let _config = Config::load(args.config.as_deref())?;
    println!("rustmetrics CLI ready");
    Ok(())
}
