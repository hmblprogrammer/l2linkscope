# Testing

L2LinkScope testing should grow in layers as functionality is added.

## Unit Tests

Unit tests should cover small model, parser, and formatting behaviors. They
should be deterministic and should avoid privileged operating-system features.

## Fixture-Driven Protocol Tests

Protocol parsers should use reviewed fixtures for valid packets and realistic
variants. Fixtures should be small, documented, and stored deliberately.

Packet captures such as `.pcap` and `.pcapng` files are ignored by default until
a future reviewed fixture directory and retention policy exist.

## Malformed And Truncated Packet Tests

Every parser should reject malformed, truncated, contradictory, and oversized
input safely. Parser tests must not depend on global process state or live
network traffic.

## Property And Fuzz Testing

Parser entry points should be structured so they can be fuzzed without Linux
networking, sockets, CLI state, or privileges. Continuous fuzzing is future
work, but parser APIs should not make it difficult.

## Unprivileged Workspace Tests

Ordinary model and parser tests must not require root. The default local and CI
test path should remain usable by contributors without elevated privileges.

## Linux Network-Namespace Integration Tests

Future Linux integration tests should use network namespaces, virtual Ethernet
pairs, and controlled fixture services. They should keep all traffic inside an
isolated topology and verify host interface state before and after tests.

## Privileged Raw-Socket Tests

Tests that require raw sockets or Linux capabilities should live in clearly
named integration suites and CI jobs. They should fail with clear diagnostics
when required privileges are missing.

## Hardware-Specific Tests

Hardware-specific tests may eventually validate behavior on real NICs, switches,
or wireless devices. They should not be required for ordinary pull-request CI.
