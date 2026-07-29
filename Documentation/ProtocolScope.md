# Protocol Scope

## Implemented in 0.1.0

The only wire protocol implemented by the first release is the bounded DHCPv4
Discover/Offer workflow described in [DHCPv4](DhcpV4.md). Protocol construction
and parsing live in `l2linkscope-protocols`; Linux transmission and reception
live in `l2linkscope-linux`.

Linux interface inventory is also implemented, but it is platform discovery
rather than a wire-protocol parser.

## Explicitly not implemented

Version 0.1.0 does not expose placeholder commands or claim support for:

* passive ARP monitoring
* VLAN observation
* IPv6 Router Advertisement or DHCPv6
* LLDP, mDNS, DNS-SD, SSDP, CDP, EAPOL, STP, or LACP
* Wi-Fi discovery or topology mapping
* subnet scanning, arbitrary capture, or promiscuous mode
* DHCP lease acceptance or any automatic configuration

These may be considered incrementally after 0.1.0. Their possible future use
does not justify speculative public APIs in this release.
