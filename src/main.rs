use clap::Parser;
use rustmetrics::{run, Args, RunStatus};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match run(args)? {
        RunStatus::Ok => Ok(()),
        RunStatus::ThresholdViolations => std::process::exit(1),
    }
}
