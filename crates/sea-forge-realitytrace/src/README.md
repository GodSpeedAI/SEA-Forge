# sea-forge-realitytrace/src

## Overview
Source for the `sea-forge-realitytrace` crate: ledger-derived security evidence for RealityTrace and independent auditors (`sea-forge.security-trace/v1`). Per ADR-012, SEA Forge owns authority, enforcement, trusted observation and settlement recorded in its append-only ledger; RealityTrace owns the structured comparison of a prior expectation with observed consequences, and the auditor owns the verdict. This crate is the bridge: it admits the frozen security-acceptance proof (E7) against the operational settlement (E6) through `sxr_core::proof_ingest` and emits the `EvidenceRecorded` (E8) difference via `sxr_core::evidence_emit`. No daemon, no second authority store.

## Interface
- `TRACE_SCHEMA = "sea-forge.security-trace/v1"` — the trace contract identifier [lib.rs:TRACE_SCHEMA]
- `EXPECTATION_KIND = "security_frozen_expectation"` — ledger record kind committed before the attempt, binding the expectation [lib.rs:EXPECTATION_KIND]
- `PROOF_PRODUCER = "sea-forge-security-acceptance"` — the only producer RealityTrace accepts for E7 proofs [lib.rs:PROOF_PRODUCER]
- `FrozenSuite` — immutable acceptance reference (tag, 40-hex commit, 64-hex manifest digest) an expectation is frozen against [lib.rs:FrozenSuite]
- `TraceError` — `Provenance`, `Correlation`, `RealityTrace` refusals [lib.rs:TraceError]
- `expectation_record(suite, contract_id, case_id, work_request_id, expected_outcome) -> Result<Value, TraceError>` — payload of the `security_frozen_expectation` record, including `expected_outcome_sha256` [lib.rs:expectation_record]
- `project(entries: &[LedgerEntry], work_request_id: &str) -> Value` — project one action's committed ledger records into the correlation contract; claims and observations stay separate, missing records are listed as `unresolved` [lib.rs:project]
- `ProofContext`, `proof_completed(...)`, `record_evidence(proof, settlement, state, ctx, evidence_event_id, affordance_id)` — admit E7 against E6 through RealityTrace and emit its E8 comparison [lib.rs:ProofContext], [lib.rs:proof_completed], [lib.rs:record_evidence]
- `record_category(kind) -> Option<&'static str>` — evidence category of a SEA Forge security record [lib.rs:record_category]
- `NativeObservation` and `project_attempts(entries, native) -> Value` — project every brokered attempt (successful and failed) with its causal chain and native observation bytes for the auditor [lib.rs:NativeObservation], [lib.rs:project_attempts]

## Invariants
- Nothing here decides, settles, or fills in missing evidence: absent records are reported as `unresolved`, never inferred [lib.rs:module doc]
- The E7 proof producer is the frozen security-acceptance suite, never SEA Forge itself; `sxr_core` binds each cycle to `ProofProducer::SecurityAcceptance` and refuses any other stamp, including `sea-forge` (the system under test) [lib.rs:PROOF_PRODUCER]
- Provenance is fail-closed: the frozen suite commit must be 40 lowercase hex digits, the manifest digest 64 lowercase hex digits, and identity fields non-empty, or `TraceError::Provenance` is returned [lib.rs:expectation_record], [lib.rs:hex40], [lib.rs:hex64]
- `expected_outcome` is stated in the observed form the trusted observer produces, so a consistent run compares `equivalent` and any divergence is visible [lib.rs:expectation_record]
- Evidence categories never merge: a workload's request is a claim, the authority verdict a decision, what the broker did an enforcement act, what a trusted observer saw an observation, and the broker's closing record a settlement [lib.rs:record_category]
- Duplicate proof delivery is refused (`Admission::DuplicateDelivery` -> `TraceError::RealityTrace`), as is any `sxr_core` refusal [lib.rs:record_evidence]
- The crate is an evidence consumer only: no execution crate depends on it (Cargo.toml comment); each attempt with no settlement is reported `unresolved` [Cargo.toml], [lib.rs:project_attempts]

## Side Effects
- None. The bridge is pure projection over committed `LedgerEntry` values and in-memory `sxr_core` ingest/emit calls; no daemon, no second authority store [lib.rs:module doc]

## Verification
- Run: `cargo test -p sea-forge-realitytrace` (repo-wide: `just test`)
