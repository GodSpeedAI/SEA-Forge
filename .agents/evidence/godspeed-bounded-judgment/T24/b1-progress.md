# T24 builder progress notes (B1)

Append-only working notes. Durable decisions and their reasons are recorded
here as they are made.

## Round 1

- Context loaded: plan v1.5.0 T24/T25 entries; T21 prereg + addenda 1-3
  (hash-verified against the frozen sha256s); T23 round-3 settled route
  record (sha256 c6b9ee9cff7c8f2047dd29e2eef5576f9b113d72d5f8618c6849ec618bb27097);
  T21 route_contracts.py (sha256 0d802387...) + T23 provenance.py
  (sha256 0baa7f63...) imported UNCHANGED; T22/T23 payment counters and
  T15/T16 corpus records read for the T25 payment/novelty work.
- File-role split decided: `candidate-case.yaml` is the FROZEN candidate
  envelope (case representation with replay fields explicitly empty);
  `replay-record.yaml` carries the replay run evidence; `accepted-case.yaml`
  / `rejected-case.yaml` each carry the bare typed T21 generated_case_record
  (exact 10-field schema, validated by route_contracts.validate) so the
  frozen contract module applies to the final artifact verbatim. The T21
  generated_case_record schema requires an exact field set, so the candidate
  envelope (which carries the target recipe, observer horizon, frozen env
  block, prompt config) cannot itself be that schema before replay fills the
  typed fields; the envelope pins everything and the typed record is
  materialized by the replay from it.
- The case is derived SETTLEMENT-FIRST from the route's own settled ledger:
  the route's baseline basis (e2e-happy@4 settled, 3 settlement_committed
  events; also r*=4 with r*-1=3 budget_exhausted) makes (e2e-happy, 4) the
  one case for which route t23-route-001 is a legitimate solution. The
  provider ladder is used exactly as frozen (deepseek-clinepass primary, ONE
  muse substitution on provider_error/timeout/schema_failure, declared
  deterministic fallback): the provider is asked for the case derivation and
  its answer is admitted only if it agrees with the mechanical derivation;
  acceptance is NEVER the provider's assertion - it is the independent
  replay's real settlement or nothing.
- Candidate id: t24-case-001; correlation_id: t24-case-001. Seeds: replay
  seed t24-case-001-replay-r1-seed; T7 attack seed t24-t7-attack-seed; both
  checked unique against every GAUNTLET_ID_SEED recorded in T22/T23
  probe-run-meta sidecars and teeth records.
- EXECUTION RESULTS (round 1, all append-only):
  - generate_case.py: primary deepseek-clinepass responded ok on the first
    ladder step (no substitution, no fallback); its derivation echoed
    (e2e-happy, 4) with a ledger-faithful rationale. Candidate frozen sha256
    6bf2610c12a609cee98d48ad571baf2fc58d383232eb72bdb9275b245ea25977 at
    2026-09-19T04:08:04Z (freeze pin attests no replay record existed).
  - replay_case.py: fresh copy, seed t24-case-001-replay-r1-seed (fresh vs
    12 prior recorded seeds), no budget sed needed (case value = fixture
    default 4); replay -> settled, 3 settlement_committed, exit 0, wall
    0.505s; provenance.recompute clean; accepted-case.yaml written.
  - run_tooth.py (T7): attack case e2e-happy@2 replayed for real ->
    budget_exhausted, 1 settlement (exactly mirrors the route's own
    probe-i3-cand-1), acceptance path rejected (accepted=false with
    invalid_reason), rejected record under teeth/ only; developmental
    records byte-identical before/after.
  - verify_case.py: pre-freeze trial failed ONLY on the then-missing
    manifest (by design; gate unmodified afterwards); official gate after
    the manifest freeze: PASS 154 checks, exit 0.
  - Whole-task gauntlet executions: 2 (1 candidate replay + 1 T7 attack
    run); provider calls: 1 (generation primary); wall: see
    payment-counters.yaml + replay/tooth records.

