# CLI Exit Codes

The initial CLI assigns stable categories to process exit codes. Human-readable
results go to standard output. Argument errors and runtime diagnostics go to
standard error. JSON mode does not change the category.

| Code | Category | Meaning |
| ---: | --- | --- |
| 0 | success | The command completed; for a probe, at least one valid Offer was observed. |
| 1 | no offers | The bounded probe completed with no valid Offer. No configuration changed. |
| 2 | invalid arguments | CLI syntax or a bounded option such as timeout was invalid. |
| 3 | interface not found | The requested interface did not exist when resolved. |
| 4 | unsupported interface | The interface exists but is not suitable for this Ethernet DHCPv4 probe. |
| 5 | insufficient privileges | The packet operation lacked the required Linux privilege. |
| 6 | transport failure | Binding, sending, receiving, or another socket operation failed. |
| 7 | malformed-only responses | Responses arrived, but none was a valid matching Offer. |
| 70 | internal error | An unexpected internal failure prevented a trustworthy result. |

No-offer is deliberately distinct from transport failure. Scripts can treat
code 1 as a successfully completed negative observation and code 7 as evidence
that hostile or malformed input was handled safely.

These numeric values are part of the 0.1 CLI contract. New categories should
use new codes rather than changing the meaning of an existing one.
