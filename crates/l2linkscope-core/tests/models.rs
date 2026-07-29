use std::net::{IpAddr, Ipv4Addr};

use l2linkscope_core::{
    AdministrativeState, CarrierState, DhcpV4Offer, DiscoveryMethod, DiscoverySession,
    DiscoverySessionId, DiscoverySnapshot, EvidenceClass, Interface, InterfaceAddress, InterfaceId,
    JSON_SCHEMA_VERSION, MacAddress, Observation, ObservationId, ObservationSource,
    ObservationTimestamp, OperationalState, ProbeSupport,
};
use serde_json::json;
use uuid::Uuid;

fn fixed_session_id() -> DiscoverySessionId {
    DiscoverySessionId::from_uuid(Uuid::from_u128(1))
}

fn fixed_observation_id() -> ObservationId {
    ObservationId::from_uuid(Uuid::from_u128(2))
}

fn interface_id() -> InterfaceId {
    InterfaceId {
        index: 2,
        hardware_address: Some(MacAddress::new([0, 17, 34, 51, 68, 85])),
    }
}

fn offer() -> DhcpV4Offer {
    DhcpV4Offer {
        transaction_id: 0x1234_5678,
        client_hardware_address: MacAddress::new([0, 17, 34, 51, 68, 85]),
        offered_address: Ipv4Addr::new(192, 0, 2, 117),
        server_identifier: Some(Ipv4Addr::new(192, 0, 2, 1)),
        subnet_mask: Some(Ipv4Addr::new(255, 255, 255, 0)),
        routers: vec![Ipv4Addr::new(192, 0, 2, 1)],
        dns_servers: vec![Ipv4Addr::new(192, 0, 2, 10)],
        domain_name: Some("example.test".to_owned()),
        domain_search: vec!["example.test".to_owned()],
        lease_time_seconds: Some(28_800),
        renewal_time_seconds: Some(14_400),
        rebinding_time_seconds: Some(25_200),
        interface_mtu: Some(1500),
        classless_static_routes: Vec::new(),
    }
}

#[test]
fn dhcp_offer_is_always_peer_advertised_evidence() {
    let observation = Observation::dhcp_v4_offer(
        fixed_observation_id(),
        fixed_session_id(),
        interface_id(),
        ObservationTimestamp::from_unix_milliseconds(1_700_000_000_123),
        offer(),
    );

    assert_eq!(
        observation.evidence_class(),
        EvidenceClass::AdvertisedByPeer
    );
    assert_eq!(observation.source(), ObservationSource::DhcpV4Probe);
}

#[test]
fn deserialization_rejects_misclassified_dhcp_offer() -> Result<(), serde_json::Error> {
    let observation = Observation::dhcp_v4_offer(
        fixed_observation_id(),
        fixed_session_id(),
        interface_id(),
        ObservationTimestamp::from_unix_milliseconds(1_700_000_000_123),
        offer(),
    );
    let mut value = serde_json::to_value(observation)?;
    value["evidence_class"] = json!("directly_observed");

    let result = serde_json::from_value::<Observation>(value);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn evidence_classes_have_stable_json_names() -> Result<(), serde_json::Error> {
    let values = [
        (EvidenceClass::DirectlyObserved, "\"directly_observed\""),
        (EvidenceClass::AdvertisedByPeer, "\"advertised_by_peer\""),
        (EvidenceClass::Derived, "\"derived\""),
        (EvidenceClass::Speculative, "\"speculative\""),
    ];

    for (evidence, expected) in values {
        assert_eq!(serde_json::to_string(&evidence)?, expected);
    }
    Ok(())
}

#[test]
fn snapshot_serialization_is_explicit_and_versioned() -> Result<(), serde_json::Error> {
    let id = interface_id();
    let session = DiscoverySession {
        id: fixed_session_id(),
        interface_id: Some(id.clone()),
        method: DiscoveryMethod::DhcpV4Probe,
        started_at: ObservationTimestamp::from_unix_milliseconds(1_700_000_000_000),
        completed_at: Some(ObservationTimestamp::from_unix_milliseconds(
            1_700_000_003_000,
        )),
    };
    let observation = Observation::dhcp_v4_offer(
        fixed_observation_id(),
        fixed_session_id(),
        id.clone(),
        ObservationTimestamp::from_unix_milliseconds(1_700_000_001_000),
        offer(),
    );
    let mut snapshot = DiscoverySnapshot::new(session);
    snapshot.interfaces.push(Interface {
        id,
        name: "enp3s0".to_owned(),
        administrative_state: AdministrativeState::Up,
        operational_state: OperationalState::Up,
        carrier_state: CarrierState::Present,
        mtu: 1500,
        addresses: vec![InterfaceAddress {
            address: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 25)),
            prefix_length: 24,
        }],
        is_loopback: false,
        dhcp_v4_probe: ProbeSupport::supported(),
    });
    snapshot.observations.push(observation);

    let value = serde_json::to_value(snapshot)?;
    assert_eq!(value["schema_version"], json!(JSON_SCHEMA_VERSION));
    assert_eq!(value["session"]["method"], json!("dhcp_v4_probe"));
    assert_eq!(
        value["interfaces"][0]["id"]["hardware_address"],
        json!("00:11:22:33:44:55")
    );
    assert_eq!(
        value["observations"][0]["evidence_class"],
        json!("advertised_by_peer")
    );
    assert_eq!(
        value["observations"][0]["kind"]["type"],
        json!("dhcp_v4_offer")
    );
    assert_eq!(
        value["observations"][0]["kind"]["details"]["offered_address"],
        json!("192.0.2.117")
    );
    Ok(())
}

#[test]
fn mac_address_round_trips_as_canonical_string() -> Result<(), Box<dyn std::error::Error>> {
    let address: MacAddress = "0A:1b:2C:3d:4E:5f".parse()?;
    assert_eq!(address.to_string(), "0a:1b:2c:3d:4e:5f");
    let json = serde_json::to_string(&address)?;
    assert_eq!(json, "\"0a:1b:2c:3d:4e:5f\"");
    let decoded: MacAddress = serde_json::from_str(&json)?;
    assert_eq!(decoded, address);
    Ok(())
}
