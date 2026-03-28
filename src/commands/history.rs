use clap::Args;

use crate::{
    client::HaClient,
    error::CliError,
    output::{OutputFormat, print_output},
};

/// Arguments for the `history` command.
///
/// Queries `GET /api/history/period/<timestamp>`.
#[derive(Debug, Args)]
pub struct HistoryCommand {
    /// Filter results to a specific entity ID (e.g. `sensor.temperature`).
    #[arg(long)]
    pub entity_id: Option<String>,

    /// Start of the history window as an ISO 8601 timestamp
    /// (e.g. `2024-01-01T00:00:00+00:00`).  Defaults to 1 day ago when omitted.
    #[arg(long)]
    pub from: Option<String>,

    /// End of the history window as an ISO 8601 timestamp.
    #[arg(long)]
    pub to: Option<String>,

    /// Return a minimal response (only `state` and `last_changed` fields).
    #[arg(long)]
    pub minimal: bool,

    /// Exclude attributes from the response to reduce payload size.
    #[arg(long)]
    pub no_attributes: bool,

    /// Only include entries where the state value actually changed.
    #[arg(long)]
    pub significant_changes_only: bool,
}

/// Executes the `history` command.
///
/// # Errors
///
/// Propagates [`CliError`] from the HTTP client or output formatter.
pub async fn run(
    cmd: HistoryCommand,
    client: &HaClient,
    output: &OutputFormat,
) -> Result<(), CliError> {
    // The path timestamp segment is optional in the HA API.
    let path = match &cmd.from {
        Some(ts) => format!("/history/period/{ts}"),
        None => "/history/period".to_string(),
    };

    // Build query parameters from optional flags.
    // `reqwest` accepts `&[(&str, &str)]` — parameters with empty values still
    // appear in the URL (e.g. `?minimal_response=`) which HA accepts as a flag.
    let mut params: Vec<(&str, &str)> = Vec::new();

    if let Some(entity_id) = &cmd.entity_id {
        params.push(("filter_entity_id", entity_id.as_str()));
    }
    if let Some(end_time) = &cmd.to {
        params.push(("end_time", end_time.as_str()));
    }
    if cmd.minimal {
        params.push(("minimal_response", ""));
    }
    if cmd.no_attributes {
        params.push(("no_attributes", ""));
    }
    if cmd.significant_changes_only {
        params.push(("significant_changes_only", ""));
    }

    let value = client.get_json_with_params(&path, &params).await?;
    print_output(&value, output)
}
