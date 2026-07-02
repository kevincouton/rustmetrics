use rustmetrics::{run, Args};
use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    run(args)
}
