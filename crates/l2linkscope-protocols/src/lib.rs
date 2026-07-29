//! Portable protocol encoding and parsing for L2LinkScope.
//!
//! This crate owns network-protocol bytes, but never opens sockets or performs
//! packet I/O. The initial API builds DHCPv4 Discover messages and parses
//! matching DHCPv4 Offer messages. Callers remain responsible for generating a
//! cryptographically strong transaction ID and for enforcing a bounded probe.
//!
//! All parsers accept borrowed byte slices, keep no global state, bounds-check
//! every read, and return structured errors. This makes [`dhcpv4::parse_offer`]
//! suitable as a fuzzing entry point.
//!
//! ```
//! use l2linkscope_protocols::dhcpv4::DhcpDiscover;
//!
//! // Acquisition code must generate this ID with a secure random source.
//! let transaction_id = 0x1234_5678;
//! let client_mac = [0x02, 0, 0, 0, 0, 1];
//! let payload = DhcpDiscover::new(transaction_id, client_mac).encode();
//! assert_eq!(&payload[4..8], &transaction_id.to_be_bytes());
//! ```
//!
//! See the repository's [architecture documentation] and [evidence model] for
//! the boundary between hostile protocol input and normalized observations.
//!
//! [architecture documentation]: https://github.com/hmblprogrammer/l2linkscope/blob/main/Documentation/Architecture.md
//! [evidence model]: https://github.com/hmblprogrammer/l2linkscope/blob/main/Documentation/EvidenceModel.md
#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// DHCPv4 message encoding and parsing.
pub mod dhcpv4;

/// Returns a concise description of this crate's responsibility.
#[must_use]
pub const fn crate_purpose() -> &'static str {
    "L2LinkScope protocol encoding and parsing"
}
