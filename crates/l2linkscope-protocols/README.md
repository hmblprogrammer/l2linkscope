# l2linkscope-protocols

Protocol-byte ownership for L2LinkScope.

This crate contains portable protocol encoding and parsing. It must not
transmit packets, receive packets, open sockets, or depend on Linux-only
acquisition APIs.

The `dhcpv4` module builds DHCP Discover UDP payloads and parses matching DHCP
Offer payloads. It deliberately exposes no DHCP lease-acceptance state machine.
The default Discover requests common configuration without transmitting a host
name. Parsed configuration remains peer-advertised data and is not trusted or
applied by this crate.

```rust
use l2linkscope_protocols::dhcpv4::{DhcpDiscover, OfferExpectation, parse_offer};

let transaction_id = 0x1234_5678; // Generate securely in acquisition code.
let mac = [0x02, 0, 0, 0, 0, 1];
let discover_payload = DhcpDiscover::new(transaction_id, mac).encode();

// A transport can send `discover_payload`, then parse each received UDP
// payload. Parsing does not accept or apply an offered lease.
# let received_payload: &[u8] = &[];
let result = parse_offer(
    received_payload,
    OfferExpectation { transaction_id, client_hardware_address: mac },
);
```

`parse_offer` is a stateless, bounds-checked fuzzing entry point. Malformed and
unrelated packets return structured errors rather than panicking.

This crate forbids unsafe code.
