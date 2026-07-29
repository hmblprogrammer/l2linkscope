//! DHCPv4 Discover encoding and Offer parsing.
//!
//! The encoder deliberately emits only DHCP Discover. In particular, this
//! module has no API for DHCP Request, Decline, Release, or Inform messages.

use std::error::Error;
use std::fmt;
use std::net::Ipv4Addr;
use std::str;

const BOOTP_FIXED_LENGTH: usize = 236;
const OPTIONS_OFFSET: usize = 240;
const MINIMUM_MESSAGE_LENGTH: usize = 300;
/// Largest DHCPv4 UDP payload accepted by the defensive parser.
pub const MAX_DHCP_V4_MESSAGE_SIZE: usize = 4_096;
const MAX_OPTION_INSTANCES: usize = 128;
const MAX_LIST_ITEMS: usize = 32;
const MAX_DOMAIN_NAMES: usize = 16;
const DHCP_MAGIC_COOKIE: [u8; 4] = [99, 130, 83, 99];

const OPTION_PAD: u8 = 0;
const OPTION_SUBNET_MASK: u8 = 1;
const OPTION_ROUTER: u8 = 3;
const OPTION_DNS_SERVER: u8 = 6;
#[cfg(test)]
const OPTION_HOST_NAME: u8 = 12;
const OPTION_DOMAIN_NAME: u8 = 15;
const OPTION_INTERFACE_MTU: u8 = 26;
const OPTION_LEASE_TIME: u8 = 51;
const OPTION_MESSAGE_TYPE: u8 = 53;
const OPTION_SERVER_IDENTIFIER: u8 = 54;
const OPTION_PARAMETER_REQUEST_LIST: u8 = 55;
const OPTION_RENEWAL_TIME: u8 = 58;
const OPTION_REBINDING_TIME: u8 = 59;
const OPTION_DOMAIN_SEARCH: u8 = 119;
const OPTION_CLASSLESS_STATIC_ROUTE: u8 = 121;
const OPTION_END: u8 = 255;

const DHCPDISCOVER: u8 = 1;
const DHCPOFFER: u8 = 2;

/// DHCP option codes requested by the default Discover message.
///
/// The list requests common network configuration while omitting host names
/// and other unnecessary client-identifying data.
pub const DEFAULT_PARAMETER_REQUEST_LIST: &[u8] = &[
    OPTION_SUBNET_MASK,
    OPTION_ROUTER,
    OPTION_DNS_SERVER,
    OPTION_DOMAIN_NAME,
    OPTION_DOMAIN_SEARCH,
    OPTION_INTERFACE_MTU,
    OPTION_LEASE_TIME,
    OPTION_SERVER_IDENTIFIER,
    OPTION_RENEWAL_TIME,
    OPTION_REBINDING_TIME,
    OPTION_CLASSLESS_STATIC_ROUTE,
];

/// Description of a DHCPv4 Discover message to encode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DhcpDiscover {
    transaction_id: u32,
    client_hardware_address: [u8; 6],
    broadcast: bool,
}

impl DhcpDiscover {
    /// Creates a broadcast DHCP Discover for `client_hardware_address`.
    ///
    /// The caller must generate `transaction_id` with a cryptographically
    /// secure random-number generator. Randomness is intentionally outside the
    /// protocol-byte layer.
    #[must_use]
    pub const fn new(transaction_id: u32, client_hardware_address: [u8; 6]) -> Self {
        Self {
            transaction_id,
            client_hardware_address,
            broadcast: true,
        }
    }

    /// Selects whether the BOOTP broadcast flag is set.
    #[must_use]
    pub const fn with_broadcast(mut self, broadcast: bool) -> Self {
        self.broadcast = broadcast;
        self
    }

    /// Returns the transaction ID encoded by this message.
    #[must_use]
    pub const fn transaction_id(&self) -> u32 {
        self.transaction_id
    }

    /// Returns the client Ethernet hardware address encoded by this message.
    #[must_use]
    pub const fn client_hardware_address(&self) -> [u8; 6] {
        self.client_hardware_address
    }

    /// Encodes this DHCP Discover as a BOOTP/DHCP UDP payload.
    ///
    /// The result is padded to 300 octets for compatibility with BOOTP-era
    /// receivers. It contains a Discover message type, the documented default
    /// parameter-request list, and an end option. It never contains a host-name
    /// option.
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut message = vec![0_u8; OPTIONS_OFFSET];
        message[0] = 1; // BOOTREQUEST
        message[1] = 1; // Ethernet
        message[2] = 6;
        message[4..8].copy_from_slice(&self.transaction_id.to_be_bytes());
        if self.broadcast {
            message[10..12].copy_from_slice(&0x8000_u16.to_be_bytes());
        }
        message[28..34].copy_from_slice(&self.client_hardware_address);
        message[BOOTP_FIXED_LENGTH..OPTIONS_OFFSET].copy_from_slice(&DHCP_MAGIC_COOKIE);

        message.extend_from_slice(&[OPTION_MESSAGE_TYPE, 1, DHCPDISCOVER]);
        message.push(OPTION_PARAMETER_REQUEST_LIST);
        message.push(DEFAULT_PARAMETER_REQUEST_LIST.len() as u8);
        message.extend_from_slice(DEFAULT_PARAMETER_REQUEST_LIST);
        message.push(OPTION_END);
        message.resize(MINIMUM_MESSAGE_LENGTH, OPTION_PAD);
        message
    }
}

/// Values used to associate a received reply with a Discover.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OfferExpectation {
    /// Transaction ID sent in the Discover.
    pub transaction_id: u32,
    /// Ethernet address sent in the Discover `chaddr` field.
    pub client_hardware_address: [u8; 6],
}

/// One route advertised through DHCP option 121.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClasslessStaticRoute {
    /// Network address with host bits cleared according to `prefix_length`.
    pub destination: Ipv4Addr,
    /// CIDR prefix length in the range 0 through 32.
    pub prefix_length: u8,
    /// Router advertised for the destination prefix.
    pub router: Ipv4Addr,
}

/// A validated DHCPv4 Offer decoded from a UDP payload.
///
/// Every configuration field in this type is peer-advertised data. Parsing an
/// Offer does not imply that the data is trusted, accepted, or configured.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedDhcpV4Offer {
    /// Transaction ID copied from the corresponding Discover.
    pub transaction_id: u32,
    /// Client Ethernet address copied from the corresponding Discover.
    pub client_hardware_address: [u8; 6],
    /// IPv4 address proposed in the BOOTP `yiaddr` field.
    pub offered_address: Ipv4Addr,
    /// DHCP server identifier option, when advertised.
    pub server_identifier: Option<Ipv4Addr>,
    /// Subnet mask option, when advertised.
    pub subnet_mask: Option<Ipv4Addr>,
    /// Router addresses, in wire order.
    pub routers: Vec<Ipv4Addr>,
    /// DNS server addresses, in wire order.
    pub dns_servers: Vec<Ipv4Addr>,
    /// Domain name option, when advertised and valid UTF-8.
    pub domain_name: Option<String>,
    /// RFC 3397 domain-search names, in wire order.
    pub domain_search: Vec<String>,
    /// Lease duration in seconds, when advertised.
    pub lease_time_seconds: Option<u32>,
    /// Renewal time in seconds, when advertised.
    pub renewal_time_seconds: Option<u32>,
    /// Rebinding time in seconds, when advertised.
    pub rebinding_time_seconds: Option<u32>,
    /// Interface MTU, when advertised.
    pub interface_mtu: Option<u16>,
    /// RFC 3442 classless static routes, in wire order.
    pub classless_static_routes: Vec<ClasslessStaticRoute>,
}

impl ParsedDhcpV4Offer {
    /// Converts the validated protocol result into the portable core model.
    ///
    /// All configuration remains peer-advertised and untrusted; this conversion
    /// does not imply that any value was accepted or applied.
    #[must_use]
    pub fn into_normalized(self) -> l2linkscope_core::DhcpV4Offer {
        l2linkscope_core::DhcpV4Offer {
            transaction_id: self.transaction_id,
            client_hardware_address: l2linkscope_core::MacAddress::new(
                self.client_hardware_address,
            ),
            offered_address: self.offered_address,
            server_identifier: self.server_identifier,
            subnet_mask: self.subnet_mask,
            routers: self.routers,
            dns_servers: self.dns_servers,
            domain_name: self.domain_name,
            domain_search: self.domain_search,
            lease_time_seconds: self.lease_time_seconds,
            renewal_time_seconds: self.renewal_time_seconds,
            rebinding_time_seconds: self.rebinding_time_seconds,
            interface_mtu: self.interface_mtu,
            classless_static_routes: self
                .classless_static_routes
                .into_iter()
                .map(|route| l2linkscope_core::ClasslessStaticRoute {
                    destination: l2linkscope_core::Ipv4Network {
                        address: route.destination,
                        prefix_length: route.prefix_length,
                    },
                    router: route.router,
                })
                .collect(),
        }
    }
}

/// Structured DHCPv4 parsing failure.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ParseError {
    /// The UDP payload is shorter than the fixed BOOTP header and magic cookie.
    Truncated {
        /// Received payload length.
        actual: usize,
        /// Smallest payload length the parser can inspect.
        minimum: usize,
    },
    /// The payload exceeds the parser's defensive size limit.
    PacketTooLarge {
        /// Received payload length.
        actual: usize,
        /// Largest payload length accepted by the parser.
        maximum: usize,
    },
    /// The BOOTP operation is not `BOOTREPLY`.
    NotBootReply {
        /// Operation value found on the wire.
        actual: u8,
    },
    /// The hardware type is not Ethernet.
    UnsupportedHardwareType {
        /// Hardware type found on the wire.
        actual: u8,
    },
    /// The BOOTP hardware-address length is not six octets.
    InvalidHardwareAddressLength {
        /// Hardware-address length found on the wire.
        actual: u8,
    },
    /// The DHCP magic cookie is absent or incorrect.
    MissingMagicCookie,
    /// The reply transaction ID differs from the expected ID.
    WrongTransactionId {
        /// Transaction ID sent by the probe.
        expected: u32,
        /// Transaction ID found in the reply.
        actual: u32,
    },
    /// The reply client hardware address differs from the expected address.
    WrongClientHardwareAddress {
        /// Hardware address sent by the probe.
        expected: [u8; 6],
        /// Hardware address found in the reply.
        actual: [u8; 6],
    },
    /// An option header or value extends beyond the input.
    TruncatedOption {
        /// Option code, when its code byte was available.
        code: Option<u8>,
        /// Byte offset at which input ended.
        offset: usize,
    },
    /// More option instances were supplied than the parser permits.
    TooManyOptions {
        /// Maximum option-instance count accepted by the parser.
        maximum: usize,
    },
    /// The required DHCP end option is missing.
    MissingEndOption,
    /// A required option is missing.
    MissingOption {
        /// Required option code.
        code: u8,
    },
    /// An option has a length that is invalid for its type.
    InvalidOptionLength {
        /// Malformed option code.
        code: u8,
        /// Length found on the wire.
        actual: usize,
        /// Human-readable valid-length constraint.
        expected: &'static str,
    },
    /// Repeated scalar options disagree.
    ConflictingOption {
        /// Repeated option code.
        code: u8,
    },
    /// The DHCP message type is not Offer.
    NotOffer {
        /// DHCP message type found on the wire.
        actual: u8,
    },
    /// A decoded collection exceeds its defensive item limit.
    TooManyValues {
        /// Option code containing the oversized collection.
        code: u8,
        /// Maximum item count accepted by the parser.
        maximum: usize,
    },
    /// The domain-name option is not valid UTF-8.
    InvalidDomainName,
    /// RFC 3397 domain-search data is malformed.
    InvalidDomainSearch {
        /// Offset within the concatenated option value.
        offset: usize,
        /// Stable explanation of the malformed encoding.
        reason: &'static str,
    },
    /// RFC 3442 classless-route data is malformed.
    InvalidClasslessRoute {
        /// Offset within the concatenated option value.
        offset: usize,
        /// Stable explanation of the malformed encoding.
        reason: &'static str,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated { actual, minimum } => {
                write!(
                    formatter,
                    "DHCP packet is {actual} bytes; at least {minimum} are required"
                )
            }
            Self::PacketTooLarge { actual, maximum } => {
                write!(
                    formatter,
                    "DHCP packet is {actual} bytes; limit is {maximum}"
                )
            }
            Self::NotBootReply { actual } => {
                write!(formatter, "BOOTP operation {actual} is not BOOTREPLY")
            }
            Self::UnsupportedHardwareType { actual } => {
                write!(formatter, "BOOTP hardware type {actual} is not Ethernet")
            }
            Self::InvalidHardwareAddressLength { actual } => {
                write!(formatter, "BOOTP hardware address length {actual} is not 6")
            }
            Self::MissingMagicCookie => {
                formatter.write_str("DHCP magic cookie is missing or invalid")
            }
            Self::WrongTransactionId { expected, actual } => {
                write!(
                    formatter,
                    "DHCP transaction ID {actual:#010x} does not match {expected:#010x}"
                )
            }
            Self::WrongClientHardwareAddress { expected, actual } => {
                write!(
                    formatter,
                    "DHCP client hardware address {actual:02x?} does not match {expected:02x?}"
                )
            }
            Self::TruncatedOption { code, offset } => {
                write!(
                    formatter,
                    "DHCP option {code:?} is truncated at offset {offset}"
                )
            }
            Self::TooManyOptions { maximum } => {
                write!(formatter, "DHCP packet has more than {maximum} options")
            }
            Self::MissingEndOption => formatter.write_str("DHCP end option is missing"),
            Self::MissingOption { code } => {
                write!(formatter, "required DHCP option {code} is missing")
            }
            Self::InvalidOptionLength {
                code,
                actual,
                expected,
            } => {
                write!(
                    formatter,
                    "DHCP option {code} has length {actual}; expected {expected}"
                )
            }
            Self::ConflictingOption { code } => write!(
                formatter,
                "repeated DHCP option {code} has conflicting values"
            ),
            Self::NotOffer { actual } => {
                write!(formatter, "DHCP message type {actual} is not Offer")
            }
            Self::TooManyValues { code, maximum } => {
                write!(
                    formatter,
                    "DHCP option {code} contains more than {maximum} values"
                )
            }
            Self::InvalidDomainName => formatter.write_str("DHCP domain name is not valid UTF-8"),
            Self::InvalidDomainSearch { offset, reason } => {
                write!(
                    formatter,
                    "DHCP domain-search data is invalid at offset {offset}: {reason}"
                )
            }
            Self::InvalidClasslessRoute { offset, reason } => {
                write!(
                    formatter,
                    "DHCP classless-route data is invalid at offset {offset}: {reason}"
                )
            }
        }
    }
}

impl Error for ParseError {}

#[derive(Clone, Copy)]
struct OptionRef<'a> {
    code: u8,
    value: &'a [u8],
}

/// Parses and validates one matching DHCPv4 Offer UDP payload.
///
/// Unknown options and padding are ignored. Matching is performed before
/// option decoding so unrelated traffic can be discarded with a specific
/// error. Repeated list options are combined; repeated scalar options are
/// accepted only when their values are identical.
///
/// # Errors
///
/// Returns [`ParseError`] for truncated or malformed input, non-Offer DHCP
/// messages, or replies that do not match `expectation`.
pub fn parse_offer(
    packet: &[u8],
    expectation: OfferExpectation,
) -> Result<ParsedDhcpV4Offer, ParseError> {
    if packet.len() < OPTIONS_OFFSET {
        return Err(ParseError::Truncated {
            actual: packet.len(),
            minimum: OPTIONS_OFFSET,
        });
    }
    if packet.len() > MAX_DHCP_V4_MESSAGE_SIZE {
        return Err(ParseError::PacketTooLarge {
            actual: packet.len(),
            maximum: MAX_DHCP_V4_MESSAGE_SIZE,
        });
    }
    if packet[0] != 2 {
        return Err(ParseError::NotBootReply { actual: packet[0] });
    }
    if packet[1] != 1 {
        return Err(ParseError::UnsupportedHardwareType { actual: packet[1] });
    }
    if packet[2] != 6 {
        return Err(ParseError::InvalidHardwareAddressLength { actual: packet[2] });
    }
    if packet[BOOTP_FIXED_LENGTH..OPTIONS_OFFSET] != DHCP_MAGIC_COOKIE {
        return Err(ParseError::MissingMagicCookie);
    }

    let transaction_id = read_u32(&packet[4..8]);
    if transaction_id != expectation.transaction_id {
        return Err(ParseError::WrongTransactionId {
            expected: expectation.transaction_id,
            actual: transaction_id,
        });
    }
    let mut client_hardware_address = [0_u8; 6];
    client_hardware_address.copy_from_slice(&packet[28..34]);
    if client_hardware_address != expectation.client_hardware_address {
        return Err(ParseError::WrongClientHardwareAddress {
            expected: expectation.client_hardware_address,
            actual: client_hardware_address,
        });
    }

    let options = scan_options(packet)?;
    let message_type =
        parse_single_u8(&options, OPTION_MESSAGE_TYPE)?.ok_or(ParseError::MissingOption {
            code: OPTION_MESSAGE_TYPE,
        })?;
    if message_type != DHCPOFFER {
        return Err(ParseError::NotOffer {
            actual: message_type,
        });
    }

    let domain_name = single_value(&options, OPTION_DOMAIN_NAME)?
        .map(|value| {
            str::from_utf8(value)
                .map(str::to_owned)
                .map_err(|_| ParseError::InvalidDomainName)
        })
        .transpose()?;
    let domain_search_bytes = concatenated_value(&options, OPTION_DOMAIN_SEARCH)?;
    let route_bytes = concatenated_value(&options, OPTION_CLASSLESS_STATIC_ROUTE)?;

    Ok(ParsedDhcpV4Offer {
        transaction_id,
        client_hardware_address,
        offered_address: ipv4(&packet[16..20]),
        server_identifier: parse_single_ipv4(&options, OPTION_SERVER_IDENTIFIER)?,
        subnet_mask: parse_single_ipv4(&options, OPTION_SUBNET_MASK)?,
        routers: parse_ipv4_list(&options, OPTION_ROUTER)?,
        dns_servers: parse_ipv4_list(&options, OPTION_DNS_SERVER)?,
        domain_name,
        domain_search: domain_search_bytes
            .as_deref()
            .map(parse_domain_search)
            .transpose()?
            .unwrap_or_default(),
        lease_time_seconds: parse_single_u32(&options, OPTION_LEASE_TIME)?,
        renewal_time_seconds: parse_single_u32(&options, OPTION_RENEWAL_TIME)?,
        rebinding_time_seconds: parse_single_u32(&options, OPTION_REBINDING_TIME)?,
        interface_mtu: parse_single_u16(&options, OPTION_INTERFACE_MTU)?,
        classless_static_routes: route_bytes
            .as_deref()
            .map(parse_classless_routes)
            .transpose()?
            .unwrap_or_default(),
    })
}

fn scan_options(packet: &[u8]) -> Result<Vec<OptionRef<'_>>, ParseError> {
    let mut options = Vec::new();
    let mut offset = OPTIONS_OFFSET;
    let mut found_end = false;
    while offset < packet.len() {
        let code = packet[offset];
        offset += 1;
        match code {
            OPTION_PAD => {}
            OPTION_END => {
                found_end = true;
                break;
            }
            _ => {
                let length = packet
                    .get(offset)
                    .copied()
                    .ok_or(ParseError::TruncatedOption {
                        code: Some(code),
                        offset,
                    })? as usize;
                offset += 1;
                let end = offset
                    .checked_add(length)
                    .ok_or(ParseError::TruncatedOption {
                        code: Some(code),
                        offset,
                    })?;
                let value = packet.get(offset..end).ok_or(ParseError::TruncatedOption {
                    code: Some(code),
                    offset,
                })?;
                if options.len() >= MAX_OPTION_INSTANCES {
                    return Err(ParseError::TooManyOptions {
                        maximum: MAX_OPTION_INSTANCES,
                    });
                }
                options.push(OptionRef { code, value });
                offset = end;
            }
        }
    }
    if !found_end {
        return Err(ParseError::MissingEndOption);
    }
    Ok(options)
}

fn values<'a>(options: &'a [OptionRef<'a>], code: u8) -> impl Iterator<Item = &'a [u8]> + 'a {
    options
        .iter()
        .filter(move |option| option.code == code)
        .map(|option| option.value)
}

fn single_value<'a>(
    options: &'a [OptionRef<'a>],
    code: u8,
) -> Result<Option<&'a [u8]>, ParseError> {
    let mut matching = values(options, code);
    let Some(first) = matching.next() else {
        return Ok(None);
    };
    if matching.any(|value| value != first) {
        return Err(ParseError::ConflictingOption { code });
    }
    Ok(Some(first))
}

fn concatenated_value(options: &[OptionRef<'_>], code: u8) -> Result<Option<Vec<u8>>, ParseError> {
    let mut combined = Vec::new();
    let mut present = false;
    for value in values(options, code) {
        present = true;
        if combined.len().saturating_add(value.len()) > MAX_DHCP_V4_MESSAGE_SIZE {
            return Err(ParseError::InvalidOptionLength {
                code,
                actual: combined.len().saturating_add(value.len()),
                expected: "at most 4096 bytes in total",
            });
        }
        combined.extend_from_slice(value);
    }
    Ok(present.then_some(combined))
}

fn parse_single_u8(options: &[OptionRef<'_>], code: u8) -> Result<Option<u8>, ParseError> {
    single_value(options, code)?
        .map(|value| {
            if value.len() != 1 {
                return Err(ParseError::InvalidOptionLength {
                    code,
                    actual: value.len(),
                    expected: "1 byte",
                });
            }
            Ok(value[0])
        })
        .transpose()
}

fn parse_single_u16(options: &[OptionRef<'_>], code: u8) -> Result<Option<u16>, ParseError> {
    single_value(options, code)?
        .map(|value| {
            if value.len() != 2 {
                return Err(ParseError::InvalidOptionLength {
                    code,
                    actual: value.len(),
                    expected: "2 bytes",
                });
            }
            Ok(u16::from_be_bytes([value[0], value[1]]))
        })
        .transpose()
}

fn parse_single_u32(options: &[OptionRef<'_>], code: u8) -> Result<Option<u32>, ParseError> {
    single_value(options, code)?
        .map(|value| {
            if value.len() != 4 {
                return Err(ParseError::InvalidOptionLength {
                    code,
                    actual: value.len(),
                    expected: "4 bytes",
                });
            }
            Ok(read_u32(value))
        })
        .transpose()
}

fn parse_single_ipv4(options: &[OptionRef<'_>], code: u8) -> Result<Option<Ipv4Addr>, ParseError> {
    single_value(options, code)?
        .map(|value| {
            if value.len() != 4 {
                return Err(ParseError::InvalidOptionLength {
                    code,
                    actual: value.len(),
                    expected: "4 bytes",
                });
            }
            Ok(ipv4(value))
        })
        .transpose()
}

fn parse_ipv4_list(options: &[OptionRef<'_>], code: u8) -> Result<Vec<Ipv4Addr>, ParseError> {
    let mut addresses = Vec::new();
    for value in values(options, code) {
        if value.is_empty() || value.len() % 4 != 0 {
            return Err(ParseError::InvalidOptionLength {
                code,
                actual: value.len(),
                expected: "a non-empty multiple of 4 bytes",
            });
        }
        for bytes in value.chunks_exact(4) {
            if addresses.len() >= MAX_LIST_ITEMS {
                return Err(ParseError::TooManyValues {
                    code,
                    maximum: MAX_LIST_ITEMS,
                });
            }
            addresses.push(ipv4(bytes));
        }
    }
    Ok(addresses)
}

fn parse_domain_search(data: &[u8]) -> Result<Vec<String>, ParseError> {
    let mut names = Vec::new();
    let mut offset = 0;
    while offset < data.len() {
        if names.len() >= MAX_DOMAIN_NAMES {
            return Err(ParseError::TooManyValues {
                code: OPTION_DOMAIN_SEARCH,
                maximum: MAX_DOMAIN_NAMES,
            });
        }
        let (name, consumed) = parse_dns_name(data, offset)?;
        if consumed == 0 {
            return Err(ParseError::InvalidDomainSearch {
                offset,
                reason: "name consumed no input",
            });
        }
        names.push(name);
        offset += consumed;
    }
    Ok(names)
}

fn parse_dns_name(data: &[u8], start: usize) -> Result<(String, usize), ParseError> {
    let mut labels = Vec::new();
    let mut cursor = start;
    let mut consumed = 0;
    let mut jumped = false;
    let mut visited = vec![false; data.len()];
    loop {
        let length = *data.get(cursor).ok_or(ParseError::InvalidDomainSearch {
            offset: cursor,
            reason: "truncated label",
        })?;
        if !jumped {
            consumed += 1;
        }
        if length == 0 {
            break;
        }
        if length & 0xc0 == 0xc0 {
            let low = *data
                .get(cursor + 1)
                .ok_or(ParseError::InvalidDomainSearch {
                    offset: cursor,
                    reason: "truncated compression pointer",
                })?;
            if !jumped {
                consumed += 1;
            }
            let pointer = (usize::from(length & 0x3f) << 8) | usize::from(low);
            if pointer >= data.len() {
                return Err(ParseError::InvalidDomainSearch {
                    offset: cursor,
                    reason: "compression pointer is out of range",
                });
            }
            if visited[pointer] {
                return Err(ParseError::InvalidDomainSearch {
                    offset: cursor,
                    reason: "compression pointer loop",
                });
            }
            visited[pointer] = true;
            cursor = pointer;
            jumped = true;
            continue;
        }
        if length & 0xc0 != 0 || length > 63 {
            return Err(ParseError::InvalidDomainSearch {
                offset: cursor,
                reason: "invalid label length",
            });
        }
        let label_start = cursor + 1;
        let label_end = label_start + usize::from(length);
        let label = data
            .get(label_start..label_end)
            .ok_or(ParseError::InvalidDomainSearch {
                offset: cursor,
                reason: "truncated label contents",
            })?;
        let label = str::from_utf8(label).map_err(|_| ParseError::InvalidDomainSearch {
            offset: cursor,
            reason: "label is not valid UTF-8",
        })?;
        labels.push(label);
        if !jumped {
            consumed += usize::from(length);
        }
        cursor = label_end;
        if labels.iter().map(|label| label.len()).sum::<usize>() + labels.len().saturating_sub(1)
            > 253
        {
            return Err(ParseError::InvalidDomainSearch {
                offset: start,
                reason: "decoded name is too long",
            });
        }
    }
    Ok((labels.join("."), consumed))
}

fn parse_classless_routes(data: &[u8]) -> Result<Vec<ClasslessStaticRoute>, ParseError> {
    let mut routes = Vec::new();
    let mut offset = 0;
    while offset < data.len() {
        if routes.len() >= MAX_LIST_ITEMS {
            return Err(ParseError::TooManyValues {
                code: OPTION_CLASSLESS_STATIC_ROUTE,
                maximum: MAX_LIST_ITEMS,
            });
        }
        let prefix_length = data[offset];
        if prefix_length > 32 {
            return Err(ParseError::InvalidClasslessRoute {
                offset,
                reason: "prefix length exceeds 32",
            });
        }
        offset += 1;
        let destination_length = usize::from(prefix_length).div_ceil(8);
        let route_length = destination_length + 4;
        let route_end =
            offset
                .checked_add(route_length)
                .ok_or(ParseError::InvalidClasslessRoute {
                    offset,
                    reason: "route length overflow",
                })?;
        let encoded = data
            .get(offset..route_end)
            .ok_or(ParseError::InvalidClasslessRoute {
                offset,
                reason: "truncated destination or router",
            })?;
        let mut destination = [0_u8; 4];
        destination[..destination_length].copy_from_slice(&encoded[..destination_length]);
        if prefix_length % 8 != 0 && destination_length != 0 {
            let mask = u8::MAX << (8 - (prefix_length % 8));
            destination[destination_length - 1] &= mask;
        }
        let router_start = destination_length;
        routes.push(ClasslessStaticRoute {
            destination: Ipv4Addr::from(destination),
            prefix_length,
            router: ipv4(&encoded[router_start..router_start + 4]),
        });
        offset = route_end;
    }
    Ok(routes)
}

fn read_u32(bytes: &[u8]) -> u32 {
    u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn ipv4(bytes: &[u8]) -> Ipv4Addr {
    Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_contains_required_shape_and_no_hostname() {
        let packet = DhcpDiscover::new(0x1234_5678, [0, 1, 2, 3, 4, 5]).encode();
        assert_eq!(packet.len(), MINIMUM_MESSAGE_LENGTH);
        assert_eq!(&packet[4..8], &[0x12, 0x34, 0x56, 0x78]);
        assert_eq!(&packet[10..12], &[0x80, 0]);
        assert_eq!(&packet[28..34], &[0, 1, 2, 3, 4, 5]);
        assert_eq!(&packet[236..240], &DHCP_MAGIC_COOKIE);
        assert!(
            packet
                .windows(3)
                .any(|value| value == [OPTION_MESSAGE_TYPE, 1, DHCPDISCOVER])
        );
        assert!(!packet.windows(2).any(|value| value[0] == OPTION_HOST_NAME));
        assert!(packet.contains(&OPTION_END));
    }

    #[test]
    fn discover_can_clear_broadcast_flag() {
        let packet = DhcpDiscover::new(1, [1; 6]).with_broadcast(false).encode();
        assert_eq!(&packet[10..12], &[0, 0]);
    }

    #[test]
    fn arbitrary_input_never_panics() {
        let expected = OfferExpectation {
            transaction_id: 0,
            client_hardware_address: [0; 6],
        };
        for length in 0..=512 {
            let input = vec![0xa5; length];
            let _result = parse_offer(&input, expected);
        }
    }

    #[test]
    fn rejects_payload_larger_than_defensive_limit() {
        let packet = vec![0_u8; MAX_DHCP_V4_MESSAGE_SIZE + 1];
        assert!(matches!(
            parse_offer(
                &packet,
                OfferExpectation {
                    transaction_id: 0,
                    client_hardware_address: [0; 6],
                }
            ),
            Err(ParseError::PacketTooLarge { .. })
        ));
    }
}
