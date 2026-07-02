use rustmetrics::providers::function_size::FunctionSizeProvider;
use std::path::PathBuf;

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
