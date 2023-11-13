use anyhow::Result;
use clap::{Parser, Subcommand};
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
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_names(true)
        .init();

    let cli = Cli::parse();

    match cli.command {
        CliCommand::Build => amardiscord::db::build().await?,
        CliCommand::Serve => amardiscord::serve::serve().await?,
    }

    Ok(())
}
