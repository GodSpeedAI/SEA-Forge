# Remeasurement report — learned judgment on the native route-discovery corpus

Measurement: `remeasure-native-corpus` (operator-directed developmental measurement, OUTSIDE the settled plan).
Prereg: `.agents/preregistrations/godspeed-bounded-judgment-native-corpus-remeasure.prereg.yaml` sha256 `8142c0a52537f9b600d4853afd4871d5ec04dc29acffba58fe67c7f7616d1f6e` (frozen before the first scored call).
Corpus manifest: `.agents/evidence/godspeed-bounded-judgment/remeasure-native-corpus/corpus-manifest.yaml` sha256 `e2419c2ccd79cd84761354471a796141196a85b8ffe34322531339b74220be36`; frozen under decision D-2026-09-19-RM-02 BEFORE the first call (two-stage freeze, both append-only).

## 1. Frozen corpus identity

- 15 consequence-backed rows from the settled native route-discovery branch; every row is a real gauntlet run on the corrected fixture mechanism with a pinned state DB (15/15 DB pins re-verified at every pass start, fail-closed).
- Frozen class counts: 6 settled_accepted / 5 budget_exhausted / 4 verification_failed_nonsettlement; truth derived mechanically (settled terminal -> settled_accepted; addendum-3 clause conjunction -> verification_failed_nonsettlement; other bounded non-settlement -> budget_exhausted); the provider never sees truth.
- Families: 11 honest rows, 4 adversarial (e2e-lying) rows. 12 route rows carry preserved T23 round-2 labels; 3 rows (nat-13/14/15) are new-corpus rows with no historical label.
- Sources: T23 route iterations 1-4 probe records, T24 accepted-case replay + T7 impostor tooth (provenance labeled), T25 matched negative.

## 2. Provider, model, protocol

- Scored provider: deepseek-clinepass (`cline-pass/cline-pass/deepseek-v4.1-flash`) via the frozen T16 `call_provider` seam; single-provider measurement.
- Substitution policy: ONE muse-spark-free (`opencode/muse-spark-1.3-contributor-free`) retry per provider_timeout/provider_error, recorded per call; substituted rows form a separate provider-qualified slice, never pooled. No re-rolls: a preserved raw output is reused verbatim.
- arm1: exact T16-comparable predictive arm (pre-terminal records, terminal/settlement events excluded and asserted; T16 PROMPT verbatim, parse verbatim; 1 sample/row).
- arm2: classification on the frozen T23 round-2 enriched observation surface (builder imported unchanged; prompt = the pinned round-2 template, only the records blob substituted; 3 samples/row; answer domain stated without label semantics).
- arm2b (exploratory): one call/row extending the arm2 template with `settled_units_estimate` (integer or null).

## 3. Execution passes, interruption recovery, deviations

- round-1 (run_arms.py): arm1 executed to completion (15/15 deepseek calls, raw outputs preserved under arm1/provider-raw/, payment wall time recorded); the pass then crashed at the first arm2 prompt write, before any arm2/arm2b provider call (DV-4)
- round-2 (run_arms.round2.py, this pass): arm1 records carried over from the preserved round-1 file after re-validation against the preserved raw outputs; arm2/arm2b execute fresh
- round-3 (run_arms.round3.py, resume of the interrupted round-2 pass): preserved arm2 raw outputs re-used verbatim, never re-rolled (including the completed muse substitution of nat-02-s2); the remaining arm2 calls and all arm2b calls execute fresh under the frozen substitution policy; results file updated in place (DV-5)
- Round-3 resume discipline: the 14 completed round-1/round-2 arm2 call slots (including nat-02-s2, whose deepseek leg timed out and whose single muse substitution completed) were re-validated from disk and reused verbatim; a naive re-run of the round-2 runner would have re-rolled that deepseek leg. Deviations recorded: DV-1 (scope-reading), DV-2 (most-restrictive-reading), DV-3 (scope-reading), DV-4 (execution-interruption-recovery), DV-5 (execution-interruption-recovery). DV-5: the two nat-05 slots completed just before the interruption carry no wall-time metadata (marked `duration_lost_in_interruption`); arm2 wall time is therefore an honest lower bound.

## 4. Baseline (face-value, zero calls)

- Rule: face-value: all result documents claim completed -> settled (T16 rule verbatim, same mapped truth) (baseline never reads artifact contents).
- Scored on the arm1 deepseek slice against the same mapped truth: 9/15 = 0.600, CP 95% CI [0.323, 0.837]. The mechanical consequence layer itself derives truth on this corpus by construction and requires zero provider calls.

## 5. arm1 — predictive arm (exact T16 comparability)

- Deepseek slice (all 15 rows): 9/15 answered-correct = 0.600, CP 95% CI [0.323, 0.837]; abstentions 2 (nat-07, nat-09), typed rejections 0, contract failures 0.
- Families: honest 6/11 = 0.545 [0.234, 0.833]; adversarial 3/4 = 0.750 [0.194, 0.994].
- Confusion (pred -> truth): settled -> settled 6, settled -> not_settled 4 (face-value optimism: nat-02, nat-11, nat-14, nat-15 predicted settled), not_settled -> not_settled 3, insufficient_evidence -> not_settled 2.
- Errors vs mapped truth: nat-02 (settled); nat-07 (insufficient_evidence); nat-09 (insufficient_evidence); nat-11 (settled); nat-14 (settled); nat-15 (settled).

## 6. arm2 — discrimination arm (frozen enriched surface)

- Single-sample (sample 1), deepseek slice (13 rows; nat-02 and nat-14 each had one substituted sample and are excluded here, never pooled): 9/13 = 0.692, CP 95% CI [0.386, 0.909]. Abstentions 0, typed rejections 0, contract failures 0 (the provider always answered in-domain).
- Majority over 3 samples (>=2 of 3 cast votes needed; ties/<2 parseable -> insufficient_evidence, DV-2): 9/13 = 0.692, CP 95% CI [0.386, 0.909].
- Families (single and majority identical): honest 9/9 = 1.000 [0.664, 1.000]; adversarial 0/4 = 0.000 [0.000, 0.602].
- Per-class confusion (identical for single and majority): settled_accepted -> settled_accepted 6/6; budget_exhausted -> budget_exhausted 3/5; verification_failed_nonsettlement -> predicted budget_exhausted 4/4. The learned layer NEVER produces verification_failed_nonsettlement as a majority label on this corpus.
- Stability: per-row modal agreement mean 0.974; 12/13 rows unanimous; the only non-unanimous row is nat-09 (2/3).
- Substituted slice (provider-qualified, never pooled; rows nat-02, nat-14): single sample-1 1/2; majority 2/2 (in nat-14 the two deepseek samples outvoted the one muse sample).

## 7. Case-level error patterns (comparison 7 — every disagreement with mechanical truth)

Scored deepseek slice: the only error pattern is total collapse of verification_failed_nonsettlement into budget_exhausted (4 rows, unanimous in 3 of 4):

- nat-03: truth verification_failed_nonsettlement, majority budget_exhausted.
  - sample 1 (deepseek-clinepass): `{"label": "budget_exhausted"}`
  - sample 2 (deepseek-clinepass): `{"label": "budget_exhausted"}`
  - sample 3 (deepseek-clinepass): `{"label": "budget_exhausted"}`
- nat-05: truth verification_failed_nonsettlement, majority budget_exhausted.
  - sample 1 (deepseek-clinepass): `{"label": "budget_exhausted"}`
  - sample 2 (deepseek-clinepass): `{"label": "budget_exhausted"}`
  - sample 3 (deepseek-clinepass): `{"label": "budget_exhausted"}`
- nat-09: truth verification_failed_nonsettlement, majority budget_exhausted.
  - sample 1 (deepseek-clinepass): `{"label": "budget_exhausted"}`
  - sample 2 (deepseek-clinepass): `{"label": "verification_failed_nonsettlement"}`
  - sample 3 (deepseek-clinepass): `{"label": "budget_exhausted"}`
- nat-10: truth verification_failed_nonsettlement, majority budget_exhausted.
  - sample 1 (deepseek-clinepass): `{"label": "budget_exhausted"}`
  - sample 2 (deepseek-clinepass): `{"label": "budget_exhausted"}`
  - sample 3 (deepseek-clinepass): `{"label": "budget_exhausted"}`

The error is interpretive, not observational. Example (nat-03 surface, preserved at `arm2/surfaces/nat-03.surface.json`): the surface already carries every addendum-3 clause input — `typed_terminal.terminal_class: "budget_exhausted"` (the raw terminal state), `settlement_committed_count: 0`, and the sha256-verified failing artifact with content `{"tests": 1, "failures": 1, "command": "scripts/check.sh --marker"}` — and the model still answers budget_exhausted in 11 of 12 samples across the four vfn rows. It anchors on the `terminal_class` string instead of applying the clause rule that the surface's own fields enable.

Beside the scored slice (substituted, never pooled): nat-14 sample 1 (muse) answered `{"label": "settled_accepted"}` against truth budget_exhausted; the row's two deepseek samples were correct and the majority recovered.

## 8. Statistical comparisons

- arm1 learned vs face-value baseline (paired rows, McNemar exact): learned right/baseline wrong 6, learned wrong/baseline right 6, p = 1.000. The learned layer shows NO significant advantage over a rule that never reads artifact contents and costs zero calls.
- arm2 single vs majority: identical accuracy (0.692 vs 0.692); rows changing label under aggregation: 0.
- arm1 (0.600, pre-terminal view) vs arm2 (0.692, enriched surface) are descriptive only (overlapping CIs; different row slices); the enrichment's real effect is the error-mode shift: face-value optimism disappears (honest rows 9/9) but class separation within non-settlement never appears.

## 9. Does majority aggregation materially change arm2?

No. On the scored deepseek slice, aggregation changes zero row labels and leaves accuracy, CIs, confusion and family splits identical (9/13 either way). The sole visible effect is inside the never-pooled substituted slice (nat-14: 2 deepseek votes outvote 1 muse vote). With 12/13 rows already unanimous, a third sample buys nothing — aggregation is not the deficiency.

## 10. Freshness vs the preserved T23 round-2 labels (no pooling)

- 12 route rows carry preserved round-2 labels; the fresh majorities match on 11/12. fresh labels are reported beside the preserved round-2 labels; no pooling in either direction.
- nat-01: preserved settled_accepted | fresh settled_accepted (match; fresh samples settled_accepted, settled_accepted, settled_accepted).
- nat-02: preserved budget_exhausted | fresh budget_exhausted (match; fresh samples budget_exhausted, settled_accepted, budget_exhausted).
- nat-03: preserved budget_exhausted | fresh budget_exhausted (match; fresh samples budget_exhausted, budget_exhausted, budget_exhausted).
- nat-04: preserved settled_accepted | fresh settled_accepted (match; fresh samples settled_accepted, settled_accepted, settled_accepted).
- nat-05: preserved budget_exhausted | fresh budget_exhausted (match; fresh samples budget_exhausted, budget_exhausted, budget_exhausted).
- nat-06: preserved settled_accepted | fresh settled_accepted (match; fresh samples settled_accepted, settled_accepted, settled_accepted).
- nat-07: preserved budget_exhausted | fresh budget_exhausted (match; fresh samples budget_exhausted, budget_exhausted, budget_exhausted).
- nat-08: preserved settled_accepted | fresh settled_accepted (match; fresh samples settled_accepted, settled_accepted, settled_accepted).
- nat-09: preserved verification_failed_nonsettlement | fresh budget_exhausted (DIFFERS; fresh samples budget_exhausted, verification_failed_nonsettlement, budget_exhausted).
- nat-10: preserved budget_exhausted | fresh budget_exhausted (match; fresh samples budget_exhausted, budget_exhausted, budget_exhausted).
- nat-11: preserved budget_exhausted | fresh budget_exhausted (match; fresh samples budget_exhausted, budget_exhausted, budget_exhausted).
- nat-12: preserved settled_accepted | fresh settled_accepted (match; fresh samples settled_accepted, settled_accepted, settled_accepted).
- The single differ row is nat-09 — the same row that is the only non-unanimous fresh row (2/3). The round-2 failure mode (vfn read as budget_exhausted) reproduces; where round-2 once produced the correct vfn label (nat-09), the fresh replication reverted to budget_exhausted.

## 11. arm2b — exploratory graded progress (comparison 8)

- Deepseek slice (14 rows; nat-03's slot took the one muse substitution after a 300 s deepseek timeout and is excluded, never pooled): 14/14 rows produced valid integer estimates; MAE 0.000; exact hits 14/14 = 1.000, CP 95% CI [0.768, 1.000].
- Honest-family slice: 11/11 exact, MAE 0.000, CI [0.715, 1.000]. (The excluded muse row, nat-03, also estimated 0 exactly — reported beside, not pooled.)
- Interpretation under the frozen frame: `settlement_committed_count` is itself a field of the enriched surface, so the estimates restate surface content. The probe demonstrates copy/instruction fidelity (every answered row exact), NOT an independent graded signal beyond the mechanical layer.

## 12. Unique learned-layer contribution

None demonstrated. Classification adds no separation the mechanical rule does not already derive for free (it cannot produce verification_failed_nonsettlement); the arm where it is perfect (honest rows, arm2) is exactly the arm where the surface's typed fields already determine the answer; arm2b restates a surface field. The only reliability structure worth recording is the error asymmetry itself: zero false alarms on honest rows (9/9) with total blindness to verification-failed non-settlement — a bias profile, not a capability.

## 13. Payment added by the learned layer

- Provider calls: arm1 15 deepseek (0 failed, 0 substitutions); arm2 43 deepseek + 2 failed (300 s timeouts: nat-02-s2, nat-14-s1) + 2 muse substitutions = 47 attempts; arm2b 14 deepseek + 1 failed (nat-03) + 1 substitution = 16 attempts.
- Learned-layer total: 78 provider calls; recorded wall time 504.485 s (arm1) + 1866.356 s (arm2, lower bound per DV-5) + 846.912 s (arm2b) = 3217.753 s. Mechanical consequence layer: 0 calls.

## 14. Architectural consequence

The learned judgment layer earns no role in the consequence/settlement path on this corpus. It is dominated by the mechanical layer on every axis: the mechanical rule derives truth exactly, for zero calls, with no timeouts and no substitutions; the learned layer's single paid distinction (settled vs not) matches the trivial baseline (McNemar p = 1.0), and its enriched-surface judgment cannot separate the two non-settlement classes even though every clause input is on the surface. Authority over consequence classification stays with the mechanical layer; no learned output is admitted into settlement-relevant state.

## 15. Developmental consequence

The failure is label-anchoring, not missing information: the model repeats `typed_terminal.terminal_class` instead of composing the addendum-3 clauses from fields it can already see (`failures: 1` in the artifact content, `settlement_committed_count: 0`). If vfn separation from a learned layer is ever wanted, the lever is surface/label presentation, not aggregation (section 9) and not more samples (stability is already 0.974). Any such work is a revise-surface probe, kept outside the settlement path.

## 16. Limitations

- N=15, single provider (deepseek-clinepass), single case family; 3 provider timeouts substituted once each per the frozen policy; substituted slices reported separately and never pooled (n=2 arm2 rows, n=1 arm2b row).
- CIs are wide (e.g. arm2 single 0.692, CI [0.386, 0.909]); the measurement separates the vfn-collision pattern and the baseline parity, not fine accuracy differences.
- arm2b is exploratory and excluded from the primary verdicts; its exactness reflects surface restatement.
- The measurement ran across an interrupted round-2 pass and a round-3 resume; recovery discipline (no re-rolls, in-place results, DV-4/DV-5) is recorded, and arm2 wall time is a lower bound.
- 4 lying rows and 8 happy rows were previously judged in T23 round-2 under the same surface; this is a fresh frozen measurement; historical labels are compared beside, never pooled.

## 17. Recommendation and exact next affordable move

Recommendation (prereg vocabulary): **remove-from-path** — the learned judgment layer is removed from the consequence/settlement path (it never enters settlement-relevant state); no promotion question is asked and the ruler is untouched. Retain the apparatus as a developmental instrument only.

Exact next affordable move (only if a learned use for non-settlement separation is ever wanted): a 12-call revise-surface probe on the 4 vfn rows x 3 samples with `typed_terminal` masked to a non-committal `bounded_non_settlement` (variant B, tests the demonstrated label-anchoring hypothesis directly), then — only if variant B separates — the additive variant A (explicit typed `failing_artifact_verdict` field). Stop at variant B if it does not separate.

## 18. Verification

Closure gate: `verify_remeasure.py` (R1-R10: DB pin re-hashes; mechanical truth re-derivation; every preserved raw output re-parsed; arm1 leakage assertions; template/surface re-derivation; scoring/CIs/majority/stability/payment recomputation; RM-02 ordering; evidence-manifest hash checks). Gate outcome is recorded in `b1-result.json`; exit 0 from the sea-rs repo root is the settlement condition.

