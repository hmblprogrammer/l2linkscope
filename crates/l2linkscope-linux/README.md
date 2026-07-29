# l2linkscope-linux

Safe Linux acquisition for L2LinkScope.

This crate provides read-only interface inventory and a bounded DHCPv4
Discover/Offer transport. Interface addresses come from the kernel
`getifaddrs` API, while link metadata comes from Linux sysfs. The DHCP transport
binds UDP/68 to exactly one selected interface, sends one Discover, collects and
deduplicates matching Offers, and stops without accepting or applying a lease.

```rust,no_run
use l2linkscope_linux::{DhcpV4ProbeOptions, interfaces, probe_dhcp_v4};

let inventory = interfaces()?;
let snapshot = probe_dhcp_v4("enp3s0", DhcpV4ProbeOptions::default())?;
# Ok::<(), l2linkscope_linux::LinuxError>(())
```

Inventory is unprivileged. Probing normally requires `CAP_NET_RAW` for
interface binding and `CAP_NET_BIND_SERVICE` for UDP/68, or root. Socket
conflicts with an existing DHCP client are reported and the client is never
stopped. The public interface is entirely safe Rust and the crate performs no
network-configuration operation.

See the root architecture, DHCPv4, security, and testing documents for error
behavior and integration guarantees.
