use clap::Parser;
use epcheck::{run, Cli};
use std::process;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    if let Err(e) = run(cli).await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }

    Ok(())
}