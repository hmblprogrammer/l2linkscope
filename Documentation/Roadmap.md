# Roadmap

The roadmap separates shipped release scope from possible later work. Future
items are not promises and must not appear as working CLI commands before they
are implemented and tested.

## 0.1.0: architecture proof

The initial release establishes:

* portable interface, session, observation, evidence, warning, and error models
* DHCPv4 Discover construction and Offer parsing
* Linux interface inventory and bounded packet transport
* human-readable and versioned JSON CLI output
* unprivileged fixture tests and an isolated privileged namespace test path
* non-publishing CI and release-readiness checks

The active DHCP state machine ends after collecting Offers. Configuration and
lease acceptance remain explicit non-goals.

## After 0.1.0

Candidate increments include passive ARP and VLAN observations, IPv6 Neighbor
Discovery and Router Advertisements, LLDP, DHCPv6, and richer inference. Each
increment should begin with evidence semantics and parser fixtures, then add
acquisition and presentation separately.

Other possible later protocols include mDNS/DNS-SD, SSDP, CDP, EAPOL, STP, and
LACP. Wi-Fi discovery, topology mapping, and a background mode require separate
design review.

## Public API and publication

The 0.x JSON schema and Rust APIs are experimental. Before crates.io
publication, maintainers must review public APIs, confirm package-name
availability, complete package metadata, add compatible versions to path
dependencies, declare publication intent per package, and configure Trusted
Publishing or another short-lived credential flow. GitHub source and binary
releases may precede crates.io publication.

## External consumers

Downstream integrations—including a possible JoshOS
`NetworkDiscoveryService` adapter—belong outside this repository. They consume
public crates without introducing downstream filesystem, IPC, service, or
build-pipeline assumptions into L2LinkScope.
