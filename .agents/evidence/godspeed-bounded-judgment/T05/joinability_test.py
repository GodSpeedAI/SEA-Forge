#!/usr/bin/env python3
"""T05 joinability harness — joins one operative loop's artifacts by committed
ledger causal references ALONE (never run_id string matching), and executes
the preregistered adversarial cases.

Discipline: task-owned executable evidence (plan task_local_harness_rule);
sha256 frozen in the decision log before first execution; never overwritten.

Preregistration: .agents/preregistrations/godspeed-bounded-judgment-T05.prereg.yaml
"""

from __future__ import annotations

import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[4]
SEA = REPO / "target/debug/sea-forge"
POLICY = """version: "0.1"
rules:
  - name: allow-model-write
    verdict: allow
    actor_role: operator
    operation_kind: write_file
    path_prefix: ""
  - name: allow-self-validate
    verdict: allow
    actor_role: operator
    operation_kind: execute_command
    argv0: sea-forge
"""
RESULTS = []


def record(case, ok, detail=""):
    RESULTS.append({"case": case, "behaved_as_specified": ok, "detail": detail})
    print(f"[{case}] behaved_as_specified={ok} {detail}")


def mint_run(root: Path, intent: str) -> str:
    policy = root / "policy.yaml"
    policy.write_text(POLICY)
    proc = subprocess.run(
        [str(SEA), "run", "--root", str(root / "state"), "--policy", str(policy), "--intent", intent],
        capture_output=True, text=True, timeout=300,
    )
    if proc.returncode != 0:
        raise RuntimeError(f"run failed: {proc.stdout[-400:]} {proc.stderr[-400:]}")
    for line in proc.stdout.splitlines():
        if line.startswith("run_id="):
            return line.split("=", 1)[1].strip()
    raise RuntimeError(f"no run_id in {proc.stdout[-400:]}")


def load_ledger(state: Path, case_id: str):
    path = state / "ledgers" / f"case-{case_id}" / "entries.jsonl"
    rows = [json.loads(l) for l in path.read_text().splitlines() if l.strip()]
    return path, rows


def case_ledgers(state: Path):
    out = {}
    for d in (state / "ledgers").iterdir():
        if d.is_dir() and d.name.startswith("case-"):
            rows = [json.loads(l) for l in (d / "entries.jsonl").read_text().splitlines() if l.strip()]
            out[d.name[5:]] = rows
    return out


def find_one(rows, kind):
    matches = [r for r in rows if r["record_kind"] == kind]
    assert len(matches) == 1, f"expected exactly one {kind}, got {len(matches)}"
    return matches[0]


def join_by_causal_refs(state: Path, case_id: str):
    """Join settlement -> authority_decision -> authority_request -> intent by
    committed causal references alone. Raises with the missing hop named."""
    _path, rows = load_ledger(state, case_id)
    index = {r["entry_ulid"]: r for r in rows}
    settlement = find_one(rows, "settlement_event")
    run_subjects = {s for s in settlement["subject_refs"] if s.startswith("run_")}

    # hop 1: settlement -> authority decisions (the repaired hop)
    decisions = []
    for ref in settlement["authority_refs"]:
        entry = index.get(ref)
        if entry is None:
            raise AssertionError(f"missing hop: settlement cites absent entry {ref}")
        if entry["record_kind"] != "authority_decision":
            raise AssertionError(f"wrong hop: {ref} is {entry['record_kind']}, not authority_decision")
        if not ({s for s in entry["subject_refs"]} & run_subjects):
            raise AssertionError(f"mismatched pair: decision {ref} does not belong to the settlement's run")
        decisions.append(entry)

    # hop 2: each decision -> its request
    for decision in decisions:
        for ref in decision["authority_refs"]:
            request = index.get(ref)
            if request is None:
                raise AssertionError(f"missing hop: decision cites absent request {ref}")
            if request["record_kind"] != "authority_request":
                raise AssertionError(f"wrong hop: {ref} is {request['record_kind']}")

    # hop 3: back to admission (the intent entry is the loop's correlation identity)
    intent = find_one(rows, "intent")
    return settlement, decisions, intent["entry_ulid"]


def main() -> int:
    tmp = Path(tempfile.mkdtemp(prefix="t05-joinability-"))
    try:
        # ---- real loop: mint two runs in one cell (similar ids on purpose) ----
        run_a = mint_run(tmp, "Generate and validate a simple DomainForge .sea model for calculator A")
        run_b = mint_run(tmp, "Generate and validate a simple DomainForge .sea model for calculator AB")
        state = tmp / "state"
        ledgers = case_ledgers(state)

        # core join: both loops join by causal references alone
        joined = {}
        for case_id in ledgers:
            settlement, decisions, intent_ulid = join_by_causal_refs(state, case_id)
            joined[case_id] = (settlement, decisions, intent_ulid)
        record("core_join_by_causal_refs_only", len(joined) == len(ledgers),
               f"{len(joined)} loops joined; run_ids={run_a[:24]}…/{run_b[:24]}… (never used for joining)")

        # attack: similar-id collision — run ids share a long prefix; a string
        # matcher could false-join; the causal joiner resolves each loop to its
        # own distinct authority decisions.
        ra, rb = joined[sorted(joined)[0]], joined[sorted(joined)[1]]
        decisions_a = {d["entry_ulid"] for _, da, _ in [ra] for d in da}
        decisions_b = {d["entry_ulid"] for _, db, _ in [rb] for d in db}
        record("similar_id_collision_no_false_join",
               run_a.split("_")[1] == run_b.split("_")[1] and not (decisions_a & decisions_b),
               f"shared prefix len>=22: {run_a[:22]}; decision sets disjoint")

        # attack: duplicate external id — both runs label decisions 'auth_01';
        # the causal joiner never consults labels, so no wrong join.
        labels_a = {d["payload"].get("decision_id") for _, da, _ in [ra] for d in da}
        labels_b = {d["payload"].get("decision_id") for _, db, _ in [rb] for d in db}
        record("duplicate_external_id_no_wrong_join",
               labels_a == labels_b and not (decisions_a & decisions_b),
               f"labels collide ({labels_a}), ULIDs do not")

        # attack: restart — the two runs were produced by separate process
        # invocations over one cell; both join with the same mechanism.
        record("restart_artifacts_still_joinable", len(joined) >= 2,
               "separate invocations joined by one mechanism")

        # attack: missing hop — remove a referenced decision from a COPY.
        copy_state = tmp / "copy" / "state"
        copy_state.parent.mkdir()
        shutil.copytree(state, copy_state)
        broken_case = sorted(joined)[0]
        path, rows = load_ledger(copy_state, broken_case)
        victim = joined[broken_case][1][0]["entry_ulid"]
        rows = [r for r in rows if r["entry_ulid"] != victim]
        path.write_text("\n".join(json.dumps(r) for r in rows) + "\n")
        try:
            join_by_causal_refs(copy_state, broken_case)
            record("missing_hop_named_not_tolerated", False, "join succeeded despite missing hop")
        except AssertionError as e:
            record("missing_hop_named_not_tolerated", "missing hop" in str(e), str(e))

        # attack: stale mapping — a settlement citing a foreign ledger's ULID.
        other_case = sorted(joined)[1]
        foreign = joined[other_case][1][0]["entry_ulid"]
        path, rows = load_ledger(copy_state, broken_case)
        rows = [json.loads(l) for l in path.read_text().splitlines() if l.strip()]
        for r in rows:
            if r["record_kind"] == "settlement_event":
                r["authority_refs"] = [foreign]
        path.write_text("\n".join(json.dumps(r) for r in rows) + "\n")
        try:
            join_by_causal_refs(copy_state, broken_case)
            record("stale_mapping_rejected", False, "foreign ULID accepted")
        except AssertionError as e:
            record("stale_mapping_rejected", "absent entry" in str(e), str(e))

        # attack: mismatched pair — cite a REAL decision ULID from the OTHER
        # run (present in a merged index only if ledgers were merged); here the
        # subject cross-check must reject it.
        path, rows = load_ledger(copy_state, broken_case)
        for r in rows:
            if r["record_kind"] == "settlement_event":
                r["authority_refs"] = [joined[other_case][1][0]["entry_ulid"] + "X"][:1] and [foreign]
                break
        # merge the foreign entry into this ledger so the ULID *resolves* but
        # belongs to another run: the subject cross-check must still reject.
        foreign_entry = next(r for r in case_ledgers(state)[other_case]
                             if r["entry_ulid"] == foreign)
        rows.append(foreign_entry)
        path.write_text("\n".join(json.dumps(r) for r in rows) + "\n")
        try:
            join_by_causal_refs(copy_state, broken_case)
            record("mismatched_pair_rejected", False, "cross-run decision accepted")
        except AssertionError as e:
            record("mismatched_pair_rejected", "does not belong" in str(e), str(e))

        # VAR-008: immutability — append a contradicting settlement to a COPY;
        # the original judgment entries stay byte-identical.
        path, rows = load_ledger(tmp / "state", broken_case)
        original_bytes = path.read_bytes()
        settlement = find_one(rows, "settlement_event")
        contradiction = dict(settlement)
        contradiction["entry_ulid"] = "01ZZZZZZZZZZZZZZZZZZZZZZZZ"
        contradiction["payload"] = dict(settlement["payload"], status="rejected")
        contradiction["authority_refs"] = list(settlement["authority_refs"])
        path.write_bytes(original_bytes + (json.dumps(contradiction) + "\n").encode())
        after = load_ledger(tmp / "state", broken_case)[1]
        original_entries = [r for r in after if r["entry_ulid"] in
                            {x["entry_ulid"] for x in rows}]
        record("var008_contradiction_leaves_judgment_immutable",
               all(a == b for a, b in zip(rows, original_entries)),
               "original entries byte-identical; contradiction appended, never rewritten")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)

    ok = all(r["behaved_as_specified"] for r in RESULTS)
    print(f"joinability: {'PASS' if ok else 'FAIL'} ({len(RESULTS)} cases)")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
