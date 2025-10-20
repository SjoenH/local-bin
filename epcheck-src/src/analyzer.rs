use crate::cli::CheckArgs;
use crate::openapi::{extract_endpoints, Endpoint};
use crate::scanner::{ContentScanner, FileScanner};
use std::path::Path;

/// Analysis result for an endpoint
#[derive(Debug, Clone)]
pub struct EndpointResult {
    pub endpoint: Endpoint,
    pub status: EndpointStatus,
    pub usage_count: usize,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EndpointStatus {
    Used,
    Unused,
}

/// Complete analysis results
#[derive(Debug)]
pub struct AnalysisResult {
    pub endpoints: Vec<EndpointResult>,
    pub total_files_scanned: usize,
    pub scan_time_ms: u128,
}

/// Main endpoint analyzer
pub struct EndpointAnalyzer {
    spec_endpoints: Vec<Endpoint>,
    cli: CheckArgs,
}

impl EndpointAnalyzer {
    pub fn new(spec: crate::openapi::OpenApiSpec, cli: CheckArgs) -> Self {
        let spec_endpoints = extract_endpoints(&spec);
        Self { spec_endpoints, cli }
    }

    /// Analyze a directory for endpoint usage
    pub async fn analyze_directory(&self, dir: &Path) -> anyhow::Result<AnalysisResult> {
        let start_time = std::time::Instant::now();

        // Find all source files
        let scanner = FileScanner::new(self.cli.clone());
        let files = scanner.find_files(dir)?;

        // Create content scanner
        let content_scanner = ContentScanner::new(&self.spec_endpoints)?;

        // Scan files for endpoint usage
        let usage_results = content_scanner.scan_files(files.clone()).await?;

        // Build results
        let mut results = Vec::new();
        for endpoint in &self.spec_endpoints {
            let (_total_matches, files) = usage_results.get(endpoint)
                .map(|(count, files)| (*count, files.clone()))
                .unwrap_or((0, Vec::new()));

            let file_count = files.len();
            let status = if file_count > 0 {
                EndpointStatus::Used
            } else {
                EndpointStatus::Unused
            };

            let result = EndpointResult {
                endpoint: endpoint.clone(),
                status,
                usage_count: file_count, // Number of files, not total matches
                files,
            };

            results.push(result);
        }

        // Filter results based on CLI options
        let mut filtered_results = results;
        if self.cli.unused_only {
            filtered_results.retain(|r| r.status == EndpointStatus::Unused);
        }

        if let Some(pattern) = &self.cli.pattern {
            let regex = regex::Regex::new(pattern)?;
            filtered_results.retain(|r| regex.is_match(&r.endpoint.to_string()));
        }

        // Sort results by endpoint path, then by method
        filtered_results.sort_by(|a, b| {
            match a.endpoint.path.cmp(&b.endpoint.path) {
                std::cmp::Ordering::Equal => a.endpoint.method.as_str().cmp(b.endpoint.method.as_str()),
                other => other,
            }
        });

        let scan_time = start_time.elapsed().as_millis();

        Ok(AnalysisResult {
            endpoints: filtered_results,
            total_files_scanned: files.len(),
            scan_time_ms: scan_time,
        })
    }


}