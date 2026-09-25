#!/usr/bin/env python3
"""Independent critic driver for T02 (prereg confirmation_condition:
"An independent agent's negative tests pass on a fresh cell").

Boots REAL sea-forge-server processes on FRESH temp cells and drives verbatim
refusals + the positive dual-principal path over the real Unix socket.

Scenarios (mirroring tests/sfwp_delegated_identity.rs seeds, but executed
against a real server binary as a separate process, not an in-process run()):
  (a) on_behalf_of from a uid bound as operator      -> must be refused
  (b) gateway section omitted entirely               -> must be refused
  (c) on_behalf_of naming a bound-but-not-allowlisted actor -> must be refused
  (d) on_behalf_of claiming a role the target lacks  -> must be refused
  (e) positive: gateway speaks for A then B; B resolves A's approval;
      read delegation-audit + approval_resolution durable records.
Each refusal is hash-proven to leave ledgers/approvals byte-identical.
"""
import hashlib
import json
import os
import shutil
import signal
import socket
import subprocess
import sys
import tempfile
import time
from pathlib import Path

REPO = Path("/home/sprime01/projects/sea-rs")
SERVER_BIN = REPO / "target/debug/sea-forge-server"
UID = os.getuid()
OTHER_UID = 4243 if UID == 4242 else 4242
FIXTURES = REPO / "fixtures/cells/e2e"

GATEWAY_ACTOR = {"actor_id": "gateway", "role": "service"}


def log(msg):
    print(msg, flush=True)


class Client:
    def __init__(self, sock_path):
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.sock.settimeout(60)
        self.sock.connect(str(sock_path))
        self.buf = b""

    def call(self, **body):
        line = json.dumps(body) + "\n"
        self.sock.sendall(line.encode())
        while b"\n" not in self.buf:
            chunk = self.sock.recv(65536)
            if not chunk:
                raise RuntimeError("connection closed")
            self.buf += chunk
        out, self.buf = self.buf.split(b"\n", 1)
        return json.loads(out.decode())


class Server:
    def __init__(self, root, label):
        self.root = root
        self.label = label
        self.socket = root / f"{label}.sock"
        env = dict(os.environ)
        env["SEA_FORGE_ROOT"] = str(root)
        env["SEA_FORGE_SOCKET"] = str(self.socket)
        self.logfile = open(root / "server-stderr.log", "wb")
        self.proc = subprocess.Popen(
            [str(SERVER_BIN)], env=env, stdout=self.logfile, stderr=self.logfile
        )
        deadline = time.time() + 30
        while not self.socket.exists():
            if self.proc.poll() is not None:
                raise RuntimeError(
                    f"{label}: server exited {self.proc.returncode} before socket; "
                    f"log tail: {self.tail()}"
                )
            if time.time() > deadline:
                raise RuntimeError(f"{label}: socket never appeared")
            time.sleep(0.05)

    def tail(self):
        try:
            return (self.root / "server-stderr.log").read_text()[-2000:]
        except OSError:
            return "<unreadable>"

    def stop(self):
        self.proc.send_signal(signal.SIGTERM)
        try:
            self.proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            self.proc.kill()
            self.proc.wait()
        self.logfile.close()


def seed_gateway_cell(root):
    """Mirror of tests/sfwp_delegated_identity.rs::seed_gateway_cell."""
    (root / "server.yaml").write_text(
        f"""# critic T02 cell: gateway principal is uid {UID}; end users at uid {OTHER_UID}
identity:
  bindings:
    - uid: {UID}
      actor_id: gateway
      roles: ["service"]
    - uid: {OTHER_UID}
      actor_id: operator_a
      roles: ["operator"]
    - uid: {OTHER_UID}
      actor_id: operator_b
      roles: ["operator"]
    - uid: {OTHER_UID}
      actor_id: operator_c
      roles: ["operator"]
    - uid: {OTHER_UID}
      actor_id: security_officer
      roles: ["R-SO"]
gateway:
  uid: {UID}
  actor: gateway
  delegable_actors: [operator_a, operator_b, security_officer]
"""
    )


def seed_operator_bound_cell(root):
    """Mirror of seed_operator_bound_cell: THIS uid is a trusted direct operator."""
    (root / "server.yaml").write_text(
        f"""identity:
  bindings:
    - uid: {UID}
      actor_id: operator_local
      roles: ["operator"]
    - uid: {OTHER_UID}
      actor_id: gateway
      roles: ["service"]
    - uid: {OTHER_UID}
      actor_id: operator_b
      roles: ["operator"]
gateway:
  uid: {OTHER_UID}
  actor: gateway
  delegable_actors: [operator_b]
"""
    )


def seed_ungoverned_cell(root):
    """Mirror of seed_ungoverned_cell: no gateway: section at all."""
    (root / "server.yaml").write_text(
        f"""identity:
  bindings:
    - uid: {UID}
      actor_id: operator_local
      roles: ["operator"]
"""
    )


def install_e2e_fixtures(root):
    templates = root / "templates"
    templates.mkdir(exist_ok=True)
    for entry in (FIXTURES / "templates").iterdir():
        if entry.suffix == ".yaml":
            shutil.copy(entry, templates / entry.name)
    authority = root / "authority"
    authority.mkdir(exist_ok=True)
    shutil.copy(FIXTURES / "policy.yaml", authority / "active-policy.json")


def cell_digest(root):
    """sha256 over sorted (relpath + content) of ledgers/** and approvals.jsonl."""
    paths = []
    approvals = root / "approvals.jsonl"
    if approvals.exists():
        paths.append(approvals)
    ledgers = root / "ledgers"
    if ledgers.exists():
        paths.extend(p for p in ledgers.rglob("*") if p.is_file())
    paths.sort()
    h = hashlib.sha256()
    for p in paths:
        h.update(str(p.relative_to(root)).encode())
        h.update(p.read_bytes())
    return h.hexdigest()


def case_count(root):
    cases = root / "cases"
    return len(list(cases.iterdir())) if cases.exists() else 0


NONCE = [0]


def commit_request(actor, entity=None):
    NONCE[0] += 1
    return {
        "verb": "case_commit",
        "template_ref": "e2e-sentry-chain@0.1.0",
        "params": {
            "dataset_name": "orders-q3",
            "dataset_label": "t02critic",
            "max_rows": "25",
            "out_dir": "work",
        },
        "policy": "authority/active-policy.json",
        "entity": entity or actor,
        "process": "t02_critic",
        "request_id": f"critic-commit-{NONCE[0]}",
    }


def delegated(body, actor, role):
    body = dict(body)
    body["actor"] = dict(GATEWAY_ACTOR)
    body["on_behalf_of"] = {"actor_id": actor, "role": role}
    return body


def run_attack(server, client_factory, label, body, expect_class, expect_snippet):
    """Send one request, capture the refusal verbatim, prove zero writes."""
    before_digest = cell_digest(server.root)
    before_cases = case_count(server.root)
    audit_exists = (server.root / "ledgers/delegation-audit").exists()
    log(f"\n[attack {label}] before: digest={before_digest[:16]}… cases={before_cases} "
        f"delegation_audit_exists={audit_exists}")
    log(f"[attack {label}] request: {json.dumps(body)}")
    response = client_factory().call(**body)
    log(f"[attack {label}] RESPONSE (verbatim): {json.dumps(response)}")
    after_digest = cell_digest(server.root)
    after_cases = case_count(server.root)
    audit_after = (server.root / "ledgers/delegation-audit").exists()
    log(f"[attack {label}] after:  digest={after_digest[:16]}… cases={after_cases} "
        f"delegation_audit_exists={audit_after}")

    ok = True
    def check(cond, msg):
        nonlocal ok
        if not cond:
            ok = False
            log(f"[attack {label}] FAIL: {msg}")

    check(response.get("error_class") == expect_class,
          f"error_class {response.get('error_class')!r} != {expect_class!r}")
    check(response.get("no_side_effect") is True, "no_side_effect is not true")
    if expect_snippet:
        check(expect_snippet in str(response.get("error", "")),
              f"error message missing {expect_snippet!r}")
    check(after_digest == before_digest, "ledgers/approvals digest CHANGED (write on refusal!)")
    check(after_cases == before_cases, "case count CHANGED (write on refusal!)")
    check(not audit_after or not (audit_after and not audit_exists),
          "delegation-audit ledger appeared on a refused request")
    log(f"[attack {label}] verdict: {'PASS' if ok else 'FAIL'}")
    return ok, response


ESCALATE_RULES = (
    "  - name: escalate-command\n    verdict: escalate\n    actor_role: operator\n"
    "    operation_kind: execute_command\n    argv0: sea-forge\n"
)
ADD_ITEM_RULE = (
    "  - name: allow-propose\n    verdict: allow\n    actor_role: operator\n"
    "    operation_kind: discretionary_task_add\n"
)
APPROVAL_RESOLUTION_RULE = (
    "  - name: allow-approval-resolution\n    verdict: allow\n    actor_role: operator\n"
    "    operation_kind: approval_resolution\n"
)


def sandbox_plan_item(item_id, name, required, manual=False):
    return {
        "plan_item_id": item_id,
        "name": name,
        "operations": [{
            "kind": "execute_command",
            "argv": [str(sea_forge_link[0])],
            "cwd": ".",
        }],
        "entry_criteria": [],
        "exit_criteria": [],
        "settlement_criteria": {"required_artifacts": [], "required_declarations": []},
        "item_kind": "sandboxed_task",
        "markers": {"required": required, **({"manual_activation": True} if manual else {})},
        "max_instances": 1,
        "depends_on": [],
    }


sea_forge_link = []


def positive_sod(server):
    """(e) gateway speaks for A then B; B resolves A's approval. Both principals durable."""
    root = server.root
    # ONE policy file serves both halves (the server normalizes the episode
    # policy into authority/active-policy.json, so the resolution rule must
    # live in the same file the episode policy names — as the builder's test
    # does).
    policy_name = "authority/active-policy.json"
    (root / "authority").mkdir(exist_ok=True)
    (root / policy_name).write_text(
        f'version: "0.1"\nrules:\n{ESCALATE_RULES}{ADD_ITEM_RULE}{APPROVAL_RESOLUTION_RULE}'
    )
    sea_forge_link.append(root / "sea-forge")
    if sea_forge_link[0].exists() or sea_forge_link[0].is_symlink():
        sea_forge_link[0].unlink()
    sea_forge_link[0].symlink_to(SERVER_BIN.resolve())

    plan = {
        "version": "0.2",
        "plan_id": "plan_t02_critic",
        "case_id": "case_placeholder",
        "run_id": "run_placeholder",
        "intent_id": "int_t02_critic",
        "items": [sandbox_plan_item("task", "sandboxed", True, manual=True)],
        "template_ref": None,
        "job_contract_ref": None,
    }
    (root / "plan-critic.json").write_text(json.dumps(plan))

    client_a = Client(server.socket)
    client_b = Client(server.socket)
    ok = True

    submitted = client_a.call(**delegated({
        "verb": "submit", "plan": "plan-critic.json", "policy": policy_name,
        "entity": "operator_a", "process": "t02_critic", "timeout": 60,
        "request_id": "critic-pos-submit",
    }, "operator_a", "operator"))
    log(f"\n[positive] A submits via gateway: {json.dumps(submitted)[:300]}")
    if "error" in submitted:
        log(f"[positive] FAIL: submit refused: {submitted}")
        return False
    case_id = submitted["case_id"]

    proposed = client_a.call(**delegated({
        "verb": "case_add_item", "case_id": case_id,
        "item": sandbox_plan_item("disc", "discretionary", False),
        "policy": policy_name, "request_id": "critic-pos-add",
    }, "operator_a", "operator"))
    log(f"[positive] A adds discretionary item via gateway: {json.dumps(proposed)[:300]}")
    if proposed.get("proposed_by") != "operator_a":
        log(f"[positive] FAIL: proposed_by != operator_a: {proposed}")
        ok = False

    executed = client_a.call(**delegated({
        "verb": "item_execute", "case_id": case_id, "item_id": "disc",
        "policy": policy_name, "timeout": 60, "request_id": "critic-pos-exec",
    }, "operator_a", "operator"))
    log(f"[positive] A executes item (escalates): {json.dumps(executed)[:400]}")
    status = (executed.get("episodes") or [{}])[0].get("settlement_status")
    if status != "escalated":
        log(f"[positive] FAIL: settlement_status {status!r} != 'escalated'")
        ok = False

    listed = client_b.call(verb="approval_list", case_id=case_id)
    approvals = listed.get("approvals") or []
    if not approvals:
        log(f"[positive] FAIL: no pending approval: {listed}")
        return False
    approval_id = approvals[0]["approval_id"]
    log(f"[positive] pending approval: {approval_id}")

    resolved = client_b.call(**delegated({
        "verb": "approval_decide", "case_id": case_id, "approval_id": approval_id,
        "decision": "approve", "note": "distinct end user, same gateway (critic)",
        "request_id": "critic-pos-decide",
    }, "operator_b", "operator"))
    log(f"[positive] B resolves A's approval via gateway: {json.dumps(resolved)[:400]}")
    if resolved.get("ok") is not True:
        log("[positive] FAIL: approval_decide not ok")
        ok = False
    if "resolved_by=operator_b" not in str(resolved.get("output", "")):
        log("[positive] FAIL: output does not name resolved_by=operator_b")
        ok = False

    # --- durable reads ---
    audit_path = root / "ledgers/delegation-audit/entries.jsonl"
    audit = [json.loads(l) for l in audit_path.read_text().splitlines() if l.strip()]
    deciding = [r for r in audit if r["payload"].get("verb") == "approval_decide"]
    log(f"[positive] delegation-audit records ({len(audit)}):")
    for r in audit:
        log(f"           {json.dumps(r['payload'])}")
    if not deciding:
        log("[positive] FAIL: no approval_decide audit record")
        ok = False
    else:
        p = deciding[0]["payload"]
        if not (p["gateway_actor_id"] == "gateway"
                and p["gateway_uid"] == UID
                and p["effective_actor_id"] == "operator_b"
                and p["effective_role"] == "operator"):
            log(f"[positive] FAIL: audit payload wrong: {p}")
            ok = False
    if not any(r["payload"].get("effective_actor_id") == "operator_a" for r in audit):
        log("[positive] FAIL: no audit record attributing operator_a")
        ok = False

    case_ledger = root / f"ledgers/case-{case_id}/entries.jsonl"
    entries = [json.loads(l) for l in case_ledger.read_text().splitlines() if l.strip()]
    rendered = json.dumps(entries)
    resolution = [e for e in entries if e["record_kind"] == "approval_resolution"]
    if resolution:
        log(f"[positive] approval_resolution payload: {json.dumps(resolution[-1]['payload'])}")
        if resolution[-1]["payload"].get("resolved_by") != "operator_b":
            log("[positive] FAIL: resolved_by is not operator_b")
            ok = False
    else:
        log("[positive] FAIL: no approval_resolution record")
        ok = False
    escalation = [
        e for e in entries
        if e["record_kind"] == "authority_decision"
        and e["payload"].get("plan_item_id") == "disc"
        and e["payload"].get("verdict") == "escalate"
    ]
    requester = None
    if escalation:
        requester = (escalation[0]["payload"]
                     .get("action_request", {}).get("actor", {}).get("actor_id"))
        log(f"[positive] escalation requester (SoD field): {requester}")
    if requester != "operator_a":
        log(f"[positive] FAIL: escalation requester {requester!r} != operator_a")
        ok = False
    for bad in ('"actor_id":"gateway"', '"principal":"gateway"', '"resolved_by":"gateway"'):
        if bad in rendered:
            log(f"[positive] FAIL: gateway named in an actor/principal field: {bad}")
            ok = False
    approvals_jsonl = (root / "approvals.jsonl").read_text()
    if '"resolved_by":"gateway"' in approvals_jsonl:
        log("[positive] FAIL: approvals.jsonl names gateway as resolver")
        ok = False

    # identity.get reports the effective actor (delegated inspect)
    view = client_a.call(**delegated({"verb": "identity_get"}, "operator_a", "operator"))
    log(f"[positive] identity.get (delegated inspect): {json.dumps(view)}")
    eff = view.get("effective_actor") or {}
    if not (eff.get("actor_id") == "operator_a"
            and eff.get("delegated_by", {}).get("actor_id") == "gateway"):
        log("[positive] FAIL: identity.get effective_actor wrong")
        ok = False

    log(f"[positive] verdict: {'PASS' if ok else 'FAIL'}")
    return ok


def main():
    if not SERVER_BIN.exists():
        log(f"FATAL: server binary missing: {SERVER_BIN}")
        return 2
    results = {}

    # ---------------- Cell G: gateway cell (attacks c, d + positive e) --------
    root_g = Path(tempfile.mkdtemp(prefix="t02-critic-gateway-"))
    seed_gateway_cell(root_g)
    install_e2e_fixtures(root_g)
    server_g = Server(root_g, "gateway-cell")
    try:
        client = Client(server_g.socket)

        # (c) bound-but-not-allowlisted actor
        results["c-bound-not-allowlisted"], _ = run_attack(
            server_g, lambda: client,
            "c", delegated(commit_request("operator_c"), "operator_c", "operator"),
            "identity_delegation_refused", "not in this cell's gateway-delegable allowlist")

        # (d) role the target lacks (widening attempt)
        results["d-role-exceeds-binding"], _ = run_attack(
            server_g, lambda: client,
            "d", delegated(commit_request("security_officer"), "security_officer", "operator"),
            "identity_delegation_refused", "exceeds the roles bound to `security_officer`")

        # extra falsifier probes (not required by mandate, cheap on this cell)
        # probe 1: unknown actor entirely
        results["x-unknown-actor"], _ = run_attack(
            server_g, lambda: client,
            "x1", delegated(commit_request("ghost"), "ghost", "operator"),
            "identity_delegation_refused", None)
        # probe 2: garbled on_behalf_of falls back to the direct path; the
        # gateway's own actor must then be bound at THIS uid (it is) -> the
        # request proceeds as the GATEWAY principal, attributed to `gateway`,
        # never to a half-named end user. Check the entity mismatch guard.
        garbled = commit_request("operator_a")
        garbled["actor"] = dict(GATEWAY_ACTOR)
        garbled["on_behalf_of"] = "operator_a"  # malformed -> parses as absent
        results["x-garbled-on-behalf-of-entity-mismatch"], _ = run_attack(
            server_g, lambda: client, "x2", garbled,
            "identity_entity_mismatch", None)

        results["e-positive-sod"] = positive_sod(server_g)
    finally:
        server_g.stop()

    # ---------------- Cell O: operator-bound uid (attack a) -------------------
    root_o = Path(tempfile.mkdtemp(prefix="t02-critic-operator-"))
    seed_operator_bound_cell(root_o)
    install_e2e_fixtures(root_o)
    server_o = Server(root_o, "operator-cell")
    try:
        client = Client(server_o.socket)
        body = commit_request("operator_b")
        body["actor"] = {"actor_id": "operator_local", "role": "operator"}
        body["on_behalf_of"] = {"actor_id": "operator_b", "role": "operator"}
        results["a-operator-bound-uid"], _ = run_attack(
            server_o, lambda: client, "a", body,
            "identity_delegation_refused", "is not the configured gateway uid")
    finally:
        server_o.stop()

    # ---------------- Cell U: no gateway section (attack b) -------------------
    root_u = Path(tempfile.mkdtemp(prefix="t02-critic-ungoverned-"))
    seed_ungoverned_cell(root_u)
    install_e2e_fixtures(root_u)
    server_u = Server(root_u, "ungoverned-cell")
    try:
        client = Client(server_u.socket)
        results["b-gateway-section-omitted"], _ = run_attack(
            server_u, lambda: client, "b",
            delegated(commit_request("operator_local"), "operator_local", "operator"),
            "identity_delegation_refused", "configures no `gateway` principal")

        # same connection, direct claim must still work (per the builder's test)
        direct = commit_request("operator_local")
        direct["actor"] = {"actor_id": "operator_local", "role": "operator"}
        resp = client.call(**direct)
        log(f"\n[attack b] direct claim on same connection after refusal: "
            f"{'ok' if 'error' not in resp else json.dumps(resp)}")
        results["b-direct-still-works"] = "error" not in resp
    finally:
        server_u.stop()

    log("\n================ CRITIC RUN SUMMARY ================")
    all_ok = True
    for k, v in results.items():
        log(f"  {k}: {'PASS' if v else 'FAIL'}")
        all_ok = all_ok and bool(v)
    log(f"OVERALL: {'PASS' if all_ok else 'FAIL'}")
    log(f"cells kept for inspection: {root_g} {root_o} {root_u}")
    return 0 if all_ok else 1


if __name__ == "__main__":
    sys.exit(main())
