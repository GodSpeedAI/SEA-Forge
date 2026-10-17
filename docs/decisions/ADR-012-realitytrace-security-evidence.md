# ADR-012: RealityTrace security evidence contract

Status: contract accepted; first vertical slice (SEC-12 denied write) implemented.

## Context

SEA Forge owns identity, authority, enforcement, trusted resource
observation and settlement, recorded in its append-only hash-chained ledger.
RealityTrace (`GodSpeedAI/RealityTrace`, crates `sxr-*`) owns the structured
comparison of a prior expectation with observed consequences. An independent
auditor (human, deterministic verifier or separate agent) owns the audit
verdict; the governance/settlement process owns consequential decisions.
RealityTrace must never manufacture missing observations, infer success from
authorization, or issue the verdict.

RealityTrace already ingests SEA Forge's `OperationalSettlement` (E6) through
`sxr_core::proof_ingest::ingest` together with a `ProofCompleted` (E7), and
emits `EvidenceRecorded` (E8) via `sxr_core::evidence_emit`, whose
`difference` is the digest comparison of expected and observed outcome.

## Decisions

1. **No new ingest surface, daemon or store in SEA Forge.** The bridge is the
   library crate `sea-forge-realitytrace`, depending on `sxr-core` pinned by
   git rev. No execution crate depends on it. SEA Forge's ledger stays the
   only authority record; RealityTrace records are evidence about it.
2. **Expectation is frozen before the attempt.** A
   `security_frozen_expectation` ledger record (schema
   `sea-forge.security-trace/v1`) binds contract and case IDs, the work
   request, and the frozen suite (`tag`, `commit`, `manifest_sha256`) to an
   `expected_outcome` stated in the trusted observer's output form. Its
   ledger ordinal must precede the authorized invocation.
3. **Claims and observations are separate records.** The executor's
   `ExecutionObservation` (`security_submitted_observation`) and the
   workload's own report (`security_workload_report`) are claims. The
   launcher-side resource digests (`security_resource_observation`) and the
   settlement's `resource_observation` witnesses are SEA Forge
   observations, attributed by ledger `writer_identity_ref`.
4. **E7 carries only ledger-derived values.** `proof_completed` refuses when
   the expectation is missing, late, or non-canonical, or when the supplied
   settlement is not byte-identical to the ledger's
   `security_returned_settlement`.
5. **Missing evidence stays missing.** `project` lists absent records under
   `unresolved`; nothing defaults to success.
6. **The auditor re-verifies independently**: the suite tag and manifest
   digest against the pinned git object, the SEA Forge ledger hash chain from
   raw JSONL, expectation ordering, E7 to ledger expectation and settlement
   equality, and E8 to E7/E6 correlation. RealityTrace's content binding alone
   cannot detect a self-consistent forged E6; the ledger check does (tested).

## Correlation contract (v1)

| Required element | Source record |
| --- | --- |
| Run / case / work request | ledger `subject_refs`, E6 `work_request_id` |
| Agent and authority actor | `authority_decision.actor` |
| Verified workload identity | **open**: runtime records do not yet carry the isolated workload UID |
| Frozen expectation and criteria | `security_frozen_expectation`, E7 `proof_contract` |
| Authorization request, policy hash | `authority_decision` (`action_hash`, `policy_hash`) |
| Decision and exact grant | `authority_decision.decision_id`, invocation `authority_decision_id` |
| Execution attempt | `security_authorized_invocation`, `security_workload_report` (claim) |
| Trusted resource observation | `security_resource_observation`, E6 `observed_effects[].resource_observation` |
| Claimed vs observed outcome | `security_submitted_observation` vs witnesses; E8 `expected_outcome`/`observed_outcome` |
| Structured discrepancy | E8 `difference` (digest-level) |
| Settlement reference | `security_returned_settlement`, E7 `operational_settlement_ref` |
| Digests, observer, time, provenance | ledger `payload_hash`/`entry_hash`/`writer_identity_ref`/`committed_at`/`authority_refs`; E7/E8 `provenance.chain` |

Raw credentials and sensitive plaintext never enter these records; MCP
arguments are already hashed (ADR-011 decision 7).

## Open integration obligations

1. RealityTrace's v1 gate admits E7 only from the `swe-seed` producer label.
   The slice uses that label and records the true producer in
   `proof_contract.producer = sea-forge-security-acceptance`. Minimal
   RealityTrace addition: admit a distinct frozen-acceptance producer.
2. `EvidenceRecorded` has no row in RealityTrace's `record_kinds.csv`, so E8
   is returned by the library but not persisted in an `sxr-ledger`.
   Minimal addition: register `evidence.recorded`.
3. `difference` is digest-level (`equivalent`/`divergent`); auditors receive
   both canonical values but no field-level discrepancy structure.
4. Ledger observer attribution is `writer_identity_ref`, not a signature.
5. Extend to identity, MCP, PostgreSQL, credential, exposure, delegation,
   injection evidence as those enforcement paths are qualified.
