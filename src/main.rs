use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use tracing::{error, info};
use tracing_subscriber::filter::LevelFilter;

#[derive(Parser)]
#[clap(name = "amardiscord")]
struct Cli {
    #[clap(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand)]
enum CliCommand {
    /// Build the message database.
    Build {
        /// Path to the backup directory (default: `./data`).
        path: Option<PathBuf>,
    },
    /// Serve the content.
    Serve {
        /// Path to the backup directory (default: `./data`).
        path: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_names(true)
        .init();

    let cli = Cli::parse();

    match cli.command {
        CliCommand::Build { path } => {
            if let Err(e) = amardiscord::db::build(path).await {
                error!("Building database: {e}");
            }
        },
        CliCommand::Serve { path } => {
            if !Path::new(amardiscord::SQLITE_ARCHIVE_PATH).exists() {
                info!("Database file doesn't exist. Building it.");

                if let Err(e) = amardiscord::db::build(path).await {
                    error!("Building database: {e}");
                    return;
                }
            }

            if let Err(e) = amardiscord::serve::serve().await {
                error!("Server error: {e}");
            }
        },
    }
}
