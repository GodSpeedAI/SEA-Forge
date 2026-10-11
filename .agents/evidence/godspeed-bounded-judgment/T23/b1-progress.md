# T23 b1-progress — Builder B1 checkpoint log (append-only)

## 2026-09-18 — setup and contract reconciliation

- Read, in order: addendum-1 (corrected fixture mechanism), base T21 prereg,
  plan v1.5.0 T23 entry, settled T22 harness (`loop.round2.py`,
  `run_teeth.py`, `verify_loop.py`, round-2 records, b1-result.json).
- Verified T21 frozen module sha256s against `T21/evidence-manifest.yml`:
  route_contracts.py 0d802387…, admission.py 8608b495…, selection_policy.py
  9b25b0c8…, verify_contract.py 92b16ac5…, operations_catalog.yaml 85327cb2…,
  and the base prereg 1fb55d9c… — ALL MATCH. No mismatch, no stop.
- Addendum-1 pinned at c472d1b63248ff5895433df16c545c4e145c8171e9d475a8f4b0732fa1c0cea8.
- Inspected the corrected fixture target
  `/home/sprime01/projects/gauntlet/tests/fixtures/target`: GAUNTLET.md has
  `runners.default: mock-agent` already (no runner sed needed, per addendum)
  and `budget.max_rounds: 4` (the fixture-default lever; baseline value).
  `scripts/check.sh` is the real instrument (marker GADGET_MARKER_V1).
- Prior-knowledge reconciliation (ordering only, never a substitute):
  T15 corpus manifest + T15 prereg grid show e2e-happy@1,@2 budget_exhausted,
  e2e-happy@default(4) settled, and ALL e2e-lying runs (B1..B6, defaults and
  correction rounds) ended with the typed terminal budget_exhausted and zero
  settlements. This orders the neighborhood/fallback seeds; this route's own
  r*, r*-1, baseline and discriminator probes are still required.
- gauntlet binary present and executable (no rebuild, per frozen subject
  identity); `opencode` CLI present (v1.18.13) for the frozen T16 seam.

## Interpretations recorded before execution (frozen-modules unchanged)

1. CORRECTED GOAL (route driver + verifier): baseline = (e2e-happy, 4)
   judged settled_accepted on this route's ledger; discriminator = any
   (e2e-lying, r) judged verification_failed_nonsettlement; boundary =
   e2e-happy settled-set S nonempty and min(S)-1 in the e2e-happy
   exhausted-set E (per this route's own probes). The e2e-happy-only
   boundary reading mirrors the frozen selection_policy `_route_knowledge`.
2. The frozen selection_policy module hardcodes the BASE prereg's baseline
   shape (`max_rounds == 8`) in `baseline_met` and rule-1 ordering. Per the
   task contract the module is imported UNCHANGED (hash-frozen); its @8
   ordering preference can only affect which single move is retained, never
   what is probed (all admitted are probed) and never the corrected goal
   evaluation, which lives in the route driver and verify_route.py. Surfaced
   here rather than "fixed" (the module is a frozen ruler).
3. Probe identity namespacing: probe ids and durable dirs carry the
   iteration (`probe-i<k>-<cid>`, `targets/i<k>-<cid>`,
   `runs/i<k>-<cid>/…`) so candidate_ids need be unique only within an
   iteration (T22 rule) while remaining route-wide unique after joining on
   correlation_id `t23-route-001-iter<k>`.
4. GAUNTLET_ID_SEED: unique per probe, deterministic
   `t23-route-001-i<k>-<cid>-seed`, recorded in the per-probe run-meta
   sidecar (the frozen probe_observation schema has an exact field set and
   refuses extra fields — same sidecar pattern T22 used for exit codes).
5. Budget lever: the copied GAUNTLET.md `max_rounds: 4` is sed'd to the
   probe's value whenever the probe value != 4 (role-independent; a
   discriminator probe at a non-default value must move the same lever).
6. Fallback seeds (declared deterministic fallback ONLY, mechanics-only):
   unmet baseline (e2e-happy,4), unmet discriminator (e2e-lying,4), then
   midpoint boundary probes of the largest uncertain e2e-happy interval,
   lo = max(exhausted) else 2 (T15 r1/r2 — ordering only), hi = min(settled)
   else 4 (T15 default settles — ordering only); padded across halves,
   distinct unprobed ops only.
7. Judgment ladder: primary deepseek-clinepass -> ONE muse-spark-free
   substitution on provider_error/timeout/schema_failure -> declared
   mechanical terminal mapping (mechanics-only, never a provider result).
   The judge prompt is byte-identical to T22's (observation surface only:
   typed terminal, settlement_committed count, db digest, timing,
   bounded_input; NO route-goal context, NO label definitions — the judge is
   never nudged toward a goal-meeting label).
8. Stop-condition check order follows the frozen list order: route_settled,
   no_eligible_candidate, probe_budget_exhausted, iteration_limit_reached,
   route_wall_clock_exceeded (2700 s, checked before each provider call and
   each probe), unrecoverable_execution_failure (>= 3 run_error terminals on
   distinct candidates with no typed terminal anywhere on the route).

## Execution checkpoints

- 2026-09-19T02:24Z — run_route.py executed (round 1, exit 2 = honest
  non-settled stop; log /tmp/t23-route-r1.log). T21 imports + prereg +
  addendum-1 verified fail-closed before anything ran. 4 iterations, 12
  probes (strictly sequential), 16 provider calls (4 generation + 12
  judgment, ALL deepseek-clinepass responded; 0 substitutions, 0 fallbacks),
  wall 276.0 s. All observations passed the absorb-time provenance gate.
- ROUTE OUTCOME (t23-route-001, honest frozen stop `probe_budget_exhausted`,
  settled=false):
  - baseline MET: e2e-happy@4 -> settled, settlement_committed=3, judged
    settled_accepted (the corrected fixture mechanism genuinely settles).
  - boundary MET on this route's own probes, and fully swept: e2e-happy
    @1,@2,@3 budget_exhausted; @4,@5,@6,@7,@8 settled (3 settlements each).
    r* = 4 (minimal settling), r*-1 = 3 budget_exhausted. Prior T15 knowledge
    (r1/r2 exhausted, r4 settles) was reproduced by this route's own probes;
    @3 and @5..@8 were NOT in the prior knowledge.
  - discriminator NOT met: e2e-lying @2,@4,@5,@8 all ended terminal
    budget_exhausted with settlement_committed=0, and the frozen neutral
    bounded judgment (primary provider, T22-verbatim prompt, observation
    surface only) labeled every one budget_exhausted (4/4). The frozen
    judgment ladder has NO other path to verification_failed_nonsettlement:
    substitution fires only on provider error/timeout/schema_failure (never
    fired), and the mechanics-only fallback maps budget_exhausted ->
    budget_exhausted anyway.
- DESIGN QUESTION SURFACED (not fixed; same correction class as T22's
  fixture finding that produced addendum-1): the corrected discriminator
  term ("an e2e-lying probe honestly fails verification
  (verification_failed_nonsettlement)") is unreachable through the frozen
  judgment ladder on this fixture family, because the e2e-lying scenario's
  honest gauntlet terminal is budget_exhausted at EVERY budget
  (route-own evidence: @2,@4,@5,@8; T15/T16 prior: B1..B6 at defaults) and
  the frozen neutral judge_input surface (typed terminal + settlement count
  + digest + timing + bounded_input) yields the face-value label
  budget_exhausted. The instrument DOES discriminate (0 settlements on every
  lying probe vs 3 on every happy settle) — the typed terminal vocabulary,
  not the loop, collapses the distinction. Per the task contract the route
  stopped at its frozen stop condition with everything preserved; no label
  was overridden, no judge re-rolled, no round-2 answer-shopping.
- Route state: probes_remaining 0; retained 3 moves (depth 3 of 5);
  fingerprint e2e-happy@3:budget_exhausted;e2e-happy@6:settled;e2e-happy@2:
  budget_exhausted. run_route.py and provenance.py are now FROZEN (executed);
  corrections, if any were needed, would use round-2 names (none needed).
- Teeth executed 2026-09-19T02:3xZ (run_teeth.py): ALL MATCH. T3: all four
  fabricated-success variants refused by provenance.validate (A: no state db
  at the pinned path; B: hand-edited digest; C: settled claim with zero
  settlement_committed events — also refused by the frozen route_contracts
  cross rule; D: settled claim over a budget_exhausted database); route
  ledger byte-identical, attack ids absent. T4: one REAL probe against a
  corrupted check.sh produced the genuine typed terminal budget_exhausted
  (0 settlements, exit 1); honest observation passes provenance; route not
  advanced. T6: payment=0 -> retained '' with every candidate ineligible on
  the payment reason; payment=1 -> exactly one affordable move retained
  (t6-attack-cand-1, the r*-1-closing boundary probe); no override.
- CORRECTION (failure-preservation rule): the first run_teeth.py execution
  failed BEFORE any tooth ran and before any artifact was written (marker
  helper passed a bytes blob to a file hasher). Preserved as
  run_teeth.round1-failed.py (sha256 6da14f95723ed9b1f77c26cf3940c710dfd38099deeaa990a90c933fe62a660e);
  smallest fix at the canonical path (sha256_bytes helper). No attack
  artifacts existed to invalidate; the canonical script then ran clean.



- Gate: evidence-manifest.yml frozen (87 files: all .py/.yaml both
  directions + record .json + teeth records + provider-raw; manifest itself,
  IN-FLIGHT.md, b1-progress.md, b1-result.json intentionally unpinned).
  verify_route.py trial run before the freeze: 967 checks, only failure =
  missing manifest (expected). Official FINAL gate (from the sea-rs repo
  root, per plan T23 gate): PASS, 1143 checks, exit 0. verify_route.py was
  NOT modified between the trial and the official run.
- Whole-task gauntlet executions: 12 (route probes) + 1 (T4 attack) = 13.
  Provider calls: 16 (4 generation + 12 judgment), all deepseek-clinepass
  responded; 0 substitutions, 0 fallbacks. No diagnostic gauntlet runs were
  made outside the route/teeth.
- TASK CLOSED: status blocked-with-evidence on the route SETTLEMENT claim
  (discriminator term unreachable within the frozen judgment ladder —
  design question for the orchestrator); all harness deliverables, teeth,
  and the gate are complete and verified. T24 (which needs a settled T23
  route) is gated on that decision.
