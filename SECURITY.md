# Security Policy

## Supported Versions

L2LinkScope is pre-1.0 and has not published a stable release. Security fixes
are expected to target the default branch and any actively maintained pre-1.0
release branches once they exist.

## Reporting Vulnerabilities

Use GitHub private vulnerability reporting if it is enabled for this repository.

If private vulnerability reporting is not available, contact the repository
owner privately through GitHub rather than opening a public issue with
exploitable details.

Please do not publicly disclose exploitable parser, packet-handling,
privilege-boundary, or unsafe-code issues before maintainers have had a chance
to coordinate a fix.

## Hostile Input Assumptions

L2LinkScope must assume that network input is hostile. Future parsers and packet
acquisition code should expect malformed packets, truncated frames, oversized
fields, duplicate options, contradictory advertisements, and intentionally
misleading peer-provided configuration.

Future privileged code must assume that privilege boundaries are security
sensitive and must fail closed when capabilities, sockets, or interface state do
not match expectations.
