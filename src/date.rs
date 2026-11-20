//! Microsoft `/Date(milliseconds)/` date format parsing and serde helpers.
//!
//! Vietstock's API (ASP.NET backend) serializes dates using Microsoft's
//! proprietary JSON date format: `/Date(1780368384730)/`
//!
//! The timestamp value is **Unix milliseconds in UTC**.

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Deserializer};
use std::fmt;

use crate::error::ApiError;

/// Parse a Microsoft `/Date(milliseconds)/` string into a `DateTime<Utc>`.
///
/// # Arguments
///
/// * `s` — A string like `/Date(1780368384730)/`
///
/// # Errors
///
/// Returns `ApiError::DateParse` if the string does not match the expected format.
/// Returns `ApiError::InvalidTimestamp` if the millisecond value is out of range.
///
/// # Examples
///
/// ```
/// use vietstock::date::parse_ms_date;
/// use chrono::Datelike;
///
/// let dt = parse_ms_date("/Date(1780368384730)/").unwrap();
/// assert_eq!(dt.year(), 2026);
/// ```
pub fn parse_ms_date(s: &str) -> Result<DateTime<Utc>, ApiError> {
    let ts_str = s
        .strip_prefix("/Date(")
        .and_then(|rest| rest.strip_suffix(")/"))
        .ok_or_else(|| ApiError::DateParse(s.to_string()))?;

    let ms: i64 = ts_str
        .parse()
        .map_err(|_| ApiError::DateParse(s.to_string()))?;

    Utc.timestamp_millis_opt(ms)
        .single()
        .ok_or(ApiError::InvalidTimestamp(ms))
}

// ---------------------------------------------------------------------------
// Serde Deserializers
// ---------------------------------------------------------------------------

/// Custom Serde deserializer for Microsoft's `/Date(timestamp)/` JSON format.
///
/// Use with `#[serde(deserialize_with = "deserialize_ms_date")]` on
/// `chrono::DateTime<Utc>` fields.
///
/// # Examples
///
/// ```
/// use vietstock::date::deserialize_ms_date;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Example {
///     #[serde(deserialize_with = "deserialize_ms_date")]
///     pub last_update: chrono::DateTime<chrono::Utc>,
/// }
///
/// let json = r#"{"last_update": "/Date(1780368384730)/"}"#;
/// let ex: Example = serde_json::from_str(json).unwrap();
/// ```
pub fn deserialize_ms_date<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    struct MsDateVisitor;

    impl<'de> serde::de::Visitor<'de> for MsDateVisitor {
        type Value = DateTime<Utc>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string in Microsoft /Date(milliseconds)/ format")
        }

        fn visit_str<E>(self, s: &str) -> Result<DateTime<Utc>, E>
        where
            E: serde::de::Error,
        {
            parse_ms_date(s).map_err(E::custom)
        }
    }

    deserializer.deserialize_str(MsDateVisitor)
}

/// Custom Serde deserializer for `Option<DateTime<Utc>>` accepting Microsoft date format or null.
///
/// Use with `#[serde(deserialize_with = "deserialize_opt_ms_date")]`.
///
/// Unlike `deserialize_ms_date`, this accepts `null` JSON values (mapped to `None`).
pub fn deserialize_opt_ms_date<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct OptMsDateVisitor;

    impl<'de> serde::de::Visitor<'de> for OptMsDateVisitor {
        type Value = Option<DateTime<Utc>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("null or a string in Microsoft /Date(milliseconds)/ format")
        }

        fn visit_none<E>(self) -> Result<Option<DateTime<Utc>>, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Option<DateTime<Utc>>, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }

        fn visit_str<E>(self, s: &str) -> Result<Option<DateTime<Utc>>, E>
        where
            E: serde::de::Error,
        {
            parse_ms_date(s).map(Some).map_err(E::custom)
        }
    }

    deserializer.deserialize_any(OptMsDateVisitor)
}

/// Custom Serde deserializer that trims trailing whitespace from strings.
///
/// Vietstock's API returns `FileExt` values like `".zip "` with trailing spaces.
/// This deserializer normalizes them.
///
/// # Examples
///
/// ```
/// use vietstock::date::deserialize_trim_trailing;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Test {
///     #[serde(deserialize_with = "deserialize_trim_trailing")]
///     pub value: String,
/// }
///
/// let t: Test = serde_json::from_str(r#"{"value": ".zip "}"#).unwrap();
/// assert_eq!(t.value, ".zip");
/// ```
pub fn deserialize_trim_trailing<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.trim_end().to_string())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_parse_ms_date_valid() {
        let dt = parse_ms_date("/Date(1780368384730)/").unwrap();
        assert_eq!(dt.year(), 2026);
        assert!(dt.month() >= 5 && dt.month() <= 7);
    }

    #[test]
    fn test_parse_ms_date_trading_date() {
        let dt = parse_ms_date("/Date(1781596800673)/").unwrap();
        assert_eq!(dt.year(), 2026);
    }

    #[test]
    fn test_parse_ms_date_invalid_format() {
        assert!(parse_ms_date("2024-01-01").is_err());
        assert!(parse_ms_date("/Date(abc)/").is_err());
        assert!(parse_ms_date("").is_err());
    }

    #[test]
    fn test_parse_ms_date_negative_timestamp() {
        let dt = parse_ms_date("/Date(-1000)/").unwrap();
        assert_eq!(dt.year(), 1969);
    }

    #[test]
    fn test_deserialize_ms_date_on_struct() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "deserialize_ms_date")]
            ts: DateTime<Utc>,
        }

        let t: Test = serde_json::from_str(r#"{"ts": "/Date(1780368384730)/"}"#).unwrap();
        assert_eq!(t.ts.year(), 2026);
    }

    #[test]
    fn test_deserialize_opt_ms_date_null() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "deserialize_opt_ms_date")]
            ts: Option<DateTime<Utc>>,
        }

        let t: Test = serde_json::from_str(r#"{"ts": null}"#).unwrap();
        assert!(t.ts.is_none());
    }

    #[test]
    fn test_deserialize_opt_ms_date_value() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "deserialize_opt_ms_date")]
            ts: Option<DateTime<Utc>>,
        }

        let t: Test = serde_json::from_str(r#"{"ts": "/Date(1780368384730)/"}"#).unwrap();
        assert!(t.ts.is_some());
    }

    #[test]
    fn test_trim_trailing_deserializer() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "deserialize_trim_trailing")]
            value: String,
        }

        let t: Test = serde_json::from_str(r#"{"value": ".zip "}"#).unwrap();
        assert_eq!(t.value, ".zip");

        let t: Test = serde_json::from_str(r#"{"value": ".pdf"}"#).unwrap();
        assert_eq!(t.value, ".pdf");

        let t: Test = serde_json::from_str(r#"{"value": "no-spaces"}"#).unwrap();
        assert_eq!(t.value, "no-spaces");
    }
}
