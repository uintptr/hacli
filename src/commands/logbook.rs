use clap::Args;

use crate::{
    client::HaClient,
    error::CliError,
    output::{OutputFormat, print_output},
};

/// Arguments for the `logbook` command.
///
/// Queries `GET /api/logbook/<timestamp>`.
#[derive(Debug, Args)]
pub struct LogbookCommand {
    /// Filter results to a specific entity ID.
    #[arg(long)]
    pub entity_id: Option<String>,

    /// Start of the logbook window as an ISO 8601 timestamp.  Defaults to the
    /// beginning of the current day when omitted.
    #[arg(long)]
    pub from: Option<String>,

    /// End of the logbook window as an ISO 8601 timestamp.
    #[arg(long)]
    pub to: Option<String>,
}

/// Executes the `logbook` command.
///
/// # Errors
///
/// Propagates [`CliError`] from the HTTP client or output formatter.
pub async fn run(
    cmd: LogbookCommand,
    client: &HaClient,
    output: &OutputFormat,
) -> Result<(), CliError> {
    let path = match &cmd.from {
        Some(ts) => format!("/logbook/{ts}"),
        None => "/logbook".to_string(),
    };

    let mut params: Vec<(&str, &str)> = Vec::new();

    if let Some(entity_id) = &cmd.entity_id {
        params.push(("entity", entity_id.as_str()));
    }
    if let Some(end_time) = &cmd.to {
        params.push(("end_time", end_time.as_str()));
    }

    let value = client.get_json_with_params(&path, &params).await?;
    print_output(&value, output)
}
