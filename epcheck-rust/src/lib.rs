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
    // Load and parse OpenAPI specification
    let spec = cli::load_openapi_spec(&cli.spec)?;

    // Create analyzer
    let analyzer = EndpointAnalyzer::new(spec, cli.clone());

    // Scan directory for endpoint usage
    let results = analyzer.analyze_directory(&cli.dir).await?;

    // Format and output results
    let formatter = OutputFormatter::new(cli.format);
    formatter.output(results, &cli)?;

    Ok(())
}