# SEA Forge Source Traceability Map

This map connects the system's concepts and invariants to the code, tests, and persisted records that implement them. Use it to move from a claim in the docs to a file you can inspect.

Paths are relative to the repository root. Line anchors are omitted on purpose: they drift. Open the symbol by name.

---

## 1. Invariants and Enforcement

| Invariant | Claim | Implementation | Verification |
|---|---|---|---|
| **AUTH-01** | No workload process or workload payload exists without `Allow` | [`ActionGrant`](../crates/sea-forge-authority/src/lib.rs), [`run_intent`](../crates/sea-forge-cli/src/pipeline.rs), [`CaseRunner`](../crates/sea-forge-case-runner/src/lib.rs) | [`cell_episode.rs`](../crates/sea-forge-case-runner/tests/cell_episode.rs). Intent runs create control-plane directories before evaluation; the invariant covers workload effects. |
| **AUTH-02** | One authority fabric serves CLI, server, and protected paths | [`PolicyAuthorityEngine`](../crates/sea-forge-authority/src/lib.rs), [`case_dispatch`](../crates/sea-forge-server/src/case_dispatch.rs) | [`conformance_m0_authority.rs`](../crates/sea-forge-authority/tests/conformance_m0_authority.rs) |
| **DOM-01** | Settlement never trusts exit status alone | [`settle`](../crates/sea-forge-settlement/src/lib.rs), [`SettlementCriteria`](../crates/sea-forge-core/src/types.rs) | [`criteria_provenance.rs`](../crates/sea-forge-settlement/tests/criteria_provenance.rs) |
| **DOM-02** | Every settled activation has trace and evidence | [`TraceEvent`](../crates/sea-forge-core/src/types.rs), [`EvidenceRecord`](../crates/sea-forge-core/src/types.rs) | [`cell_episode_live.rs`](../crates/sea-forge-case-runner/tests/cell_episode_live.rs) |
| **DOM-03** | A bound domain model is resolved before authority decides | [`load_validate`](../crates/sea-forge-domainforge/src/lib.rs), [`evaluate`](../crates/sea-forge-domainforge/src/lib.rs) | [`conformance_m10.rs`](../crates/sea-forge-planner/tests/conformance_m10.rs) |
| **BUILD-01** | 22 kernel crates stay free of Tokio, async runtimes, and HTTP clients | Crate manifests | `just no-async-kernel` |
| **DATA-01** | Source history is append-only; views are rebuildable | [`LedgerStream`](../crates/sea-forge-ledger/src/types.rs) | [`conformance_m0_ledger.rs`](../crates/sea-forge-ledger/tests/conformance_m0_ledger.rs) |
| **DATA-02** | Flat and case-owned run layouts resolve through one locator | [`run_views`](../crates/sea-forge-server/src/sfwp/run_views.rs) | Server conformance tests |
| **DATA-03** | Request ids are idempotency keys | [`correlation`](../crates/sea-forge-server/src/sfwp/correlation.rs) | Server conformance tests |
| **API-01** | Rust types are canonical; JSON Schema and TypeScript are committed projections | [`SCHEMA_TYPES`](../crates/sea-forge-server/src/sfwp/mod.rs), [`gen_sfwp_schema`](../crates/sea-forge-server/src/bin/gen_sfwp_schema.rs) | `just workbench-check` |
| **GEN-01** | Generated zones are protected | [`path`](../crates/sea-forge-core/src/path.rs) | `just proof` P4 |
| **OPS-01** | A present but invalid `server.yaml` fails startup; root and socket stay fixed | [`ServerConfig`](../crates/sea-forge-server/src/config.rs) | Server unit tests |

---

## 2. Rust Workspace (24 Crates)

Two crates are async edges: `sea-forge-server` and `sea-forge-agent`. The other 22 are synchronous.

| Crate | Responsibility | Main entry points |
|---|---|---|
| [`sea-forge-core`](../crates/sea-forge-core/src/lib.rs) | Types, IDs, errors, canonical JSON, path rules, approvals fold | `ForgeError`, `ids`, `canonical`, `approvals` |
| [`sea-forge-domain`](../crates/sea-forge-domain/src/lib.rs) | Intent vocabulary and interpretation | `interpret`, `IntentPattern` |
| [`sea-forge-planner`](../crates/sea-forge-planner/src/lib.rs) | Deterministic intent-to-plan translation, criteria, sentries | `plan`, `stage_case_plan`, `criteria` |
| [`sea-forge-authority`](../crates/sea-forge-authority/src/lib.rs) | Policy decisions, candidate evaluation, grant minting | `PolicyAuthorityEngine`, `ActionGrant` |
| [`sea-forge-sandbox`](../crates/sea-forge-sandbox/src/lib.rs) | Sandbox selection, Landlock, Seatbelt, environment, network posture, OpenShell adapter | `select_sandbox`, `JailSandbox`, `NetworkPosture` |
| [`sea-forge-runtime`](../crates/sea-forge-runtime/src/lib.rs) | Grant consumption, executable re-resolution, process execution | `execute` |
| [`sea-forge-trace`](../crates/sea-forge-trace/src/lib.rs) | Append-only JSONL lifecycle trace | `JsonlTraceRecorder` |
| [`sea-forge-evidence`](../crates/sea-forge-evidence/src/lib.rs) | Evidence capture, artifact SHA-256, pre-mint identity | `JsonlEvidenceWriter`, `capture_file` |
| [`sea-forge-settlement`](../crates/sea-forge-settlement/src/lib.rs) | Criteria evaluation, batch records, declarations | `settle`, `declaration` |
| [`sea-forge-capability`](../crates/sea-forge-capability/src/lib.rs) | Envelopes, promotion policy, memory extraction, SQLite index | `append`, `recall`, `promotion` |
| [`sea-forge-ledger`](../crates/sea-forge-ledger/src/lib.rs) | ULID streams, hash chains, MMR proofs, Ed25519 checkpoints, witness receipts | `LedgerStream`, `LedgerManager` |
| [`sea-forge-domainforge`](../crates/sea-forge-domainforge/src/lib.rs) | Side-effect-free `.sea` parsing and candidate evaluation | `load_validate`, `evaluate` |
| [`sea-forge-server`](../crates/sea-forge-server/src/lib.rs) | Async SFWP edge, admission, correlation, identity, events, agent/MCP/Postgres edges | `ServerState`, `sfwp`, `config` |
| [`sea-forge-cli`](../crates/sea-forge-cli/src/main.rs) | Operator CLI: 21 visible top-level commands plus four hidden test commands | `main`, `pipeline`, `plan_pipeline`, `commands` |
| [`sea-forge-case-runner`](../crates/sea-forge-case-runner/src/lib.rs) | Shared CMMN-subset case execution primitives | `CaseRunner` |
| [`sea-forge-spec-pipeline`](../crates/sea-forge-spec-pipeline/src/lib.rs) | Deterministic ADR/PRD/Code/Test stage chaining | `compute_chain_hash`, `run_pipeline` |
| [`sea-forge-cell`](../crates/sea-forge-cell/src/lib.rs) | Federation bundle import/export and template adoption | `bundle`, `template` |
| [`sea-forge-artifact-ip`](../crates/sea-forge-artifact-ip/src/lib.rs) | Artifact-to-IP transitions | `transition` |
| [`sea-forge-self-model`](../crates/sea-forge-self-model/src/lib.rs) | Bundled canonical models and release verification | `bundled`, `verify_bundled` |
| [`sea-forge-extension`](../crates/sea-forge-extension/src/lib.rs) | Governed extension registry and descriptor validation | `ExtensionRegistry` |
| [`sea-forge-thoth`](../crates/sea-forge-thoth/src/lib.rs) | Manager loop, progress judgment, typed ask, separation of duties | `ThothManager`, `service` |
| [`sea-forge-agent`](../crates/sea-forge-agent/src/lib.rs) | Async Anthropic, OpenAI-compatible, and ACP drivers; sealed transcripts | `run_delegation`, `acp` |
| [`sea-forge-changeset`](../crates/sea-forge-changeset/src/lib.rs) | Git-backed transactional workspaces, validation, compare-and-swap promotion | `capture_base`, `collect`, `validate`, `promote` |
| [`sea-forge-realitytrace`](../crates/sea-forge-realitytrace/src/lib.rs) | Ledger-derived security evidence bridge to RealityTrace | `expectation_record`, `proof_completed`, `project` |

The four hidden CLI commands are `validate`, `stage-check`, `internal-test-sleep`, and `internal-test-swe-seed`. They exist for the executable's own validation path and internal tests; they are not part of the operator surface.

---

## 3. Workbench (`workbench/`)

| Component | Directory | Responsibility | Source |
|---|---|---|---|
| **React renderer** | `workbench/apps/desktop/src` | Routes, drafts, queries, XState machines, G1–G10 pre-flight guards | [`router.tsx`](../workbench/apps/desktop/src/router.tsx), [`guards.ts`](../workbench/apps/desktop/src/guards/guards.ts) |
| **Tauri host** | `workbench/apps/desktop/src-tauri` (deleted) | Former socket transport and event loop; removed 2026-10-13 | None. The renderer still imports `@tauri-apps/api`, so no live backend bridge exists. |
| **Contracts** | `workbench/packages/contracts` | Generated TypeScript types and AJV validators | [`generate.ts`](../workbench/packages/contracts/scripts/generate.ts) |
| **UI components** | `workbench/packages/sea-forge-ui-components` | Governed primitives and stories | `src/` |
| **UI tokens** | `workbench/packages/sea-forge-ui-tokens` | Design tokens for the Astryx neutral theme | `tokens.json`, `dist/` |

The guard verdict set is `passed`, `failed`, `indeterminate`, or `not_applicable`. Guards describe readiness; they do not enforce authority.

---

## 4. Persisted Records

| Record | Path | Shape | Write path |
|---|---|---|---|
| Intent case record | `.sea-forge/cases/<case_id>.json` | `Case` | `write_json_new` |
| Planned case record | `.sea-forge/cases/<case_id>/case.json` | `Case` | `write_json_new` |
| Case events | `.sea-forge/cases/<case_id>/case-events.jsonl` | `TraceEvent` | Append with ledger commit |
| Plan | `.sea-forge/runs/<run_id>/plan.json` | `CasePlan` | Written once before `plan_created` |
| Authority view | `.sea-forge/runs/<run_id>/authority.json` | `AuthorityDecision[]` | Materialized from ledger |
| Trace | `.sea-forge/runs/<run_id>/trace.jsonl` | `TraceEvent` | `JsonlTraceRecorder` |
| Evidence | `.sea-forge/runs/<run_id>/evidence.jsonl` | `EvidenceRecord` | `JsonlEvidenceWriter` |
| Settlement | `.sea-forge/runs/<run_id>/settlement.json` | `SettlementEvent` | After `settle` |
| Semantic envelope | `.sea-forge/runs/<run_id>/semantic-envelope.json` | `SemanticEnvelope` | On settlement |
| Capabilities | `.sea-forge/capabilities.jsonl` | `SemanticEnvelope` | `capability::append` |
| Memory items | `.sea-forge/memory/items.jsonl` | Extracted memory items | Deterministic extraction |
| Memory index | `.sea-forge/memory/index.sqlite` | SQLite FTS | Derived; `sea-forge memory rebuild` |
| Approvals | `.sea-forge/approvals.jsonl` | `ApprovalRequest` | `approvals::append`; latest line wins |
| Ledger | `.sea-forge/ledgers/<stream>/entries.jsonl` | `LedgerEntry` | `LedgerStream` with `flock` |

`.sea-forge/` is runtime output, not source. Do not treat any path under it as a repository artifact.

---

## 5. Where the Normative Text Lives

* [`AGENTS.md`](../AGENTS.md) defines the operating contract and required gates.
* [`.agents/specs/`](../.agents/specs) holds normative specs.
* [`.agents/CURRENT_STATUS.yaml`](../.agents/CURRENT_STATUS.yaml) is the handoff record.
* [`docs/execution/`](execution/) is historical execution material, not the current architecture description.

When code and prose disagree, believe the code and tests, then fix the prose.
