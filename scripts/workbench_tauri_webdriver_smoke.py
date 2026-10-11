#!/usr/bin/env python3
"""Small dependency-free W3C WebDriver client for the real Linux Tauri smoke.

The outer shell script owns process lifecycle and fixture creation. This file
only drives the native WebKit document and captures protocol/DOM evidence.
"""

from __future__ import annotations

import argparse
import base64
import http.client
import json
import re
import socket
import sys
import time
from pathlib import Path
from urllib.parse import urlparse


def request(base: str, method: str, path: str, body: object | None = None) -> object:
    parsed = urlparse(base)
    connection = http.client.HTTPConnection(parsed.hostname, parsed.port, timeout=20)
    payload = None if body is None else json.dumps(body).encode()
    headers = {"Content-Type": "application/json"} if payload else {}
    connection.request(method, path, payload, headers)
    response = connection.getresponse()
    raw = response.read().decode()
    if response.status >= 400:
        raise RuntimeError(f"WebDriver {method} {path} returned {response.status}: {raw}")
    decoded = json.loads(raw) if raw else {}
    return decoded.get("value", decoded)


def sfwp(socket_path: Path, request_body: dict[str, object]) -> object:
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
        client.settimeout(10)
        client.connect(str(socket_path))
        client.sendall((json.dumps(request_body) + "\n").encode())
        response = b""
        while not response.endswith(b"\n"):
            part = client.recv(65536)
            if not part:
                raise RuntimeError("SFWP socket closed before a response")
            response += part
    return json.loads(response)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--driver", required=True)
    parser.add_argument("--application", required=True)
    parser.add_argument("--socket", required=True)
    parser.add_argument("--artifacts", required=True)
    parser.add_argument("--filter", default="")
    parser.add_argument("--fresh-cell", action="store_true")
    args = parser.parse_args()
    artifacts = Path(args.artifacts)
    socket_path = Path(args.socket)

    selected = args.filter.strip()
    known = (
        ("initialization",)
        if args.fresh_cell
        else ("hello", "identity", "reconnect", "request recovery")
    )
    try:
        selected_scenarios = [name for name in known if not selected or re.search(selected, name, re.I)]
    except re.error as error:
        raise RuntimeError(f"invalid native smoke filter {args.filter!r}: {error}") from error
    if not selected_scenarios:
        raise RuntimeError(f"filter {args.filter!r} selects no native smoke scenario")

    session = request(
        args.driver,
        "POST",
        "/session",
        {
            "capabilities": {
                "alwaysMatch": {
                    "browserName": "wry",
                    "tauri:options": {"application": args.application},
                }
            }
        },
    )
    session_id = session["sessionId"]
    try:
        if not args.fresh_cell:
            deadline = time.monotonic() + 20
            while not socket_path.exists() and time.monotonic() < deadline:
                time.sleep(0.1)
            if not socket_path.exists():
                raise RuntimeError("compiled Workbench did not publish its real sidecar socket")

        title = request(args.driver, "GET", f"/session/{session_id}/title")
        deadline = time.monotonic() + 20
        source = ""
        while time.monotonic() < deadline:
            source = request(args.driver, "GET", f"/session/{session_id}/source")
            if "Readiness Overview" in str(source):
                break
            time.sleep(0.2)
        diagnostics = request(
            args.driver,
            "POST",
            f"/session/{session_id}/execute/sync",
            {
                "script": """
                    return {
                      ready_state: document.readyState,
                      root_html: document.querySelector('#root')?.innerHTML ?? null,
                      scripts: Array.from(document.scripts).map((script) => ({
                        src: script.src,
                        type: script.type,
                        ready_state: script.readyState ?? null,
                      })),
                    };
                """,
                "args": [],
            },
        )
        screenshot = request(args.driver, "GET", f"/session/{session_id}/screenshot")
        (artifacts / "native-title.json").write_text(json.dumps(title, indent=2) + "\n")
        (artifacts / "native-page-source.html").write_text(str(source))
        (artifacts / "native-dom-diagnostics.json").write_text(
            json.dumps(diagnostics, indent=2) + "\n"
        )
        (artifacts / "native-window.png").write_bytes(base64.b64decode(screenshot))

        if "Readiness Overview" not in str(source):
            raise RuntimeError(
                "native renderer did not expose the readiness document before the 20-second deadline"
            )

        if "initialization" in selected_scenarios:
            if socket_path.exists():
                raise RuntimeError(
                    "a fresh cell published a sidecar socket before the operator confirmed initialization"
                )
            clicked = request(
                args.driver,
                "POST",
                f"/session/{session_id}/execute/sync",
                {
                    "script": """
                        const button = Array.from(document.querySelectorAll('button')).find(
                          (candidate) => candidate.textContent?.trim() === 'Initialize this cell',
                        );
                        if (!button) return { clicked: false, text: document.body.innerText };
                        button.click();
                        return { clicked: true };
                    """,
                    "args": [],
                },
            )
            if not clicked.get("clicked"):
                raise RuntimeError(
                    "fresh-cell readiness did not expose an explicit Initialize this cell action: "
                    f"{clicked.get('text', '')}"
                )

            deadline = time.monotonic() + 20
            while not socket_path.exists() and time.monotonic() < deadline:
                time.sleep(0.1)
            if not socket_path.exists():
                raise RuntimeError(
                    "confirming fresh-cell initialization did not publish the real sidecar socket"
                )

            hello = sfwp(socket_path, {"verb": "system_hello", "protocol_version": "1"})
            (artifacts / "sfwp-initialization.json").write_text(
                json.dumps(hello, indent=2) + "\n"
            )
            if hello.get("server_protocol_version") != "1":
                raise RuntimeError(f"initialized cell did not serve SFWP: {hello}")

        if "hello" in selected_scenarios:
            hello = sfwp(socket_path, {"verb": "system_hello", "protocol_version": "1"})
            (artifacts / "sfwp-hello.json").write_text(json.dumps(hello, indent=2) + "\n")
            if hello.get("server_protocol_version") != "1":
                raise RuntimeError(f"unexpected SFWP hello: {hello}")
            if "identity.get" not in hello.get("implemented_methods", []):
                raise RuntimeError(f"real sidecar did not advertise identity.get: {hello}")

        if "identity" in selected_scenarios:
            identity = sfwp(socket_path, {"verb": "identity_get"})
            (artifacts / "sfwp-identity.json").write_text(json.dumps(identity, indent=2) + "\n")
            if identity.get("error_class"):
                raise RuntimeError(f"real identity lookup failed: {identity}")

        if "reconnect" in selected_scenarios:
            first = sfwp(socket_path, {"verb": "system_hello", "protocol_version": "1"})
            second = sfwp(socket_path, {"verb": "system_hello", "protocol_version": "1"})
            (artifacts / "sfwp-reconnect.json").write_text(
                json.dumps({"first": first, "second": second}, indent=2) + "\n"
            )
            if first.get("server_protocol_version") != "1" or second != first:
                raise RuntimeError(f"real sidecar did not recover a fresh socket call: {first}, {second}")

        if "request recovery" in selected_scenarios:
            recovery = sfwp(socket_path, {"verb": "request_get_status", "request_id": "never-issued"})
            (artifacts / "sfwp-request-recovery.json").write_text(json.dumps(recovery, indent=2) + "\n")
            if recovery.get("status") != "unknown":
                raise RuntimeError(f"request-status recovery contract changed: {recovery}")
    finally:
        try:
            request(args.driver, "DELETE", f"/session/{session_id}")
        except Exception as error:  # Cleanup must not mask the actual assertion.
            print(f"warning: could not delete native session: {error}", file=sys.stderr)


if __name__ == "__main__":
    main()
