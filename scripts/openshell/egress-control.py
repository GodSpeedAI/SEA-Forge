#!/usr/bin/env python3
"""Host-side arrival listener for the OpenShell live egress proof.

live-proof.sh starts this before the live tests. The guest's allowed variant
connects to it (the projection's `cell_live_egress_allow` endpoint names the
same host and port), and the denied variant must not reach it. One line of
JSON per accepted connection is appended to the log:

    {"ts": "...", "peer": "ip:port", "first_bytes": "..."}

The live tests read the log directly (they run on the same host) and match the
token the guest sent as its first bytes, so an arrival is attributable to one
run and a non-arrival is evidence, not an assumption.

Usage:
    egress-control.py serve --port PORT --log PATH --ready PATH --pidfile PATH

No third-party dependency is used or needed.
"""

import argparse
import datetime
import json
import os
import sys


def now_iso():
    return datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="milliseconds")


def serve(args):
    import socket

    log = open(args.log, "a", encoding="utf-8")
    ready = args.ready
    with open(args.pidfile, "w", encoding="utf-8") as handle:
        handle.write(f"{os.getpid()}\n")

    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    listener.bind(("0.0.0.0", args.port))
    listener.listen(16)
    # Signal readiness only once the socket is actually listening.
    if ready:
        with open(ready, "w", encoding="utf-8") as handle:
            handle.write(f"0.0.0.0:{args.port} listening\n")

    while True:
        try:
            conn, addr = listener.accept()
        except OSError:
            break
        try:
            conn.settimeout(10)
            try:
                first = conn.recv(256)
            except OSError:
                first = b""
            record = {
                "ts": now_iso(),
                "peer": f"{addr[0]}:{addr[1]}",
                "first_bytes": first.decode("utf-8", errors="backslashreplace").strip(),
            }
            log.write(json.dumps(record, sort_keys=True) + "\n")
            log.flush()
            try:
                conn.sendall(b"logged\n")
            except OSError:
                pass
        finally:
            conn.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    runner = sub.add_parser("serve", help="accept connections and log arrivals")
    runner.add_argument("--port", type=int, required=True)
    runner.add_argument("--log", required=True, help="arrival log, appended to")
    runner.add_argument("--ready", help="file to write the bound address to once listening")
    runner.add_argument("--pidfile", help="file to write this process's pid to")
    runner.set_defaults(func=serve)

    args = parser.parse_args()
    try:
        args.func(args)
    except KeyboardInterrupt:
        sys.exit(0)
    except OSError as error:
        print(f"egress-control: {error}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
