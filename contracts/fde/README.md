# FDE contracts (WP-11)

Reviewed-before-code contracts for the Field Case path. JSON Schema draft 2020-12, every object closed (`additionalProperties: false`). These files are hand-authored; unlike `contracts/schema/`, no generator writes them. A change to a schema file needs a new line in `DIGESTS` in the same commit (`scripts/check-fde-contract-digests.sh --update`); consumers vendor by digest.

| Contract | File | Fixtures (valid / invalid, schema + semantic) | Test |
|---|---|---|---|
| Frozen reference | `frozen-reference.v1.json` | 3 / 16 + 1 | `crates/sea-forge-realitytrace/tests/fde_contracts.rs` |
| Anchor | `anchor.v1.json` | 3 / 8 + 2 | same |
| `case.record.append` | `case-record-append.v1.json` | 12 / 18 + 1, 11 scenarios | same |
| Field Case document | `field-case.v1.json` | 3 / 20 + 0 | same |
| Record-kind registry | `case-record-kinds.v1.json` | 1 / 16 + 2 | same |
| `case.record.append` v2 | `case-record-append.v2.json` | 42 / 33 + 9, 54 scenarios | same |
| Execution `episode_ref` | `execution-episode-ref.v1.json` | 26 / 25 + 8, 82 scenarios, 12 frames | same |
| Delegation link | `delegation-link.v1.json` | 9 / 21 + 22, 7 scenarios | same |
| Continuation and read verbs | `continuation.v1.json` | 12 / 31 + 11, 4 frames | same |
| Reserved operation | `reserved-operation.v1.json` | 23 / 57 + 17, 14 scenarios | same |
| Assignment | `assignment.v1.json` | 7 / 26 + 5, 10 scenarios | same |

Test canonicalisation: the SEA-Forge test hashes fixtures with a small ASCII-only helper that refuses (errors on) non-ASCII text and non-integer numbers instead of hashing them wrongly. Full RFC 8785 conformance is tested in the CEP repository only (CEP-0008 section 16.5 vectors).

Run: `cargo test --locked -p sea-forge-realitytrace --test fde_contracts` and `scripts/check-fde-contract-digests.sh`.

Source sections are in the GodSpeed FDE integration architecture (E), `.agents/reports/fde-convergence-2026-11-12/E-integration-architecture.md`, and its review record (H).

Hash rule: every digest is SHA-256 over the RFC 8785 canonical JSON of the stated value, written as 64 lowercase hex characters (CEP-0008 section 16.5, CEP repository). The `evaluate` facts and output schemas belong to WP-12 (`domainforge/schemas/evaluate-facts.v1.json`, `evaluate-output.v1.json`) and are only named here; no schema is provided.

## 1. Frozen reference (`frozen-reference.v1.json`)

Document `{episode_ref, body}`. The digest input is `body` alone; `episode_ref` is not in it (RD-23). Anything that can change the verdict is in `body`, so nothing outside the digest can be edited after the freeze. A result, observation or outcome field in `body` is refused by the closed schema (NT-17).

`episode_ref` = `episode:<case_id>:<plan_item_id>:<n>@sha256:<digest>`, where `<digest>` = SHA-256(RFC 8785(`body`)), and `case_id`, `plan_item_id`, `n` equal `body`'s.

| Field | Type | Req | Meaning | E |
|---|---|---|---|---|
| `episode_ref` | string, pattern above | yes | identity of the episode; computed, not part of the digest | §2 |
| `body.case_id`, `body.plan_item_id` | string, no `:` `@` whitespace | yes | episode key part | §2, §3 |
| `body.n` | integer >= 1 | yes | episode counter, allocated by the case keeper | §2, I-3 |
| `body.world_ref` | `world:<name>@sha256:<64hex>` | yes | pinned model | §2 |
| `body.kind` | `evaluated` \| `exploratory` | yes | an evaluated episode is a test; exploratory never confirms | §3, §14 |
| `body.executor_provenance` | `program` \| `live` \| `injected_fault` | yes | provenance of the execution | §3, §14 |
| `body.criteria.intention` | string | yes | what the episode is for | §13 |
| `body.criteria.expected_change` | string | yes | expected change | §13 |
| `body.criteria.invariants` | string[] | yes (may be empty) | what must not change | §13 |
| `body.criteria.predicted_irrelevancies` | string[] | yes (may be empty) | what is predicted not to matter | §13 |
| `body.criteria.success_region`, `failure_region` | string | yes | regions | §13 |
| `body.criteria.falsifiers` | string[] | yes; >= 1 item when `kind` is `evaluated`, may be empty when `exploratory` | what would show the expectation wrong | §13, review 2026-11-14 |
| `body.comparison_profile` | `{id: profile:<token>, sha256: 64 hex}` | yes | the comparison profile frozen with the episode; `sha256` is of the profile document (CEP-0008 §16.5) | RD-08, review 2026-11-14 |
| `body.decision_maker` | `{principal, role}` | yes | who settles (I-2 reads this and the delegation chain) | §13 |
| `body.evidence_sources[]` | `{source_id, system, purpose}` | yes, >= 1 | evidence source and what it is for (RD-03) | §13 |
| `body.coverage_manifest[]` | `{emitter, event_kinds[] >= 1}` | yes, >= 1 | which emitters must report which event kinds; frozen, never self-reported (RD-12) | §5, §13 |

### Errors (semantic, beyond the schema)

| Code | Rule |
|---|---|
| `EPISODE_REF_DIGEST_MISMATCH` | `episode_ref`'s digest is not SHA-256(RFC 8785(`body`)), or its `case_id`/`plan_item_id`/`n` differ from `body`'s |

## 2. Anchor (`anchor.v1.json`)

A signed annotated tag `fde-anchor/<kind>/<id>` pushed to the GitHub remote by the case-keeper service identity, never an executor credential, plus the verifier's receipt (RD-04). Anchored: model registration (`model`), each freeze (`freeze`), pre-registration (`prereg`).

| Field | Type | Req | Meaning | E |
|---|---|---|---|---|
| `tag` | `fde-anchor/<kind>/<id>` | yes | the tag name | §5 |
| `kind` | `model` \| `freeze` \| `prereg` | yes | what is anchored | §5 |
| `id` | `[A-Za-z0-9][A-Za-z0-9._-]{0,127}` | yes | git-ref-safe token (an `episode_ref` contains `:` and cannot be a tag name) | §5 |
| `subject_ref` | string | yes | `world_ref` (model), `episode_ref` (freeze), pre-registration id (prereg) | §5 |
| `digest` | 64 lowercase hex | yes | anchored digest | §5 |
| `pusher.identity` | string | yes | case-keeper service identity | §5, RD-04 |
| `pusher.credential_scope` | `shared_os_user` \| `dedicated_anchor_credential` | yes | `shared_os_user` makes anchoring rows PARTIAL (RD-26, OD-6) | §5 |
| `verifier_receipt.receipt_id`, `.verifier` | string | yes | the verifier's own receipt, issued at freeze | §5 |
| `verifier_receipt.issued_at`, `.server_push_time` | RFC 3339 timestamp | yes | `server_push_time` is the remote's, not the pusher's | §5 |
| `verifier_receipt.tag_object_id` | 40 or 64 hex | yes | annotated tag object | §5 |
| `verifier_receipt.digest` | 64 lowercase hex | yes | digest the verifier read from the tag | §5 |

### Errors

| Code | Rule | Source |
|---|---|---|
| `ANCHOR_TAG_MISMATCH` | `tag` is not `fde-anchor/<kind>/<id>` | schema cannot compare fields |
| `ANCHOR_RECEIPT_MISMATCH` | `verifier_receipt.digest` differs from `digest` | same |
| `ANCHOR_PREDATES_FAILED` | tag absent, or its server push time is not before the first run record (NT-16) | verification, WP-14 |
| `MODEL_ANCHOR_LATE` | model anchored after the first run event (NT-18) | verification, WP-14 |

Only the first two are exercised by fixtures here; the last two need a ledger and a remote and belong to WP-14.

## 3. `case.record.append` (`case-record-append.v1.json`)

One write verb. `$defs`: `Request`, `Response`, `Error`, `EpisodeFrozenPayload`, `EpisodeSettledPayload`. Registry at this stage: `episode.frozen`, `episode.settled`. WP-20 adds the other four kinds and the read verbs. The case is addressed by the SFWP route; a payload whose `episode_ref` names another case is refused as `SCHEMA_INVALID`.

Episode key: `(case_id, plan_item_id, n)`.

### Request

| Field | Type | Req | Meaning |
|---|---|---|---|
| `kind` | `episode.frozen` \| `episode.settled` | yes | record kind; any other value is `KIND_UNKNOWN` |
| `payload` | object, per kind | yes | below |
| `idempotency_key` | string, no whitespace, <= 256 | yes | cross-system record key `(system, record_id)` written as one token (RD-12) |

`episode.frozen` payload (freeze order step 2, after the RealityTrace plan, question and threshold exist):

| Field | Type | Req | Meaning |
|---|---|---|---|
| `episode_ref` | string | yes | equals `reference.episode_ref` |
| `reference` | frozen reference (section 1) | yes | the canonical reference |
| `digest` | 64 hex | yes | equals the digest inside `episode_ref` |
| `realitytrace_refs` | `{plan_id, question_ids[] >= 1, threshold_ids[] >= 1}` | yes | records written first |

`kind`, `executor_provenance` and the coverage manifest of E §3 travel inside `reference.body`, not beside it, so they cannot differ from the frozen copy.

`episode.settled` payload:

| Field | Type | Req | Meaning |
|---|---|---|---|
| `episode_ref` | string | yes | the frozen episode |
| `result` | `S` \| `F` \| `U` \| `C` | yes | epistemic result |
| `reliability` | object over `T A I K D X N`, string values | yes (may be `{}`) | settlement reliability, apart from `result` |
| `independence` | `{independent_source_count >= 0, verifier_context: fresh\|shared\|unknown}` | yes | source independence, apart from `result` |
| `settlement_ref` | string | yes | RealityTrace settlement id |

The settling principal is the authenticated principal of the request, not a payload field (I-2 is checked on the execution identity, not a caller tag).

### Response

`{outcome: "recorded", kind, record_id, episode_key: {case_id, plan_item_id, n}, replayed}`. `replayed` is true when the same `idempotency_key` and body were recorded before; the original response is returned and nothing is written.

### Error

`{outcome: "refused", code, message, invariant?}`. `invariant` is required for the three invariant codes and forbidden for the others.

| Code | Invariant | Raised when | NT |
|---|---|---|---|
| `SCHEMA_INVALID` | | request or payload fails this schema (other than `kind`, `result`) | |
| `KIND_UNKNOWN` | | `kind` is not a registered kind | |
| `IDEMPOTENCY_CONFLICT` | | `idempotency_key` seen with a different body | |
| `EPISODE_REF_DIGEST_MISMATCH` | | `payload.digest`, `episode_ref` and the recomputed body digest disagree | NT-17 |
| `RESULT_INVALID` | | `result` outside S/F/U/C, or `reliability` merged into `result` | NT-42 |
| `EXPLORATORY_NOT_SETTLEABLE` | | `episode.settled` with `result` `S` or `F` (the confirmed / contradicted classes) for an episode whose frozen `kind` is `exploratory`; only `U` and `C` are accepted. Needs no payload change: the kind is read from the freeze record | NT-13, RD-12 |
| `FREEZE_KEY_EXISTS` | I-3 | a freeze already exists for the episode key, whatever its digest, under a different `idempotency_key`; changed criteria allocate `n+1` | NT-01 |
| `FREEZE_MISSING` | I-1 | `episode.settled` for an episode with no freeze record (also the delegation refusal of WP-20) | NT-32 |
| `FREEZE_INCOMPLETE` | I-1 | `episode.frozen` references a RealityTrace plan, question or threshold that does not exist (the reverse half, and the missing anchor, are checked at delegation) | NT-33 |
| `SELF_SETTLEMENT_REFUSED` | I-2 | the settling principal's delegation chain contains the executor's execution or agent identity | NT-10 |

`SCHEMA_INVALID` vs `RESULT_INVALID`/`KIND_UNKNOWN`: an implementation reports the specific code when the failing pointer is `/kind` or `/payload/result` and `/payload/reliability`, otherwise `SCHEMA_INVALID`.

### Scenarios

`fixtures/case-record-append/scenarios/*.json` give ledger state, request and expected response for I-1, I-2, I-3 and idempotency. The test validates the request and the expected response against this schema and checks that each scenario's invariant matches its response; WP-14 and WP-21 make the case keeper produce them.

# FDE contracts, S2 package (WP-20)

Contracts only: schemas, request/response/error tables, fixtures, digest pins and checks. No server behaviour changes; WP-21 (records, gate, I-2 chain), WP-23 (assignment binding) and WP-31 (A3 check) implement them. The v1 files above are closed and digest-pinned and are unchanged; every S2 contract is a new file. `DIGESTS` pins all of them and `oracle-codes.json`; the CHANGELOG (`CHANGELOG.md`) lists what each adds, which S1 limit it closes, and which WP and frozen tests it serves.

Further checks: `scripts/check-fde-field-case-schema.sh` (the schemas cannot drift from SOP 4.2/10.1 or the model enums), `scripts/check-fde-oracle-codes.sh` (every G oracle code is defined in exactly one table; mapping in `oracle-codes.json`), `scripts/check-fde-contract-digests.sh`. Run: `cargo test --locked -p sea-forge-realitytrace --test fde_contracts`.

Fixture layout per contract: `fixtures/<contract>/{valid,invalid}/*.json` with `manifest.json` (as in S1: `invalid` entries name the pointer and the validator message; `semantic_invalid` entries pass the schema and fail the stated rule), plus `scenarios/` where a sequence or ledger state matters (ledger-state refusals the schema cannot state) and `frames/` for SFWP frames. **Frames are expected frames, not goldens**: the golden capture from a live server into `apps/godspeed-casework-go/internal/adapters/sfwp/testdata/frames.json` is left to WP-21, which owns the server behaviour. Fixtures named `x-decision` in the lists below mark every place where E, G and the SOP did not determine a field or rule; section 13 lists them all.

## 4. Field Case document (`field-case.v1.json`)

The case-scoped document `cases/<case_id>/field_case.json` of a case whose template is `fde.field_case@1` (E section 3). Root `{case_id, template_ref, revision, sop {document, version, sha256}, fields}`; `fields` holds the six SOP 4.2 groups (`purpose_and_outcome`, `framing`, `world_and_boundary`, `observer_and_authority`, `current_horizon`, `continuity`), each with one `FieldValue` per SOP field (45 fields). The group and field keys are the SOP labels in snake case; `x-sop-label` and `x-sop-dagger` carry the exact SOP text and the narrative-SOP mark, and the check script compares them with the SOP text (sha256 pinned in `docs/sop/README.md`).

`FieldValue` = `{status, source_ref, value?}`:

| Field | Type | Req | Rule |
|---|---|---|---|
| `status` | `declared` \| `observed` \| `inferred` \| `proposed` \| `unknown` | yes | E section 3 |
| `source_ref` | `{kind: record \| realitytrace \| cognate \| horizon \| operator \| sop \| none, ref}` | yes | `kind` is `none` (and `ref` is `none`) exactly when `status` is `unknown`; `observed` takes only `record`, `realitytrace`, `cognate` or `horizon` (never the operator's say-so or the SOP text) |
| `value` | any non-null JSON | when `status` is not `unknown` | absent when `status` is `unknown` |

### Check against the model

The declared model does not declare the 4.2 fields: translation-map row `4.2` is `unknown`, limitation S7 (scope), L2 (lists). The schema is therefore generated from the SOP 4.2 table, not from model elements; `scripts/check-fde-field-case-schema.sh` binds it to the SOP text and fails if the map row ever stops being `unknown` (then the schema must be regenerated from the model). The same script checks the enums the S2 contracts reuse against the model: gate names and states, disposition kinds and levels, case lifecycle, episode kinds, epistemic results, payment currencies, reliability and aim dimensions, and the SOP 10.1 items of `continuation.v1`.

### Errors

None of its own: a document that fails the schema read from the store is `STORE_INTEGRITY` (section 6); a request that revises it wrongly is refused by `case.record.append` (`SCHEMA_INVALID`, `REVISION_CONFLICT`).

## 5. Record-kind registry (`case-record-kinds.v1.json`)

The registry instance is `fixtures/case-record-kinds/valid/registry.json`. Per kind: payload schema (`schema` + `def`), idempotency-key rule, natural key, refusal codes, immutability rule, and what a suspended and a closed case do with it. At registry level it also lists the Field Case paths that are read-only after the first freeze.

| Kind | Since | Scope | Payload schema | Natural key | Immutability | Suspended | Closed |
|---|---|---|---|---|---|---|---|
| `episode.frozen` | v1 | episode | `case-record-append.v1` `EpisodeFrozenPayload` (unchanged) | `(case_id, plan_item_id, n)` | `frozen_forever` (I-3) | refuse | refuse |
| `episode.settled` | v1 | episode | `case-record-append.v1` `EpisodeSettledPayload` (unchanged) | `(episode_ref)` | `single_settlement` | refuse | accept (late evidence) |
| `field_case.revised` | v2 | case | `case-record-append.v2` `FieldCaseRevisedPayload` | `(case_id, base_revision)` | `revision_chain` | accept | refuse |
| `strategy.recorded` | v2 | case | `StrategyRecordedPayload` | `(case_id, revision)` | `revision_chain` | accept | refuse |
| `gate.assessed` | v2 | case | `GateAssessedPayload` | none | `append_only_history` | accept | refuse |
| `disposition.recorded` | v2 | case | `DispositionRecordedPayload` | none | `append_only_history` | only the case-level `continue` (lifts) or `close-*` (closes) | refuse |

Idempotency-key rule, all kinds: one whitespace-free token of at most 256 characters, the cross-system record key `(system, record_id)` (RD-12). Same key and same body replays the original response (`replayed: true`); same key and another body is `IDEMPOTENCY_CONFLICT`. The registry is closed to these six; a seventh is a new registry version. The refusal codes of each kind are listed in the registry and checked against the v2 Error enum by the test. `REGISTRY_INCOMPLETE` (a kind lacks a code its rule needs, including `CASE_CLOSED` or `CASE_SUSPENDED` where it refuses) and `REGISTRY_NATURAL_KEY_INVALID` (a case-scoped kind names an episode field) are test-local.

**Read-only after the first freeze** (`reserved_paths_after_freeze`, M5): `world_and_boundary.world_ref_and_version`, `observer_and_authority.independent_verifier`, `observer_and_authority.settlement_and_disposition_authority`, `observer_and_authority.delegated_actors`, `observer_and_authority.risk_and_external_effect_limits`. After the case's first `episode.frozen`, a `field_case.revised` that changes one of them is the reserved operation `revise_frozen_boundary_field` (section 10): without an `approval_ref` backed by an A3 approval it is refused at session level with `RESERVED_DECISION_REFUSED`. Before any freeze these paths revise like any other.

## 6. `case.record.append` v2 (`case-record-append.v2.json`)

Supersedes v1 **for new writers**; v1 stays valid and is unchanged. `$defs`: `Request`, `Response`, `Error`, `FieldCaseRevisedPayload`, `FieldChange`, `StrategyRecordedPayload`, `GateAssessedPayload`, `DispositionRecordedPayload`; the two v1 payloads are referenced from v1.

**Dialects (review H1).** The `contract: "fde.case-record-append@2"` marker selects the **response dialect only**. Every rule below applies to every writer. A request without the marker is a v1 writer: it may use only the two v1 kinds (`KIND_UNKNOWN` otherwise) and is answered in the v1 dialect, where each v2-only code is replaced by a **fallback that is itself a refusal** (the v2 code name prefixes the message), or is unreachable because only a v2-only kind can raise it (`HARD_GATE_NOT_WAIVABLE`, `REVISION_CONFLICT`, `RESERVED_DECISION_REFUSED`). The marker is required for the four new kinds. A v1 writer therefore never receives a code its closed v1 Error schema does not know, and never an acceptance where a v2 writer is refused: the test asserts that no scenario in the v1 dialect is accepted, and that `oracle-codes.json` gives every v2-only code a refusal fallback (`AUTHORITY_DENIED` included).

### Request

`{contract?, kind, payload, idempotency_key}`, closed. `payload` per kind:

| Kind | Payload fields |
|---|---|
| `field_case.revised` | `base_revision` (integer >= 1, the revision this was written against), `changes[]` (1 to 64, each `{path: "<group>.<field>", field: FieldValue}`), `rationale`, `approval_ref?` (required for the read-only paths after a freeze). Creates revision `base_revision + 1` |
| `strategy.recorded` | `revision` (1 for the first), `supersedes` (record_id of revision-1, null for 1), `where_to_play`, `how_to_win`, `underserved_outcome`, `tested_assumption`, `abandonment_trigger` (SOP 4.4), `revisit_reason` (null for 1, required text after) |
| `gate.assessed` | `gate` (the nine of SOP 8.1), `state` (`met`, `blocked`, `unassessed`, `not_applicable`), `evidence_refs[]` (>= 1 when `met`; no duplicates), `rationale`, `assessor` (E section 3 lists it; must equal the authenticated principal, else `SCHEMA_INVALID`), `proposer?` (required for `prove_reliability`; assessor equal to proposer on canonical actor strings is `SELF_SETTLEMENT_REFUSED`), `approval_ref?` (required for any `not_applicable`: SOP 8.1 "authorized"; absent is `HARD_GATE_NOT_WAIVABLE` for a hard gate and `AUTHORITY_DENIED` for any other) |
| `disposition.recorded` | `level` (`case`, `episode`), `disposition` (`close-satisfied`, `close-failed`, `continue`, `revise`, `escalate`, `suspend`), `episode_ref` (exactly when `level` is `episode`), `rationale`, `approval_ref` |
| `episode.frozen`, `episode.settled` | as v1, unchanged |

**Approvals (review H3).** `approval_ref` is not free text. The approval it names (`ApprovalRecord` in `reserved-operation.v1`) must carry an `operation_digest` equal to the digest of the canonical operation built from this very record (per-operation REQUIRED `params`, including `record_digest` = SHA-256(RFC 8785(payload without `approval_ref`)), `case_id`, and the disposition kind and level, the gate id, or the episode_ref as applicable), must belong to the same case, be `allow`, unexpired and **unused**; it is consumed by the record that uses it (single use). Anything else, or no approval, is `AUTHORITY_DENIED`. For a human `episode.settled` (whose closed v1 payload has no `approval_ref`) the approval is found by the digest of `settle_episode_human` over this record; none found is `AUTHORITY_DENIED`, for v1 and v2 writers alike. Scenarios: same approval for two dispositions, approval bound to another disposition, cross-case reuse, a payload changed after approval, human settlement without approval (v2 and v1 writer).

Hard gates (`approval_ref` backed by an A3 approval): `allocate_authority`, `integrate_govern` (D12). Lifecycle is the pure function of E section 3 over the ledger: `closed` when a case-level `close-*` disposition exists (final, `CASE_CLOSED` for anything after it that is refused below); `suspended` when the latest case-level disposition is `suspend`; otherwise `open`. Episode-level dispositions never change it. Case-level priority: a `continue` or `suspend` that follows a `close-*` is refused (`CASE_CLOSED`).

### Response

`{outcome: "recorded", kind, record_id, replayed, episode_key?, revision?, lifecycle?}`: `episode_key` exactly for the two episode kinds, `revision` exactly for `field_case.revised` and `strategy.recorded`, `lifecycle` exactly for `disposition.recorded` (the lifecycle after the record).

### Error

`{outcome: "refused", code, message, invariant?}`; `invariant` is required for the invariant codes and forbidden for the others. "v1 writers" says what a v1 writer receives for the same condition.

| Code | Invariant | Raised when | v1 writers |
|---|---|---|---|
| `SCHEMA_INVALID` |  | request or payload fails this schema (other than the cases below) | yes (existing) |
| `KIND_UNKNOWN` |  | `kind` is not one of the six registered kinds, or is a v2-only kind sent by a v1 writer (no `contract` marker) | yes (existing) |
| `IDEMPOTENCY_CONFLICT` |  | `idempotency_key` seen before with a different body | yes (existing) |
| `EPISODE_REF_DIGEST_MISMATCH` |  | `payload.digest`, `episode_ref` and the recomputed SHA-256 of `reference.body` disagree | yes (existing) |
| `RESULT_INVALID` |  | `result` outside S/F/U/C, or `reliability` merged into `result` | yes (existing) |
| `FREEZE_KEY_EXISTS` | I-3 | a freeze exists for `(case_id, plan_item_id, n)` under a different `idempotency_key`, same `kind`, whatever its digest | yes (existing) |
| `FREEZE_MISSING` | I-1 | `episode.settled` for an episode with none of the freeze parts; at the execution gate, a Field Case request with no `episode_ref`, or one naming an episode with none of the parts (RealityTrace records, case record, anchor when configured) (section 7) | yes (existing) |
| `FREEZE_INCOMPLETE` | I-1 | `episode.frozen` names a RealityTrace plan, question or threshold that does not exist; at the execution gate and at settlement: some but not all of RealityTrace records, case record and anchor (when configured) exist, including the anchor-only state (NT-33 a, b, c) | yes (existing) |
| `SELF_SETTLEMENT_REFUSED` | I-2 | the settling principal is in the executor set of the episode: an actor or on_behalf_of principal of any run of the episode (item_execute and case_advance included), a link delegate, or a transitive delegator of those (section 8); also a `gate.assessed` whose assessor is the proposer where SOP 8.1 requires independence (`prove_reliability`) | yes (existing) |
| `EXPLORATORY_NOT_SETTLEABLE` |  | `episode.settled` with `result` S or F for an episode whose frozen `kind` is `exploratory` | yes (existing) |
| `WORLD_NOT_PINNED` |  | `episode.frozen` whose `reference.body.world_ref` is not the world currently pinned to the case in the SEA-Forge world pin record (WP-18), not a Field Case field (S1 finding F8) | no; v1 writer gets `SCHEMA_INVALID`, message prefixed `WORLD_NOT_PINNED: ` |
| `SETTLEMENT_EXISTS` |  | `episode.settled` for an episode that already has a settlement, under a different `idempotency_key` (any result); same key and body replays, same key and other body is `IDEMPOTENCY_CONFLICT` | no; v1 writer gets `IDEMPOTENCY_CONFLICT`, message prefixed `SETTLEMENT_EXISTS: ` |
| `KIND_IMMUTABLE` | I-3 | `episode.frozen` for an existing episode key whose `kind` differs from the recorded freeze's (NT-13: relabelling an exploratory episode as evaluated); takes precedence over `FREEZE_KEY_EXISTS`. Also the execution gate: `episode_ref.kind` differs from the frozen kind | no; v1 writer gets `FREEZE_KEY_EXISTS` (v1 README: a relabel is a second freeze for the key) |
| `REFERENCE_DERIVED_FROM_RESULT` |  | (best-effort detection, section 6) `episode.frozen` whose `reference.body` carries result content: a property from the closed result-field list (`result`, `outcome`, `observation`, `settlement`, `verdict`, `residual`) or a string quoting a RealityTrace result id (`observation:`, `evidence:`, `settlement:`, `residual:` + 26-character id) (NT-17). Replaces `SCHEMA_INVALID` at those pointers | no; v1 writer gets `SCHEMA_INVALID` (closed body) |
| `MODEL_ANCHOR_LATE` |  | `episode.settled` (and `episode.frozen` when the case already holds a run event for the world) when the `model` anchor for the episode's `world_ref` is absent, or its server push time is not before the first run event of the case (NT-18) | no; v1 writer gets `FREEZE_INCOMPLETE`, message prefixed `MODEL_ANCHOR_LATE: ` |
| `AUTHORITY_DENIED` |  | a record rests on an approval or decision that is denied, expired, unused-check failed, bound to another record or case, already used, or required and absent (fail closed): `disposition.recorded`, `gate.assessed` with `not_applicable`, `field_case.revised` of a boundary path, human `episode.settled`; at the execution gate, the program-execution decision absent, denied, exhausted or not binding this execution (NT-34); a read verb for a caller entitled to know the case but without the read grant | **yes, refused**: a v1 writer gets `SCHEMA_INVALID`, message prefixed `AUTHORITY_DENIED: ` (the marker never relaxes a rule) |
| `ANCHOR_PUBLISHER_INVALID` |  | the anchor's `pusher.identity` is not the configured case-keeper service identity, i.e. an executor credential (NT-48); checked at freeze when an anchor is configured (the configured credential, before the record is written, so the key is not burned), at the execution gate and at settlement (the pushed tag) | no; v1 writer gets `FREEZE_INCOMPLETE`, message prefixed `ANCHOR_PUBLISHER_INVALID: ` |
| `STORE_INTEGRITY` |  | the case store failed its integrity check on read (broken record chain, missing `field_case.json` of a Field Case, unreadable ledger); nothing was written; retrying does not help | no; v1 writer gets the SFWP error frame, `error_class: store_integrity` |
| `PAYLOAD_TOO_LARGE` |  | the encoded request exceeds 1048576 bytes; nothing was read or written | no; v1 writer gets the SFWP error frame, `error_class: input_error` |
| `HARD_GATE_NOT_WAIVABLE` |  | `gate.assessed` with `state` `not_applicable` for a hard gate (`allocate_authority`, `integrate_govern`) without `approval_ref` (NT-11, SOP 8.1b); an approval that is not A3-backed fails as `AUTHORITY_DENIED` | unreachable: `gate.assessed` is a v2 kind |
| `CASE_SUSPENDED` |  | the latest case-level disposition is `suspend` (NT-22) and the request is `episode.frozen`, `episode.settled`, an episode-level disposition, or a case-level disposition other than `continue` (lifts) or `close-*` (closes); also execution and delegation (section 7). Owner decision D8: G NT-22, E and the SOP name only freezes and delegations, so settlement and new dispositions are refused by default; revision, strategy and gate records are still accepted | no; v1 writer gets `SCHEMA_INVALID`, message prefixed `CASE_SUSPENDED: ` |
| `REVISION_CONFLICT` |  | `field_case.revised.base_revision`, or `strategy.recorded.revision - 1`, is not the current revision (optimistic concurrency); retry after `case.field.get` | unreachable: both are v2 kinds |
| `VERIFIER_CONTEXT_MISMATCH` |  | `independence.verifier_context` differs from the context derived from run records, or a settler sharing a delegation root with an executor settles `S` or `F` without a derived `fresh` context (H2, section 6) | no; v1 writer gets `SCHEMA_INVALID`, message prefixed `VERIFIER_CONTEXT_MISMATCH: ` |
| `CASE_CLOSED` |  | a case-level `close-*` disposition exists and the request is `episode.frozen`, `gate.assessed`, `field_case.revised`, `strategy.recorded` or any disposition (including `continue` or `suspend` after a close); closed is final. `episode.settled` is accepted as late evidence (M3) | no; v1 writer gets `SCHEMA_INVALID`, message prefixed `CASE_CLOSED: ` |
| `RESERVED_DECISION_REFUSED` |  | `field_case.revised` changes a read-only boundary path after the case's first freeze without an `approval_ref` backed by A3 (section 5); defined in section 10 | unreachable: `field_case.revised` is a v2 kind |

**Precedence** (first that applies wins): `PAYLOAD_TOO_LARGE`; `KIND_UNKNOWN`; `SCHEMA_INVALID` and its specific forms (`RESULT_INVALID`, `REFERENCE_DERIVED_FROM_RESULT`); `STORE_INTEGRITY` (any read the following need fails); idempotency (`IDEMPOTENCY_CONFLICT`, replay: a replay returns the original record and is never refused by a later rule, D43); `CASE_CLOSED`; `CASE_SUSPENDED`; `RESERVED_DECISION_REFUSED`; `EPISODE_REF_DIGEST_MISMATCH` (freeze: the payload digests disagree with the recomputed body digest); `WORLD_NOT_PINNED`; `ANCHOR_PUBLISHER_INVALID` (freeze, anchor configured); `KIND_IMMUTABLE`; `FREEZE_KEY_EXISTS`; `FREEZE_MISSING`; `FREEZE_INCOMPLETE`; `EPISODE_REF_DIGEST_MISMATCH` (settlement: the cited digest is not the frozen episode's); `MODEL_ANCHOR_LATE`; `ANCHOR_PUBLISHER_INVALID` (settlement); `SETTLEMENT_EXISTS`; `EXPLORATORY_NOT_SETTLEABLE`; `SELF_SETTLEMENT_REFUSED`; `VERIFIER_CONTEXT_MISMATCH`; `HARD_GATE_NOT_WAIVABLE`; `AUTHORITY_DENIED`; `REVISION_CONFLICT`. The `SCHEMA_INVALID` rule of v1 (specific code when the failing pointer is `/kind`, `/payload/result`, `/payload/reliability`) extends to `/payload/reference/body/<result-field>` (`REFERENCE_DERIVED_FROM_RESULT`).

**Pin source (`WORLD_NOT_PINNED`, M5).** The world a freeze is compared with is the SEA-Forge **world pin record** (the `world-pin:<operation_id>` record in the CEP ledger, `crates/sea-forge-authority/src/cep/world_pin.rs` `pin_state`, WP-18; the current pin is the last `to_world_ref`, else the baseline), never the Field Case's `world_ref_and_version` field, which a session could edit.

**Freeze-time anchor check (M6).** When an anchor is configured, `episode.frozen` also checks that the configured anchor credential is the case-keeper identity; a configured executor credential is `ANCHOR_PUBLISHER_INVALID` **before** the record is written, so a bad anchor configuration does not burn the episode key. The checks at the execution gate and at settlement remain (the pushed tag is verified there).

**`REFERENCE_DERIVED_FROM_RESULT` is best-effort (M4).** It detects a result-field property in `reference.body` and a string quoting a RealityTrace result id. It does not and cannot prove a reference was not reconstructed from a result (a paraphrase passes). The real protection is ordering: the freeze record and its anchor exist before the first run event (NT-16, `ANCHOR_PREDATES_FAILED`, `MODEL_ANCHOR_LATE`), so a reference cannot depend on a result that did not yet exist.

**Verifier context is derived, not asserted (H2).** `independence.verifier_context` in the closed v1 `episode.settled` payload is informational. The server derives the context from run records (section 8): `fresh` when the settler has a `verifier` run whose `workspace_ref` is not any executor run's, `shared` when it has one that is, `unknown` when it has none. A declared value that differs from the derived value is `VERIFIER_CONTEXT_MISMATCH`. A settler in the same delegation root as an executor (a sibling) cannot settle `S` or `F` unless the derived context is `fresh`; `U` and `C` are allowed.

Scenarios (`fixtures/case-record-append-v2/scenarios/*.json`): ledger state, request, expected answer, for every v2-only code, the v1-writer fallbacks, replay, suspension, closure, the read-only paths, approvals, verifier context, lifecycle by disposition, and the S1 refusals (I-1, I-2, I-3). The test validates request and answer against the right dialect's schema, runs the rules that need the ledger and request (lifecycle, read-only paths, anchor at freeze, approval, verifier context, human settlement), and fails if a v2-only code has no scenario.

## 7. Execution gating (`execution-episode-ref.v1.json`)

Closes S1 finding F1(b): I-1 "freeze before move" was enforced only at settlement. The field `episode_ref` is added to the three execution entry points: `item_execute`, `case_advance` and `agent_run.delegate` (wire verb `delegate`). It is `{case_id, plan_item_id, n, kind: evaluated | exploratory}` and names the episode by key; the case keeper resolves the frozen `episode_ref` from the ledger and checks `kind` against the freeze, never trusting the caller's tag. **I-1 is checked on the server-resolved execution identity (the case resolved from `case_id`, the plan item and episode counter from the ledger), never on a caller-supplied tag or on the mere presence of a field** (E section 3; G NT-32).

**Case binding (C1).** `delegate` has a required `case_id`. Earlier, a Field Case delegate without `episode_ref` was byte-identical to a plain delegate, so the template could not be told from the request. Now the server resolves the template from `case_id`; a `case_id` that does not resolve is `CASE_UNRESOLVED` (fail closed, never assumed plain); on a Field Case a missing `episode_ref` is `FREEZE_MISSING`; `episode_ref.case_id` must equal `case_id` (`SCHEMA_INVALID` otherwise). A delegate with no `case_id` is no longer a valid request, which changes the contract for legacy callers: they must name their case. Request shapes are the existing ones (`$defs` `ItemExecuteRequest`, `CaseAdvanceRequest`, `DelegateRequest`, `HumanTaskCompleteRequest`) plus the new fields; `delegate` also gets `assignment_id` (section 11) on a Field Case. The response of an accepted request is the verb's existing response, unchanged; the case keeper records the resolved `episode_ref` on the run record (section 8).

A freeze is **complete** (one definition, here and in section 6) when all of these exist: the RealityTrace plan, question and threshold; the `episode.frozen` case record; the external anchor, published by the case-keeper identity, **when an anchor is configured** (a deployment with no anchor configured is complete without it). `FREEZE_MISSING` means **none** of the parts exist; `FREEZE_INCOMPLETE` means **some but not all** (NT-33 a, b, c), including the anchor-only state (anchor exists, RealityTrace records and case record do not).

| Rule | Condition (template `fde.field_case@1`) | Refusal | NT |
|---|---|---|---|
| R1 | request has no `episode_ref` (and, for `case_advance`, no `transition_items`), whatever the ledger holds | `FREEZE_MISSING` | NT-32 |
| R2 | `episode_ref` names an episode with none of the parts | `FREEZE_MISSING` | NT-32 |
| R3 | some but not all parts exist (including anchor-only) | `FREEZE_INCOMPLETE` | NT-33 a, b, c |
| R4 | all parts exist, the anchor was not pushed by the case-keeper identity | `ANCHOR_PUBLISHER_INVALID` | NT-48 |
| R5 | `episode_ref.kind` differs from the frozen kind | `KIND_IMMUTABLE` | NT-13 |
| R6 | the latest case-level disposition is `suspend` | `CASE_SUSPENDED` | NT-22 |
| R7 | the program-execution decision (section 10) is denied, expired, absent, exhausted (`runs_used >= max_runs`), or does not bind this execution (any of the nine bound members differs) | `AUTHORITY_DENIED` | NT-34 |
| R8 | complete freeze, kind matches, decision allows | accepted | AT-04 |
| R9 | the case template is not `fde.field_case@1` | unaffected; `episode_ref` is accepted and ignored. A `case_id` that does not resolve is `CASE_UNRESOLVED` | |
| R10 | `delegate`: `assignment_id` is not `assign:<case_id>:<plan_item_id>:<n>:<k>` of this episode (`ASSIGNMENT_ID_INVALID`); a **new** delegation takes an unused `k` (`ASSIGNMENT_ID_INVALID` if `k` is issued and that assignment is not interrupted); a **resume** reuses the id of an interrupted assignment, and only while its link has not expired (`ASSIGNMENT_LINK_EXPIRED`: a new link and a new `k` are then needed) | see left | AT-17 |
| R11 | **any path** to a terminal state with no case-level `close-*` disposition: `case_advance`, an `item_execute` that finishes the last item, `human_task_complete`, any complete verb | `CLOSE_DISPOSITION_REQUIRED` | NT-23 |
| R12 | the episode's `n` is below the latest allocated `n` for the plan item, or the episode is already settled | `EPISODE_SUPERSEDED`, `EPISODE_SETTLED` | RD-10 |
| R13 | the case has a case-level `close-*` disposition | `CASE_CLOSED` | RD-10 |
| R14 | `case_advance` on a Field Case without `episode_ref`: `transition_items` must list each affected item with `transition_only: true` (`skip` or `complete`); an item that has an execution step is refused | `TRANSITION_ONLY_INVALID`; none stated is `FREEZE_MISSING` | AT-10 |

Order of evaluation: R9, R13, R6, R11 and R14 (the transition-only path ends here), R1/R2, R3, R4, R5, R12, request consistency, R7, R10. A gate refusal happens before any run, spend or record: the Error carries `no_side_effect: true`. Request consistency: for `item_execute` `case_id` and `item_id` equal the `episode_ref`'s; for `case_advance` and `delegate` `case_id` equals it; `delegate` requires `assignment_id` with `episode_ref` (`SCHEMA_INVALID`, or `ASSIGNMENT_ID_INVALID` for a mismatching id). `case_advance` with an `episode_ref` executes only that episode's plan item. The `x-decision` "an advance that executes nothing needs no `episode_ref`" of the first draft is **reversed** (H7): the advance must say so, per item, with `transition_only: true`.

Error table (`$defs/Error`, `{outcome: "refused", code, message, invariant?, no_side_effect: true}`): `FREEZE_MISSING` (I-1), `FREEZE_INCOMPLETE` (I-1), `KIND_IMMUTABLE` (I-3), `SCHEMA_INVALID`, `CASE_SUSPENDED`, `CASE_CLOSED`, `AUTHORITY_DENIED`, `ANCHOR_PUBLISHER_INVALID`, `MODEL_ANCHOR_LATE`, `CLOSE_DISPOSITION_REQUIRED` (home), `ASSIGNMENT_ID_INVALID`, `ASSIGNMENT_LINK_EXPIRED`, and the codes defined here: `EPISODE_SUPERSEDED` (R12: `n` below the latest allocated `n`), `EPISODE_SETTLED` (R12), `CASE_UNRESOLVED` (R9: `case_id` does not resolve), `TRANSITION_ONLY_INVALID` (R14). Other conditions are those of section 6 and section 11. `MODEL_ANCHOR_LATE` at the gate: the model anchor is absent or not before the first run event. These verbs are not append verbs; v1 writers do not apply.

SFWP frames: `fixtures/execution-episode-ref/frames/*.json` give, for each of the three verbs, the request with and without the field on a Field Case (refused `FREEZE_MISSING`; accepted; refused `FREEZE_INCOMPLETE`) and without it on another template (accepted; byte-identical to today's frame except that `delegate` now carries `case_id`). They are `status: expected`; the goldens are captured by WP-21. Scenarios (`scenarios/*.json`, 82) give the freeze state and case state for each rule and verb; the test runs the rule table above over them, including the pair of delegates that differ only by the case they resolve to.

## 8. Delegation link and run records (`delegation-link.v1.json`)

Closes the I-2 residual "delegate then settle" (S1 finding F7) and the hole found in review (C2): `item_execute` and `case_advance` execute under the request actor and write no link, so "delegate of any link" was an empty set for them. Records, all written by the case keeper, never a client:

- `Link` `{link_id, case_id, episode_ref, assignment_id, delegator, gateway_actor, delegate, issued_at, expires_at}` for every accepted `delegate` of a Field Case episode. `delegator` is the principal the run was delegated on behalf of (`on_behalf_of` when a gateway sent it, else the request actor); `gateway_actor` is the gateway actor or null. Both are recorded (M7).
- `RunRecord` `{run_id, case_id, episode_ref, verb, role, actor, on_behalf_of, delegated, link_id, workspace_ref, started_at}` for **every** execution of any verb carrying the episode's `episode_ref`. `actor` null means it could not be resolved.

**Ordering rule.** The link is written **before** the run starts: `link.issued_at` is not after the `started_at` of any run that cites it. A delegated run has a `link_id` (schema); a link written after its run started refuses settlement.

**Executor set (C2).** The union of: the `actor` and the `on_behalf_of` principals of every `executor` run of the episode; the `delegate` of every link (except a link cited only by `verifier` runs); and the transitive closure of the `delegator`s of those. Settlement is refused with `SELF_SETTLEMENT_REFUSED` (invariant `I-2`, the code `case-record-append.v1` uses) when the settling principal is in the set, whatever the verb the run used.

**Fail closed.** `DELEGATION_LINK_INVALID`: a run with no resolvable `actor`; a delegated run whose `link_id` has no `Link` (or whose link delegate is not the run's actor); a link written after its run; a link or run naming another episode; `expires_at` not after `issued_at`; `delegator` equal to `delegate`; an assignment id disagreeing with the episode or `case_id`. Expired links still count at settlement (a link is never deleted). A human root settles only by A3 (section 10), still subject to this rule.

**Identity aliasing (M7).** Principals are compared after the repository's canonicalisation: trimmed and case-folded, as `canonical_actor` in `crates/sea-forge-server/src/sfwp/case_records.rs` does (it adds refusals, never misses one; Unicode NFC is not applied). Residual until S3: two unrelated ids for one human are indistinguishable.

**Verifier context (H2).** Derived from the `RunRecord`s as in section 6; scenario input `Settlement` carries `result`, `declared_verifier_context` and `settler_root` (the root of the settler's own delegation chain). `VERIFIER_CONTEXT_MISMATCH` carries no invariant (section 6 error table; D40).

**Verifier runs (D44).** A verifier run is a `delegate` to one of the configured `verifier_endpoints`; its principal is `agent:<endpoint>` and its `role` is `verifier`. A verifier run never adds its actor to the executor set; a verifier endpoint that is also an executor of the episode is in the executor set through its executor run.

`$defs`: `Link`, `RunRecord`, `Settlement` (the check input), `Error` (`SELF_SETTLEMENT_REFUSED`, `DELEGATION_LINK_INVALID`, `VERIFIER_CONTEXT_MISMATCH`; invariant `I-2` on the first two, none on `VERIFIER_CONTEXT_MISMATCH`, D40). `DELEGATION_LINK_INVALID` has its home here. `SELF_SETTLEMENT_REFUSED` and `VERIFIER_CONTEXT_MISMATCH` are defined in section 6 and listed here as also raised. v1 writers: the rule applies to every settlement; v1 writers receive the v1 code as before.

## 9. Continuation and read verbs (`continuation.v1.json`)

`case.continuation` is computed on every read from the ledger plus RealityTrace and Cognate references, never stored (E section 3). `Continuation` has one property per item of SOP 10.1 (`x-sop-item` 1 to 9, `x-sop-segment` the SOP text), `availability` and `computed_from`:

| SOP 10.1 item | Property | Content |
|---|---|---|
| case ID, owner, purpose, current decision | `case` | `case_id`, `lifecycle`, and `owner`, `purpose`, `current_decision` as `FieldValue` (from the Field Case document) |
| pinned world and checkpoint | `pinned_world` | `world_ref`, `checkpoint` |
| gate evidence | `gate_evidence` | all nine gates in SOP order, each once: `state`, `evidence_refs`, `record_id` (unassessed with null record when never assessed) |
| frozen or pending episodes | `episodes` | per episode `episode_key`, `episode_ref`, `kind`, `state` (`pending`, `frozen`, `settled`), `result`, `settlement_ref` |
| exact authority/approval state | `authority` | pending approvals, decisions with assurance (`outcome` is `allow`, `deny` or `expired`, the CEP/S1 spelling used by the authority decisions, `cep_disposition` in `crates/sea-forge-authority/src/cep.rs`), `a3_channel` |
| disputed claims | `disputed_claims` | `ref`, `why` |
| latest observations with RealityTrace references, unknown and contradictory evidence exposed | `latest_observations` | `observations` (RealityTrace ref, `standing`, `basis`), `unknown_evidence`, `contradictory_evidence` |
| active assignments and trajectories, artifact locations, verified commands, remaining budgets | `work` | assignments, artifacts, verified commands, remaining budget per currency (number or `"unknown"`, never 0 for unmeasured) |
| next affordable action or blocker, who can authorize it, what would settle it | `next` | `kind` (`action`, `blocker`), `description`, `authorizer`, `would_settle`, `dimension` for a blocker (one of the seven SOP 7.2 dimensions) |

**Unknown versus empty (M2).** `availability` gives each of the nine sections `known` or `unknown`. `[]` means "none" only in a `known` section; a section whose source (RealityTrace, Cognate, the approval store) was unreachable is `unknown` and its lists are empty only because nothing could be read. `case`, `pinned_world`, `gate_evidence`, `episodes` and `next` come from the ledger and are always `known` (an unreadable ledger is `STORE_INTEGRITY`). A null `realitytrace_snapshot` forces `latest_observations` and `disputed_claims` to `unknown`. `computed_from`: `ledger_head`, `realitytrace_snapshot`, `cognate_ref`, `computed_at`. Semantic rules (`CONTINUATION_INVALID`, test-local): nine gates in order, each once; a `met` gate has evidence; currencies unique; a closed case has no active assignment; the availability rules above.

**Who sees `pending_id` (M1).** `pending_approvals[].pending_id` is returned only to the case lead (the `observer_and_authority.case_lead` principal) and to the approver of record of that pending operation; every other caller gets `null`. The pending operation itself is unchanged.

### Read verbs

| Logical verb | Wire request | Response | Errors |
|---|---|---|---|
| `case.field.get` | `{verb: "case_field_get", case_id, actor, request_id?}` | `{case_id, revision, document}` (`document` is `field-case.v1`, `document.revision` equals `revision`) | `AUTHORITY_DENIED`, `NOT_A_FIELD_CASE`, `STORE_INTEGRITY` |
| `case.continuation` | `{verb: "case_continuation", case_id, actor, request_id?}` | `{case_id, continuation}` | `AUTHORITY_DENIED`, `NOT_A_FIELD_CASE`, `STORE_INTEGRITY` |

Both are reads: no mutation, no `request_id` requirement; `actor` is required. Authorization: a caller with no read grant on the case gets **the same answer as for an unknown case** (the existing SFWP error frame, `error_class` not found), so template membership is not revealed; `AUTHORITY_DENIED` is returned only to a caller who is entitled to know the case exists but lacks the read operation. `NOT_A_FIELD_CASE` (home here) is therefore given only after authorization: the case's template is not `fde.field_case@1`. `STORE_INTEGRITY` is defined in section 6. Frames: `fixtures/continuation/frames/*.json` (expected, not goldens: WP-21 captures them).

## 10. Reserved operations and the program-execution decision (`reserved-operation.v1.json`)

### Operation, class, evidence (E section 8, RD-06, RD-21)

`fixtures/reserved-operation/valid/operation-table.json` is the table instance (15 operations); the schema rejects a reserved row below P2, P3 rows accepting A3-T, and non-reserved rows with an A3 surface. Classes follow the owner's decision D23 (E's "P2/P3" row has no stated condition, so the stricter reading applies).

| Operation | Class | Reserved | Evidence (alternatives; inner lists required together) | A3-T |
|---|---|---|---|---|
| `session_work` (read, navigate, draft, note, exploratory observation) | P0/P1 (written `P1`) | no | `session` | n/a |
| `freeze_episode` | P1 | no | `authenticated_principal` + `preview_check_code` | n/a |
| `assign_within_grant` | P1 | no | `authenticated_principal` + `preview_check_code` | n/a |
| `program_execution` | P1 | no | `authenticated_principal` + `authority_decision_bound_to_digest` | n/a |
| `settle_episode_agent` (an independent verifier agent, AT-09) | P2 | **no** | `authenticated_principal` + `verifier_not_executor` + `verifier_context_derived` | n/a |
| `settle_episode_human` | P2 | yes | `a3_proof` + `verifier_not_executor` | policy |
| `set_episode_disposition` | P2 | yes | `a3_proof` + `sea_forge_approval` | policy |
| `set_case_disposition` (continue, revise, escalate, suspend) | P2 | yes | `a3_proof` + `sea_forge_approval` | policy |
| `set_case_close_disposition` (`close-satisfied`, `close-failed`) | P3 | yes | `a3_proof` + `sea_forge_approval` | never |
| `set_gate_not_applicable` (non-hard gate; SOP 8.1 "authorized") | P2 | no | `authenticated_principal` + `sea_forge_approval` | n/a |
| `set_hard_gate_not_applicable` | P3 | yes | `a3_proof` + `sea_forge_approval` | never |
| `revise_frozen_boundary_field` (section 5) | P2 | yes | `a3_proof` + `sea_forge_approval` | policy |
| `promote_world`, `bootstrap_controller`, `recover_controller` | P3 | yes | `a3_proof` + `online_decision_bound_to_digest_and_expiry` | never |

`settle_episode` was one operation with one reserved row whose first alternative had no `a3_proof`, which a conforming implementation would have refused for the independent-verifier agent AT-09 needs (H4). It is split: an agent settlement is not reserved (its safeguards are I-2, the derived verifier context and `SELF_SETTLEMENT_REFUSED`), a human settlement is.

### A3 proof minimum

The pending operation (`PendingOperation`) is issued by SEA-Forge for the ten operations that need an approval: `{pending_id, operation, class, case_id, subject_ref, canonical_operation, operation_digest, issued_at, expires_at, evidence_required}`; `operation_digest` = SHA-256(RFC 8785(`canonical_operation`)). **`params` are required per operation and part of the digest (H3):** a disposition carries `level`, `disposition`, `case_id`, `episode_ref` (null at case level) and `record_digest`; a gate waiver carries `case_id`, `gate`, `state` and `record_digest`; a human settlement carries `case_id`, `episode_ref`, `result`, `settlement_ref` and `record_digest`; a boundary revision carries `case_id`, `paths`, `base_revision` and `record_digest`; `promote_world` carries `candidate_world_ref` and `proposal_digest`; controller operations carry `controller_id` and `record_digest`. `params: {}` is invalid. The A3 surface (the paired phone, class A3-UV) receives `canonical_operation` from SEA-Forge by `pending_id`, never from a page. The proof (`A3Proof`) is exactly the minimum of E section 13 plus the key that binds it: `pending_id`, `challenge` (SEA-Forge-issued, single use), `operation_digest`, `authenticator_assertion` (`credential_id`, `authenticator_data`, `client_data_json`, `signature`), `confirmation_surface_id` (`surface:<token>`, resolved by SEA-Forge to A3-UV or A3-T; an unknown id is not an A3 surface), `expires_at`. SEA-Forge, not Horizon, validates it. The decision (`Decision`) is `{outcome: "confirmed", decision_id, pending_id, operation, operation_digest, assurance, decided_at, expires_at}`, `assurance` `session | A3-UV | A3-T`. The `ApprovalRecord` (`approval_ref`, `operation`, `operation_digest`, `case_id`, `outcome` `allow | deny | expired`, `assurance`, `used_by`, `issued_at`, `expires_at`) is what section 6 checks.

Staged enforcement (RD-21): the check is off until WP-31; before it, decisions are recorded with assurance `session` and every row depending on a human-reserved decision is PARTIAL until S3. The Chromium virtual authenticator is valid for automated regression only (OD-1, OD-5).

### Errors

| Code | Raised when | NT |
|---|---|---|
| `RESERVED_DECISION_REFUSED` | enforcement is on and a reserved operation arrives with no A3 proof: an agent session, or the paired-browser session without the agent header, submits a disposition or promotion approval; also a `field_case.revised` of a read-only boundary path after a freeze without an A3-backed `approval_ref` | NT-12 |
| `A3_SURFACE_REQUIRED` | a proof is present but its `confirmation_surface_id` is not an accepted A3 surface: the desk page even with a valid passkey assertion; A3-T for a P3 operation (case-level close dispositions, hard-gate waivers, world and controller operations); A3-T for a P2 operation unless policy accepts it | NT-24 |
| `A3_CHANNEL_UNAVAILABLE` | no A3 channel is configured or reachable; carries `how_to_restore` | |
| `RESERVED_EXPIRED` | the pending operation, the proof, an approval or a program-execution decision is past `expires_at` (or expires before it was issued) | |
| `RESERVED_DIGEST_MISMATCH` | the proof's `pending_id` or `operation_digest` differs from the pending operation's, or a decision record's digest does not bind what it names | |
| `A3_PROOF_INVALID` | the assertion does not verify against the enrolled credential and challenge | |
| `A3_PROOF_REPLAYED` | the `challenge` was already used | |
| `AUTHORITY_DENIED` | SEA-Forge's own approval or decision for the operation is denied, unmatched, used or absent (defined in section 6) | NT-34 |

Order: enforcement off; `A3_CHANNEL_UNAVAILABLE`; `RESERVED_DECISION_REFUSED`; `RESERVED_EXPIRED`; `RESERVED_DIGEST_MISMATCH`; `A3_SURFACE_REQUIRED`; `A3_PROOF_REPLAYED`; `A3_PROOF_INVALID`; `AUTHORITY_DENIED`. The scenarios under `fixtures/reserved-operation/scenarios` run it. `RESERVED_DECISION_REFUSED` also appears in the append v2 Error enum (also raised there).

### Program-execution decision (`ProgramExecutionDecision`)

S1 showed SEA-Forge records no authority decision for program execution, so RealityTrace could not resolve `authority.decision_id` of execution events. A `program_execution` operation in a Field Case episode requires this record to exist (rule R7):

`{decision_id, operation_id, case_id, execution, binds_digest, max_runs, runs_used, decided_by: {principal, role}, outcome: allow | deny, assurance, decided_at, expires_at}`

- **Who decides**: SEA-Forge's authority gate, through the existing approval path, on a principal that is the frozen reference's `decision_maker` or holds the approver role, and is never the executor or one of its delegators (section 8). Assurance is `session` until WP-31.
- **What it binds (H5)**: `binds_digest` = SHA-256(RFC 8785(`execution`)) with `execution` = `{episode_ref, patch_sha256, argv, cwd, env_digest, workspace_ref, sandbox_profile, world_ref, timeout_s}`: the full `episode_ref`, the patch (SHA-256 of the empty string when none), the command argv, working directory, the environment (`env_digest`), the workspace, the sandbox profile, the pinned world and the timeout. An execution that differs in any member has no decision (`AUTHORITY_DENIED`); a record whose digest does not bind its own `execution` is `RESERVED_DIGEST_MISMATCH`.
- **Use counter**: `max_runs` (default 1, always present) and `runs_used`, incremented by the case keeper when a run starts under the decision. A run when `runs_used >= max_runs` is `AUTHORITY_DENIED`; a record with `runs_used > max_runs` is corrupt (`AUTHORITY_DENIED`).
- **Resolution**: `decision_id` is the value emitters send as `authority.decision_id` (E section 5); RealityTrace verifies it against SEA-Forge, scoped by `operation_id`. `outcome` uses the CEP/S1 spelling (`allow`, `deny`).
- **Realisation in SEA-Forge (D39)**: the decision is an existing CEP `authority_decision` entry in the `cognate-authority` ledger whose `operation.resource_id` is `execution:sha256:<binds_digest>`; the CEP profile is unchanged. The server builds `execution` from the frozen record and the plan item, never from the request. A decision is usable for `execution_decision_ttl_secs` (fde-config.json, default 86400) after `decided_at`. `max_runs` is `execution_max_runs` (default 1); `runs_used` is counted per `decision_id` in the case store (`field-exec-decisions.jsonl`) under the case lock. The run record and the `run.started` interaction event carry the `decision_id`.

## 11. Assignment (`assignment.v1.json`)

`assignment_id` = `assign:<caseId>:<stepId>:<n>:<k>` (E section 2, RD-13, RD-23), at most 320 characters. Regex `^assign:([^:@\s]+):([^:@\s]+):([1-9][0-9]*):([1-9][0-9]*)$`. Parse: split on `:` into exactly five parts; `caseId` is the episode's `case_id`, `stepId` its `plan_item_id`, `n` the episode counter, `k` the delegation instance within episode `n`, both from 1. Horizon proposes the id at Hand off from the case keeper's counters; SEA-Forge validates it (R10): `caseId`, `stepId` and `n` equal the `episode_ref`'s. **New versus resume (M9):** a **resume** reuses the id of an **interrupted** assignment and is allowed only while its link has not expired; a **new** delegation instance (including a resume after link expiry, which needs a new link) takes an unused `k`, normally `k+1`; a changed episode takes a new `n` and `k = 1`. `k` already issued for an assignment that is not interrupted is `ASSIGNMENT_ID_INVALID`. The old four-part form is refused (pattern).

`Assignment`: `{assignment_id, case_id, step_id, n, k, episode_ref, brief, budget, issued_at}` (the fields repeat the id and must agree with it and with `episode_ref`, semantically).

| Field | Rule |
|---|---|
| `brief` | `{instruction, adoptable_problem, domain_meaning, context_refs[]}`; each context ref `{ref, trust: "untrusted_reference"}` (AT-17: retrieved context is labelled untrusted) |
| `budget` | exactly five entries, one per currency of SOP 5.1 (`time`, `attention`, `money`, `risk`, `recovery`), each `{currency, limit: number >= 0 \| "unknown", unit}`; `unit` is an enum **per currency**: `seconds`, `operator_minutes`, an ISO 4217 code for `money`, `class`, `revert_steps`. An unmeasured currency is `"unknown"`, never 0 (SOP 5.3) |
| `Event` | `{assignment_id, episode_ref, event, at, spend, reason?}`; `event` is `issued`, `started`, `interrupted`, `resumed`, `completed`, `failed`, `cancelled`; `spend` has exactly five per-currency entries (number or `"unknown"`); `reason` is required for `interrupted`, `failed`, `cancelled` |

Lifecycle: `issued` -> `started` -> (`interrupted` <-> `resumed`)* -> `completed` | `failed` | `cancelled` (`interrupted` may be cancelled). One id and one `episode_ref` for the whole sequence (AT-17).

**Versioned binding (`assignment-binding.v1`, WP-23).** A Field Case `delegate` does not carry the assignment identity alone: the request carries `assignment` (`AssignmentApproval`), the approved Brief and Budget values — explicit `domain_meaning` and exactly five SOP 5.1 currency ceilings from the approval path, each ceiling a number or `"unknown"` (SOP 5.3: never 0 for unmeasured). `adoptable_problem` is never caller-supplied: the frozen case supplies it (the frozen reference's `criteria.intention`), and `instruction` already travels on the delegate request. SEA-Forge composes the full `Assignment`, validates it against this contract, and records it on the assignment row as `binding` `{version, assignment, link_id, run_id, binding_digest, recorded_at}`, written in the same guarded batch as the link and run records under the case lock; `binding_digest` = SHA-256(RFC 8785(`{assignment_id, episode_ref, brief, budget, link_id, run_id}`)). The existing run is the reservation and correlation identity: no separate id is minted. A resume (M9) reuses the recorded binding: the request's approval must equal the recorded `brief.domain_meaning`, `brief.context_refs` and `budget` (`instruction` travels per request); an interrupted row with no recorded binding is a legacy row and fails closed — a new `k` with a fresh approval is then needed. The response of an accepted delegate is unchanged. A trusted read joins the exact case, episode, assignment, link and run; a legacy row without the exact link, or a partial append, fails closed (`ASSIGNMENT_BINDING_INVALID`).

| Code | Raised when |
|---|---|
| `ASSIGNMENT_ID_INVALID` | the id does not parse, its parts disagree with the episode or with the assignment's fields, `k` is issued and not interrupted, or an event carries another id than its sequence |
| `ASSIGNMENT_EPISODE_CHANGED` | an event of a sequence names another `episode_ref` |
| `ASSIGNMENT_TRANSITION_INVALID` | the event is not allowed after the previous one (including any event after a terminal one) |
| `ASSIGNMENT_BUDGET_INVALID` | a currency appears twice in `budget` or `spend` |
| `ASSIGNMENT_BUDGET_EXCEEDED` | a numeric `spent` is above the numeric `limit` of its currency (an unknown limit or unknown spend never trips it) |
| `ASSIGNMENT_LINK_EXPIRED` | a `resumed` event, or a delegate that reuses an interrupted assignment's id, comes after the delegation link's `expires_at` |
| `ASSIGNMENT_APPROVAL_REQUIRED` (assignment-binding.v1) | a Field Case delegate carries no `assignment`: the generic case budget and token ceiling are insufficient |
| `ASSIGNMENT_APPROVAL_INVALID` (assignment-binding.v1) | the approval is not `AssignmentApproval` (shape, a currency missing or repeated, a wrong unit), or the frozen episode supplies no adoptable problem |
| `ASSIGNMENT_APPROVAL_MISMATCH` (assignment-binding.v1) | a resume's approval differs from the recorded binding, or the interrupted assignment is a legacy row with no recorded binding |
| `ASSIGNMENT_BINDING_INVALID` (assignment-binding.v1) | the trusted read cannot join case, episode, assignment, link and run exactly: a legacy row without the binding's exact link, a partial append, a digest that does not recompute, or a row naming another episode |

## 12. Independent observation (normative)

S1 finding: executor-only evidence cannot confirm. RealityTrace caps an episode whose every occurrence comes from the executor's emitter at class `insufficient`, basis `executor_only_evidence` (E section 5; G NT-20). For execution steps, these SEA-Forge-side records count as an **independent emitter**, because SEA-Forge writes them, not the executor:

1. the program-execution decision (section 10), emitted by `sea-forge-authority` with `source.record_id` = `decision_id`;
2. the case-ledger event for the execution (the `RunRecord` of section 8 the case keeper writes when it accepts and completes the gated request), emitted by `sea-forge-ledger`;
3. the verifier's own observation (WP-17/WP-43), which is not an execution step and is out of this contract.

The executor's own emitter (Cognate, the agent's tool log) never counts toward independence. Both SEA-Forge events carry the episode's `episode_ref`, the `assignment_id`, `authority.decision_id` equal to the decision, `executor_provenance`, and `kind`; they are resolvable against the owning system's record (E section 5). `confirmed` needs at least one occurrence from such an emitter. The expected event shape is `fixtures/independent-observation/` (three events: authority decision, ledger run, executor run, in the CEP `godspeed.interaction_event.v1` shape; `semantic_ref` and `declaration_key` are copied from the CEP fixture and are illustrative). This section and the fixtures are documentation; no schema is added here and WP-21/WP-17 emit the events.

## 13. Decisions where E, G and the SOP do not determine a field or rule

Each is the minimal option and is marked `x-decision` for review; none is inside a schema. Review fixes of 2026-11-15 reversed or replaced D4, D8, D12, D17, D23 and D25 and added D31 to D38; the owner accepted D13 and D15 as stricter than E and the second hard gate in D12 as stricter.

| # | Where | `x-decision` |
|---|---|---|
| D1 | field-case | `x-decision: "WP-20: the model does not declare the SOP 4.2 fields (map row 4.2 is unknown, S7), so the schema is generated from the SOP 4.2 table and bound to it by hash and by label; the check fails if the map row changes"` |
| D2 | field-case | `x-decision: "WP-20: FieldValue is {status, source_ref, value?}; source_ref is {kind, ref} with kind none exactly for unknown; value is any non-null JSON (E fixes no per-field type)"` |
| D3 | field-case | `x-decision: "WP-20: 45 fields (the SOP table has no count; the map says about 60); keys are the SOP labels in snake case; revision starts at 1 at case open; the document names the SOP version and sha256 it follows"` |
| D4 | append v2 | `x-decision: "WP-20: REVERSED by review H1. The contract marker selects the response dialect only; every rule applies to every writer; each v2-only code has a v1 fallback that is itself a refusal, or is unreachable because only a v2-only kind raises it; a test asserts no v1-dialect scenario is accepted"` |
| D5 | append v2 | `x-decision: "WP-20: KIND_IMMUTABLE takes precedence over FREEZE_KEY_EXISTS for v2 writers when the kind differs; v1 README decision 8 said it was unused. v1 writers keep FREEZE_KEY_EXISTS"` |
| D6 | append v2 | `x-decision: "WP-20: REFERENCE_DERIVED_FROM_RESULT is best-effort detection: a result-field property in reference.body (list: result, outcome, observation, settlement, verdict, residual) or a string quoting a RealityTrace result id with a 26-character id. It is not complete and does not pretend to be: the real protection is ordering, freeze and anchor before the first run event"` |
| D7 | append v2 | `x-decision: "WP-20: WORLD_NOT_PINNED compares with the SEA-Forge world pin record (WP-18 world_pin), never a Field Case field; MODEL_ANCHOR_LATE and ANCHOR_PUBLISHER_INVALID are raised at settlement and at the execution gate, and ANCHOR_PUBLISHER_INVALID also at freeze when an anchor is configured (the configured credential, before the record is written, so the key is not burned)"` |
| D8 | append v2 | `x-decision: "WP-20: REPLACED by owner decision. G NT-22 (freeze and delegate refused), E (new freezes and delegations are refused, NT-22) and the SOP are silent on more, so a suspended case also refuses settlement and every new disposition except the case-level continue that lifts it and a close-* that closes it; revision, strategy and gate records are still accepted"` |
| D9 | append v2 | `x-decision: "WP-20: PAYLOAD_TOO_LARGE limit is 1048576 bytes of encoded request (the 1 MiB frame limit the server already uses in mcp.rs); STORE_INTEGRITY and PAYLOAD_TOO_LARGE reach v1 writers as the existing SFWP error frame"` |
| D10 | append v2 | `x-decision: "WP-20: new code REVISION_CONFLICT (optimistic concurrency for field_case.revised base_revision and strategy.recorded revision); no G row needs it"` |
| D11 | append v2 | `x-decision: "WP-20: field_case.revised carries base_revision >= 1, a list of path/FieldValue changes and a rationale; strategy.recorded carries the five SOP 4.4 questions plus revision, supersedes, revisit_reason"` |
| D12 | append v2 | `x-decision: "WP-20: REVERSED by owner decision. assessor is back in the gate.assessed payload (E section 3 lists it) and must equal the authenticated principal; for prove_reliability (SOP 8.1: independent repeated settlement) a proposer is required and assessor equal to proposer is SELF_SETTLEMENT_REFUSED. Hard gates remain allocate_authority and integrate_govern (accepted as stricter than G, which names only allocate_authority); any not_applicable needs approval_ref"` |
| D13 | append v2 | `x-decision: "WP-20: disposition.recorded requires approval_ref at both levels (E: behind the existing approval path); an episode-level disposition of any kind never changes the lifecycle; the response carries the resulting lifecycle"` |
| D14 | kinds | `x-decision: "WP-20: the registry is data (fixtures/case-record-kinds/valid/registry.json) validated by a closed schema with exactly six kinds; immutability classes frozen_forever, single_settlement, revision_chain, append_only_history; REGISTRY_INCOMPLETE and REGISTRY_NATURAL_KEY_INVALID are test-local"` |
| D15 | execution | `x-decision: "WP-20: episode_ref is the key {case_id, plan_item_id, n, kind}, not the digest form; both evaluated and exploratory episodes need a complete freeze (E says evaluated; exploratory frozen references exist in S1 and an exploratory run must also be labelled at the time)"` |
| D16 | execution | `x-decision: "WP-20: FREEZE_MISSING means none of RealityTrace records, case record, anchor exist; FREEZE_INCOMPLETE means some but not all (NT-33 a, b, c); ANCHOR_PUBLISHER_INVALID when all exist but the pusher is not the case keeper"` |
| D17 | execution | `x-decision: "WP-20: REVERSED by review H7. case_advance on a Field Case carries episode_ref or states per item transition_items with transition_only: true; an execution item is refused (TRANSITION_ONLY_INVALID); every path to a terminal state is gated by CLOSE_DISPOSITION_REQUIRED. Non-Field-Case cases still accept and ignore episode_ref"` |
| D18 | execution | `x-decision: "WP-20: delegate carries assignment_id beside episode_ref (the wire verb has no case_id); refusals before any side effect carry no_side_effect: true, the field the existing SFWP refusals use; the accepted response is unchanged"` |
| D19 | execution | `x-decision: "WP-20: a missing or denied program-execution decision is AUTHORITY_DENIED (fail closed), not a separate code"` |
| D20 | delegation | `x-decision: "WP-20: the executor set is the union of actor and on_behalf_of of every executor run, link delegates and the transitive delegator closure (review C2, wider than the first draft); expired links still count; the link precedes the run; derived, not asserted, verifier context (review H2; the payload field is informational and must equal the derived value); the link is written by the case keeper in cases/<id>/delegation_links.jsonl (path is WP-21's)"` |
| D21 | continuation | `x-decision: "WP-20: one property per SOP 10.1 item (nine), plus availability and computed_from; a null RealityTrace snapshot forces observations and disputed claims to unknown, never absent; decision outcomes are allow, deny, expired (CEP/S1 spelling); pending_id is returned only to the case lead and the approver of record; next.dimension uses the seven SOP 7.2 ids; CONTINUATION_INVALID is test-local"` |
| D22 | read verbs | `x-decision: "WP-20: wire verbs case_field_get and case_continuation (existing read verbs are case_get_overview, case_get_horizon); actor is required; an unauthorized caller gets the unknown-case answer, AUTHORITY_DENIED only for a caller entitled to know the case; new code NOT_A_FIELD_CASE only after authorization"` |
| D23 | reserved | `x-decision: "WP-20: REPLACED by owner decision (follow E, do not downgrade). E section 8 table row: `case-level disposition; set a hard gate not-applicable \| P2/P3 human-reserved \| A3 + SEA-Forge approval` states no condition for which applies; E RD-21 text: `A3-T ... is never accepted for a P3 operation`. So case-level close-* dispositions and every hard-gate waiver are P3 (A3-T never), episode-level and other case-level dispositions P2 (A3-T by policy); program_execution is P1 with a decision bound to the digest; the proof is E's five fields plus pending_id; the surface class is resolved by SEA-Forge"` |
| D24 | reserved | `x-decision: "WP-20: codes RESERVED_DECISION_REFUSED (NT-12) and A3_SURFACE_REQUIRED (NT-24) are G's; added A3_CHANNEL_UNAVAILABLE (with how_to_restore), RESERVED_EXPIRED, RESERVED_DIGEST_MISMATCH, A3_PROOF_INVALID, A3_PROOF_REPLAYED; settle_episode is split into settle_episode_agent (not reserved) and settle_episode_human (reserved); before WP-31 decisions are recorded with assurance session"` |
| D25 | reserved | `x-decision: "WP-20: the program-execution decision is decided by SEA-Forge's authority gate on the frozen reference's decision_maker or an approver principal that is not the executor or its delegator; binds_digest covers the nine-member execution object {episode_ref, patch_sha256, argv, cwd, env_digest, workspace_ref, sandbox_profile, world_ref, timeout_s} (review H5, wider than the first draft); max_runs defaults to 1 with a runs_used counter; no patch is the SHA-256 of the empty string"` |
| D26 | assignment | `x-decision: "WP-20: brief is {instruction, adoptable_problem, domain_meaning, context_refs[]}; budget is exactly five entries; unit is an enum per currency (seconds, operator_minutes, ISO 4217 for money, class, revert_steps), the model declares only the currency ids; spent above a numeric limit is ASSIGNMENT_BUDGET_EXCEEDED"` |
| D27 | assignment | `x-decision: "WP-20: the lifecycle events and their state machine, and the four ASSIGNMENT_* codes, are defined here; E names only interruption and resume"` |
| D28 | oracle codes | `x-decision: "WP-20: exactly one error table means exactly one home table defining the condition; another contract's Error enum may list the code (also_raised_by) by reference; v1 and v2 of the append contract are one lineage"` |
| D29 | independent observation | `x-decision: "WP-20: the SEA-Forge records that count as independent for execution steps are the program-execution decision and the ledger run record; the verifier's own observation is out of scope; the fixture's semantic_ref and declaration_key are copied from the CEP fixture"` |
| D30 | all | `x-decision: "WP-20: fixtures for case-record-append.v2 live in fixtures/case-record-append-v2 (the v1 directory is untouched); frames are status expected, not goldens"` |
| D31 | append v2 | `x-decision: "WP-20: approval_ref is bound, single-use evidence: the approval's operation_digest equals the digest of the canonical operation built from this record (per-operation required params including record_digest, case_id, disposition kind and level, gate id or episode_ref), same case, allow, unexpired, unused; otherwise AUTHORITY_DENIED. A human settlement finds its approval by that digest because the closed v1 settled payload has no approval_ref"` |
| D32 | append v2 | `x-decision: "WP-20: CASE_CLOSED is final: after a case-level close-* the case refuses freeze, gate assessment, field revision, strategy revision and every disposition (continue and suspend included); late settlement is still accepted"` |
| D33 | append v2 | `x-decision: "WP-20: the Field Case paths world_and_boundary.world_ref_and_version, observer_and_authority.independent_verifier, settlement_and_disposition_authority, delegated_actors and risk_and_external_effect_limits are read-only after the first freeze; revising them is the reserved operation revise_frozen_boundary_field (P2, A3 + approval), refused with RESERVED_DECISION_REFUSED at session level"` |
| D34 | execution | `x-decision: "WP-20: delegate carries a required case_id (a contract change for legacy callers, which had none); an unresolvable case is CASE_UNRESOLVED; R12 refuses EPISODE_SUPERSEDED and EPISODE_SETTLED; freeze complete means anchor too only when an anchor is configured; FREEZE_MISSING is none of the parts, FREEZE_INCOMPLETE some but not all, one definition in sections 6 and 7"` |
| D35 | delegation | `x-decision: "WP-20: both the gateway actor and the on-behalf-of principal are recorded on links and runs; principals are compared after the existing canonical_actor rule (trim, case-fold); two unrelated ids for one human stay indistinguishable until S3"` |
| D36 | reserved | `x-decision: "WP-20: non-hard gates may be set not_applicable only with an approval (set_gate_not_applicable, P2, not reserved, SOP 8.1 'authorized'); the pending operation covers ten operations, with params required per operation"` |
| D37 | all | `x-decision: "WP-20: bounds: maxLength 4096 on every S2 string unless bounded otherwise, maxItems 256 on arrays unless bounded otherwise (changes 64, budget and spend 5); an observed field cannot carry source kind operator or sop"` |
| D38 | assignment | `x-decision: "WP-20: a resume reuses the id of an interrupted assignment while its link has not expired; a new delegation instance, and a resume after link expiry, take a new k (ASSIGNMENT_LINK_EXPIRED / ASSIGNMENT_ID_INVALID)"` |
| D39 | reserved | `x-decision: "WP-21a: the program-execution decision is a CEP authority_decision whose operation.resource_id is execution:sha256:<RFC 8785 digest of the nine-member execution object>; no CEP profile change; the server builds the object from the frozen record and the plan; TTL execution_decision_ttl_secs (default 86400); runs_used counted per decision_id in the case store under the case lock; max_runs = execution_max_runs (default 1)"` |
| D40 | delegation | `x-decision: "WP-21a: VERIFIER_CONTEXT_MISMATCH carries no invariant (section 6 error table); the delegation-link fixture sibling-self-asserted-fresh-is-refused expected I-2 in error and is corrected; a new append-v2 scenario separates declared != derived from the sibling rule"` |
| D41 | append v2 | `x-decision: "WP-21a: MODEL_ANCHOR_LATE at freeze applies only once the case holds a run; before the first run a model anchor cannot be late"` |
| D42 | append v2 | `x-decision: "WP-21a: a closed case refuses every write except the action that ends the plan (CASE_CLOSED is final, D32)"` |
| D43 | append v2 | `x-decision: "WP-21a: idempotency precedes EPISODE_REF_DIGEST_MISMATCH (the S1 order): a replay of an accepted request returns the original record; the precedence list is corrected to the implemented order"` |
| D44 | delegation | `x-decision: "WP-21a: verifier runs are delegations to the configured verifier_endpoints with principal agent:<endpoint> and role verifier"` |


## 14. Bounds (review LOW)

Every S2 string has a `maxLength` (4096 unless a pattern, enum or const bounds it; identifiers 128 to 320) and every array a `maxItems` (256 unless stated): `field_case.revised.changes` 64, `budget` and `spend` exactly 5, `evidence_required` 256. Free-form objects are the documented exceptions: the `value` of a `FieldValue`, `canonical_operation.params` (bounded by its per-operation shape), `criteria` of `delegate` (the server's existing type), and the `expect`/`ledger` sections of scenario fixtures. The test fails if an S2 string or array is unbounded.

