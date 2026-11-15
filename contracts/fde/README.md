# FDE contracts (WP-11)

Reviewed-before-code contracts for the Field Case path. JSON Schema draft 2020-12, every object closed (`additionalProperties: false`). These files are hand-authored; unlike `contracts/schema/`, no generator writes them. A change to a schema file needs a new line in `DIGESTS` in the same commit (`scripts/check-fde-contract-digests.sh --update`); consumers vendor by digest.

| Contract | File | Fixtures (valid / invalid, schema + semantic) | Test |
|---|---|---|---|
| Frozen reference | `frozen-reference.v1.json` | 3 / 16 + 1 | `crates/sea-forge-realitytrace/tests/fde_contracts.rs` |
| Anchor | `anchor.v1.json` | 3 / 8 + 2 | same |
| `case.record.append` | `case-record-append.v1.json` | 12 / 18 + 1, 11 scenarios | same |
| Field Case document | `field-case.v1.json` | 3 / 18 + 0 | same |
| Record-kind registry | `case-record-kinds.v1.json` | 1 / 11 + 2 | same |
| `case.record.append` v2 | `case-record-append.v2.json` | 38 / 31 + 6, 26 scenarios | same |
| Execution `episode_ref` | `execution-episode-ref.v1.json` | 18 / 20 + 7, 42 scenarios, 12 frames | same |
| Delegation link | `delegation-link.v1.json` | 5 / 11 + 9, 3 scenarios | same |
| Continuation and read verbs | `continuation.v1.json` | 10 / 23 + 7, 4 frames | same |
| Reserved operation | `reserved-operation.v1.json` | 17 / 32 + 8, 12 scenarios | same |
| Assignment | `assignment.v1.json` | 7 / 23 + 5, 7 scenarios | same |

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
| `source_ref` | `{kind: record \| realitytrace \| cognate \| horizon \| operator \| sop \| none, ref}` | yes | `kind` is `none` (and `ref` is `none`) exactly when `status` is `unknown` |
| `value` | any non-null JSON | when `status` is not `unknown` | absent when `status` is `unknown` |

### Check against the model

The declared model does not declare the 4.2 fields: translation-map row `4.2` is `unknown`, limitation S7 (scope), L2 (lists). The schema is therefore generated from the SOP 4.2 table, not from model elements; `scripts/check-fde-field-case-schema.sh` binds it to the SOP text and fails if the map row ever stops being `unknown` (then the schema must be regenerated from the model). The same script checks the enums the S2 contracts reuse against the model: gate names and states, disposition kinds and levels, case lifecycle, episode kinds, epistemic results, payment currencies, reliability and aim dimensions, and the SOP 10.1 items of `continuation.v1`.

### Errors

None of its own: a document that fails the schema read from the store is `STORE_INTEGRITY` (section 6); a request that revises it wrongly is refused by `case.record.append` (`SCHEMA_INVALID`, `REVISION_CONFLICT`).

## 5. Record-kind registry (`case-record-kinds.v1.json`)

The registry instance is `fixtures/case-record-kinds/valid/registry.json`. Per kind: payload schema (`schema` + `def`), idempotency-key rule, natural key, refusal codes, immutability rule, whether a suspended case refuses it.

| Kind | Since | Scope | Payload schema | Natural key | Immutability | Refused when suspended |
|---|---|---|---|---|---|---|
| `episode.frozen` | v1 | episode | `case-record-append.v1` `EpisodeFrozenPayload` (unchanged) | `(case_id, plan_item_id, n)` | `frozen_forever` (I-3) | yes |
| `episode.settled` | v1 | episode | `case-record-append.v1` `EpisodeSettledPayload` (unchanged) | `(episode_ref)` | `single_settlement` | no |
| `field_case.revised` | v2 | case | `case-record-append.v2` `FieldCaseRevisedPayload` | `(case_id, base_revision)` | `revision_chain` | no |
| `strategy.recorded` | v2 | case | `StrategyRecordedPayload` | `(case_id, revision)` | `revision_chain` | no |
| `gate.assessed` | v2 | case | `GateAssessedPayload` | none | `append_only_history` | no |
| `disposition.recorded` | v2 | case | `DispositionRecordedPayload` | none | `append_only_history` | no |

Idempotency-key rule, all kinds: one whitespace-free token of at most 256 characters, the cross-system record key `(system, record_id)` (RD-12). Same key and same body replays the original response (`replayed: true`); same key and another body is `IDEMPOTENCY_CONFLICT`. The registry is closed to these six; a seventh is a new registry version. The refusal codes of each kind are listed in the registry and checked against the v2 Error enum by the test; the semantic rules `REGISTRY_INCOMPLETE` (a kind lacks a code its invariant needs) and `REGISTRY_NATURAL_KEY_INVALID` (a case-scoped kind names an episode field) are test-local.

## 6. `case.record.append` v2 (`case-record-append.v2.json`)

Supersedes v1 **for new writers**; v1 stays valid and is unchanged. `$defs`: `Request`, `Response`, `Error`, `FieldCaseRevisedPayload`, `FieldChange`, `StrategyRecordedPayload`, `GateAssessedPayload`, `DispositionRecordedPayload`; the two v1 payloads are referenced from v1.

**Dialects.** A request with `contract: "fde.case-record-append@2"` is a v2 writer and is answered with the v2 Error enum. A request without it is a v1 writer: it may use only the two v1 kinds (`KIND_UNKNOWN` otherwise) and is answered in the v1 dialect, where each v2-only code is replaced by the fallback below (the v2 code name prefixes the message) or is unreachable. The `contract` marker is required for the four new kinds. A v1 writer therefore never receives a code its closed v1 Error schema does not know. Scenario fixtures `*-v1-writer-fallback.json` validate those v1 answers against the v1 schema.

### Request

`{contract?, kind, payload, idempotency_key}`, closed. `payload` per kind:

| Kind | Payload fields |
|---|---|
| `field_case.revised` | `base_revision` (integer >= 1, the revision this was written against), `changes[]` (>= 1, each `{path: "<group>.<field>", field: FieldValue}`), `rationale`. Creates revision `base_revision + 1` |
| `strategy.recorded` | `revision` (1 for the first), `supersedes` (record_id of revision-1, null for 1), `where_to_play`, `how_to_win`, `underserved_outcome`, `tested_assumption`, `abandonment_trigger` (SOP 4.4), `revisit_reason` (null for 1, required text after) |
| `gate.assessed` | `gate` (the nine of SOP 8.1), `state` (`met`, `blocked`, `unassessed`, `not_applicable`), `evidence_refs[]` (>= 1 when `met`; no duplicates), `rationale`, `approval_ref?`. The assessor is the authenticated principal, recorded by the case keeper (as for settlement), not a payload field |
| `disposition.recorded` | `level` (`case`, `episode`), `disposition` (`close-satisfied`, `close-failed`, `continue`, `revise`, `escalate`, `suspend`), `episode_ref` (exactly when `level` is `episode`), `rationale`, `approval_ref` |
| `episode.frozen`, `episode.settled` | as v1, unchanged |

Hard gates (`approval_ref` required for `not_applicable`): `allocate_authority`, `integrate_govern` (x-decision, section 13). Lifecycle is the pure function of E section 3 over the ledger: `closed` only when the case state is terminal and a case-level `close-*` disposition exists; `suspended` when the latest case-level disposition is `suspend`; otherwise `open`. Episode-level dispositions never change it.

### Response

`{outcome: "recorded", kind, record_id, replayed, episode_key?, revision?, lifecycle?}`: `episode_key` exactly for the two episode kinds, `revision` exactly for `field_case.revised` and `strategy.recorded`, `lifecycle` exactly for `disposition.recorded` (the lifecycle after the record).

### Error

`{outcome: "refused", code, message, invariant?}`; `invariant` is required for the invariant codes and forbidden for the others. "v1 writers" says whether, and how, a v1 writer receives the code.

| Code | Invariant | Raised when | v1 writers |
|---|---|---|---|
| `SCHEMA_INVALID` |  | request or payload fails this schema (other than the cases below) | yes (existing) |
| `KIND_UNKNOWN` |  | `kind` is not one of the six registered kinds, or is a v2-only kind sent by a v1 writer (no `contract` marker) | yes (existing) |
| `IDEMPOTENCY_CONFLICT` |  | `idempotency_key` seen before with a different body | yes (existing) |
| `EPISODE_REF_DIGEST_MISMATCH` |  | `payload.digest`, `episode_ref` and the recomputed SHA-256 of `reference.body` disagree | yes (existing) |
| `RESULT_INVALID` |  | `result` outside S/F/U/C, or `reliability` merged into `result` | yes (existing) |
| `FREEZE_KEY_EXISTS` | I-3 | a freeze exists for `(case_id, plan_item_id, n)` under a different `idempotency_key`, same `kind`, whatever its digest | yes (existing) |
| `FREEZE_MISSING` | I-1 | `episode.settled` for an episode with no freeze record; at the execution gate: neither RealityTrace records nor a case record exist (section 7) | yes (existing) |
| `FREEZE_INCOMPLETE` | I-1 | `episode.frozen` names a RealityTrace plan, question or threshold that does not exist; at the execution gate and at settlement: some but not all of RealityTrace records, case record and verified anchor exist (NT-33 a, b, c) | yes (existing) |
| `SELF_SETTLEMENT_REFUSED` | I-2 | the settling principal is an executor of the episode, or the delegator (directly or through a chain of delegation links) of one (section 8) | yes (existing) |
| `EXPLORATORY_NOT_SETTLEABLE` |  | `episode.settled` with `result` S or F for an episode whose frozen `kind` is `exploratory` | yes (existing) |
| `WORLD_NOT_PINNED` |  | `episode.frozen` whose `reference.body.world_ref` is not the world currently pinned to the case (S1 finding F8) | no; v1 writer gets `SCHEMA_INVALID`, message prefixed `WORLD_NOT_PINNED: ` |
| `SETTLEMENT_EXISTS` |  | `episode.settled` for an episode that already has a settlement, under a different `idempotency_key` (any result); same key and body replays, same key and other body is `IDEMPOTENCY_CONFLICT` | no; v1 writer gets `IDEMPOTENCY_CONFLICT`, message prefixed `SETTLEMENT_EXISTS: ` |
| `KIND_IMMUTABLE` | I-3 | `episode.frozen` for an existing episode key whose `kind` differs from the recorded freeze's (NT-13: relabelling an exploratory episode as evaluated); takes precedence over `FREEZE_KEY_EXISTS`. Also the execution gate: `episode_ref.kind` differs from the frozen kind | no; v1 writer gets `FREEZE_KEY_EXISTS` (v1 README: a relabel is a second freeze for the key) |
| `REFERENCE_DERIVED_FROM_RESULT` |  | `episode.frozen` whose `reference.body` carries result content: a property from the closed result-field list (`result`, `outcome`, `observation`, `settlement`, `verdict`, `residual`) or a string quoting a RealityTrace result id (`observation:`, `evidence:`, `settlement:`, `residual:` + 26-character id) (NT-17). Replaces `SCHEMA_INVALID` at those pointers | no; v1 writer gets `SCHEMA_INVALID` (closed body) |
| `MODEL_ANCHOR_LATE` |  | `episode.settled` (and `episode.frozen` when the case already holds a run event for the world) when the `model` anchor for the episode's `world_ref` is absent, or its server push time is not before the first run event of the case (NT-18) | no; v1 writer gets `FREEZE_INCOMPLETE`, message prefixed `MODEL_ANCHOR_LATE: ` |
| `AUTHORITY_DENIED` |  | the SEA-Forge authority decision or approval a record rests on exists and is denied or expired, or is required and absent (fail closed): `disposition.recorded` (`approval_ref`), `gate.assessed` with `approval_ref`, human `episode.settled`; at the execution gate, the program-execution decision (NT-34) | unreachable: v1 kinds carry no authority decision |
| `ANCHOR_PUBLISHER_INVALID` |  | the freeze anchor's `pusher.identity` is not the configured case-keeper service identity, i.e. it was pushed with an executor credential (NT-48); checked at settlement and at the execution gate | no; v1 writer gets `FREEZE_INCOMPLETE`, message prefixed `ANCHOR_PUBLISHER_INVALID: ` |
| `STORE_INTEGRITY` |  | the case store failed its integrity check on read (broken record chain, missing `field_case.json` of a Field Case, unreadable ledger); nothing was written; retrying does not help | no; v1 writer gets the SFWP error frame, `error_class: store_integrity` |
| `PAYLOAD_TOO_LARGE` |  | the encoded request exceeds 1048576 bytes; nothing was read or written | no; v1 writer gets the SFWP error frame, `error_class: input_error` |
| `HARD_GATE_NOT_WAIVABLE` |  | `gate.assessed` with `state` `not_applicable` for a hard gate (`allocate_authority`, `integrate_govern`) without `approval_ref` (NT-11, SOP 8.1b) | unreachable: `gate.assessed` is a v2 kind |
| `CASE_SUSPENDED` |  | `episode.frozen` while the case's latest case-level disposition is `suspend` (NT-22); also delegation (section 7). Records that do not start work (`field_case.revised`, `strategy.recorded`, `gate.assessed`, `disposition.recorded`, `episode.settled`) are accepted | no; v1 writer gets `SCHEMA_INVALID`, message prefixed `CASE_SUSPENDED: ` |
| `REVISION_CONFLICT` |  | `field_case.revised.base_revision`, or `strategy.recorded.revision - 1`, is not the current revision (optimistic concurrency); retry after `case.field.get` | unreachable: both are v2 kinds |

**Precedence** (first that applies wins): `PAYLOAD_TOO_LARGE`; `KIND_UNKNOWN`; `SCHEMA_INVALID` and its specific forms (`RESULT_INVALID`, `REFERENCE_DERIVED_FROM_RESULT`); `EPISODE_REF_DIGEST_MISMATCH`; `STORE_INTEGRITY` (any read the following need fails); idempotency (`IDEMPOTENCY_CONFLICT`, replay); `CASE_SUSPENDED`; `WORLD_NOT_PINNED`; `KIND_IMMUTABLE`; `FREEZE_KEY_EXISTS`; `FREEZE_MISSING`; `FREEZE_INCOMPLETE`; `MODEL_ANCHOR_LATE`; `ANCHOR_PUBLISHER_INVALID`; `SETTLEMENT_EXISTS`; `EXPLORATORY_NOT_SETTLEABLE`; `SELF_SETTLEMENT_REFUSED`; `HARD_GATE_NOT_WAIVABLE`; `AUTHORITY_DENIED`; `REVISION_CONFLICT`. The `SCHEMA_INVALID` rule of v1 (specific code when the failing pointer is `/kind`, `/payload/result`, `/payload/reliability`) extends to `/payload/reference/body/<result-field>` (`REFERENCE_DERIVED_FROM_RESULT`).

Scenarios (`fixtures/case-record-append-v2/scenarios/*.json`): ledger state, request, expected answer, for every v2-only code, the v1-writer fallbacks, replay, suspension (which still accepts a field-case revision), lifecycle by disposition, and the S1 refusals (I-1, I-2, I-3). The test validates request and answer against the right dialect's schema, runs the rules that need only the request, and fails if a v2-only code has no scenario.

## 7. Execution gating (`execution-episode-ref.v1.json`)

Closes S1 finding F1(b): I-1 "freeze before move" was enforced only at settlement. The field `episode_ref` is added to the three execution entry points: `item_execute`, `case_advance` and `agent_run.delegate` (wire verb `delegate`). It is `{case_id, plan_item_id, n, kind: evaluated | exploratory}` and names the episode by key; the case keeper resolves the frozen `episode_ref` from the ledger and checks `kind` against the freeze, never trusting the caller's tag. Request shapes are the existing ones (`$defs` `ItemExecuteRequest`, `CaseAdvanceRequest`, `DelegateRequest`) plus the new fields; `delegate` also gets `assignment_id` (section 11) because it has no `case_id` on the wire (the case is `episode_ref.case_id`). The response of an accepted request is the verb's existing response, unchanged; the case keeper records the resolved `episode_ref` on the run record.

A freeze is **complete** when all three exist: the RealityTrace plan, question and threshold; the `episode.frozen` case record; the external anchor, published by the case-keeper identity (E "Freeze order", RD-12).

| Rule | Condition (template `fde.field_case@1`) | Refusal | NT |
|---|---|---|---|
| R1 | request has no `episode_ref` and would execute | `FREEZE_MISSING` | NT-32 |
| R2 | `episode_ref` names an episode with none of the three parts | `FREEZE_MISSING` | NT-32 |
| R3 | some but not all three parts exist | `FREEZE_INCOMPLETE` | NT-33 a, b, c |
| R4 | all three exist, the anchor was not pushed by the case-keeper identity | `ANCHOR_PUBLISHER_INVALID` | NT-48 |
| R5 | `episode_ref.kind` differs from the frozen kind | `KIND_IMMUTABLE` | NT-13 |
| R6 | the latest case-level disposition is `suspend` | `CASE_SUSPENDED` | NT-22 |
| R7 | the program-execution decision (section 10) is denied, expired or absent | `AUTHORITY_DENIED` | NT-34 |
| R8 | complete freeze, kind matches, decision allowed | accepted | AT-04 |
| R9 | the case template is not `fde.field_case@1` | unaffected; `episode_ref` is accepted and ignored | |
| R10 | `delegate`: `assignment_id` is not `assign:<case_id>:<plan_item_id>:<n>:<k>` of this episode, or `k` is already issued for it (a resume reuses its id; a new delegation takes `k+1`) | `ASSIGNMENT_ID_INVALID` | AT-17 |
| R11 | `case_advance` would reach Completed with no case-level `close-*` disposition | `CLOSE_DISPOSITION_REQUIRED` | NT-23 |

Order of evaluation: R9, R6, R11, R1/R2, R3, R4, R5, request consistency (R10, item/case match), R7, R10 `k`. A gate refusal happens before any run, spend or record: the Error carries `no_side_effect: true`. Request consistency (semantic): for `item_execute` `case_id` and `item_id` equal the `episode_ref`'s; for `case_advance` `case_id` equals it; for `delegate` `assignment_id` is required with `episode_ref`. Violations are `SCHEMA_INVALID`, or `ASSIGNMENT_ID_INVALID` for a mismatching id. `case_advance` with an `episode_ref` executes only that episode's plan item; a `case_advance` that executes nothing (a state transition only) needs no `episode_ref` (x-decision).

Error table (`$defs/Error`, `{outcome: "refused", code, message, invariant?, no_side_effect: true}`): `FREEZE_MISSING` (I-1), `FREEZE_INCOMPLETE` (I-1), `KIND_IMMUTABLE` (I-3), `SCHEMA_INVALID`, `CASE_SUSPENDED`, `AUTHORITY_DENIED`, `ANCHOR_PUBLISHER_INVALID`, `MODEL_ANCHOR_LATE`, `CLOSE_DISPOSITION_REQUIRED`, `ASSIGNMENT_ID_INVALID`. Their conditions are the rows above and the section 6 table, except `CLOSE_DISPOSITION_REQUIRED` (defined here) and `ASSIGNMENT_ID_INVALID` (section 11). `MODEL_ANCHOR_LATE` at the gate: the model anchor is absent or not before the first run event. v1 writers: these verbs are not append verbs; a client that does not send `episode_ref` is "unaffected" only on non-Field-Case cases.

SFWP frames: `fixtures/execution-episode-ref/frames/*.json` give, for each of the three verbs, the request with and without the field on a Field Case (refused `FREEZE_MISSING`; accepted; refused `FREEZE_INCOMPLETE`) and without it on another template (accepted; byte-identical to today's frame). They are `status: expected`; the goldens are captured by WP-21. Scenarios (`scenarios/*.json`, 42) give the freeze state and case state for each rule and verb; the test runs the rule table above over them.

## 8. Delegation link (`delegation-link.v1.json`)

Closes the I-2 residual "delegate then settle" (S1 finding F7: no record linked an agent run's actor to its delegator, so the delegator of an executor could settle). The case keeper writes one `Link` per accepted `agent_run.delegate` of a Field Case episode (never a client): `{link_id, case_id, episode_ref, assignment_id, delegator, delegate, issued_at, expires_at}`.

| Field | Meaning |
|---|---|
| `delegator` | the authenticated principal the run was delegated on behalf of (request actor; `on_behalf_of` when sent through a gateway) |
| `delegate` | the actor identity the run executes as |
| `episode_ref`, `assignment_id` | the frozen episode and the assignment (section 11); `assignment_id`'s case, step and `n` equal the episode's |
| `issued_at`, `expires_at` | UTC; no new run starts under the link after expiry; the link is never deleted, and settlement checks use expired links |

**Rule (I-2).** Settlement of an episode is refused with `SELF_SETTLEMENT_REFUSED` (invariant `I-2`; the exact code `case-record-append.v1` uses) when the settling principal is an executor of the episode (a `delegate` of any of its links), or is the `delegator` of any executor, directly or through a chain of links (every principal in the closure of delegators of executors). A human root settles only by A3 (section 10), still subject to this rule. A sibling verifier under the same root is not refused by the link rule; it needs `independence.verifier_context` `fresh` (E section 3), which `episode.settled` already carries.

`$defs`: `Link`, `Settlement` (the check input), `Error` (`SELF_SETTLEMENT_REFUSED`, `DELEGATION_LINK_INVALID`; invariant `I-2`). `DELEGATION_LINK_INVALID` (home here): `expires_at` not after `issued_at`, `delegator` equal to `delegate`, the assignment id disagreeing with the episode or `case_id`, or a link of another episode in a settlement's link set. `SELF_SETTLEMENT_REFUSED` is defined in section 6 and listed here as also raised. v1 writers: the rule applies to every settlement; v1 writers receive the v1 code as before.

## 9. Continuation and read verbs (`continuation.v1.json`)

`case.continuation` is computed on every read from the ledger plus RealityTrace and Cognate references, never stored (E section 3). `Continuation` has one property per item of SOP 10.1 (`x-sop-item` 1 to 9, `x-sop-segment` the SOP text) and `computed_from`:

| SOP 10.1 item | Property | Content |
|---|---|---|
| case ID, owner, purpose, current decision | `case` | `case_id`, `lifecycle`, and `owner`, `purpose`, `current_decision` as `FieldValue` (from the Field Case document) |
| pinned world and checkpoint | `pinned_world` | `world_ref`, `checkpoint` |
| gate evidence | `gate_evidence` | all nine gates in SOP order, each once: `state`, `evidence_refs`, `record_id` (unassessed with null record when never assessed) |
| frozen or pending episodes | `episodes` | per episode `episode_key`, `episode_ref`, `kind`, `state` (`pending`, `frozen`, `settled`), `result`, `settlement_ref` |
| exact authority/approval state | `authority` | pending approvals, decisions with assurance, `a3_channel` |
| disputed claims | `disputed_claims` | `ref`, `why` |
| latest observations with RealityTrace references, unknown and contradictory evidence exposed | `latest_observations` | `observations` (RealityTrace ref, `standing`, `basis`), `unknown_evidence`, `contradictory_evidence` |
| active assignments and trajectories, artifact locations, verified commands, remaining budgets | `work` | assignments, artifacts, verified commands, remaining budget per currency (number or `"unknown"`, never 0 for unmeasured) |
| next affordable action or blocker, who can authorize it, what would settle it | `next` | `kind` (`action`, `blocker`), `description`, `authorizer`, `would_settle`, `dimension` for a blocker (one of the seven SOP 7.2 dimensions) |

`computed_from`: `ledger_head`, `realitytrace_snapshot` (null when RealityTrace was unreachable: observations are then `unknown`, never silently absent), `cognate_ref`, `computed_at`. Semantic rules (`CONTINUATION_INVALID`, test-local): nine gates in order, each once; a `met` gate has evidence; currencies unique; a closed case has no active assignment.

### Read verbs

| Logical verb | Wire request | Response | Errors |
|---|---|---|---|
| `case.field.get` | `{verb: "case_field_get", case_id, request_id?, actor?}` | `{case_id, revision, document}` (`document` is `field-case.v1`, `document.revision` equals `revision`) | `NOT_A_FIELD_CASE`, `STORE_INTEGRITY` |
| `case.continuation` | `{verb: "case_continuation", case_id, request_id?, actor?}` | `{case_id, continuation}` | `NOT_A_FIELD_CASE`, `STORE_INTEGRITY` |

Both are reads: no mutation, no `request_id` requirement. `NOT_A_FIELD_CASE` (home here): the case's template is not `fde.field_case@1`. An unknown `case_id` is the existing SFWP error frame (`error_class`), not defined here. `STORE_INTEGRITY` is defined in section 6. Frames: `fixtures/continuation/frames/*.json` (expected, not goldens: WP-21 captures them).

## 10. Reserved operations and the program-execution decision (`reserved-operation.v1.json`)

### Operation, class, evidence (E section 8, RD-06, RD-21)

`fixtures/reserved-operation/valid/operation-table.json` is the table instance; the schema rejects a reserved row below P2, P3 rows accepting A3-T, and non-reserved rows with an A3 surface.

| Operation | Class | Reserved | Evidence (alternatives; inner lists required together) | A3-T |
|---|---|---|---|---|
| `session_work` (read, navigate, draft, note, exploratory observation) | P0/P1 (written `P1`) | no | `session` | n/a |
| `freeze_episode` | P1 | no | `authenticated_principal` + `preview_check_code` | n/a |
| `assign_within_grant` | P1 | no | `authenticated_principal` + `preview_check_code` | n/a |
| `program_execution` | P1 | no | `authenticated_principal` + `authority_decision_bound_to_digest` | n/a |
| `settle_episode` | P2 | yes | `authenticated_principal` + `verifier_not_executor`; or `a3_proof` + `verifier_not_executor` (a human settles) | policy |
| `set_case_disposition` | P2 | yes | `a3_proof` + `sea_forge_approval` | policy |
| `set_hard_gate_not_applicable` | P2 | yes | `a3_proof` + `sea_forge_approval` | policy |
| `promote_world` | P3 | yes | `a3_proof` + `online_decision_bound_to_digest_and_expiry` | never |
| `bootstrap_controller` | P3 | yes | same | never |
| `recover_controller` | P3 | yes | same | never |

### A3 proof minimum

The pending operation (`PendingOperation`) is issued by SEA-Forge: `{pending_id, operation, class, case_id, subject_ref, canonical_operation, operation_digest, issued_at, expires_at, evidence_required}`; `operation_digest` = SHA-256(RFC 8785(`canonical_operation`)). The A3 surface (the paired phone, class A3-UV) receives `canonical_operation` from SEA-Forge by `pending_id`, never from a page. The proof (`A3Proof`) is exactly the minimum of E section 13 plus the key that binds it: `pending_id`, `challenge` (SEA-Forge-issued, single use), `operation_digest`, `authenticator_assertion` (`credential_id`, `authenticator_data`, `client_data_json`, `signature`), `confirmation_surface_id` (`surface:<token>`, resolved by SEA-Forge to A3-UV or A3-T; an unknown id is not an A3 surface), `expires_at`. SEA-Forge, not Horizon, validates it. The decision (`Decision`) is `{outcome: "confirmed", decision_id, pending_id, operation, operation_digest, assurance, decided_at, expires_at}`, `assurance` `session | A3-UV | A3-T`.

Staged enforcement (RD-21): the check is off until WP-31; before it, decisions are recorded with assurance `session` and every row depending on a human-reserved decision is PARTIAL until S3. The Chromium virtual authenticator is valid for automated regression only (OD-1, OD-5).

### Errors

| Code | Raised when | NT |
|---|---|---|
| `RESERVED_DECISION_REFUSED` | enforcement is on and a reserved operation arrives with no A3 proof: an agent session, or the paired-browser session without the agent header, submits a disposition or promotion approval | NT-12 |
| `A3_SURFACE_REQUIRED` | a proof is present but its `confirmation_surface_id` is not an accepted A3 surface: the desk page even with a valid passkey assertion; A3-T for a P3 operation; A3-T for a P2 operation unless policy accepts it | NT-24 |
| `A3_CHANNEL_UNAVAILABLE` | no A3 channel is configured or reachable; carries `how_to_restore` | |
| `RESERVED_EXPIRED` | the pending operation or the proof is past `expires_at` | |
| `RESERVED_DIGEST_MISMATCH` | the proof's `pending_id` or `operation_digest` differs from the pending operation's, or a decision record's digest does not bind what it names | |
| `A3_PROOF_INVALID` | the assertion does not verify against the enrolled credential and challenge | |
| `A3_PROOF_REPLAYED` | the `challenge` was already used | |
| `AUTHORITY_DENIED` | SEA-Forge's own approval or decision for the operation is denied (defined in section 6) | NT-34 |

Order: enforcement off; `A3_CHANNEL_UNAVAILABLE`; `RESERVED_DECISION_REFUSED`; `RESERVED_EXPIRED`; `RESERVED_DIGEST_MISMATCH`; `A3_SURFACE_REQUIRED`; `A3_PROOF_REPLAYED`; `A3_PROOF_INVALID`; `AUTHORITY_DENIED`. The scenarios under `fixtures/reserved-operation/scenarios` run it. v1 writers: not append codes.

### Program-execution decision (`ProgramExecutionDecision`)

S1 showed SEA-Forge records no authority decision for program execution, so RealityTrace could not resolve `authority.decision_id` of execution events. A `program_execution` operation in a Field Case episode requires this record to exist (rule R7):

`{decision_id, operation_id, case_id, episode_ref, patch_sha256, command: {argv[]}, binds_digest, decided_by: {principal, role}, outcome: allow | deny, assurance, decided_at, expires_at}`

- **Who decides**: SEA-Forge's authority gate, through the existing approval path, on a principal that is the frozen reference's `decision_maker` or holds the approver role, and is never the executor or one of its delegators (I-2 chain, section 8). Assurance is `session` until WP-31 (x-decision).
- **What it binds**: `binds_digest` = SHA-256(RFC 8785(`{episode_ref, patch_sha256, command}`)): the full `episode_ref`, the SHA-256 of the patch the program applies (of the empty string when none), and the command. An execution that differs in any of the three has no decision (`AUTHORITY_DENIED`; a decision record that does not bind its own fields is `RESERVED_DIGEST_MISMATCH`).
- **Resolution**: `decision_id` is the value emitters send as `authority.decision_id` (E section 5); RealityTrace verifies it against SEA-Forge, scoped by `operation_id`. Fixtures: allow, deny, no patch.

## 11. Assignment (`assignment.v1.json`)

`assignment_id` = `assign:<caseId>:<stepId>:<n>:<k>` (E section 2, RD-13, RD-23). Regex `^assign:([^:@\s]+):([^:@\s]+):([1-9][0-9]*):([1-9][0-9]*)$`. Parse: split on `:` into exactly five parts; `caseId` is the episode's `case_id`, `stepId` its `plan_item_id`, `n` the episode counter, `k` the delegation instance within episode `n`, both from 1. Horizon proposes the id at Hand off from the case keeper's counters; SEA-Forge validates it (R10): `caseId`, `stepId` and `n` equal the `episode_ref`'s, and `k` is the next unused instance of that episode. A resume keeps the id; a new delegation of the same episode takes `k+1`; a changed episode takes a new `n` and `k = 1`. The old four-part form is refused (pattern).

`Assignment`: `{assignment_id, case_id, step_id, n, k, episode_ref, brief, budget, issued_at}` (the fields repeat the id and must agree with it and with `episode_ref`, semantically).

| Field | Rule |
|---|---|
| `brief` | `{instruction, adoptable_problem, domain_meaning, context_refs[]}`; each context ref `{ref, trust: "untrusted_reference"}` (AT-17: retrieved context is labelled untrusted) |
| `budget` | exactly five entries, one per currency of SOP 5.1 (`time`, `attention`, `money`, `risk`, `recovery`), each `{currency, limit: number >= 0 \| "unknown", unit}`; an unmeasured currency is `"unknown"`, never 0 (SOP 5.3) |
| `Event` | `{assignment_id, episode_ref, event, at, spend, reason?}`; `event` is `issued`, `started`, `interrupted`, `resumed`, `completed`, `failed`, `cancelled`; `spend` has exactly five per-currency entries (number or `"unknown"`); `reason` is required for `interrupted`, `failed`, `cancelled` |

Lifecycle: `issued` -> `started` -> (`interrupted` <-> `resumed`)* -> `completed` | `failed` | `cancelled` (`interrupted` may be cancelled). One id and one `episode_ref` for the whole sequence (AT-17: the episode reference is unchanged across resume).

| Code | Raised when |
|---|---|
| `ASSIGNMENT_ID_INVALID` | the id does not parse, its parts disagree with the episode or with the assignment's fields, `k` is already used, or an event carries another id than its sequence |
| `ASSIGNMENT_EPISODE_CHANGED` | an event of a sequence names another `episode_ref` |
| `ASSIGNMENT_TRANSITION_INVALID` | the event is not allowed after the previous one (including any event after a terminal one) |
| `ASSIGNMENT_BUDGET_INVALID` | a currency appears twice in `budget` or `spend` |

## 12. Independent observation (normative)

S1 finding: executor-only evidence cannot confirm. RealityTrace caps an episode whose every occurrence comes from the executor's emitter at class `insufficient`, basis `executor_only_evidence` (E section 5; G NT-20). For execution steps, these SEA-Forge-side records count as an **independent emitter**, because SEA-Forge writes them, not the executor:

1. the program-execution decision (section 10), emitted by `sea-forge-authority` with `source.record_id` = `decision_id`;
2. the case-ledger event for the execution (the run record the case keeper writes when it accepts and completes the gated request), emitted by `sea-forge-ledger`;
3. the verifier's own observation (WP-17/WP-43), which is not an execution step and is out of this contract.

The executor's own emitter (Cognate, the agent's tool log) never counts toward independence. Both SEA-Forge events carry the episode's `episode_ref`, the `assignment_id`, `authority.decision_id` equal to the decision, `executor_provenance`, and `kind`; they are resolvable against the owning system's record (E section 5). `confirmed` needs at least one occurrence from such an emitter. The expected event shape is `fixtures/independent-observation/` (three events: authority decision, ledger run, executor run, in the CEP `godspeed.interaction_event.v1` shape; `semantic_ref` and `declaration_key` are copied from the CEP fixture and are illustrative). This section and the fixtures are documentation; no schema is added here and WP-21/WP-17 emit the events.

## 13. Decisions where E, G and the SOP do not determine a field or rule

Each is the minimal option and is marked `x-decision` for review; none is inside a schema.

| # | Where | `x-decision` |
|---|---|---|
| D1 | field-case | `x-decision: "WP-20: the model does not declare the SOP 4.2 fields (map row 4.2 is unknown, S7), so the schema is generated from the SOP 4.2 table and bound to it by hash and by label; the check fails if the map row changes"` |
| D2 | field-case | `x-decision: "WP-20: FieldValue is {status, source_ref, value?}; source_ref is {kind, ref} with kind none exactly for unknown; value is any non-null JSON (E fixes no per-field type)"` |
| D3 | field-case | `x-decision: "WP-20: 45 fields (the SOP table has no count; the map says about 60); keys are the SOP labels in snake case; revision starts at 1 at case open; the document names the SOP version and sha256 it follows"` |
| D4 | append v2 | `x-decision: "WP-20: v2 writers mark requests with contract=fde.case-record-append@2 (required for the four new kinds); a request without it is a v1 writer and gets the v1 dialect with the fallback codes of section 6"` |
| D5 | append v2 | `x-decision: "WP-20: KIND_IMMUTABLE takes precedence over FREEZE_KEY_EXISTS for v2 writers when the kind differs; v1 README decision 8 said it was unused. v1 writers keep FREEZE_KEY_EXISTS"` |
| D6 | append v2 | `x-decision: "WP-20: REFERENCE_DERIVED_FROM_RESULT is a result-field property in reference.body (list: result, outcome, observation, settlement, verdict, residual) or a string quoting a RealityTrace result id with a 26-character id; NT-17 names no rule"` |
| D7 | append v2 | `x-decision: "WP-20: WORLD_NOT_PINNED compares with the world currently pinned to the case (F8); MODEL_ANCHOR_LATE and ANCHOR_PUBLISHER_INVALID are raised at settlement and at the execution gate, because episode.frozen carries no anchor (freeze order step 3 follows the record)"` |
| D8 | append v2 | `x-decision: "WP-20: CASE_SUSPENDED refuses episode.frozen and the execution verbs only (E: new freezes and delegations); revision, strategy, gate, disposition and settlement records are still accepted so a suspended case can be resumed or closed"` |
| D9 | append v2 | `x-decision: "WP-20: PAYLOAD_TOO_LARGE limit is 1048576 bytes of encoded request (the 1 MiB frame limit the server already uses in mcp.rs); STORE_INTEGRITY and PAYLOAD_TOO_LARGE reach v1 writers as the existing SFWP error frame"` |
| D10 | append v2 | `x-decision: "WP-20: new code REVISION_CONFLICT (optimistic concurrency for field_case.revised base_revision and strategy.recorded revision); no G row needs it"` |
| D11 | append v2 | `x-decision: "WP-20: field_case.revised carries base_revision >= 1, a list of path/FieldValue changes and a rationale; strategy.recorded carries the five SOP 4.4 questions plus revision, supersedes, revisit_reason"` |
| D12 | append v2 | `x-decision: "WP-20: gate.assessed has no assessor field (the authenticated principal is recorded, as for settlement); hard gates are allocate_authority and integrate_govern (the model leaves hard_governance_condition to the caller; G names only allocate_authority); approval_ref carries the A3-backed approval"` |
| D13 | append v2 | `x-decision: "WP-20: disposition.recorded requires approval_ref at both levels (E: behind the existing approval path); an episode-level disposition of any kind never changes the lifecycle; the response carries the resulting lifecycle"` |
| D14 | kinds | `x-decision: "WP-20: the registry is data (fixtures/case-record-kinds/valid/registry.json) validated by a closed schema with exactly six kinds; immutability classes frozen_forever, single_settlement, revision_chain, append_only_history; REGISTRY_INCOMPLETE and REGISTRY_NATURAL_KEY_INVALID are test-local"` |
| D15 | execution | `x-decision: "WP-20: episode_ref is the key {case_id, plan_item_id, n, kind}, not the digest form; both evaluated and exploratory episodes need a complete freeze (E says evaluated; exploratory frozen references exist in S1 and an exploratory run must also be labelled at the time)"` |
| D16 | execution | `x-decision: "WP-20: FREEZE_MISSING means none of RealityTrace records, case record, anchor exist; FREEZE_INCOMPLETE means some but not all (NT-33 a, b, c); ANCHOR_PUBLISHER_INVALID when all exist but the pusher is not the case keeper"` |
| D17 | execution | `x-decision: "WP-20: a Field Case case_advance with episode_ref executes only that episode's plan item; an advance that executes nothing (state transition) needs no episode_ref; non-Field-Case cases accept and ignore episode_ref (unaffected)"` |
| D18 | execution | `x-decision: "WP-20: delegate carries assignment_id beside episode_ref (the wire verb has no case_id); refusals before any side effect carry no_side_effect: true, the field the existing SFWP refusals use; the accepted response is unchanged"` |
| D19 | execution | `x-decision: "WP-20: a missing or denied program-execution decision is AUTHORITY_DENIED (fail closed), not a separate code"` |
| D20 | delegation | `x-decision: "WP-20: the link set is closed under delegators (transitive); expired links still count at settlement; a sibling verifier is not refused by the link rule but needs verifier_context fresh; the link is written by the case keeper in cases/<id>/delegation_links.jsonl (path is WP-21's)"` |
| D21 | continuation | `x-decision: "WP-20: one property per SOP 10.1 item (nine), plus computed_from; null RealityTrace snapshot means observations are unknown, never absent; next.dimension uses the seven SOP 7.2 ids; CONTINUATION_INVALID is test-local"` |
| D22 | read verbs | `x-decision: "WP-20: wire verbs case_field_get and case_continuation (existing read verbs are case_get_overview, case_get_horizon); new code NOT_A_FIELD_CASE; unknown case ids use the existing SFWP error frame"` |
| D23 | reserved | `x-decision: "WP-20: classes: set_case_disposition and set_hard_gate_not_applicable are P2 (E says P2/P3); program_execution is P1 with an authority decision bound to the digest; A3-T is policy-accepted for P2 and never for P3; the proof is E's five fields plus pending_id; the surface class is resolved by SEA-Forge from confirmation_surface_id"` |
| D24 | reserved | `x-decision: "WP-20: codes RESERVED_DECISION_REFUSED (NT-12) and A3_SURFACE_REQUIRED (NT-24) are G's; added A3_CHANNEL_UNAVAILABLE (with how_to_restore), RESERVED_EXPIRED, RESERVED_DIGEST_MISMATCH, A3_PROOF_INVALID, A3_PROOF_REPLAYED for the four WP-31 refusals and proof checks; before WP-31 decisions are recorded with assurance session"` |
| D25 | reserved | `x-decision: "WP-20: the program-execution decision is decided by SEA-Forge's authority gate on the frozen reference's decision_maker or an approver principal that is not the executor or its delegator; binds_digest covers {episode_ref, patch_sha256, command}; no patch is the SHA-256 of the empty string"` |
| D26 | assignment | `x-decision: "WP-20: brief is {instruction, adoptable_problem, domain_meaning, context_refs[]} (SOP 7.4 names the first problem and meaning; B's ten-part list is not in the frozen SOP); budget is exactly five entries with a unit string; unit values are not constrained"` |
| D27 | assignment | `x-decision: "WP-20: the lifecycle events and their state machine, and the four ASSIGNMENT_* codes, are defined here; E names only interruption and resume"` |
| D28 | oracle codes | `x-decision: "WP-20: exactly one error table means exactly one home table defining the condition; another contract's Error enum may list the code (also_raised_by) by reference; v1 and v2 of the append contract are one lineage"` |
| D29 | independent observation | `x-decision: "WP-20: the SEA-Forge records that count as independent for execution steps are the program-execution decision and the ledger run record; the verifier's own observation is out of scope; the fixture's semantic_ref and declaration_key are copied from the CEP fixture"` |
| D30 | all | `x-decision: "WP-20: fixtures for case-record-append.v2 live in fixtures/case-record-append-v2 (the v1 directory is untouched); frames are status expected, not goldens"` |

