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
* a malformed response carrying the active transaction and client identities;
* a short garbage datagram that cannot establish probe identity;
* an otherwise valid matching Offer sent from the wrong UDP source port;
* malformed traffic with an unrelated transaction ID;
* malformed traffic with an unrelated client hardware address;
* a sequence of unrelated garbage followed by no Offer;
* a valid matching Offer after unrelated malformed traffic;
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
No `cargo-fuzz` target is included in 0.1.0, and the ordinary malformed-input
tests must not be described as coverage-guided fuzzing. Adding a standard target
seeded from the checked-in fixtures remains bounded follow-up work.

## Dependency security

Release readiness installs the pinned `cargo-audit` and `cargo-deny` versions
under current stable Rust, then runs:

```bash
scripts/check-security.sh
```

The script prints both tool versions, executes `cargo audit --deny warnings`,
and checks advisory, license, duplicate/source, and registry policy through
`cargo deny`. It fails on a failed tool invocation. The Security workflow runs
this job on manual and scheduled invocations and on pushes to `main`; pull
requests retain the fast secret scan while release readiness always executes the
dependency checks.

## MUSL compatibility

The release-readiness workflow installs the `x86_64-unknown-linux-musl` standard
library and checks every workspace package and target:

```bash
cargo check --workspace --all-targets --all-features --locked \
  --target x86_64-unknown-linux-musl
```

This covers `l2linkscope-core`, `l2linkscope-protocols`, `l2linkscope-linux`, and
the `l2linkscope` CLI at compile/check time. The 0.1.0 workflow still produces a
dynamically linked GNU/Linux binary artifact; a MUSL runtime artifact and
hardware validation are separate future release decisions.

## CI separation

The `CI / workspace` job runs the fast unprivileged checks. The separate
`Privileged Linux Integration / network-namespace-dhcp` workflow invokes the
namespace suite with `sudo`; it does not contact an external DHCP server or the
runner's real interfaces. Parser-only work therefore remains testable without
privileged setup.

Hardware-specific tests are not required for ordinary pull requests. Results
from real NICs or DHCP infrastructure must never replace the isolated fixture
suite.
