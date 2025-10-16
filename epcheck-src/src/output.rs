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
            OutputFormat::Markdown => self.output_markdown(results, cli),
        }
    }

    fn output_table(&self, results: AnalysisResult, cli: &Cli) -> anyhow::Result<()> {
        println!("\n{}", "=".repeat(80));
        println!("OpenAPI Endpoint Usage Report");
        println!("Generated on {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
        println!("API Spec: {}", cli.spec);
        println!("Search Dir: {}", cli.dir.display());

        // Show exclusions if any
        if !cli.exclude.is_empty() {
            println!("Excluding: {}", cli.exclude.join(", "));
        }

        // Show search mode
        println!("Search: ripgrep (fast)");

        // Show mode
        if cli.truncate {
            println!("Mode: Truncated file lists");
        } else {
            println!("Mode: Full file lists (use --truncate to limit)");
        }

        // Show filter if unused_only is enabled
        if cli.unused_only {
            println!("Filter: Unused endpoints only");
        }

        println!("{}", "=".repeat(80));

        // Calculate dynamic column widths
        let mut max_endpoint_len = 7; // "Endpoint" header
        let mut max_method_len = 7;  // "Methods" header
        let mut max_status_len = 6;  // "Status" header
        let mut max_count_len = 5;   // "Count" header

        for result in &results.endpoints {
            max_endpoint_len = max_endpoint_len.max(result.endpoint.path.len());
            max_method_len = max_method_len.max(result.endpoint.method.as_str().len());
            max_status_len = max_status_len.max(match result.status {
                EndpointStatus::Used => 6, // "✓ USED"
                EndpointStatus::Unused => 8, // "✗ UNUSED"
            });
            max_count_len = max_count_len.max(result.usage_count.to_string().len());
        }

        // Ensure minimum widths
        max_endpoint_len = max_endpoint_len.max(7);
        max_method_len = max_method_len.max(7);
        max_status_len = max_status_len.max(6);
        max_count_len = max_count_len.max(5);

        // Print table header
        println!("\n{:endpoint_width$} {:method_width$} {:status_width$} {:count_width$} Files",
                 "Endpoint", "Methods", "Status", "Count",
                 endpoint_width = max_endpoint_len,
                 method_width = max_method_len,
                 status_width = max_status_len,
                 count_width = max_count_len);
        println!("{}", "-".repeat(max_endpoint_len + max_method_len + max_status_len + max_count_len + 6)); // +6 for spaces between columns

        let mut used_count = 0;
        let mut total_count = 0;
        let mut total_file_refs = 0;

        for result in &results.endpoints {
            total_count += 1;
            total_file_refs += result.usage_count;

            if result.status == EndpointStatus::Used {
                used_count += 1;
            }

            let status = match result.status {
                EndpointStatus::Used => "✓ USED",
                EndpointStatus::Unused => "✗ UNUSED",
            };

            let files_str = if result.files.is_empty() {
                "-".to_string()
            } else if cli.truncate && result.files.len() > 3 {
                format!("{} files (truncated)", result.files.len())
            } else {
                result.files.iter()
                    .map(|f| std::path::Path::new(f).file_name()
                         .and_then(|n| n.to_str())
                         .unwrap_or(f))
                    .collect::<Vec<_>>()
                    .join(", ")
            };

            println!("{:endpoint_width$} {:method_width$} {:status_width$} {:count_width$} {}",
                     result.endpoint.path,
                     result.endpoint.method.as_str(),
                     status,
                     result.usage_count,
                     files_str,
                     endpoint_width = max_endpoint_len,
                     method_width = max_method_len,
                     status_width = max_status_len,
                     count_width = max_count_len);
        }

        println!("\nSummary:");
        println!("  Total endpoints: {}", total_count);
        println!("  Used: {}", used_count);
        println!("  Unused: {}", total_count - used_count);
        if total_count > 0 {
            println!("  Coverage: {:.1}%", (used_count as f64 / total_count as f64) * 100.0);
        }
        println!("  Total file references: {}", total_file_refs);

        // Detailed file references section
        let multi_usage_endpoints: Vec<_> = results.endpoints.iter()
            .filter(|r| r.usage_count >= 2)
            .collect();

        if !multi_usage_endpoints.is_empty() {
            println!("\nDetailed File References (for endpoints with 2+ usages):");
            for result in multi_usage_endpoints {
                println!("  {} {}: {} files",
                         result.endpoint.method.as_str(),
                         result.endpoint.path,
                         result.usage_count);
                for file in &result.files {
                    let filename = std::path::Path::new(file).file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(file);
                    println!("    - {}", filename);
                }
            }
        } else {
            let message = if cli.unused_only {
                "No unused endpoints have multiple file references."
            } else {
                "No endpoints with 2 or more file references found."
            };
            println!("\nDetailed File References (for endpoints with 2+ usages):");
            println!("  {}", message);
        }

        println!("\nNote: This script searches for endpoint usage in multiple patterns:");
        println!("      1. Exact string matches: \"{}\"", "");
        println!("      2. Method calls: .GET(\"\"), .POST(\"\"), etc.");
        println!("      3. Path parameters: {{id}} matches actual values like 123, abc, etc.");
        println!("      The API spec file is automatically excluded from the search results.");

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
                "api_spec": cli.spec,
                "search_dir": cli.dir.to_string_lossy(),
                "files_scanned": results.total_files_scanned,
                "scan_time_ms": results.scan_time_ms
            },
            "endpoints": endpoints
        });

        println!("{}", serde_json::to_string_pretty(&output)?);
        Ok(())
    }

    fn output_markdown(&self, results: AnalysisResult, cli: &Cli) -> anyhow::Result<()> {
        // Print table header
        println!("| Endpoint | Methods | Status | Count | Files |");
        println!("|----------|---------|--------|-------|-------|");

        for result in &results.endpoints {
            let status = match result.status {
                EndpointStatus::Used => "✓ USED",
                EndpointStatus::Unused => "✗ UNUSED",
            };

            let files_str = if result.files.is_empty() {
                "-".to_string()
            } else if cli.truncate && result.files.len() > 3 {
                format!("{} files (truncated)", result.files.len())
            } else {
                result.files.iter()
                    .map(|f| std::path::Path::new(f).file_name()
                         .and_then(|n| n.to_str())
                         .unwrap_or(f))
                    .collect::<Vec<_>>()
                    .join(", ")
            };

            println!("| {} | {} | {} | {} | {} |",
                     result.endpoint.path,
                     result.endpoint.method.as_str(),
                     status,
                     result.usage_count,
                     files_str);
        }

        Ok(())
    }
}