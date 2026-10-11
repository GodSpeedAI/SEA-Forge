# T22 IN-FLIGHT — Candidate -> governed probe -> bounded judgment -> deterministic selection loop

- Task: T22 of `.agents/plans/godspeed-bounded-judgment-plan.yaml` (v1.5.0)
- Builder: B1
- Depends on: T21 (SETTLED; frozen modules imported UNCHANGED, sha256-verified against `T21/evidence-manifest.yml`)
- Started: 2026-09-18 — status at last update: COMPLETE (round-2 real advance; gate PASS)

## Scope (binding)
- Work ONLY in `.agents/evidence/godspeed-bounded-judgment/T22/` plus durable probe data
  under `$HOME/.local/share/godspeed-route-discovery/T22/`. No plan, status, prereg, T21,
  crate, or gauntlet-source edits. No cargo/just/build commands. Zero compiles.
- Authoritative contract: `.agents/preregistrations/godspeed-bounded-judgment-T21.prereg.yaml`
  (frozen, sha256 1fb55d9c56d87d94adc4850bc5ca713db3e600297fbdc997accfab54c98ea8b5).
- ONE real route advance end-to-end: neighborhood -> K=3 candidates -> admission ->
  real sequential gauntlet probes -> bounded judgment -> selection-policy-v1 ->
  route record + payment counters. Teeth T1/T2/T5. Gate: verify_loop.py PASS.

## State (final)
- [x] T21 frozen-module sha256s verified against T21/evidence-manifest.yml (all 5 match; prereg matches).
- [x] Round-1 (`loop.py`, records `iteration-1/`) EXECUTED and FAILED: all 3 real probes
      run_error (binary refuses to start without three REQUIRED control settings).
      Failed round preserved untouched; diagnosis in b1-progress.md.
- [x] Round-2 correction (`loop.round2.py`, records `round-2/`, durable probe data
      `~/.local/share/godspeed-route-discovery/T22/round2/`, route id t22-route-001-round2):
      full real advance — 3 provider candidates (deepseek-clinepass), all admitted,
      3 real sequential gauntlet probes with typed `budget_exhausted` terminals,
      3 real provider judgments (budget_exhausted x3), selection-policy-v1 retained
      cand-3 (boundary probe e2e-happy@7), fingerprint `e2e-happy@7:budget_exhausted`,
      route NOT settled. Gate: `python3 .agents/evidence/godspeed-bounded-judgment/T22/verify_loop.py`
      PASS from the sea-rs root (exit 0).
- [x] Teeth T1/T2/T5 executed under `teeth/` — all MATCH expected rejections
      (a first teeth run surfaced a round-1 gate defect; the failed gate and failed
      teeth outputs are preserved under round1-failed names; see b1-progress.md).
- [x] evidence-manifest.yml frozen (62 files: every T22 .py/.yaml both directions,
      plus record .json, teeth attack records, t5-swapped copies, provider-raw).
- [x] b1-result.json written (LAST step).

## For the orchestrator (next move)
- FROZEN-CONTRACT CONTRADICTION (surfaced, not fixed, recorded in b1-progress.md and
  b1-result.json deviations): the prereg's fixture mechanism (demo-calculator copies +
  e2e-* mock scenarios) can never produce a `settled` terminal — the scenario workspace
  effects are keyed to tests/fixtures/target's answer key, so on demo-calculator every
  probe is budget_exhausted at ANY max_rounds and the frozen baseline goal
  ("e2e-happy@8 settles") is unreachable. T23 route settlement cannot close through
  this mechanism as frozen; the T15 clean corpus recipe (tests/fixtures/target +
  GAUNTLET_CLOCK=deterministic + per-case GAUNTLET_ID_SEED) is the working alternative
  and needs an append-only prereg addendum by the orchestrator before T23.
- T22 leaves the route OPEN (1 retained move, 9 probes remaining) for T23.

## SETTLED (orchestrator, 2026-09-18)

- Gate re-run personally: verify_loop.py PASS (exit 0) on the final tree.
- One real route advance end-to-end with bijective correlation joins; probe state DBs pinned (path+sha256) under $HOME/.local/share/godspeed-route-discovery/T22/; no settlement claimed at depth 1.
- Teeth T1 (authority bypass), T2 (model override), T5 (correlation swap) executed and rejected exactly as expected.
- Round-1 failure preserved (probe_execution layer); round-2 correction used round-2 paths; the fixture-mechanism contradiction the builder surfaced is corrected by frozen addendum D-2026-09-18-T21-03 BEFORE any T23 execution.
- Scope audit clean; T21 imports sha256-verified. T22 SETTLED (P2, builder_with_teeth).
