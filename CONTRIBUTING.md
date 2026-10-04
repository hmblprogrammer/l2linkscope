# Contributing

Thank you for helping with L2LinkScope. This repository is young, so clear
boundaries and small changes matter more than speed.

Contributors and automated coding agents must follow [AGENTS.md](AGENTS.md).

## Setup

Install the Rust toolchain declared in `rust-toolchain.toml`. On Linux, a normal
Rustup installation is sufficient for the current scaffold.

For the ephemeral live Ubuntu test host used by maintainers, run:

```bash
scripts/setup-live-ubuntu-dev.sh
```

That host setup script installs tools. The normal local check script does not.

## Build

```bash
cargo check --workspace --all-targets
```

## Formatting

```bash
cargo fmt --all --check
```

Run `cargo fmt --all` before sending formatting-only fixes.

## Linting

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

The lint set is intentionally small at this stage. Avoid adding broad
opinionated lint groups unless the rationale is documented.

## Testing

```bash
cargo test --workspace --all-features
```

Ordinary model and parser tests must not require root. Future privileged tests
should be clearly named and separated from normal pull-request checks.

## Documentation

Public crates and public items should have useful documentation. Planned
features must be described as planned, not implemented.

Build documentation with:

```bash
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
```

## Pull Requests

Prefer small pull requests with one clear purpose. Keep refactors separate from
feature work when practical.

Security-sensitive changes, especially parser boundaries, raw sockets,
capability handling, and unsafe code, require explicit review.

Do not claim that functionality exists until it is implemented and tested.
