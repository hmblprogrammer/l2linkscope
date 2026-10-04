# AI-Assisted Development Record

## 1. Purpose

L2LinkScope uses AI-assisted development. This file preserves human-readable
provenance for material work performed with AI assistance, including source
code, tests, documentation, automation, security review, and release
preparation. It also records significant mistakes and later corrections.

Git commits remain the authoritative implementation history. This record adds
context that is difficult to recover from commit subjects alone: which
requirements were human-directed, which implementation details were selected
by the AI, what validation actually ran, and what work remained for human
review or a later release. It is a summary, not a prompt transcript or a record
of private reasoning.

This document is not a substitute for source review, tests, continuous
integration, security review, or release approval. AI-generated work remains
subject to human review. A successful automated check establishes only the
property exercised by that check.

Unless an entry says otherwise, factual statements below are based on the
commits and diffs, repository documentation and tests, issue 1 and PR 2, or
GitHub Actions results. Rationale labeled as an AI choice is reconstructed from
those sources. Recommendations are not descriptions of settings or work that
was completed.

## 2. Identity and attribution

* **Josh Gitlin** is the human project owner/operator. This role does not, by
  itself, mean that Josh authored, line-reviewed, approved, or security-reviewed
  an individual change.
* **OpenAI Codex** is the AI coding system that produced the work represented
  by this record. The 17 commits covered below use the Git author identity
  `Codex Bot <codex@digitalfruition.com>` and include AI-generation trailers.
  Codex created those commits and pushed updates to the PR branch through its
  service-account environment.
* **Finley Brooks**, `finley.brooks`, and `fbrooks` identify the OpenAI Codex
  service account used during this work. They do not identify a human
  maintainer, operator, reviewer, or project decision-maker.

### Historical attribution correction

The 17 AI-generated commits covered here incorrectly identify a service account
named “Finley Brooks” as the human operator. Finley Brooks is the OpenAI Codex
service-account identity. Josh Gitlin is the human project owner/operator. The
incorrect trailers must not be interpreted as evidence of human authorship or
review.

The task that created this record ultimately instructed that the preceding 17
commits not be rewritten. They therefore remain unchanged in Git; this durable
record corrects their interpretation without hiding the original error. Josh's
identification as project owner/operator likewise does not assert that he
reviewed or approved each historical commit.

## 3. Human and AI decision boundary

### Human-directed decisions

The following boundaries are supported by issue 1 and the subsequent direction
from Josh for PR 2:

* L2LinkScope is a standalone, public, MIT-licensed project rather than an
  implementation developed initially inside LoomOS. The repository's current
  architecture documents likewise place any later JoshOS adapter outside this
  project.
* Reusable crates and a standalone Linux CLI are separate concerns. The initial
  architecture has four packages: portable core models, protocol bytes, Linux
  acquisition, and the `l2linkscope` CLI.
* The product observes and probes but does not configure. Passive observation
  is the default; the initial active DHCPv4 probe must be explicit and bounded,
  must stop after Offers, and must never accept a lease or change interface,
  address, route, DNS, VLAN, or authentication state.
* DHCPv4 Discover/Offer was selected as the first end-to-end capability. Hostile
  input, evidence classification, parser isolation, and a small privileged
  Linux boundary were explicit requirements.
* During hardening, Josh accepted the invariant that malformed input may affect
  a probe result only after enough evidence associates it with that probe.
* The supported 0.1.0 package boundary has three reusable libraries and one
  binary, with no top-level facade library. All packages remain
  `publish = false`; crates.io publication is deferred pending API review and
  downstream experience.
* MUSL compilation was required as compatibility evidence for later downstream
  LoomOS use, without adding LoomOS-specific code to L2LinkScope.
* Josh reviewed the AI pre-release findings and directed the subsequent
  attribution, timestamp, terminal, security-gating, API/publication, fixture,
  and repository-settings hardening. That direction is not the same as a
  completed line-by-line review or release approval.

### AI-selected implementation choices

Within those constraints, Codex selected or drafted material details including:

* the exact Rust types, serde representation, module boundaries, error enums,
  schema envelope, timestamp representation, and initial exit-code mapping;
* the dependency set (`serde`, `uuid`, `getrandom`, `if-addrs`, `socket2`,
  `clap`, and `serde_json`) and lockfile;
* DHCP option encoding, parser limits, duplicate-option behavior, stateless
  correlation helpers, and normalized representations for domain search and
  classless routes;
* Linux inventory via `getifaddrs` and sysfs, a UDP/68 socket bound to the
  selected interface, timeout bounds, retransmission policy, offer limits, and
  deduplication mechanics;
* human and JSON rendering, terminal-safe escaping, and the separation of the
  observed UDP peer from the advertised DHCP Server Identifier;
* the Python DHCP fixture, network-namespace/veth topology, state snapshots,
  and negative traffic scenarios;
* the shape and triggers of CI, security, privileged-integration, and
  release-readiness workflows, including pinned tool/action versions;
* development-host bootstrap automation and most repository and release
  documentation.

These were implementation choices made by the AI while carrying out human
direction. Their presence in the repository is not evidence of human approval.

## 4. Chronological AI contribution log

The following entries cover every commit from base
`453473ad56d5f7afe610bf42882f5b61c41064e0` through the implementation candidate
`a2ded29ef670791be5088390daabfe01ee53b034`.

### 2026-07-26 — disposable Ubuntu development bootstrap

**Commits:** `41263d155174d52562053e8ba33b2bc7a515705b`,
`5648f5a9ff2d9ff52739ae75d52012d348a20505`
**AI system:** OpenAI Codex

**Human direction.** Prepare a repeatable Linux development and test
environment before implementing issue 1.

**Work performed by AI.** Codex added an idempotent shell bootstrap for a
disposable Ubuntu host, an LF policy for shell scripts, and then corrected the
script's executable mode. The script installs Rust and native build/network
tools, checks out or updates the repository, and verifies that basic Linux
network-namespace operations are available. The AI used `sudo` on the
disposable development host for package installation and namespace validation.

**Material choices.** The bootstrap included compilation tools and utilities
needed for DHCP and namespace testing. It was development-host configuration,
not shipped product functionality, and it did not implement discovery.

**Validation and limits.** The script contains tool-presence and namespace
smoke checks. The repository does not retain a complete terminal transcript for
these two commits, so this record does not claim more than the scripted checks.

### 2026-07-27 — initial repository and four-package scaffold

**Commit:** `b26180f8ba28c310ecb3f41ef2884dd5ada6887f`
**AI system:** OpenAI Codex

**Human direction.** Establish the standalone project, four-package dependency
direction, MIT licensing, responsible-AI policy, and build/release scaffolding
without yet implementing network discovery.

**Work performed by AI.** Codex created the Cargo workspace and placeholder
packages `l2linkscope-core`, `l2linkscope-protocols`, `l2linkscope-linux`, and
`l2linkscope`; established Rust 1.85/edition 2024 metadata and lints; and wrote
the initial README, architecture, evidence, roadmap, testing, contribution,
security, conduct, changelog, and repository-settings documents. It also added
local checks, Dependabot configuration, and separate ordinary CI, security,
privileged-integration, and release-readiness workflows.

**Material choices.** The dependency graph and workflow separation made parser
and model checks unprivileged while reserving namespaces for an isolated Linux
job. Publication was explicitly disabled. Release automation packaged and
checksummed artifacts but did not publish crates.

**Validation and limits.** The scaffold supplied formatting, check, Clippy,
test, rustdoc, release-build, metadata, and package-content commands. At this
point the crates were placeholders: interface discovery, DHCP parsing,
transport, and a functional CLI did not exist.

### 2026-07-27 — security tools on stable Rust

**Commit:** `93d56a0ba01bc28b4406e88ea92ab9472b9a4c97`
**AI system:** OpenAI Codex

**Human direction.** Preserve the project's Rust 1.85/MSRV contract while
allowing current security tools to process current advisory and policy data.

**Work performed by AI.** Codex changed the security workflow to install and
run pinned `cargo-audit` and `cargo-deny` versions with stable Rust, rather than
changing the product toolchain. It tightened the zero-dependency scaffold's
license allow-list.

**Material choices.** Security-tool compilation was isolated from product
compilation. This is a tooling compatibility boundary, not a claim that code
requiring newer Rust entered the product.

**Validation and limits.** The commit message records validation on the Linux
workstation, but no durable per-command log is committed. Later PR security
runs only prove the jobs that GitHub reports as executed; a skipped dependency
job is not an audit result.

### 2026-07-28 — initial 0.1 dependency set

**Commit:** `9d643a8ad57d02762bf64da71573ddfecee0e21e`
**AI system:** OpenAI Codex

**Human direction.** Add the dependencies needed for issue 1 while preserving
the four-package boundaries and license policy.

**Work performed by AI.** Codex selected `serde` for public serialization,
`uuid` for session and observation identities, and `serde_json` for tests and
CLI JSON. The protocol crate depended only on core. The Linux crate gained
Linux-targeted `getrandom`, `if-addrs`, and `socket2`; the CLI gained `clap` and
the three libraries. Codex generated the lockfile and adjusted the license
policy for transitive Unicode data.

**Material choices.** Cryptographic randomness was placed in acquisition, not
protocol parsing; socket and interface dependencies stayed Linux-specific; and
CLI parsing stayed out of libraries.

**Validation and limits.** Cargo metadata and lockfile consistency are checked
by the repository scripts and later successful CI. Dependency selection still
required human review; no crates.io publication was enabled.

### 2026-07-28 — portable evidence and domain models

**Commit:** `480c187fee9a1c0abefcf717b9f7838ac23ec352`
**AI system:** OpenAI Codex

**Human direction.** Implement portable interface, session, observation,
evidence, DHCP Offer, diagnostics, and stable-JSON concepts while never
representing an inference as observation.

**Work performed by AI.** Codex created stable runtime interface identity from
kernel index plus available hardware identity; interface state and address
models; UUID-backed session and observation IDs; millisecond timestamps;
structured warnings/errors; normalized DHCP Offer and classless-route types;
and a `DiscoverySnapshot` envelope with schema version `0.1`.

**Material choices.** The four evidence classes serialize explicitly.
`Observation::dhcp_v4_offer` fixes the source/method and
`advertised_by_peer` classification, while custom deserialization rejects an
Offer relabeled as observed, derived, or speculative. The model contains no OS
handles and core forbids unsafe code.

**Validation and limits.** Unit tests cover evidence names, rejection of a
misclassified Offer, deterministic explicit JSON fields, schema version, and
MAC serialization. The `0.x` JSON schema remains experimental; runtime
interface identity is not promised across boots.

### 2026-07-28 — DHCPv4 protocol bytes

**Commit:** `11ff400c05dbb7fcb066703a4f8c265e2a65c1a1`
**AI system:** OpenAI Codex

**Human direction.** Build and parse DHCPv4 without performing I/O, accepting a
lease, or exposing unnecessary client identity.

**Work performed by AI.** Codex implemented broadcast DHCP Discover encoding
with the magic cookie, Discover type, useful parameter-request list, End option,
and no hostname. It implemented stateless Offer parsing and normalization for
addresses, server identifier, mask, routers, DNS, domain/domain-search, lease
timers, MTU, and classless routes.

**Material choices.** Every read is bounds-checked; message, option, address,
and domain counts are capped. Padding and unknown options are tolerated.
Duplicate list options are accumulated within caps, while conflicting scalar
duplicates are rejected. Structured errors cover truncation, lengths, cookie,
message type, correlation, routes, and compressed-domain loops. No Request or
lease-acceptance API was added.

**Validation and limits.** Fixture-driven tests cover minimal/common Offers,
multiple routers/DNS servers, unknown and padded options, truncation and bad
lengths, missing cookie, wrong XID/MAC, ACK-as-Offer, duplicate options,
classless-route errors, and domain compression loops. The byte-slice parser is
fuzz-harness ready, but no coverage-guided fuzz run was performed or claimed.

### 2026-07-28 — Linux inventory, acquisition, and CLI

**Commit:** `a3fc018e3c18fd43bf47c1f9186a034257bbc7b7`
**AI system:** OpenAI Codex

**Human direction.** Provide the functional Linux CLI and an explicit bounded
DHCPv4 probe without configuration changes.

**Work performed by AI.** Codex implemented interface inventory from
`getifaddrs` and sysfs, probe eligibility, safe Linux error normalization, a
cryptographically random XID, UDP/68 binding, `SO_BINDTODEVICE`, broadcast
transmission to UDP/67, collection, deduplication, and multiple-Offer warnings.
It implemented `interfaces` and `probe dhcp4`, human and versioned JSON output,
verbosity, timeout parsing, and documented exit categories.

**Material choices.** The default sends one Discover and collects for five
seconds; accepted timeouts are bounded from 250 milliseconds through 30
seconds. The socket is tied to the selected interface and never falls back.
The safe public Rust interface contains no network-configuration operation.
The initial privilege contract was root or `CAP_NET_RAW` plus
`CAP_NET_BIND_SERVICE`; existing-client bind conflicts become explicit errors.

**Validation and limits.** Unit tests cover normalization, options, argument
parsing, output helpers, and exit mapping; later CI compiled and tested these
paths on Linux. Root/capability and real packet behavior required the separate
namespace suite. Privileges were not dropped after socket setup, a limitation
later retained as follow-up work.

### 2026-07-28 — isolated privileged integration suite

**Commit:** `de8dd1dcdfab8826341c7c47be0dd0769f986f2e`
**AI system:** OpenAI Codex

**Human direction.** Exercise the real Linux transport only inside an isolated,
reproducible topology and prove that probing does not configure the client.

**Work performed by AI.** Codex added a Python DHCP fixture and shell harness
using network namespaces and a veth pair. The fixture records every DHCP
message received and provides deterministic one-Offer, two-Offer, no-response,
malformed, wrong-XID, wrong-MAC, and duplicate-Offer scenarios. The harness
also tests insufficient privilege, interface-down, and disappearing-interface
errors.

**Material choices.** The harness captures link, address, and route state before
and after each probe, requires exactly one Discover, and fails on Request or any
unexpected follow-up message. Test traffic remains in the namespaces and does
not contact an external DHCP server.

**Validation and limits.** The privileged GitHub job later passed at the
implementation candidate. At this historical point the acquisition loop still
could count unrelated malformed UDP/67 traffic as relevant; wrong-XID and
wrong-MAC valid Offers were ignored, but the stronger pre-parse attribution
invariant had not yet been implemented.

### 2026-07-28 — initial 0.1.0 release documentation

**Commit:** `3601a8bb559544738fa0c0dab225b3e3a09ed617`
**AI system:** OpenAI Codex

**Human direction.** Document the candidate's architecture, behavior,
privileges, security, tests, JSON, exits, and reproducible release process.

**Work performed by AI.** Codex expanded the root and crate READMEs and the
architecture, evidence, DHCP, JSON, security, testing, exit-code, release, and
repository-settings documents. It strengthened ordinary, privileged, secret
scanning, and release workflows and added checksummed GNU/Linux artifact
construction.

**Material choices.** Human output was separated from diagnostics, JSON was
declared experimental during `0.x`, and the release workflow remained
non-publishing. Pinned GitHub Actions and a full-history Gitleaks scan were
used.

**Validation and limits.** CI, privileged integration, and committed-secret
GitHub runs completed successfully for this SHA. The dependency-security job
was skipped on the PR event. The documentation incorrectly presented 0.1.0
with a release date even though no tag or GitHub release existed; later
hardening moved it back to an Unreleased candidate and added a gate against
that error.

### 2026-10-03 — AI-assisted pre-release review and attribution regressions

**Commit:** `28aeaabc8ec5ce004fa451f3fda71007e6de6557`
**AI system:** OpenAI Codex

**Human direction.** Following an AI-performed pre-release review, add tests
for the packet-attribution boundary before changing acquisition behavior. Josh
reviewed the findings and directed the hardening work; this was not release
approval.

**Work performed by AI.** The review identified that unrelated malformed DHCP
traffic could affect the probe result, ambiguity around the supported library
and publication contract, a dependency-security job that had not actually run
on the PR path, inaccurate release metadata, missing inventory timestamps,
terminal-control concerns, conflation of UDP source with DHCP Server
Identifier, fuzzing and privilege-boundary follow-ups, and repository security
setting recommendations. Codex then expanded the fixture with wrong-source-port,
short-garbage, malformed wrong-XID/MAC, unrelated-garbage, and
valid-after-unrelated scenarios.

**Material choices.** The tests express the boundary separately from full DHCP
syntax parsing. The test-only commit intentionally documented behavior that the
next implementation commit corrected; no completed GitHub run for this
intermediate SHA is present in the inspected Actions history.

**Validation and limits.** These were AI-discovered findings and AI-generated
regressions, not an independent human security assessment. Coverage-guided
fuzzing, privilege dropping, repository administration, and release approval
remained deferred.

### 2026-10-03 — DHCP reply association and evidence correction

**Commit:** `5c2142c2ccea61c8cbec34e468cc99acd3a7d08f`
**AI system:** OpenAI Codex

**Human direction.** Enforce the approved rule: **a packet may affect the probe
result only after sufficient evidence associates it with that probe.**

**Work performed by AI.** Codex added a bounds-checked pre-parse association
helper and changed the Linux receive loop to require source UDP port 67, a
matching transaction ID, and matching client hardware address before full
parsing. Unrelated short or malformed traffic is ignored; malformed traffic
that is positively associated remains a warning/error evidence source. It also
added `DhcpV4TransportPeer` to the normalized model and output.

**Material choices.** The actually observed UDP source address/port is direct
transport evidence and is kept separate from DHCP Option 54, which is an
unauthenticated peer-advertised Server Identifier. Deduplication includes the
transport peer, and the multiple-server warning considers both observed peers
and advertised identifiers. Neither identity is treated as authorization.

**Validation and limits.** Protocol unit tests exercise association states;
namespace scenarios cover source-port, XID, `chaddr`, unrelated malformed,
associated malformed, and a valid Offer following unrelated traffic. Later
candidate CI and privileged integration passed. This does not authenticate a
DHCP server or make advertised configuration trustworthy.

### 2026-10-03 — timestamps and terminal hardening

**Commit:** `fa6b18f762d34825da2c5ad484bc657dcdc784a3`
**AI system:** OpenAI Codex

**Human direction.** Make retained inventory output time-bounded and make
human-facing terminal output safe without corrupting machine-readable values.

**Work performed by AI.** Codex added a high-level interface snapshot with
session start/completion timestamps and the existing versioned envelope,
updated CLI JSON to use it, and added deterministic serialization tests. It
centralized human-output escaping for control, escape, and invisible
bidirectional/formatting characters.

**Material choices.** Ordinary Unicode remains readable. Potentially dangerous
characters are rendered as visible escapes in terminal-oriented output and
diagnostics, while JSON preserves the underlying string value and relies on
JSON escaping. This keeps safety presentation separate from evidence storage.

**Validation and limits.** Unit tests pin timestamp fields and terminal escape
behavior. The change protects this CLI's rendering paths; it is not a general
terminal emulator or an assertion that arbitrary downstream JSON renderers are
safe.

### 2026-10-03 — security, MUSL, and release gating

**Commit:** `e3ed30b6471b3236101688f6446094181ebe0a8e`
**AI system:** OpenAI Codex

**Human direction.** Make dependency policy and MUSL compatibility mandatory
release-readiness evidence and prevent candidate metadata from claiming a
release prematurely.

**Work performed by AI.** Codex added a shared security script that prints the
pinned tool versions and runs `cargo audit --deny warnings` plus `cargo deny`
advisory/license/ban/source checks. It wired the gate into main/scheduled/manual
security runs and release readiness, added all-workspace/all-target MUSL
compilation, and added tag/candidate metadata, package-license, version,
artifact, and checksum checks.

**Material choices.** The fast PR security path continues to run committed
secret scanning while the expensive dependency-security job is skipped. Main,
scheduled/manual security runs and release readiness require it. Stable Rust is
used for the security tools; product compilation retains the declared Rust
toolchain. The release workflow validates but does not publish crates or create
a release.

**Validation and limits.** The inspected PR run reports the dependency-security
job as skipped, and no Release Check run exists in the inspected GitHub Actions
history. Therefore the workflow definitions establish required future gates,
not current evidence that `cargo audit`, `cargo deny`, MUSL compilation, or
artifact checksum verification ran on this exact candidate.

### 2026-10-03 — candidate contract and public API reconciliation

**Commit:** `e6931dd51d16b2d320b124ff81a8489f8cf792d4`
**AI system:** OpenAI Codex

**Human direction.** Reconcile documentation with the approved 0.1.0 package,
publication, evidence, privilege, security, testing, and release boundaries.

**Work performed by AI.** Codex revised the changelog and project/crate
documentation to describe an Unreleased candidate, association-before-parse,
timestamps, transport evidence, capability caveats, actual test coverage, and
the distinction between syntactic validation and semantic trust.

**Material choices.** The documented supported boundary is:
`l2linkscope-core` for portable models, `l2linkscope-protocols` for pure
protocol work, `l2linkscope-linux` for the supported high-level Linux API, and
`l2linkscope` for the binary CLI. There is no facade library. All four packages
use `publish = false`; crates.io is deferred. Interim Rust consumers are told
to pin a reviewed Git revision or, after a human creates it, a tag.

**Validation and limits.** This was documentation reconciliation, not API
stability approval or publication authorization. Public APIs and JSON remain
experimental during `0.x`.

### 2026-10-03 — address-free DHCP fixture

**Commit:** `b7ca2f4d928a9ec1d7ec1170a4d06b7c65d56b1d`
**AI system:** OpenAI Codex

**Human direction.** Strengthen proof that an observed transport source and an
advertised server identity are different evidence.

**Work performed by AI.** Codex removed the fixture server's configured IPv4
address and added assertions that the test link stays address-free and that the
observed UDP peer differs from Option 54. The fixture therefore observes
`0.0.0.0` as the datagram source while the packet advertises a deterministic
server identifier.

**Material choices.** The test no longer accidentally made the two identities
equal, so it would fail if acquisition again copied the advertised identifier
into the observed-source field.

**Validation and limits.** CI, privileged namespace integration, and the
committed-secret workflow completed successfully for this SHA. The topology is
still a deterministic fixture, not interoperability testing against arbitrary
DHCP implementations.

### 2026-10-03 — authenticated, read-only repository-settings inspection

**Commit:** `a2ded29ef670791be5088390daabfe01ee53b034`
**AI system:** OpenAI Codex

**Human direction.** Inspect GitHub settings without changing administration,
then correct the candidate documentation to distinguish observed state from
recommendation.

**Work performed by AI.** Codex used authenticated, read-only GitHub access and
documented that secret scanning and push protection were enabled. It also
recorded that `main` had no branch protection/ruleset, Dependabot alerts and
security updates were disabled, private vulnerability reporting was disabled,
and non-provider-pattern secret scanning and validity checks were disabled.

**Material choices.** The document separately recommends human decisions about
branch protection, required checks, dependency alerts, private reporting,
extra scanning modes, workflow permissions, and tag protection.

**Validation and limits.** No GitHub administrative setting was changed. The
observations are a dated snapshot (2026-10-03), not a continuing guarantee.
The AI committed and pushed this documentation update to PR 2. Candidate CI,
privileged namespace integration, and committed-secret scanning completed
successfully; the dependency-security PR job was skipped by design.

## 5. Current release-candidate evidence

The implementation candidate covered by this backfill is
`a2ded29ef670791be5088390daabfe01ee53b034`. The commit that introduces this
record changes documentation and policy only; it does not change product
behavior.

### Source and repository validation

The repository defines ordinary checks for formatting, locked metadata,
workspace/all-target compilation, Clippy with warnings denied, all-feature
tests, rustdoc with warnings denied, release CLI build, package metadata, and
package contents. At `a2ded29`, GitHub reports the `CI / workspace` job
successful. That is evidence for those scripted checks on GitHub's Ubuntu
runner, not human code review.

The parser/model suite is unprivileged and includes malformed-input,
serialization, evidence, arguments, and exit-code coverage. A fuzzable parser
entry point exists, but no continuous or coverage-guided fuzz campaign is
recorded.

### Privileged Linux integration

At `a2ded29`, GitHub reports `Privileged Linux Integration /
network-namespace-dhcp` successful. The suite uses isolated namespaces/veth,
records outgoing DHCP types, exercises the documented Offer and negative
scenarios, and compares client link/address/route state before and after. This
supports the tested Linux fixture invariants; it is not proof for every kernel,
distribution, network manager, or DHCP server.

### Security and release workflow evidence

At `a2ded29`, GitHub reports `Security / committed-secrets` successful and
`Security / dependency-security` skipped under the PR trigger. No Release Check
run was found in the inspected Actions history. Consequently, the candidate
has workflow definitions requiring pinned `cargo-audit`, `cargo-deny`, MUSL
compilation, metadata checks, package/license inspection, and artifact checksum
verification, but this record does **not** claim those gates ran successfully
on this exact SHA.

### Remaining procedural gates

There is no evidence in the inspected repository state that 0.1.0 has been
tagged or released. Before release, a human maintainer still must review the
changes and risk boundary, run or inspect the release-readiness and dependency
security results on the chosen commit, decide on repository protections,
confirm release metadata, intentionally create the tag/release, and decide
whether any artifact is suitable for distribution. Crates.io publication,
privilege dropping, a fuzzing campaign, additional discovery protocols, and
downstream adapter work remain deferred.

## 6. Maintenance of this record

Material future AI-assisted code, architecture, test, security, CI/CD, release,
or documentation work must update this file in the same workstream. Entries
should identify human direction separately from AI-selected implementation
details, record meaningful validation and limitations, and preserve both
AI-discovered defects and their corrections. Minor mechanical changes may be
grouped with their related work. Raw chain-of-thought, credentials, private
customer data, inappropriate infrastructure detail, and sensitive prompt
content do not belong here.

This backfill itself was prepared by OpenAI Codex on 2026-10-04 at Josh
Gitlin's direction. Codex inspected all 17 commits and their diffs, relevant
tests and documentation, issue 1, PR 2, GitHub Actions results, and read-only
repository settings evidence. It added this record, strengthened `AGENTS.md`,
and linked the record from the README. This work is a proposed provenance
correction and does not constitute human review or release approval.
