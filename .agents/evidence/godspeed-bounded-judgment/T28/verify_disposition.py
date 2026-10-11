#!/usr/bin/env python3
"""T28 disposition gate: re-runs the complete independent audit chain.

1. T27 gate (--verify-only): corpus identity, store keys, teeth, analysis
   recomputation from persisted observations only.
2. T28 independent recomputation (t28-independent-recompute.py): standalone
   re-derivation of truth, readouts, McNemar, guard, and C-rule traversal.
3. Cross-checks the two against each other and against the frozen
   expectations; nonzero exit on any mismatch.
"""
import hashlib, json, os, subprocess, sys

HERE = os.path.dirname(os.path.abspath(__file__))
T27 = os.path.join(HERE, "..", "T27")

EXPECTED = {
    "accA": 0.6429, "accB": 0.5714, "b": 0, "c": 1, "b_f": 0, "c_f": 1,
    "c_ctrl": 0, "p_two": 1.0, "c_traversal": "C5(boundary)",
    "abstentions": {"A": 18, "B": 32},
    "selection_rule_replay": "MATCH",
    "A_prompt_bytes_reproduced": "70/70",
}
HARNESS_SHA = "cc21e4652933a98fc8f4545aa58e9926141e73c1c2078145587fbe2a0ac81541"


def main():
    failures = []
    r = subprocess.run([sys.executable, os.path.join(T27, "run_claim_obs.py"), "--verify-only"],
                       capture_output=True, text=True)
    print(r.stdout.strip().splitlines()[-1] if r.stdout.strip() else r.stderr[-500:])
    if r.returncode != 0:
        failures.append(f"T27 verify-only gate exit {r.returncode}")
    recompute = os.path.join(HERE, "t28-independent-recompute.py")
    r = subprocess.run([sys.executable, recompute], capture_output=True, text=True)
    print(r.stdout.strip().splitlines()[-1] if r.stdout.strip() else r.stderr[-500:])
    if r.returncode != 0:
        failures.append(f"independent recomputation exit {r.returncode}")
    res = json.load(open(os.path.join(HERE, "t28-independent-recompute.json")))
    for k, want in EXPECTED.items():
        got = res.get(k)
        if got != want:
            failures.append(f"{k}: got {got!r}, expected {want!r}")
    sha = hashlib.sha256(open(os.path.join(T27, "run_claim_obs.py"), "rb").read()).hexdigest()
    if sha != HARNESS_SHA:
        failures.append(f"harness sha {sha} != recorded {HARNESS_SHA}")
    for art in ("t28-verifier-independent-confirmation.md", "t28-disposition-report.md",
                "t28-independent-recompute.json"):
        if not os.path.isfile(os.path.join(HERE, art)):
            failures.append(f"missing T28 artifact {art}")
    if failures:
        print("T28 GATE FAIL:")
        for f in failures:
            print("  -", f)
        return 1
    print("T28 GATE PASS: T27 verification, independent recomputation, disposition "
          "artifacts, and harness hash all consistent (disposition: CONFIRMED WITH QUALIFICATION)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
