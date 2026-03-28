use clap::{Args, Subcommand};

use crate::{
    client::HaClient,
    error::CliError,
    output::{OutputFormat, print_output},
    parse::parse_fields_to_object,
};

/// Arguments for the `service` subcommand.
#[derive(Debug, Args)]
pub struct ServiceCommand {
    #[command(subcommand)]
    pub action: ServiceAction,
}

/// Actions available under `hacli service`.
#[derive(Debug, Subcommand)]
pub enum ServiceAction {
    /// List all available services, optionally filtered to a single domain.
    List {
        /// Only show services belonging to this domain (e.g. `light`, `switch`).
        domain: Option<String>,
    },

    /// Call a service.
    ///
    /// # Example
    ///
    /// ```text
    /// hacli service call light turn_on --field entity_id=light.living_room --field brightness=200
    /// ```
    Call {
        /// Service domain (e.g. `light`).
        domain: String,
        /// Service name (e.g. `turn_on`).
        service: String,
        /// Service data field as `KEY=VALUE` (may be specified multiple times).
        ///
        /// Values are parsed as JSON first; plain strings are used as a fallback.
        #[arg(long = "field", value_name = "KEY=VALUE")]
        fields: Vec<String>,
        /// Include the service's return value in the output.
        #[arg(long)]
        return_response: bool,
    },
}

/// Executes a `service` subcommand action.
///
/// # Errors
///
/// Propagates [`CliError`] from the HTTP client, field parsing, or output formatter.
pub async fn run(
    cmd: ServiceCommand,
    client: &HaClient,
    output: &OutputFormat,
) -> Result<(), CliError> {
    match cmd.action {
        ServiceAction::List { domain } => {
            let value = client.get_json("/services").await?;

            if let Some(filter) = domain {
                // Filter the returned array to the requested domain.
                let filtered = match &value {
                    serde_json::Value::Array(services) => {
                        let matches: Vec<_> = services
                            .iter()
                            .filter(|s| {
                                s.get("domain")
                                    .and_then(|d| d.as_str())
                                    .is_some_and(|d| d == filter)
                            })
                            .cloned()
                            .collect();
                        serde_json::Value::Array(matches)
                    }
                    other => other.clone(),
                };
                return print_output(&filtered, output);
            }

            print_output(&value, output)
        }
        ServiceAction::Call {
            domain,
            service,
            fields,
            return_response,
        } => {
            let data = parse_fields_to_object(&fields).map_err(CliError::Config)?;
            let path = format!("/services/{domain}/{service}");

            let value = if return_response {
                client
                    .post_json_with_params(&path, &data, &[("return_response", "")])
                    .await?
            } else {
                client.post_json(&path, &data).await?
            };

            print_output(&value, output)
        }
    }
}
