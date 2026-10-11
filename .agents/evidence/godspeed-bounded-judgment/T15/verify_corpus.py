#!/usr/bin/env python3
"""T15 corpus verifier: mechanism provenance + class coverage."""
import json, sqlite3, sys, yaml
m = yaml.safe_load(open(".agents/preregistrations/godspeed-bounded-judgment-T15.corpus-manifest.yml"))
errors = []
classes = {}
for run in m["runs"]:
    conn = sqlite3.connect(f"file:{run['state_db_path']}?mode=ro", uri=True)
    term = conn.execute("select payload from event_log where kind='run_terminal'").fetchall()
    if len(term) != 1:
        errors.append(f"{run['case_id']}: no single run_terminal")
        continue
    cls = json.loads(term[0][0])["state"]
    if cls != run["consequence_class"]:
        errors.append(f"{run['case_id']}: manifest class {run['consequence_class']} != observed {cls}")
    settled_events = conn.execute("select count(*) from event_log where kind='settlement_committed'").fetchone()[0]
    if cls == "settled" and settled_events == 0:
        errors.append(f"{run['case_id']}: settled without settlement_committed events")
    classes[cls] = classes.get(cls, 0) + 1
if len(classes) < 2:
    errors.append(f"single-class corpus: {classes}")
print("classes:", classes)
if errors:
    print("verify_corpus: FAIL"); [print("-", e) for e in errors]; sys.exit(1)
print("verify_corpus: PASS")
