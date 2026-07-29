# Architecture

L2LinkScope 0.1.0 is a four-package Cargo workspace. Each package owns one
layer of the discovery pipeline:

```text
DHCP bytes
    | encode / parse
    v
l2linkscope-protocols
    | normalized protocol values
    v
l2linkscope-linux --------> l2linkscope-core
    | safe Linux inventory    domain models, evidence, errors
    | and bounded transport
    v
l2linkscope CLI
    human-readable or JSON presentation
```

## Package responsibilities

### `l2linkscope-core`

The portable domain layer defines interface identity and metadata, discovery
sessions, observations, evidence classes, DHCPv4 offer results, snapshots,
warnings, and structured errors. Its serializable models do not contain raw OS
handles or debug-formatted implementation details.

It forbids unsafe code and has no dependency on Linux APIs, raw sockets, CLI
libraries, or an asynchronous runtime.

### `l2linkscope-protocols`

The protocol layer owns DHCP bytes. It constructs a DHCPv4 Discover and parses
untrusted replies into validated protocol values. Parsing is deterministic,
bounds-checked, independent of packet acquisition, and directly usable by unit
tests or a future fuzz target. After validation,
`ParsedDhcpV4Offer::into_normalized()` performs the explicit conversion into
the portable `l2linkscope_core::DhcpV4Offer` domain model.

It never opens a socket, transmits a frame, or formats CLI output. It forbids
unsafe code.

### `l2linkscope-linux`

The platform layer inventories Linux interfaces and performs the explicitly
requested DHCP probe. It resolves exactly one interface, constructs the probe
through `l2linkscope-protocols`, transmits and receives through the selected
interface, matches replies, enforces the collection deadline, deduplicates
identical offers, and normalizes the result through `l2linkscope-core`.

Version 0.1.0 obtains configured addresses through the kernel `getifaddrs`
interface and reads link index, type, flags, operational state, carrier, MAC,
and MTU from `/sys/class/net`. This small synchronous implementation avoids an
asynchronous netlink runtime while preserving the same normalized public model;
a later netlink backend can replace it without changing CLI presentation.

Linux privilege and socket details remain behind its small safe public
interface. The current implementation forbids unsafe code. Any future direct
syscall use must stay confined to an internal module with documented invariants
and explicit review.

### `l2linkscope`

The executable parses arguments, invokes libraries, renders human-readable or
JSON output, sends diagnostics to standard error, and maps failures to process
exit codes. It contains no DHCP wire-format logic or Linux socket operations.

## Dependency direction

```text
l2linkscope-core

l2linkscope-protocols
    -> l2linkscope-core

l2linkscope-linux
    -> l2linkscope-core
    -> l2linkscope-protocols

l2linkscope
    -> l2linkscope-core
    -> l2linkscope-protocols
    -> l2linkscope-linux
```

Dependencies flow toward portable policy and protocol layers. Cycles are not
allowed. The CLI may depend on every library package, but no library depends on
the CLI.

## Key design decisions

### Parsing and acquisition are separate

Network bytes are hostile. Keeping parsing pure makes malformed-input behavior
unit-testable and fuzzable without Linux, privileges, timing, or live network
state. The tradeoff is an explicit conversion step between protocol values and
domain observations; that step is preferable to coupling parsers to sockets.

### Models and presentation are separate

Library consumers need structured results without terminal wording. The CLI
therefore owns tables, prose, JSON envelopes, and exit codes. The tradeoff is
that additions to a public result may require coordinated presentation work.

### The first active workflow stops at Offer

The DHCP transport implements only Discover transmission and Offer collection.
It cannot accept a lease because the workflow never constructs or transmits a
Request. This narrow state machine is easier to audit and test. Lease
negotiation belongs outside the product boundary, not behind an option.

### Socket privileges are isolated

Interface inventory should work without elevated privileges. Only the active
transport crosses the Linux socket privilege boundary. Binding UDP port 68 and
binding the socket to one device are kept in that layer so deployments can use
`CAP_NET_BIND_SERVICE` and `CAP_NET_RAW` rather than broad root operation.

## Interface identity

An interface name can be renamed and is not a stable identity. Version 0.1.0
uses the kernel interface index plus available hardware metadata as a runtime
identity and carries the current name as metadata. This is sufficient for a
bounded session but does not guarantee persistent identity across reboots,
network-namespace recreation, or hardware replacement.

## External consumers

The CLI and any future adapters are sibling consumers of the public crates.
L2LinkScope remains standalone and contains no JoshOS filesystem paths,
Services, Worlds, IPC conventions, build hooks, or release terminology. A
future NetworkDiscoveryService integration must be implemented as an external
adapter without changing these boundaries.
