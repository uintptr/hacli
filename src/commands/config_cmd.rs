use std::io::{self, Write};

use clap::{Args, Subcommand};

use crate::{
    config::{ConfigFile, config_file_path, write_config_file},
    error::CliError,
};

/// Arguments for the `config` subcommand.
#[derive(Debug, Args)]
pub struct ConfigCommand {
    #[command(subcommand)]
    pub action: ConfigAction,
}

/// Actions available under `hacli config`.
#[derive(Debug, Subcommand)]
pub enum ConfigAction {
    /// Interactively create or overwrite `~/.config/hacli/config.toml`.
    Init,
    /// Print the path to the config file.
    Path,
}

/// Executes a `config` subcommand action.
///
/// This command does **not** require Home Assistant credentials since its
/// purpose is to set them up.
///
/// # Errors
///
/// Propagates [`CliError`] from I/O or config-write operations.
pub fn run(cmd: &ConfigCommand) -> Result<(), CliError> {
    match &cmd.action {
        ConfigAction::Init => run_init(),
        ConfigAction::Path => {
            match config_file_path() {
                Some(path) => println!("{}", path.display()),
                None => println!("(config directory not available on this platform)"),
            }
            Ok(())
        }
    }
}

/// Prompts the user for Home Assistant URL and token, then writes the config.
fn run_init() -> Result<(), CliError> {
    println!("Initializing hacli configuration.");
    println!("Tip: get a Long-Lived Access Token from your HA profile page.");
    println!();

    let url = prompt("Home Assistant URL (e.g. http://homeassistant.local:8123): ")?;
    // rpassword hides the token while typing so it does not appear in the terminal.
    let token = rpassword::prompt_password("Long-Lived Access Token: ")
        .map_err(|e| CliError::Config(format!("Failed to read token: {e}")))?;

    let url = url.trim().to_string();
    let token = token.trim().to_string();

    if url.is_empty() {
        return Err(CliError::Config("URL must not be empty".to_string()));
    }
    if token.is_empty() {
        return Err(CliError::Config("Token must not be empty".to_string()));
    }

    let cfg = ConfigFile {
        url: Some(url),
        token: Some(token),
    };

    let path = write_config_file(&cfg)?;
    println!();
    println!("Configuration written to: {}", path.display());
    Ok(())
}

/// Prints a prompt to stdout and reads a line from stdin.
///
/// # Errors
///
/// Returns [`CliError::Io`] on I/O failure.
fn prompt(message: &str) -> Result<String, CliError> {
    print!("{message}");
    // Flush stdout so the prompt appears before we block on stdin.
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input
        .trim_end_matches('\n')
        .trim_end_matches('\r')
        .to_string())
}
