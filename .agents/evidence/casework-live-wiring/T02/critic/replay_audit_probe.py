#!/usr/bin/env python3
"""Two falsifier probes on the gateway cell:

1. replay/dedupe: the same delegated request_id+payload twice from the
   gateway must return the recorded outcome and write exactly ONE
   delegation-audit record (audit sits after dedupe).
   Also: replaying with a DIFFERENT on_behalf_of under the same request_id
   must be refused (payload hash mismatch -> request_id_reused), never
   re-attributed.

2. unrecordable audit: with ledgers/ made unwritable, a delegated request
   must be REFUSED (delegation_audit_unwritable) rather than run
   unattributed, and the cell must show no case from it.
"""
import json
import os
import stat
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from attack_driver import (  # noqa: E402
    Client, Server, seed_gateway_cell, install_e2e_fixtures, delegated,
    commit_request, cell_digest, case_count,
)


def audit_records(root):
    p = root / "ledgers/delegation-audit/entries.jsonl"
    if not p.exists():
        return []
    return [json.loads(l) for l in p.read_text().splitlines() if l.strip()]


def main():
    ok = True
    root = Path(tempfile.mkdtemp(prefix="t02-critic-replay-"))
    seed_gateway_cell(root)
    install_e2e_fixtures(root)
    server = Server(root, "replay-cell")
    try:
        client = Client(server.socket)

        body = delegated(commit_request("operator_a"), "operator_a", "operator")
        first = client.call(**body)
        print(f"[replay] first: {'ok' if 'error' not in first else first}", flush=True)
        second = client.call(**body)
        print(f"[replay] identical replay: {json.dumps(second)[:200]}", flush=True)
        recs = audit_records(root)
        print(f"[replay] delegation-audit records: {len(recs)}", flush=True)
        if len(recs) != 1:
            ok = False
            print("[replay] FAIL: replay wrote extra audit records", flush=True)

        # Same request_id, different on_behalf_of (re-attribution attempt).
        # The identity gate runs BEFORE dedupe: with entity left at
        # operator_a, the effective actor becomes operator_b and the entity
        # consistency check refuses it (identity_entity_mismatch) before the
        # correlation store is even consulted.
        tampered = dict(body)
        tampered["on_behalf_of"] = {"actor_id": "operator_b", "role": "operator"}
        third = client.call(**tampered)
        print(f"[replay] same-id different-actor: "
              f"{json.dumps(third)[:200]}", flush=True)
        if third.get("error_class") not in ("request_id_reused",
                                            "identity_entity_mismatch",
                                            "identity_delegation_refused"):
            ok = False
            print("[replay] FAIL: id re-pointing at another actor was not refused",
                  flush=True)
        # Same request_id, same actor, different payload -> request_id_reused.
        repointed = dict(body)
        repointed["params"] = dict(body["params"], max_rows="26")
        fourth = client.call(**repointed)
        print(f"[replay] same-id different-payload: "
              f"{json.dumps(fourth)[:200]}", flush=True)
        if fourth.get("error_class") != "request_id_reused":
            ok = False
            print("[replay] FAIL: reused id with different payload not refused as reused",
                  flush=True)
        recs = audit_records(root)
        if len(recs) != 1:
            ok = False
            print("[replay] FAIL: audit records changed on refused replays", flush=True)
    finally:
        server.stop()

    root2 = Path(tempfile.mkdtemp(prefix="t02-critic-unwritable-"))
    seed_gateway_cell(root2)
    install_e2e_fixtures(root2)
    server2 = Server(root2, "unwritable-cell")
    try:
        client = Client(server2.socket)
        # Seed the ledgers dir by one lawful delegated commit first.
        good = client.call(**delegated(commit_request("operator_a"), "operator_a", "operator"))
        print(f"\n[unwritable] seeding commit: {'ok' if 'error' not in good else good}",
              flush=True)
        # Make the EXISTING delegation-audit ledger unappendable: the entries
        # file itself read-only (appending to an existing file needs no
        # directory write permission, so chmod'ing the dir alone is not enough,
        # as the first run of this probe demonstrated).
        audit_dir = root2 / "ledgers/delegation-audit"
        audit_file = audit_dir / "entries.jsonl"
        dir_mode = audit_dir.stat().st_mode
        file_mode = audit_file.stat().st_mode
        os.chmod(audit_dir, stat.S_IRUSR | stat.S_IXUSR)
        os.chmod(audit_file, stat.S_IRUSR)
        try:
            before = cell_digest(root2)
            cases_before = case_count(root2)
            body = delegated(commit_request("operator_b"), "operator_b", "operator")
            refused = client.call(**body)
            print(f"[unwritable] RESPONSE: {json.dumps(refused)[:400]}", flush=True)
            after = cell_digest(root2)
            cases_after = case_count(root2)
            cls = refused.get("error_class")
            if cls != "delegation_audit_unwritable":
                ok = False
                print(f"[unwritable] FAIL: error_class={cls!r} != "
                      f"'delegation_audit_unwritable'", flush=True)
            if refused.get("no_side_effect") is not True:
                ok = False
                print("[unwritable] FAIL: no_side_effect is not true", flush=True)
            if cases_after != cases_before:
                ok = False
                print("[unwritable] FAIL: a case appeared despite the refusal", flush=True)
            print(f"[unwritable] cases {cases_before}->{cases_after}, "
                  f"digest {'unchanged' if before == after else 'CHANGED'}", flush=True)
        finally:
            os.chmod(audit_dir, dir_mode)
            os.chmod(audit_file, file_mode)
    finally:
        server2.stop()

    print(f"\nPROBES: {'PASS' if ok else 'FAIL'}", flush=True)
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
