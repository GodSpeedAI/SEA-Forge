# T25 builder progress notes (B1)

Append-only working notes. Durable decisions and their reasons are recorded
here as they are made.

## Round 1

- Context loaded: plan v1.5.0 T25 entry; prereg matched_negative_mutation
  (single factor: payment capacity r* -> r*-1; label read from the
  run_terminal, never assigned) + novelty_metric (fingerprint = ordered
  retained-move sequence of (scenario, max_rounds) plus the final terminal
  of each; novel | near-duplicate (same operation set, different order) |
  structural duplicate (equal fingerprint); duplicates measured, no
  auto-rejection); T24 accepted-case.yaml (sha256
  02927ee42a4bf1858083a9179b4d812c2313657fac6a503cae8578643010ed58) and its
  replay record (positive side of the pair); T22/T23 payment counters; T15
  corpus manifest (9 runs: settled x3, budget_exhausted x6; clean round-3
  batch; rounds 1-2 preserved failed) + T15 prereg case table (budget
  dimensions: defaults, max_rounds 1 (C1,C2), 2 (C3)); T16 expansion
  addendum (N=25: e2e-happy x9 settled, e2e-lying x3, budget-r1 x2,
  e2e-forged-report x2; generation procedure = the same fixture recipe) +
  expanded manifest + reports; decisions.yml for the operator-decision
  counts.
- Baseline-side honesty rule: quote only what the T15/T16 records actually
  record (corpus sizes, rounds history, recorded provider answers where
  present); wall clock and other unobservable quantities are marked n/a with
  no estimates.
- Negative design: the single mutated factor is max_rounds 4 -> 3 (the
  route's discovered r*-1). Everything else identical: scenario, fixture
  recipe, frozen env block shape, timeout, command. Fresh seed
  t25-negative-r1-seed checked against ALL recorded seeds (T22, T23, T24
  replay + T7). Prior expectation on the route's own ledger: e2e-happy@3
  budget_exhausted with 2 honest settlements (probe-i1-cand-2) — but the
  negative's label is READ from the new run's run_terminal, never assigned;
  if the run settles, that is recorded honestly and the gate fails the
  matched-pair requirement.
- payment-comparison.yaml / novelty.yaml are assembled BY the gate from
  recorded evidence files (deterministic content, no wall-clock timestamps
  inside), then byte-verified on re-runs; they are gate outputs and are
  excluded from the manifest's unpinned-file walk like b1-result.json, while
  every other T24/T25 artifact is hash-pinned fail-closed.
- EXECUTION RESULTS (round 1, all append-only):
  - run_negative.py: the single-factor diff was computed and recorded
    (exactly bounded_input.max_rounds 4 -> 3); negative run ->
    budget_exhausted with 2 settlement_committed events, exit 1, wall
    0.303s; DB pinned sha256 097e1af5bad144c661c6c11126a6077e0ccbddedde654ba9e0702f885bf19d88;
    seed fresh vs 14 prior recorded seeds. The actual non-settlement
    consequence held on the first run.
  - run_tooth.py (T8): duplicate classification structural-duplicate,
    detected, no auto-rejection; matched-pair.yaml unchanged.
  - verify_payment.py: pre-freeze trial failed ONLY on the then-missing
    manifest (by design; gate unmodified afterwards); official gate after
    the manifest freeze: PASS 137 checks, exit 0. Assembled counters (all
    from recorded files): 7 branch decision entries; wall 406.054s recorded
    (T23 r2/r3 n/a); 37 model calls (7 generation + 30 judgment); 22 tool
    executions; 3 failed executions (T22 round-1); baseline quoted from
    T15/T16 records (9 -> 25 runs; 21 baseline decision entries; wall n/a).
    Novelty: novel (grid levels [1,2,4]; 5/6/7 never probed; route levels
    2/3/6 with 3 and 6 outside the grid).
  - Whole-task gauntlet executions: 1 (the negative).

