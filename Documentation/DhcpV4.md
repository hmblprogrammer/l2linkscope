# DHCPv4 Discover/Offer Boundary

L2LinkScope 0.1.0 implements one active protocol workflow: send a DHCPv4
Discover on a user-selected interface and collect matching Offers for a bounded
period. It is discovery, not a DHCP client.

## State machine

```text
explicit CLI request
        |
        v
resolve and validate selected interface
        |
        v
generate random transaction ID
        |
        v
send DHCP Discover once
        |
        v
collect matching Offers until deadline
        |
        v
report and exit
```

There is no transition from Offer to Request. The workflow has no code path for
DHCP Request, Decline, Release, or Inform and never applies configuration.

## Discover contents

The encoder creates a BOOTP/DHCP payload with:

* a cryptographically strong transaction ID supplied by the transport;
* the selected interface's Ethernet hardware address;
* the broadcast flag;
* the DHCP magic cookie;
* DHCP message type Discover;
* a bounded parameter-request list; and
* the DHCP end option.

The parameter-request list asks for subnet mask, routers, DNS servers, domain
name, domain search, interface MTU, lease time, server identifier, renewal and
rebinding time, and classless static routes when supported by the parser. The
default message does not send a hostname and avoids unnecessary
client-identifying options.

## Reply validation

All input is hostile. The parser bounds-checks the fixed header and every
option, tolerates padding and unknown options, caps repeated collection values,
and returns structured errors for invalid lengths. An Offer is eligible only
when all of these match the active probe:

* the BOOTP reply structure and DHCP magic cookie are valid;
* DHCP option 53 identifies an Offer;
* the transaction ID matches; and
* the client hardware address matches.

ACKs, unrelated transaction IDs, unrelated client addresses, truncated
payloads, malformed options, and malformed classless routes are never reported
as valid offers. Duplicate options are handled according to the parser's
documented normalization rather than indexing beyond packet bounds.

## Collection and deduplication

The default probe transmits one Discover and waits for a conservative bounded
collection interval. User-supplied timeouts are range-checked; there is no
indefinite retry mode. Distinct Offers are retained, including competing server
advertisements. A retransmitted otherwise-identical Offer is deduplicated.

When more than one server responds, the snapshot retains every distinct offer
and carries a warning. Zero valid Offers is a completed observation window, not
proof that DHCP is absent.

## Configuration invariants

The workflow does not:

* bind an offered address to an interface;
* add or remove a route;
* write resolver configuration;
* change link flags, MTU, VLANs, or authentication state;
* stop, signal, or modify another DHCP client; or
* select a different interface if the requested one fails.

The privileged integration test snapshots link, address, and route state before
and after probing and records received DHCP message types. See
[Testing](Testing.md).

## Existing DHCP clients

Another DHCP client can already be using UDP ports 67/68 or can independently
transmit DHCP traffic. L2LinkScope binds its socket to the chosen interface and
matches both transaction ID and hardware address, so unrelated replies are
ignored. An obvious bind, permission, or interface error is returned instead of
silently probing elsewhere. L2LinkScope never asks users to disable their
network manager for interface inventory and never disables it itself.
