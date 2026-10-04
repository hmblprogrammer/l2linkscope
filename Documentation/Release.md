# Release Process

L2LinkScope releases are deliberate maintainer actions. Ordinary CI and the
release-readiness workflow never publish crates or create a GitHub release.

## Reproducible checks

From a clean checkout of the candidate commit on Linux:

```bash
rustup show
cargo metadata --locked --format-version 1 >/dev/null
scripts/check.sh
cargo build --workspace --release --locked
./target/release/l2linkscope --version
cargo check --workspace --all-targets --all-features --locked \
  --target x86_64-unknown-linux-musl
scripts/check-security.sh
for manifest in crates/*/Cargo.toml; do
  cargo package --locked --list --manifest-path "$manifest"
done
```

The output must report version `0.1.0` for the initial release. Review package
contents and confirm that every package contains its license and required
documentation. The current manifests set `publish = false`; changing
publication intent is a separate reviewed decision. `scripts/check-security.sh`
prints the `cargo-audit` and `cargo-deny` versions before running the audit and
advisory, license, ban, and source-policy checks; a skipped command is not a
passing security result.

Before the tag exists, the changelog content stays under `Unreleased` without a
release date. The human maintainer moves that content to a dated `0.1.0`
heading only as part of the intentional tag and release action. The release
readiness workflow enforces the distinction between a branch candidate and a
`v0.1.0` tag.

## Library consumption boundary

There is no top-level `l2linkscope` facade library in 0.1.0. The public package
roles are:

* `l2linkscope-core` for portable domain and evidence models;
* `l2linkscope-protocols` for portable protocol encoding and parsing;
* `l2linkscope-linux` for the supported high-level Linux library entry point;
* `l2linkscope` for the Linux CLI binary only.

All four packages remain `publish = false`. Until crates.io publication is
separately reviewed, an external consumer can pin the supported Linux library
to a reviewed commit:

```toml
l2linkscope-linux = { git = "https://github.com/hmblprogrammer/l2linkscope.git", rev = "<reviewed-commit-sha>" }
```

Git dependencies automatically resolve the workspace path dependencies used by
that package. After a maintainer creates a real immutable release tag, a
consumer may use `tag = "v0.1.0"` instead of `rev`; do not depend on a tag that
does not yet exist.

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

The candidate must also pass a compile-time compatibility check for
`x86_64-unknown-linux-musl`. The check covers the complete workspace, including
the CLI, but does not create or claim a tested MUSL release artifact. Runtime
network-namespace validation currently exercises the GNU/Linux build.

## Maintainer release checklist

1. Obtain human review of public Rust APIs, JSON schema, unsafe boundaries, and
   the DHCP state-machine invariant.
2. Run unprivileged and isolated privileged checks and review their logs.
3. Confirm the exact candidate passed `cargo-audit`, `cargo-deny`, the MUSL
   workspace check, and the privileged network-namespace suite; retain the
   command, tool-version, and result logs.
4. Confirm `CHANGELOG.md` keeps candidate content under `Unreleased`. When and
   only when making the release, move it to a dated `0.1.0` heading using the
   actual release date.
5. Inspect `cargo package --list` output and license inclusion for every crate.
6. Build the binary from the tag, record the target and glibc environment, and
   verify the generated checksum on a second invocation.
7. Create the GitHub source release manually, attach the binary archive and
   checksum, and include privilege and dynamic-linking notes.
8. Do not publish to crates.io until the libraries have been consumed by an
   external project and their APIs, package metadata, compatible dependency
   versions, and Trusted Publishing have been reviewed separately.

Crates.io publication is intentionally deferred and is not automated by this
repository's 0.1.0 workflows.
