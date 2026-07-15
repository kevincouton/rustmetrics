use rustmetrics::providers::function_size::FunctionSizeProvider;
use rustmetrics::providers::MetricProvider;
use std::path::PathBuf;

#[test]
fn test_function_size_default_impl() {
    let provider: FunctionSizeProvider = Default::default();
    assert_eq!(provider.name(), "function_size");
}

#[test]
fn test_function_size_on_inline_source() {
    let provider = FunctionSizeProvider::new();
    let source = r#"
fn small() {
    let x = 1;
    let y = 2;
}

fn big() {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    if a > 0 {
        println!("yes");
    }
}
"#;
    let metrics = provider.analyze_source(source, PathBuf::from("src/lib.rs"));
    assert_eq!(metrics.len(), 2);

    let small = metrics.iter().find(|m| m.name == "small").unwrap();
    assert_eq!(small.line_count, 4);
    assert_eq!(small.statement_count, 2);

    let big = metrics.iter().find(|m| m.name == "big").unwrap();
    assert!(big.line_count >= 9);
    assert!(big.statement_count >= 6);
}

#[test]
fn test_function_size_invalid_source_returns_empty() {
    let provider = FunctionSizeProvider::new();
    let metrics = provider.analyze_source("not valid rust! {{", PathBuf::from("src/lib.rs"));
    assert!(metrics.is_empty());
}

#[test]
fn test_function_size_nested_functions_not_attributed() {
    let provider = FunctionSizeProvider::new();
    let source = r#"
fn outer() {
    let x = 1;
    fn inner() {
        let a = 1;
        let b = 2;
    }
    let y = 2;
}
"#;
    let metrics = provider.analyze_source(source, PathBuf::from("src/lib.rs"));
    assert_eq!(metrics.len(), 2);
    let outer = metrics.iter().find(|m| m.name == "outer").unwrap();
    assert_eq!(outer.line_count, 8);
    // inner statements should not be counted in outer
    assert_eq!(outer.statement_count, 2);
}
