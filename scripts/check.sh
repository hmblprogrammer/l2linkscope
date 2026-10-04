#!/usr/bin/env bash
set -euo pipefail

cargo fmt --all --check
cargo metadata --locked --format-version 1 >/dev/null
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps --locked
cargo build --release --locked -p l2linkscope
