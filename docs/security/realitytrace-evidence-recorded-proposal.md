# `evidence.recorded` v1 record kind

Status: **ACCEPTED and REGISTERED** (2026-10-18). The owner accepted this
proposal and decided its CEP semantics; RealityTrace `main` 42879a8
registers the kind, and SEA-Forge pins `sxr-core` there. Addresses ADR-012
obligation 2.

Owner decisions: the Question is the frozen expectation; `direction`
follows the verdict mechanically (`equivalent` → `supports`, `divergent` →
`contradicts`); compared values are cited by digest only, as
content-unavailable representations; built on the producer-bound,
field-level E8 shape.

## Problem

`sxr_core::evidence_emit::emit_evidence_recorded` returns the E8
`EvidenceRecorded` envelope: RealityTrace's expected-versus-observed
comparison of a frozen `ProofCompleted` expectation with an
`OperationalSettlement`'s observed effects. RealityTrace's record-kind
registry (`sxr-core/src/record_kinds.csv`, spec §5.3b) has no row for it, so
the comparison is returned by the library but never persisted in an
`sxr-ledger`. Auditors therefore depend on SEA Forge bundling it.

## Proposed contract

| Artifact | Path |
| --- | --- |
| Envelope schema (JSON Schema 2020-12) | `crates/sea-forge-realitytrace/contracts/evidence-recorded.v1.schema.json` |
| Registry row, in the pinned 9-column format | `crates/sea-forge-realitytrace/contracts/evidence-recorded.record-kind.psv` |
| Conformance tests | `crates/sea-forge-realitytrace/tests/evidence_recorded_contract.rs`, live check in `tests/sec12_trace.rs` |

The schema transcribes what the pinned emitter produces; it adds no field. `difference.fields`/`fields_truncated` and a `proof_producer:` provenance line are optional, so the schema covers both the earlier and the current emitter.

- **Envelope:** `schema_version` `v1`, `source_agent` and
  `provenance.origin` `realitytrace` (the exclusive producer), `event_type`
  `EvidenceRecorded`, UUID or ULID `event_id`, content-derived 64-hex
  `idempotency_key`.
- **Payload:** model anchor (`domain_model_hash`, `domain_model_ref`),
  `world_ref`, `work_request_id`, `affordance_id`, both outcomes (non-null),
  `difference` (`equivalent` or `divergent` plus both canonical digests),
  and `sha256:` references to the proof and the settlement.
- **Closed at every level** (`additionalProperties: false`): any new field,
  including plaintext or credential material, is a contract change rather
  than silent drift. The payload carries outcome descriptors and digests;
  SEA Forge never puts raw credentials or sensitive plaintext into the
  records E8 compares (ADR-011 decision 7).
- **Invariants beyond the schema** (checked by `invariants()` in the test):
  the model anchor agrees in both places; the verdict is `equivalent`
  exactly when the digests match; each digest is the SHA-256 of the
  canonical JSON of its outcome; the idempotency key is
  `sha256("|EvidenceRecorded|" + canonical_json(payload))`; the provenance
  chain opens with the model anchor, then the settlement and proof
  `caused_by` parents.

### Registry row

- `envelope_kind`: `evidence_packet`, an existing CEP-0008 §9 value (the
  vocabulary is CEP's; this proposal does not extend it). It shares the kind
  with `evidence.created` but is distinct: `evidence.created` is question-bound
  CEP Evidence, `evidence.recorded` is a derived comparison.
- `settlement_dependency`: consumes a settlement, never settles.
- `authority_class`: derived comparison, not a verdict and not authority. The
  auditor still owns the verdict (ADR-012).
- `allowed_predecessors` names a `proof.completed` kind, which is not
  registered either; it depends on ADR-012 obligation 1 (a distinct
  frozen-acceptance producer label) and is to be settled together with it.

## Compatibility

- **Additive:** a new registry row and no change to existing kinds,
  envelopes or emitter output. Existing consumers are unaffected.
- **Versioning:** `schema_version` stays `v1` for this shape; adding,
  removing or retyping any field requires `v2` and a new schema file.
- **Reconciliation:** `registered_row_matches_the_accepted_contract`
  requires RealityTrace's registered row to equal
  `contracts/evidence-recorded.record-kind.psv` field for field.

## Conformance evidence

- The committed SEC-12 bundle's E8 and the live divergent and equivalent
  emissions validate against the schema.
- Rejected by the schema: an added plaintext field, an added envelope field,
  an unknown verdict, a missing `difference`, a null observation, a wrong
  producer, a different schema version, a bare settlement reference, a
  shortened provenance chain.
- Rejected only by the invariants: a flipped verdict, an edited observation,
  model-anchor drift.
- Mutation control: opening the payload to extra fields fails the suite
  (`.agents/evidence/security-v2/round3-20261017/mutation-evidence-recorded-schema.txt`).

## RealityTrace side (42879a8)

- `record_kinds.csv` row; `sxr-core::evidence_record::evidence_recorded_parts`
  maps an emitted E8 to its CEP envelope and refuses malformed or
  non-RealityTrace E8; registry gate and dedicated tests cover it.
- `build_envelope` now marks declared known omissions
  (`known_omissions_recorded`) instead of always writing
  `no_known_omissions`.

## Still open

Field-level differences are bounded (256 entries, then `fields_truncated`);
ledger observer attribution is not a signature (ADR-012 obligation 4).
