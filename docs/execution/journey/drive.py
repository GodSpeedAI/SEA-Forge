import json, os, socket, sys

SOCK = "/tmp/sf-journey.sock"
CELL = sys.argv[1]


class C:
    def __init__(self):
        self.s = socket.socket(socket.AF_UNIX)
        self.s.connect(SOCK)
        self.f = self.s.makefile("rw")

    def call(self, **kw):
        self.f.write(json.dumps(kw) + "\n")
        self.f.flush()
        return json.loads(self.f.readline())


c = C()
print("== system_hello ==")
print(json.dumps(c.call(verb="system_hello"))[:300])

# A policy with no rules: default deny.
policy = os.path.join(CELL, "deny.yaml")
open(policy, "w").write('version: "0.1"\nrules:\n')

plan = {
    "version": "0.2", "plan_id": "plan_journey", "case_id": "case_placeholder",
    "run_id": "run_placeholder", "intent_id": "int_journey",
    "items": [{
        "plan_item_id": "task", "name": "denied sandbox task",
        "operations": [{"kind": "execute_command", "argv": ["/bin/echo", "hi"], "cwd": "."}],
        "entry_criteria": [], "exit_criteria": [],
        "settlement_criteria": {"require_exit_zero": True},
        "item_kind": "sandboxed_task",
        "markers": {"required": True},
        "max_instances": 1, "depends_on": [],
    }],
}
plan_path = os.path.join(CELL, "plan.json")
open(plan_path, "w").write(json.dumps(plan))

print("== submit (deny policy) ==")
r = c.call(verb="submit", plan=plan_path, policy=policy,
           entity="operator_local", process="journey", timeout=30)
print(json.dumps(r)[:400])
case_id = r.get("case_id")

print("== run_list ==")
rl = c.call(verb="run_list")
runs = rl.get("runs", [])
print(f"{len(runs)} run(s):", [x.get("run_id") for x in runs])

for entry in runs:
    rid = entry["run_id"]
    print(f"== run_get {rid} ==")
    rec = c.call(verb="run_get", run_id=rid)
    if "error" in rec:
        print("  ERROR:", json.dumps(rec)[:200])
        continue
    print("  settlement:", json.dumps(rec.get("settlement"))[:200])
    print("  records present:",
          [x.get("name") for x in rec.get("records", []) if x.get("present")])

print("== case_get_overview ==")
ov = c.call(verb="case_get_overview", case_id=case_id)
print(json.dumps({k: ov[k] for k in ("case_id", "state", "settlements") if k in ov})[:400])

print("== filesystem: did the denial create a workspace? ==")
runs_root = os.path.join(CELL, "cases", case_id, "runs")
for rid in sorted(os.listdir(runs_root)):
    d = os.path.join(runs_root, rid)
    print(f"  {rid}: {sorted(os.listdir(d))}")
