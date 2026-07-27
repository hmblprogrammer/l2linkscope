#![doc = "\
Portable domain foundation for L2LinkScope.

This crate is reserved for future observation and evidence models shared across
protocol, Linux acquisition, and presentation layers. It intentionally contains
no network-discovery implementation in the repository scaffold.
"]
#![forbid(unsafe_code)]

/// Returns a concise description of this crate's intended responsibility.
#[must_use]
pub const fn crate_purpose() -> &'static str {
    "portable L2LinkScope domain concepts"
}

#[cfg(test)]
mod tests {
    use super::crate_purpose;

    #[test]
    fn purpose_is_available() {
        assert_eq!(crate_purpose(), "portable L2LinkScope domain concepts");
    }
}
