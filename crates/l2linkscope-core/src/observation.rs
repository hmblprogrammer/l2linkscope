use serde::{Deserialize, Serialize};

use crate::{DhcpV4Offer, DiscoverySessionId, InterfaceId, ObservationId, ObservationTimestamp};

/// Strength and provenance classification for a discovery claim.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceClass {
    /// Data seen directly in traffic or local platform state.
    DirectlyObserved,
    /// A configuration or fact claimed by a network peer.
    AdvertisedByPeer,
    /// A value computed from observed or advertised inputs.
    Derived,
    /// A cautious hypothesis that is not established as fact.
    Speculative,
}

/// Bounded activity performed by a discovery session.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryMethod {
    /// Read-only inventory of operating-system interface state.
    InterfaceInventory,
    /// An explicitly requested DHCPv4 Discover/Offer probe.
    DhcpV4Probe,
}

/// Identity and timing metadata for one discovery activity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DiscoverySession {
    /// Unique session identity.
    pub id: DiscoverySessionId,
    /// Selected interface for interface-scoped discovery, if any.
    pub interface_id: Option<InterfaceId>,
    /// Discovery method used by this session.
    pub method: DiscoveryMethod,
    /// Time at which collection started.
    pub started_at: ObservationTimestamp,
    /// Time at which bounded collection completed, when complete.
    pub completed_at: Option<ObservationTimestamp>,
}

/// Acquisition source that produced an observation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationSource {
    /// Read-only operating-system interface inventory.
    InterfaceInventory,
    /// A response to an explicitly requested DHCPv4 probe.
    DhcpV4Probe,
}

/// Protocol-specific normalized content carried by an observation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", content = "details", rename_all = "snake_case")]
pub enum ObservationKind {
    /// Configuration advertised in a valid DHCPv4 Offer.
    DhcpV4Offer(DhcpV4Offer),
}

/// One evidence-bearing result collected during discovery.
///
/// Fields are read-only so callers cannot relabel peer-advertised DHCP data as
/// directly observed configuration.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Observation {
    id: ObservationId,
    session_id: DiscoverySessionId,
    interface_id: InterfaceId,
    observed_at: ObservationTimestamp,
    source: ObservationSource,
    evidence_class: EvidenceClass,
    kind: ObservationKind,
}

#[derive(Deserialize)]
struct ObservationWire {
    id: ObservationId,
    session_id: DiscoverySessionId,
    interface_id: InterfaceId,
    observed_at: ObservationTimestamp,
    source: ObservationSource,
    evidence_class: EvidenceClass,
    kind: ObservationKind,
}

impl TryFrom<ObservationWire> for Observation {
    type Error = &'static str;

    fn try_from(value: ObservationWire) -> Result<Self, Self::Error> {
        match (&value.kind, value.source, value.evidence_class) {
            (
                ObservationKind::DhcpV4Offer(_),
                ObservationSource::DhcpV4Probe,
                EvidenceClass::AdvertisedByPeer,
            ) => Ok(Self {
                id: value.id,
                session_id: value.session_id,
                interface_id: value.interface_id,
                observed_at: value.observed_at,
                source: value.source,
                evidence_class: value.evidence_class,
                kind: value.kind,
            }),
            (ObservationKind::DhcpV4Offer(_), _, _) => {
                Err("DHCPv4 offers must be peer-advertised evidence from a DHCPv4 probe")
            }
        }
    }
}

impl<'de> Deserialize<'de> for Observation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = ObservationWire::deserialize(deserializer)?;
        Self::try_from(wire).map_err(serde::de::Error::custom)
    }
}

impl Observation {
    /// Constructs an observation from a valid DHCPv4 Offer.
    ///
    /// The evidence classification is always
    /// [`EvidenceClass::AdvertisedByPeer`].
    #[must_use]
    pub fn dhcp_v4_offer(
        id: ObservationId,
        session_id: DiscoverySessionId,
        interface_id: InterfaceId,
        observed_at: ObservationTimestamp,
        offer: DhcpV4Offer,
    ) -> Self {
        Self {
            id,
            session_id,
            interface_id,
            observed_at,
            source: ObservationSource::DhcpV4Probe,
            evidence_class: EvidenceClass::AdvertisedByPeer,
            kind: ObservationKind::DhcpV4Offer(offer),
        }
    }

    /// Returns this observation's identity.
    #[must_use]
    pub const fn id(&self) -> ObservationId {
        self.id
    }

    /// Returns the session which collected this observation.
    #[must_use]
    pub const fn session_id(&self) -> DiscoverySessionId {
        self.session_id
    }

    /// Returns the interface on which this observation was collected.
    #[must_use]
    pub const fn interface_id(&self) -> &InterfaceId {
        &self.interface_id
    }

    /// Returns the collection timestamp.
    #[must_use]
    pub const fn observed_at(&self) -> ObservationTimestamp {
        self.observed_at
    }

    /// Returns the acquisition source.
    #[must_use]
    pub const fn source(&self) -> ObservationSource {
        self.source
    }

    /// Returns the evidence classification.
    #[must_use]
    pub const fn evidence_class(&self) -> EvidenceClass {
        self.evidence_class
    }

    /// Returns the normalized observation content.
    #[must_use]
    pub const fn kind(&self) -> &ObservationKind {
        &self.kind
    }
}
