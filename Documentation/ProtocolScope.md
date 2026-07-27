# Protocol Scope

This document lists anticipated protocol and platform areas. The items below are
planned or possible future work, not implemented functionality.

## Initial Areas

* interface inventory
* Ethernet
* IEEE 802.1Q VLAN metadata
* ARP
* IPv4 and IPv6
* ICMPv6 Neighbor Discovery
* Router Advertisements
* DHCPv4
* DHCPv6
* LLDP

## Potential Later Additions

* mDNS and DNS-SD
* SSDP
* CDP
* EAPOL
* STP
* LACP
* VRRP and similar advertisements
* ethtool diagnostics
* Wi-Fi discovery through Linux `nl80211`

## Current Status

The repository scaffold does not implement any protocol parser, encoder,
packet-capture path, or active probe.
