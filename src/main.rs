use std::path::{Path, PathBuf};

use clap::Parser;
use tracing::{error, info};
use tracing_subscriber::filter::LevelFilter;

#[derive(Parser)]
#[clap(name = "amardiscord")]
struct Cli {
    /// Path to the Discord backup directory (default: `./data`).
    path: Option<PathBuf>,
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

    let Cli { path } = Cli::parse();

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
}
