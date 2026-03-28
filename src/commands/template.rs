use clap::Args;
use serde_json::json;

use crate::{
    client::HaClient,
    error::CliError,
    output::{OutputFormat, print_text},
};

/// Arguments for the `template` command.
///
/// Sends `POST /api/template` with the supplied Jinja2 template string
/// and prints the rendered result.
#[derive(Debug, Args)]
pub struct TemplateCommand {
    /// Jinja2 template string to render.
    ///
    /// # Example
    ///
    /// ```text
    /// hacli template "The sun is {{ states('sun.sun') }}"
    /// ```
    pub template: String,
}

/// Executes the `template` command.
///
/// The `--output` format is intentionally ignored because `POST /api/template`
/// returns a rendered plain-text string, not JSON.
///
/// # Errors
///
/// Propagates [`CliError`] from the HTTP client.
pub async fn run(
    cmd: TemplateCommand,
    client: &HaClient,
    _output: &OutputFormat,
) -> Result<(), CliError> {
    let body = json!({ "template": cmd.template });
    let rendered = client.post_text("/template", &body).await?;
    print_text(&rendered);
    Ok(())
}
