# JSON Output

`interfaces --json` and `probe dhcp4 <INTERFACE> --json` emit normalized data,
not Rust debug output or raw packet bytes. The top-level object includes
`schema_version` so consumers can reject an incompatible format deliberately.

## Stability

The JSON schema is experimental during the 0.x series. Field additions may be
compatible for tolerant readers; removals, renames, type changes, or semantic
changes require a new `schema_version`. Consumers should ignore unknown fields
and must not infer that an absent optional field was observed to be empty.

Version 0.1.0 uses schema version `0.1`.

## General rules

* field names are explicit `snake_case` strings;
* addresses are canonical strings rather than OS socket structures;
* durations are represented in explicit units rather than display prose;
* evidence classes are explicit values such as `advertised_by_peer`;
* optional protocol values use `null` or an absent optional field according to
  the documented Rust model; collections use arrays;
* warnings and errors are structured objects, never unstable debug strings;
* no file descriptor, raw socket handle, pointer, or raw packet is serialized.

## Interface inventory

The interface result includes the runtime identity, kernel name and index,
hardware address when available, administrative and operational state, MTU,
configured IPv4 and IPv6 prefixes, loopback status, and whether the interface
appears suitable for an Ethernet DHCPv4 probe.

The runtime identity is not guaranteed to persist across boots. Consumers that
store results should retain both the identity and the observation timestamp.

## DHCP probe result

The top-level probe result is a discovery snapshot containing session metadata,
zero or more observations, and structured warnings. Every DHCP Offer carries
`advertised_by_peer` evidence. Useful normalized fields include the offered
address, server identifier, subnet mask, routers, DNS servers, domain data,
lease/renewal/rebinding durations, interface MTU, and decoded classless routes
when present.

An implementation may include observation IDs and timestamps alongside those
fields. Consumers must not interpret offered values as installed local state.

## Examples

Interface inventory, abbreviated:

```json
{
  "schema_version": "0.1",
  "interfaces": [
    {
      "id": { "index": 2, "hardware_address": "00:11:22:33:44:55" },
      "name": "enp3s0",
      "mtu": 1500,
      "administrative_state": "up",
      "operational_state": "up",
      "carrier_state": "present",
      "addresses": [
        { "address": "192.0.2.25", "prefix_length": 24 }
      ],
      "is_loopback": false,
      "dhcp_v4_probe": { "supported": true }
    }
  ]
}
```

Probe snapshot, abbreviated:

```json
{
  "schema_version": "0.1",
  "session": {
    "id": "11111111-1111-4111-8111-111111111111",
    "interface_id": {
      "index": 2,
      "hardware_address": "00:11:22:33:44:55"
    },
    "method": "dhcp_v4_probe",
    "started_at": { "unix_milliseconds": 1785268800000 },
    "completed_at": { "unix_milliseconds": 1785268805000 }
  },
  "interfaces": [],
  "observations": [
    {
      "id": "22222222-2222-4222-8222-222222222222",
      "session_id": "11111111-1111-4111-8111-111111111111",
      "interface_id": {
        "index": 2,
        "hardware_address": "00:11:22:33:44:55"
      },
      "observed_at": { "unix_milliseconds": 1785268800100 },
      "source": "dhcp_v4_probe",
      "evidence_class": "advertised_by_peer",
      "kind": {
        "type": "dhcp_v4_offer",
        "details": {
          "transaction_id": 305419896,
          "client_hardware_address": "00:11:22:33:44:55",
          "offered_address": "192.0.2.117",
          "server_identifier": "192.0.2.1",
          "subnet_mask": "255.255.255.0",
          "routers": ["192.0.2.1"],
          "dns_servers": ["192.0.2.10", "192.0.2.11"],
          "domain_name": "fixture.example",
          "domain_search": [],
          "lease_time_seconds": 28800,
          "renewal_time_seconds": 14400,
          "rebinding_time_seconds": 25200,
          "interface_mtu": 1500,
          "classless_static_routes": []
        }
      }
    }
  ],
  "warnings": []
}
```

Examples omit some metadata for readability and are not a substitute for
checking `schema_version`.
