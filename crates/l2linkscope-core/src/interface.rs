use std::fmt;
use std::net::IpAddr;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

/// An IEEE 802 hardware address.
///
/// JSON uses canonical lower-case colon-separated notation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MacAddress([u8; 6]);

impl MacAddress {
    /// Constructs a MAC address from its six octets.
    #[must_use]
    pub const fn new(octets: [u8; 6]) -> Self {
        Self(octets)
    }

    /// Returns the six address octets.
    #[must_use]
    pub const fn octets(self) -> [u8; 6] {
        self.0
    }
}

impl fmt::Display for MacAddress {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [a, b, c, d, e, f] = self.0;
        write!(formatter, "{a:02x}:{b:02x}:{c:02x}:{d:02x}:{e:02x}:{f:02x}")
    }
}

impl FromStr for MacAddress {
    type Err = MacAddressParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut octets = [0_u8; 6];
        let mut parts = value.split(':');
        for octet in &mut octets {
            let part = parts.next().ok_or(MacAddressParseError)?;
            if part.len() != 2 {
                return Err(MacAddressParseError);
            }
            *octet = u8::from_str_radix(part, 16).map_err(|_| MacAddressParseError)?;
        }
        if parts.next().is_some() {
            return Err(MacAddressParseError);
        }
        Ok(Self(octets))
    }
}

impl Serialize for MacAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for MacAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(de::Error::custom)
    }
}

/// Error parsing a colon-separated MAC address.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MacAddressParseError;

impl fmt::Display for MacAddressParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("expected a six-octet colon-separated MAC address")
    }
}

impl std::error::Error for MacAddressParseError {}

/// Runtime identity of an operating-system network interface.
///
/// Linux acquisition should supply the interface index and available hardware
/// address. This is more robust than a mutable interface name, but persistent
/// identity across reboots is not guaranteed in L2LinkScope 0.1.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct InterfaceId {
    /// Operating-system interface index.
    pub index: u32,
    /// Hardware address when the interface reports one.
    pub hardware_address: Option<MacAddress>,
}

/// Administrative state configured on an interface.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdministrativeState {
    /// The interface is administratively enabled.
    Up,
    /// The interface is administratively disabled.
    Down,
    /// The acquisition layer could not determine the state.
    Unknown,
}

/// Operational state reported by an interface.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationalState {
    /// The link is operational.
    Up,
    /// The link is not operational.
    Down,
    /// The link is waiting for an external event.
    Dormant,
    /// A lower-layer link is unavailable.
    LowerLayerDown,
    /// The interface is not present.
    NotPresent,
    /// The interface is in a test state.
    Testing,
    /// The acquisition layer could not map the reported state.
    Unknown,
}

/// Physical carrier state for an interface.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CarrierState {
    /// Carrier is present.
    Present,
    /// Carrier is absent.
    Absent,
    /// Carrier state is unavailable.
    Unknown,
}

/// A configured interface address and prefix length.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct InterfaceAddress {
    /// Configured IPv4 or IPv6 address.
    pub address: IpAddr,
    /// Network prefix length reported by the operating system.
    pub prefix_length: u8,
}

/// Whether an interface appears usable for a DHCPv4 probe.
///
/// This is an eligibility assessment, not proof that a probe will succeed.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProbeSupport {
    /// Whether acquisition considers the probe supported.
    pub supported: bool,
    /// Stable, human-readable reason when probing is not supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl ProbeSupport {
    /// Creates a supported assessment.
    #[must_use]
    pub const fn supported() -> Self {
        Self {
            supported: true,
            reason: None,
        }
    }

    /// Creates an unsupported assessment with an explanatory reason.
    #[must_use]
    pub fn unsupported(reason: impl Into<String>) -> Self {
        Self {
            supported: false,
            reason: Some(reason.into()),
        }
    }
}

/// Normalized metadata for one network interface.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Interface {
    /// Runtime interface identity.
    pub id: InterfaceId,
    /// Kernel or operating-system interface name.
    pub name: String,
    /// Administrative state.
    pub administrative_state: AdministrativeState,
    /// Operational link state.
    pub operational_state: OperationalState,
    /// Physical carrier state.
    pub carrier_state: CarrierState,
    /// Interface maximum transmission unit in octets.
    pub mtu: u32,
    /// Existing configured IPv4 and IPv6 addresses.
    pub addresses: Vec<InterfaceAddress>,
    /// Whether the operating system identifies this as a loopback interface.
    pub is_loopback: bool,
    /// Whether a bounded Ethernet DHCPv4 probe appears supported.
    pub dhcp_v4_probe: ProbeSupport,
}
