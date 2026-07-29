# Release Process

L2LinkScope releases are deliberate maintainer actions. Ordinary CI and the
release-readiness workflow never publish crates or create a GitHub release.

## Reproducible checks

From a clean checkout of the intended tag on Linux:

```bash
rustup show
cargo metadata --locked --format-version 1 >/dev/null
scripts/check.sh
cargo build --release --locked -p l2linkscope
./target/release/l2linkscope --version
for manifest in crates/*/Cargo.toml; do
  cargo package --locked --list --manifest-path "$manifest"
done
```

The output must report version `0.1.0` for the initial release. Review package
contents and confirm that every package contains its license and required
documentation. The current manifests set `publish = false`; changing
publication intent is a separate reviewed decision.

## Linux binary artifact

The release workflow builds `l2linkscope` for
`x86_64-unknown-linux-gnu`, creates a compressed archive, and writes a SHA-256
checksum. To reproduce that locally:

```bash
install -d dist
cp target/release/l2linkscope dist/l2linkscope
cp LICENSE README.md dist/
tar -C dist -czf l2linkscope-0.1.0-x86_64-unknown-linux-gnu.tar.gz \
  l2linkscope LICENSE README.md
sha256sum l2linkscope-0.1.0-x86_64-unknown-linux-gnu.tar.gz \
  > l2linkscope-0.1.0-x86_64-unknown-linux-gnu.tar.gz.sha256
```

This is a dynamically linked GNU/Linux x86-64 binary. Release notes must name
the distribution and glibc baseline used by the release runner. Installation
requires copying the binary and, for active probing, granting only the runtime
privileges described in [Security and Privileges](SecurityModel.md).

A musl build is desirable only after raw-socket and interface behavior has been
validated on that target; it is not a 0.1.0 requirement.

## Maintainer release checklist

1. Obtain human review of public Rust APIs, JSON schema, unsafe boundaries, and
   the DHCP state-machine invariant.
2. Run unprivileged and isolated privileged checks and review their logs.
3. Confirm `CHANGELOG.md` accurately describes the tag and contains no
   unreleased claim presented as shipped.
4. Inspect `cargo package --list` output and license inclusion for every crate.
5. Build the binary from the tag, record the target and glibc environment, and
   verify the generated checksum on a second invocation.
6. Create the GitHub source release manually, attach the binary archive and
   checksum, and include privilege and dynamic-linking notes.
7. Do not publish to crates.io until package names, metadata, compatible path
   dependency versions, and Trusted Publishing have been reviewed separately.

Crates.io publication may happen after the GitHub release; it is not automated
by this repository's 0.1.0 workflows.
