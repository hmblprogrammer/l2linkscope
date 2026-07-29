# l2linkscope-core

Portable discovery-domain models for L2LinkScope.

The crate contains normalized interface, session, observation, evidence, DHCPv4
offer, snapshot, warning, and error concepts shared by protocol parsers, Linux
acquisition code, and presentation layers.

DHCP offers are classified as peer-advertised evidence. They never indicate that
the proposed configuration was trusted, accepted, or applied. `DiscoverySnapshot`
includes a JSON schema version; its JSON format remains experimental throughout
the `0.x` series.

This crate forbids unsafe code and must remain independent of Linux-only APIs,
raw sockets, command-line parsing, and asynchronous runtimes unless a future
architecture document explicitly justifies a change.

```rust
use l2linkscope_core::{EvidenceClass, JSON_SCHEMA_VERSION};

assert_eq!(JSON_SCHEMA_VERSION, "0.1");
assert_ne!(EvidenceClass::AdvertisedByPeer, EvidenceClass::DirectlyObserved);
```
