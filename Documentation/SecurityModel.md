# Security and Privilege Model

L2LinkScope processes unauthenticated data from the local network. Every packet
and every peer advertisement is untrusted, even when it appears internally
consistent.

## Trust boundaries

The DHCP parser treats lengths, option codes, counts, addresses, strings, and
route data as hostile. Reads are bounds-checked, collection sizes are capped,
unknown options are tolerated, and malformed input returns a structured error
instead of panicking. Parser entry points have no global state, socket access,
or CLI dependency, which allows deterministic tests and fuzzing.

DHCP configuration is a claim by a peer. L2LinkScope does not authenticate the
server and must not describe an Offer as trusted or authoritative.

The acquisition layer records the UDP source address and port separately from
the DHCP Server Identifier option. The former is directly observed transport
metadata and the latter is a peer advertisement; neither proves authorization.
Packets from a non-server source port or without enough matching BOOTP identity
are ignored as unrelated to the active probe. Only traffic matching UDP/67, the
transaction ID, and the client hardware address can contribute a malformed
matching-response result.

## Privileges

`l2linkscope interfaces` is unprivileged. The active probe binds Linux UDP
sockets to privileged DHCP ports and to the selected interface. It requires
root or narrow capabilities: `CAP_NET_RAW` for binding to the selected device
and `CAP_NET_BIND_SERVICE` for binding UDP port 68.

For an initial root invocation:

```bash
sudo l2linkscope probe dhcp4 enp3s0
```

For capability-based deployment, first inspect the built binary and grant only
the capabilities required on the target distribution:

```bash
sudo setcap cap_net_raw,cap_net_bind_service=ep /usr/local/bin/l2linkscope
getcap /usr/local/bin/l2linkscope
```

File capabilities attach to a particular binary and must be reassessed after
replacement. Do not grant `CAP_NET_ADMIN`; the probe neither needs nor uses it
to configure the network.

The current single-process implementation opens and binds the interface-scoped
UDP/68 socket before sending the Discover. It does not currently drop root
identity or effective capabilities after the socket is created. Packet parsing,
model construction, and output serialization therefore run with the process's
remaining invocation privileges. Safe Rust and a narrow API reduce risk but do
not constitute privilege separation.

The intended future reduction boundary is immediately after the selected socket
is created and bound. A later design may drop UID/GID and capabilities in the
same process or pass the prepared descriptor from a small privileged helper to
an unprivileged parser. That change requires separate architecture and
platform-specific review and is intentionally not part of 0.1.0 hardening.

## Safe Linux boundary

Portable crates forbid unsafe code. Linux-only privileged operations live in
`l2linkscope-linux` behind a small safe interface. Any direct syscall or unsafe
block must document pointer validity, buffer ownership, lifetime, and kernel
return-value invariants and receive focused human review.

## Bounded active behavior

Active traffic occurs only after `probe dhcp4` is explicitly requested. The
probe uses a bounded timeout and bounded transmission count. It does not enter
promiscuous mode, scan a subnet, authenticate, join a VLAN, or configure an
interface.

## Existing clients and local policy

NetworkManager, systemd-networkd, dhclient, or another process may already be
using DHCP. L2LinkScope never stops or modifies those processes. Socket
conflicts and permission failures are reported; the program never silently
switches interfaces. Operators should understand that an explicit Discover can
still elicit traffic from DHCP servers even though no lease is accepted.

## Vulnerability reporting

Do not place captures containing customer or confidential network information
in public issues. Follow [SECURITY.md](../SECURITY.md) for coordinated reporting.
