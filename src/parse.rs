//! Shared helpers for parsing `KEY=VALUE` command-line arguments.

/// Parses a single `KEY=VALUE` string into a typed `(String, serde_json::Value)` pair.
///
/// The value portion is first attempted as JSON (so `true`, `42`, `null`,
/// `[1,2]`, `{"a":1}` all become their respective JSON types).  If that parse
/// fails the raw string is used as a JSON string value.
///
/// # Errors
///
/// Returns an error string when the input contains no `=` character.
///
/// # Examples
///
/// ```
/// let (k, v) = parse_key_value("entity_id=light.living_room").unwrap();
/// assert_eq!(k, "entity_id");
/// assert_eq!(v, serde_json::Value::String("light.living_room".into()));
///
/// let (k, v) = parse_key_value("brightness=128").unwrap();
/// assert_eq!(v, serde_json::json!(128));
/// ```
pub fn parse_key_value(s: &str) -> Result<(String, serde_json::Value), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("expected KEY=VALUE, got: {s}"))?;
    let key = s[..pos].to_string();
    let raw_val = s.get(pos.saturating_add(1)..).unwrap_or_default();
    // Try JSON parse first; fall back to treating the raw string as a JSON string.
    let value = serde_json::from_str(raw_val)
        .unwrap_or_else(|_| serde_json::Value::String(raw_val.to_string()));
    Ok((key, value))
}

/// Parses a slice of `KEY=VALUE` strings into a JSON object (`serde_json::Value::Object`).
///
/// Calls [`parse_key_value`] on each element and collects the results.
///
/// # Errors
///
/// Returns an error string if any element is malformed.
///
/// # Examples
///
/// ```
/// let obj = parse_fields_to_object(&[
///     "entity_id=light.kitchen".to_string(),
///     "brightness=200".to_string(),
/// ]).unwrap();
/// assert_eq!(obj["entity_id"], "light.kitchen");
/// assert_eq!(obj["brightness"], 200);
/// ```
pub fn parse_fields_to_object(fields: &[String]) -> Result<serde_json::Value, String> {
    let pairs = fields
        .iter()
        .map(|s| parse_key_value(s))
        .collect::<Result<Vec<_>, _>>()?;

    // serde_json::Map implements FromIterator<(String, Value)>
    let map: serde_json::Map<String, serde_json::Value> = pairs.into_iter().collect();
    Ok(serde_json::Value::Object(map))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn parses_string_value() {
        let (k, v) = parse_key_value("entity_id=light.room").unwrap();
        assert_eq!(k, "entity_id");
        assert_eq!(v, serde_json::Value::String("light.room".into()));
    }

    #[test]
    fn parses_integer_value() {
        let (_k, v) = parse_key_value("brightness=128").unwrap();
        assert_eq!(v, serde_json::json!(128));
    }

    #[test]
    fn parses_boolean_value() {
        let (_, v) = parse_key_value("enabled=true").unwrap();
        assert_eq!(v, serde_json::Value::Bool(true));
    }

    #[test]
    fn parses_null_value() {
        let (_, v) = parse_key_value("field=null").unwrap();
        assert_eq!(v, serde_json::Value::Null);
    }

    #[test]
    fn value_with_equals_sign() {
        // Only the first '=' is the separator; the rest is the value
        let (k, v) = parse_key_value("url=http://host?a=1").unwrap();
        assert_eq!(k, "url");
        assert_eq!(v, serde_json::Value::String("http://host?a=1".into()));
    }

    #[test]
    fn missing_equals_returns_error() {
        assert!(parse_key_value("noequals").is_err());
    }

    #[test]
    fn parse_fields_builds_object() {
        let obj = parse_fields_to_object(&[
            "entity_id=light.kitchen".to_string(),
            "brightness=200".to_string(),
        ])
        .unwrap();
        assert_eq!(obj["entity_id"], "light.kitchen");
        assert_eq!(obj["brightness"], 200);
    }

    #[test]
    fn parse_fields_empty_slice() {
        let obj = parse_fields_to_object(&[]).unwrap();
        assert!(obj.as_object().expect("should be object").is_empty());
    }
}
