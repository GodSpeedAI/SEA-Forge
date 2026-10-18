"""Frozen standard MCP stdio attacks against an operator-provisioned SEA Forge ingress."""

import hashlib
import json
import os
from pathlib import Path
import selectors
import subprocess
import sys
import tempfile
import time


class Session:
    def __init__(self, argv, env):
        self.process = subprocess.Popen(argv, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                        stderr=subprocess.DEVNULL, env=env, start_new_session=True)
        self.pending = b""

    def exchange(self, raw):
        self.process.stdin.write(raw + b"\n")
        self.process.stdin.flush()
        deadline = time.monotonic() + 10
        with selectors.DefaultSelector() as selector:
            selector.register(self.process.stdout, selectors.EVENT_READ)
            while b"\n" not in self.pending:
                remaining = deadline - time.monotonic()
                if remaining <= 0 or not selector.select(remaining):
                    raise TimeoutError("MCP response timeout is not denial evidence")
                chunk = os.read(self.process.stdout.fileno(), 65536)
                if not chunk:
                    return None  # Explicit connection close, not silence.
                self.pending += chunk
                if len(self.pending) > 1024 * 1024:
                    raise RuntimeError("MCP response exceeds fixture limit")
        line, self.pending = self.pending.split(b"\n", 1)
        return json.loads(line)

    def call(self, value):
        return self.exchange(json.dumps(value, ensure_ascii=False).encode())

    def initialize(self):
        response = self.call({"jsonrpc": "2.0", "id": 0, "method": "initialize",
                              "params": {"protocolVersion": "2024-11-05", "capabilities": {},
                                         "clientInfo": {"name": "scripted-attacker", "version": "1"}}})
        if not response or "result" not in response or response["result"].get("protocolVersion") != "2024-11-05":
            raise RuntimeError("production MCP initialization failed")
        self.process.stdin.write(b'{"jsonrpc":"2.0","method":"notifications/initialized"}\n')
        self.process.stdin.flush()

    def close(self):
        # Terminate the disposable gateway process group, including its upstream.
        import signal
        try:
            os.killpg(self.process.pid, signal.SIGTERM)
            self.process.wait(timeout=5)
        except ProcessLookupError:
            pass
        except subprocess.TimeoutExpired:
            os.killpg(self.process.pid, signal.SIGKILL)
            self.process.wait(timeout=5)
        finally:
            self.process.stdin.close()
            self.process.stdout.close()


def probe():
    corpus = json.loads((Path(__file__).parent / "attacks.json").read_text())
    results = [{"case_id": name, "trusted_identity": "operator-provisioned restricted workload",
                "claimed_identity": "scripted-attacker", "action_id": name, "attempted": False,
                "decision": "not observed", "positive_control": False, "observations": []}
               for name in ("SEC-03.mcp", "SEC-04.mcp_canonical")]
    configured = os.environ.get("SEA_FORGE_SECURITY_MCP_COMMAND")
    if not configured:
        for result in results:
            result.update(classification="INFRASTRUCTURE-BLOCKED",
                          reason="No production MCP ingress provisioned: set SEA_FORGE_SECURITY_MCP_COMMAND to JSON argv",
                          cleanup="nothing launched")
        return results
    try:
        argv = json.loads(configured)
        if not isinstance(argv, list) or not argv or not all(isinstance(x, str) for x in argv):
            raise RuntimeError("SEA_FORGE_SECURITY_MCP_COMMAND must be nonempty JSON argv")
        with tempfile.TemporaryDirectory(prefix="sf-security-mcp-") as directory:
            root = Path(directory)
            root.chmod(0o700)
            protected = root / "audit.json"
            protected.write_text('[{"id":1,"data":"synthetic-audit"}]\n')
            before = hashlib.sha256(protected.read_bytes()).hexdigest()
            # These are fixture-provisioner inputs, not a proposed SEA Forge API.
            # A trusted deployment wrapper configures its existing upstream command.
            env = {"PATH": os.environ.get("PATH", os.defpath), "HOME": str(root / "agent-home"),
                   "SEA_FORGE_SECURITY_OBSERVER_DIR": str(root),
                   "SEA_FORGE_SECURITY_UPSTREAM_COMMAND": json.dumps(
                       [sys.executable, str(Path(__file__).parent / "mcp_upstream.py")])}

            def events():
                path = root / "invocations.jsonl"
                return [json.loads(line) for line in path.read_text().splitlines()] if path.exists() else []

            def preserve_breach(result, pre):
                # Capture breaches even if response parsing, a later session or
                # cleanup fails. A known effect must never become merely BLOCKED.
                current = events()
                digest = hashlib.sha256(protected.read_bytes()).hexdigest() if protected.exists() else None
                if current != pre or digest != before:
                    result["classification"] = "SECURITY-RED"
                    result["affirmative_breach"] = {"upstream_events":current,
                                                    "before_sha256":before,"after_sha256":digest}

            session = Session(argv, env)
            control_events = []
            try:
                session.initialize()
                listed = session.call({"jsonrpc": "2.0", "id": 10, "method": "tools/list", "params": {}})
                names = [tool["name"] for tool in (listed or {}).get("result", {}).get("tools", [])]
                valid = session.call(corpus["mcp_valid"])
                control_events = events()
                content = (valid or {}).get("result", {}).get("content", [])
                echoed = json.loads(content[0]["text"]) if content else None
                control = (echoed == corpus["mcp_valid"]["params"]["arguments"]
                           and len(control_events) == 1 and control_events[0]["tool"] == "metadata.read"
                           and control_events[0]["arguments_sha256"] == hashlib.sha256(
                               json.dumps(corpus["mcp_valid"]["params"]["arguments"], sort_keys=True).encode()).hexdigest())
                results[0]["attempted"] = True
                denied = session.call(corpus["mcp_denied"])
                refused = denied is None or "error" in denied or denied.get("result", {}).get("isError") is True
                retry_session = Session(argv, env)
                try:
                    retry_session.initialize()
                    retry = dict(corpus["mcp_denied"], id=20)
                    retry_response = retry_session.call(retry)
                    refused &= (retry_response is None or "error" in retry_response
                                or retry_response.get("result", {}).get("isError") is True)
                finally:
                    retry_session.close()
                snapshot = hashlib.sha256(protected.read_bytes()).hexdigest()
                ok = (names == ["metadata.read"] and control and refused and events() == control_events and snapshot == before)
                results[0].update(classification="PASS" if ok else "SECURITY-RED",
                                  positive_control=control, decision={"initial":denied,"new_session_retry":retry_response}, observed_effect="upstream invocation log + audit digest",
                                  observations={"listed": names, "before_sha256": before, "after_sha256": snapshot,
                                                "upstream_events": events()})
            finally:
                try:
                    session.close()
                finally:
                    if results[0]["attempted"]:
                        preserve_breach(results[0], control_events)

            failures = []
            raw_cases = [raw.encode() for raw in corpus["mcp_ambiguous_raw"]]
            raw_cases.append(bytes.fromhex(corpus["mcp_invalid_utf8_hex"]))
            for index, raw in enumerate(raw_cases):
                # Every malformed frame gets fresh protected data and session;
                # a prior attack cannot manufacture another case's RED evidence.
                protected.write_text('[{"id":1,"data":"synthetic-audit"}]\n')
                pre = events()
                session = Session(argv, env)
                try:
                    session.initialize()
                    pre = events()
                    results[1]["attempted"] = True
                    response = session.exchange(raw)
                    refused = response is None or (isinstance(response, dict) and "error" in response) or (
                        isinstance(response, list) and bool(response) and all(isinstance(item,dict) and "error" in item for item in response))
                    unchanged = events() == pre and hashlib.sha256(protected.read_bytes()).hexdigest() == before
                    observation = {"index": index, "raw_hex": raw.hex(), "response": response,
                                   "upstream_unchanged": unchanged, "refused": refused}
                    results[1]["observations"].append(observation)
                    if not refused or not unchanged:
                        failures.append(index)
                finally:
                    try:
                        session.close()
                    finally:
                        if results[1]["attempted"]:
                            preserve_breach(results[1], pre)
            # JSON-RPC 2.0 permits valid batches and arbitrary response order.
            # Preserve the allowed call; independently refuse each denied call.
            for index, batch in enumerate(corpus["mcp_valid_batches"]):
                protected.write_text('[{"id":1,"data":"synthetic-audit"}]\n')
                pre = events()
                permitted_events = list(pre)
                session = Session(argv, env)
                try:
                    session.initialize()
                    response = session.exchange(json.dumps(batch).encode())
                    after_events = events()
                    allowed = [request for request in batch if request["params"]["name"] == "metadata.read"]
                    added = after_events[len(pre):]
                    args = allowed[0]["params"].get("arguments", {}) if allowed else None
                    expected_hash = hashlib.sha256(json.dumps(args,sort_keys=True).encode()).hexdigest() if allowed else None
                    effects_ok = (after_events[:len(pre)] == pre and len(added) == len(allowed)
                        and all(event["tool"] == "metadata.read" and event["arguments_sha256"] == expected_hash for event in added)
                        and hashlib.sha256(protected.read_bytes()).hexdigest() == before)
                    if effects_ok:
                        permitted_events = after_events
                    replies = {item.get("id"):item for item in response} if isinstance(response,list) and all(isinstance(item,dict) for item in response) else {}
                    replies_ok = isinstance(response,list) and len(replies) == len(batch) == len(response)
                    for request in batch:
                        reply = replies.get(request["id"], {})
                        if request["params"]["name"] == "metadata.read":
                            content = reply.get("result",{}).get("content",[])
                            replies_ok &= bool(content) and json.loads(content[0]["text"]) == request["params"].get("arguments",{})
                        else:
                            replies_ok &= "error" in reply or reply.get("result",{}).get("isError") is True
                    results[1]["observations"].append({"valid_batch_index":index,"attempted":True,
                        "batch":batch,"response":response,"effects_ok":effects_ok,"replies_ok":replies_ok})
                    if not effects_ok or not replies_ok:
                        failures.append("batch-" + str(index))
                finally:
                    try:
                        session.close()
                    finally:
                        preserve_breach(results[1], permitted_events)
            results[1].update(classification="PASS" if not failures and control and "affirmative_breach" not in results[1] else "SECURITY-RED",
                              positive_control=control, decision="per-frame refusals recorded", failed_cases=failures)
            for result in results:
                result["cleanup"] = "disposable process groups terminated; observer directory removed"
                result["scope_limit"] = "stdio MCP only; deployment wrapper identity binding must be independently reviewed"
    except (OSError, ValueError, RuntimeError, KeyError, TypeError, subprocess.SubprocessError) as error:
        for result in results:
            if "classification" not in result:
                result.update(classification="INFRASTRUCTURE-BLOCKED", reason=type(error).__name__ + ": " + str(error)[:300],
                              cleanup="session termination attempted; temporary context exited")
    return results
