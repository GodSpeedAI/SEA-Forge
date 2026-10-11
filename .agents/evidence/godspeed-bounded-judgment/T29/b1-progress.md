# T29 B1 progress notes (append-only)

## 2026-09-19 — start

- Builder B1 dispatched for T29 (native route-discovery breadth, second
  operation family; family `e2e-forged-report` x budget ladder).
- Prereg verified frozen: sha256 e844fce47c9860eeaacac0b8d27f0d28ef386da21f1c45ed878e85ba02af9672
  = the sha in decision D-2026-09-19-T29-01.
- T21 frozen modules re-hashed; ALL match T21/evidence-manifest.yml:
  route_contracts.py 0d8023877846c606bb1359d1c4ae8a683706caf7f4e0d7318165c124dac7fe8c
  admission.py 8608b495eebb6e281f50e41ee7c22317260b4219263247161789870d792bf49b
  selection_policy.py 9b25b0c8c9a65ce0771f1821b4f34b4161adfdb11dd5c5aa1da620c7b19ff5ef
  operations_catalog.yaml 85327cb23ac35df21e767c4cfe603cd287a7724906e453f4ab1d589731fec7dd
- Family-1 frozen ledger read (T23 round-3 mechanical-labels.yaml +
  route-record.round3.yaml): 12 probes (8 honest e2e-happy, 4 lying
  e2e-lying); all 12 pinned state DBs re-hashed under
  `$HOME/.local/share/godspeed-route-discovery/T23/route-001-r1/runs/` —
  every pin matches. G3 inputs are intact.
- Environment checks: gauntlet debug binary present + executable; fixture
  target present (GAUNTLET.md budget lever `max_rounds: 4` at line 19);
  forged-report fixture re-read (its workspace effects carry
  `{"tests": N, "failures": 0}` passing-claim artifacts; scripts/check.sh
  really fails on the missing marker); opencode 1.18.13 present (the frozen
  provider seam used by T23/T24 today).
- PYTHONDONTWRITEBYTECODE=1 for all python invocations; no cargo/just/build
  anywhere in this task.
- Next: author harness.py, freeze its sha256, then iteration 1.

## Frozen harness identity (written before first execution)

- harness.py sha256: fd555c9eb8536039fd0ee20d3e83abee8af66748f091b9de00c5ebda714b0f9c
  (syntax-checked only — never executed before this freeze; one typo fixed
  pre-freeze in the MAX_RETAINED constant line, then re-hashed; this is the
  executed identity. No overwrite after first execution; corrections use
  round-2 names.)
- Pre-execution hygiene: the syntax-check created a __pycache__/ directory in
  the T29 tree despite PYTHONDONTWRITEBYTECODE=1 (py_compile writes
  explicitly); it was deleted before the first execution. No other debris.

## 2026-09-19 — route execution

- harness.py route run: 2 iterations, 6 probes, all sequential (one gauntlet
  process at a time), wall 41.344 s. deepseek-clinepass primary proposed both
  ladders exactly as recommended (@4/@2/@6, then @3/@1/@5); no muse
  substitution, no deterministic fallback, no schema failures.
- Probe results (UP rule, all recomputed from pinned DBs): forged@4 FIRED
  (G1), @2 FIRED, @6 FIRED, @3 FIRED, @5 FIRED, @1 DID NOT FIRE (0
  dispatches, 0 artifacts). d* = 2; d*-1 = 1 probed and not firing. All 6
  probes: budget_exhausted terminal, 0 settlements (the settlement gate
  refuses the forged reports). addendum-3 fired on NO forged probe.
- Passing-claim artifact family sha256 98cbdd080be79908b5ec5a87daef45325ce89ea793fb501b91990b99f2ad0e12
  content {"tests": 1, "failures": 0, "command": "scripts/check.sh --marker"}.
- Probe seeds (all distinct): t29-route-001-i1-cand-1-seed, …-i1-cand-2-seed,
  …-i1-cand-3-seed, …-i2-cand-2-3-seed, …-i2-cand-2-1-seed, …-i2-cand-2-5-seed.
- g3 subcommand: partition verdict holds (12 frozen family-1 rows recomputed
  from pins: honest 0/8 UP, lying 0/4 UP + 4/4 addendum-3; forged 5/6 UP +
  0/6 addendum-3; all budgets >= d* fired).
- Observer: 2 calls (one per retained move: probe-i1-cand-2, probe-i2-cand-2-1),
  deepseek-clinepass primary, labels [budget_exhausted, budget_exhausted]
  (observational only); route-record.yaml byte-identical before/after.
  Deviation vs the prereg's anticipated "exactly 4" scale recorded in
  IN-FLIGHT.md (the route legitimately retained 2 moves).
- Breadth comparison written (route/breadth-comparison.yaml) from frozen
  records: family-1 branch 7 decision entries / 37 model calls / 22 tool
  executions / 406.054 s recorded vs T29 1 / 4 / 6 / 41.344 s (+2 observer
  calls); currencies not collapsed.

## 2026-09-19 — teeth + gate (correction history preserved)

- run_teeth round 1 (sha e02d5c84…): CRASHED before any attack completed —
  fresh_copy copytree recursed (copying the T29 tree into its own teeth/
  subdir). Preserved as run_teeth.round1-failed.py; no records written.
- run_teeth.round2.py (sha 2c0324f7…): recursion fixed
  (ignore_patterns + "teeth"). Run against verify_breadth round 1
  (sha 3785afff…): T1 REFUSED, T2 REFUSED, T3 FAILED LOUDLY — correctly
  catching a REAL gate defect: the gate's repo-root ancestry applied 5
  dirnames instead of 4, hashing preregs at a nonexistent path. Failed-round
  artifacts preserved under teeth/gate-round1-failed/.
- Gate round 2 attempt (sha b82e44ef…): the sed fix did not match the
  line-wrapped expression (docstring-only change); teeth failed again in T3.
  Preserved as verify_breadth.round2-failed.py; artifacts under
  teeth/gate-round2-failed/.
- verify_breadth.py round 3 (sha e79f9260d823276f4a1604fc488c696999f09f2e98ce6356322aeca632b70d94):
  path computation actually fixed (4 dirnames); no check logic ever changed.
  Teeth re-run: T1 REFUSED (absorb-time gate + gate recomputation), T2 REFUSED
  (hand-set d*=3 vs recomputed 2 + negative-label flip + g3 hand-set all
  refused), T3 REJECTED (flipped observer labels -> substantive findings [],
  route-record byte-identical). attacks.yaml written.
- evidence-manifest.yml (432 files, fail-closed both directions; regenerated
  once to add route/claims-report.yaml — data-only change, gate byte-identical).
- OFFICIAL GATE RUN (from the sea-rs repo root): VERIFY-BREADTH: PASS
  (618 checks), exit 0. Frozen script identities re-confirmed post-gate:
  harness fd555c9e…, verify_breadth e79f9260…, run_teeth.round2 2c0324f7….
- Status: route complete, goal met, all claims bounded per
  route/claims-report.yaml. Result contract b1-result.json follows as the
  last artifact. Remaining per plan: fresh independent adversarial
  confirmation + orchestrator settlement.
