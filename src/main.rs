//! Canopy CLI entry point

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod commands;

#[derive(Parser)]
#[command(name = "canopy")]
#[command(about = "Live hierarchical code architecture visualization", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Repository root path (defaults to current directory)
    #[arg(short, long, default_value = ".")]
    root: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the visualization server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "7890")]
        port: u16,

        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Open browser automatically
        #[arg(short, long)]
        open: bool,
    },
    /// Index the repository and exit
    Index,
    /// Clear the cache
    Clear,
    /// Show version
    Version,
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
    tracing::info!("Repository root: {}", cli.root.display());

    match cli.command {
        Commands::Serve { port, host, open } => {
            commands::serve(cli.root, host, port, open).await
        }
        Commands::Index => {
            commands::index(cli.root).await
        }
        Commands::Clear => {
            commands::clear(cli.root)
        }
        Commands::Version => {
            println!("Canopy v{}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    }
}
