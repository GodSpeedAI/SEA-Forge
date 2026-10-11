# T29 IN-FLIGHT — native route-discovery breadth, second operation family

- Task: T29 of `.agents/plans/godspeed-bounded-judgment-plan.yaml` (v1.7.0), builder B1.
- Prereg: FROZEN `.agents/preregistrations/godspeed-bounded-judgment-T29.prereg.yaml`
  sha256 `e844fce47c9860eeaacac0b8d27f0d28ef386da21f1c45ed878e85ba02af9672`
  (decision D-2026-09-19-T29-01). Executed EXACTLY; no redesign.
- Family: `e2e-forged-report` x budget ladder {1..8}. Mechanical rule: the frozen
  family-2 UP rule (unsubstantiated-pass detection). Route goals G1/G2/G3,
  bounds (<=5 retained, <=8 probes, <=4 iterations, 240 s probe wall, 1800 s
  route wall), matched negative at d*-1, optional observer (observational
  only) — all per prereg.

## RESULT (route complete; gate PASS 618 checks exit 0; awaiting independent confirmation)

- goal_met=true; G1 true (forged@4 UP-fired on this route's own probe);
  d* = 2; d*-1 = 1 probed and NOT firing (matched negative, single factor
  budget); G3 partition verdict: holds (forged fires UP only; 4 frozen lying
  rows fire addendum-3 only; 8 frozen honest rows fire neither; recomputed
  from the 12 pinned family-1 DBs, no new runs).
- 6 real probes (of 8 budget), 2 iterations (of 4), wall 41.344 s (of 1800).
  Provider: deepseek-clinepass primary BOTH generations (no substitution, no
  fallback). Observer: 2 calls (one per retained move — see deviations).
- UP rule fired on forged budgets {2,3,4,5,6}; NOT at 1. addendum-3 fired on
  NO forged probe. Passing-claim artifact family sha256 98cbdd08…ad0e12.
- Gate: `python3 .agents/evidence/godspeed-bounded-judgment/T29/verify_breadth.py`
  from the repo root -> PASS (618 checks), exit 0.
- Teeth: T1 REFUSED (fabricated record, no pinned DB), T2 REFUSED (hand-set
  d*=3 vs recomputed 2; negative-label flip refused), T3 REJECTED (flipped
  observer labels are computationally inert; route record byte-identical).
- Breadth comparison vs family 1 recorded (route/breadth-comparison.yaml);
  claims recorded (route/claims-report.yaml): 3 may_earn claims EARNED, all
  may_not_earn claims NOT earned.

## Execution sequence (all steps complete)

1. [x] Verify prereg sha256 + T21 module hashes + family-1 DB pins (all 12 match).
2. [x] IN-FLIGHT.md + b1-progress.md.
3. [x] Author `harness.py`; FROZEN at sha256
       `fd555c9eb8536039fd0ee20d3e83abee8af66748f091b9de00c5ebda714b0f9c`
       before first execution (never modified since; confirmed post-gate).
4. [x] Iteration 1: provider ladder @4/@2/@6 (exactly the recommended spread);
       all admitted; probed sequentially; UP fired at all three.
5. [x] Iteration 2: provider ladder @3/@1/@5; probed; UP fired at 3 and 5,
       NOT at 1 -> d*=2 with d*-1 probed and not firing; goal met; route
       settled (characterize-and-stop).
6. [x] G3: recomputed from the frozen family-1 ledger (harness.py g3);
       partition verdict holds (route/g3-separation.yaml).
7. [x] Matched negative: forged@1 (route probe i2-cand-2-1, fresh seed)
       vs forged@2 (i1-cand-2): single declared factor budget; label read.
8. [x] Observer: 2 calls (one per retained move), arm2-style surfaces via the
       T23 round-2 builder imported unmodified, raw preserved under
       observer-raw/; labels observational only; inertness self-check + tooth T3.
9. [x] Payment counters (route/payment-counters.yaml) + breadth comparison
       (route/breadth-comparison.yaml).
10. [x] Teeth (teeth/attacks.yaml; runner run_teeth.round2.py; failed first
        attempt preserved as run_teeth.round1-failed.py).
11. [x] Gate: verify_breadth.py (correction round 3; failed rounds 1-2
        preserved as verify_breadth.round{1,2}-failed.py — the teeth caught a
        real gate path bug); evidence-manifest.yml (432 files, both
        directions); official run PASS 618 checks exit 0.
12. [x] b1-result.json (LAST).

## Deviations (recorded honestly)

1. Observer calls = 2, not the prereg's anticipated "exactly 4": the prereg's
   own rule is "one per retained move" and the route legitimately met its
   frozen goal at iteration 2 with 2 retained moves; the frozen stop rule
   (route goal met) outranks inventing extra iterations to reach the
   anticipated scale.
2. Loop judgment labels are mechanics-only deterministic terminal mappings
   (selection-policy-v1 typed-record input only): the prereg defines no
   per-probe provider judge for T29; the learned observer is the only
   judgment surface, and the UP rule — not judgment — is the route-goal
   evaluator (addendum-3 architecture carried to family 2).
3. verify_breadth.py needed two failed correction rounds (path-ancestry bug
   caught by tooth T3 on the gate's first execution); both failed rounds are
   preserved under their round-failed names together with the failed teeth
   artifact sets; the check logic never changed.
4. route_record.settled=true uses the typed schema's only success
   stop_condition (route_settled) and means ONLY that the frozen
   characterization goal G1+G2+G3 was met; the underlying gauntlet runs never
   settle in this family (0 settlements everywhere) — documented in
   route/route-record.yaml.
5. docs/explanation/native-route-discovery.md NOT updated by this builder:
   the prereg makes the update conditional and the dispatch scope is T29/
   only; the durable understanding did change (second family survived), so
   the update is flagged for the orchestrator.

## Honesty rails (status)

The UP rule fired on real probes; d* exists in {1..8}; no rule was weakened
or redefined post hoc; no prereg prediction was contradicted (predicted
d* in {2,3}; observed 2). NEXT MOVE: fresh independent adversarial
confirmation per the plan's T29 independent_confirmation block, then
orchestrator settlement.
