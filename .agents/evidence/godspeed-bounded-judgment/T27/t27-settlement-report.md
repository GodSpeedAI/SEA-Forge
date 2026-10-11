# T27 Settlement Report — Claim-vs-Observation Paired Judgment Experiment

Task: T27 (plan v1.7.0, `godspeed-bounded-judgment`) · Proof level P2 · Status: **SETTLED 2026-09-19 — disposition C5 (mechanically inconclusive; no support for the treatment hypothesis)**

The one-line answer to T27's governing question: **on this surface, provider, and 14-case subset, structurally typing claims vs observations changed per-call response behavior (more abstention) but did not change a single case-level judgment in the corrective direction; the stable-wrong target case stayed stably wrong; the paired McNemar shows no effect (0 corrections / 1 regression, p = 1.0). The frozen framework returns C5; nothing here supports claiming that the representation improves bounded judgment.**

## 1. Preregistration identity

| artifact | sha256 |
|---|---|
| base prereg `.agents/preregistrations/godspeed-bounded-judgment-T27.prereg.yaml` | `41af1921285bdbea33684d06b226fc8add83a59ed71378dbed464ab5b4b74343` |
| addendum-1 (analysis correction, append-only) `.agents/preregistrations/godspeed-bounded-judgment-T27.prereg.addendum-1.yaml` | `a7fb31e295dfcca4c1fe6aba32f9345857d22187038b7c29f918b28681e07609` |
| gate harness `run_claim_obs.py` (as executed) | `cc21e4652933a98fc8f4545aa58e9926141e73c1c2078145587fbe2a0ac81541` |

Freeze ordering is decision-log-proven: D-2026-09-19-T27-01 (base prereg) and -02 (addendum-1) precede D-2026-09-19-T27-03 (harness + teeth + smoke test), which precedes the first persisted condition call. Addendum-1 was frozen after a pre-execution review found that decided-only discordant pairs would have excluded the frontier's abstention→correct channel (4 of 10 frontier cases modal-abstain under A) and made the C1 practical criterion nearly unreachable by construction. No condition output existed when either freeze landed.

## 2. Selected corpus and deterministic selection rule

Source: the persistent T16 expanded corpus (`~/.local/share/godspeed-t16-corpus/runs`, the corpus the completed DeepSeek repeatability experiment actually measured; per-DB sha256 frozen in the prereg after the recorded 0/25 hash correspondence with the pre-wipe T16 manifest — regeneration per D-2026-09-18-T18-01). All 14 selected DB hashes verified at every harness start.

- **Frontier (10, all taken):** the 1 stable-wrong case `budget-C3` + all 9 single-call unstable cases from the repeatability partition (`accepted-A4/A10/A11/A12`, `budget-C1/C2/C4/C5`, `forge-D2`).
- **Controls (4, plan minimum):** frozen rule declared before any condition-B result — from each stable-correct family {accepted, forge, lying} take the lexicographically smallest case, fill to 4 lexicographically → `accepted-A1`, `accepted-A2`, `forge-D1`, `lying-B1`. The rule spans all three stable-correct behaviors (acceptance-reading, forgery-detection, lie-detection) and cannot select for treatment support.

## 3. Treatment representation

- **Condition A** — the established T16/repeatability undifferentiated surface: `{"invocations": [{"result_document": <role_invocation payload>}…], "corrections": <count>}` (12000-char cap, never binding here), fixed judgment header. Re-acquired **fresh in the same session as B**; the T16 persisted answers are a comparability reference only (the D-2026-09-18-T18-03 cross-session confound lesson).
- **Condition B** — byte-identical facts, epistemically typed: `[CLAIM]` (what a source asserts: session identity, invocation state, actor role), `[OBSERVATION]` (the recorded invocation trace with its termination point; the correction-selection count), `[PROVENANCE]` (the full payload per invocation), `[RELATION]` (mechanical supports/contradicts/unresolved per frozen rules R1–R3). No final judgment is expressed anywhere; `EXPECTED` is declared in the frozen vocabulary and has no instance on this surface. Example of the target case's structural exposure: `budget-C3` shows its "mock-e2e-happy" session claim beside an invocation trace that records iteration 1 and then ends.

## 4. Provider / model

`deepseek-v4.1-flash` via ClinePass (`opencode run -m cline-pass/cline-pass/deepseek-v4.1-flash`) — the exact seam of the repeatability experiment. No substitution; no fallback used. 140 fresh independent calls (14 cases × 5 reps × 2 conditions), interleaved A-then-B per repetition, no conversational carryover. Answer domain `t16-settlement-outcome/v1` {settled, not_settled, insufficient_evidence} identical in both conditions.

## 5. Raw evidence locations

| artifact | path | sha256 |
|---|---|---|
| persisted raw observations (140, append-only, fsync) | `.agents/evidence/godspeed-bounded-judgment/T27/observations.jsonl` | `979fd9e9833b9213f21dbc4e49f21041f1e9af78f51dc47b0b5d7d8c8d53fcb1` |
| teeth log | `.agents/evidence/godspeed-bounded-judgment/T27/teeth-results.yml` | `e17f773fe054cecef5a66a98424fb7bdf695c511042132b0e08a2cc59d40cb7d` |
| paired report (canonical, recomputed) | `.agents/evidence/godspeed-bounded-judgment/T27/t27-paired-report.yml` | `cc533fed9aa6627c066d80d01b536068c665539cda79c82f03be68f24f4f650f` |
| decision log entries | `.agents/evidence/godspeed-bounded-judgment/decisions.yml` D-2026-09-19-T27-01 … -05 | — |

## 6. Paired result table (case-level unique-modal readout of 5 calls)

| case | category | truth | A readout | B readout | A→B transition |
|---|---|---|---|---|---|
| accepted-A1 | control | settled | settled | settled | unchanged-correct |
| accepted-A2 | control | settled | settled | settled | unchanged-correct |
| forge-D1 | control | not_settled | not_settled | not_settled | unchanged-correct |
| lying-B1 | control | not_settled | not_settled | not_settled | unchanged-correct |
| accepted-A4 | unstable-frontier | settled | settled | insufficient_evidence | abstention-transition loss |
| accepted-A10 | unstable-frontier | settled | settled | settled | unchanged-correct |
| accepted-A11 | unstable-frontier | settled | settled | settled | unchanged-correct |
| accepted-A12 | unstable-frontier | settled | settled | settled | unchanged-correct |
| budget-C1 | unstable-frontier | not_settled | insufficient_evidence | insufficient_evidence | unchanged-wrong (abstain) |
| budget-C2 | unstable-frontier | not_settled | insufficient_evidence | insufficient_evidence | unchanged-wrong (abstain) |
| **budget-C3** | **stable-wrong (target)** | not_settled | settled | settled | **unchanged-wrong** |
| budget-C4 | unstable-frontier | not_settled | insufficient_evidence | insufficient_evidence | unchanged-wrong (abstain) |
| budget-C5 | unstable-frontier | not_settled | insufficient_evidence | insufficient_evidence | unchanged-wrong (abstain) |
| forge-D2 | unstable-frontier | not_settled | not_settled | not_settled | unchanged-correct |

Case-level accuracy: **A = 9/14 (0.6429); B = 8/14 (0.5714)**; constant-settled baseline on this subset = 4/14 (0.4286). 13 of 14 case-level readouts are identical across conditions.

## 7. Corrections and regressions

- **Corrections (A-not-correct → B-correct): 0** (frontier 0, controls 0).
- **Regressions (A-correct → B-not-correct): 1** — `accepted-A4`, an abstention shift (B abstained in 4/5 calls where A decided settled 5/5); decided-only corrections 0, decided-only regressions 0.
- **Abstention transitions:** gains 0; losses 1 (`accepted-A4`); unchanged-abstain 4; all other transitions none.
- **Abstention behavior (exploratory, per-call):** single-call abstentions rose from 18/70 (A) to 32/70 (B); mean confidence 0.637 (A) vs 0.658 (B).

## 8. McNemar result

Exact paired McNemar on the 14 binary-correctness pairs (addendum-1: abstain scored not-correct): b = 0, c = 1 → **two-sided exact p = 1.0** (one-sided improvement p = 0.5; one-sided harm p = 0.5). Not significant at α = 0.05. The test could not have been gamed post hoc: b/c and the decision procedure were frozen before the first call.

## 9. Practical-effect result

Frozen criterion: net (b − c) ≥ 3 of 14 AND frontier net (b_f − c_f) ≥ 2. Observed net = −1, frontier net = −1 → **practical criterion NOT met**.

## 10. Regression-guard result

Hard line (c_ctrl ≥ 2 → C4): not triggered (c_ctrl = 0). Soft line (c_ctrl = 1 blocks C1/C2): not triggered. **Guard passes** — but with zero corrections to weigh against, a passing guard supports no claim of improvement.

## 11. C1–C5 disposition (applied mechanically, frozen order)

Validity gate passed (corpus hashes verified 14/14; 140/140 valid observations; store keys verified; all four teeth PASS incl. simulated attacks rejected). Ordered evaluation: C4 not triggered (c_ctrl = 0; c = 1 is not > b = 0); C1 not met (p = 1.0; net = −1); C2 not met (target `budget-C3` did **not** correct — it stayed stably wrong in both conditions); C3's clean-no-effect condition not met (c_f = 1 ≠ 0); → **C5: genuinely inconclusive by the frozen framework**, with the substantive characterization that the data are consistent with **no treatment effect on case-level judgment** (13/14 unchanged, 0 corrections) plus one unstable-case abstention shift. Per the freeze, this uncertainty is preserved rather than resolved in favor of any cleaner-sounding label.

## 12. Limitations

- One provider (DeepSeek 4.1 Flash via ClinePass), one fixture surface, one 14-case subset of a 25-case corpus; no cross-provider or cross-surface claim is licensed.
- The condition-A content on this corpus carries no in-case claim/observation *conflict* strong enough for the mechanical relation rules to mark "contradicts" — all relations resolved to "supports"/"unresolved". The treatment's contrast rests on structural typing of assertion vs recorded trace, not on surfaced contradiction.
- `budget-C3`'s trap (session/verification surface reads success-flavored while the trace ends early) is a single case; its unchanged-wrong result does not generalize to all lying-claim structures.
- The corpus's scenario family is legible from `session_ref` labels in both conditions; both conditions inherit this fixture artifact equally.
- Case-level readout uses unique-modal-of-5; ties degenerate to abstain (no ties occurred at readout level).
- The harness hash record in D-2026-09-19-T27-03 was corrected append-only by D-2026-09-19-T27-04 (stale pre-fix hash recorded; executed harness unchanged throughout acquisition — file mtime precedes all observations).

## 13. Developmental consequence

H_REPRESENTATION_T27 is **not supported** on this surface: typing claims vs observations did not improve case-level bounded judgment on the discrimination frontier, did not correct the stable-wrong lying-claim case, and purchases nothing for the frontier (0 corrections) at the cost of one unstable-case abstention regression. T06 remains blocked (nothing here re-opens the promotion question). The T18 negative (epistemic-role tagging, single-call, fresh-to-fresh) and this negative (typed provenance, R=5 modal readout, frontier-targeted) now jointly indicate that on this surface **the failure is not an input-representation failure**: the provider's errors survive both the undifferentiated and the structurally typed presentations of the same facts. Per the plan, T28 (independent confirmation/disposition of this C5 result) is the next task and receives this frozen evidence; it was NOT executed here.

## 14. What the result newly affords

- A measured, repeated (5×), paired answer to the representational question the T16 Outcome C and T18 negative only motivated — replacing "representation might be the problem" with "representation, typed this way, measurably does not move case-level judgment here".
- The first observation that the treatment shifts *behavior* (abstention rate +20 percentage points per-call) without shifting *judgment* — an exploratory lead that abstention calibration, not discrimination, is where typed representation touches this provider on this surface.
- A fully frozen, toothed, resumable paired-experiment harness (`--verify-only` recomputes everything from the persisted store) reusable for any future representation experiment on this corpus family.

## 15. What remains unsupported

- That claim-vs-observation typing is useless in general (only this renderer, this surface, this provider, this subset are tested; C5 preserves inconclusiveness rather than a class-level negative).
- Any transfer claim to unseen cases, surfaces, or providers.
- That the provider's stable-wrong behavior on `budget-C3` has a single cause, or that any representation could fix it.
- Any warrant for `.sea` grammar changes, architectural promotion, or provider authority — none is claimed, and T27's evidence grants none.
- That correct judgment here equals SEA-Forge settlement: ground truth is the fixture's `run_terminal` event, nothing more.
