//! Error types for the Vietstock Finance API client.

use thiserror::Error;

/// All possible errors that can occur when interacting with the Vietstock API.
#[derive(Debug, Error)]
pub enum ApiError {
    /// HTTP transport error (connection, timeout, DNS, TLS, etc.)
    #[error("HTTP request failed: {0}")]
    Http(#[from] wreq::Error),

    /// API returned a non-success HTTP status.
    /// Contains the status code and response body for diagnostics.
    #[error("API error: HTTP {status} — {body}")]
    Api {
        /// The HTTP status code returned by the server
        status: wreq::StatusCode,
        /// The response body text (may be truncated)
        body: String,
    },

    /// CSRF token could not be extracted from the session initialization response.
    /// This usually means the Vietstock page structure changed or the URL path is wrong.
    #[error(
        "CSRF token not found in Set-Cookie headers or cookie store. The session page may have changed."
    )]
    CsrfTokenNotFound,

    /// Failed to parse a Microsoft `/Date(timestamp)/` formatted string.
    #[error("Failed to parse Microsoft date format: '{0}'")]
    DateParse(String),

    /// The parsed Unix millisecond timestamp is out of the valid chrono `DateTime` range.
    #[error("Invalid timestamp: {0} milliseconds is out of valid date range")]
    InvalidTimestamp(i64),

    /// Vietstock answered with its own error envelope, e.g.
    /// `RequestUpgradeAccount_Permission` when the account tier cannot see the data.
    #[error("Vietstock error {code}: {message}")]
    Vietstock {
        /// `ErrorCode` from the response.
        code: String,
        /// `ErrorMessage` from the response (Vietnamese).
        message: String,
    },

    /// Local file-system error (e.g. while saving a downloaded document).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// An empty or invalid stock code was provided.
    /// Stock codes must be non-empty alphanumeric strings.
    #[error("Invalid stock code: '{0}'")]
    InvalidStockCode(String),

    /// Session not initialized — must call `init_session()` first.
    #[error(
        "Session not initialized. Call VietstockClient::init_session() before making authenticated requests."
    )]
    SessionNotInitialized,

    /// API returned HTTP 200 but the body was not valid JSON.
    /// This typically means the server returned an HTML error page or
    /// an empty response (often due to CSRF validation failure).
    #[error("Unexpected response (not valid JSON). Parse error: {parse_error}\nBody: {body}")]
    UnexpectedResponse {
        /// The serde_json parse error message
        parse_error: String,
        /// The actual response body (first 2000 chars)
        body: String,
    },
}

impl ApiError {
    /// True when Vietstock refused the request because the account tier is too low.
    pub fn is_permission_denied(&self) -> bool {
        matches!(self, Self::Vietstock { code, .. } if code.ends_with("_Permission"))
    }
}
