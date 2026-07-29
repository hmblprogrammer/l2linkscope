# Testing

L2LinkScope separates portable, unprivileged checks from tests that need Linux
network namespaces and privileged DHCP ports.

## Unprivileged checks

Run from the repository root:

```bash
scripts/check.sh
```

The script executes:

```bash
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps --locked
cargo build --release --locked -p l2linkscope
```

Ordinary model, JSON, parser, interface-normalization, CLI parsing, and exit-code
tests do not require root. Protocol tests use deterministic byte fixtures and
cover valid Offers, common and repeated options, padding, unknown options,
truncation, invalid lengths, missing cookies, mismatched transaction IDs and
hardware addresses, non-Offer message types, malformed classless routes, and
duplicate-option behavior.

## Privileged namespace suite

On Linux with `iproute2`, Python 3, and root:

```bash
cargo build --locked -p l2linkscope
sudo scripts/test-privileged.sh
```

The unprivileged command builds the debug CLI. The privileged script then
creates two disposable network namespaces connected only by a veth pair:

```text
client namespace                       server namespace
  lsclient0  <------ veth pair ------>   lsserver0
  l2linkscope                             DHCP fixture (UDP/67)
```

Neither veth endpoint is connected to the host network, a bridge, or the
Internet. The standard-library fixture records every DHCP message type it
receives and returns deterministic payloads on UDP/68.

The current suite exercises:

* one Offer from one server;
* two distinct Offers representing competing servers;
* no response;
* a malformed response;
* unrelated transaction ID;
* unrelated client hardware address;
* retransmission of an identical Offer;
* an unprivileged process receiving the documented privilege error; and
* the selected interface becoming unavailable during a synchronized probe.

For every probe scenario it snapshots the client's link details, addresses,
and routes before and after. It fails if any state changes, if the observed
Offer count differs, or if the fixture records anything other than the single
expected DHCP Discover. This explicitly detects a Request or any other
follow-up message.

The unavailable-interface scenario waits until the probe owns UDP port 68,
administratively lowers the disposable veth, and verifies the stable transport
failure exit category. This state change is intentionally outside the normal
before/after invariants because the test topology, not L2LinkScope, causes it.

## Fuzzing preparation

The DHCP parser accepts an immutable byte slice plus explicit expected
transaction and hardware identity. It does not depend on sockets, Linux,
privileges, global state, clocks, or the CLI. A future `cargo-fuzz` target can
call that entry point directly with arbitrary bytes and treat every structured
success or error as valid; any panic, abort, or out-of-bounds read is a defect.

A minimal future target is conceptually:

```rust,ignore
fuzz_target!(|data: &[u8]| {
    let expectation = OfferExpectation {
        transaction_id: expected_transaction_id,
        client_hardware_address: expected_mac,
    };
    let _ = parse_offer(data, expectation);
});
```

Continuous fuzzing is not required for 0.1.0. Curated malformed fixtures remain
part of ordinary CI so known regressions do not depend on a fuzzing service.

## CI separation

The `CI / workspace` job runs the fast unprivileged checks. The separate
`Privileged Linux Integration / network-namespace-dhcp` workflow invokes the
namespace suite with `sudo`; it does not contact an external DHCP server or the
runner's real interfaces. Parser-only work therefore remains testable without
privileged setup.

Hardware-specific tests are not required for ordinary pull requests. Results
from real NICs or DHCP infrastructure must never replace the isolated fixture
suite.
