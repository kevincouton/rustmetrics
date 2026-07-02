use rustmetrics::providers::coverage::CoverageProvider;

#[test]
fn test_parse_llvm_cov_json() {
    let provider = CoverageProvider::new("cargo-llvm-cov".to_string());
    let json = r#"
{
  "data": [{
    "totals": {
      "lines": { "count": 200, "covered": 169, "percent": 84.5 },
      "functions": { "count": 20, "covered": 15, "percent": 75.0 },
      "branches": { "count": 50, "covered": 40, "percent": 80.0 }
    }
  }]
}
"#;
    let summary = provider.parse_json(json).unwrap();
    assert_eq!(summary.line_rate, 0.845);
    assert_eq!(summary.lines_covered, 169);
    assert_eq!(summary.lines_total, 200);
    assert_eq!(summary.function_rate, Some(0.75));
    assert_eq!(summary.branch_rate, Some(0.80));
}
