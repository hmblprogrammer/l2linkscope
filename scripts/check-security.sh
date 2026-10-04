#!/usr/bin/env bash
set -euo pipefail

SECURITY_TOOLCHAIN="${SECURITY_TOOLCHAIN:-stable}"
SECURITY_CARGO=(cargo "+$SECURITY_TOOLCHAIN")

"${SECURITY_CARGO[@]}" audit --version
"${SECURITY_CARGO[@]}" deny --version
"${SECURITY_CARGO[@]}" audit --deny warnings
"${SECURITY_CARGO[@]}" deny check advisories licenses bans sources
