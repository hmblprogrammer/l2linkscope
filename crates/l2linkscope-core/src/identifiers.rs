use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Stable identity for one discovery session.
///
/// The value serializes as a lowercase hyphenated UUID string.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct DiscoverySessionId(Uuid);

impl DiscoverySessionId {
    /// Generates a new random session identity.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Constructs an identity from a UUID.
    #[must_use]
    pub const fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    /// Returns the underlying UUID.
    #[must_use]
    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for DiscoverySessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for DiscoverySessionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Stable identity for one observation within a discovery result.
///
/// The value serializes as a lowercase hyphenated UUID string.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ObservationId(Uuid);

impl ObservationId {
    /// Generates a new random observation identity.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Constructs an identity from a UUID.
    #[must_use]
    pub const fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    /// Returns the underlying UUID.
    #[must_use]
    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for ObservationId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ObservationId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Portable timestamp represented as signed milliseconds from the Unix epoch.
///
/// JSON uses an object with the explicit field `unix_milliseconds` rather than a
/// platform-specific time representation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ObservationTimestamp {
    /// Signed milliseconds since 1970-01-01 00:00:00 UTC.
    pub unix_milliseconds: i64,
}

impl ObservationTimestamp {
    /// Constructs a timestamp from signed Unix milliseconds.
    #[must_use]
    pub const fn from_unix_milliseconds(unix_milliseconds: i64) -> Self {
        Self { unix_milliseconds }
    }

    /// Converts a [`SystemTime`] into a portable timestamp.
    ///
    /// # Errors
    ///
    /// Returns [`TimestampError`] if the time is outside the range representable
    /// by a signed 64-bit millisecond count.
    pub fn from_system_time(value: SystemTime) -> Result<Self, TimestampError> {
        match value.duration_since(UNIX_EPOCH) {
            Ok(duration) => {
                let milliseconds =
                    i64::try_from(duration.as_millis()).map_err(|_| TimestampError::OutOfRange)?;
                Ok(Self::from_unix_milliseconds(milliseconds))
            }
            Err(error) => {
                let magnitude = error.duration().as_millis();
                let milliseconds =
                    i64::try_from(magnitude).map_err(|_| TimestampError::OutOfRange)?;
                Ok(Self::from_unix_milliseconds(-milliseconds))
            }
        }
    }

    /// Returns the current wall-clock time.
    ///
    /// # Errors
    ///
    /// Returns [`TimestampError`] if the platform clock value cannot fit in the
    /// portable representation.
    pub fn now() -> Result<Self, TimestampError> {
        Self::from_system_time(SystemTime::now())
    }
}

/// Error converting a platform clock value into an [`ObservationTimestamp`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimestampError {
    /// The platform clock value cannot fit into signed Unix milliseconds.
    OutOfRange,
}

impl fmt::Display for TimestampError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("system time is outside the supported timestamp range")
    }
}

impl std::error::Error for TimestampError {}
