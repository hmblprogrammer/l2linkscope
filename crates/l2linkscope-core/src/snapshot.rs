use serde::{Deserialize, Serialize};

use crate::{DiscoverySession, DiscoveryWarning, Interface, JSON_SCHEMA_VERSION, Observation};

/// Top-level normalized output from a discovery activity.
///
/// JSON consumers must inspect [`Self::schema_version`]. Compatibility is
/// experimental during the `0.x` release series.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DiscoverySnapshot {
    /// JSON schema version for this top-level result.
    pub schema_version: String,
    /// Identity, method, and timing for the discovery activity.
    pub session: DiscoverySession,
    /// Interfaces relevant to this activity.
    pub interfaces: Vec<Interface>,
    /// Evidence-bearing observations collected by this activity.
    pub observations: Vec<Observation>,
    /// Non-fatal diagnostics collected during this activity.
    pub warnings: Vec<DiscoveryWarning>,
}

impl DiscoverySnapshot {
    /// Creates an empty snapshot using the current JSON schema version.
    #[must_use]
    pub fn new(session: DiscoverySession) -> Self {
        Self {
            schema_version: JSON_SCHEMA_VERSION.to_owned(),
            session,
            interfaces: Vec::new(),
            observations: Vec::new(),
            warnings: Vec::new(),
        }
    }
}
