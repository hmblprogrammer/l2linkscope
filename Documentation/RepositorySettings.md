# Repository Settings

These settings cannot be reliably committed as repository files. Maintainers
should configure them in GitHub. Repository files document the intended policy
but do not attempt to change repository settings.

## Observed state for the 0.1.0 candidate

A read-only GitHub inspection on 2026-10-03 found the repository public, with:

* no branch protection rule on `main`;
* no repository rulesets;
* Dependabot alerts disabled;
* private vulnerability reporting disabled; and
* secret-scanning availability not fully observable with the inspecting token.

The committed Gitleaks workflow does run on pull requests, but workflow files
are not substitutes for repository protection settings. No repository settings
were changed during release hardening. A repository administrator must verify
and enable the settings below before deciding whether to release.

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
* a completed dependency-security run on the exact candidate before release
  decisions

Disallow force pushes to the default branch. Disallow deletion of the default
branch.

## Pull Requests

Require human review for pull requests. Prefer small, focused pull requests that
separate generated implementation, generated tests, documentation, and review
fixes when practical.

Security-sensitive changes should receive explicit review from a maintainer who
understands the privilege boundary.

## Security

Enable vulnerability alerts and Dependabot alerts. Consider Dependabot security
updates after alerts are enabled; ordinary version-update automation can remain
a separate maintainer decision.

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
version `0.20.2` with `cargo +stable install --locked --version`, invokes the
shared `scripts/check-security.sh` gate, and runs a pinned Gitleaks action
against committed history. Dependency checks run on manual dispatch, the
schedule, and pushes to `main`; pull requests retain the faster committed-secret
check. Release readiness independently executes the same dependency gate, so a
release candidate cannot pass that workflow while audit or policy checks are
skipped.
The project MSRV remains Rust `1.85`, while the security tools run under the
current stable toolchain so they can understand the current RustSec advisory
database and policy formats. This keeps scheduled checks explicit without
committing third-party binaries or adding long-lived credentials.

## Tags And Releases

Protect version tags such as `v*` from deletion or forced updates where GitHub
rulesets permit it.

Release publishing should require an explicit human action and should not be
triggered by ordinary pull-request CI.
