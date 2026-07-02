pub mod cli;
pub mod config;
pub mod model;
pub mod providers;
pub mod reporter;
pub mod runner;

pub use model::*;
pub use providers::{MetricError, MetricProvider, ProviderOutput};

use anyhow::Result;

pub fn run() -> Result<()> {
    println!("rustmetrics scaffold ready");
    Ok(())
}
