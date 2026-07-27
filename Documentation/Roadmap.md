# Roadmap

This roadmap describes phases for future work. It is not a detailed issue list
and does not imply that any listed capability is currently implemented.

## Phase 0: Repository Scaffold

Objective: establish the public project foundation.

Likely deliverables: Cargo workspace, placeholder crates, documentation, local
checks, CI scaffolding, release-readiness checks, and repository hygiene files.

Exit criteria: the workspace compiles, docs build, checks pass, and no discovery
functionality exists.

Exclusions: protocol parsing, packet acquisition, active probing, and public
crate publication.

## Phase 1: Core Evidence And Observation Models

Objective: define portable concepts for interfaces, sessions, observations,
evidence classes, warnings, and errors.

Likely deliverables: documented Rust types, serialization planning, unit tests,
and examples that do not require root.

Exit criteria: future protocol and Linux crates can normalize findings through
stable core concepts.

Exclusions: protocol-specific packet parsing and Linux socket code.

## Phase 2: Protocol Fixtures And Passive ARP/VLAN Parsing

Objective: introduce fixture-driven protocol parsing for early passive
observations.

Likely deliverables: Ethernet, VLAN metadata, and ARP parsing fixtures;
malformed-input tests; and parser error behavior.

Exit criteria: parsers handle valid, unknown, truncated, and malformed inputs
without panics.

Exclusions: live packet acquisition and active probes.

## Phase 3: Linux Interface Inventory And Packet Acquisition

Objective: enumerate Linux interfaces and acquire packets safely.

Likely deliverables: interface metadata collection, acquisition boundaries,
privilege diagnostics, and unprivileged tests where practical.

Exit criteria: Linux-specific code can provide normalized observations without
configuring interfaces.

Exclusions: DHCP state transitions and broad protocol inference.

## Phase 4: IPv6 Neighbor Discovery And Router Advertisements

Objective: parse and normalize IPv6 local-network discovery signals.

Likely deliverables: ICMPv6 Neighbor Discovery and Router Advertisement parsing,
fixtures, and evidence classification.

Exit criteria: IPv6 observations are represented without overstating what they
prove.

Exclusions: changing IPv6 addresses, routes, or router preferences.

## Phase 5: LLDP

Objective: observe LLDP advertisements from adjacent devices.

Likely deliverables: LLDP parser coverage, fixture tests, and normalized peer
metadata.

Exit criteria: LLDP findings distinguish advertised peer metadata from local
configuration.

Exclusions: switch configuration and topology-control features.

## Phase 6: Bounded DHCP Offer Probes

Objective: send bounded DHCP probes that observe offers without accepting
leases.

Likely deliverables: DHCPv4 and later DHCPv6 construction, offer parsing,
timeouts, deduplication, privilege diagnostics, and integration tests.

Exit criteria: probes report zero or more offers and prove that no address,
route, DNS setting, VLAN, or interface flag changed.

Exclusions: DHCP Request, Decline, Release, lease acceptance, and automatic
configuration.

## Phase 7: Inference And Presentation

Objective: present observations and cautious derived findings clearly.

Likely deliverables: human-readable CLI output, JSON output format, evidence
labels, warnings, and exit-code documentation.

Exit criteria: users can distinguish observed, advertised, derived, and
speculative information.

Exclusions: unsupported commands and claims of configuration authority.

## Phase 8: Privileged Integration Tests And Hardening

Objective: validate privileged Linux behavior in isolated environments.

Likely deliverables: network-namespace topologies, fixture services, packet
recording, state-before/state-after checks, and privileged CI separation.

Exit criteria: privileged tests are reproducible and do not touch external
networks or runner host configuration.

Exclusions: hardware-specific assumptions in ordinary CI.

## Phase 9: Public Crate Stabilization And Publication

Objective: prepare crates for public consumption.

Likely deliverables: API review, package metadata, versioned internal
dependencies, documentation review, license review, and release checks.

Exit criteria: maintainers can decide whether and when to publish crates.

Exclusions: automatic publication from ordinary CI.

## Phase 10: External Consumers, Including JoshOS

Objective: support downstream consumers through public crates.

Likely deliverables: documented library usage, compatibility guidance, and
consumer examples where appropriate.

Exit criteria: external consumers can integrate without coupling L2LinkScope to
their internal architecture.

Exclusions: JoshOS-specific code, filesystem paths, IPC conventions, services,
Worlds, build pipelines, initramfs, or ISO construction in this repository.
