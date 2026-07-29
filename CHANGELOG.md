# Changelog

All notable changes to L2LinkScope are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Semantic Versioning](https://semver.org/) for release
versions. Public Rust APIs and JSON formats remain experimental during 0.x.

## Unreleased

No changes yet.

## [0.1.0] - 2026-07-28

### Added

* Four-package Cargo workspace separating portable models, DHCP protocol bytes,
  Linux acquisition, and CLI presentation.
* Evidence-aware interface, discovery session, observation, DHCPv4 offer,
  warning, error, and snapshot models with versioned JSON serialization.
* Bounds-checked DHCPv4 Discover encoding and Offer parsing, including common
  options, domain search, classless routes, malformed-input fixtures, and
  matching by transaction ID and client hardware address.
* Unprivileged Linux interface inventory with runtime identity, link state,
  hardware address, MTU, configured addresses, and DHCP probe eligibility.
* Explicit, bounded DHCPv4 Discover/Offer probing on a selected interface with
  multiple-offer collection and identical-offer deduplication.
* Human-readable and JSON `interfaces` and `probe dhcp4` commands with stable
  initial exit-code categories.
* Isolated Linux network-namespace integration suite that records DHCP message
  types and verifies link, address, and route state remain unchanged.
* Separate unprivileged, privileged, security, and non-publishing release-check
  workflows, including checksummed GNU/Linux candidate artifacts.

### Security

* Core and protocol crates forbid unsafe code; the Linux implementation uses
  safe Rust interfaces and does not apply advertised configuration.
* Active probing is explicit and bounded, sends no hostname by default, and
  stops after Offer collection without DHCP Request or lease acceptance.
* Dependency advisory, license/source policy, and committed-secret checks are
  part of repository automation.

[0.1.0]: https://github.com/hmblprogrammer/l2linkscope/releases/tag/v0.1.0
