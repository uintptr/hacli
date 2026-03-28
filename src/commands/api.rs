use clap::{Args, Subcommand};

use crate::{
    client::HaClient,
    error::CliError,
    output::{OutputFormat, print_output, print_text},
};

/// Arguments for the `api` subcommand.
#[derive(Debug, Args)]
pub struct ApiCommand {
    #[command(subcommand)]
    pub action: ApiAction,
}

/// Actions available under `hacli api`.
#[derive(Debug, Subcommand)]
pub enum ApiAction {
    /// Verify the API is accessible and the token is valid.
    Ping,
    /// Show the current Home Assistant configuration (components, location, version, etc.).
    Config,
    /// Print the session error log (plain text).
    ErrorLog,
}

/// Executes an `api` subcommand action.
///
/// # Errors
///
/// Propagates [`CliError`] from the HTTP client or output formatter.
pub async fn run(
    cmd: ApiCommand,
    client: &HaClient,
    output: &OutputFormat,
) -> Result<(), CliError> {
    match cmd.action {
        ApiAction::Ping => {
            let value = client.get_json("/").await?;
            print_output(&value, output)
        }
        ApiAction::Config => {
            let value = client.get_json("/config").await?;
            print_output(&value, output)
        }
        ApiAction::ErrorLog => {
            let text = client.get_text("/error_log").await?;
            print_text(&text);
            Ok(())
        }
    }
}
