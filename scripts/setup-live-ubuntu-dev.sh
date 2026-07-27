#!/usr/bin/env bash
set -euo pipefail

# Idempotent development bootstrap for the ephemeral Ubuntu-like live host.
# This script installs the tools needed to build and test L2LinkScope after
# every reboot of the ISO-backed test machine.

REPO_URL="${REPO_URL:-https://github.com/hmblprogrammer/l2linkscope.git}"
REPO_BRANCH="${REPO_BRANCH:-feature/1-initial-implementation}"
WORKDIR="${WORKDIR:-$HOME/src/l2linkscope}"
RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"
CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"

export DEBIAN_FRONTEND=noninteractive
export RUSTUP_HOME
export CARGO_HOME

log() {
  printf '\n==> %s\n' "$*"
}

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    printf 'Required command not found after setup: %s\n' "$1" >&2
    exit 1
  fi
}

log "Installing system packages"
sudo apt-get update
sudo apt-get install -y --no-install-recommends \
  build-essential \
  ca-certificates \
  clang \
  curl \
  dnsmasq-base \
  git \
  iproute2 \
  iputils-ping \
  isc-dhcp-client \
  libcap2-bin \
  libpcap-dev \
  llvm \
  net-tools \
  pkg-config \
  tcpdump \
  tshark \
  wireshark-common

log "Installing Rust toolchain"
if ! command -v rustup >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile default --default-toolchain stable
fi

if [ -f "$CARGO_HOME/env" ]; then
  # shellcheck disable=SC1091
  source "$CARGO_HOME/env"
else
  export PATH="$CARGO_HOME/bin:$PATH"
fi

rustup default stable
rustup component add rustfmt clippy

log "Preparing repository checkout"
mkdir -p "$(dirname "$WORKDIR")"
if [ -d "$WORKDIR/.git" ]; then
  git -C "$WORKDIR" fetch origin "$REPO_BRANCH"
  git -C "$WORKDIR" checkout "$REPO_BRANCH"
  git -C "$WORKDIR" pull --ff-only origin "$REPO_BRANCH"
else
  git clone --branch "$REPO_BRANCH" "$REPO_URL" "$WORKDIR"
fi

log "Installing repository Rust toolchain"
(
  cd "$WORKDIR"
  rustup show
  rustup component add rustfmt clippy
)

log "Verifying development tools"
require_command git
require_command ip
require_command rustc
require_command cargo
require_command tcpdump
require_command tshark

(
  cd "$WORKDIR"
  rustc --version
  cargo --version
)
ip -Version

log "Checking Linux network namespace support"
sudo ip netns add l2linkscope-setup-check
sudo ip netns delete l2linkscope-setup-check

log "Setup complete"
printf 'Repository: %s\n' "$WORKDIR"
printf 'Branch:     %s\n' "$(git -C "$WORKDIR" branch --show-current)"
