use clap::{Args, Subcommand};

use crate::{
    client::HaClient,
    error::CliError,
    output::{OutputFormat, print_output},
};

/// Arguments for the `calendar` subcommand.
#[derive(Debug, Args)]
pub struct CalendarCommand {
    #[command(subcommand)]
    pub action: CalendarAction,
}

/// Actions available under `hacli calendar`.
#[derive(Debug, Subcommand)]
pub enum CalendarAction {
    /// List all calendar entities.
    List,

    /// Fetch events from a calendar entity within a date range.
    ///
    /// # Example
    ///
    /// ```text
    /// hacli calendar events calendar.my_calendar \
    ///     --start 2024-01-01T00:00:00Z \
    ///     --end   2024-01-31T23:59:59Z
    /// ```
    Events {
        /// Calendar entity ID (e.g. `calendar.my_calendar`).
        calendar_id: String,
        /// Range start as an ISO 8601 timestamp (exclusive).
        #[arg(long)]
        start: String,
        /// Range end as an ISO 8601 timestamp (exclusive).
        #[arg(long)]
        end: String,
    },
}

/// Executes a `calendar` subcommand action.
///
/// # Errors
///
/// Propagates [`CliError`] from the HTTP client or output formatter.
pub async fn run(
    cmd: CalendarCommand,
    client: &HaClient,
    output: &OutputFormat,
) -> Result<(), CliError> {
    match cmd.action {
        CalendarAction::List => {
            let value = client.get_json("/calendars").await?;
            print_output(&value, output)
        }
        CalendarAction::Events {
            calendar_id,
            start,
            end,
        } => {
            let path = format!("/calendars/{calendar_id}");
            let params = [("start", start.as_str()), ("end", end.as_str())];
            let value = client.get_json_with_params(&path, &params).await?;
            print_output(&value, output)
        }
    }
}
