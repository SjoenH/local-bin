use clap::{ArgEnum, Parser};
use std::path::PathBuf;

/// Fast OpenAPI endpoint usage checker
#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Path or URL to OpenAPI specification file (JSON or YAML). If not provided, searches for common spec files in current and parent directories.
    #[clap(short, long, value_name = "SPEC")]
    pub spec: Option<String>,

    /// Directory to search for endpoint usage
    #[clap(short, long, value_name = "DIR", default_value = ".")]
    pub dir: PathBuf,

    /// Output format
    #[clap(short, long, arg_enum, default_value_t = OutputFormat::Table)]
    pub format: OutputFormat,

    /// Filter endpoints by regex pattern
    #[clap(short, long, value_name = "PATTERN")]
    pub pattern: Option<String>,

    /// Show only unused endpoints
    #[clap(long)]
    pub unused_only: bool,

    /// Show detailed file information
    #[clap(short, long)]
    pub verbose: bool,

    /// Interactive mode with fuzzy search
    #[clap(short, long)]
    pub interactive: bool,

    /// Quick mode (skip detailed analysis)
    #[clap(short, long)]
    pub quick: bool,

    /// Truncate long file lists
    #[clap(long)]
    pub truncate: bool,

    /// Disable colored output
    #[clap(long)]
    pub no_colors: bool,

    /// Files to exclude from search
    #[clap(short, long, value_name = "FILE")]
    pub exclude: Vec<String>,
}

/// Supported output formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, ArgEnum)]
pub enum OutputFormat {
    Table,
    Csv,
    Json,
    Markdown,
}

/// Load and parse OpenAPI specification from file or URL
pub async fn load_openapi_spec(spec_path: &str) -> anyhow::Result<crate::openapi::OpenApiSpec> {
    let content = if spec_path.starts_with("http://") || spec_path.starts_with("https://") {
        // Fetch from URL using curl
        let output = tokio::process::Command::new("curl")
            .arg("-s")
            .arg(spec_path)
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch URL: {}. Make sure curl is installed.", e))?;
        String::from_utf8(output.stdout)?
    } else {
        // Read from file
        std::fs::read_to_string(spec_path)?
    };

    // Determine format from file extension or content
    let extension = std::path::Path::new(spec_path)
        .extension()
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

/// Find the closest OpenAPI specification file by searching common names in current and parent directories
pub fn find_openapi_spec() -> Option<String> {
    let current = std::env::current_dir().ok()?;
    let names = [
        "openapi.json",
        "openapi.yaml",
        "openapi.yml",
        "swagger.json",
        "swagger.yaml",
        "swagger.yml",
    ];

    let mut dir = current.as_path();
    loop {
        for name in &names {
            let path = dir.join(name);
            if path.exists() {
                return Some(path.to_string_lossy().to_string());
            }
        }
        if let Some(parent) = dir.parent() {
            dir = parent;
        } else {
            break;
        }
    }
    None
}