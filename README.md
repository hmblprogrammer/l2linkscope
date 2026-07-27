# L2LinkScope

L2LinkScope is a Rust project for discovering and describing the local Layer 2
and adjacent network environment.

Current status: this repository is a scaffold only. It contains the initial
Cargo workspace, documentation, and CI structure for future implementation
work. It does not yet contain network-discovery functionality.

The executable name is `l2linkscope`.

> L2LinkScope observes and probes. It does not configure.

Even when future releases add active probing, L2LinkScope must never
automatically:

* accept a DHCP lease
* assign an address
* install a route
* join a VLAN
* authenticate to a network
* change interface configuration

## Intended Users

L2LinkScope is intended for:

* developers building network-discovery tools
* Linux operators who need transparent local-network observations
* downstream applications that need reusable discovery crates
* future JoshOS integration through a separate consumer service

L2LinkScope is not part of JoshOS and does not depend on JoshOS paths,
services, IPC, build pipelines, or release processes.

## Workspace

The workspace is organized into four packages:

* `l2linkscope-core`: portable domain concepts for future observations and
  evidence.
* `l2linkscope-protocols`: protocol encoding and parsing, without packet I/O.
* `l2linkscope-linux`: Linux-specific acquisition and transport code.
* `l2linkscope`: the Linux command-line executable.

The current crates intentionally expose only placeholders. Future pull requests
will add models, parsers, Linux acquisition, and presentation in small steps.

## Architecture

The intended dependency direction is:

```text
l2linkscope-core

l2linkscope-protocols
    -> l2linkscope-core when normalized models are needed

l2linkscope-linux
    -> l2linkscope-core
    -> l2linkscope-protocols when packet parsers are needed

l2linkscope CLI
    -> l2linkscope-core
    -> l2linkscope-protocols
    -> l2linkscope-linux
```

See [Architecture](Documentation/Architecture.md) for the package boundaries
and the relationship between portable crates, Linux code, the CLI, and future
external consumers.

## Safety Posture

No unsafe code exists in this scaffold. `l2linkscope-core` and
`l2linkscope-protocols` forbid unsafe code.

Future Linux syscall or raw-socket work must be isolated, reviewed explicitly,
wrapped behind safe Rust interfaces, and justified with safety comments. Unsafe
code must not leak into the core or protocol crates.

## Build And Test

Install the Rust toolchain declared in `rust-toolchain.toml`, then run:

```bash
cargo fmt --all --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
```

Or run the local helper:

```bash
scripts/check.sh
```

The placeholder CLI currently supports only:

```bash
l2linkscope --help
l2linkscope --version
```

Discovery commands have not yet been implemented.

## Linux Expectations

The eventual command-line utility is Linux-oriented. Ordinary model and parser
tests should remain unprivileged and portable where practical. Future packet
acquisition, raw-socket, and network-namespace integration tests will require
Linux and may require elevated privileges or Linux capabilities.

## Non-Goals

This scaffold does not implement:

* interface inventory
* packet capture
* packet parsing
* DHCP, ARP, VLAN, LLDP, IPv6 Neighbor Discovery, or Router Advertisements
* raw sockets or netlink
* JoshOS integration
* crate publication
* release publishing

## Project Documents

* [Architecture](Documentation/Architecture.md)
* [Evidence Model](Documentation/EvidenceModel.md)
* [Protocol Scope](Documentation/ProtocolScope.md)
* [Roadmap](Documentation/Roadmap.md)
* [Testing](Documentation/Testing.md)
* [Repository Settings](Documentation/RepositorySettings.md)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Contributors and automated coding
agents must also follow [AGENTS.md](AGENTS.md).

## Security

See [SECURITY.md](SECURITY.md) for vulnerability reporting guidance.

## License

L2LinkScope is licensed under the [MIT License](LICENSE).
