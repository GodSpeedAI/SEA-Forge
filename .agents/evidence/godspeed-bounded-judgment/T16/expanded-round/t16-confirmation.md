# T16 Expanded-Corpus Independent Adversarial Confirmation
- Date: 2026-09-18
- Verifier: fresh independent context; builder narrative excluded
- **Verdict: CONFIRM**

The verifier independently recomputed every number from the persisted raw results: accuracy 0.833/0.28, abstention 0.12/0.68, entropy 0.779/0.363 nats, disagreement 3/9, confusion matrices (all exact float matches). The exact permutation tooth was independently computed via the closed-form hypergeometric shortcut over C(25,12) = 5,200,300: DeepSeek p = 49/5,200,300 = 9.4e-6; Muse p = 204,204/5,200,300 = 0.039. The frozen D-rule (conjunctive) is unmet → no promotion → letter C is correct. Scope discipline confirmed.

Seven defects found, none outcome-changing:
1. Provenance break: the round-6 harness sha (e7aed0f4) matches no on-disk artifact (untracked; the actual generating script had ea26d0b9)
2. DeepSeek was re-run contradicting D-T16-16's "NOT re-run" freeze — but the fresh re-sample independently reproduced 20/25, strengthening the finding
3. Report says "Muse has 24 contract failures" (stale prime-agent text; Muse had 0)
4. Disclosure block stale ("3 families" / "run_count 3"; actual: 4 families / 25 rows)
5. The missed settled row is accepted-A11 (timeout), not "a lying-builder row"
6. Muse's "within noise band" overstates (p = 0.039 < 0.05; "at chance" is defensible)
7. 7 DeepSeek rows have confidence ≠ distribution[answer] by 0.01-0.08 (provider rounding, schema-valid)

Items 1-4 are provenance/text hygiene (do not affect any number or the outcome letter); items 5-7 are descriptive corrections. All recorded in the decision log (D-2026-09-18-T16-18).
