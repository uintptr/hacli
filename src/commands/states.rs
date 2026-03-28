use clap::{Args, Subcommand};
use serde_json::json;

use crate::{
    client::HaClient,
    error::CliError,
    output::{OutputFormat, print_output},
    parse::parse_fields_to_object,
};

/// Arguments for the `state` subcommand.
#[derive(Debug, Args)]
pub struct StateCommand {
    #[command(subcommand)]
    pub action: StateAction,
}

/// Actions available under `hacli state`.
#[derive(Debug, Subcommand)]
pub enum StateAction {
    /// List all entity states.
    List,

    /// Get the current state of a specific entity.
    Get {
        /// Entity ID, e.g. `sensor.living_room_temperature`.
        entity_id: String,
    },

    /// Create or update an entity's state.
    Set {
        /// Entity ID to create or update.
        entity_id: String,

        /// New state value (e.g. `on`, `23.5`, `unavailable`).
        #[arg(long)]
        state: String,

        /// Attribute as `KEY=VALUE` (may be specified multiple times).
        ///
        /// Values are parsed as JSON first; plain strings are used as a fallback.
        /// Example: `--attr brightness=200 --attr color_temp=370`
        #[arg(long = "attr", value_name = "KEY=VALUE")]
        attrs: Vec<String>,
    },

    /// Remove an entity from Home Assistant.
    Delete {
        /// Entity ID to remove.
        entity_id: String,
    },
}

/// Executes a `state` subcommand action.
///
/// # Errors
///
/// Propagates [`CliError`] from the HTTP client, attribute parsing, or output formatter.
pub async fn run(
    cmd: StateCommand,
    client: &HaClient,
    output: &OutputFormat,
) -> Result<(), CliError> {
    match cmd.action {
        StateAction::List => {
            let value = client.get_json("/states").await?;
            print_output(&value, output)
        }
        StateAction::Get { entity_id } => {
            let value = client.get_json(&format!("/states/{entity_id}")).await?;
            print_output(&value, output)
        }
        StateAction::Set {
            entity_id,
            state,
            attrs,
        } => {
            let attributes = parse_fields_to_object(&attrs).map_err(CliError::Config)?;

            let body = json!({
                "state": state,
                "attributes": attributes,
            });

            let value = client
                .post_json(&format!("/states/{entity_id}"), &body)
                .await?;
            print_output(&value, output)
        }
        StateAction::Delete { entity_id } => {
            let value = client.delete(&format!("/states/{entity_id}")).await?;
            print_output(&value, output)
        }
    }
}
