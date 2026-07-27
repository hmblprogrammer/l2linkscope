# l2linkscope-linux

Linux-specific acquisition boundary for L2LinkScope.

This crate will eventually contain Linux interface inventory, packet
acquisition, privilege handling, and transport code. It must keep Linux-specific
behavior out of the portable core and protocol crates.

Current status: placeholder only. No netlink, raw-socket, packet capture, or
network-discovery functionality is implemented yet.

Future unsafe Linux syscall code must be isolated in small modules, reviewed
explicitly, documented with safety invariants, and wrapped behind safe
interfaces.
