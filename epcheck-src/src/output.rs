use crate::analyzer::{AnalysisResult, EndpointStatus};
use crate::cli::{Cli, OutputFormat};
use std::io::{self, Write};

/// Output formatter for analysis results
pub struct OutputFormatter {
    format: OutputFormat,
}

impl OutputFormatter {
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    /// Output the analysis results
    pub fn output(&self, results: AnalysisResult, cli: &Cli) -> anyhow::Result<()> {
        match self.format {
            OutputFormat::Table => self.output_table(results, cli),
            OutputFormat::Csv => self.output_csv(results),
            OutputFormat::Json => self.output_json(results, cli),
        }
    }

    fn output_table(&self, results: AnalysisResult, cli: &Cli) -> anyhow::Result<()> {
        println!("\n{}", "=".repeat(80));
        println!("OpenAPI Endpoint Usage Report");
        println!("Generated on {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
        println!("API Spec: {}", cli.spec.display());
        println!("Search Dir: {}", cli.dir.display());
        println!("Files scanned: {}", results.total_files_scanned);
        println!("Scan time: {}ms", results.scan_time_ms);
        println!("{}", "=".repeat(80));

        println!("\n{:30} {:8} {:8} {:8} {}",
                 "Endpoint", "Methods", "Status", "Count", "Files");
        println!("{}", "-".repeat(100));

        let mut used_count = 0;
        let mut total_count = 0;

        for result in &results.endpoints {
            total_count += 1;
            if result.status == EndpointStatus::Used {
                used_count += 1;
            }

            let status = match result.status {
                EndpointStatus::Used => "✓ USED",
                EndpointStatus::Unused => "✗ UNUSED",
            };

            let files_str = if result.files.is_empty() {
                "-".to_string()
            } else {
                result.files.iter()
                    .map(|f| std::path::Path::new(f).file_name()
                         .and_then(|n| n.to_str())
                         .unwrap_or(f))
                    .collect::<Vec<_>>()
                    .join(", ")
            };

            println!("{:30} {:8} {:8} {:8} {}",
                     result.endpoint.path,
                     result.endpoint.method.as_str(),
                     status,
                     result.usage_count,
                     files_str);
        }

        println!("\nSummary:");
        println!("  Total endpoints: {}", total_count);
        println!("  Used: {}", used_count);
        println!("  Unused: {}", total_count - used_count);
        if total_count > 0 {
            println!("  Coverage: {:.1}%", (used_count as f64 / total_count as f64) * 100.0);
        }

        Ok(())
    }

    fn output_csv(&self, results: AnalysisResult) -> anyhow::Result<()> {
        println!("Endpoint,Method,Status,Usage Count,Files");

        for result in &results.endpoints {
            let status = match result.status {
                EndpointStatus::Used => "USED",
                EndpointStatus::Unused => "UNUSED",
            };

            let files = result.files.join(";");

            println!("\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"",
                     result.endpoint.path,
                     result.endpoint.method.as_str(),
                     status,
                     result.usage_count,
                     files);
        }

        Ok(())
    }

    fn output_json(&self, results: AnalysisResult, cli: &Cli) -> anyhow::Result<()> {
        use serde_json::json;

        let endpoints: Vec<serde_json::Value> = results.endpoints
            .iter()
            .map(|result| {
                let status = match result.status {
                    EndpointStatus::Used => "used",
                    EndpointStatus::Unused => "unused",
                };

                json!({
                    "endpoint": result.endpoint.path,
                    "method": result.endpoint.method.as_str(),
                    "status": status,
                    "usage_count": result.usage_count,
                    "files": result.files
                })
            })
            .collect();

        let output = json!({
            "report": {
                "generated": chrono::Utc::now().to_rfc3339(),
                "api_spec": cli.spec.to_string_lossy(),
                "search_dir": cli.dir.to_string_lossy(),
                "files_scanned": results.total_files_scanned,
                "scan_time_ms": results.scan_time_ms
            },
            "endpoints": endpoints
        });

        println!("{}", serde_json::to_string_pretty(&output)?);
        Ok(())
    }
}