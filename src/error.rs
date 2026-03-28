use thiserror::Error;

/// All errors that can occur during CLI execution.
///
/// Variants wrap underlying library errors where possible so that
/// the `?` operator propagates them automatically.
#[derive(Debug, Error)]
pub enum CliError {
    /// A reqwest HTTP transport or protocol error.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// General configuration problem (invalid value, parse failure, etc.).
    #[error("Configuration error: {0}")]
    Config(String),

    /// JSON serialization or deserialization failure.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Filesystem I/O failure (reading config file, writing output, etc.).
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// The Home Assistant API returned a non-2xx HTTP status code.
    #[error("Home Assistant API error (HTTP {status}): {message}")]
    Api { status: u16, message: String },

    /// The config file could not be parsed as valid TOML.
    #[error("Config file TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),

    /// A required config value was absent from all sources.
    ///
    /// Tells the user exactly how to supply the missing value.
    #[error(
        "Missing required config '{field}' — supply via --{flag} flag, \
         {env} environment variable, or run `hacli config init`"
    )]
    MissingConfig {
        field: &'static str,
        flag: &'static str,
        env: &'static str,
    },

    /// An HTTP header value could not be constructed (e.g. token contains illegal bytes).
    #[error("Invalid HTTP header value: {0}")]
    InvalidHeader(#[from] reqwest::header::InvalidHeaderValue),
}
