use std::fmt;

use serde::{Deserialize, Serialize};

/// Stable category for a non-fatal discovery warning.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryWarningCode {
    /// More than one DHCP server returned a distinct offer.
    MultipleDhcpServers,
    /// One or more hostile or malformed responses were ignored.
    MalformedResponse,
    /// A configured collection limit was reached.
    CollectionLimitReached,
    /// An interface changed or disappeared during discovery.
    InterfaceChanged,
}

/// Structured non-fatal warning attached to a discovery result.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DiscoveryWarning {
    /// Machine-readable warning category.
    pub code: DiscoveryWarningCode,
    /// Human-readable diagnostic without sensitive packet contents.
    pub message: String,
}

impl DiscoveryWarning {
    /// Constructs a warning from a stable category and display message.
    #[must_use]
    pub fn new(code: DiscoveryWarningCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

/// Stable category for a discovery failure.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryErrorCode {
    /// The named interface does not exist.
    InterfaceNotFound,
    /// The interface cannot safely support the requested operation.
    UnsupportedInterface,
    /// The process lacks a required privilege or Linux capability.
    InsufficientPrivileges,
    /// A socket or other acquisition transport failed.
    TransportFailure,
    /// Responses arrived, but none could be safely parsed as a valid result.
    MalformedResponses,
    /// A caller supplied an invalid model or operation argument.
    InvalidInput,
    /// An unexpected internal failure occurred.
    Internal,
}

/// Structured discovery failure suitable for machine-readable output.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DiscoveryError {
    /// Machine-readable failure category.
    pub code: DiscoveryErrorCode,
    /// Human-readable diagnostic that is safe to display.
    pub message: String,
}

impl DiscoveryError {
    /// Constructs a discovery error from a stable category and display message.
    #[must_use]
    pub fn new(code: DiscoveryErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for DiscoveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for DiscoveryError {}
