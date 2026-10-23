#!/usr/bin/env python3
"""
Simple sACN monitor for LumaCore development.

Listens for E1.31 (sACN) multicast traffic on one or more universes and prints
basic packet details plus a compact view of active DMX slots.

Examples:
  python3 tools/sacn_monitor.py
  python3 tools/sacn_monitor.py --universe 1 --universe 2
  python3 tools/sacn_monitor.py --universe 1 --show-all
  python3 tools/sacn_monitor.py --bind 0.0.0.0 --timeout 0.5
"""

from __future__ import annotations

import argparse
import socket
import struct
import sys
import time

SACN_PORT = 5568
ACN_PID = b"ASC-E1.17\x00\x00\x00"


def multicast_addr_for_universe(universe: int) -> str:
    if not 1 <= universe <= 63999:
        raise ValueError(f"Universe must be 1-63999, got {universe}")
    return f"239.255.{(universe >> 8) & 0xFF}.{universe & 0xFF}"


def parse_source_name(packet: bytes) -> str:
    # Framing layer source name starts at byte 44 and is 64 bytes max.
    raw = packet[44:108]
    return raw.split(b"\x00", 1)[0].decode("utf-8", errors="replace")


def parse_cid(packet: bytes) -> str:
    cid = packet[22:38]
    hexed = cid.hex()
    return (
        f"{hexed[0:8]}-{hexed[8:12]}-{hexed[12:16]}-"
        f"{hexed[16:20]}-{hexed[20:32]}"
    )


def parse_packet(packet: bytes) -> dict[str, object] | None:
    if len(packet) < 126:
        return None
    if packet[4:16] != ACN_PID:
        return None

    universe = struct.unpack("!H", packet[113:115])[0]
    priority = packet[108]
    sequence = packet[111]
    options = packet[112]
    property_value_count = struct.unpack("!H", packet[123:125])[0]
    if property_value_count < 1:
        return None

    # DMP property values include start code as slot 0.
    dmx_with_start = packet[125 : 125 + property_value_count]
    if len(dmx_with_start) != property_value_count:
        return None

    start_code = dmx_with_start[0]
    dmx = dmx_with_start[1:]

    return {
        "universe": universe,
        "priority": priority,
        "sequence": sequence,
        "options": options,
        "start_code": start_code,
        "source_name": parse_source_name(packet),
        "cid": parse_cid(packet),
        "dmx": dmx,
    }


def summarize_active_slots(dmx: bytes, show_all: bool) -> str:
    if show_all:
        return " ".join(f"{i + 1}:{value}" for i, value in enumerate(dmx))

    active = [(i + 1, value) for i, value in enumerate(dmx) if value != 0]
    if not active:
        return "no active slots"

    preview = " ".join(f"{slot}:{value}" for slot, value in active[:16])
    if len(active) > 16:
        preview += f" ... (+{len(active) - 16} more)"
    return preview


def build_socket(bind_addr: str, universes: list[int]) -> socket.socket:
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    if hasattr(socket, "SO_REUSEPORT"):
        try:
            sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEPORT, 1)
        except OSError:
            pass
    try:
        sock.bind((bind_addr, SACN_PORT))
    except OSError as exc:
        raise SystemExit(f"Could not bind UDP {bind_addr}:{SACN_PORT}: {exc}") from exc

    for universe in universes:
        mcast = socket.inet_aton(multicast_addr_for_universe(universe))
        iface = socket.inet_aton("0.0.0.0")
        sock.setsockopt(socket.IPPROTO_IP, socket.IP_ADD_MEMBERSHIP, mcast + iface)

    return sock


def main() -> int:
    parser = argparse.ArgumentParser(description="Simple sACN multicast monitor")
    parser.add_argument(
        "--universe",
        type=int,
        action="append",
        default=[1],
        help="Universe to join. Repeat for multiple universes. Default: 1",
    )
    parser.add_argument(
        "--bind",
        default="0.0.0.0",
        help="Local bind address. Default: 0.0.0.0",
    )
    parser.add_argument(
        "--timeout",
        type=float,
        default=1.0,
        help="Socket timeout in seconds. Default: 1.0",
    )
    parser.add_argument(
        "--show-all",
        action="store_true",
        help="Print all 512 DMX slots instead of only active ones",
    )
    args = parser.parse_args()

    universes = sorted(set(args.universe))
    sock = build_socket(args.bind, universes)
    sock.settimeout(args.timeout)

    joined = ", ".join(f"{u} ({multicast_addr_for_universe(u)})" for u in universes)
    print(f"Listening on UDP {args.bind}:{SACN_PORT}")
    print(f"Joined universes: {joined}")
    print("Press Ctrl+C to stop.")

    try:
        while True:
            try:
                packet, addr = sock.recvfrom(2048)
            except socket.timeout:
                continue

            parsed = parse_packet(packet)
            if not parsed:
                continue

            ts = time.strftime("%H:%M:%S")
            slots = summarize_active_slots(parsed["dmx"], args.show_all)
            print(
                f"[{ts}] {addr[0]} "
                f"U{parsed['universe']} "
                f"prio={parsed['priority']} "
                f"seq={parsed['sequence']} "
                f"start={parsed['start_code']} "
                f"source={parsed['source_name']} "
                f"cid={parsed['cid']}"
            )
            print(f"        {slots}")
            sys.stdout.flush()
    except KeyboardInterrupt:
        print("\nStopped.")
        return 0


if __name__ == "__main__":
    raise SystemExit(main())
