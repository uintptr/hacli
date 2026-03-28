use clap::ValueEnum;
use tabled::{builder::Builder, settings::Style};

use crate::error::CliError;

// ---------------------------------------------------------------------------
// Format enum
// ---------------------------------------------------------------------------

/// Output format selected by the global `--output` / `-o` flag.
#[derive(Debug, Clone, Default, ValueEnum)]
pub enum OutputFormat {
    /// Pretty-printed JSON (default).
    #[default]
    Json,
    /// ASCII table.  Falls back to JSON for scalar values.
    Table,
    /// Key: value pairs for objects; one value per line for arrays.
    Plain,
}

// ---------------------------------------------------------------------------
// Public dispatch
// ---------------------------------------------------------------------------

/// Prints a [`serde_json::Value`] using the requested [`OutputFormat`].
///
/// # Errors
///
/// Returns [`CliError::Json`] when JSON serialization fails (only possible
/// for the `Json` variant in pathological cases).
pub fn print_output(value: &serde_json::Value, format: &OutputFormat) -> Result<(), CliError> {
    match format {
        OutputFormat::Json => print_json(value)?,
        OutputFormat::Table => print_table(value),
        OutputFormat::Plain => print_plain(value),
    }
    Ok(())
}

/// Prints a plain text string, ignoring the output format (used for endpoints
/// that return non-JSON responses such as `/api/error_log` and `/api/template`).
pub fn print_text(text: &str) {
    println!("{text}");
}

// ---------------------------------------------------------------------------
// Format implementations
// ---------------------------------------------------------------------------

/// Pretty-prints `value` as indented JSON.
fn print_json(value: &serde_json::Value) -> Result<(), CliError> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

/// Renders `value` as a rounded ASCII table.
///
/// - **Array of objects** → one row per element, columns from the first element's keys.
/// - **Single object** → two-column `Key / Value` table.
/// - **Other** → falls back to plain display.
fn print_table(value: &serde_json::Value) {
    match value {
        serde_json::Value::Array(items) => print_array_as_table(items),
        serde_json::Value::Object(map) => {
            let mut builder = Builder::default();
            builder.push_record(["Key", "Value"]);
            for (k, v) in map {
                builder.push_record([k.as_str(), &value_to_display(v)]);
            }
            let mut table = builder.build();
            table.with(Style::rounded());
            println!("{table}");
        }
        other => println!("{}", value_to_display(other)),
    }
}

/// Renders an array of JSON values as a table.
fn print_array_as_table(items: &[serde_json::Value]) {
    if items.is_empty() {
        println!("(no results)");
        return;
    }

    // Derive column names from the first element's keys (if it is an object).
    let Some(first) = items.first() else {
        return;
    };
    let headers: Vec<String> = if let serde_json::Value::Object(map) = first {
        map.keys().cloned().collect()
    } else {
        vec!["value".to_string()]
    };

    let mut builder = Builder::default();
    // Push header row
    builder.push_record(headers.iter().map(String::as_str));

    for item in items {
        let row: Vec<String> = if let serde_json::Value::Object(obj) = item {
            headers
                .iter()
                .map(|k| obj.get(k).map(value_to_display).unwrap_or_default())
                .collect()
        } else {
            vec![value_to_display(item)]
        };
        builder.push_record(row.iter().map(String::as_str));
    }

    let mut table = builder.build();
    table.with(Style::rounded());
    println!("{table}");
}

/// Renders `value` as human-readable `key: value` lines.
///
/// - **Object** → one `key: value` line per field.
/// - **Array** → each element is printed under a `[n]` header.
/// - **Other** → single line.
fn print_plain(value: &serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                println!("{k}: {}", value_to_display(v));
            }
        }
        serde_json::Value::Array(items) => {
            for (i, item) in items.iter().enumerate() {
                println!("[{i}]");
                if let serde_json::Value::Object(map) = item {
                    for (k, v) in map {
                        println!("  {k}: {}", value_to_display(v));
                    }
                } else {
                    println!("  {}", value_to_display(item));
                }
            }
        }
        other => println!("{}", value_to_display(other)),
    }
}

// ---------------------------------------------------------------------------
// Internal helper
// ---------------------------------------------------------------------------

/// Converts a JSON value to a compact display string.
///
/// Strings are returned as-is (no surrounding quotes).
/// Null becomes an empty string.
/// All other values use their JSON representation.
fn value_to_display(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn value_to_display_string() {
        assert_eq!(value_to_display(&json!("hello")), "hello");
    }

    #[test]
    fn value_to_display_null() {
        assert_eq!(value_to_display(&json!(null)), "");
    }

    #[test]
    fn value_to_display_number() {
        assert_eq!(value_to_display(&json!(42)), "42");
    }

    #[test]
    fn print_json_does_not_error_on_object() {
        let result = print_json(&json!({"key": "value"}));
        assert!(result.is_ok());
    }

    #[test]
    fn print_table_empty_array() {
        // Should not panic, just print "(no results)"
        print_table(&json!([]));
    }
}
