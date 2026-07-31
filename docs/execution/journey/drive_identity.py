"""Live identity + separation-of-duty journey against a real server.

Run against a booted cell whose `server.yaml` binds two actors to this uid:

    python3 drive_identity.py <cell-root> <socket-path>

Everything here goes over the real Unix socket, so the uid the server reads
comes from `SO_PEERCRED` — not from anything this script can assert.
"""

import json
import os
import socket
import sys

CELL, SOCK = sys.argv[1], sys.argv[2]


class C:
    def __init__(self):
        self.s = socket.socket(socket.AF_UNIX)
        self.s.connect(SOCK)
        self.f = self.s.makefile("rw")

    def call(self, **kw):
        self.f.write(json.dumps(kw) + "\n")
        self.f.flush()
        return json.loads(self.f.readline())


def show(label, value, limit=400):
    print(f"  {label}: {json.dumps(value)[:limit]}")


failures = []


def expect(condition, description):
    print(("  PASS  " if condition else "  FAIL  ") + description)
    if not condition:
        failures.append(description)


c = C()

print("== identity.get: who does this cell think I am? ==")
me = c.call(verb="identity_get")
show("view", me)
expect(me.get("configured") is True, "the cell reports itself configured")
expect(
    {a["actor_id"] for a in me.get("available", [])} == {"operator_a", "operator_b"},
    "both bound actors are claimable on this connection",
)
expect(isinstance(me.get("uid"), int), "the peer uid was read from the socket")

print("== identity.get discloses nothing about other uids ==")
expect(
    "9999" not in json.dumps(me),
    "no binding for a uid other than this connection's appears in the view",
)

print("== a protected verb with no actor block ==")
bare = c.call(verb="submit", plan="/nope.json", policy="/nope.yaml",
              entity="operator_a", process="live", timeout=30)
show("response", bare)
expect(bare.get("error_class") == "identity_required", "refused as identity_required")
expect(bare.get("no_side_effect") is True, "refusal states no side effect")

print("== claiming an actor this uid does not hold ==")
imposter = c.call(verb="submit", actor={"actor_id": "operator_z", "role": "operator"},
                  plan="/nope.json", policy="/nope.yaml",
                  entity="operator_z", process="live", timeout=30)
expect(imposter.get("error_class") == "identity_not_bound", "refused as identity_not_bound")

print("== verifying one actor while attributing work to another ==")
mismatch = c.call(verb="submit", actor={"actor_id": "operator_a", "role": "operator"},
                  plan="/nope.json", policy="/nope.yaml",
                  entity="operator_b", process="live", timeout=30)
expect(
    mismatch.get("error_class") == "identity_entity_mismatch",
    "refused as identity_entity_mismatch",
)

print("== operator_a submits work that escalates ==")
policy = os.path.join(CELL, "escalate.yaml")
with open(policy, "w") as fh:
    fh.write('version: "0.1"\nrules:\n  - name: escalate-command\n'
             "    verdict: escalate\n    actor_role: operator\n"
             "    operation_kind: execute_command\n    argv0: sea-forge\n"
             # Resolving the approval is itself a governed action. Without this
             # rule the cell escalates work it can never let anyone approve.
             # The authority engine independently requires the approver to
             # differ from the requester, so this rule grants the *capability*
             # to approve, not the right to self-approve.
             "  - name: resolve-approval\n    verdict: allow\n"
             "    actor_role: operator\n    operation_kind: approval_resolution\n")

# argv0 must be an absolute path named `sea-forge` that canonicalizes to the
# running executable: policy matches on the file name, and `untrusted_executable`
# requires the canonical path to equal `current_exe()`.
argv0 = os.path.join(CELL, "sea-forge")
plan = {
    "version": "0.2", "plan_id": "plan_sod", "case_id": "case_placeholder",
    "run_id": "run_placeholder", "intent_id": "int_sod",
    "items": [{
        "plan_item_id": "task", "name": "escalating task",
        "operations": [{"kind": "execute_command", "argv": [argv0, "--version"], "cwd": "."}],
        "entry_criteria": [], "exit_criteria": [],
        "settlement_criteria": {"require_exit_zero": True},
        "item_kind": "sandboxed_task", "markers": {"required": True},
        "max_instances": 1, "depends_on": [],
    }],
}
plan_path = os.path.join(CELL, "plan.json")
with open(plan_path, "w") as fh:
    fh.write(json.dumps(plan))

submitted = c.call(verb="submit", actor={"actor_id": "operator_a", "role": "operator"},
                   plan=plan_path, policy=policy,
                   entity="operator_a", process="live", timeout=60)
show("submit", submitted)
case_id = submitted.get("case_id")
expect(case_id is not None, "the submit produced a case")

print("== the approval it opened ==")
inbox = c.call(verb="approval_list")
approvals = inbox.get("approvals", [])
show("inbox", inbox)
expect(len(approvals) == 1, "exactly one approval is waiting")

if approvals:
    approval_id = approvals[0]["approval_id"]

    print("== operator_a tries to resolve their own approval ==")
    mine = c.call(verb="approval_decide",
                  actor={"actor_id": "operator_a", "role": "operator"},
                  case_id=case_id, approval_id=approval_id, decision="approve")
    show("response", mine)
    expect(mine.get("error_class") == "separation_of_duty", "refused as separation_of_duty")
    expect(mine.get("no_side_effect") is True, "refusal states no side effect")

    print("== the same attempt on a brand-new connection ==")
    reconnected = C().call(verb="approval_decide",
                           actor={"actor_id": "operator_a", "role": "operator"},
                           case_id=case_id, approval_id=approval_id, decision="approve")
    expect(
        reconnected.get("error_class") == "separation_of_duty",
        "a fresh connection does not launder the self-approval",
    )

    print("== the approval is still pending, i.e. nothing was resolved ==")
    still = c.call(verb="approval_list").get("approvals", [])
    expect(
        any(a["approval_id"] == approval_id for a in still),
        "the refused approval remains in the inbox",
    )

    print("== operator_b resolves it ==")
    theirs = c.call(verb="approval_decide",
                    actor={"actor_id": "operator_b", "role": "operator"},
                    case_id=case_id, approval_id=approval_id, decision="approve")
    show("response", theirs)
    expect(
        theirs.get("error_class") != "separation_of_duty",
        "a second actor is not blocked by separation of duty",
    )
    # The assertion the socket tests cannot make: `approval_decide` shells out
    # to the real CLI, which only exists in a real cell. This is where an
    # escalation that opens an unresolvable approval gets caught.
    expect(
        "criteria" not in json.dumps(theirs.get("error", "")),
        "the approval carries a usable criteria binding",
    )
    left_inbox = not any(
        a["approval_id"] == approval_id
        for a in c.call(verb="approval_list").get("approvals", [])
    )
    expect(left_inbox, "the resolved approval left the inbox")
    # The audit trail must name the actor who actually decided. The server
    # shells out to the CLI to resolve an approval, and the CLI defaults its
    # actor to `operator_local` — so before the verified actor was threaded
    # through, every approval resolved over SFWP was recorded as decided by
    # someone who had not decided it.
    expect(
        "resolved_by=operator_b" in theirs.get("output", ""),
        "the resolution is attributed to the actor who made it",
    )

print()
if failures:
    print(f"FAILED ({len(failures)}):")
    for description in failures:
        print(f"  - {description}")
    sys.exit(1)
print("all identity journey checks passed")
