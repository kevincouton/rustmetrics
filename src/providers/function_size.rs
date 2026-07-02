use crate::model::FunctionMetric;
use crate::providers::{MetricError, MetricProvider, ProviderOutput};
use std::path::{Path, PathBuf};
use tree_sitter::{Node, Parser, Query, QueryCursor};

pub struct FunctionSizeProvider;

impl FunctionSizeProvider {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze_source(&self, source: &str, file: PathBuf) -> Vec<FunctionMetric> {
        let language = tree_sitter_rust::LANGUAGE.into();

        let mut parser = Parser::new();
        parser
            .set_language(&language)
            .expect("tree-sitter rust grammar should load");

        let tree = match parser.parse(source, None) {
            Some(tree) => tree,
            None => {
                eprintln!(
                    "warning: failed to parse Rust source in '{}'",
                    file.display()
                );
                return Vec::new();
            }
        };

        let query_str = "(function_item name: (identifier) @name) @function";
        let query = match Query::new(&language, query_str) {
            Ok(q) => q,
            Err(e) => {
                eprintln!(
                    "warning: failed to build tree-sitter query for '{}': {}",
                    file.display(),
                    e
                );
                return Vec::new();
            }
        };

        let root = tree.root_node();
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, root, source.as_bytes());

        let mut metrics = Vec::new();
        for m in matches {
            let mut name_node: Option<Node> = None;
            let mut function_node: Option<Node> = None;

            for capture in m.captures {
                let name = query.capture_names()[capture.index as usize];
                if name == "name" {
                    name_node = Some(capture.node);
                } else if name == "function" {
                    function_node = Some(capture.node);
                }
            }

            let (Some(name_node), Some(function_node)) = (name_node, function_node) else {
                continue;
            };

            let name = source[name_node.byte_range()].to_string();
            let start = function_node.start_position();
            let end = function_node.end_position();
            let line_count = end.row.saturating_sub(start.row) + 1;
            let statement_count = count_statements(function_node);

            metrics.push(FunctionMetric {
                file: file.clone(),
                name,
                line_start: start.row + 1,
                line_end: end.row + 1,
                line_count,
                statement_count,
            });
        }

        metrics
    }
}

fn count_statements(function_node: Node) -> usize {
    let body = function_node
        .children(&mut function_node.walk())
        .find(|child| child.kind() == "block");

    let Some(body) = body else { return 0 };

    let mut count = 0;
    count_statements_in_node(body, &mut count);
    count
}

fn count_statements_in_node(node: Node, count: &mut usize) {
    let kind = node.kind();
    if is_statement(kind) {
        *count += 1;
    }

    // Don't attribute nested functions or closures to the enclosing function.
    if kind == "function_item" || kind == "closure_expression" {
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        count_statements_in_node(child, count);
    }
}

fn is_statement(kind: &str) -> bool {
    matches!(
        kind,
        "expression_statement"
            | "let_declaration"
            | "const_item"
            | "static_item"
            | "if_expression"
            | "match_expression"
            | "for_expression"
            | "while_expression"
            | "loop_expression"
            | "return_expression"
            | "break_expression"
            | "continue_expression"
    )
}

impl MetricProvider for FunctionSizeProvider {
    fn name(&self) -> &'static str {
        "function_size"
    }

    fn collect(&self, crate_root: &Path) -> Result<ProviderOutput, MetricError> {
        let mut metrics = Vec::new();

        for entry in walkdir::WalkDir::new(crate_root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            if path.components().any(|c| c.as_os_str() == "target") {
                continue;
            }

            let source = match std::fs::read_to_string(path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("warning: failed to read '{}': {}", path.display(), e);
                    continue;
                }
            };

            let relative = path.strip_prefix(crate_root).unwrap_or(path).to_path_buf();
            metrics.extend(self.analyze_source(&source, relative));
        }

        Ok(ProviderOutput::FunctionSizes(metrics))
    }
}
