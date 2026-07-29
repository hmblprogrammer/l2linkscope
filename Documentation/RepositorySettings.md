# Repository Settings

These settings cannot be reliably committed as repository files. Maintainers
should configure them in GitHub. Repository files document the intended policy
but do not attempt to change repository settings.

## Branches

Recommended default branch: `main`.

Protect the default branch with a branch protection rule or ruleset. Require
pull requests before merging and require status checks to pass.

Recommended required checks:

* `CI / workspace`
* `Privileged Linux Integration / network-namespace-dhcp`
* `Security / committed-secrets`
* `Release Check / release-readiness` and
  `Release Check / privileged-integration` when preparing a release
* scheduled dependency-security results before release decisions

Disallow force pushes to the default branch. Disallow deletion of the default
branch.

## Pull Requests

Require human review for pull requests. Prefer small, focused pull requests that
separate generated implementation, generated tests, documentation, and review
fixes when practical.

Security-sensitive changes should receive explicit review from a maintainer who
understands the privilege boundary.

## Security

Enable vulnerability alerts and Dependabot alerts.

Enable secret scanning where available for the repository visibility and GitHub
plan.

Enable private vulnerability reporting when available so parser and
privilege-boundary issues can be coordinated before public disclosure.

## Actions

Set workflow permissions to read-only by default. Grant broader permissions only
to specific workflows that require them.

Do not store long-lived crates.io publishing tokens as repository secrets.
Future publication should use crates.io Trusted Publishing or another
short-lived credential mechanism.

The security workflow installs `cargo-audit` version `0.22.2` and `cargo-deny`
version `0.20.2` with `cargo +stable install --locked --version` and runs a
pinned Gitleaks action against committed history.
The project MSRV remains Rust `1.85`, while the security tools run under the
current stable toolchain so they can understand the current RustSec advisory
database and policy formats. This keeps scheduled checks explicit without
committing third-party binaries or adding long-lived credentials.

## Tags And Releases

Protect version tags such as `v*` from deletion or forced updates where GitHub
rulesets permit it.

Release publishing should require an explicit human action and should not be
triggered by ordinary pull-request CI.
