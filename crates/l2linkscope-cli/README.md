# l2linkscope

Linux command-line interface for L2LinkScope 0.1.0.

The package produces the `l2linkscope` executable. It keeps command parsing,
human-readable presentation, versioned JSON envelopes, diagnostics, and stable
process exit categories outside the reusable crates.

```bash
l2linkscope interfaces
l2linkscope interfaces --json
sudo l2linkscope probe dhcp4 enp3s0
sudo l2linkscope probe dhcp4 enp3s0 --timeout 5s --json
```

The DHCP command sends one Discover during a bounded collection period and
reports zero or more matching Offers. It does not send Request, Decline,
Release, or Inform and never applies advertised configuration. Standard output
contains results; diagnostics and verbose progress use standard error.

See the root README and `Documentation/ExitCodes.md` for privilege, JSON, exit
code, and safety details.
