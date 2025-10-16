use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Fast OpenAPI endpoint usage checker
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to OpenAPI specification file (JSON or YAML)
    #[arg(short, long, value_name = "FILE")]
    pub spec: PathBuf,

    /// Directory to search for endpoint usage
    #[arg(short, long, value_name = "DIR")]
    pub dir: PathBuf,

    /// Output format
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Table)]
    pub format: OutputFormat,

    /// Filter endpoints by regex pattern
    #[arg(short, long, value_name = "PATTERN")]
    pub pattern: Option<String>,

    /// Show only unused endpoints
    #[arg(long)]
    pub unused_only: bool,

    /// Show detailed file information
    #[arg(short, long)]
    pub verbose: bool,

    /// Interactive mode with fuzzy search
    #[arg(short, long)]
    pub interactive: bool,

    /// Quick mode (skip detailed analysis)
    #[arg(short, long)]
    pub quick: bool,

    /// Truncate long file lists
    #[arg(long)]
    pub truncate: bool,

    /// Disable colored output
    #[arg(long)]
    pub no_colors: bool,

    /// Files to exclude from search
    #[arg(short, long, value_name = "FILE")]
    pub exclude: Vec<String>,
}

/// Supported output formats
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Table,
    Csv,
    Json,
    Markdown,
}

/// Load and parse OpenAPI specification
pub fn load_openapi_spec(path: &PathBuf) -> anyhow::Result<crate::openapi::OpenApiSpec> {
    let content = std::fs::read_to_string(path)?;
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    match extension {
        "json" => {
            let spec: crate::openapi::OpenApiSpec = serde_json::from_str(&content)?;
            Ok(spec)
        }
        "yaml" | "yml" => {
            let spec: crate::openapi::OpenApiSpec = serde_yaml::from_str(&content)?;
            Ok(spec)
        }
        _ => {
            // Try JSON first, then YAML
            match serde_json::from_str(&content) {
                Ok(spec) => Ok(spec),
                Err(_) => {
                    let spec: crate::openapi::OpenApiSpec = serde_yaml::from_str(&content)?;
                    Ok(spec)
                }
            }
        }
    }
}