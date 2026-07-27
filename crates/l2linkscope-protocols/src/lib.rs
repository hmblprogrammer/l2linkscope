#![doc = "\
Protocol-byte ownership for L2LinkScope.

This crate is reserved for future protocol encoders, parsers, and fixtures. It
intentionally contains no packet structures, protocol parsing, or packet I/O in
the repository scaffold.
"]
#![forbid(unsafe_code)]

/// Returns a concise description of this crate's intended responsibility.
#[must_use]
pub const fn crate_purpose() -> &'static str {
    "L2LinkScope protocol encoding and parsing"
}

#[cfg(test)]
mod tests {
    use super::crate_purpose;

    #[test]
    fn purpose_is_available() {
        assert_eq!(crate_purpose(), "L2LinkScope protocol encoding and parsing");
    }
}
