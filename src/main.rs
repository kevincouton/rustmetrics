use clap::Parser;
use rustmetrics::{run, Args};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    run(args)
}
