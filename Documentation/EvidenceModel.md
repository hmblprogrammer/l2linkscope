# Evidence Model

L2LinkScope records both a value and how that value became known. Confidence or
convenience must never promote an inference into an observed fact.

## Evidence classes

`directly_observed`
: A fact taken directly from a received frame or local kernel state. Examples
  include the selected interface index, its current MTU, or the fact that a
  matching DHCP packet arrived during the collection window.

`advertised_by_peer`
: A claim made by another protocol participant. The proposed address, subnet
  mask, routers, DNS servers, lease timers, MTU, domain information, and routes
  in a DHCP Offer are all peer advertisements. They are not proof of authority,
  reachability, acceptance, or local configuration.

`derived`
: A value computed from other evidence. For example, a displayed network
  prefix can be derived from an offered address and advertised subnet mask. A
  derived value must retain links to or context about its inputs where the
  public model provides them.

`speculative`
: A cautious hypothesis that may guide investigation but is not established by
  the available evidence. Version 0.1.0 does not invent speculative DHCP facts
  merely because an option is absent.

## DHCPv4 example

Receiving a matching packet is directly observed. The packet's DHCP options
are still claims by its sender, so normalized configuration from an Offer is
classified `advertised_by_peer`. If the CLI calculates `192.0.2.0/24` from
offered address `192.0.2.117` and subnet mask `255.255.255.0`, that network is
derived; it is not a configured route or proof that the subnet is usable.

An offer does not mean that:

* L2LinkScope accepted a lease;
* the sender is the legitimate DHCP server;
* the advertised router or DNS server is reachable or trustworthy;
* the host installed any advertised value; or
* no other DHCP server exists.

## Missing evidence

No Offer during a bounded probe means only that no matching, valid Offer was
observed during that window. It does not prove that the network has no DHCP
server. Malformed or unrelated replies are not silently converted into facts;
they may produce structured warnings or an exit category described in
[Exit Codes](ExitCodes.md).

## Serialization

Evidence classes use explicit stable strings in JSON rather than Rust debug
representations. The top-level envelope declares a schema version. See
[JSON Output](JsonOutput.md). The schema remains experimental during 0.x.

## Product boundary

Evidence classification reinforces the core rule: **L2LinkScope observes and
probes. It does not configure.** The model must never describe a DHCP offer as
trusted, accepted, leased, installed, or locally configured.
