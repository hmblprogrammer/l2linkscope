# l2linkscope-core

Portable domain foundation for L2LinkScope.

This crate will eventually contain normalized observation, evidence, interface,
session, warning, and error concepts shared by protocol parsers, Linux
acquisition code, and presentation layers.

Current status: placeholder only. No observation model is implemented yet.

This crate forbids unsafe code and must remain independent of Linux-only APIs,
raw sockets, command-line parsing, and asynchronous runtimes unless a future
architecture document explicitly justifies a change.
