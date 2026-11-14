# FDE contracts (WP-11)

Reviewed-before-code contracts for the Field Case path. JSON Schema draft 2020-12, every object closed (`additionalProperties: false`). These files are hand-authored; unlike `contracts/schema/`, no generator writes them. A change to a schema file needs a new line in `DIGESTS` in the same commit (`scripts/check-fde-contract-digests.sh --update`); consumers vendor by digest.

| Contract | File | Fixtures (valid / invalid, schema + semantic) | Test |
|---|---|---|---|
| Frozen reference | `frozen-reference.v1.json` | 3 / 16 + 1 | `crates/sea-forge-realitytrace/tests/fde_contracts.rs` |
| Anchor | `anchor.v1.json` | 3 / 8 + 2 | same |
| `case.record.append` | `case-record-append.v1.json` | 12 / 18 + 1, 11 scenarios | same |

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
