use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};

use crate::MacAddress;

/// An IPv4 network advertised in a classless static route.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Ipv4Network {
    /// Network address with host bits cleared by the protocol parser.
    pub address: Ipv4Addr,
    /// Prefix length from 0 through 32.
    pub prefix_length: u8,
}

/// A classless static route advertised by a DHCP peer.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ClasslessStaticRoute {
    /// Destination IPv4 network.
    pub destination: Ipv4Network,
    /// Router through which the destination was advertised as reachable.
    pub router: Ipv4Addr,
}

/// Configuration advertised in one valid DHCPv4 Offer.
///
/// Every field is peer-provided and untrusted. Presence in this structure does
/// not indicate that any value was accepted, applied, reachable, or authoritative.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct DhcpV4Offer {
    /// BOOTP transaction identifier matched to the probe.
    pub transaction_id: u32,
    /// Client hardware address matched to the probing interface.
    pub client_hardware_address: MacAddress,
    /// IPv4 address proposed for the client (`yiaddr`).
    pub offered_address: Ipv4Addr,
    /// DHCP server identifier option, when advertised.
    pub server_identifier: Option<Ipv4Addr>,
    /// Subnet mask option, when advertised.
    pub subnet_mask: Option<Ipv4Addr>,
    /// Router options, in advertised order.
    pub routers: Vec<Ipv4Addr>,
    /// DNS server options, in advertised order.
    pub dns_servers: Vec<Ipv4Addr>,
    /// Domain name option, when advertised and valid.
    pub domain_name: Option<String>,
    /// Domain-search names, in advertised order.
    pub domain_search: Vec<String>,
    /// Lease duration in seconds, when advertised.
    pub lease_time_seconds: Option<u32>,
    /// Renewal (`T1`) duration in seconds, when advertised.
    pub renewal_time_seconds: Option<u32>,
    /// Rebinding (`T2`) duration in seconds, when advertised.
    pub rebinding_time_seconds: Option<u32>,
    /// Interface MTU, when advertised.
    pub interface_mtu: Option<u16>,
    /// Classless static routes, in advertised order.
    pub classless_static_routes: Vec<ClasslessStaticRoute>,
}
