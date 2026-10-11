# T22 B1 Progress Log (append-only)

## 2026-09-18 — start

- Read plan T22 entry (plan v1.5.0), frozen prereg
  `godspeed-bounded-judgment-T21.prereg.yaml`, and all four T21 frozen modules.
- Verified T21 imports UNCHANGED: sha256(route_contracts.py)=0d802387...7fe8c,
  sha256(admission.py)=8608b495...bf49b, sha256(selection_policy.py)=9b25b0c8...f5ef,
  sha256(verify_contract.py)=92b16ac5...8a657, sha256(operations_catalog.yaml)=85327cb2...c7dd,
  prereg sha256=1fb55d9c...ea8b5 — all match T21/evidence-manifest.yml. Proceeding.
- Probe mechanism confirmed against T15 frozen mechanism
  (`T15/batch-round1-failed.sh`): cp -r demo-calculator, chmod +x scripts,
  sed default runner -> mock-agent, optional sed max_rounds, env
  GAUNTLET_STATE_DIR/GAUNTLET_ARTIFACT_DIR/GAUNTLET_TOOL_TEST/GAUNTLET_MAX_ATTEMPTS_PER_UNIT=2/GAUNTLET_MOCK_SCENARIO,
  headless `gauntlet run "Build and verify the calculator page"`, timeout 240.
- Terminal extraction pattern confirmed against `T15/verify_corpus.py`:
  sqlite ro `select payload from event_log where kind='run_terminal'` -> payload JSON
  `.state`; `settlement_committed` events counted; DB file `gauntlet-state.db` under
  the state dir.
- Provider seam confirmed against `T16/run_remeasure.py` call_provider():
  primary `opencode run -m cline-pass/cline-pass/deepseek-v4.1-flash`,
  substitution `opencode run -m opencode/muse-spark-1.3-contributor-free`.
- Created T22/ scaffolding: IN-FLIGHT.md, b1-progress.md, iteration-1/, teeth/.
- PYTHONDONTWRITEBYTECODE=1 will be exported for every python invocation and
  `sys.dont_write_bytecode = True` set inside loop.py before T21 imports.

## Interpretations recorded (most-restrictive-reading decisions; also in deviations)

1. probe_observation schema (frozen T21 route_contracts) has an exact field set and
   refuses unknown fields, so per-probe exit code / wall seconds cannot live inside the
   typed probe_observation record; they are recorded in a companion typed sidecar
   `iteration-1/probe-run-meta.yaml` keyed by probe_id (task-instruction conflict with
   the frozen schema resolved in favor of the frozen schema; recorded as deviation).
2. Route record stop_condition for this task-scoped run: the loop executes exactly ONE
   advance per the task contract, so the recorded run ends at its own iteration bound
   (`iteration_limit_reached` with the task-scoped bound of 1 documented); the route
   itself remains open for T23 under the frozen 6-iteration bound and `settled: false`.
3. Provider candidates must echo the iteration correlation_id exactly and use a
   filesystem-safe candidate_id (^[a-z0-9][a-z0-9-]{0,63}$) because candidate_id names
   the durable probe target/state directories; violations are treated as schema_failure
   for that generation attempt (substitution -> fallback).
4. selection_policy consumes route_state AFTER this iteration's probes are absorbed
   (probed entries carry judgment labels) because _route_knowledge derives goal/boundary
   state solely from route_state["probed"]; the retained move is appended after select().
5. Payment passed to select() is the post-probe remaining budget (probes already paid
   this iteration; retention itself costs no additional probe unit).

## 2026-09-18 — ROUND-1 EXECUTED AND FAILED (preserved); correction round 2

- Round-1 (loop.py, records under iteration-1/) ran end-to-end: provider
  generation OK (deepseek-clinepass, 3 candidates, all admitted), but ALL
  THREE real probes ended run_error (exit=1 after ~0.1 s; no state db).
  Selection retained nothing; the failed round is preserved untouched as the
  round-1 evidence.
- Diagnosis (3 diagnostic gauntlet executions in /tmp/t22-diag, recorded here
  as tool executions outside the route; scratch dir cleaned up):
  1. The reused binary at gauntlet commit 3a9955fa REFUSES to start without
     three REQUIRED control settings that have no spec default
     (GAUNTLET_STALL_TIMEOUT_MS, GAUNTLET_RETRY_BASE_MS,
     GAUNTLET_RETRY_MAX_MS; see crates/gauntlet-app/src/config/resolve.rs,
     "the spec tables no default, so the control loop refuses to run without
     it"). The frozen 5-var env predates this requirement. This was the
     round-1 run_error cause.
  2. Without GAUNTLET_CLOCK=deterministic the mock replay's fixed-2023 event
     epoch is stale against the real clock; the stall detector kills every
     invocation and the terminal is "escalated" (no typed settled/
     budget_exhausted terminal is reachable). This is exactly the
     "wall-clock mismatch" lesson recorded in the T15 failed rounds; the
     working T15 corpus generator and the repo's e2e_full_loop_mock recipe
     both set the deterministic clock.
  3. FROZEN-CONTRACT CONTRADICTION (surfaced, NOT fixed): the mock scenario
     fixtures (e2e-happy/e2e-lying/e2e-forged-report) carry workspace effects
     keyed to tests/fixtures/target's answer key (AK-901..903:
     marker/full-check reports, GADGET_MARKER). The frozen demo-calculator
     fixture's answer key (AK-9011..9013) demands markup/eval/check reports
     that NO scenario writes, and its instrument (scripts/check.sh) runs in
     the mock workspace where calculator.html is absent (produced report:
     tests=1, failures=1). Consequence: through the frozen mechanism every
     probe returns a typed budget_exhausted consequence at ANY max_rounds and
     the frozen baseline goal ("e2e-happy@8 settles") is unreachable. The
     prereg's fixture wording (demo-calculator) conflicts with the actual
     working T15 corpus recipe (tests/fixtures/target + deterministic clock +
     per-case id seed). B1 cannot edit the prereg; this is recorded as a
     design question for the orchestrator (it will gate T23 route
     settlement), and the frozen mechanism is executed faithfully as frozen.
- Round-2 (loop.round2.py, records under round-2/, durable probe data under
  $HOME/.local/share/godspeed-route-discovery/T22/round2/, fresh route id
  t22-route-001-round2): smallest correction at the probe_execution layer —
  the three REQUIRED controls at repo-canonical values (60000/100/2000,
  liveness/backoff only, NOT budget/authority levers), GAUNTLET_CLOCK=
  deterministic (part of the working T15 mechanism the prereg freezes by
  reference), and probe stdout/stderr tails captured in the run-meta sidecar.
  The fixture is executed exactly as frozen.
- Tool executions ledger update: round-1 probes 3 (run_error) + diagnostics 3
  (scratch) + round-2 probes (see round-2 payment-counters.yaml).

## 2026-09-19 — ROUND-2 EXECUTED: real route advance complete; teeth; gate PASS

- Round-2 loop (loop.round2.py) end-to-end: provider generation via
  deepseek-clinepass (primary path; no substitution/fallback needed) proposed
  cand-1 e2e-happy@8 (establish_baseline), cand-2 e2e-lying@8
  (establish_discriminator), cand-3 e2e-happy@7 (boundary_probe); all three
  admitted by the frozen T21 admission module; all three probed SEQUENTIALLY
  with the corrected frozen mechanism — every probe returned a typed
  budget_exhausted terminal (exit=1, ~0.6-0.8 s wall, 0 settlement_committed),
  exactly as predicted by the fixture mismatch analysis; the provider judged
  all three observations budget_exhausted (real provider judgments;
  insufficient_evidence never needed); selection-policy-v1 retained cand-3
  (tier-2 boundary ordering) -> route advance fingerprint
  `e2e-happy@7:budget_exhausted`; route_record.settled=false; 9 probes remain.
- Round-2 wall clock 56.456 s; 4 provider calls (1 generation + 3 judgments,
  all deepseek-clinepass, all responded); 3 probes; 0 failed executions
  (all terminals typed); tokens/cost n/a (seam does not expose them).
- Teeth first run: T1/T2 MATCH, T5 revealed a ROUND-1 GATE DEFECT —
  verify_loop.py enumerated single-dict record files by key (selection-record
  failed as "record must be a mapping" x7), so the real records failed for the
  wrong reason. Correction per the failure-preservation rule: failed gate
  script preserved as verify_loop.round1-failed.py; failed teeth outputs
  preserved as teeth/round1-failed-*.json/yaml; smallest fix applied at the
  canonical path (check_schemas treats single-dict files as one record); teeth
  re-run: ALL THREE MATCH (T5 swapped copy fails naming the state-db/candidate
  binding violation; unswapped records pass).
- Round-1 failed records re-checked with the corrected gate (--records-dir
  iteration-1 --records-only): PASS (the failed round is coherent evidence).
- evidence-manifest.yml frozen: 62 files pinned (every T22 .py/.yaml in both
  directions + record .json + teeth records + t5-swapped copies +
  provider-raw). IN-FLIGHT.md, b1-progress.md and b1-result.json are
  intentionally unpinned (mutable narrative / post-gate result contract).
- OFFICIAL GATE: `python3 .agents/evidence/godspeed-bounded-judgment/T22/verify_loop.py`
  from /home/sprime01/projects/sea-rs -> PASS, exit 0.
- Tool executions ledger (whole task): round-1 probes 3 (run_error) +
  diagnostics 3 (scratch /tmp) + round-2 probes 3 (typed) = 9 gauntlet runs;
  teeth ran zero gauntlet processes; provider calls: round-1 4 + round-2 4 = 8
  (all deepseek-clinepass, all responded; 0 substitutions; 0 fallbacks).
