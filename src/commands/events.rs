use clap::{Args, Subcommand};

use crate::{
    client::HaClient,
    error::CliError,
    output::{OutputFormat, print_output},
    parse::parse_fields_to_object,
};

/// Arguments for the `event` subcommand.
#[derive(Debug, Args)]
pub struct EventCommand {
    #[command(subcommand)]
    pub action: EventAction,
}

/// Actions available under `hacli event`.
#[derive(Debug, Subcommand)]
pub enum EventAction {
    /// List all registered event types and their listener counts.
    List,

    /// Fire a custom event, optionally with data.
    ///
    /// # Example
    ///
    /// ```text
    /// hacli event fire my_custom_event --field key=value
    /// ```
    Fire {
        /// Event type to fire (e.g. `my_custom_event`).
        event_type: String,
        /// Event data field as `KEY=VALUE` (may be specified multiple times).
        ///
        /// Values are parsed as JSON first; plain strings are used as a fallback.
        #[arg(long = "field", value_name = "KEY=VALUE")]
        fields: Vec<String>,
    },
}

/// Executes an `event` subcommand action.
///
/// # Errors
///
/// Propagates [`CliError`] from the HTTP client, field parsing, or output formatter.
pub async fn run(
    cmd: EventCommand,
    client: &HaClient,
    output: &OutputFormat,
) -> Result<(), CliError> {
    match cmd.action {
        EventAction::List => {
            let value = client.get_json("/events").await?;
            print_output(&value, output)
        }
        EventAction::Fire { event_type, fields } => {
            let data = parse_fields_to_object(&fields).map_err(CliError::Config)?;
            let value = client
                .post_json(&format!("/events/{event_type}"), &data)
                .await?;
            print_output(&value, output)
        }
    }
}
