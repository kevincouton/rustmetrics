use rustmetrics::cli::Args;
use rustmetrics::config::Config;
use rustmetrics::runner::Runner;
use clap::Parser;

#[test]
fn test_runner_respects_cli_flags() {
    let config = Config::load(None).unwrap();
    let args = Args::parse_from(["rustmetrics", "--no-coverage", "--no-dead-code", "."]);
    let runner = Runner::new(config, args);
    assert_eq!(runner.providers.len(), 1); // only function_size
}

#[test]
fn test_runner_format_is_stored() {
    let config = Config::load(None).unwrap();
    let args = Args::parse_from(["rustmetrics", "--format", "json", "."]);
    let runner = Runner::new(config, args);
    assert_eq!(runner.format(), "json");
}
