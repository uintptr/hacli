mod client;
mod commands;
mod config;
mod error;
mod output;
mod parse;

use clap::Parser;

use commands::Command;
use output::OutputFormat;

/// Home Assistant CLI
///
/// Interact with a Home Assistant instance over its REST API.
/// Credentials can be supplied via flags, environment variables, or a config file.
///
/// Run `hacli config init` to create the config file interactively.
#[derive(Debug, Parser)]
#[command(name = "hacli", version, author, propagate_version = true)]
struct Cli {
    /// Home Assistant base URL (e.g. `http://homeassistant.local:8123`).
    ///
    /// Overrides `HA_URL` and the config file value.
    #[arg(long, global = true, env = "HA_URL")]
    url: Option<String>,

    /// Long-lived access token.
    ///
    /// Overrides `HA_TOKEN` and the config file value.
    #[arg(long, global = true, env = "HA_TOKEN")]
    token: Option<String>,

    /// Output format.
    #[arg(long, short = 'o', global = true, default_value = "json", value_enum)]
    output: OutputFormat,

    #[command(subcommand)]
    command: Command,
}

#[tokio::main]
async fn main() {
    // Load a .env file if one exists in the working directory.
    // Errors (e.g. file not found) are silently ignored.
    dotenvy::dotenv().ok();

    // Initialise tracing from the RUST_LOG environment variable.
    // Defaults to showing only errors if RUST_LOG is unset.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("error")),
        )
        .init();

    let cli = Cli::parse();

    if let Err(e) = commands::run_command(cli.command, cli.url, cli.token, cli.output).await {
        tracing::error!("{e}");
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
