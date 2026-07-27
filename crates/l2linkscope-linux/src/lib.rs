#![doc = "\
Linux-specific acquisition boundary for L2LinkScope.

This crate is reserved for future Linux interface inventory, packet acquisition,
and transport code. It intentionally contains no netlink, raw-socket, packet
capture, or network-discovery implementation in the repository scaffold.
"]
#![deny(unsafe_code)]

/// Returns a concise description of this crate's intended responsibility.
#[must_use]
pub const fn crate_purpose() -> &'static str {
    "Linux-specific L2LinkScope acquisition boundary"
}

#[cfg(test)]
mod tests {
    use super::crate_purpose;

    #[test]
    fn purpose_is_available() {
        assert_eq!(
            crate_purpose(),
            "Linux-specific L2LinkScope acquisition boundary"
        );
    }
}
