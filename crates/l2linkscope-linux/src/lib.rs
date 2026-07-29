//! Safe Linux acquisition for L2LinkScope.
//!
//! This crate inventories Linux interfaces and performs a bounded DHCPv4
//! Discover/Offer exchange. It never applies an address, route, DNS setting,
//! interface flag, or lease. Protocol bytes remain owned by
//! `l2linkscope-protocols`; normalized results remain owned by
//! `l2linkscope-core`.
//!
//! The DHCP transport uses a UDP socket bound to both port 68 and the selected
//! interface. On Linux this normally requires `CAP_NET_BIND_SERVICE` and
//! `CAP_NET_RAW` (or root).
//! An existing DHCP client which already owns the socket is reported as a
//! conflict and is never stopped or reconfigured.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::fmt;
use std::time::Duration;

use l2linkscope_core::{DiscoveryError, DiscoveryErrorCode, DiscoverySnapshot, Interface};

/// Default bounded DHCP Offer collection window.
pub const DEFAULT_DHCP_V4_TIMEOUT: Duration = Duration::from_secs(5);
/// Smallest accepted DHCP Offer collection window.
pub const MIN_DHCP_V4_TIMEOUT: Duration = Duration::from_millis(250);
/// Largest accepted DHCP Offer collection window.
pub const MAX_DHCP_V4_TIMEOUT: Duration = Duration::from_secs(30);

/// Options for one explicitly requested DHCPv4 probe.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DhcpV4ProbeOptions {
    /// Time spent collecting zero or more matching Offers after one Discover.
    pub timeout: Duration,
}

impl DhcpV4ProbeOptions {
    /// Validates and constructs probe options.
    ///
    /// # Errors
    ///
    /// Returns [`LinuxError::InvalidTimeout`] when `timeout` is outside the
    /// documented 250 millisecond through 30 second bounds.
    pub fn new(timeout: Duration) -> Result<Self, LinuxError> {
        if !(MIN_DHCP_V4_TIMEOUT..=MAX_DHCP_V4_TIMEOUT).contains(&timeout) {
            return Err(LinuxError::InvalidTimeout(timeout));
        }
        Ok(Self { timeout })
    }
}

impl Default for DhcpV4ProbeOptions {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_DHCP_V4_TIMEOUT,
        }
    }
}

/// A normalized Linux acquisition or transport failure.
#[derive(Debug)]
#[non_exhaustive]
pub enum LinuxError {
    /// The requested interface name was not present.
    InterfaceNotFound(String),
    /// The interface cannot safely support this probe.
    UnsupportedInterface(String),
    /// The process lacks the privilege needed to bind the DHCP client port.
    InsufficientPrivileges(String),
    /// Another process, commonly a DHCP client, already owns the needed socket.
    SocketConflict(String),
    /// A kernel inventory or socket operation failed.
    Transport(String),
    /// Responses were received, but every matching response was malformed.
    MalformedResponses(usize),
    /// The requested collection duration is outside the enforced bounds.
    InvalidTimeout(Duration),
    /// An invariant such as timestamp or random-number generation failed.
    Internal(String),
    /// Live acquisition was requested on a non-Linux build.
    UnsupportedPlatform,
}

impl LinuxError {
    /// Returns the stable public error category for this failure.
    #[must_use]
    pub const fn code(&self) -> DiscoveryErrorCode {
        match self {
            Self::InterfaceNotFound(_) => DiscoveryErrorCode::InterfaceNotFound,
            Self::UnsupportedInterface(_) | Self::UnsupportedPlatform => {
                DiscoveryErrorCode::UnsupportedInterface
            }
            Self::InsufficientPrivileges(_) => DiscoveryErrorCode::InsufficientPrivileges,
            Self::SocketConflict(_) | Self::Transport(_) => DiscoveryErrorCode::TransportFailure,
            Self::MalformedResponses(_) => DiscoveryErrorCode::MalformedResponses,
            Self::InvalidTimeout(_) => DiscoveryErrorCode::InvalidInput,
            Self::Internal(_) => DiscoveryErrorCode::Internal,
        }
    }

    /// Converts the acquisition failure to the portable structured error model.
    #[must_use]
    pub fn as_discovery_error(&self) -> DiscoveryError {
        DiscoveryError::new(self.code(), self.to_string())
    }
}

impl fmt::Display for LinuxError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InterfaceNotFound(name) => write!(formatter, "interface '{name}' was not found"),
            Self::UnsupportedInterface(reason) => formatter.write_str(reason),
            Self::InsufficientPrivileges(detail) => write!(
                formatter,
                "insufficient privileges for DHCPv4 probing: {detail}; run as root or grant CAP_NET_BIND_SERVICE and CAP_NET_RAW"
            ),
            Self::SocketConflict(detail) => write!(
                formatter,
                "DHCP client socket conflict: {detail}; an existing network manager or DHCP client may own UDP port 68"
            ),
            Self::Transport(detail) => write!(formatter, "Linux transport failure: {detail}"),
            Self::MalformedResponses(count) => write!(
                formatter,
                "received {count} matching DHCP response(s), but none was a valid Offer"
            ),
            Self::InvalidTimeout(value) => write!(
                formatter,
                "timeout {} ms is outside the supported range of {} through {} ms",
                value.as_millis(),
                MIN_DHCP_V4_TIMEOUT.as_millis(),
                MAX_DHCP_V4_TIMEOUT.as_millis()
            ),
            Self::Internal(detail) => write!(formatter, "internal acquisition error: {detail}"),
            Self::UnsupportedPlatform => {
                formatter.write_str("live interface acquisition is supported only on Linux")
            }
        }
    }
}

impl std::error::Error for LinuxError {}

/// Returns normalized metadata for every Linux interface.
///
/// This read-only operation does not require DHCP or raw-socket privileges.
///
/// # Errors
///
/// Returns a structured error if kernel interface metadata cannot be read.
pub fn interfaces() -> Result<Vec<Interface>, LinuxError> {
    platform::interfaces()
}

/// Sends one DHCP Discover on `interface_name` and collects matching Offers.
///
/// The operation sends no DHCP Request, Decline, Release, or Inform and performs
/// no configuration syscall. A completed probe with no Offers is represented by
/// an empty `observations` array in the returned snapshot.
///
/// # Errors
///
/// Returns a structured error for an invalid interface, insufficient privilege,
/// an existing client socket conflict, malformed-only matching responses, or a
/// Linux transport failure.
pub fn probe_dhcp_v4(
    interface_name: &str,
    options: DhcpV4ProbeOptions,
) -> Result<DiscoverySnapshot, LinuxError> {
    DhcpV4ProbeOptions::new(options.timeout)?;
    platform::probe_dhcp_v4(interface_name, options)
}

#[cfg(target_os = "linux")]
mod platform {
    use std::collections::{HashMap, HashSet};
    use std::fmt;
    use std::fs;
    use std::io;
    use std::net::{IpAddr, Ipv4Addr, SocketAddrV4, UdpSocket};
    use std::path::Path;
    use std::time::Instant;

    use if_addrs::IfAddr;
    use l2linkscope_core::{
        AdministrativeState, CarrierState, DiscoveryMethod, DiscoverySession, DiscoverySessionId,
        DiscoverySnapshot, DiscoveryWarning, DiscoveryWarningCode, Interface, InterfaceAddress,
        InterfaceId, MacAddress, Observation, ObservationId, ObservationTimestamp,
        OperationalState, ProbeSupport,
    };
    use l2linkscope_protocols::dhcpv4::{
        DhcpDiscover, MAX_DHCP_V4_MESSAGE_SIZE, OfferExpectation, ParseError, parse_offer,
    };
    use socket2::{Domain, Protocol, SockAddr, Socket, Type};

    use super::{DhcpV4ProbeOptions, LinuxError};

    const DHCP_CLIENT_PORT: u16 = 68;
    const DHCP_SERVER_PORT: u16 = 67;
    const MAX_OFFERS: usize = 32;
    // The extra byte makes every oversized datagram remain oversized after
    // kernel truncation, so the protocol parser rejects it.
    const RECEIVE_BUFFER_SIZE: usize = MAX_DHCP_V4_MESSAGE_SIZE + 1;
    const IFF_UP: u32 = 0x1;
    const IFF_LOOPBACK: u32 = 0x8;
    const ARPHRD_ETHER: i32 = 1;

    pub(super) fn interfaces() -> Result<Vec<Interface>, LinuxError> {
        let mut addresses: HashMap<String, Vec<InterfaceAddress>> = HashMap::new();
        let kernel_addresses = if_addrs::get_if_addrs()
            .map_err(|error| LinuxError::Transport(format!("getifaddrs failed: {error}")))?;
        for entry in kernel_addresses {
            let normalized = match entry.addr {
                IfAddr::V4(address) => InterfaceAddress {
                    address: IpAddr::V4(address.ip),
                    prefix_length: ipv4_prefix_length(address.netmask),
                },
                IfAddr::V6(address) => InterfaceAddress {
                    address: IpAddr::V6(address.ip),
                    prefix_length: ipv6_prefix_length(address.netmask.octets()),
                },
            };
            addresses.entry(entry.name).or_default().push(normalized);
        }

        let directory = fs::read_dir("/sys/class/net").map_err(|error| {
            LinuxError::Transport(format!("cannot enumerate /sys/class/net: {error}"))
        })?;
        let mut result = Vec::new();
        for entry in directory {
            let entry = entry.map_err(|error| {
                LinuxError::Transport(format!("cannot read interface directory entry: {error}"))
            })?;
            let name = entry.file_name().into_string().map_err(|_| {
                LinuxError::Transport("kernel returned a non-UTF-8 interface name".to_owned())
            })?;
            let path = entry.path();
            let index = read_required::<u32>(&path, "ifindex")?;
            let mtu = read_required::<u32>(&path, "mtu")?;
            let flags = read_hex_u32(&path.join("flags"))?;
            let hardware_type = read_required::<i32>(&path, "type")?;
            let mac = read_optional_mac(&path.join("address"));
            let administrative_state = if flags & IFF_UP != 0 {
                AdministrativeState::Up
            } else {
                AdministrativeState::Down
            };
            let operational_state = read_trimmed(&path.join("operstate"))
                .map_or(OperationalState::Unknown, |value| {
                    map_operational_state(&value)
                });
            let carrier_state =
                read_trimmed(&path.join("carrier")).map_or(CarrierState::Unknown, |value| {
                    match value.as_str() {
                        "1" => CarrierState::Present,
                        "0" => CarrierState::Absent,
                        _ => CarrierState::Unknown,
                    }
                });
            let is_loopback = flags & IFF_LOOPBACK != 0;
            let dhcp_v4_probe = assess_probe_support(
                is_loopback,
                hardware_type,
                mac,
                administrative_state,
                operational_state,
                carrier_state,
            );
            let mut interface_addresses = addresses.remove(&name).unwrap_or_default();
            interface_addresses.sort_by_key(|address| (address.address, address.prefix_length));
            interface_addresses.dedup();
            result.push(Interface {
                id: InterfaceId {
                    index,
                    hardware_address: mac,
                },
                name,
                administrative_state,
                operational_state,
                carrier_state,
                mtu,
                addresses: interface_addresses,
                is_loopback,
                dhcp_v4_probe,
            });
        }
        result.sort_by_key(|interface| interface.id.index);
        Ok(result)
    }

    pub(super) fn probe_dhcp_v4(
        interface_name: &str,
        options: DhcpV4ProbeOptions,
    ) -> Result<DiscoverySnapshot, LinuxError> {
        let interface = interfaces()?
            .into_iter()
            .find(|candidate| candidate.name == interface_name)
            .ok_or_else(|| LinuxError::InterfaceNotFound(interface_name.to_owned()))?;
        if !interface.dhcp_v4_probe.supported {
            return Err(LinuxError::UnsupportedInterface(
                interface
                    .dhcp_v4_probe
                    .reason
                    .clone()
                    .unwrap_or_else(|| format!("interface '{interface_name}' is unsupported")),
            ));
        }
        let mac = interface.id.hardware_address.ok_or_else(|| {
            LinuxError::UnsupportedInterface(format!(
                "interface '{interface_name}' has no six-octet Ethernet address"
            ))
        })?;
        let started_at =
            ObservationTimestamp::now().map_err(|error| LinuxError::Internal(error.to_string()))?;
        let session_id = DiscoverySessionId::new();
        let transaction_id = generate_transaction_id()?;
        let discover = DhcpDiscover::new(transaction_id, mac.octets()).encode();
        let socket = create_socket(interface_name)?;
        socket
            .send_to(
                &discover,
                SocketAddrV4::new(Ipv4Addr::BROADCAST, DHCP_SERVER_PORT),
            )
            .map_err(|error| map_socket_error("send DHCP Discover", error))?;

        let deadline = Instant::now() + options.timeout;
        let expectation = OfferExpectation {
            transaction_id,
            client_hardware_address: mac.octets(),
        };
        let mut offers = Vec::new();
        let mut server_sources = HashSet::new();
        let mut malformed_responses = 0_usize;
        let mut reached_limit = false;
        let mut buffer = [0_u8; RECEIVE_BUFFER_SIZE];

        loop {
            let now = Instant::now();
            if now >= deadline {
                break;
            }
            socket
                .set_read_timeout(Some(deadline.saturating_duration_since(now)))
                .map_err(|error| map_socket_error("set receive timeout", error))?;
            match socket.recv_from(&mut buffer) {
                Ok((length, source)) => match parse_offer(&buffer[..length], expectation) {
                    Ok(offer) => {
                        if let IpAddr::V4(address) = source.ip() {
                            server_sources.insert(address);
                        }
                        if !offers
                            .iter()
                            .any(|(existing, _observed_at)| existing == &offer)
                        {
                            if offers.len() == MAX_OFFERS {
                                reached_limit = true;
                                break;
                            }
                            let observed_at = ObservationTimestamp::now()
                                .map_err(|error| LinuxError::Internal(error.to_string()))?;
                            offers.push((offer, observed_at));
                        }
                    }
                    Err(ParseError::WrongTransactionId { .. })
                    | Err(ParseError::WrongClientHardwareAddress { .. }) => {}
                    Err(_) => malformed_responses = malformed_responses.saturating_add(1),
                },
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                    ) =>
                {
                    break;
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) => return Err(map_socket_error("receive DHCP Offer", error)),
            }
        }

        if offers.is_empty() && malformed_responses > 0 {
            return Err(LinuxError::MalformedResponses(malformed_responses));
        }

        let current_interface = interfaces()?
            .into_iter()
            .find(|candidate| candidate.name == interface_name)
            .ok_or_else(|| {
                LinuxError::Transport(format!(
                    "interface '{interface_name}' disappeared during the probe"
                ))
            })?;
        if current_interface.id != interface.id {
            return Err(LinuxError::Transport(format!(
                "interface '{interface_name}' changed identity during the probe"
            )));
        }
        if !current_interface.dhcp_v4_probe.supported {
            return Err(LinuxError::Transport(format!(
                "interface '{interface_name}' became unavailable during the probe"
            )));
        }

        let completed_at =
            ObservationTimestamp::now().map_err(|error| LinuxError::Internal(error.to_string()))?;
        let session = DiscoverySession {
            id: session_id,
            interface_id: Some(interface.id.clone()),
            method: DiscoveryMethod::DhcpV4Probe,
            started_at,
            completed_at: Some(completed_at),
        };
        let mut snapshot = DiscoverySnapshot::new(session);
        snapshot.interfaces.push(interface.clone());
        for (offer, observed_at) in offers {
            snapshot.observations.push(Observation::dhcp_v4_offer(
                ObservationId::new(),
                session_id,
                interface.id.clone(),
                observed_at,
                offer.into_normalized(),
            ));
        }
        let advertised_servers: HashSet<_> = snapshot
            .observations
            .iter()
            .filter_map(|observation| match observation.kind() {
                l2linkscope_core::ObservationKind::DhcpV4Offer(offer) => offer.server_identifier,
            })
            .collect();
        if advertised_servers.len() > 1 || server_sources.len() > 1 {
            snapshot.warnings.push(DiscoveryWarning::new(
                DiscoveryWarningCode::MultipleDhcpServers,
                "multiple DHCP servers advertised distinct Offers",
            ));
        }
        if malformed_responses > 0 {
            snapshot.warnings.push(DiscoveryWarning::new(
                DiscoveryWarningCode::MalformedResponse,
                format!("ignored {malformed_responses} malformed matching response(s)"),
            ));
        }
        if reached_limit {
            snapshot.warnings.push(DiscoveryWarning::new(
                DiscoveryWarningCode::CollectionLimitReached,
                format!("stopped after the defensive limit of {MAX_OFFERS} Offers"),
            ));
        }
        Ok(snapshot)
    }

    fn create_socket(interface_name: &str) -> Result<UdpSocket, LinuxError> {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
            .map_err(|error| map_socket_error("create UDP socket", error))?;
        socket
            .bind_device(Some(interface_name.as_bytes()))
            .map_err(|error| map_socket_error("bind socket to selected interface", error))?;
        socket
            .set_broadcast(true)
            .map_err(|error| map_socket_error("enable broadcast", error))?;
        let address = SockAddr::from(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, DHCP_CLIENT_PORT));
        socket
            .bind(&address)
            .map_err(|error| map_socket_error("bind UDP port 68", error))?;
        Ok(socket.into())
    }

    fn map_socket_error(operation: &str, error: io::Error) -> LinuxError {
        match error.kind() {
            io::ErrorKind::PermissionDenied => {
                LinuxError::InsufficientPrivileges(format!("{operation}: {error}"))
            }
            io::ErrorKind::AddrInUse => LinuxError::SocketConflict(format!("{operation}: {error}")),
            _ => LinuxError::Transport(format!("{operation}: {error}")),
        }
    }

    fn generate_transaction_id() -> Result<u32, LinuxError> {
        let mut bytes = [0_u8; 4];
        getrandom::fill(&mut bytes).map_err(|error| {
            LinuxError::Internal(format!("secure transaction ID generation failed: {error}"))
        })?;
        Ok(u32::from_be_bytes(bytes))
    }

    fn read_required<T>(directory: &Path, file: &str) -> Result<T, LinuxError>
    where
        T: std::str::FromStr,
        T::Err: fmt::Display,
    {
        let path = directory.join(file);
        let value = read_trimmed(&path).map_err(|error| {
            LinuxError::Transport(format!("cannot read {}: {error}", path.display()))
        })?;
        value.parse::<T>().map_err(|error| {
            LinuxError::Transport(format!(
                "invalid {} value '{value}': {error}",
                path.display()
            ))
        })
    }

    fn read_hex_u32(path: &Path) -> Result<u32, LinuxError> {
        let value = read_trimmed(path).map_err(|error| {
            LinuxError::Transport(format!("cannot read {}: {error}", path.display()))
        })?;
        let digits = value.strip_prefix("0x").unwrap_or(&value);
        u32::from_str_radix(digits, 16).map_err(|error| {
            LinuxError::Transport(format!(
                "invalid {} value '{value}': {error}",
                path.display()
            ))
        })
    }

    fn read_trimmed(path: &Path) -> io::Result<String> {
        fs::read_to_string(path).map(|value| value.trim().to_owned())
    }

    fn read_optional_mac(path: &Path) -> Option<MacAddress> {
        read_trimmed(path).ok()?.parse().ok()
    }

    fn map_operational_state(value: &str) -> OperationalState {
        match value {
            "up" => OperationalState::Up,
            "down" => OperationalState::Down,
            "dormant" => OperationalState::Dormant,
            "lowerlayerdown" => OperationalState::LowerLayerDown,
            "notpresent" => OperationalState::NotPresent,
            "testing" => OperationalState::Testing,
            _ => OperationalState::Unknown,
        }
    }

    fn assess_probe_support(
        is_loopback: bool,
        hardware_type: i32,
        mac: Option<MacAddress>,
        administrative_state: AdministrativeState,
        operational_state: OperationalState,
        carrier_state: CarrierState,
    ) -> ProbeSupport {
        if is_loopback {
            ProbeSupport::unsupported("loopback interfaces do not support Ethernet DHCPv4")
        } else if hardware_type != ARPHRD_ETHER {
            ProbeSupport::unsupported("interface is not an Ethernet-compatible link")
        } else if mac.is_none() {
            ProbeSupport::unsupported("interface has no six-octet Ethernet address")
        } else if administrative_state != AdministrativeState::Up {
            ProbeSupport::unsupported("interface is administratively down")
        } else if carrier_state == CarrierState::Absent {
            ProbeSupport::unsupported("interface has no carrier")
        } else if matches!(
            operational_state,
            OperationalState::Down
                | OperationalState::LowerLayerDown
                | OperationalState::NotPresent
        ) {
            ProbeSupport::unsupported("interface is not operational")
        } else {
            ProbeSupport::supported()
        }
    }

    fn ipv4_prefix_length(mask: Ipv4Addr) -> u8 {
        mask.octets()
            .iter()
            .map(|octet| octet.count_ones() as u8)
            .sum()
    }

    fn ipv6_prefix_length(mask: [u8; 16]) -> u8 {
        mask.iter().map(|octet| octet.count_ones() as u8).sum()
    }

    #[cfg(test)]
    mod tests {
        use std::net::{Ipv4Addr, Ipv6Addr};

        use l2linkscope_core::{AdministrativeState, CarrierState, MacAddress, OperationalState};

        use super::{
            MAX_DHCP_V4_MESSAGE_SIZE, RECEIVE_BUFFER_SIZE, assess_probe_support,
            ipv4_prefix_length, ipv6_prefix_length,
        };

        #[test]
        fn receive_buffer_exposes_oversized_datagrams_to_parser() {
            assert_eq!(RECEIVE_BUFFER_SIZE, MAX_DHCP_V4_MESSAGE_SIZE + 1);
        }

        #[test]
        fn normalizes_ipv4_and_ipv6_prefix_lengths() {
            assert_eq!(ipv4_prefix_length(Ipv4Addr::new(255, 255, 254, 0)), 23);
            assert_eq!(
                ipv6_prefix_length(
                    Ipv6Addr::new(0xffff, 0xffff, 0xffff, 0xffff, 0, 0, 0, 0).octets()
                ),
                64
            );
        }

        #[test]
        fn rejects_loopback_and_down_interfaces() {
            let mac = Some(MacAddress::new([0, 1, 2, 3, 4, 5]));
            assert!(
                !assess_probe_support(
                    true,
                    1,
                    mac,
                    AdministrativeState::Up,
                    OperationalState::Up,
                    CarrierState::Present
                )
                .supported
            );
            assert!(
                !assess_probe_support(
                    false,
                    1,
                    mac,
                    AdministrativeState::Down,
                    OperationalState::Down,
                    CarrierState::Absent
                )
                .supported
            );
            assert!(
                assess_probe_support(
                    false,
                    1,
                    mac,
                    AdministrativeState::Up,
                    OperationalState::Up,
                    CarrierState::Present
                )
                .supported
            );
        }
    }
}

#[cfg(not(target_os = "linux"))]
mod platform {
    use l2linkscope_core::{DiscoverySnapshot, Interface};

    use super::{DhcpV4ProbeOptions, LinuxError};

    pub(super) fn interfaces() -> Result<Vec<Interface>, LinuxError> {
        Err(LinuxError::UnsupportedPlatform)
    }

    pub(super) fn probe_dhcp_v4(
        _interface_name: &str,
        _options: DhcpV4ProbeOptions,
    ) -> Result<DiscoverySnapshot, LinuxError> {
        Err(LinuxError::UnsupportedPlatform)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use l2linkscope_core::DiscoveryErrorCode;

    use super::{DhcpV4ProbeOptions, LinuxError};

    #[test]
    fn timeout_bounds_are_enforced() {
        assert!(DhcpV4ProbeOptions::new(Duration::from_millis(249)).is_err());
        assert!(DhcpV4ProbeOptions::new(Duration::from_millis(250)).is_ok());
        assert!(DhcpV4ProbeOptions::new(Duration::from_secs(30)).is_ok());
        assert!(DhcpV4ProbeOptions::new(Duration::from_millis(30_001)).is_err());
    }

    #[test]
    fn errors_map_to_stable_categories() {
        assert_eq!(
            LinuxError::InterfaceNotFound("eth0".to_owned()).code(),
            DiscoveryErrorCode::InterfaceNotFound
        );
        assert_eq!(
            LinuxError::MalformedResponses(1).code(),
            DiscoveryErrorCode::MalformedResponses
        );
    }
}
