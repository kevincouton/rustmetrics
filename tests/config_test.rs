use rustmetrics::config::Config;

#[test]
fn test_load_default_config() {
    let cfg = Config::load(None).unwrap();
    assert!(cfg.providers.coverage.enabled);
    assert!(cfg.providers.dead_code.enabled);
    assert!(cfg.providers.function_size.enabled);
    assert_eq!(cfg.thresholds.min_line_coverage, Some(0.80));
    assert_eq!(cfg.thresholds.max_function_lines, Some(100));
}

#[test]
fn test_load_config_from_path() {
    let toml = r#"
[providers]
coverage = { enabled = false }

[thresholds]
max_function_lines = 50
"#;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("rustmetrics.toml");
    std::fs::write(&path, toml).unwrap();

    let cfg = Config::load(Some(&path)).unwrap();
    assert!(!cfg.providers.coverage.enabled);
    assert_eq!(cfg.thresholds.max_function_lines, Some(50));
}

#[test]
fn test_partial_provider_table_preserves_string_defaults() {
    let toml = r#"
[providers.coverage]
enabled = true

[providers.dead_code]
enabled = true
"#;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("rustmetrics.toml");
    std::fs::write(&path, toml).unwrap();

    let cfg = Config::load(Some(&path)).unwrap();
    assert!(cfg.providers.coverage.enabled);
    assert_eq!(cfg.providers.coverage.command, "cargo-llvm-cov");
    assert!(cfg.providers.dead_code.enabled);
    assert_eq!(cfg.providers.dead_code.rustflags, "-W dead_code");
}
