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
        self.parse_output(jsonl, _crate_root).0
    }

    fn parse_output(&self, jsonl: &str, _crate_root: PathBuf) -> (Vec<DeadCodeItem>, bool) {
        let mut items = Vec::new();
        let mut has_error = false;

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

            if message.message.level == "error" {
                has_error = true;
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

        (items, has_error)
    }

    fn rustflags_env(&self) -> String {
        match std::env::var("RUSTFLAGS") {
            Ok(existing) if !existing.trim().is_empty() => {
                format!("{} {}", existing.trim(), self.rustflags)
            }
            _ => self.rustflags.clone(),
        }
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
    level: String,
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
            .env("RUSTFLAGS", self.rustflags_env())
            .current_dir(crate_root)
            .output()
            .map_err(|e| MetricError::CommandFailed {
                command: "cargo check".to_string(),
                source: e,
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let (items, has_error) = self.parse_output(&stdout, crate_root.to_path_buf());

        if has_error || (!output.status.success() && items.is_empty()) {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(MetricError::CommandExit {
                command: "cargo check".to_string(),
                status: output.status.code().unwrap_or(-1),
                stderr: stderr.to_string(),
            });
        }

        Ok(ProviderOutput::DeadCode(items))
    }
}
