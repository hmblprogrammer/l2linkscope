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
use l2linkscope_protocols::dhcpv4::{
    DhcpDiscover, OfferExpectation, ReplyAssociation, associate_reply, parse_offer,
};

let transaction_id = 0x1234_5678; // Generate securely in acquisition code.
let mac = [0x02, 0, 0, 0, 0, 1];
let discover_payload = DhcpDiscover::new(transaction_id, mac).encode();

// A transport can send `discover_payload`, associate each UDP/67 payload with
// this probe, and fully parse only matching payloads. Parsing does not accept
// or apply an offered lease.
# let received_payload: &[u8] = &[];
let association = associate_reply(
    received_payload,
    OfferExpectation { transaction_id, client_hardware_address: mac },
);
if association == ReplyAssociation::Matching {
    let _result = parse_offer(
        received_payload,
        OfferExpectation { transaction_id, client_hardware_address: mac },
    );
}
```

`associate_reply` and `parse_offer` are stateless, bounds-checked functions that
accept arbitrary byte slices. Malformed and unrelated packets return structured
results rather than panicking. They are suitable entry points for a future
coverage-guided fuzz target, but ordinary malformed-input tests are not claimed
as fuzz coverage.

This crate forbids unsafe code.
