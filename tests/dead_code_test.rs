use rustmetrics::providers::dead_code::DeadCodeProvider;
use std::path::PathBuf;

#[test]
fn test_parse_dead_code_messages() {
    let provider = DeadCodeProvider::new("-W dead_code".to_string());
    let jsonl = r#"
{"reason":"compiler-message","package_id":"pkg","target":{"kind":["lib"],"name":"mylib","src_path":"/tmp/lib.rs"},"message":{"rendered":"warning: function `unused_fn` is never used\n --> src/lib.rs:3:1\n  |\n3 | fn unused_fn() {}\n  | ^^^^^^^^^^^^^^\n","spans":[{"file_name":"src/lib.rs","byte_start":0,"byte_end":16,"line_start":3,"line_end":3,"column_start":1,"column_end":17}],"code":{"code":"dead_code"},"level":"warning","message":"function `unused_fn` is never used"}}
{"reason":"compiler-artifact","package_id":"pkg","target":{"kind":["lib"],"name":"mylib","src_path":"/tmp/lib.rs"},"filenames":[],"executable":null,"fresh":false}
"#;
    let items = provider.parse_messages(jsonl, PathBuf::from("/tmp/my-crate"));
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].name, "unused_fn");
    assert_eq!(items[0].item_kind, "function");
    assert_eq!(items[0].line, 3);
    assert_eq!(items[0].column, 1);
}

#[test]
fn test_parse_dead_code_messages_skips_compiler_errors() {
    let provider = DeadCodeProvider::new("-W dead_code".to_string());
    let jsonl = r#"
{"reason":"compiler-message","package_id":"pkg","target":{"kind":["lib"],"name":"mylib","src_path":"/tmp/lib.rs"},"message":{"rendered":"error: cannot find value `x` in this scope\n --> src/lib.rs:5:1\n  |\n5 | x;\n  | ^\n","spans":[{"file_name":"src/lib.rs","byte_start":0,"byte_end":1,"line_start":5,"line_end":5,"column_start":1,"column_end":2}],"code":{"code":"E0425"},"level":"error","message":"cannot find value `x` in this scope"}}
{"reason":"compiler-message","package_id":"pkg","target":{"kind":["lib"],"name":"mylib","src_path":"/tmp/lib.rs"},"message":{"rendered":"warning: constant `UNUSED` is never used\n --> src/lib.rs:7:1\n  |\n7 | const UNUSED: u32 = 1;\n  | ^^^^^^^^^^^^^^^^^^^^\n","spans":[{"file_name":"src/lib.rs","byte_start":0,"byte_end":21,"line_start":7,"line_end":7,"column_start":1,"column_end":22}],"code":{"code":"dead_code"},"level":"warning","message":"constant `UNUSED` is never used"}}
"#;
    let items = provider.parse_messages(jsonl, PathBuf::from("/tmp/my-crate"));
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].name, "UNUSED");
    assert_eq!(items[0].item_kind, "constant");
    assert_eq!(items[0].line, 7);
    assert_eq!(items[0].column, 1);
}
