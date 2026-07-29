# L2LinkScope

L2LinkScope is a standalone Rust project for discovering and describing the
local Layer 2 and adjacent network environment on Linux.

Version 0.1.0 establishes four reusable packages and one deliberately narrow
end-to-end feature: it can send a DHCPv4 Discover on an explicitly selected
interface, collect DHCP Offers for a bounded period, and report what peers
advertised. It never advances the DHCP state machine or applies an offer.

> **L2LinkScope observes and probes. It does not configure.**

L2LinkScope is not part of JoshOS. The crates do not depend on JoshOS paths,
services, IPC conventions, build pipelines, or terminology. A future JoshOS
adapter can consume the same public crates as a separate downstream project.

## Capabilities in 0.1.0

* inventory Linux interfaces without elevated privileges
* encode a DHCPv4 Discover without a hostname or other unnecessary client
  identifiers
* collect, validate, match, and deduplicate zero or more DHCPv4 Offers
* distinguish peer-advertised configuration from observations and derivations
* render human-readable output or versioned JSON
* stop after the bounded collection window without sending DHCP Request,
  Decline, Release, or Inform

Passive interface inventory is the default. The DHCP probe is active and runs
only when the user explicitly requests it.

## Build

The minimum supported Rust version is 1.85.0, as pinned by
`rust-toolchain.toml`. On a Linux host:

```bash
git clone https://github.com/hmblprogrammer/l2linkscope.git
cd l2linkscope
cargo build --release --locked -p l2linkscope
./target/release/l2linkscope --version
```

Run the full unprivileged validation suite with:

```bash
scripts/check.sh
```

That helper runs formatting, compilation, Clippy, unit tests, documentation,
and a release build. Parser and model tests do not need root.

## Usage

List interfaces (no special privileges required):

```bash
l2linkscope interfaces
l2linkscope interfaces --json
```

Probe one explicitly selected Ethernet-like interface:

```bash
sudo l2linkscope probe dhcp4 enp3s0
sudo l2linkscope probe dhcp4 enp3s0 --timeout 5s
sudo l2linkscope probe dhcp4 enp3s0 --json
```

Example human-readable interface output:

```text
enp3s0
  Index:       2
  Link:        connected
  MAC:         00:11:22:33:44:55
  MTU:         1500
  Addresses:   192.0.2.25/24
  DHCP probe:  supported
```

Example probe output:

```text
DHCPv4 offers observed on enp3s0

Offer 1
  Server:            192.0.2.1
  Proposed address:  192.0.2.117
  Derived network:   192.0.2.0/24
  Routers:           192.0.2.1
  DNS servers:       192.0.2.10, 192.0.2.11
  Lease duration:    8 hours

This configuration was advertised but not accepted or applied.
```

JSON is intended for programs and carries a top-level `schema_version`. For
example (metadata is abbreviated here; the command emits the complete model):

```json
{
  "schema_version": "0.1",
  "session": {
    "id": "11111111-1111-4111-8111-111111111111",
    "method": "dhcp_v4_probe"
  },
  "observations": [
    {
      "evidence_class": "advertised_by_peer",
      "kind": {
        "type": "dhcp_v4_offer",
        "details": {
          "offered_address": "192.0.2.117",
          "server_identifier": "192.0.2.1",
          "routers": ["192.0.2.1"],
          "dns_servers": ["192.0.2.10", "192.0.2.11"]
        }
      }
    }
  ],
  "warnings": []
}
```

The exact normalized fields are described in
[JSON Output](Documentation/JsonOutput.md). The JSON format is experimental
through the 0.x series; incompatible changes will change `schema_version`.

## Privileges and existing DHCP clients

Interface inventory is unprivileged. DHCP probing uses a UDP socket bound to
the selected interface and privileged DHCP client port 68. It currently
requires root or the narrow capabilities `CAP_NET_RAW` (for binding to the
device) and `CAP_NET_BIND_SERVICE` (for port 68):

```bash
sudo setcap cap_net_raw,cap_net_bind_service=ep ./target/release/l2linkscope
getcap ./target/release/l2linkscope
```

Capabilities attach to that exact binary and are normally lost when it is
replaced. Review the binary and local policy before granting them. The program
does not require `CAP_NET_ADMIN` and does not change interface configuration.

NetworkManager, systemd-networkd, dhclient, or another DHCP client may already
be active. L2LinkScope never stops or reconfigures it. An explicit probe can
still cause a DHCP server to emit offers and can overlap with another client's
traffic; matching by transaction ID and client hardware address prevents that
unrelated traffic from becoming a reported result. Socket or interface errors
are reported against the selected interface rather than falling back to a
different interface.

See [Security and Privileges](Documentation/SecurityModel.md) for the trust and
privilege boundaries.

## Safety guarantees and limitations

For the DHCPv4 Discover workflow, L2LinkScope:

* transmits a bounded number of Discover messages and listens for a bounded
  time
* never sends DHCP Request, Decline, Release, or Inform
* never accepts a lease or changes addresses, routes, DNS, VLAN membership,
  authentication, administrative state, or interface flags
* treats every packet as hostile and rejects malformed lengths without panic
* labels offered configuration `advertised_by_peer`; it is not trusted local
  state

The runtime interface identity combines the kernel interface index with
available hardware metadata. It is stable for a discovery session, but 0.1.0
does not promise persistent identity across reboots or hardware replacement.

Version 0.1.0 targets Linux Ethernet-like interfaces. It does not implement
passive packet monitoring, DHCPv6, IPv6 Router Advertisements, ARP observation,
VLAN discovery, LLDP, Wi-Fi discovery, subnet scanning, promiscuous capture,
background service operation, or automatic network configuration.

## Workspace

```text
crates/l2linkscope-core       portable models and evidence
crates/l2linkscope-protocols  protocol encoding and parsing; no I/O
crates/l2linkscope-linux      Linux inventory and privileged transport
crates/l2linkscope-cli        CLI parsing, presentation, and exit codes
```

The dependency direction and design rationale are documented in
[Architecture](Documentation/Architecture.md).

## Project documents

* [Architecture](Documentation/Architecture.md)
* [Evidence Model](Documentation/EvidenceModel.md)
* [DHCPv4](Documentation/DhcpV4.md)
* [JSON Output](Documentation/JsonOutput.md)
* [Security and Privileges](Documentation/SecurityModel.md)
* [Testing](Documentation/Testing.md)
* [Exit Codes](Documentation/ExitCodes.md)
* [Roadmap](Documentation/Roadmap.md)
* [Release Process](Documentation/Release.md)

Contributors and automated coding agents must follow
[AGENTS.md](AGENTS.md). Vulnerability reports should follow
[SECURITY.md](SECURITY.md).

## License

L2LinkScope is licensed under the [MIT License](LICENSE).
