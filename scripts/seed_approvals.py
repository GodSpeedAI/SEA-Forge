"""Seed a cell with real escalations and a real resolved approval.

Everything here goes over the cell's real Unix socket to a real running server.
Nothing is hand-written into the ledger: each approval below exists because a
real submission was really escalated by a real policy, and the resolved one was
really decided by a second actor. A seeded cell must be indistinguishable from
one an operator produced by hand, or it is fabricated evidence in a system whose
entire product is trustworthy evidence.

Every request is sent as the cell's gateway principal (`gateway`, the only OS
principal this script runs as) with an `on_behalf_of` block naming the end user.
That is the supported way to act for an actor that has no uid of its own: one
uid maps to exactly one actor, so `operator_a` and `operator_b` both act through
the gateway, and the server attributes each request to the end user.

    python3 seed_approvals.py <cell-root> <socket-path> <argv0-tool-path>

`argv0-tool-path` is an absolute path, outside the cell, whose file name is
`sea-forge` and which resolves to the running server binary. The policy matches
on that file name, and the authority engine requires it to canonicalize to the
process that executes the work.
"""

import json
import os
import socket
import sys
import uuid

CELL, SOCK, ARGV0 = sys.argv[1], sys.argv[2], sys.argv[3]

# Paths sent to the server are workspace-relative to the cell root (F-16).
POLICY_REL = "demo-escalate.yaml"
GATEWAY = {"actor_id": "gateway", "role": "service"}


class Conn:
    def __init__(self):
        self.s = socket.socket(socket.AF_UNIX)
        self.s.connect(SOCK)
        self.f = self.s.makefile("rw")

    def call(self, request):
        self.f.write(json.dumps(request) + "\n")
        self.f.flush()
        return json.loads(self.f.readline())


def fail(message):
    print(f"  ERROR {message}", file=sys.stderr)
    sys.exit(1)


c = Conn()


def as_end_user(actor_id, verb, **fields):
    """Send one durable request as `actor_id`, delegated through the gateway.

    Each request gets a fresh request_id: the server refuses a reused one and
    refuses a durable mutation that carries none.
    """
    request = {
        "verb": verb,
        "actor": GATEWAY,
        "on_behalf_of": {"actor_id": actor_id, "role": "operator"},
        "entity": actor_id,
        "request_id": uuid.uuid4().hex,
    }
    request.update(fields)
    return c.call(request)


# The policy escalates any command, and separately permits approval resolution.
# Without the second rule the cell would escalate work nobody could ever
# approve. It grants the *capability* to approve; the authority engine still
# independently refuses a self-approval.
policy = os.path.join(CELL, POLICY_REL)
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

if not os.path.exists(ARGV0):
    fail(f"{ARGV0} does not exist; the seed script must create the tool link there")


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
                    {"kind": "execute_command", "argv": [ARGV0, "--version"], "cwd": "."}
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
    plan_rel = f"demo-plan-{plan_id}.json"
    with open(os.path.join(CELL, plan_rel), "w") as fh:
        json.dump(plan_for(name, plan_id), fh)
    response = as_end_user(
        "operator_a",
        "submit",
        plan=plan_rel,
        policy=POLICY_REL,
        process="demo",
        timeout=60,
    )
    if response.get("error_class"):
        fail(f"submit '{name}' refused ({response['error_class']}): {response.get('error')}")
    return response["case_id"]


def approvals_for(case_id):
    listed = c.call({"verb": "approval_list", "case_id": case_id})
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
refused = as_end_user(
    "operator_a",
    "approval_decide",
    case_id=resolved_case,
    approval_id=approval_id,
    decision="approve",
)
if refused.get("error_class") != "separation_of_duty":
    fail(f"a self-approval was not refused as separation_of_duty: {refused}")
print(f"  operator_a was refused their own approval ({refused['error_class']})")

decided = as_end_user(
    "operator_b",
    "approval_decide",
    case_id=resolved_case,
    approval_id=approval_id,
    decision="approve",
)
if decided.get("error_class"):
    fail(f"operator_b could not resolve the approval ({decided['error_class']}): {decided}")
if "resolved_by=operator_b" not in decided.get("output", ""):
    fail(f"the resolution was not attributed to operator_b: {decided}")
print(f"  case {resolved_case}: {approval_id} resolved by operator_b")

print(f"\nseeded 2 cases: {pending_case} (waiting), {resolved_case} (resolved)")
