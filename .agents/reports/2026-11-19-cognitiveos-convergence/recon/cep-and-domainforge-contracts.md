# Recon: CEP profiles and DomainForge CEP integration

Scope: `/home/sprime01/projects/cep` (main 5111ec3), `/home/sprime01/projects/domainforge` (main 35c4ed1).
Status: FINAL — read-only recon, no repository modified.

## 0. Repo baselines

- CEP: `/home/sprime01/projects/cep` @ `5111ec3` — `feat(profiles): godspeed.context_bundle, partial retrieval is never complete (#3)`. Worktree clean.
- DomainForge: `/home/sprime01/projects/domainforge` @ `35c4ed1` — `docs(graft): refresh graft skill wording (#183)`. Worktree clean.

## 1. CEP profiles

Profiles live in `schemas/profiles/godspeed/` (generated) and are specified by
`spec/profiles/GODSPEED-PROFILES-v1.md`. Generator: `tools/generate_godspeed_profiles.py`.

Precedence note: the spec doc says "Six CEP-0008 envelope profiles"
(`spec/profiles/GODSPEED-PROFILES-v1.md:5`) but its own table (§3) lists **seven**
profiles, and the directory holds **seven** profile schemas plus a non-profile
record schema (`world-transition.v1.schema.json`). The "Six" count is stale.

Shared declaration: every profile requires `extensions["cep.profile"]` with
`profile_id` + `profile_version` (`tools/generate_godspeed_profiles.py:99-107`),
`profile_version` const `1.0.0` (`tools/generate_godspeed_profiles.py:24`),
`profile_id` const `godspeed.<envelope_kind>` (`tools/generate_godspeed_profiles.py:152`).

### 1.1 The seven profiles

All paths under `/home/sprime01/projects/cep/`.

| # | Schema file | `profile_id` | `profile_version` | Required fields (envelope level) | Purpose |
|---|---|---|---|---|---|
| 1 | `schemas/profiles/godspeed/semantic_snapshot.v1.schema.json` | `godspeed.semantic_snapshot` | `1.0.0` | `scope`, `extensions`, `provenance` + `anyOf` branch | Bounded state of a semantic world, pinned by `world_ref` (CEP-0008 §9.11) |
| 2 | `schemas/profiles/godspeed/context_bundle.v1.schema.json` | `godspeed.context_bundle` | `1.0.0` | `scope`, `extensions`, `provenance`, `completeness_status` | Context retrieved for a work request; partial is never complete (§9.2, §34) |
| 3 | `schemas/profiles/godspeed/authority_request.v1.schema.json` | `godspeed.authority_request` | `1.0.0` | `scope`, `extensions`, `transformations`, `authority`, `provenance`, `references` | A request for authority to perform one transformation |
| 4 | `schemas/profiles/godspeed/authority_decision.v1.schema.json` | `godspeed.authority_decision` | `1.0.0` | `scope`, `extensions`, `transformations`, `authority`, `provenance`, + `lineage_refs` (minItems 1) | The decision on one request; never a boolean |
| 5 | `schemas/profiles/godspeed/execution_trace.v1.schema.json` | `godspeed.execution_trace` | `1.0.0` | `scope`, `extensions`, `transformations`, `provenance`, + `lineage_refs` (minItems 1) | Observation of what was done (not settlement) |
| 6 | `schemas/profiles/godspeed/evidence_packet.v1.schema.json` | `godspeed.evidence_packet` | `1.0.0` | `scope`, `extensions`, `evidence`, `questions`, `provenance`, + `lineage_refs` (minItems 1) | Evidence relative to a question (not raw data, not settlement) |
| 7 | `schemas/profiles/godspeed/settlement_packet.v1.schema.json` | `godspeed.settlement_packet` | `1.0.0` | `scope`, `extensions`, `settlements`, `evidence`, `questions`, `provenance`, + `lineage_refs` (minItems 1) | The only kind that settles, and only from evidence |

Per-profile extension requirements (`extensions[...]`):

| profile_id | required extension namespace + fields |
|---|---|
| `godspeed.semantic_snapshot` | `domainforge.identity` → `domain_model_identity`, `semantic_closure_hash` (valid branch); `domainforge` → `model_validation_status: "invalid"` (invalid branch) |
| `godspeed.context_bundle` | `godspeed.context_bundle` → `work_request_ref`, `retrieval_completeness` (`complete`/`partial`/`none`) |
| `godspeed.authority_request` | `godspeed.authority_request` → `operation_id`, `correlation_id` |
| `godspeed.authority_decision` | `godspeed.authority_decision` → `decision`, `operation_id`, `request_ref` (+ `policy_basis` for `constrained_allow`/`boundary`/`degraded`; + `compensating_controls` for `degraded`) |
| `godspeed.execution_trace` | `godspeed.execution_trace` → `operation_id`, `authority_decision_ref`, `correlation_id` |
| `godspeed.evidence_packet` | `godspeed.evidence_packet` → `operation_id` |
| `godspeed.settlement_packet` | `godspeed.settlement_packet` → `operation_id` |

Item-level requirements: `authority[]` items require `authority_id` +
`subject_actor_ref` (`tools/generate_godspeed_profiles.py:190-193`);
`transformations[]` items require `transformation_id` + `transformation_type`
(same file `:369-372`, `:392-394`).

`authority_decision` is the only profile with an `anyOf` on dispositions;
`decision` ∈ `allow`, `deny`, `escalate`, `unknown`, `constrained_allow`,
`boundary`, `degraded` (`tools/generate_godspeed_profiles.py:174-176`), and "is
never a boolean" (`spec/profiles/GODSPEED-PROFILES-v1.md:52`).

### 1.2 Eighth schema in the same directory is NOT a profile

`schemas/profiles/godspeed/world-transition.v1.schema.json` is a standalone record
schema (no `allOf` over the base envelope, no `cep.profile` declaration). Required
fields: `source_world_ref`, `target_world_ref`, `transition_kind`, `reason`,
`compatibility`, `semantic_closure_equal`
(`spec/profiles/GODSPEED-PROFILES-v1.md:72`). The envelope kind that would carry
it (`semantic_diff`) "is profiled later" — i.e. not yet a profile.


## 2. Which profile carries what

All paths under `/home/sprime01/projects/cep/`.

**Semantic world reference / revision** — `scope.world_ref`, required by *every*
profile via `scope_requirements(pinned=True)`
(`tools/generate_godspeed_profiles.py:78-96`, requirement inserted at `:95`),
with the sole exception of the `semantic_snapshot` invalid-world branch, which
instead requires the *absence* of a pinned world
(`tools/generate_godspeed_profiles.py:257-277`).

- Field name: `world_ref`; grammar `world:<name>@sha256:<64 lowercase hex>`
  (`tools/generate_godspeed_profiles.py:28-29`).
- `scope.world_alias` (`world:<name>`) is the mutable selector and
  `scope.world_label` is presentation; both "MUST NOT be the only world reference
  on a durable or consequential record" (`spec/profiles/GODSPEED-PROFILES-v1.md:33`).
- The *revision* is the digest. `godspeed.semantic_snapshot` is the only profile
  that additionally requires `integrity` and `extensions["domainforge.identity"]`
  carrying the eleven-field `domain_model_identity` + `semantic_closure_hash`
  (`spec/profiles/GODSPEED-PROFILES-v1.md:42`); the eleven fields are enumerated
  at `tools/generate_godspeed_profiles.py:35-46` (9 required + 2 nullable).

**Authority request** — profile `godspeed.authority_request`; extension fields
`operation_id`, `correlation_id`; `authority` items require `subject_actor_ref`
(`spec/profiles/GODSPEED-PROFILES-v1.md:44`).

**Authority decision** — profile `godspeed.authority_decision`; extension fields
`decision`, `operation_id`, `request_ref`, plus non-empty `lineage_refs` naming the
request (`spec/profiles/GODSPEED-PROFILES-v1.md:45`).

**Evidence references** — two distinct mechanisms, in two different sections:
- `references` (envelope §13.1 reference records): required by
  `godspeed.authority_request`, and by `godspeed.context_bundle` in its
  `complete`/`partial` branches (`tools/generate_godspeed_profiles.py:330`).
- `evidence` (the envelope's evidence section): required by
  `godspeed.evidence_packet` and `godspeed.settlement_packet`
  (`spec/profiles/GODSPEED-PROFILES-v1.md:47-48`).
- Non-empty `lineage_refs`: required by `authority_decision`, `execution_trace`,
  `evidence_packet`, `settlement_packet` (`lineage_min=1` at
  `tools/generate_godspeed_profiles.py:193`, `:384`, `:399`, `:406`).

**Settlement receipt** — the `settlements` section, required only by
`godspeed.settlement_packet`, which also requires `evidence` and `questions`
(`spec/profiles/GODSPEED-PROFILES-v1.md:48`). Quoted distinction: "Only a
`settlement_packet` settles, and only from evidence."
(`spec/profiles/GODSPEED-PROFILES-v1.md:59`).

## 3. Does CEP define an RLM / recursion / agent-execution contract? No.

Searches run from `/home/sprime01/projects/cep` (repo root):

```
$ rg -n -i 'rlm' .
(no output — 0 matches)

$ rg -n -i 'recursi' .
docs/subsystems/schema-and-validation.md:79  ...Recursively validates nested definitions...
docs/subsystems/schema-and-validation.md:85  ...Object recursion: Enforces all `required` keys...
docs/explanation/content-based-detection-rationale.md:75  ...Deterministic code breaks the recursion.
contracts/typescript/tests/emitted.test.ts:21  rmSync(dir, { recursive: true, force: true });
contracts/typescript/src/index.ts:93  ...recursively sorted keys, no whitespace.
```

Every `recursi*` hit is JSON-schema traversal, a test cleanup flag, or prose about
infinite regress in the anti-collapse rationale — none is a recursive-agent
contract. `spec/` and `src/` contain no RLM symbol:

```
$ rg -n -i 'rlm' spec/ src/      -> (no output)
$ rg -n -i 'agent[-_](execution|loop|run)|self[-_]reference|inner[-_ ]monologue' .
spec/CEP-0009-benchmark-model.md:1000  - agent_runtime
benchmarks/reality-gap/cases/rg-0002/observed.jsonl:2  ...src/app/agent_runner.py...
```

`agent_runtime` is a *System Under Evaluation* category in the benchmark model
(`spec/CEP-0009-benchmark-model.md:1025`) — an evaluation subject, not an execution
contract.

The two agent-adjacent artifacts CEP ships are consumer-side, not contract-defining
for agents:
- `schemas/vendored/sea.agent.event.v1.json` — a **vendored copy** of the SEA Forge
  v1 wire event schema; `$id` `https://godspeed.ai/contracts/sea.agent.event.v1.json`,
  `schema_version` const `"v1"` (`schemas/vendored/sea.agent.event.v1.json:3-19`).
  CEP is the consumer: "cep owns the CEP-0008 semantic-envelope *profile* ... and is
  also a consumer of the v1 wire schema" (`tests/test_contract_drift.py:3-5`).
- `docs/settlement-call-surface.md` — a synchronous scoring *call surface* for a
  producer's live agent loop (`docs/settlement-call-surface.md:20-25`).


## 4. DomainForge's CEP integration

### 4.1 Interface

**A CLI subcommand, plus one public Rust function. No LSP, no Python/TS binding.**

- CLI: `domainforge envelope` — registered at
  `/home/sprime01/projects/domainforge/domainforge-core/src/cli/mod.rs:62-63`
  (`Envelope(envelope::EnvelopeArgs)`), handler
  `domainforge-core/src/cli/envelope.rs:93`.
- Library: `pub fn build_cep_envelope` (`domainforge-core/src/application/envelope.rs:1327`,
  params `CepEnvelopeParams` `:1292`, world params `CepWorldParams` `:1313`).
  Grep shows `build_cep_envelope` referenced only from
  `src/cli/envelope.rs`, `src/application/mod.rs` docs, and two Rust test files —
  it is **not** surfaced through the Python/TS/WASM bindings.

### 4.2 The CLI command that produces it

```
domainforge envelope --emit cep --world-name <name> \
  [--world-alias world:<name>] [--world-label <text>] <entry.sea>
```

`--emit` values are `representation` (default) | `cep` | `both` | `verification-contract`
(`domainforge-core/src/cli/envelope.rs:74-83`). `--world-name` / `--world-alias` /
`--world-label` are declared at `domainforge-core/src/cli/envelope.rs:48-60`, with
`--world-alias` and `--world-label` declared `requires = "world_name"`.

### 4.3 Emitted JSON — top-level shape

Inserted in this order by `build_cep_envelope`
(`domainforge-core/src/application/envelope.rs:1343-1520`):

| key | value / source |
|---|---|
| `envelope_id` | fresh UUID per emission (`:1345`) |
| `cep_version` | const `"0.1.0"` — `CEP_VERSION` at `:1287`, inserted `:1346` |
| `envelope_version` | const `"1.0.0"` — `CEP_ENVELOPE_VERSION` at `:1278`, inserted `:1347-1350` |
| `envelope_kind` | const `"semantic_snapshot"` — `:1281`, inserted `:1351-1354` |
| `created_at` | supplied by CLI (`:1355`) |
| `created_by` | `format!("{} {}", CEP_PRODUCER_NAME, env!("CARGO_PKG_VERSION"))` (`:1328`, inserted `:1356`) |
| `scope` | caller `--scope` JSON, plus `world_ref` / `world_alias` / `world_label` when pinned (`:1358-1384`) |
| `boundary_record` | `{scope, included_sections, excluded_sections, known_omissions, unknowns, redactions, compression_notes, out_of_scope_entities, limitations}` (`:1385-1398`) |
| `completeness_status` | `complete_for_declared_state` \| `incomplete_for_declared_state` (`:1399-1406`) |
| `provenance_refs` | `["source-set:<hash>"]` (`:1407-1410`) |
| `omission_status` | `"known_omissions_recorded"` (`:1411-1414`) |
| `lineage_refs` | `[]` (`:1415`) |
| `provenance` | one record; `production_method: "domainforge envelope --emit cep"` (`:1418-1431`) |
| `references` | one `ref_type: "source_set"` record (`:1432-1448`) |
| `validation_status` | `"validated"` (both branches: `:1451-1454`, `:1493-1496`) |
| `representations` | `[build_representation(...)]` when D exists (`:1456-1459`) |
| `integrity` | `{content_hash, semantic_hash}` only when pinned (`:1470-1476`) |
| `conformance_status` | `"conformant"`, failure path only (`:1489-1492`) |
| `omissions` | failure path only: `omission_type: "representation_unavailable"` (`:1497-1514`) |
| `extensions` | inserted only when non-empty (`:1534-1538`) |

`extensions` members:

```
"cep.profile"           -> {"profile_id": "godspeed.semantic_snapshot",
                            "profile_version": "1.0.0"}      // only if p.world.is_some()
"domainforge.identity"  -> {"domain_model_identity": {...}, "semantic_closure_hash": "..."}
"domainforge"           -> {"model_validation_status": "invalid", "diagnostics": [...]}
```

Constants: `CEP_PROFILE_SEMANTIC_SNAPSHOT = "godspeed.semantic_snapshot"` and
`CEP_PROFILE_VERSION = "1.0.0"` (`domainforge-core/src/application/envelope.rs:1320-1321`).

**Critical gate:** the profile declaration is emitted only when a world binding was
requested:

```rust
if p.world.is_some() {
    extensions.insert("cep.profile".to_string(), serde_json::json!({
        "profile_id": CEP_PROFILE_SEMANTIC_SNAPSHOT, "profile_version": CEP_PROFILE_VERSION }));
}
```
(`domainforge-core/src/application/envelope.rs:1525-1532`)

So `domainforge envelope --emit cep <entry>` **without** `--world-name` emits a
`semantic_snapshot`-kind envelope that does *not* claim the `godspeed.semantic_snapshot`
profile — and therefore fails profile validation by design (see §6, CEP test at
`tests/test_domainforge_snapshot_conformance.py:66-70`).

### 4.4 Schema id / version of the emitted document

The emitted CEP envelope carries **no** `$schema` and **no** `schema_version` field.
Its only version markers are `cep_version: "0.1.0"` and `envelope_version: "1.0.0"`
(both above). DomainForge publishes **no JSON Schema** for the CEP envelope; the only
envelope schema it ships is for the canonical representation D —
`schemas/canonical-semantic-envelope-v1.schema.json`, `$id`
`https://domainforge.dev/schemas/canonical-semantic-envelope-v1.schema.json`, with
`schema_version` const `"domainforge-semantic-envelope/v1"`
(`schemas/canonical-semantic-envelope-v1.schema.json:3`, `:16`), matching
`ENVELOPE_SCHEMA_VERSION` (`domainforge-core/src/application/envelope.rs:19`).


## 5. Compatibility / version statements

### 5.1 In DomainForge

`docs/reference/cli-reference.md:143` — the fullest statement of the contract:

> `cep` with `--world-name` pins the world it describes. For a valid model: `scope.world_ref`
> (plus `world_alias` and `world_label` when given), `integrity` (`content_hash`,
> `semantic_hash` = `semantic_closure_hash`), `extensions["domainforge.identity"]` carrying the
> 11-field `domain_model_identity` whose canonical digest is the `world_ref` digest, and
> `extensions["cep.profile"]` declaring `godspeed.semantic_snapshot` v1.0.0 (profile defined in
> the CEP repository, `spec/profiles/GODSPEED-PROFILES-v1.md`). For an invalid model the profile
> is declared, `extensions.domainforge.model_validation_status` is `invalid`, and no `world_ref`
> is carried, so an invalid world cannot masquerade as a valid one.

`docs/reference/cli-reference.md:135` — the back-compat clause:

> `--world-name <NAME>`: Bind the `cep` snapshot to a named semantic world (GodSpeed
> `semantic_snapshot` profile). Opt-in: without it the emission is unchanged.

`docs/reference/cli-reference.md:150` — exit-code contract:

> `1`: Model validation failed (syntax error, unresolvable import, or policy violation). In
> `--emit cep` or `--emit both` modes, a schema-valid CEP failure envelope is emitted with
> explicit omissions and diagnostics.

`docs/reference/sea-application-contract.md:943` — the "do not re-implement" clause:

> A `world_ref` is the ergonomic immutable reference to exactly one resolved semantic-world
> revision. It is defined here once, in `domainforge-core/src/application/world.rs`; other
> repositories consume it through the Rust crate or the Python, TypeScript and WASM bindings
> and must not re-implement the grammar.

`docs/reference/sea-application-contract.md:952` — single-hash clause:

> The digest is `canonical_digest()` and nothing else; there is no second hash.

`docs/reference/sea-application-contract.md:955` — revision semantics:

> Because the identity binds the exact source set, the producer release and the interpretation
> versions, a comment-only edit or a DomainForge upgrade mints a new `world_ref`.
> `semantic_closure_hash` is unchanged in both cases and is the field that proves two revisions
> mean the same thing.

`docs/specs/cep/VENDORED.md:9-12` — the pin table (see §5.3 for the drift):

```
| File | Source | Pinned commit | cep_version |
| CEP-0008-semantic-envelope.md | github/local `cep` repo, `spec/` | `ae9b73b7526e307ed6b03595329114aa5413143c` | 0.1.0 |
| CEP-0009-benchmark-model.md   | github/local `cep` repo, `spec/` | `ae9b73b7526e307ed6b03595329114aa5413143c` | 0.1.0 |
```

Runtime advertisement: `domainforge envelope --capabilities` prints
`"contracts": ["CEP-0008", "domainforge-semantic-envelope/v1"]`
(`domainforge-core/src/cli/envelope.rs:98-104`).

Conformance for the CEP envelope therefore lives in the **CEP repo's** JSON Schemas
(`schemas/profiles/godspeed/*.v1.schema.json`), not in DomainForge.

### 5.2 In CEP

`spec/profiles/GODSPEED-PROFILES-v1.md:26` — declaration obligation:

> An envelope claiming a profile MUST declare it at `extensions["cep.profile"]` with
> `profile_id` (`godspeed.<envelope_kind>`) and `profile_version` (`1.0.0`). A tool that assumes
> a profile the envelope does not declare is interpreting it invalidly (CEP-0008 §18.3).

`spec/profiles/GODSPEED-PROFILES-v1.md:32` — grammar ownership:

> DomainForge owns the grammar and the identity: `world:<name>@sha256:<digest>`, where the digest
> is `DomainModelIdentity::canonical_digest()`. The authoritative definition is
> `docs/reference/sea-application-contract.md` §14.1 in DomainForge. The schema patterns in this
> repository mirror it and are pinned to DomainForge's golden vectors
> (`schemas/vendored/world_ref/`).

`schemas/vendored/world_ref/README.md:13` — vendoring rule and release provenance:

> Copied byte-for-byte from DomainForge (`domainforge-core/tests/fixtures/world_ref/`; first
> vendored from `feat/world-ref` a3a4157, now on DomainForge `main` since PR #132 and published
> in release `v0.19.0`; the content is byte-identical, so the hash pin is unchanged). ... Do not
> edit; re-vendor from DomainForge.

`tests/test_domainforge_snapshot_conformance.py:3-6` — consumer-side version statement (the
branch name is stale; the flag is on DomainForge `main` as of `35c4ed1`):

> Gated: set CEP_TEST_DOMAINFORGE_BIN to a `domainforge` build that supports `--world-name`
> (DomainForge feat/cep-snapshot-world-ref or later).

`schemas/semantic-envelope.mapping.md:17-21` — the admitted v1 gap:

> `semantic-envelope.schema.json` is a JSON Schema profile of CEP-0008's Semantic Envelope,
> scoped to what the v1 flat contract (`sea.agent.event.v1.json`, durably enforced at
> agent-memory-ledger's boundary) can actually carry.

### 5.3 Version consistency, and one drift finding

Consistent across both repos:
- `profile_version` `1.0.0` — CEP `tools/generate_godspeed_profiles.py:24`; DomainForge
  `domainforge-core/src/application/envelope.rs:1321`.
- `profile_id` `godspeed.semantic_snapshot` — CEP generator `:152` (`f"godspeed.{kind}"`);
  DomainForge `:1320`.
- `cep_version` `0.1.0` — CEP-0008 frontmatter `version: 0.1.0`
  (`spec/CEP-0008-semantic-envelope.md:6`); DomainForge `CEP_VERSION` `:1287`; vendored pin
  column `cep_version` = `0.1.0`.
- `envelope_version` `1.0.0` (DomainForge `:1278`) is **not** constrained by any CEP profile
  schema. The CEP base envelope requires the *field* to exist
  (`schemas/cep-semantic-envelope.schema.json`) but no profile pins its value.

Drift — DomainForge's CEP spec pin predates the profile specification it cites:
- `docs/specs/cep/VENDORED.md` pins commit `ae9b73b7526e307ed6b03595329114aa5413143c`.
- That commit **exists** in `/home/sprime01/projects/cep`, but
  `git ls-tree ae9b73b -- spec/profiles` and `... -- schemas/profiles/godspeed` both return
  **empty** — neither the profile spec nor the profile schemas existed at the pinned commit.
- CEP added `spec/profiles/GODSPEED-PROFILES-v1.md` only afterwards
  (`git log -- spec/profiles/GODSPEED-PROFILES-v1.md` → `5111ec3`, `3b79e55`, `527f9ca`).
- Yet `docs/reference/cli-reference.md:143` names `spec/profiles/GODSPEED-PROFILES-v1.md` as
  the profile definition. The pointer resolves against CEP `HEAD` (`5111ec3`) but not against
  the commit DomainForge pins. `cep_version` is `0.1.0` in both, so this is commit-pin drift,
  not a spec-version mismatch.
- Also stale: CEP's `tests/test_domainforge_snapshot_conformance.py:3-6` still refers to
  DomainForge branch `feat/cep-snapshot-world-ref`.

## 6. Tests that assert the envelope shape

### 6.1 DomainForge

`/home/sprime01/projects/domainforge/domainforge-core/tests/envelope_emit_tests.rs`

- `assert_envelope_shape` (`:55-98`) — asserts twelve top-level keys are present, plus
  kind/version:

```rust
for required in ["envelope_id", "cep_version", "envelope_version", "envelope_kind",
    "created_at", "created_by", "scope", "boundary_record", "completeness_status",
    "provenance_refs", "omission_status", "lineage_refs"] {
    assert!(object.contains_key(required), "missing {required}");
}
assert_eq!(envelope["envelope_kind"], "semantic_snapshot");
assert_eq!(envelope["envelope_version"], "1.0.0");
```

  It also asserts all nine `boundary_record` keys (`:77-90`), that
  `boundary_record.known_omissions` contains `"source_comments"` (`:90-94`), and that
  `provenance` and `references` are non-empty (`:95-96`).
- `ac_df_001_envelope_shape_matches_canonical_contract` (`:99-129`) — applies the above per
  fixture, and asserts `validation_status == "validated"`, exactly one representation, and
  that `representation.content_hash` / `.semantic_hash` match the `sha256:<64 hex>` shape.
- `ac_df_003_invalid_model_still_emits_valid_envelope` (`:217-268`) — asserts
  `conformance_status == "conformant"`, `validation_status == "validated"`, exactly one
  omission with `omission_type == "representation_unavailable"` and
  `known_or_suspected == "known"`, plus `extensions.model_validation_status == "invalid"`
  and a hash-shaped `invalid_declared_checkpoint_hash`.
- `build_cep_envelope_is_pure_and_deterministic` (`:329-362`) — calls
  `build_cep_envelope` directly; identical inputs give identical output, and only
  `envelope_id` / `created_at` vary.

`/home/sprime01/projects/domainforge/domainforge-core/tests/cep_snapshot_world_tests.rs`

- `valid_world_is_pinned_with_identity_integrity_and_profile` (`:64-113`) — asserts
  `scope.world_ref` starts with `world:godspeed-corporate@sha256:`, that
  `scope.world_alias` / `world_label` round-trip, that the `world_ref` verifies against the
  carried identity, and the profile object exactly (`:104-107`):

```rust
assert_eq!(env["extensions"]["cep.profile"],
    json!({"profile_id": "godspeed.semantic_snapshot", "profile_version": "1.0.0"}))
```

- `without_world_name_emission_is_unchanged` (`:192-202`) — asserts `scope.world_ref` absent,
  `integrity` absent, and `env["extensions"].is_null()` ("no extensions for a plain valid
  snapshot"). This is the back-compat guard.
- `invalid_world_is_flagged_and_never_pinned` (`:204-233`) — non-zero exit, no `world_ref`,
  no `world_alias`, `extensions.domainforge.model_validation_status == "invalid"`,
  `extensions["cep.profile"].profile_id == "godspeed.semantic_snapshot"`, no
  `domainforge.identity`, and `completeness_status == "incomplete_for_declared_state"`.
- Also: `registry_hash_validation_controls_world_pinning` (`:115-170`),
  `cli_world_ref_equals_the_library_world_ref_for_the_same_source` (`:172-183`),
  `binding_is_stable_across_runs_apart_from_envelope_identity` (`:184-190`),
  `comment_only_edit_is_a_new_world_with_the_same_closure` (`:237-270`),
  `world_flags_are_validated` (`:272-315`, exit-code-2 cases),
  `world_binding_applies_to_both_emit_mode` (`:317-323`) — CLI/library parity and flag
  validation.

### 6.2 CEP

`/home/sprime01/projects/cep/tests/test_domainforge_snapshot_conformance.py` — the real-binary
conformance test, gated on `CEP_TEST_DOMAINFORGE_BIN` (`:22`), invoking
`[BIN, "envelope", "--emit", "cep", *flags, path]` (`:34-35`).

- `test_valid_world_snapshot_conforms_and_verifies` (`:42-56`):

```python
assert code == 0
assert godspeed.validate_profile(env, "semantic_snapshot") == []
assert godspeed.verify_semantics(env, "semantic_snapshot") == []
assert godspeed.world_ref_of(env) is not None
```

- `test_invalid_syntax_world_is_flagged_not_pinned_and_conforms` (`:58-64`) — non-zero exit,
  `world_ref_of(env) is None`, yet still schema-valid and semantic-valid.
- `test_snapshot_without_world_name_is_not_a_profile_snapshot` (`:66-70`) — the inverse
  assertion, that a plain `--emit cep` emission "must not claim the profile":

```python
code, env = _emit(HEADER + VALID)
assert code == 0
assert godspeed.validate_profile(env, "semantic_snapshot"), "must not claim the profile"
```

- `test_standard_jsonschema_agrees_on_emitted_snapshots` (`:72-…`) — cross-checks the
  independent `jsonschema` library against CEP's own validator on emitted snapshots.

`/home/sprime01/projects/cep/tests/test_godspeed_profiles.py` — profile-corpus conformance:

- `test_corpus_schema_verdicts` (`:80`) and `test_corpus_semantic_verdicts` (`:96`) —
  every fixture under `schemas/fixtures/godspeed/` must get its declared verdict, with a
  required substring in the error.
- `test_every_profile_has_a_valid_and_an_invalid_fixture` (`:107-111`) and
  `test_all_seven_dispositions_have_valid_fixtures` (`:114-125`) — corpus completeness over
  the seven profiles and seven dispositions.
- `test_schema_world_ref_mirrors_domainforge_vectors` (`:58-65`) — the mirrored `world_ref`
  pattern must accept/reject exactly what the vendored DomainForge golden vectors say.
- `test_digest_recomputation_matches_the_vendored_reference` (`:68-76`) — asserts
  `godspeed.identity_digest(case["identity"]) == case["canonical_digest"]`, and the same for
  the vendored reference implementation.
- `test_vendored_domainforge_vectors_match_their_pin` (`:42-44`) — sha256 pins on
  `golden-vectors.json` (`PIN_VECTORS`, `:30`) and `world_ref_reference.py`
  (`PIN_REFERENCE`, `:31`).
- `test_lineage_must_stay_in_one_world` (`:129-138`) and
  `test_world_transition_is_explicit_and_must_move` (`:141-156`).
- `test_standard_jsonschema_library_agrees_with_cep_validator_on_the_whole_corpus`
  (`:165-176`) — `standard_ok == cep_ok == (case["schema"] == "valid")` over the full corpus.

## 7. Bottom line

- CEP defines **seven** envelope profiles at version `1.0.0`, all requiring
  `scope.world_ref` (pinned) except the `semantic_snapshot` invalid-world branch, and all
  requiring `extensions["cep.profile"]`.
- `world_ref` grammar and identity are **owned by DomainForge**; CEP mirrors the pattern and
  pins the vendored golden vectors.
- CEP defines **no** RLM / recursion / agent-execution contract.
- DomainForge emits exactly **one** CEP profile — `godspeed.semantic_snapshot` v1.0.0 — via
  `domainforge envelope --emit cep --world-name <name>`, and only when `--world-name` is
  supplied. It emits no `authority_request`, `authority_decision`, `execution_trace`,
  `evidence_packet`, `settlement_packet`, or `context_bundle` envelope.
- No version-negotiation layer exists beyond `profile_version` and `cep_version`.

## VERIFIED / UNVERIFIED

**VERIFIED**

- Both repo HEADs (`cep` 5111ec3, `domainforge` 35c4ed1) and clean worktrees.
- The seven profile ids, versions, required fields, extension requirements, and the eighth
  non-profile `world-transition` schema — read from the generated schemas, the generator
  source, and the profile spec.
- `scope.world_ref` as the world reference/revision carrier, its grammar, and the
  alias/label non-identity rules.
- The `authority_request` / `authority_decision` / `evidence`+`references` / `settlements`
  field assignments.
- Zero RLM matches in CEP, with the exact searches reproduced.
- DomainForge's integration interface (CLI subcommand + one public Rust fn; no LSP, no
  binding), the exact CLI invocation, the full emitted top-level key set with line anchors,
  the `cep_version` / `envelope_version` constants, the absence of any `$schema` /
  `schema_version` in the emission, and the `p.world.is_some()` gate on the profile
  declaration.
- The compatibility/version quotes in §5, and the fact that DomainForge's pinned CEP commit
  `ae9b73b` contains neither `spec/profiles/` nor `schemas/profiles/godspeed/`.
- The test locations and the quoted assertions in §6.

**UNVERIFIED**

- The emitted envelope was **not** executed end-to-end. §4.3's shape is read from
  `build_cep_envelope` source, not from captured `domainforge` stdout; I did not run
  `cargo test`.
- `tests/test_domainforge_snapshot_conformance.py` was **not** run — it is gated on
  `CEP_TEST_DOMAINFORGE_BIN` and needs a built `domainforge` binary.
- Whether any other producer in this ecosystem (e.g. `godspeed_agent`) emits the remaining six
  profiles was not investigated; neither repo contains such an emitter.
- The upstream provenance of the vendored `sea.agent.event.v1.json`
  (`hassos-addon-agent-memory-ledger`) was not checked — only the local pin and hash were read.
- The claim that DomainForge's CEP pin *predates* the profile spec is inferred from
  `git ls-tree` at that commit in the local CEP clone. Whether the pin was *intended* to cover
  the profile spec is not stated anywhere I read, so §5.3's "drift" is a factual observation
  (`ls-tree` empty) with an interpretive label attached.
