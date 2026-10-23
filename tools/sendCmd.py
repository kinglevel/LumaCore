#!/usr/bin/env python3
"""
Send a command to a running LumaCore instance over UDP.

Examples:
  python3 tools/sendCmd.py --target 127.0.0.1 --user Admin --cmd "list output"
  python3 tools/sendCmd.py --target 192.168.1.50 --user Debug --cmd "fixture 1 thru 2 at full"
"""

from __future__ import annotations

import argparse
import json
import socket
import sys

REMOTE_CMD_PORT = 47653


def main() -> int:
    parser = argparse.ArgumentParser(description="Send a UDP command to a running LumaCore instance")
    parser.add_argument("--target", required=True, help="Target host or IP running LumaCore")
    parser.add_argument("--user", required=True, help="LumaCore user profile that should execute the command")
    parser.add_argument("--cmd", required=True, help="Command string to execute")
    parser.add_argument(
        "--port",
        type=int,
        default=REMOTE_CMD_PORT,
        help=f"UDP control port. Default: {REMOTE_CMD_PORT}",
    )
    parser.add_argument(
        "--timeout",
        type=float,
        default=2.0,
        help="Seconds to wait for a reply. Default: 2.0",
    )
    args = parser.parse_args()

    payload = json.dumps(
        {
            "kind": "command",
            "user": args.user,
            "cmd": args.cmd,
        }
    ).encode("utf-8")

    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sock:
        sock.settimeout(args.timeout)
        sock.sendto(payload, (args.target, args.port))
        try:
            data, _ = sock.recvfrom(65535)
        except socket.timeout:
            print("No reply from LumaCore before timeout.", file=sys.stderr)
            return 1

    try:
        reply = json.loads(data.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        print(f"Invalid reply from LumaCore: {exc}", file=sys.stderr)
        return 1

    message = reply.get("message", "")
    if message:
        print(message)

    return 0 if reply.get("ok") else 1


if __name__ == "__main__":
    raise SystemExit(main())
