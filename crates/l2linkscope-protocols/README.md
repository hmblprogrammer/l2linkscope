# l2linkscope-protocols

Protocol-byte ownership for L2LinkScope.

This crate will eventually contain protocol encoding, parsing, fixtures, and
malformed-input handling. It must not transmit packets, receive packets, open
sockets, or depend on Linux-only acquisition APIs.

Current status: placeholder only. No packet formats or parsers are implemented
yet.

This crate forbids unsafe code.
