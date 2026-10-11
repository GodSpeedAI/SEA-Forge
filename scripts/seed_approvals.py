"""Seed a cell with real escalations and a real resolved approval.

Everything here goes over the cell's real Unix socket to a real running server.
Nothing is hand-written into the ledger: each approval below exists because a
real submission was really escalated by a real policy, and the resolved one was
really decided by a second actor. A seeded cell must be indistinguishable from
one an operator produced by hand, or it is fabricated evidence in a system whose
entire product is trustworthy evidence.

    python3 seed_approvals.py <cell-root> <socket-path>
"""

import json
import os
import socket
import sys

CELL, SOCK = sys.argv[1], sys.argv[2]


class Conn:
    def __init__(self):
        self.s = socket.socket(socket.AF_UNIX)
        self.s.connect(SOCK)
        self.f = self.s.makefile("rw")

    def call(self, **kw):
        self.f.write(json.dumps(kw) + "\n")
        self.f.flush()
        return json.loads(self.f.readline())


def fail(message):
    print(f"  ERROR {message}", file=sys.stderr)
    sys.exit(1)


c = Conn()

# The policy escalates any command, and separately permits approval resolution.
# Without the second rule the cell would escalate work nobody could ever
# approve. It grants the *capability* to approve; the authority engine still
# independently refuses a self-approval.
policy = os.path.join(CELL, "demo-escalate.yaml")
with open(policy, "w") as fh:
    fh.write(
        'version: "0.1"\n'
        "rules:\n"
        "  - name: escalate-command\n"
        "    verdict: escalate\n"
        "    actor_role: operator\n"
        "    operation_kind: execute_command\n"
        "    argv0: sea-forge\n"
        "  - name: resolve-approval\n"
        "    verdict: allow\n"
        "    actor_role: operator\n"
        "    operation_kind: approval_resolution\n"
    )

# argv0 must be an absolute path whose file name is `sea-forge` AND which
# canonicalizes to the process that runs it. For work submitted in-process by
# the server, that process is the *server*, which is why `<cell>/sea-forge`
# points at sea-forge-server and the CLI lives beside it under another name.
argv0 = os.path.join(CELL, "sea-forge")
if not os.path.exists(argv0):
    fail(f"{argv0} does not exist; the seed script must symlink the server there")


def plan_for(name, plan_id):
    return {
        "version": "0.2",
        "plan_id": plan_id,
        "case_id": "case_placeholder",
        "run_id": "run_placeholder",
        "intent_id": f"int_{plan_id}",
        "items": [
            {
                "plan_item_id": "task",
                "name": name,
                "operations": [
                    {"kind": "execute_command", "argv": [argv0, "--version"], "cwd": "."}
                ],
                "entry_criteria": [],
                "exit_criteria": [],
                "settlement_criteria": {"require_exit_zero": True},
                "item_kind": "sandboxed_task",
                "markers": {"required": True},
                "max_instances": 1,
                "depends_on": [],
            }
        ],
    }


def submit(name, plan_id):
    path = os.path.join(CELL, f"demo-plan-{plan_id}.json")
    with open(path, "w") as fh:
        json.dump(plan_for(name, plan_id), fh)
    response = c.call(
        verb="submit",
        actor={"actor_id": "operator_a", "role": "operator"},
        plan=path,
        policy=policy,
        entity="operator_a",
        process="demo",
        timeout=60,
    )
    if response.get("error_class"):
        fail(f"submit '{name}' refused: {response}")
    return response.get("case_id")


def approvals_for(case_id):
    listed = c.call(verb="approval_list", case_id=case_id)
    return listed.get("approvals", [])


print("== seeding an approval that is waiting for a decision ==")
pending_case = submit("Deploy the pricing projection", "pending")
pending = approvals_for(pending_case)
if not pending:
    fail(f"case {pending_case} opened no approval; the escalate rule did not match")
print(f"  case {pending_case}: {pending[0]['approval_id']} is waiting in the inbox")

print("== seeding an approval that a second actor resolved ==")
resolved_case = submit("Publish the quarterly capability report", "resolved")
opened = approvals_for(resolved_case)
if not opened:
    fail(f"case {resolved_case} opened no approval")
approval_id = opened[0]["approval_id"]

# operator_a raised this work, so operator_a cannot clear it. Recording the
# refusal in the seeded ledger is deliberate: separation of duty is a headline
# behaviour of this system and the demo cell should show that it really fires.
refused = c.call(
    verb="approval_decide",
    actor={"actor_id": "operator_a", "role": "operator"},
    case_id=resolved_case,
    approval_id=approval_id,
    decision="approve",
)
if refused.get("error_class") != "separation_of_duty":
    fail(f"a self-approval was not refused as separation_of_duty: {refused}")
print(f"  operator_a was refused their own approval ({refused['error_class']})")

decided = c.call(
    verb="approval_decide",
    actor={"actor_id": "operator_b", "role": "operator"},
    case_id=resolved_case,
    approval_id=approval_id,
    decision="approve",
)
if decided.get("error_class"):
    fail(f"operator_b could not resolve the approval: {decided}")
if "resolved_by=operator_b" not in decided.get("output", ""):
    fail(f"the resolution was not attributed to operator_b: {decided}")
print(f"  case {resolved_case}: {approval_id} resolved by operator_b")

print(f"\nseeded 2 cases: {pending_case} (waiting), {resolved_case} (resolved)")
