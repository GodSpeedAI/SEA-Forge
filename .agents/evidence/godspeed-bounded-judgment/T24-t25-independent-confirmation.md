# T24/T25 Independent Confirmation — fresh verifier, recompute-everything

- Verifier: fresh independent agent, no prior involvement in T21..T25.
- Date: 2026-09-18 (verification session).
- Subject: T24 and T25 of `.agents/plans/godspeed-bounded-judgment-plan.yaml` (v1.5.0).
- Frozen contract: `.agents/preregistrations/godspeed-bounded-judgment-T21.prereg.yaml` + addenda 1-3.
- Method: read-only recompute from raw data. State databases re-opened read-only
  (`sqlite3 file:...?mode=ro&immutable=1`); every pinned sha256 recomputed with
  `sha256sum`; configs diffed directly; decision log and counter files re-counted.
  Builder narratives (`b1-progress.md`, `b1-result.json`, `IN-FLIGHT.md` in T24/T25)
  were NOT read. No cargo/just/build commands run. Only this report file was written.

## Frozen contract anchors (re-verified by hash)

| File | sha256 (recomputed) | Matches pin in |
|---|---|---|
| T21 base prereg | `1fb55d9c…8ea8b5` | route-record.round3 authority chain |
| addendum-1 | `c472d1b6…0cea8` | route-record.round3 authority chain |
| addendum-2 | `087887c2…4c5a` | route-record.round3 authority chain |
| addendum-3 | `1c13889a…6ddb` | route-record.round3 authority chain |
| T23 `route-record.round3.yaml` | `c6b9ee9c…7097` | `candidate-case.yaml` source_route_record_sha256 |

T23's settled route facts used below (read from `route-record.round3.yaml` directly):
route t23-route-001 settled; boundary r\*=4 with r\*-1=3 on its own ledger
(e2e-happy@4 settled with 3 settlement_committed; @3/@2/@1 budget_exhausted with
2/1/0 settlements); fingerprint `e2e-happy@3:budget_exhausted;e2e-happy@6:settled;e2e-happy@2:budget_exhausted`.

## T24 — settlement-first inverse case generation

### T24-A: the replay run genuinely settled — CONFIRMED (recomputed)

- Pin: `accepted-case.yaml`, `replay-record.yaml`, and `matched-pair.yaml` all pin
  `…/T24/replay-001-r1/runs/replay-r1/state/gauntlet-state.db` at
  `5cf2459efbdc8159d81d78ccbbaf865c2790df05f77fb58dba11fdeb89f45fbc`.
  Recomputed sha256: identical.
- Re-opened the DB read-only myself:
  - `event_log` contains exactly one `run_terminal`: `{"state":"settled"}`.
  - `settlement_committed` count = 3 (AK-901, AK-902, AK-903; distinct
    verification refs `vrf-F7CH…`, `vrf-1X1K…`, `vrf-32AK…`).
  - `runs.budget` limits: `"rounds":4` — matches the case's max_rounds 4.
  - `runs.run_id` = `BJAAGTH2PCVHAZ5FC48DFWK1V4`, matching the
    `gauntlet deliver --run BJAAGTH2PCVHAZ5FC48DFWK1V4 --apply` line in the
    recorded stdout tail (the observation joins the real run).
- Recorded observation (terminal settled, 3 settlements) equals the raw DB. The
  replay is a genuine settlement through gauntlet's own event log.

### T24-B: freeze-before-replay + config matches the settled route — CONFIRMED (recomputed)

- Hash pin: current `candidate-case.yaml` sha256 = `6bf2610c…5977` = the pin inside
  `candidate-freeze.json` (`candidate_sha256`) = the T24 evidence-manifest pin =
  the `freeze_ordering.candidate_seen_sha256` in `replay-record.yaml`.
- Ordering: `candidate-freeze.json` frozen_at `2026-09-19T04:08:04.784746+00:00`
  with `replay_record_exists_at_freeze: false`; observed file mtimes
  candidate 1789790884.784598 < freeze 1789790884.784857 (matches the
  replay-record's recorded mtimes); replay started `04:09:30.816315+00:00` —
  freeze strictly precedes replay. All ordering sources agree.
- Config vs settled route: accepted case bounded_input = (e2e-happy, max_rounds 4);
  target recipe = fresh disposable copy of
  `/home/sprime01/projects/gauntlet/tests/fixtures/target`; runner default
  mock-agent with NO runner sed and NO budget sed
  (`max_rounds_sed_applied: false`; the case value equals the fixture default —
  fixture `GAUNTLET.md` records `budget.max_rounds: 4` and
  `runners.default: mock-agent`, re-read directly); frozen_env_block matches
  addendum-1's env exactly (STATE_DIR, ARTIFACT_DIR, MOCK_SCENARIO=e2e-happy,
  unique ID_SEED, CLOCK=deterministic, STALL 60000, RETRY 100/2000,
  MAX_ATTEMPTS_PER_UNIT=2, TOOL_TEST=<target>/scripts/check.sh, timeout 240 s).
  This is exactly the settled route's discovered boundary: r\*=4 via
  e2e-happy@4 settled / e2e-happy@3 budget_exhausted on the route's own ledger.
- Fresh disposable copy verified by content: `diff -rq fixture target` returns
  zero differences for the replay target (state/artifacts live outside the target
  under the durable run dir). Seed `t24-case-001-replay-r1-seed` is absent from
  all T22/T23 records (grep); the 12 prior recorded seeds are the 12 T23 route
  probe seeds, matching `prior_recorded_seeds_checked: 12`.

### T24-C: tooth T7 (@2 lookalike rejected) — CONFIRMED (recomputed from its DB)

- Pin: `…/T24/teeth-r1/runs/t7-attack/state/gauntlet-state.db` pinned at
  `7136ad9c9333586ceb2f0e8671ca77066f9fa9d6e0d9f59ca11ced1f8a99841d`;
  recomputed sha256: identical.
- Re-opened read-only: exactly one `run_terminal` with state `budget_exhausted`;
  `settlement_committed` count = 1 (AK-901); `runs.budget` limits `"rounds":2`.
  The disposable target's `GAUNTLET.md` shows `max_rounds: 2` (sed 4→2 applied to
  the copy, fixture untouched).
- Acceptance path: `teeth/t7-attack-rejected-case.yaml` records `accepted: false`
  with an explicit invalid_reason and is preserved only under `teeth/` (marked
  ATTACK); the developmental `accepted-case.yaml` is byte-identical before/after
  (immutability block hashes equal the manifest pins). Expected REJECTED,
  observed REJECTED — MATCH.

## T25 — matched negative + payment/novelty

### T25-A: matched negative — CONFIRMED (recomputed)

- Single factor: the negative target's `GAUNTLET.md` vs the fixture differs in
  exactly one line — `max_rounds: 4` → `max_rounds: 3` (full diff shown; no other
  file in the target differs). The mutation record pins
  `bounded_input.max_rounds` 4→3 with everything else identical to the accepted
  positive case (same scenario e2e-happy, same fixture recipe, same frozen env;
  fresh seed `t25-negative-r1-seed`, which grep shows appears in no prior record).
- Negative DB recomputed: `…/T25/negative-001-r1/runs/negative-r1/state/gauntlet-state.db`
  sha256 matches its pin `097e1af5…d88`; exactly one `run_terminal` with state
  `budget_exhausted`; `settlement_committed` count = 2 (AK-901, AK-902);
  `runs.budget` limits `"rounds":3`; run_id `9TRR7D9C515BKD9TW0V5XNGC17` matches
  the recorded stdout-tail delivery line. This is the same honest-exhaustion
  signature the route's own r\*-1 probe (e2e-happy@3, 2 settlements) showed.
- Label provenance: the recorded negative label (budget_exhausted, settled=false,
  2 settlements) equals the raw terminal consequence I recomputed from the pinned
  DB — read from `run_terminal`, not assigned. Had it settled, the pair would not
  be a matched negative; it did not settle.

### T25-B: payment comparison — CONFIRMED (re-counted), one presentational note

Decision log (re-counted from `.agents/evidence/godspeed-bounded-judgment/decisions.yml`):
- Branch range D-2026-09-18-T21-01 .. D-2026-09-19-T23-03: exactly 7 entries
  (T21-01/02/03, T22-01, T23-01/02/03). The interleaved D-2026-09-18-T18-0x
  entries belong to the paused T18 branch and are correctly not counted. The log
  ends at T23-03; T24/T25 payment-counters record `operator_decisions: 0` each.
- Baseline T15/T16 entries: exactly 21 (T15-01/02/03 + T16-01..T16-18). Matches
  `manual_baseline_t15_t16.operator_decision_entries: 21`.

Model calls (per stage, each re-checked against its named source file):
- T22 round-1 `payment-counters.yaml`: count 4 ✓ (1 generation + 3 probe_judgment).
- T22 round-2 `payment-counters.yaml`: count 4 ✓.
- T23 route `payment-counters.yaml`: count 16 ✓ (4 candidate_generation iters + 12 probe_judgment).
- T23 round-2 `payment-counters.round2.yaml`: count 12 ✓ (re-judgment only; tool_executions 0).
- T23 round-3: 0 ✓ (`route-record.round3.yaml` payment block).
- T24 `payment-counters.yaml`: 1 ✓; T25 `payment-counters.yaml`: 0 ✓.
- Total 4+4+16+12+0+1+0 = 37 ✓; generation-class calls 1+1+4+1 = 7 ✓.

Tool executions: 3 (T22 r1) + 3 (T22 r2) + 12 (T23 route) + 1 (T23 teeth T4 —
`T23/teeth/attacks.yaml` states "T4 ran the only real gauntlet execution (1);
T3/T6 ran none") + 2 (T24 replay + T7, each verified against its own DB above)
+ 1 (T25 negative) = 22 ✓. Failed executions: 3 (the preserved T22 round-1
run_error trio; round-2 file records failed_executions 0; all later stages 0) ✓.

Wall clock: 48.852 + 56.456 + 275.997 + 23.737 + 0.505 + 0.204 + 0.303 = 406.054 ✓
(matches `total_recorded`); T23 round-2/round-3 recorded no wall clock and are
excluded as n/a — verified absent from `payment-counters.round2.yaml` and the
round-3 record. Tokens/cost: marked n/a on both sides; every counter file in the
chain carries the same honest-gap note ✓.

Baseline side quotes re-verified against the actual T15/T16 records:
- T15 corpus manifest: 9 runs; class_counts 3 settled + 6 budget_exhausted ✓;
  rounds 1-2 preserved as FAILED rounds ✓; operator-authored 9-case grid
  (T15 prereg case table: A defaults, B defaults, C1/C2 @1, C3 @2) ✓.
- T16 expanded corpus manifest: 25 runs; 12 settled + 13 budget_exhausted ✓
  (row-level consequence_class counts re-counted); mechanism mix e2e-happy x9 at
  defaults / e2e-lying x3 / budget-r1 x2 / e2e-forged-report x2 ✓ (expansion
  addendum, matching case ids A4-A12, B4-B6, C4-C5, D1-D2).
- Provider answers: 9-row rerun report — deepseek-clinepass n_answered 9 /
  contract_failures 0, prime-agent 0/9 ✓ (`T16/round-1/remeasure-report.9row-rerun.yml`);
  expanded round — deepseek 24/1, muse-spark-free 25/0 ✓
  (`T16/expanded-round/remeasure-report.yml`).
- No wall clock exists anywhere in the T15/T16 manifests → `wall_clock: n/a` is
  honest, not an estimate ✓.

Presentational note (non-material): `payment-comparison.yaml`'s model_calls block
lists `generation_calls: 7`, `judgment_calls: 18`, `total: 37`. The file's
`judgment_calls` counts only the probe_judgment class in T22r1/T22r2/T23-route
(3+3+12=18) and excludes the separately itemized 12 round-2 re-judgment calls, so
7+18 ≠ 37; the gate's console summary uses the other decomposition (37−7=30,
"judgment" = all non-generation). Every per-stage number and the total are correct
against their named sources; only the class label is ambiguous between the two
renderings. No estimate, no wrong counter.

### T25-C: novelty — CONFIRMED (recomputed; "novel" is the correct frozen-metric classification)

- Fingerprint recomputed from `route-record.round3.yaml` moves using the frozen
  definition (`T21/route_contracts.py route_fingerprint`: ordered retained moves,
  `{scenario}@{max_rounds}:{terminal_class}` joined by `;`):
  `e2e-happy@3:budget_exhausted;e2e-happy@6:settled;e2e-happy@2:budget_exhausted`
  — equals the subject fingerprint recorded in `novelty.yaml` and the route record.
- T15/T16 grid recomputed from the frozen records (T15 prereg case table, T15
  corpus manifest, T16 expansion addendum, T16 expanded corpus manifest): exactly
  five distinct single-run fingerprints — `e2e-forged-report@4:budget_exhausted`,
  `e2e-happy@1:budget_exhausted`, `e2e-happy@2:budget_exhausted`,
  `e2e-happy@4:settled`, `e2e-lying@4:budget_exhausted` — identical to the
  `grid_fingerprints` list in `novelty.yaml`. Recorded grid budget levels: {1,2,4}.
- "T15/T16 grids never probed fixture-target budget levels 5/6/7": TRUE — the
  T15 case table uses defaults/1/2 and the T16 expansion addendum mechanism mix
  uses defaults/1 only; no recorded case in either manifest ran at 5, 6, or 7.
  The route's levels 3 and 6 are outside the grid (level 2 is in-grid as a single
  run, but not as part of an equal sequence).
- Classification under the frozen rule: no grid fingerprint equals the route
  fingerprint (all grid entries are single-run sequences) → not a structural
  duplicate; no grid case has the route's operation set {e2e-happy@2,
  e2e-happy@3, e2e-happy@6} → not a near-duplicate; therefore `novel` is correct.
  Duplicates policy respected: measured and recorded, no auto-rejection.

### T25-D: tooth T8 (structural duplicate detected) — CONFIRMED (recomputed)

- The synthetic record `teeth/t8-duplicate-route-record.yaml` carries the same
  ordered retained-move sequence with the same terminals as t23-route-001
  (e2e-happy@3 budget_exhausted; e2e-happy@6 settled; e2e-happy@2
  budget_exhausted). Recomputing the frozen fingerprint over its moves yields
  exactly the reference fingerprint — an equal fingerprint, which under the same
  frozen classification rule is `structural-duplicate`, exactly what
  `teeth.yaml` and `t8-attack-record.json` record (`duplicate_detected: true`,
  `auto_rejection_applied: false`, per the frozen measured-not-rejected policy).
- The attack is synthetic (0 gauntlet executions, 0 provider calls — consistent
  with a record-level computation) and is kept only under `teeth/`, marked
  ATTACK, absent from the developmental records; `matched-pair.yaml` is
  byte-identical before/after (hashes in the immutability block equal the
  manifest pin).

## Process checks

- Gates (run from the sea-rs repo root, exactly as the plan freezes them):
  - `python3 .agents/evidence/godspeed-bounded-judgment/T24/verify_case.py` →
    `PASS (154 checks)`, exit 0.
  - `python3 .agents/evidence/godspeed-bounded-judgment/T25/verify_payment.py` →
    `PASS (137 checks)`, exit 0. The gate re-derives the pair labels from the
    pinned DBs, re-diffs the target copies, re-counts the decision log and
    counters, recomputes novelty, and byte-compares its own deterministic outputs
    (`payment-comparison.yaml`, `novelty.yaml`) against disk each run.
- Manifests hash-verify: all 14 pinned files in `T24/evidence-manifest.yml` and
  all 8 pinned files in `T25/evidence-manifest.yml` match their sha256 pins
  (recomputed). The unpinned files are exactly the disclosed post-gate/derived
  ones (b1-result.json, .md notes; T25's gate outputs, which the gate
  byte-verifies every run).
- Scope: no record in T24/T25 claims promotion, capability-class, production, or
  architectural scope (grep over the pinned records); both tasks' `does_not_prove`
  boundaries are respected; teeth artifacts are marked ATTACK and absent from the
  developmental set; the frozen T23 record and prereg chain are byte-unchanged
  (hashes above).

## Discrepancies found

1. `payment-comparison.yaml` model_calls class labels: `judgment_calls: 18`
   (probe-judgment class only) vs the gate summary's "30 judgment"
   (all non-generation). Both decompositions derive from correct, source-verifiable
   stage counts and the same correct total (37); the ambiguity is presentational
   only. Non-material; noted for the next report revision.
2. No other discrepancies. Every recomputed value (hashes, terminals, settlement
   counts, budgets, config diffs, decision counts, stage counters, wall-clock sum,
   baseline quotes, fingerprints, classifications) matched the recorded evidence.

## Final verdicts

All scoped claims verified: T24's accepted case is consequence-backed by an
independent replay that genuinely settled (recomputed from the pinned DB), the
candidate was frozen before the replay existed, the case config equals the
settled route's discovered boundary, and the T7 lookalike was honestly rejected
by the same acceptance path. T25's matched pair differs in exactly one factor and
its negative label is the raw run_terminal consequence; the payment table's
numbers re-count cleanly against their named sources with honest n/a marking and
faithful baseline quotes; the novelty classification `novel` is correct under the
frozen metric against the recorded T15/T16 grid; the T8 synthetic duplicate is
correctly classified structural-duplicate by the same computation. Both gates
exit 0 on the final tree; both manifests hash-verify; no scope overreach.

T24 VERDICT: CONFIRM
T25 VERDICT: CONFIRM
