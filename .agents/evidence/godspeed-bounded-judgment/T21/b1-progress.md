# B1 progress — T21 native route-discovery contract freeze

## Checkpoints

- 2026-09-18: Started. Read the frozen prereg in full; recomputed its sha256
  from disk (1fb55d9c56d87d94adc4850bc5ca713db3e600297fbdc997accfab54c98ea8b5,
  matches the value quoted by the orchestrator). Read the plan's T21 entry
  (v1.5.0), the T15 frozen mechanism (batch-round1-failed.sh), T15 prereg
  answer domain, and the T16 harness conventions (plain stdlib + PyYAML).
- 2026-09-18: Confirmed provenance sources on disk:
  ../gauntlet/tests/fixtures/mock_runner_scenarios/{e2e-happy,e2e-lying,e2e-forged-report}.yaml
  and ../gauntlet/examples/demo-calculator/GAUNTLET.md (max_rounds: 8 lever,
  runners.default lever).
- 2026-09-18: Authored route_contracts.py (8 typed records, strict
  field-set validation, named refusal reasons, lossless YAML/JSON
  round-trips, route_fingerprint helper), admission.py (classify with
  documented precedence: authority-denied -> schema-invalid -> prior-denial
  -> unreachable -> unaffordable -> redundant -> admitted), selection_policy.py
  (selection-policy-v1: eligibility list in prereg order; ordering as tiers
  rule1 > rule3 > rule2 > other; single retention), operations_catalog.yaml
  (24 operations, full 3x8 coverage, per-operation provenance, neighborhood
  rule restated as data), verify_contract.py (gate: schema round-trips +
  refusal classes, 6 admission vectors, 8 selection unit vectors, provider
  source scan, catalog integrity, prereg digest, manifest hash check).
- 2026-09-18: Pre-flight module smoke test (imports + vector logic exercised
  directly; verify_contract.py itself not yet run).

## Interpretive readings (also listed under deviations in b1-result.json)

1. Ordering rules 1-3 are implemented as tiered precedence: rule 1 (completes
   an unmet route-goal prerequisite) > rule 3 (r*-1 boundary priority) >
   rule 2 (midpoint heuristic) > other candidates. The prereg gives rule 3
   "has boundary priority" wording; reading it as subordinate to the midpoint
   heuristic would prevent the frozen boundary goal (r* AND r*-1 both probed)
   from closing.
2. "insufficient_evidence disqualifies a candidate whose expected_role is
   boundary_probe or whose consequence would be cited by the settlement
   criterion": route_goal_frozen cites baseline, discriminator, AND boundary
   consequences, so the clause covers all three expected_roles;
   insufficient_evidence always disqualifies.
3. redundant = (scenario, max_rounds) already probed on THIS route
   (neighborhood rule (a)). Cross-route duplication is measured and recorded
   (novelty metric) but never auto-rejected — the prereg freezes no automatic
   duplicate rejection.
4. unreachable = the candidate's scenario does not resolve to a pinned
   fixture path in the operations catalog. admission.py does no filesystem
   access; catalog fixture existence is verified mechanically by
   verify_contract.py instead.
5. Admission classification precedence is fixed and documented in the module
   docstring; each gate vector exercises exactly one violation.
6. probe_observation.terminal_class uses the task-contract vocabulary
   (settled | budget_exhausted | other-nonsettlement | run_error); judgment
   labels use the frozen answer domain; the coarse-to-fine terminal-to-label
   mapping belongs to T22's judgment adapter and is intentionally absent here.
7. selection_record.policy_version is pinned to selection-policy-v1 by
   schema; a record under any other version is refused.
8. A candidate missing identity fields still yields a schema-valid
   admission_decision using placeholder identity ("unknown-candidate" /
   "unidentifiable-input") with the defect named in the reason, because every
   emitted decision must itself be a typed record.
9. route_state enforces the frozen bounds (5 retained moves, 12 probes,
   probes_remaining 0..12) as schema constraints.

## Corrections

(none — the gate passed 67/67 on its first and only official run; no script
was modified after the manifest freeze)

## Official verification (2026-09-19, final tree)

- `python3 .agents/evidence/godspeed-bounded-judgment/T21/verify_contract.py`
  (from /home/sprime01/projects/sea-rs) — PASS, 67/67 checks, exit 0
  (includes manifest/hash-check: all 5 frozen files verified against on-disk
  sha256, and prereg/frozen-sha256-matches-disk).
- `cd /home/sprime01/projects/gauntlet && just ruler` — PASS, exit 0
  (answer key and manifest digests match the frozen lock).
- `python3 -c "import yaml; yaml.safe_load(open('.agents/evidence/
  godspeed-bounded-judgment/T21/operations_catalog.yaml'))"` — OK, exit 0.

Nothing was executed beyond these read-only checks: no route loop, no probe,
no provider call, no gauntlet run, no cargo/nextest/just besides the single
ruler tripwire.
