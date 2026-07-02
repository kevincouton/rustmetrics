use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "rustmetrics")]
#[command(about = "Aggregate test coverage, dead code, and function-size metrics for Rust projects")]
pub struct Args {
    /// Path to the crate to analyze.
    #[arg(default_value = ".")]
    pub crate_root: PathBuf,

    /// Output format.
    #[arg(short, long, default_value = "human")]
    pub format: String,

    /// Path to a TOML configuration file.
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Skip the coverage provider.
    #[arg(long)]
    pub no_coverage: bool,

    /// Skip the dead-code provider.
    #[arg(long)]
    pub no_dead_code: bool,

    /// Skip the function-size provider.
    #[arg(long)]
    pub no_function_size: bool,
}

impl Args {
    /// Returns the crate root as a `PathBuf`.
    pub fn crate_root(&self) -> PathBuf {
        self.crate_root.clone()
    }
}
