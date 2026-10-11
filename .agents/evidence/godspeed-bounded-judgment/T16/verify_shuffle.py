#!/usr/bin/env python3
"""T16 shuffle tooth (plan task T16 teeth; frozen before its result was observed).

Declared expected behavior: 'Shuffle consequence targets and rescore.
Agreement collapses; a surviving high score fails the task.'

Method: permutation test over the frozen truth labels — 500 random shuffles
of the consequence labels across the nine corpus rows, re-scoring the
PERSISTED provider answers (never re-called) against each shuffled labeling.
The recorded accuracies are real only if they exceed what label noise
produces (we report mean/max/p95 per provider and require the observed
accuracy to exceed the shuffled maximum). Case/settlement identity is
preserved: answers stay bound to their own rows; only the truth column is
permuted, and any (answer, truth) association an individual row still sees
is the permutation's, never a hand-paired mismatch.
"""
from __future__ import annotations
import json, random, sys

RAW = ".agents/evidence/godspeed-bounded-judgment/T16/round-1/provider-raw-results.json"
N_SHUFFLES = 500

def main() -> int:
    data = json.load(open(RAW))
    rows = data["rows"]
    # truth labels live in the manifest's recorded consequence classes,
    # projected by the frozen mapping (budget_exhausted -> not_settled)
    manifest = __import__("yaml").safe_load(
        open(".agents/preregistrations/godspeed-bounded-judgment-T15.corpus-manifest.yml"))
    consequence = {r["case_id"]: r["consequence_class"] for r in manifest["runs"]}
    truth = [{"settled": "settled", "budget_exhausted": "not_settled"}[consequence[r["case_id"]]]
             for r in rows]
    out = {"tooth": "shuffle", "n_shuffles": N_SHUFFLES, "providers": {}}
    ok = True
    rng = random.Random(20260918)  # fixed seed: reproducible, frozen before the run
    for provider in ("deepseek-clinepass", "prime-agent"):
        answers = [r.get("answer") for r in data["results"][provider]]
        answered = [(i, a) for i, a in enumerate(answers) if a]
        observed = sum(1 for i, a in answered if a == truth[i]) / len(answers)
        shuffled_accs = []
        for _ in range(N_SHUFFLES):
            perm = truth[:]
            rng.shuffle(perm)
            acc = sum(1 for i, a in answered if a == perm[i]) / len(answers)
            shuffled_accs.append(acc)
        mean_s = sum(shuffled_accs) / len(shuffled_accs)
        max_s = max(shuffled_accs)
        srt = sorted(shuffled_accs)
        p95 = srt[int(0.95 * (len(srt) - 1))]
        collapses = observed > max_s
        out["providers"][provider] = {
            "observed_accuracy": observed,
            "shuffled_mean": mean_s, "shuffled_max": max_s, "shuffled_p95": p95,
            "collapses_below_observed": collapses,
        }
        ok = ok and collapses
        print(f"[shuffle:{provider}] observed={observed:.3f} shuffled_mean={mean_s:.3f} "
              f"max={max_s:.3f} p95={p95:.3f} collapses={collapses}")
    out["behaved_as_specified"] = ok
    out["declared_expected"] = "Agreement collapses; a surviving high score fails the task."
    with open(".agents/evidence/godspeed-bounded-judgment/T16/round-1/tooth-shuffle.yml", "w") as f:
        json.dump(out, f, indent=1)
    print(f"[shuffle] behaved_as_specified={ok}")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
