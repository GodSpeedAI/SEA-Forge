# Recon 04: DomainForge, CEP, Context Kernel

Read-only. Ahead/behind uses local `origin/*` refs (no fetch), so it may be stale. Tags: OBS = seen in code/test/doc; INF = inferred. Short names: DF = domainforge, CEP = canonical-evaluation-protocol, CK = Context-Kernel. Paths under `/home/sprime01/projects/`.

## A. DomainForge (DF)

### A1. Identity
| Checkout | Branch | HEAD | Dirty | Ahead/behind origin/main |
|---|---|---|---|---|
| `domainforge` (only worktree) | main | f90905a | 6: M `.claude/helpers/graft-hooks.cjs`, M `.claude/helpers/graft-statusline.cjs`, M `.claude/settings.json`, M `.claude/skills/graft/SKILL.md`, M `AGENTS.md`, ?? `.agents/reports/2026-11-02-branch-reconciliation.md` (tooling/docs only) | 0/0 |
| `domainforge-lsp` (only worktree) | semantic-adapter | 1e35f0c | clean | semantic-adapter 7/0 (+7362/-74, 20 files, adds semantic-pack support); dev 3/0; feature/fotmat-handler 1/22 (stale) |
| `domainforge-vsc-extension` | dev | 64bc8fa | clean | dev 2/7 (+302/-16: AST JSON commands, README) |
- No `domainforge` entry in `gsr-heads/` (OBS). Pinned DF head unknown.
- Open PR hints (OBS, from `.agents/next_steps.md`): #114 draft `agent/projection-families`; dependabot #154-163; #164 native CLI distribution.

### A2. Ownership
- OWNS: `.sea` grammar and parser, canonical `Graph`, policy evaluation, projections/exports, world identity (`world_ref`), authority resolver/pack/signing, application contract, CLI, Python/TS/WASM bindings.
- DOES NOT OWN (OBS): settlement (`src/projection/cell/evidence.rs:1-4`), evidence reliability (`src/application/verification_contract.rs:7-8` defers to CEP), observed-state ingestion (`src/projection/ai_learning/cep_eval.rs:9-11`).

### A3. Capabilities
| Capability | Evidence (domainforge-core/) | Tests | Status |
|---|---|---|---|
| PEG parser | `grammar/sea.pest` (582 lines), `src/parser/ast.rs` (3967 LOC), `ast_convert.rs` | `tests/parser_ast_v3.rs`, `*_parser_tests.rs` | IMPLEMENTED |
| Graph + KG/RDF export (turtle, rdf/xml, SHACL) | `src/graph/mod.rs`, `src/kg.rs` | 129 test files in `tests/` | IMPLEMENTED; import gap (A8) |
| Policy evaluation (3-valued, quantifiers) | `src/policy/core.rs:180` `evaluate`; `three_valued.rs`, `quantifier.rs` | `tests/authority_conformance_tests.rs` (1209 LOC) | IMPLEMENTED |
| Authority resolver/pack/signing/trace | `src/authority/*` (signing feature-gated) | authority conformance tests | IMPLEMENTED |
| Semantic pack + diff | `src/semantic_pack/diff.rs:44` `diff_packs`; builder, review, signing | not checked | IMPLEMENTED (UNVERIFIED tests) |
| World identity | `src/application/world.rs` (`WorldRef`, `WorldAlias`, `WorldCatalog::pin`) | not checked | IMPLEMENTED |
| Verification projection | `src/application/verification_contract.rs` (CEP work_request) | not checked | IMPLEMENTED |
| SBVR, CALM, Protobuf | `src/sbvr.rs`, `src/calm/` (import 1220 LOC), `src/projection/protobuf.rs` (3331 LOC) | not checked | IMPLEMENTED (UNVERIFIED) |
| Other projections | 22 dirs in `src/projection/` (A4); `docs/projection-target-implementation-status.md` marks 8 targets implemented only when gate scripts pass | `scripts/verify/projection-targets/*.sh` | PARTIAL/UNVERIFIED beyond the status table |
| `domainforge test` | `src/cli/test.rs:9` returns Ok(()) | none | STUB (OBS; D-doc-egm-2) |

### A4. Interfaces
- Top: `program = SOI ~ file_header? ~ declaration* ~ EOI` (`sea.pest:9`); `declaration_inner` (`:34`).
- Declaration kinds (keywords `:123-131`):
  - Modelling: `entity` (`:85`, `v` version, `in` domain, body), `resource` (`:108`), `flow` (`:143`, `from`/`to`/`quantity`), `role` (`:164`), `role_binding` (`:174`), `relation` (`:185`), `instance` (`:224`), `pattern` (`:157`, regex), `dimension` (`:238`), `unit` (`:244`), `record`/`enum` (`:254`/`:257`, contextual), `operation` (`:292`).
  - Policy/change: `policy` (`:337`; Constraint/Derivation/Obligation; Obligation/Prohibition/Permission; priority; `v`), `ConceptChange` (`:200`; `@from_version`, `@to_version`, `@migration_policy`, `@breaking_change`), `metric` (`:520`).
  - Interop: `Mapping`/`Projection "x" for <calm|kg|sbvr|protobuf|proto>` (`:543`, `:568`, `:547`).
  - Deployment: `cell`, `systemdependency`, `runtime`, `tool`, `dependencyset`, `service`, `mount`, `endpoint`, `networkflow`, `credential` (`:41-79`).
  - Modules: `import`/`export` (`:15`, `:22`), file annotations (`:12`). Namespaces are not a grammar keyword (INF: `module/resolver.rs`).
- Operation clauses (`:294-330`): `intent`, `direction`, `access` (public, or `policy_governed by <policy> at precondition|invariant|postcondition fails with <id>`), `input`, `output`, `state`, `effect`, `transaction`, `failure`, `evidence operation_trace`, `lifecycle synchronous_request_response`.
- Identity: `ConceptId` = UUIDv5 of `"{namespace}::{name}"` with fixed namespace (`src/concept_id.rs`); `content_hash`/`element_id` (`src/projection/ids.rs:18-25`); `world:<name>@sha256:<64hex>` (`src/application/world.rs:9`). Version: `SemanticVersion` (`src/semantic_version.rs`).
- CLI (`src/cli/mod.rs:36-62`): parse, validate, import, project, format (alias fmt), test (stub), validate-kg, normalize, registry, authority, pack, contract, envelope.
- Bindings: Rust core; features `cli`, `python` (`domainforge-python/`, pyo3), `typescript` (`domainforge-typescript/`, napi), `wasm` (`src/wasm/`, `pkg/`).
- Projection dirs: ai_learning, alloy, archimate, asyncapi, baml, bpmn, cedar, cell, cloudevents, cmmn, dagger, devbox, domain, dspy, gauge, lean, openshell, otel, rdf, rego, tla, zenml; plus calm, kg, sbvr, protobuf, yaml, flows, contracts.

### A5. Cross-system
- DF to CEP: `work_request` envelopes with content-derived `obligation_id`/`claim_id`/`question_id` (`verification_contract.rs:10-12`); eval datasets (`cep_eval.rs`, vendored CEP specs in `docs/specs/cep/`).
- DF to LSP and VS Code: grammar consumers (LSP semantic-adapter branch; VSIX from extension repo). Diagnostics and formatting only; no model comparison.
- Horizon, Cognate, RealityTrace, SXR: not traced in this recon.

### A6. UX
- No product UI. Editor surfaces only: VS Code extension (TextMate grammar, diagnostics, format, AST JSON commands) and LSP.

### A7. Unfinished
- domainforge main: tooling dirty files only.
- lsp semantic-adapter: 7 commits not on local origin/main (unmerged feature work).
- vsc dev: 2 commits unmerged.
- Debt (OBS, `.agents/current_state.md`): D-doc-egm-1 `println!` debug at `src/parser/ast.rs:1283` runs in release and can corrupt `parse --format json`; D-doc-egm-2 `test` stub; D-doc-egm-3 PARTIAL: `kg.rs:1081-1087` `to_graph` drops `sea:EntityInstance` and `sea:Policy` triples.
- 317 files mention TODO/FIXME (not triaged).

### A8. Risks and contradictions
- Three identity schemes (ConceptId, `ids.rs` content_hash, world_ref digest); reconciliation not found (INF).
- Grammar `target_format` lists 4 targets (`sea.pest:547`); `src/projection/` has ~22 families. DSL surface and projection registry are not aligned (INF).
- `ConceptChange` is declarative; no migrator found (UNVERIFIED).
- `.sea/interaction` interaction model (ADR-015:199; `kg.rs:1087`) is dead in DF main (`.agents/next_steps.md` item 7).
- `domainforge test` exits 0 without running anything: false-green risk.

### A9. Unknowns
- Callers of `concept_change` in `src/cli/` and `semantic_pack/`.
- DF pin for the GodSpeed stack (check `scripts/readiness.sh`).
- Test status not run (read-only). Reconciliation report claims 1,294 tests passed (INF, not re-run).

## B. CEP: canonical-evaluation-protocol

### B1. Identity
| Checkout | Branch | HEAD | Dirty | Ahead/behind origin/main |
|---|---|---|---|---|
| `cep` (only worktree on profiles branch) | profiles/context-bundle | d7591c1 | ?? `.agents/OBSERVED_DEBT.md` (1 entry: CEP-0008 hash algorithm undefined) | 1/1 |
| `cep` main | main | 965e639 | clean | 0/0 |
| `gsr-heads/cep` | detached | 5111ec3 | clean (pinned) | n/a |
- Branch tip and pinned head have identical trees (`git diff --stat 5111ec3 d7591c1` empty, OBS). Only commit ids differ.
- PR #3 (`feat(profiles): godspeed.context_bundle`) is the branch work; PRs #1, #2 merged to main.

### B2. Ownership
- Vendor-neutral protocol for evaluating represented transformations: question, evidence, settlement, repeatability (README). Specs CEP-0000..0010 in `spec/`.
- Does NOT own `world_ref` grammar: `src/cep/godspeed.py:8` says it never parses it; it only checks the digest suffix (`:111`).

### B3. Capabilities
| Capability | Evidence | Tests | Status |
|---|---|---|---|
| Semantic envelope (CEP-0008) | `schemas/cep-semantic-envelope.schema.json`, `spec/CEP-0008-semantic-envelope.md` | `tests/test_semantic_envelope.py` | IMPLEMENTED |
| godspeed.semantic_snapshot profile | `src/cep/godspeed.py` (`world_ref_of` :64; digest check :93-113) | `test_godspeed_profiles.py`, `test_domainforge_snapshot_conformance.py` | IMPLEMENTED (main) |
| godspeed.context_bundle profile | branch only: `schemas/godspeed/context_bundle.v1.schema.json` (222 lines), valid/invalid fixtures, `tools/generate_godspeed_*.py` | fixtures only; no dedicated pytest found | PARTIAL (not on main) |
| Anti-collapse conformance | `src/cep/conformance/anti_collapse.py`, `corpus.py`; CLI `conformance anti-collapse` (`cli.py:168`) | `test_anti_collapse_*.py`, `test_benchmark_invariants.py` | IMPLEMENTED |
| Benchmarks: authority, capability, reality_gap, settlement | `src/cep/benchmarks/*.py`; CLI `run reality-gap` (`cli.py:208`), `run authority-corpus` (`:260`) | `test_authority_bench.py`, `test_reality_gap_*.py`, `test_settlement_bench.py` | IMPLEMENTED |
| Schema validation CLI | `cli.py:100-152` | `test_validation.py` | IMPLEMENTED |
| Ledger exporter (Postgres) | `src/cep/exporters/ledger_postgres.py` | `test_ledger_exporter*.py` | IMPLEMENTED (depth UNVERIFIED) |
| Settlement | `spec/CEP-0007-settlement-model.md:73`: settled, partially settled, unsettled, failed, reversed, harmful, unauthorized, debt-bearing, settlement-blocked | bench only | SPEC + bench; no runtime settler found (INF) |

### B4. Interfaces
- Typer CLI `cep`: `doctor`; `validate {schemas, reality-gap-corpus, semantic-envelope, all}`; `conformance anti-collapse`; `run {reality-gap, reality-gap-corpus, authority-corpus}` (`src/cep/cli.py:51-260`).
- Envelope integrity: `content_hash`/`semantic_hash` pattern `^[a-z0-9]+:[0-9a-f]{16,128}$`, no algorithm (OBS).
- Profile IDs: `godspeed.semantic_snapshot`, `godspeed.context_bundle` (`spec/profiles/GODSPEED-PROFILES-v1.md:42-43`). Bundle: `completeness_status`; `extensions["godspeed.context_bundle"]` with `work_request_ref`, `retrieval_completeness` in complete|partial|none.
- World transition record (profiles §5, L72): `source_world_ref`, `target_world_ref`, `transition_kind` in semantic_change|source_edit_only|compiler_upgrade|migration, `reason`.
- Rule (profiles L35-36): historical `world_ref` is never retargeted; unknown/unverifiable world_ref at a protected boundary fails closed or escalates.
- Verdict/evidence schemas: `schemas/reality-gap-{answer,answer-key,case,evidence}.schema.json`, `benchmark-result.schema.json`, `conformance-report.schema.json`. No single verdict enum found (INF).

### B5. Cross-system
- DF to CEP: work_request (`verification_contract.rs`), eval datasets (`cep_eval.rs`), world_ref digest check.
- CK to CEP: CK implements the context_bundle projection (`ck-mcp/src/cep_bundle.rs:408`) and `cep.profile` tool (OBS).
- SEA-Forge: merge order lists cep #1/#2 merged, #3 pending (`sea-rs/.agents/reports/cep-world-migration/final-report.md`).

### B6. UX
- None. CLI and docs only.

### B7. Unfinished
- Branch PR #3: 17 files, +943/-2 vs main (`git diff --stat main...profiles/context-bundle`): bundle schema, fixtures, two generators, profile spec edit. Not merged, yet CK already implements it.
- Debt: CEP-0008 must define the hash algorithm, covered fields and canonical JSON rule; closure names `src/cep/godspeed.py` and `contracts/rust/src/lib.rs` (OBS, untracked ledger).

### B8. Contradictions and risks
- HASH CONTRADICTION (OBS): CEP-0008 defines no algorithm; CK implements `canonical_json` and `content_hash` (`cep_bundle.rs:108`, `:156`) and `verify_envelope_integrity` (`:197`).
- CONTRACT SKEW (OBS): CK depends on a schema that exists only on the unmerged CEP branch.
- Reality gap split: CEP `reality_gap` bench exists; DF says RealityGapBench needs observed-state inputs DF does not accept (`cep_eval.rs:9-11`). No bridge found.

### B9. Unknowns
- Whether DF or CK call CEP at runtime (library, schema or process). Not traced.
- Whether sea-rs pins `5111ec3` or `d7591c1` (same tree).

## C. Context Kernel (CK)

### C1. Identity
| Checkout | Branch | HEAD | Dirty | Ahead/behind origin/main |
|---|---|---|---|---|
| `Context_Kernel` (only worktree) | main | 36bd8c8 | D `.jolli/jollimemory/telemetry-queue.ndjson` (Jolli telemetry, not product) | 0/0 |
| `backup/harden-pre-sync` (branch, no checkout) | backup/harden-pre-sync | 62ee348 | n/a | 1/37; its one unique commit is `62ee348` (docs). CEP-bundle commits 5659f12, 3111d26 already on main |
| `gsr-heads/context-kernel` | detached | 81a6fdb | clean (pinned) | pinned is an ancestor of main; main is 25 ahead |
- Pinned 81a6fdb = PR #5 (stable SHA-256 doc ids, fail-closed canonical numbers, offline SSRF tests).
- sea-rs final report cites `migration/cep-context-bundle @ 270e911` for CK #2 (OBS). Pin lags main; confirm which pin the readiness gate uses.

### C2. Ownership
- OWNS: governed local context service over MCP (stdio, SSE): corpus-scoped retrieval (sources, corpora, documents, chunks, FTS, vector), library identity/alias resolution, session event memory with retention classes, artifact storage with provenance, refresh jobs, context packets and requirements, CEP context-bundle projection, append-only context-effect ledger (JSONL).
- DOES NOT OWN (INF from `docs/concepts.md` framing): authority, approval, settlement, execution.

### C3. Capabilities
| Capability | Evidence (crates/) | Tests | Status |
|---|---|---|---|
| MCP server stdio/SSE | `ck-bin` (549), `ck-transport-stdio` (51), `ck-transport-sse` (181), `ck-mcp` (2744) | `ck-mcp/tests`, `ck-bin/tests`; stdio has none | IMPLEMENTED |
| Bearer auth | `ck-auth/src` (151) | 1 file | IMPLEMENTED (depth UNVERIFIED) |
| SQLite storage | `ck-storage-sqlite` (2528): sources, corpora, corpus_sources, documents, chunks(+fts), library_identities, library_aliases(+fts), sessions, session_events(+fts), artifacts (+`<data_dir>/artifacts/*.bin`), refresh_jobs, schema_migrations | 1 file | IMPLEMENTED |
| FTS, vector, ranker, embeddings | `ck-search-fts` (559), `ck-vector` (627), `ck-ranker` (658), `ck-embeddings` (197) | 1 each | IMPLEMENTED (depth UNVERIFIED) |
| Adapters: filesystem, GitHub, web | `ck-adapter-{filesystem,github,web}` (596/642/923) | 1-2 each | IMPLEMENTED |
| Session memory retention | `ck-memory/src/lib.rs` `classify_event_kind`: decision/architecture/requirement/constraint=Pinnable; pinned/milestone=Permanent; else Ephemeral | 1 file | IMPLEMENTED (small) |
| Scheduler / refresh | `ck-scheduler` (510) | 1 file | IMPLEMENTED |
| CEP context bundle | `ck-mcp/src/cep_bundle.rs` (`project_context_bundle` :408, `to_cep_context_bundle` :531, `validate_context_bundle` :581); `verify_world_ref` in `ck-core/src/types/context.rs:89` | ck-mcp (7 files) | IMPLEMENTED against a profile not on CEP main |
| Handoff | 1 doc hit | none | NOT FOUND |

### C4. Interfaces
- MCP tools seen in `ck-mcp/src`: `get_status`, `context_required`, `cep.profile`, `godspeed.context_bundle`; payload types `context_packet`, `context_envelope`, `context_bundle`, `document`, `source`. Full list in `docs/mcp-contracts.md` (not read).
- IDs: `context_packet_id`, `context_requirement_id`, `artifact_id`, `document_id`, `session_id`, `source_id`, `corpus_id`, `chunk_id`, `envelope_id`, `content_hash`, `world_ref` (`world:<name>@sha256:<hex>`).
- Error codes: `corpus_not_found`, `corpus_root_unconfigured`, `unexplained_incompleteness` (`cep_bundle.rs:40`).
- Config (names only): `context-kernal.toml` (sic, consistent across code and examples); env `CK_DATA_DIR`, `CK_CONTEXT_CORPUS_ROOT`, `CK_CONTEXT_EFFECT_LEDGER`, `CK_LEARNING_PROPOSALS_LEDGER`.

### C5. Cross-system
- CK to CEP: in-process JSON projection (INF). Profile conformance via CEP branch schema.
- sea-rs: named as a stage in CEP world_ref migration and in `godspeed-runtime-integration-e2e-recon.md`; runtime call path not traced.
- Recorded run: sea-rs `stage11-settlement.md` cites "Context Kernel 237/0" (OBS via sea-rs report; not re-run).

### C6. UX
- None. MCP JSON only.

### C7. Unfinished
- `backup/harden-pre-sync` `62ee348` (docs) not on main; do not delete unmerged branches (per memory rule).
- `docs/plans/2026-06-02-deployment-readiness.md`: only plan file; not read.
- No TODO/FIXME in `crates/`. Telemetry deletion is tooling noise.

### C8. Risks
- Hash duplication: CK `canonical_json`/`content_hash`, CEP-0008 unspecified, DF `projection/ids.rs:18`.
- Depends on a CEP profile schema not on CEP main (B8).
- Memory vs authority: ck-memory and the effect ledger are append-only with no authority check in the crate (INF); consequential use must go through SEA-Forge authority.

### C9. Unknowns
- Runtime callers of CK (Cognate, SEA-Forge) and transport.
- Full MCP tool schemas; readiness gate's expected pin (main vs 81a6fdb).

## Cross-cutting
| Concept | DF | CEP | CK |
|---|---|---|---|
| Stable ID | ConceptId UUIDv5; ids.rs content_hash | content_hash, no algorithm | content_hash (`cep_bundle.rs:156`); SHA-256 doc ids |
| world_ref | owns grammar and digest | pins; checks digest suffix | verifies format |
| Model revision | ConceptChange, SemanticVersion, semantic_pack diff (no migrator found) | world transition record (profiles §5) | n/a |
| Observed vs declared | absent | reality_gap bench | n/a |
| Settlement | not owned | spec + bench | not owned |
| Lifecycle/gates | operation clauses only; no state-machine or stage declaration | n/a | n/a |

## Cannot express today (DF)
- Ordered lifecycle with stages, transitions, gates, roles, evidence conditions (only operation clauses and fixed `lifecycle synchronous_request_response`).
- Observed state, settlement, declared-vs-observed divergence.
- Evidence reliability or verifier choice (CEP).
- Executed model migration (declarations only).
- Interaction domain model (path dead in main).
- Full to_graph round trip for EntityInstance and Policy.

## Top findings
1. DF owns grammar and world identity; CEP and CK consume them.
2. CK implements `godspeed.context_bundle`, whose schema lives only on unmerged CEP PR #3.
3. CEP-0008 hash algorithm undefined while CK already canonicalizes (`cep_bundle.rs:108,156`).
4. No observed-vs-declared facility in DF; CEP reality_gap bench has no DF bridge.
5. Model revision = ConceptChange declarations plus immutable world_ref and CEP world-transition records; no executor found.
6. DF `test` is a false-green stub; `to_graph` round-trip gap.
7. Pins: CEP 5111ec3 (tree = branch); CK 81a6fdb (25 behind main); DF not in gsr-heads.
8. Unmerged work: lsp semantic-adapter (7 commits), vsc dev (2), CEP PR #3, CK backup docs commit.
