#!/usr/bin/env python3
"""Deterministic DHCPv4 fixture for the privileged namespace test.

This helper intentionally uses only the Python standard library. It listens for
one DHCP Discover on UDP/67, records all DHCP message types received during the
test, and emits a selected deterministic response on UDP/68. It must run inside
an isolated network namespace as root.
"""

from __future__ import annotations

import argparse
import socket
import struct
import time
from pathlib import Path


MAGIC_COOKIE = b"\x63\x82\x53\x63"
SO_BINDTODEVICE = 25


def message_type(payload: bytes) -> int | None:
    """Return DHCP option 53, or None for a malformed/non-DHCP payload."""
    if len(payload) < 240 or payload[236:240] != MAGIC_COOKIE:
        return None
    offset = 240
    while offset < len(payload):
        code = payload[offset]
        offset += 1
        if code == 0:
            continue
        if code == 255:
            return None
        if offset >= len(payload):
            return None
        length = payload[offset]
        offset += 1
        if offset + length > len(payload):
            return None
        if code == 53 and length == 1:
            return payload[offset]
        offset += length
    return None


def option(code: int, value: bytes) -> bytes:
    return bytes((code, len(value))) + value


def offer(discover: bytes, address: str, server: str) -> bytes:
    """Build a deterministic BOOTP reply matching *discover*."""
    if len(discover) < 240:
        raise ValueError("Discover is shorter than the DHCP fixed header")

    header = bytearray(236)
    header[0:4] = bytes((2, discover[1], discover[2], 0))
    header[4:8] = discover[4:8]
    header[10:12] = b"\x80\x00"
    header[16:20] = socket.inet_aton(address)
    header[20:24] = socket.inet_aton(server)
    header[28:44] = discover[28:44]

    options = b"".join(
        (
            option(53, b"\x02"),
            option(54, socket.inet_aton(server)),
            option(1, socket.inet_aton("255.255.255.0")),
            option(3, socket.inet_aton(server)),
            option(
                6,
                socket.inet_aton("192.0.2.53")
                + socket.inet_aton("192.0.2.54"),
            ),
            option(15, b"fixture.example"),
            option(26, struct.pack("!H", 1500)),
            option(51, struct.pack("!I", 28800)),
            option(58, struct.pack("!I", 14400)),
            option(59, struct.pack("!I", 25200)),
            b"\xff",
        )
    )
    payload = bytes(header) + MAGIC_COOKIE + options
    return payload.ljust(300, b"\x00")


def malformed_offer(discover: bytes) -> bytes:
    payload = offer(discover, "192.0.2.117", "192.0.2.1")
    # Replace the valid option stream with an option whose declared value runs
    # beyond the datagram. This exercises length validation without ambiguity.
    return payload[:240] + bytes((53, 1, 2, 1, 250, 255))


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--interface", required=True)
    parser.add_argument(
        "--mode",
        required=True,
        choices=("one", "two", "none", "malformed", "wrong-xid", "wrong-chaddr", "duplicate"),
    )
    parser.add_argument("--log", required=True, type=Path)
    parser.add_argument("--ready", required=True, type=Path)
    parser.add_argument("--deadline", type=float, default=8.0)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_BROADCAST, 1)
    sock.setsockopt(
        socket.SOL_SOCKET,
        SO_BINDTODEVICE,
        args.interface.encode("ascii") + b"\x00",
    )
    sock.bind(("0.0.0.0", 67))
    sock.settimeout(0.2)
    args.log.write_text("", encoding="ascii")
    args.ready.touch()

    deadline = time.monotonic() + args.deadline
    responded = False
    quiet_deadline: float | None = None
    while time.monotonic() < deadline:
        try:
            payload, _peer = sock.recvfrom(4096)
        except TimeoutError:
            if quiet_deadline is not None and time.monotonic() >= quiet_deadline:
                break
            continue

        kind = message_type(payload)
        with args.log.open("a", encoding="ascii") as log_file:
            log_file.write("malformed\n" if kind is None else f"{kind}\n")

        if responded or kind != 1:
            continue
        responded = True
        quiet_deadline = time.monotonic() + 2.0

        responses: list[bytes]
        if args.mode == "one":
            responses = [offer(payload, "192.0.2.117", "192.0.2.1")]
        elif args.mode == "two":
            responses = [
                offer(payload, "192.0.2.117", "192.0.2.1"),
                offer(payload, "198.51.100.23", "198.51.100.1"),
            ]
        elif args.mode == "duplicate":
            repeated = offer(payload, "192.0.2.117", "192.0.2.1")
            responses = [repeated, repeated]
        elif args.mode == "malformed":
            responses = [malformed_offer(payload)]
        elif args.mode == "wrong-xid":
            wrong = bytearray(offer(payload, "192.0.2.117", "192.0.2.1"))
            wrong[4:8] = struct.pack("!I", struct.unpack("!I", wrong[4:8])[0] ^ 1)
            responses = [bytes(wrong)]
        elif args.mode == "wrong-chaddr":
            wrong = bytearray(offer(payload, "192.0.2.117", "192.0.2.1"))
            wrong[28] ^= 1
            responses = [bytes(wrong)]
        else:
            responses = []

        for response in responses:
            sock.sendto(response, ("255.255.255.255", 68))
            time.sleep(0.05)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
