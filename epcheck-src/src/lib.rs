pub mod cli;
pub mod openapi;
pub mod scanner;
pub mod analyzer;
pub mod output;

pub use crate::cli::Cli;
use crate::analyzer::EndpointAnalyzer;
use crate::output::OutputFormatter;
use anyhow::Result;

/// Main entry point for the epcheck application
pub async fn run(cli: Cli) -> Result<()> {
    // Determine spec path
    let spec_path = match &cli.spec {
        Some(s) => s.clone(),
        None => cli::find_openapi_spec().ok_or_else(|| anyhow::anyhow!("No OpenAPI spec provided and none found in current or parent directories"))?,
    };

    // Load and parse OpenAPI specification
    let spec = cli::load_openapi_spec(&spec_path).await?;

    // Create analyzer
    let analyzer = EndpointAnalyzer::new(spec, cli.clone());

    // Scan directory for endpoint usage
    let results = analyzer.analyze_directory(&cli.dir).await?;

    // Format and output results
    let formatter = OutputFormatter::new(cli.format);
    formatter.output(results, &cli)?;

    Ok(())
}