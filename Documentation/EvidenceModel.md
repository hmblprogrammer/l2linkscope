# Evidence Model

Future L2LinkScope findings must distinguish how strongly a claim is supported.
The evidence model is conceptual in this scaffold; no Rust evidence types are
implemented yet.

## Evidence Classes

Directly observed evidence comes from traffic or platform state that
L2LinkScope actually observed. For example, seeing a source MAC address in a
captured Ethernet frame is direct evidence that the frame was present.

Advertised-by-peer evidence comes from configuration or facts a peer claims.
For example, DHCP-provided subnet information is advertised by a peer. It is not
proof that the local host accepted that configuration or that the peer is
authoritative.

Derived evidence is computed from directly observed or advertised inputs. For
example, a network prefix might be derived from an advertised address and subnet
mask, while preserving the fact that the subnet mask itself was advertised.

Speculative evidence is a cautious hypothesis that may help a user investigate
but must not be presented as fact.

## Examples

ARP addresses do not prove a CIDR. They show that a protocol participant used
an address in observed traffic.

An observed VLAN tag does not prove that a VLAN is available for configuration.
It only proves that tagged traffic was observed.

Missing tagged traffic does not prove that no VLAN exists. It only means no
matching traffic was observed in the collection window.

DHCP-provided routers, DNS servers, lease duration, and subnet masks are
advertised by a peer. L2LinkScope must report them as offers or claims, not as
local configuration.

## Product Boundary

L2LinkScope observes and probes. It does not configure. Evidence classes should
reinforce that boundary by avoiding language that implies configuration was
accepted, trusted, or applied.
