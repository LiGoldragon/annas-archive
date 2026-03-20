use std::fmt;

/// Errors produced by archive operations.
#[derive(Debug)]
pub enum Error {
    /// HTTP request failed.
    Network(reqwest::Error),

    /// Server returned a non-success HTTP status.
    Http { status: u16 },

    /// All configured domains returned errors.
    DomainsExhausted,

    /// Operation requires an API key but none was configured.
    KeyRequired,

    /// Authentication was rejected by the remote.
    KeyRejected,

    /// Expected HTML or JSON structure was absent.
    MissingField { field: &'static str },

    /// JSON could not be decoded.
    Decode(serde_json::Error),

    /// CSS selector failed to compile (code bug, not runtime).
    Selector { selector: &'static str },

    /// The remote API returned an error message.
    Remote { message: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Network(e) => write!(f, "network error: {e}"),
            Error::Http { status } => write!(f, "HTTP {status}"),
            Error::DomainsExhausted => {
                write!(f, "all configured domains failed")
            }
            Error::KeyRequired => {
                write!(f, "API key required but not configured")
            }
            Error::KeyRejected => write!(f, "API key rejected"),
            Error::MissingField { field } => {
                write!(f, "missing expected field: {field}")
            }
            Error::Decode(e) => write!(f, "JSON decode: {e}"),
            Error::Selector { selector } => {
                write!(f, "invalid CSS selector: {selector}")
            }
            Error::Remote { message } => {
                write!(f, "remote error: {message}")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Network(e) => Some(e),
            Error::Decode(e) => Some(e),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Network(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Decode(err)
    }
}
