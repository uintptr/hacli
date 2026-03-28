pub mod api;
pub mod calendar;
pub mod check_config;
pub mod config_cmd;
pub mod events;
pub mod history;
pub mod logbook;
pub mod services;
pub mod states;
pub mod template;

use clap::Subcommand;

use crate::{client::HaClient, config::Config, error::CliError, output::OutputFormat};

/// Top-level subcommands for `hacli`.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Initialize or inspect the local configuration file.
    Config(config_cmd::ConfigCommand),

    /// API connectivity and Home Assistant configuration.
    Api(api::ApiCommand),

    /// Entity state management (list, get, set, delete).
    State(states::StateCommand),

    /// Service discovery and invocation.
    Service(services::ServiceCommand),

    /// Event listing and firing.
    Event(events::EventCommand),

    /// Historical state data.
    History(history::HistoryCommand),

    /// Logbook entries.
    Logbook(logbook::LogbookCommand),

    /// Calendar entities and events.
    Calendar(calendar::CalendarCommand),

    /// Render a Jinja2 template against the live Home Assistant state.
    Template(template::TemplateCommand),

    /// Validate `configuration.yaml` (requires the `config` integration).
    CheckConfig,
}

/// Dispatches a parsed [`Command`] to the appropriate handler.
///
/// The `config` subcommand is handled before credentials are loaded because
/// its purpose is to create those credentials.  Every other command requires
/// a valid [`Config`] and an [`HaClient`].
///
/// # Errors
///
/// Returns [`CliError::MissingConfig`] when credentials are absent for a
/// command that needs them, or propagates errors from the chosen handler.
pub async fn run_command(
    command: Command,
    url: Option<String>,
    token: Option<String>,
    output: OutputFormat,
) -> Result<(), CliError> {
    // `config` subcommand does not need HA credentials.
    if let Command::Config(cmd) = command {
        return config_cmd::run(&cmd);
    }

    let config = Config::load(url, token)?;
    let client = HaClient::new(&config)?;

    match command {
        Command::Api(cmd) => api::run(cmd, &client, &output).await,
        Command::State(cmd) => states::run(cmd, &client, &output).await,
        Command::Service(cmd) => services::run(cmd, &client, &output).await,
        Command::Event(cmd) => events::run(cmd, &client, &output).await,
        Command::History(cmd) => history::run(cmd, &client, &output).await,
        Command::Logbook(cmd) => logbook::run(cmd, &client, &output).await,
        Command::Calendar(cmd) => calendar::run(cmd, &client, &output).await,
        Command::Template(cmd) => template::run(cmd, &client, &output).await,
        Command::CheckConfig => check_config::run(&client, &output).await,
        // Already handled above; this branch satisfies exhaustive matching.
        Command::Config(_) => unreachable!("Config command handled before client construction"),
    }
}
