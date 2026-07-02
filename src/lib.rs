pub mod cli;
pub mod config;
pub mod model;
pub mod providers;
pub mod reporter;
pub mod runner;

use anyhow::Result;

pub fn run() -> Result<()> {
    println!("rustmetrics scaffold ready");
    Ok(())
}
