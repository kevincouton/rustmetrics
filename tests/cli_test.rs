use clap::Parser;
use rustmetrics::cli::Args;

#[test]
fn test_parse_defaults() {
    let args = Args::parse_from(["rustmetrics"]);
    assert_eq!(args.crate_root, std::path::PathBuf::from("."));
    assert_eq!(args.format, "human");
    assert!(args.config.is_none());
    assert!(!args.no_coverage);
}

#[test]
fn test_parse_options() {
    let args = Args::parse_from([
        "rustmetrics",
        "--format",
        "json",
        "--config",
        "myconfig.toml",
        "--no-dead-code",
        "/tmp/my-crate",
    ]);
    assert_eq!(args.format, "json");
    assert_eq!(args.config, Some(std::path::PathBuf::from("myconfig.toml")));
    assert!(args.no_dead_code);
    assert_eq!(args.crate_root, std::path::PathBuf::from("/tmp/my-crate"));
}

#[test]
fn test_crate_root_accessor() {
    let args = Args::parse_from(["rustmetrics", "/tmp/my-crate"]);
    assert_eq!(args.crate_root(), std::path::PathBuf::from("/tmp/my-crate"));
}
