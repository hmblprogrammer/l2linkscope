# Changelog

All notable changes to L2LinkScope are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Semantic Versioning](https://semver.org/) for release
versions. Public Rust APIs and JSON formats remain experimental during 0.x.

## [Unreleased]

The changes below are proposed `0.1.0` release-candidate content. No release
date is assigned until a human maintainer intentionally creates the tag and
release.

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
* Timestamped interface-inventory snapshots for consumers that retain results.
* Separately modeled observed UDP transport peers and peer-advertised DHCP
  Server Identifier values.
* Isolated Linux network-namespace integration suite that records DHCP message
  types and verifies link, address, and route state remain unchanged.
* Separate unprivileged, privileged, security, and non-publishing release-check
  workflows, including checksummed GNU/Linux candidate artifacts.
* Git dependency documentation for the three reusable library packages while
  crates.io publication remains disabled.

### Changed

* DHCP traffic can affect a probe only after arriving from UDP/67 and matching
  both the active transaction ID and client hardware address.
* Interface inventory JSON now uses the common discovery-snapshot envelope with
  session start and completion timestamps.
* Human-readable output preserves ordinary Unicode while escaping terminal
  control characters and invisible formatting controls.
* Release readiness now executes dependency advisory/license/source policy
  checks and validates the complete workspace for `x86_64-unknown-linux-musl`.

### Security

* Core and protocol crates forbid unsafe code; the Linux implementation uses
  safe Rust interfaces and does not apply advertised configuration.
* Active probing is explicit and bounded, sends no hostname by default, and
  stops after Offer collection without DHCP Request or lease acceptance.
* Dependency advisory, license/source policy, and committed-secret checks are
  part of repository automation and release readiness.
* Unrelated or insufficiently attributable malformed UDP traffic cannot turn a
  normal no-Offer result into a malformed-response failure.
