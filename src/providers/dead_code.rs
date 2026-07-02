use crate::model::DeadCodeItem;
use crate::providers::{MetricError, MetricProvider, ProviderOutput};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct DeadCodeProvider {
    rustflags: String,
}

impl DeadCodeProvider {
    pub fn new(rustflags: String) -> Self {
        Self { rustflags }
    }

    pub fn parse_messages(&self, jsonl: &str, _crate_root: PathBuf) -> Vec<DeadCodeItem> {
        let mut items = Vec::new();

        for line in jsonl.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let Ok(message): Result<CargoMessage, _> = serde_json::from_str(line) else {
                continue;
            };

            if message.reason != "compiler-message" {
                continue;
            }

            let Some(code) = message.message.code.as_ref() else {
                continue;
            };

            if code.code != "dead_code" {
                continue;
            }

            let Some(span) = message.message.spans.first() else {
                continue;
            };

            let name = extract_name(&message.message.message);
            let item_kind = infer_kind(&message.message.message);

            items.push(DeadCodeItem {
                file: PathBuf::from(&span.file_name),
                name,
                item_kind,
                line: span.line_start,
                column: span.column_start,
            });
        }

        items
    }
}

fn extract_name(message: &str) -> String {
    // Try "function `name` is never used" or "constant `NAME` is never used"
    if let Some(start) = message.find('`') {
        if let Some(end) = message[start + 1..].find('`') {
            return message[start + 1..start + 1 + end].to_string();
        }
    }
    "unknown".to_string()
}

fn infer_kind(message: &str) -> String {
    if message.contains("function") {
        "function".to_string()
    } else if message.contains("constant") {
        "constant".to_string()
    } else if message.contains("struct") {
        "struct".to_string()
    } else if message.contains("enum") {
        "enum".to_string()
    } else if message.contains("static") {
        "static".to_string()
    } else {
        "item".to_string()
    }
}

#[derive(Debug, Deserialize)]
struct CargoMessage {
    reason: String,
    message: CompilerMessage,
}

#[derive(Debug, Deserialize)]
struct CompilerMessage {
    code: Option<MessageCode>,
    message: String,
    spans: Vec<MessageSpan>,
}

#[derive(Debug, Deserialize)]
struct MessageCode {
    code: String,
}

#[derive(Debug, Deserialize)]
struct MessageSpan {
    file_name: String,
    line_start: usize,
    column_start: usize,
}

impl MetricProvider for DeadCodeProvider {
    fn name(&self) -> &'static str {
        "dead_code"
    }

    fn collect(&self, crate_root: &Path) -> Result<ProviderOutput, MetricError> {
        let output = Command::new("cargo")
            .arg("check")
            .arg("--message-format=json")
            .env("RUSTFLAGS", &self.rustflags)
            .current_dir(crate_root)
            .output()
            .map_err(|e| MetricError::CommandFailed {
                command: "cargo check".to_string(),
                source: e,
            })?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        if !output.status.success() && stderr.contains("error") {
            return Err(MetricError::CommandExit {
                command: "cargo check".to_string(),
                status: output.status.code().unwrap_or(-1),
                stderr: stderr.to_string(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let items = self.parse_messages(&stdout, crate_root.to_path_buf());
        Ok(ProviderOutput::DeadCode(items))
    }
}
