# M8 Artifact-to-IP — spec/plan reconciliation report

Date: 2026-07-15 · Branch: `full-spec` · Scope: documentation-only architecture patch (no production code)

## 1. Substrate inspected

- `crates/sea-forge-core/src/types.rs` — `ArtifactDescriptor` (`artifact_id`, `artifact_type`, `stage: Option<ArtifactStage>`, producer, owner, license, review_status, source_refs, content_sha256, `pre_mint_identity`); `SettlementCriteriaRecord`/OriginRefs, `AuthorizedAction`, `DomainModelRef`, `EnvironmentSpec` owners confirmed across `sea-forge-core`, `sea-forge-authority`, `sea-forge-sandbox`, `sea-forge-domainforge`, `sea-forge-planner/src/criteria.rs`.
- `crates/sea-forge-evidence/src/lib.rs` — evidence capture and work-product vs stdout/stderr classification (`pre_mint` references present).
- Ledger identity/canonical encoding: M0 ledger (jcs-nfc-v1, sha256-v1, entry ULIDs, hash chain) per spec §8.1a and `sea-forge-ledger`.
- Conformance test layout: `conformance_m0*…conformance_m7.rs` per owning crate; **no `conformance_m8.rs` exists anywhere** (verified by `find crates -name "conformance_m8*"` → empty).
- `.agents/CURRENT_STATUS.md`: Tasks 1–15 implemented (M7 committed at `a09211b`); Task 16 not started; Task 17 pending. Plan checklist confirms Task 16/17 boxes unchecked.
- Minimum spec §7.3.8/G7: `pre_mint_identity` hashes `{artifact_type, stage, owner, license, review_status, content_sha256, source_refs}` — i.e., mixes mutable governance fields; demo `model.sea` descriptor declares `stage: intellectual`.
- Full spec M8 sections: §2.3 E10 (l.106), §3.1 outputs, §5 claim table, §6.2 crate map, §7.9, §8.2 rules, §10.8, §12 M8 scenario, §17.1 row, §18, Appendix A.

## 2. Reused unchanged

Existing `ArtifactDescriptor.artifact_id` (identity of one immutable artifact version), ledger canonical encoding + record identity, authority mediator/`AuthorizedAction` + the already-declared `transition_artifact_stage`/`attest_artifact_identity` operation kinds, `SettlementCriteriaRecord` + OriginRefs (one criteria truth), approval records + SoD checks, `SettlementDeclaration` strength/reliability, M7 `EnvironmentSpec`/`Evaluator`, `DomainModelRef`/concept refs, projection rebuild conventions, CasePlan/PlanItem + CLI registration, typed errors and hashing utilities. Minimum-spec history and P1–P4b untouched.

## 3. Additive extensions

- §7.8a (new section, before §7.9): `lineage_id`, `content_identity` (hash of `{artifact_type, content_sha256}` only), `descriptor_hash`, `identity_scheme`, `legacy_pre_mint_identity` (v0.1 value preserved verbatim, disqualified as stable identity because it mixes mutable governance fields), `declared_stage` vs reconstructed `recognized_stage`.
- §8.2: `transition_artifact_stage` rules gain `transition_kind`, `mode`, `gate_profile_ref`, settlement-strength and value-evidence-kind constraints; `attest_artifact_identity` gains `degraded_mode` and an explicit "identity status only" clause.
- §10.8: legacy recognition overlay (recognized stage starts `cognitive`; no fabricated historical tokens; demo `model.sea` keeps `declared_stage: intellectual`).

## 4. Genuine new M8 structures

`ArtifactRegistrationRecord`, corrected `TransitionToken` (kind + derive/promote mode, multi-source, parent-token DAG, gate-profile hash, value-evidence refs, degraded controls, identity-status before/after), `ArtifactGateProfile` (versioned, hash-pinned; fitness delegated to M7 evaluators), recognized-stage reconstruction, lineage DAG validation, artifact-state + capital projection rebuild. All confined to the planned `sea-forge-artifact-ip` crate.

## 5. Duplicates deliberately rejected

Second authority system, second approval record, second criteria store, second evaluator engine, second settlement protocol, second semantic identity system, second ledger, mutable artifact database, any migration/re-keying of v0.1 records, and any extra artifact ID for naming symmetry (ledgered records already carry ledger record identity). Mutable authoritative `current_stage` and manual `reuse_count` removed from the catalog contract — both are now derived projections.

## 6. M8 conformance test

None existed (Task 16 unimplemented), so nothing was patched or created. Per the project's convention (conformance tests ship with their milestone crate), no ignored skeleton was added; §12 M8, §17.1 M8, and the replaced Task 16 "Focused tests" section are the executable test contract. M8 is NOT green.

## 7. Specification sections changed (`.agents/specs/spec-full.md`)

| Section | Change |
|---|---|
| §2.3 E10 | Independent dimensions; `synthesize/productize/capitalize`; derive vs promote; attestation ≠ maturity; capitalization = content-preserving + SoD approval + out-of-case value evidence; quality insufficient |
| §3.1 outputs | catalog.jsonl and capital JSON declared rebuildable projections, fail-closed |
| §5 claim table | M8 gate row updated to corrected mechanics |
| §6.2 crate map | `sea-forge-artifact-ip` scope = missing M8 logic only, reuses substrate |
| §7.8a (new) | v0.1→v0.2 identity compatibility (additive) |
| §7.9 (rewritten) | Four dimensions, normative stage definitions, ArtifactRegistrationRecord, derivation vs promotion, full TransitionToken contract, 9 testable invariants |
| §8.2 | transition/attestation policy rules strengthened (see §3 above) |
| §10.8 (rewritten) | Registration + legacy recognition, three commands (no `refine`), gate profiles, generic stage gates, fail-closed capital projection |
| §12 M8 | Full adversarial proof scenario (registration/compat, promotion, synthesis, productization, capitalization, no-teleportation/lineage, identity/attestation) |
| §17.1 M8 row | Matches new scenario, incl. derive/promote, quality-only/approval-only rejection, materialized-view tampering |
| Appendix A | M8 wording aligned |

§2.2, §2.4, §4, §9, §11, §13, §14, §15 were inspected; no M8-contradicting content found, left unedited. §18 M8 entry ("gate green before any artifact is reported as reusable capital") remains valid unchanged.

Plan: Task 16 replaced in place in `.agents/plans/2026-07-11-spec-full-implementation.md` (appears exactly once; Tasks 1–15 status and Task 17 untouched).

## 8. Why the new Task 16 is implementable by a smaller model

Ontology, identity formulas, legal edges, gate contents, token fields, invariants, and the complete adversarial test list are all stated normatively in the spec sections Task 16 cites; the task orders a substrate-inspection report before coding, names every structure to reuse and every duplicate to reject, fixes the command verbs and modes, and pins done-when + six teeth checks. The agent decides file layout and Rust idioms only — no semantic invention remains.

## 9. Verification commands run

- `find crates -name "conformance_m8*"` → empty (no test existed).
- `grep` sweeps for stale wording: no remaining `synthesize|refine|capitalize` transition set, authoritative `current_stage`/`reuse_count`, single-artifact token assumptions, quality-creates-capital, attestation-changes-maturity, or content-changing capitalization claims in spec-full.md (residual `refine`/`synthesize` hits belong to E5 generator-pipeline stages, unrelated).
- `git diff --stat` → only `.agents/specs/spec-full.md` (176 lines changed) and `.agents/plans/2026-07-11-spec-full-implementation.md` (41 lines changed), plus this new report.
- `grep -c "^## Task 16"` on the plan → 1 (replaced in place); Task 17 heading present and unchanged; all Task 1–15 checklist boxes untouched.
- Regression evidence (docs-only change, code untouched): `cargo test -p sea-forge-cell --locked -q` → `11 passed; 0 failed` (M6 conformance suite green).

## 10. Unresolved issues

None blocking. One note for the implementer: the minimum spec's v0.1 `pre_mint_identity` formula (mutable-field mix) is preserved as-is by design; do not "fix" it — v0.2 identity lives in `content_identity`.
