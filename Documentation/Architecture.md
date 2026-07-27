# Architecture

L2LinkScope is organized as a four-package Rust workspace. The scaffold defines
the package boundaries without implementing discovery functionality.

## Workspace Packages

`l2linkscope-core` is the portable domain layer. It will eventually contain
observation, evidence, interface, session, snapshot, warning, and error models.
It must not depend on Linux-only APIs, raw sockets, packet acquisition, or CLI
formatting.

`l2linkscope-protocols` owns protocol bytes. It will eventually encode and parse
protocol messages and maintain protocol fixtures. It must not transmit packets,
receive packets, or open sockets.

`l2linkscope-linux` is the Linux-specific acquisition layer. It will eventually
contain interface inventory, privilege handling, packet acquisition, and bounded
active transports. Linux-specific details belong here rather than in portable
crates.

`l2linkscope` is the command-line executable. It presents results to users and
maps command-line arguments to library calls. Formatting and process exit
behavior belong here rather than in library crates.

## Dependency Direction

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

The initial crates do not add dependency edges merely to demonstrate this graph.
Edges should be introduced when real code needs them. Cyclic dependencies are
not allowed.

## Separation Of Concerns

Acquisition is responsible for obtaining bytes or platform facts from an
operating system.

Parsing is responsible for interpreting protocol bytes and rejecting malformed
input safely.

Models are responsible for normalized, reusable representations of observations
and evidence.

Presentation is responsible for human-readable and machine-readable command
output.

These responsibilities should remain separated so future agents can work on
small, reviewable changes without forcing unrelated crates to change.

## Linux And Portable Code

Portable models and protocol parsers should remain independent of Linux. Linux
code may use Linux-only APIs and privileges, but those details should be
isolated in `l2linkscope-linux`.

Future unsafe Linux syscall code must be isolated in small modules, reviewed
explicitly, wrapped in safe interfaces, justified with safety comments, and kept
out of `l2linkscope-core` and `l2linkscope-protocols`.

## External Consumers

The CLI and JoshOS integrations are sibling consumers of the public crates.
L2LinkScope is not part of JoshOS and must not depend on JoshOS filesystem
paths, services, Worlds, IPC conventions, build pipelines, initramfs or ISO
construction, or JoshOS-specific terminology.

Future JoshOS integration should live outside this repository or in a clearly
separate adapter that consumes the public crates without coupling the crates to
JoshOS.
