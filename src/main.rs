//! Canopy CLI entry point - Simplified version that just serves visualization

use clap::Parser;
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod commands;

#[derive(Parser)]
#[command(name = "canopy")]
#[command(about = "Live hierarchical code architecture visualization", long_about = None)]
struct Cli {
    /// Repository root path (defaults to current directory)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Port to listen on
    #[arg(short, long, default_value = "7890")]
    port: u16,

    /// Host to bind to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(format!("canopy={}", log_level)))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Canopy v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("Analyzing: {}", cli.path.display());
    tracing::info!("Server will run on {}:{}", cli.host, cli.port);
    
    // Simply serve the visualization
    commands::serve(cli.path, cli.host, cli.port, false).await
}