# T21 IN-FLIGHT — native route-discovery contract freeze (Builder B1)

task: T21 (plan godspeed-bounded-judgment v1.5.0)
builder: B1
evidence_dir: .agents/evidence/godspeed-bounded-judgment/T21/
prereg: .agents/preregistrations/godspeed-bounded-judgment-T21.prereg.yaml
prereg_sha256: 1fb55d9c56d87d94adc4850bc5ca713db3e600297fbdc997accfab54c98ea8b5
  (recomputed from disk at start of work, matches the frozen value)

## Contract

Freeze the native route-discovery contract as task-local Python modules plus a
gate script, and prove the contract against itself WITHOUT executing anything:
no route loop, no probe, no provider call (`opencode`), no gauntlet run, no
cargo/nextest/just (single exception: the one read-only `just ruler` digest
tripwire in ../gauntlet).

## Deliverables

- [x] IN-FLIGHT.md (this file)
- [x] route_contracts.py — typed record schemas + validation + lossless YAML/JSON round-trips
- [x] operations_catalog.yaml — 24 operations (3 scenarios x max_rounds 1..8) with derivation provenance + neighborhood_rule as data
- [x] admission.py — classify() implementing the frozen neighborhood_rule + payment rule
- [x] selection_policy.py — selection-policy-v1 frozen deterministic rule, no provider interface
- [x] verify_contract.py — gate script (runs from the sea-rs repo root)
- [x] evidence-manifest.yml — frozen 2026-09-19T01:13:26+00:00 from the final script tree
- [x] b1-result.json — written as the final step (see its own content)

## Status log

- 2026-09-18: started. Prereg read in full; sha256 verified. Fixtures confirmed
  on disk (e2e-happy.yaml, e2e-lying.yaml, e2e-forged-report.yaml under
  ../gauntlet/tests/fixtures/mock_runner_scenarios/; GAUNTLET.md max_rounds
  lever at ../gauntlet/examples/demo-calculator/). T15 mechanism script
  (batch-round1-failed.sh) read as the frozen case-execution mechanism.
  Answer domain confirmed identical to the T15 prereg vocabulary.
- 2026-09-18: All five deliverable scripts authored; pre-flight module smoke
  test (23 checks) passed without any correction needed.
- 2026-09-19: evidence-manifest.yml frozen from the final script tree. No
  script modified after this point. Official gate run pending.
- 2026-09-19: Official verification complete — verify_contract.py PASS
  (67/67, exit 0, on the manifest-pinned tree), `just ruler` PASS (exit 0),
  operations_catalog.yaml parses (exit 0). No corrections were needed.
  b1-result.json follows as the last write.

## SETTLED (orchestrator, 2026-09-18)

- Gate re-run personally on the final tree: verify_contract.py PASS 67/67 (exit 0).
- Scope audit: only .agents/evidence/godspeed-bounded-judgment/T21/ added; no crate/spec/plan file touched by the builder; just ruler PASS.
- Manifest hash-check PASS (fail-closed); prereg cross-ref matches D-2026-09-18-T21-01 (1fb55d9c…8ea8b5).
- 9 interpretive readings reviewed by the orchestrator; all faithful-to-restrictive (rule 3 outranks rule 2 so the frozen boundary goal can close; insufficient_evidence disqualifies all roles cited by the settlement criterion; bounds enforced in route_state schema).
- T21 SETTLED (P1, builder confirmation + orchestrator verification). Task-local IN-FLIGHT closed; execution of the loop begins at T22.
