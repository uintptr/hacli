use crate::{
    client::HaClient,
    error::CliError,
    output::{OutputFormat, print_output},
};

/// Executes `POST /api/config/core/check_config`.
///
/// Asks Home Assistant to validate `configuration.yaml`.
/// Requires the `config` integration to be loaded.
///
/// # Errors
///
/// Propagates [`CliError`] from the HTTP client or output formatter.
pub async fn run(client: &HaClient, output: &OutputFormat) -> Result<(), CliError> {
    let value = client.post_empty("/config/core/check_config").await?;
    print_output(&value, output)
}
