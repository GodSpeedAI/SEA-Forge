# T29 IN-FLIGHT — native route-discovery breadth, second operation family

- Task: T29 of `.agents/plans/godspeed-bounded-judgment-plan.yaml` (v1.7.0), builder B1.
- Prereg: FROZEN `.agents/preregistrations/godspeed-bounded-judgment-T29.prereg.yaml`
  sha256 `e844fce47c9860eeaacac0b8d27f0d28ef386da21f1c45ed878e85ba02af9672`
  (decision D-2026-09-19-T29-01). Executed EXACTLY; no redesign.
- Family: `e2e-forged-report` x budget ladder {1..8}. Mechanical rule: the frozen
  family-2 UP rule (unsubstantiated-pass detection). Route goals G1/G2/G3,
  bounds (<=5 retained, <=8 probes, <=4 iterations, 240 s probe wall, 1800 s
  route wall), matched negative at d*-1, optional observer (4 calls,
  observational only) — all per prereg.

## Frozen inputs (verified before first execution)

- T21 frozen modules (imported, never modified): `route_contracts.py`
  `0d802387…`, `admission.py` `8608b495…`, `selection_policy.py` `9b25b0c8…`,
  `operations_catalog.yaml` `85327cb2…`, `verify_contract.py` `92b16ac5…`
  — all match T21/evidence-manifest.yml.
- Family-1 frozen ledger (read-only): `T23/round-3/mechanical-labels.yaml` +
  `route-record.round3.yaml`; all 12 pinned state DBs re-hashed and matched
  before start.
- Probe mechanism: frozen addendum-1 (fresh copy of
  `/home/sprime01/projects/gauntlet/tests/fixtures/target` per probe;
  `max_rounds: 4` sed only when budget != 4; addendum env; timeout 240;
  one gauntlet process at a time).
- `PYTHONDONTWRITEBYTECODE=1` exported for all python; no cargo/just/build.

## Durable probe data

`$HOME/.local/share/godspeed-route-discovery/T29/route-001-r1/` (fresh dir,
created by harness.py; per-probe `runs/<iter-key>/state` + `/evidence`,
per-probe disposable `targets/<iter-key>`).

## Execution sequence (append-only under route/, teeth/, observer-raw/)

1. [x] Verify prereg sha256 + T21 module hashes + family-1 DB pins.
2. [x] IN-FLIGHT.md + b1-progress.md.
3. [x] Author `harness.py` (family-2 UP rule, probe runner, G1/G2/G3 evaluator,
       T21 admission/selection reuse, scenario e2e-forged-report); FROZEN at
       sha256 `fd555c9eb8536039fd0ee20d3e83abee8af66748f091b9de00c5ebda714b0f9c`
       before first execution (recorded in b1-progress.md).
4. [ ] Iteration 1: bounded neighborhood (forged ladder, all unprobed); K=3
       candidates (deepseek-clinepass via opencode seam; ONE muse substitution
       recorded; deterministic fallback = midpoint ladder @4/@2/@6); admit;
       probe ALL admitted SEQUENTIALLY; UP rule per probe; retain per
       selection-policy-v1.
5. [ ] Iterate (<=4 iterations, <=8 probes) until G2 d*/d*-1 established.
6. [ ] G3: recompute UP + addendum-3 rules from the FROZEN family-1 ledger DBs
       (no new runs); partition verdict.
7. [ ] Matched negative: forged@d*-1 (single factor: budget), label read.
8. [ ] Observer: exactly one call per retained move (prereg scale 4),
       arm2-style surface (reuse T23 round-2 surface builder, imported
       unmodified), deepseek-clinepass, raw under observer-raw/; labels
       OBSERVATIONAL ONLY.
9. [ ] Payment counters per probe/iteration + breadth comparison vs family 1
       (from frozen T22-T26 records).
10. [ ] Teeth (teeth/attacks.yaml): fabricated record w/o DB -> refusal;
       hand-set d* -> gate refuses; observer label flip -> inert.
11. [ ] `verify_breadth.py` gate (stdlib+PyYAML, run from sea-rs repo root),
       then evidence-manifest.yml (before the final gate run), gate -> exit 0.
12. [ ] `b1-result.json` (LAST).

## Honesty rails

If the UP rule never fires on any real probe in {1..8}: goal_met=false,
falsified layer classified per prereg redesign_trigger, status complete, stop.
Never weaken/redefine the rule post hoc. If real probes contradict prereg
predictions, the evidence wins.

## Interpretation notes (most-restrictive readings, recorded as deviations if load-bearing)

- Loop judgment labels (selection-policy-v1 input) are the mechanics-only
  deterministic terminal mapping (T23 frozen map), recorded as mechanics-only;
  the prereg defines no per-probe provider judge for T29 — the learned observer
  is the only judgment surface (one call per retained move, observational only).
  The UP rule — not judgment — is the route-goal evaluator (addendum-3
  architecture carried to family 2).
- Generation attempts that propose candidates outside the frozen family-2
  candidate schema (scenario must be e2e-forged-report) fail that attempt per
  the frozen parse rule (recorded); ladder continues (substitution/fallback).
- If the route meets its frozen goal with fewer than 4 retained moves, the
  observer gets one call per retained move (<4); the "exactly 4" scale
  anticipated 4 retained moves and the frozen stop rule (route goal met)
  outranks inventing extra iterations to reach it. Recorded as a deviation.
