#!/usr/bin/env python3
"""Independent runtime attack (T04 critic): drive case_add_item with a cycle
attempt over a REAL server socket on a FRESH cell, hashing plan.json and
case-events.jsonl before/after. Includes a no-cycle positive control so a
connection failure cannot masquerade as a refusal.

Usage: attack.py <socket> <cell_root>
"""
import hashlib
import json
import os
import socket
import sys
import time


def call(sock_file, payload):
    s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    s.settimeout(30)
    s.connect(sock_file)
    s.sendall((json.dumps(payload) + "\n").encode())
    buf = b""
    while not buf.endswith(b"\n"):
        chunk = s.recv(65536)
        if not chunk:
            break
        buf += chunk
    s.close()
    return json.loads(buf.decode().strip())


def sha256(path):
    with open(path, "rb") as fh:
        return hashlib.sha256(fh.read()).hexdigest()


def nlines(path):
    if not os.path.exists(path):
        return 0
    with open(path) as fh:
        return sum(1 for line in fh if line.strip())


def main():
    sock_file, root = sys.argv[1], sys.argv[2]
    actor = {"actor_id": "operator_local", "role": "operator"}
    plan_file = os.path.join(root, "attack-plan.json")
    item_base = {
        "name": "discretionary",
        "operations": [],
        "entry_criteria": [],
        "exit_criteria": [],
        "settlement_criteria": {"required_artifacts": [], "required_declarations": []},
        "item_kind": "human_task",
        "markers": {"required": False},
        "max_instances": 1,
    }
    plan = {
        "version": "0.2",
        "plan_id": "plan_attack",
        "case_id": "case_placeholder",
        "run_id": "run_placeholder",
        "intent_id": "int_attack",
        "items": [
            dict(item_base, plan_item_id="judge", markers={"required": True})
        ],
        "template_ref": None,
        "job_contract_ref": None,
    }
    with open(plan_file, "w") as fh:
        json.dump(plan, fh)

    sub = call(sock_file, {
        "verb": "submit", "plan": "attack-plan.json", "policy": "policy.yaml",
        "entity": "operator_local", "process": "attack", "timeout": 60,
        "request_id": "attack-submit-1", "actor": actor,
    })
    print("submit ->", json.dumps(sub))
    assert sub.get("case_id"), f"submit failed: {sub}"
    case_id = sub["case_id"]
    case_dir = os.path.join(root, "cases", case_id)
    plan_json = os.path.join(case_dir, "plan.json")
    events = os.path.join(case_dir, "case-events.jsonl")

    # --- ATTACK 1: cycle attempt -----------------------------------------
    before_sha, before_lines = sha256(plan_json), nlines(events)
    cyclic = dict(item_base, plan_item_id="cyclic", depends_on=["cyclic"])
    res = call(sock_file, {
        "verb": "case_add_item", "case_id": case_id, "item": cyclic,
        "policy": "policy.yaml", "request_id": "attack-cycle-1", "actor": actor,
    })
    after_sha, after_lines = sha256(plan_json), nlines(events)
    print("cycle add ->", json.dumps(res))
    print(f"plan.json sha before={before_sha} after={after_sha}")
    print(f"case-events lines before={before_lines} after={after_lines}")
    ok = True
    if res.get("error_class") != "plan_cycle_error":
        print("FAIL: cycle refusal class != plan_cycle_error")
        ok = False
    if before_sha != after_sha:
        print("FAIL: plan.json changed on refused proposal")
        ok = False
    if before_lines != after_lines:
        print("FAIL: case-events.jsonl changed on refused proposal")
        ok = False
    with open(events) as fh:
        kinds = [json.loads(l)["kind"] for l in fh if l.strip()]
    if "plan_mutated" in kinds:
        print("FAIL: PlanMutated present after cycle refusal")
        ok = False

    # --- ATTACK 2: terminate with empty reason (input refused, no writes) --
    res2 = call(sock_file, {
        "verb": "case_terminate", "case_id": case_id, "reason": "   ",
        "policy": "policy.yaml", "request_id": "attack-term-1", "actor": actor,
    })
    print("empty-reason terminate ->", json.dumps(res2))
    if "error" not in res2 or "non-empty reason" not in res2.get("error", ""):
        print("FAIL: empty reason not refused")
        ok = False
    if nlines(events) != after_lines or sha256(plan_json) != after_sha:
        print("FAIL: refused terminate wrote something")
        ok = False
    if json.load(open(os.path.join(case_dir, "case.json")))["state"] != "active":
        print("FAIL: case state changed on refused terminate")
        ok = False

    # --- POSITIVE CONTROL: a legal add mutates the plan --------------------
    good = dict(item_base, plan_item_id="legit", depends_on=[])
    res3 = call(sock_file, {
        "verb": "case_add_item", "case_id": case_id, "item": good,
        "policy": "policy.yaml", "request_id": "attack-add-1", "actor": actor,
    })
    print("legal add ->", json.dumps(res3))
    if res3.get("ok") is not True or res3.get("proposed_by") != "operator_local":
        print("FAIL: positive control add did not succeed with verified proposer")
        ok = False
    if sha256(plan_json) == after_sha:
        print("FAIL: legal add did not change plan.json (control says nothing works?)")
        ok = False
    with open(events) as fh:
        kinds2 = [json.loads(l)["kind"] for l in fh if l.strip()]
    if kinds2[-1] != "plan_mutated":
        print(f"FAIL: last event after legal add is {kinds2[-1]}, not plan_mutated")
        ok = False
    durable = json.load(open(plan_json))
    added = [i for i in durable["items"] if i["plan_item_id"] == "legit"]
    if not added or added[0].get("proposed_by") != "operator_local":
        print("FAIL: legal item not in durable plan with proposer overwrite")
        ok = False

    print("ATTACK RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
