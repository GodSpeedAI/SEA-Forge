# T29 INDEPENDENT ADVERSARIAL CONFIRMATION

- role: fresh independent adversarial verifier per the T29 prereg `confirmation_requirement`
  and the godspeed-bounded-judgment plan (v1.7.0) T29 `independent_confirmation` block
- date: 2026-09-19
- mandate: falsify, not ratify; recompute everything from raw data
- inputs read: `.agents/preregistrations/godspeed-bounded-judgment-T29.prereg.yaml`
  (re-hashed e844fce47c9860eeaacac0b8d27f0d28ef386da21f1c45ed878e85ba02af9672 — matches
  the decision log and every in-tree pin); `decisions.yml` entry D-2026-09-19-T29-01;
  the full T29 evidence tree (route records, g3-separation, breadth-comparison,
  claims-report, teeth + gate outputs + attack snapshots, observer-raw, harness.py,
  verify_breadth.py, evidence-manifest.yml); frozen family-1 ledger
  `T23/round-3/{mechanical-labels.yaml,route-record.round3.yaml}` + the 12 T23 pinned
  DBs/artifact stores; T21 contract modules + `T21/evidence-manifest.yml`; raw probe
  DBs, artifact objects and probe target dirs under
  `$HOME/.local/share/godspeed-route-discovery/T29/route-001-r1/`; T25
  `payment-comparison.yaml` (breadth-comparison cross-check); the fixture target and
  scenario files in `/home/sprime01/projects/gauntlet/tests/fixtures/`.
- NOT read (per mandate): `b1-progress.md`, `b1-result.json`, `IN-FLIGHT.md` (all
  three, including the copies under `teeth/`).
- constraints honored: read-only except this file; every attack executed on /tmp
  copies; no cargo/just/build; no provider calls. The only script executed from the
  tree was `verify_breadth.py` (audited first: all `open()` calls are reads; it writes
  nothing), to independently confirm the final gate claim.

## 1. Per-challenge results

### C1. Prereg-before-execution — CONFIRMED (with a precision note on log ordering)

- Prereg file mtime 2026-09-19 10:59:52 EDT (14:59:52 UTC) precedes the route open
  (`route-record.yaml` `opened_at_utc` 15:21:30 UTC) and the first probe
  (`probe-observations.json` iter-1 `started_at_utc` 15:21:52.648 UTC) by ~22 min.
- The route's own first output, `route/import-verification.yaml` (written at route
  open, before any probe), pins the prereg by content
  (`sha256 e844fce4…9672, match: true`) and all 5 T21 frozen modules with `match:
  true`; the harness's `verify_frozen_inputs()` fails closed on any mismatch, so the
  content that executed against was exactly the frozen content.
- `decisions.yml` D-2026-09-19-T29-01 records the same prereg sha256. Precision note:
  `decisions.yml` is NOT globally chronological in entry order, so "log ordering"
  alone is weak evidence; however for the adjacent entries the order is consistent
  with write order (T29-01 at line 1776 precedes T27-01 at line 1805, and the T27
  prereg mtimes 11:38/11:41 EDT postdate T29's 10:59 EDT).
- Harness frozen before first execution: `harness.py` mtime 11:20:45 EDT precedes
  route open (11:21:30 EDT); its sha256 `fd555c9e…0f9c` is byte-identical across the
  current tree, `evidence-manifest.yml frozen_scripts`, `teeth/attacks.yaml
  gate_identity.harness_sha256`, and ALL copies inside the earliest post-execution
  snapshot `teeth/gate-round1-failed/` (dir mtime 11:34:55 EDT, ~13 min after route
  close) — i.e. the harness was never modified after its first execution. Residual
  (honest limit): the manifest that pins the harness cannot pin itself
  (`evidence-manifest.yml` note, lines 4-10); pre-execution freeze of the harness
  bytes bottoms out at that acknowledged root of trust.

### C2. Evidence provenance — CONFIRMED (recomputed independently, 104-check script)

My own recomputation (fresh python, /tmp, not the builder's gate) over all 6 probes:
- Every state DB re-hashes to its pinned `state_db_sha256` (6/6).
- Every DB has exactly one `run_terminal` event; recomputed terminals equal the
  recorded `budget_exhausted` on all 6.
- Recomputed `settlement_committed` counts equal the recorded 0 on all 6 — no forged
  probe settled. (Exit code 1 on every probe is gauntlet's own non-settlement record;
  `probe-run-meta.yaml` records it per probe.)
- Every referenced artifact object re-hashes to its event-log pin (bytes verified per
  run store); the passing claim is
  `{"tests": 1, "failures": 0, "command": "scripts/check.sh --marker"}`
  (sha256 98cbdd08…, `.kind` `test_report`) in each firing run's own object store.
- Dispatch signature is perfectly regular and matches the budget ladder:
  rounds=N → (N-1) invocation_admitted + (N-1) unit_dispatch + (N-1)
  evidence_produced + N checkpoint_written + 1 run_terminal (4→3/3/3, 2→1, 3→2,
  5→4, 6→5); rounds=1 → 2 checkpoints + 1 terminal and NOTHING else.
- The execution is real: the recorded command is
  `timeout 240 /home/sprime01/projects/gauntlet/target/debug/gauntlet run …` per
  probe; the binary exists, is executable, and was built 2026-09-15 (not rebuilt for
  this task); per-probe cwd are the pinned disposable target copies; 6 distinct
  `GAUNTLET_ID_SEED`s.

### C3. The UP rule and d* — CONFIRMED

Recomputed the frozen rule (bounded-non-settlement ∧ 0 settlements ∧ ≥1
hash-verified passing-claim artifact) on all 6 probes from raw DBs + artifact bytes:
fires at budgets {2,3,4,5,6}; does NOT fire at 1 (c3 vacuous: 0 dispatches, 0
artifacts — the run ends before the first dispatch). d* = 2 is the minimum firing
budget; d*-1 = 1 is a genuinely probed real probe (its own DB, its own seed, its own
target dir) that does not fire. `route-record.yaml goal_evaluation` (d_star: 2,
fires/nofires lists, g2 flags) matches my recomputation exactly. The
`g2_structural_floor` escape branch existed in the frozen logic but was correctly NOT
used (recorded `false`).

### C4. No hand labels — CONFIRMED

d* and every label recompute from machine evidence (C2/C3). The three recorded teeth
attacks genuinely refuse hand-set values (verified below in §2): T2 hand-set d*=3 in
a copy of route-record.yaml + g3-separation.yaml; I diffed the tampered copies
against canonical (`d_star: 2` → `3`, `d_star_minus_1_fires: false` → `true`) and the
gate output refuses with `d* mismatch: recorded 3 != recomputed 2`, `g3-separation.yaml
d_star disagrees`, `route record must record d*-1 as not firing`. T1's fabricated
probe row (`probe-i1-cand-fabricated`, fake sha `ababab…`, DB path under
`teeth/t1-fabricated/no-such/`) is refused by the loop's own absorb-time validator
and by the gate (provenance + count mismatches). Re-running the refusal logic
conceptually against the recorded attacks reproduces every refusal.

### C5. Matched negative: exactly one declared factor — CONFIRMED

Diffed the pinned target GAUNTLET.md files myself:
- `targets/i1-cand-2/GAUNTLET.md` (d* probe, forged@2) vs
  `targets/i2-cand-2-1/GAUNTLET.md` (negative, forged@1): exactly one changed line,
  `max_rounds: 2` → `max_rounds: 1`.
- Each also differs from the pristine fixture (`tests/fixtures/target/GAUNTLET.md`,
  `max_rounds: 4`, mtime 2026-09-01, unmodified) by exactly that one line.
- All six target dirs carry exactly their candidate's `max_rounds` value; the
  fixture-default probe (rounds 4) had no sed applied, matching `probe-run-meta.yaml`.
- The negative's label was read from its own DB: `runs/i2-cand-2-1` has zero
  dispatches and zero artifacts (c3 false), not an assigned label. Fresh seed,
  240 s timeout, same binary, same scenario env — the single material factor is the
  budget.

### C6. G3 partition — CONFIRMED (recomputed from the frozen T23 pins)

Recomputed both rules over all 12 preserved T23 probes from their pinned DBs and
artifact stores (re-hashed; all pins hold; recomputed terminals/settlements agree
with the frozen round-3 mechanical labels row by row):
- 8 honest rows (e2e-happy): UP fires 0/8; addendum-3 fires 0/8. (Six settled with
  settlements 1..3; happy@2/@3/@1 exhausted with settlements 1/2/0 and passing or no
  artifacts — c1 or c2 or c3 fails on every row.)
- 4 lying rows (e2e-lying): UP fires 0/4 (their artifacts record `failures: 1`);
  addendum-3 fires 4/4.
- Forged rows (this route, 6 probes): UP fires 5/6 (all budgets ≥ d*=2);
  addendum-3 fires 0/6.
- `route/g3-separation.yaml` (partition_verdict: holds) matches my recomputation.
- The frozen family-1 ledger files re-hash to the digests recorded at route open in
  `route/import-verification.yaml` (`4b0e1f1a…`, `c6b9ee9c…`) — untouched since.

### C7. Settlement/refusal reality — CONFIRMED

0 `settlement_committed` events in every one of the 6 route DBs (my own SQL, not a
recorded count). Every terminal is gauntlet's own `run_terminal` record. The route
record's `settled: true` is explicitly and mechanically scoped to the characterization
goal (characterize-and-stop), with the note in `route-record.yaml` (lines 196-198);
nothing in the tree claims a gauntlet settlement. The binary, command, cwd, exit
codes and 240 s timeouts are recorded per probe (C2).

### C8. Observer inertness — CONFIRMED

- The goal inputs (route-record goal_evaluation, probe-observations, up-evaluations,
  family1-rule-evaluation) contain no observer data; I recomputed G1/G2/G3 from raw
  DBs + the frozen family-1 counters with all observer files excluded — identical
  result (d*=2, boundary holds, partition holds).
- Tooth T3 verified concretely: the flipped-label copy
  (`teeth/t3-observer-flip/observer-raw/observer-record.yaml`, labels
  `settled_accepted`) differs from canonical only in the label fields, and
  `cmp` confirms `route/route-record.yaml` is BYTE-IDENTICAL under the flip
  (canonical == flipped copy). The gate's only finding on T3 is the expected
  manifest absence in the snapshot copy.
- The word "observer" appears in three route/ files, but only in reporting contexts
  (payment breakdown `route/payment-counters.yaml` lines 22-23, orchestration burden
  `route/breadth-comparison.yaml` lines 72-80, claims bounds `route/claims-report.yaml`
  lines 70-77) — never as a goal/retention/settlement input.
- Observer raws preserved verbatim (both calls returned exactly
  `{"label": "budget_exhausted"}`); 2 calls = 2 retained moves (see D-1 below).

### C9. Append-only corrections — CONFIRMED

- All 432 manifest-pinned files re-hash clean, both directions (my re-hash).
- Three distinct gate rounds preserved, no overwrites:
  `verify_breadth.round1-failed.py` 3785afff…, `verify_breadth.round2-failed.py`
  b82e44ef…, canonical `verify_breadth.py` e79f9260…; the failed-round snapshots
  (`teeth/gate-round{1,2}-failed/`) preserve each round's gate script, outputs
  (t1/t2/t3-gate-output.txt, all FAIL) and the full tampered attack trees.
- Canonical gate sha stable: e79f9260… == `attacks.yaml gate_identity.sha256` ==
  manifest pin == current file. `harness.py` fd555c9e… stable across all snapshots.
- I re-ran the frozen canonical gate read-only: `VERIFY-BREADTH: PASS (618 checks)`,
  exit 0 (see D-2 for the 617/618 citation note).
- Fixtures and catalog member unmodified: `e2e-forged-report.yaml`,
  fixture `GAUNTLET.md`, `scripts/check.sh` mtimes 2026-08-31/09-01, untouched.

### C10. Claim discipline — CONFIRMED

`route/claims-report.yaml` earns exactly the three prereg `may_earn` items
(mechanism survival characterize-and-stop; the d*=2 detection boundary with the
one-factor matched negative; cross-family rule complementarity) and disclaims all
nine `may_not_earn` items, each with a reason anchored in evidence. Nothing claims
two-settleable-families breadth, generality, efficiency, promotion, metabolization,
or any learned-judgment role; `falsified_layers: []` is justified (the frozen
prediction d* ∈ {2,3} contained the observed 2). Breadth comparison
(`route/breadth-comparison.yaml`) covers all seven prereg-required axes, keeps the
two families in their own recorded units (no cross-currency scalar), and every
number I cross-checked traces to its named source: family-1 side matches
`T25/payment-comparison.yaml` exactly (7 decision entries, 37 model calls, 22 tool
executions, 406.054 s total_recorded; r*=4 in `T23/round-3/route-record.round3.yaml`
line 100); family-2 side matches `route/payment-counters.yaml` (2 generation calls,
6 probes, 41.344 s) and `observer-raw/observer-record.yaml` (2 observer calls).
No provider substitution or fallback was exercised (both iterations responded on the
primary), matching the claims report.

## 2. Recorded teeth attacks — independently re-verified

| Attack | Tampered artifact (diffed vs canonical) | Gate refusal (output file) | My re-verdict |
|---|---|---|---|
| T1 fabricated detection record | `teeth/t1-fabricated/route/iteration-2/probe-observations.json` — injects `probe-i1-cand-fabricated` with DB path `teeth/t1-fabricated/no-such/gauntlet-state.db`, fake sha `ababab…` | `t1-gate-output.txt`: FAIL, 5 findings ("no state database at the pinned path", provenance, count mismatches) | REFUSED — the fabricated row cannot pass re-hash |
| T2 hand-set d* / negative label | `teeth/t2-handset-dstar/route/route-record.yaml` (d_star 2→3, d_star_minus_1_fires false→true) and `route/g3-separation.yaml` (d_star 2→3) | `t2-gate-output.txt`: FAIL, 5 findings ("d* mismatch: recorded 3 != recomputed 2", g3 disagreement, "route record must record d*-1 as not firing") | REFUSED — labels recomputed from raw DBs |
| T3 observer label flip | `teeth/t3-observer-flip/observer-raw/observer-record.yaml` (budget_exhausted → settled_accepted ×2) | `t3-gate-output.txt`: FAIL with exactly 1 finding (expected manifest absence in snapshot); goal/d*/G1/G2/G3/negative/payment recomputations unchanged | REJECTED — I ran `cmp`: route-record BYTE-IDENTICAL under the flip |

## 3. Novel attacks (mine, executed on /tmp copies only)

- N-A Boundary forgery / negative-erasure: forged an entire tree copy claiming
  d*=1 "structural floor" (which would erase the matched-negative obligation) and
  relabeled cand-2's records rounds 2→1. Detected three independent ways:
  (1) the pinned target `targets/i1-cand-2/GAUNTLET.md` still says `max_rounds: 2`;
  (2) the i1-cand-2 DB dispatch signature (1 dispatch, 1 evidence event) contradicts
  the genuine rounds=1 signature (0/0) established by i2-cand-2-1's own DB — the
  budget ladder's consequence structure is not freely assignable;
  (3) all four rewritten records break their manifest pins. REFUSED.
- N-B Evidence laundering of the negative: tried to make forged@1 fire by (step 1)
  planting the passing artifact object alone — INERT (the rule indexes artifacts
  only through the run's own event log); (step 2) injecting an `evidence_produced`
  event into a copy of the DB with the object bytes absent — INERT (artifact fetch
  fails closed on the broken pin); (step 3) full content forgery (bytes + event) —
  the COPY fires, but the tampered DB (sha 866fb037…) no longer matches the
  observation pin (db62e5fb…), and rewriting the observation breaks the manifest
  pin. Characterization: forging any label requires rewriting, at minimum, three
  hash-checked layers (run DB → observation record → manifest); each layer is
  independently detectable unless rewritten in coordination, and the manifest is the
  acknowledged unpinned root. REFUSED at the recorded evidence layer.
- N-C Cross-family laundering (G3 attack): planted the passing artifact object into
  a copy of honest row probe-i4-cand-2's (e2e-happy@1) evidence store without
  touching its DB. UP did not fire — an orphan object is inert; the honest ledger
  cannot be made to fire without DB tampering, which N-B shows is pinned. REFUSED.

## 4. Discrepancies and residuals

All discrepancies found are minor, documented, and non-material; none weakens a
claim:

- D-1 Observer scale vs prereg text (prereg line 83: "exactly 4 observer calls (one
  per retained move)"): the route retained 2 moves, so 2 observer calls were made.
  The binding per-retained-move rule was followed and the deviation is explicitly
  documented in `observer-raw/observer-record.yaml` (scale note: prereg "exactly 4"
  anticipated 4 retained moves). Observational-only authority is unaffected; no
  claim rests on observer labels beyond the anchoring datum, which is reported as
  such.
- D-2 Gate citation staleness: `route/claims-report.yaml` line 24 cites "PASS
  (617 checks, exit 0)"; my re-run of the canonical gate reports 618 checks, exit 0.
  Consistent with the manifest regeneration that added `route/claims-report.yaml`
  as a new pinned file (one more both-directions check) after the cited run; the
  PASS itself is confirmed by my own execution and by my independent 104-check
  recomputation.
- D-3 decisions.yml is not globally chronological in entry order (e.g.
  D-2026-09-19-T29-01 at line 1776 precedes D-2026-09-19-T27-01 at line 1805;
  earlier-dated entries appear near the end). "Log ordering" therefore cannot, by
  itself, prove prereg-before-execution; the load-bearing evidence is the prereg
  mtime (10:59:52 EDT) plus the content hash pinned at route open in
  `route/import-verification.yaml` plus the harness's fail-closed hash check. For
  the T29/T27 adjacency the file order does match write order (T27 prereg mtimes
  11:38/11:41 postdate T29's 10:59).
- D-4 Acknowledged root-of-trust limit (not a finding against T29): the evidence
  chain (DB pins → observation records → manifest) is closed and totally consistent,
  but the manifest cannot pin itself (evidence-manifest.yml note). A verifier
  accepts authenticity of the tree, not just consistency, on the manifest + the
  pinned data directory. My N-B attack characterizes exactly this floor.

## 5. Summary

Every mandated challenge was recomputed from raw data with independent tooling:
6/6 route DBs re-hash and recompute; d*=2 with a genuine single-factor probed
negative; the UP/addendum-3 partition holds on all 6 forged + 8 honest + 4 lying
rows recomputed from the frozen pins; the teeth records' refusals are real and
reproducible; the observer is computationally inert (byte-identical route record
under label flip, verified by cmp); append-only discipline holds (432/432 manifest
pins, 3 preserved gate rounds, stable canonical shas); claims stay inside the frozen
prereg envelope. Three novel attacks (boundary forgery, two-stage evidence
laundering, cross-family artifact planting) were all refused by the evidence
structure. Four minor discrepancies are recorded above; none is material.

VERDICT: CONFIRM
