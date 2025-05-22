use clap::{Parser, Subcommand};
use tracing::error;
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
    Build,
    /// Serve the content.
    Serve,
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
        CliCommand::Build => {
            if let Err(e) = amardiscord::db::build().await {
                error!("Building database: {e}");
            }
        },
        CliCommand::Serve => {
            if let Err(e) = amardiscord::serve::serve().await {
                error!("Server error: {e}");
            }
        },
    }
}
