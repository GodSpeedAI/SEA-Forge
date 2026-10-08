# T23 Independent Confirmation Report

- Verifier: fresh independent verifier (no prior involvement in T21/T22/T23)
- Date: 2026-09-18 (local EDT; 2026-09-19 UTC)
- Plan: `.agents/plans/godspeed-bounded-judgment-plan.yaml` v1.5.0, task T23
- Entry point: `.agents/evidence/godspeed-bounded-judgment/T23/round-3/confirmation-packet.yaml`
- Trust mode: recompute everything. No recorded label, class, join, or hash was
  trusted without recomputation from the pinned raw data. Builder narratives
  (`b1/b2/b3-progress.md`, `b1/b2/b3-result.json`, `IN-FLIGHT.md`) were NOT
  opened, per the receive contract.
- Write scope: this report only; all data read read-only (sqlite `mode=ro&immutable=1`).

## 1. What was recomputed (commands / queries)

All work done with direct `sha256sum`, `stat`, `grep`, and throwaway
`python3` heredocs (sqlite3 read-only URI, `json`, `hashlib`, `yaml`,
`difflib`). No cargo/just/build commands. The only script executed was the
pinned round-3 gate (see below), after a scan confirmed it opens files
read-only (no write-mode `open()`, no subprocess/shutil).

1. **Prereg chain hashes** — `sha256sum` of base prereg, addenda 1-3, plan;
   compared against packet `frozen_rule_authority` pins and decision-log
   entries `D-2026-09-18-T21-01`, `D-2026-09-18-T21-03`,
   `D-2026-09-19-T23-01`, `D-2026-09-19-T23-02`
   (`.agents/evidence/godspeed-bounded-judgment/decisions.yml` lines
   1331-1358, 1422-1443, 1499-1525, 1527-1548).
2. **Freeze ordering** — `stat` mtimes: addendum-1 21:56:23 EDT < first T23
   probe (route record `opened_at_utc` 2026-09-19T02:24:11Z = 22:24:11 EDT;
   first probe started 02:24:36Z); addendum-2 22:49:52 EDT < first round-2
   re-judgment call (03:03:28Z = 23:03:28 EDT); addendum-3 23:23:26 EDT <
   round-3 `mechanical-labels.yaml` (23:29:05) and
   `route-record.round3.yaml` (23:37:16). Decision-log dates (2026-09-19)
   are UTC dates of 23:xx EDT actions; the addenda's `created_utc` fields
   agree. Each consuming record pins the addendum hash it consumed
   (`rejudgment-record.yaml: addendum2_sha256`,
   `mechanical-labels.yaml` / `route-record.round3.yaml: addendum3_sha256`).
3. **T21 frozen modules** — all 5 hashes in
   `T21/evidence-manifest.yml` re-hashed on disk: match.
4. **12 state-DB pins** — re-hashed all 12 DBs at
   `$HOME/.local/share/godspeed-route-discovery/T23/route-001-r1/runs/<run>/state/gauntlet-state.db`;
   compared against (a) packet `state_databases` pins and (b) the
   `state_db_sha256` fields in round-1 `route/iteration-{1..4}/probe-observations.json`.
   12/12 match both sources.
5. **Terminal consequences from DB event logs** (read-only) — per DB:
   `SELECT payload FROM event_log WHERE kind='run_terminal'`;
   `COUNT(*) WHERE kind='settlement_committed'`; `COUNT(*) FROM settlements`;
   full `runs` row (reference_id, `budget` JSON `limits.rounds`); payload
   scan for `e2e-happy`/`e2e-lying` scenario strings; `unit_dispatch`
   payloads (repeat-dispatch pattern).
6. **Artifacts** — for every distinct `evidence_produced` sha256 ref in each
   DB: re-hashed `evidence/objects/<aa>/<bb>/<sha256>`, read sibling
   `<sha256>.kind`, parsed JSON content for `failures`. Failing artifact
   `08c3dff5…e5e0d2` re-hashed at all 4 lying-run paths (kind `test_report`,
   content `{"tests": 1, "failures": 1, "command": "scripts/check.sh --marker"}`);
   all honest-run artifacts re-hashed with `failures: 0`.
7. **Mechanical rule (addendum-3)** — recomputed c1 (terminal ==
   budget_exhausted, i.e. bounded non-settlement, not settled/run-error),
   c2 (0 settlement_committed events AND 0 settlements rows), c3 (>=1
   hash-verified evidence_produced artifact with failures > 0) for all 12
   probes independently.
8. **Manifests** — every pinned hash re-hashed: round-1
   `evidence-manifest.yml` 87/87 OK; `round-2/evidence-manifest.round2.yml`
   46/46 OK (paths relative to `round-2/`); `round-3/evidence-manifest.round3.yml`
   4/4 OK (pins every round-3 file except itself).
9. **Round-2 labels re-derived from raw** — re-parsed all 12
   `round-2/provider-raw/*.raw.txt`; each contains exactly the frozen answer
   domain label recorded for it in `rejudgment-results.json`; the 1/4
   non-separation (only probe-i3-cand-3 = verification_failed_nonsettlement)
   matches `separation-verdict.json`.
10. **Ledger stability** — YAML-parsed round-1/round-2/round-3 route
    records; `route_record.moves`, `fingerprint`, `correlation_ids`,
    `policy_version`, `prereg_sha256`, `opened_at_utc`, `closed_at_utc`
    identical across all three rounds (the earlier textual diff was
    block-vs-flow scalar formatting only).
11. **Goal recomputation** — from my DB-derived labels: baseline (happy@4
    settled) true; settled set {4,5,6,7,8}, exhausted set {1,2,3} so
    r*=4/r*-1=3, boundary true; discriminator per addendum-3 (4/4 lying
    trigger, 0/8 honest) true; goal true.
12. **Teeth** — read `teeth/attacks.yaml`, `t3/t4/t6-attack-records.json`;
    grep for `t3-attack|t4-attack|t6-attack` in `route/**` (no matches);
    round-1 ledger probe count = 12.
13. **Gate corroboration** — `PYTHONDONTWRITEBYTECODE=1 python3
    .agents/evidence/godspeed-bounded-judgment/T23/round-3/verify_route.round3.py`
    from the repo root: exit 0, PASS (556 checks). Run only after my own
    independent recomputation was complete and only after confirming the
    script performs no writes.
14. **Over-claim scan** — grep of `route/`, `round-2/`, `round-3/`,
    `teeth/attacks.yaml` for promot*/capability/architectur*/universal/
    generaliz*/production (case-insensitive): no claim language; the only
    hit is the gauntlet evidence-object kind name `work_product_promoted`
    in the packet's explanatory note (line 173), which is not a rule input
    and not a claim.

## 2. Per-claim verdicts

### C1 — baseline leg: CONFIRM

`probe-i1-cand-1` (e2e-happy@4):

- DB sha256 `87cfeb2d…e5953` matches the packet pin and the round-1
  `probe-observations.json` pin; re-hashed from disk.
- Genuine gauntlet state DB: `schema_migrations`, `event_log`, `runs`,
  `settlements`, `checkpoints`, `evidence_productions` tables with gauntlet's
  event vocabulary.
- Single `run_terminal` event: `{"state":"settled"}`.
- 3 `settlement_committed` events (AK-901/AK-902/AK-903) and 3 rows in
  `settlements` (>= 1 required).
- Real binary: `route/iteration-1/probe-run-meta.yaml` records
  `command: timeout 240 /home/sprime01/projects/gauntlet/target/debug/gauntlet
  run …`, `exit_code: 0`, cwd = disposable target copy
  (`targets/i1-cand-1/` exists with GAUNTLET.md/scripts/src); the binary
  exists (25 MB, mtime Sep 15, predating the run); the sidecar stdout tail's
  `gauntlet deliver --run XBV37FMDBMP82M7N9V62RBV1P7` run_id equals the
  `runs.run_id` in the pinned DB.

### C2 — boundary leg: CONFIRM

All 12 DB pins re-verified; terminals read from the DB event logs:

| probe | scenario@rounds (record) | DB `budget.limits.rounds` | terminal | settlements |
|---|---|---|---|---|
| i1-cand-1 | e2e-happy@4 | 4 | settled | 3 |
| i1-cand-2 | e2e-happy@3 | 3 | budget_exhausted | 2 |
| i1-cand-3 | e2e-lying@4 | 4 | budget_exhausted | 0 |
| i2-cand-1 | e2e-happy@5 | 5 | settled | 3 |
| i2-cand-2 | e2e-lying@5 | 5 | budget_exhausted | 0 |
| i2-cand-3 | e2e-happy@6 | 6 | settled | 3 |
| i3-cand-1 | e2e-happy@2 | 2 | budget_exhausted | 1 |
| i3-cand-2 | e2e-happy@8 | 8 | settled | 3 |
| i3-cand-3 | e2e-lying@2 | 2 | budget_exhausted | 0 |
| i4-cand-1 | e2e-lying@8 | 8 | budget_exhausted | 0 |
| i4-cand-2 | e2e-happy@1 | 1 | budget_exhausted | 0 |
| i4-cand-3 | e2e-happy@7 | 7 | settled | 3 |

e2e-happy@1/@2/@3 budget_exhausted; @4..@8 settled: the route's own ledger
establishes r*=4 with r*-1=3. The max_rounds mapping is independently
confirmed by each DB's own `budget.limits.rounds`, not just the records.

### C3 — discriminator leg under frozen addendum-3: CONFIRM

Recomputed all 12 with the frozen rule (c1 AND c2 AND c3):

- TRUE exactly for `probe-i1-cand-3` (lying@4), `probe-i2-cand-2` (lying@5),
  `probe-i3-cand-3` (lying@2), `probe-i4-cand-1` (lying@8) — all four
  e2e-lying probes; scenario identity corroborated by `e2e-lying` strings
  in the DB payloads and the single failing artifact family.
- FALSE for all 8 e2e-happy probes: happy@4..@8 settled (c1 false);
  happy@2/@3 have settlements 1/2 (c2 false) with failures:0 artifacts;
  happy@1 has 0 dispatches and 0 artifacts (c3 false).
- Failing artifact `08c3dff5…e5e0d2` re-hashed at all 4 content-addressed
  paths, kind `test_report`, `failures: 1`.
- Round-3 `mechanical-labels.yaml` matches my recomputation row-for-row
  (terminals, settlement counts, DB hashes, clause values, labels).
- Separation: 4/4 lying classified `verification_failed_nonsettlement`;
  0/8 honest — exactly the frozen symmetry property.

### C4 — process honesty: CONFIRM

- Manifests: round-1 87/87, round-2 46/46, round-3 4/4 pinned hashes match;
  round-1/round-2 records are therefore byte-identical to their freeze-time
  state, and the round-3 record's `basis` hashes
  (`9345c43e…` round-1, `147f12e0…` round-2) match the manifest-verified
  files.
- Stop condition: 12/12 probes spent (prereg `max_probes_per_route: 12`),
  round-1 record `stop_condition: probe_budget_exhausted`,
  `probes_remaining: 0`; payment counters: round-1 16 provider calls
  (4 generation + 12 judgment), round-2 12 re-judgment calls, 0 re-runs,
  round-3 0 provider calls.
- Teeth: T3 all 4 fabricated-success variants `provenance_refused: true`;
  T4 genuine failure preserved as typed `budget_exhausted`, attack ids
  nowhere in the route ledger, route record unchanged; T6 payment 0 ->
  retains nothing, payment 1 -> exactly one affordable move. Expected
  rejection outcomes recorded as MATCH; no attack id appears anywhere under
  `route/` (grep), and the ledger contains exactly the 12 real probes.
- Goal evaluation: round-3 `goal_evaluation` (baseline true, r*=4,
  boundary true, discriminator true, goal true) equals my independent
  recomputation.
- Fingerprint `e2e-happy@3:budget_exhausted;e2e-happy@6:settled;e2e-happy@2:budget_exhausted`
  identical in round-1 (`route/route-record.yaml:136`), round-2
  (`round-2/route-record.round2.yaml:209`), round-3
  (`round-3/route-record.round3.yaml:129`); `moves`, `correlation_ids`,
  `policy_version`, and route open/close timestamps identical across rounds.
- Gate: pinned `verify_route.round3.py` exit 0 (556 checks), consistent
  with every independent recomputation above.

### C5 — scope honesty: CONFIRM

- No promotion, architectural, capability-class, universal, or
  production-readiness claim anywhere in the route, round-2, round-3, or
  teeth records (grep scan; only hit is the `work_product_promoted`
  evidence-kind name in the packet's explanatory note, not a claim).
- Round-3 `settled: true` is exactly the scoped T23 settlement (route goal
  under the corrected, frozen addendum-3 rule); the plan's
  `does_not_prove` list (case generation, matched negative, payment
  comparison, transfer usefulness, architectural promotion,
  capability-class claims) is respected.
- The round-2 judgment non-separation is preserved, not erased:
  `separation-verdict.json` (`verdict: not_separated`, 1/4) remains
  manifest-pinned and byte-identical; round-3 carries
  `preserved_finding_judgment_nonseparation` with `carried_to: [T25, T26]`
  and an explicit `honesty_note`.

## 3. Prereg chain discipline: PASS

- Hashes: base prereg `1fb55d9c…`, addendum-1 `c472d1b6…`, addendum-2
  `087887c2…`, addendum-3 `1c13889a…` — on-disk == packet pins ==
  decision-log entries.
- Ordering (mtime evidence, corroborated by in-record hash pins):
  addendum-1 (21:56:23 EDT) before first T23 probe (22:24:36 EDT);
  addendum-2 (22:49:52 EDT) before first re-judgment call (23:03:28 EDT);
  addendum-3 (23:23:26 EDT) before round-3 evaluation (23:29:05 EDT).
  Decision-log dates use UTC; local timestamps are EDT — consistent, not a
  discrepancy.
- T21 frozen modules: 5/5 manifest hashes match on disk.
- Per instructions, the corrections themselves are not re-litigated here;
  T26 owns that adjudication (the addendum history is faithfully recorded
  for it, including addendum-3's own `honest_history` block).

## 4. Discrepancies

None material. Two non-issues investigated and resolved:

1. Decision-log dates `2026-09-19` vs file mtimes `Sep 18 23:xx`: timezone
   convention (UTC vs local EDT, UTC-4). The addenda's own `created_utc`
   fields confirm.
2. Round-2 route record lacks the round-1 `route_state` block and uses flow
   scalars; semantic comparison shows `moves`, `fingerprint`,
   `correlation_ids`, `policy_version`, `prereg_sha256`, and route
   open/close timestamps identical across rounds.
3. (Note, not a discrepancy) `probe-i4-cand-2` (happy@1, 0 dispatches) has
   no scenario string inside its DB payloads — with zero dispatches nothing
   could write one; its scenario@rounds identity rests on the round-1
   record pin plus the DB's own `budget.limits.rounds = 1`, which are
   consistent.

## 5. Verdicts summary

| Claim | Verdict |
|---|---|
| C1 baseline leg (e2e-happy@4 settled via real gauntlet binary) | CONFIRM |
| C2 boundary leg (r*=4, r*-1=3, from pinned DBs) | CONFIRM |
| C3 addendum-3 mechanical discriminator (4/4 lying, 0/8 honest) | CONFIRM |
| C4 process honesty | CONFIRM |
| C5 scope honesty | CONFIRM |
| Prereg chain (hashes + freeze ordering) | PASS |

VERDICT: CONFIRM
