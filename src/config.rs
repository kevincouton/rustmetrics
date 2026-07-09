use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageProviderConfig {
    pub enabled: bool,
    #[serde(default = "default_coverage_command")]
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeProviderConfig {
    pub enabled: bool,
    #[serde(default = "default_dead_code_rustflags")]
    pub rustflags: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSizeProviderConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvidersConfig {
    #[serde(default = "default_coverage")]
    pub coverage: CoverageProviderConfig,
    #[serde(default = "default_dead_code")]
    pub dead_code: DeadCodeProviderConfig,
    #[serde(default = "default_function_size")]
    pub function_size: FunctionSizeProviderConfig,
}

fn default_coverage_command() -> String {
    "cargo-llvm-cov".to_string()
}

fn default_dead_code_rustflags() -> String {
    "-W dead_code".to_string()
}

fn default_coverage() -> CoverageProviderConfig {
    CoverageProviderConfig {
        enabled: true,
        command: default_coverage_command(),
    }
}

fn default_dead_code() -> DeadCodeProviderConfig {
    DeadCodeProviderConfig {
        enabled: true,
        rustflags: default_dead_code_rustflags(),
    }
}

fn default_function_size() -> FunctionSizeProviderConfig {
    FunctionSizeProviderConfig { enabled: true }
}

impl Default for ProvidersConfig {
    fn default() -> Self {
        Self {
            coverage: default_coverage(),
            dead_code: default_dead_code(),
            function_size: default_function_size(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thresholds {
    pub min_line_coverage: Option<f64>,
    pub max_function_lines: Option<usize>,
    pub max_dead_code_items: Option<usize>,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            min_line_coverage: Some(0.80),
            max_function_lines: Some(100),
            max_dead_code_items: Some(0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub providers: ProvidersConfig,
    #[serde(default)]
    pub thresholds: Thresholds,
}

impl Config {
    pub fn load(path: Option<&Path>) -> Result<Self> {
        if let Some(path) = path {
            let content = std::fs::read_to_string(path)
                .with_context(|| format!("failed to read config from {}", path.display()))?;
            let config: Config = toml::from_str(&content)
                .with_context(|| format!("failed to parse config from {}", path.display()))?;
            return Ok(config);
        }

        for name in [".rustmetrics.toml", "rustmetrics.toml"] {
            let candidate = PathBuf::from(name);
            if candidate.exists() {
                return Self::load(Some(&candidate));
            }
        }

        Ok(Config::default())
    }
}
