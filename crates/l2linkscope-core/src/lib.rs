//! Portable discovery-domain models for L2LinkScope.
//!
//! This crate describes interfaces, discovery sessions, observations, evidence,
//! and diagnostics without acquiring packets or depending on Linux APIs. Protocol
//! crates and acquisition layers normalize their results into these types; user
//! interfaces decide how to present them.
//!
//! A DHCP offer is always constructed as [`EvidenceClass::AdvertisedByPeer`]. It
//! describes configuration proposed by a peer and never means that configuration
//! was trusted, accepted, or applied.
//!
//! # JSON compatibility
//!
//! [`DiscoverySnapshot`] is the top-level serializable result and includes
//! [`JSON_SCHEMA_VERSION`]. The JSON format is experimental throughout the `0.x`
//! release series. Consumers should reject unsupported schema versions rather
//! than relying on undocumented implementation details.
//!
//! # Example
//!
//! ```
//! use l2linkscope_core::{
//!     DiscoveryMethod, DiscoverySession, DiscoverySessionId, DiscoverySnapshot,
//!     ObservationTimestamp, JSON_SCHEMA_VERSION,
//! };
//!
//! let session = DiscoverySession {
//!     id: DiscoverySessionId::new(),
//!     interface_id: None,
//!     method: DiscoveryMethod::InterfaceInventory,
//!     started_at: ObservationTimestamp::from_unix_milliseconds(0),
//!     completed_at: None,
//! };
//! let snapshot = DiscoverySnapshot::new(session);
//! assert_eq!(snapshot.schema_version, JSON_SCHEMA_VERSION);
//! ```
//!
//! # Safety and errors
//!
//! This crate forbids unsafe code and performs no network or operating-system
//! operations. Constructors which validate caller-provided data return explicit
//! errors; collecting a snapshot cannot change network configuration.
//!
//! The repository's [architecture](https://github.com/hmblprogrammer/l2linkscope/blob/main/Documentation/Architecture.md)
//! and [evidence model](https://github.com/hmblprogrammer/l2linkscope/blob/main/Documentation/EvidenceModel.md)
//! document the larger product boundaries.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod dhcp;
mod diagnostics;
mod identifiers;
mod interface;
mod observation;
mod snapshot;

pub use dhcp::{ClasslessStaticRoute, DhcpV4Offer, Ipv4Network};
pub use diagnostics::{DiscoveryError, DiscoveryErrorCode, DiscoveryWarning, DiscoveryWarningCode};
pub use identifiers::{DiscoverySessionId, ObservationId, ObservationTimestamp, TimestampError};
pub use interface::{
    AdministrativeState, CarrierState, Interface, InterfaceAddress, InterfaceId, MacAddress,
    MacAddressParseError, OperationalState, ProbeSupport,
};
pub use observation::{
    DiscoveryMethod, DiscoverySession, EvidenceClass, Observation, ObservationKind,
    ObservationSource,
};
pub use snapshot::DiscoverySnapshot;

/// Schema version emitted by [`DiscoverySnapshot`].
///
/// The schema is experimental during the `0.x` series.
pub const JSON_SCHEMA_VERSION: &str = "0.1";

/// Returns a concise description of this crate's intended responsibility.
#[must_use]
pub const fn crate_purpose() -> &'static str {
    "portable L2LinkScope domain concepts"
}
