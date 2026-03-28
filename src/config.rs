use std::{fs, io::Write, path::PathBuf};

use dirs::config_dir;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};

use crate::error::CliError;

// ---------------------------------------------------------------------------
// On-disk representation
// ---------------------------------------------------------------------------

/// Structure of `~/.config/hacli/config.toml`.
///
/// All fields are optional so a partial file is valid; missing values can be
/// supplied via environment variables or CLI flags.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ConfigFile {
    /// Base URL of the Home Assistant instance (e.g. `http://homeassistant.local:8123`).
    pub url: Option<String>,
    /// Long-lived access token.  Stored as plain text (file is user-owned).
    pub token: Option<String>,
}

// ---------------------------------------------------------------------------
// Resolved, ready-to-use configuration
// ---------------------------------------------------------------------------

/// Fully resolved configuration ready for use by [`crate::client::HaClient`].
pub struct Config {
    /// Base URL with any trailing slash stripped.
    pub url: String,
    /// Bearer token wrapped in [`SecretString`] to prevent accidental logging.
    pub token: SecretString,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns `~/.config/hacli/config.toml`, or `None` when the platform has no
/// config directory.
pub fn config_file_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("hacli").join("config.toml"))
}

/// Reads and parses the config file.
///
/// Returns an empty [`ConfigFile`] when the file does not exist yet rather
/// than failing, so first-run usage works without a config file.
fn load_config_file() -> Result<ConfigFile, CliError> {
    let Some(path) = config_file_path() else {
        return Ok(ConfigFile::default());
    };

    if !path.exists() {
        return Ok(ConfigFile::default());
    }

    let contents = fs::read_to_string(&path)?;
    Ok(toml::from_str(&contents)?)
}

impl Config {
    /// Load configuration with the following precedence (highest first):
    ///
    /// 1. `url_override` / `token_override` — already resolved by `clap`
    ///    from `--url` / `--token` flags **or** `HA_URL` / `HA_TOKEN` env vars.
    /// 2. Values from `~/.config/hacli/config.toml`.
    ///
    /// # Errors
    ///
    /// Returns [`CliError::MissingConfig`] when a required value is absent
    /// from all sources.
    pub fn load(
        url_override: Option<String>,
        token_override: Option<String>,
    ) -> Result<Self, CliError> {
        let file_config = load_config_file()?;
        Self::resolve(url_override, token_override, file_config)
    }

    /// Merges CLI/env overrides with a [`ConfigFile`] (from disk or injected in
    /// tests) and returns a fully resolved [`Config`].
    fn resolve(
        url_override: Option<String>,
        token_override: Option<String>,
        file_config: ConfigFile,
    ) -> Result<Self, CliError> {
        let url = url_override
            .or(file_config.url)
            .ok_or(CliError::MissingConfig {
                field: "url",
                flag: "url",
                env: "HA_URL",
            })?;

        let raw_token = token_override
            .or(file_config.token)
            .ok_or(CliError::MissingConfig {
                field: "token",
                flag: "token",
                env: "HA_TOKEN",
            })?;

        Ok(Self {
            // Strip trailing slash so path concatenation is consistent
            url: url.trim_end_matches('/').to_string(),
            token: SecretString::new(raw_token.into()),
        })
    }
}

// ---------------------------------------------------------------------------
// Writing the config file (used by `hacli config init`)
// ---------------------------------------------------------------------------

/// Persists a [`ConfigFile`] to `~/.config/hacli/config.toml`.
///
/// Creates the parent directory if it does not exist.
///
/// # Errors
///
/// Returns [`CliError::Config`] when no config directory path is available,
/// or propagates I/O errors from directory creation / file writing.
pub fn write_config_file(cfg: &ConfigFile) -> Result<PathBuf, CliError> {
    let path = config_file_path()
        .ok_or_else(|| CliError::Config("Cannot determine config directory".to_string()))?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let toml_string = toml::to_string_pretty(cfg)
        .map_err(|e| CliError::Config(format!("Failed to serialize config: {e}")))?;

    let mut file = fs::File::create(&path)?;
    file.write_all(toml_string.as_bytes())?;

    Ok(path)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Shorthand: resolve overrides against an empty config file so tests
    /// never hit the real filesystem.
    fn resolve_empty(url: Option<&str>, token: Option<&str>) -> Result<Config, CliError> {
        Config::resolve(
            url.map(String::from),
            token.map(String::from),
            ConfigFile::default(),
        )
    }

    #[test]
    fn load_prefers_override_over_file() {
        let config = resolve_empty(Some("http://ha.local:8123"), Some("my-token"))
            .expect("config should load successfully");
        assert_eq!(config.url, "http://ha.local:8123");
    }

    #[test]
    fn load_strips_trailing_slash_from_url() {
        let config = resolve_empty(Some("http://ha.local:8123/"), Some("tok"))
            .expect("config should load successfully");
        assert_eq!(config.url, "http://ha.local:8123");
    }

    #[test]
    fn load_returns_error_when_url_missing() {
        let result = resolve_empty(None, Some("tok"));
        assert!(matches!(
            result,
            Err(CliError::MissingConfig { field: "url", .. })
        ));
    }

    #[test]
    fn load_returns_error_when_token_missing() {
        let result = resolve_empty(Some("http://ha.local"), None);
        assert!(matches!(
            result,
            Err(CliError::MissingConfig { field: "token", .. })
        ));
    }
}
