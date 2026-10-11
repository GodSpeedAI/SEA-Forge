# Spec → Code Traceability — SEA Forge

Maps canonical requirements to implementation and verification. **Canonical-document precedence** (per `AGENTS.md` Source of Truth rule, lines 31-46): user request → nearest `AGENTS.md` → `.github/copilot-instructions.md` (ABSENT, see OPEN-1) → `spec-minimum.md` → `spec-full.md` → `spec-adlc-thoth-minimum.md` → `spec-agent-orchestration.md`. `ARCHITECTURE.md` and `README.md` are summaries, **not** sources of truth; both are stale (OPEN-2).

Status legend: ✅ implemented + verified this pass · 🟢 implemented per CURRENT_STATUS (not re-verified) · 🟡 partial · 🔴 absent/broken · ⚪ unverified · ⏸ intentionally deferred/skipped.

## OPEN: Document precedence questions

- **OPEN-1**: `AGENTS.md` rule #3 invokes `.github/copilot-instructions.md` "when present". The file does **not** exist in the repo. The rule is therefore inert; no action required unless the team adopts Copilot.
- **OPEN-2**: `ARCHITECTURE.md:3` header reads "Status: intended architecture, pre-implementation"; `README.md` "Project layout" lists 2 crates. Both contradict observed reality (22 crates, ~909 passing tests, working desktop). Either the docs need updating or the user is treating them as historical context. **Recommend update** but did not edit in this pass.
- **OPEN-3**: Two empty placeholder spec files exist — `.agents/specs/api-routes.md` (0 bytes), `.agents/specs/datamodel.md` (0 bytes). Their canonical role is unclear; the live API/datamodel truth lives in `.agents/specs/frontend/sea-forge-workbench-api-spec-v0.1.md`, `.agents/specs/frontend/sea-forge-workbench-api-method-catalog-v0.1.yaml`, and `crates/sea-forge-core/src/types.rs`. Either delete the placeholders or fill them.
- **OPEN-4**: `working/face/` is an unrelated parallel design dump ("face" project). Decide: delete, move out of repo, or gitignore.

---

## M-MIN — Minimum governed kernel (`spec-minimum.md`, status "Implemented v0.1")

| Requirement | Source | Implementation | Verification | Status | Confidence | Blocker |
|---|---|---|---|---|---|---|
| Two-crate workspace (core, cli) | spec-minimum §0.1 | `crates/sea-forge-core/`, `crates/sea-forge-cli/` (plus 20 more additive crates) | `cargo check --workspace` V-02 | ✅ | high | — |
| Run lifecycle: plan→authority→sandbox→trace→evidence→settlement→envelope | spec-minimum §3.1 | `crates/sea-forge-core/src/pipeline.rs`, `crates/sea-forge-cli/src/pipeline.rs` | `just proof` P1 (V-05) | ✅ | high | — |
| `.sea-forge/runs/<run_id>/` with 6 record kinds | spec-minimum §3.1 | types in `crates/sea-forge-core/src/types.rs`; writers in trace/evidence/settlement/capability | P1 file-presence + jq assertions (V-05) | ✅ | high | — |
| Authority precedes side effects; default deny | spec-minimum §10.2, ARCH invariants #1-2 | `crates/sea-forge-authority/`; `ForgeError` typed | P3 (denial halts execution; no `command_started` in trace; workspace empty) | ✅ | high | — |
| Generated zone write denial | spec-minimum §15 | generated-zone check in planner/authority | P4 (`TEST_ONLY: write generated zone` exits 3) | ✅ | high | — |
| Settlement accepted requires non-empty basis | spec-minimum §3.2 | `crates/sea-forge-core/src/settlement.rs` | `jq -e '.status == "accepted" and (.basis | length > 0)'` (V-05) | ✅ | high | — |
| Pre-mint identity `ifl:hash:<sha256>` on work products | spec-minimum §7.3.8 | `ArtifactDescriptor` in evidence records | jq regex assertion (V-05) | ✅ | high | — |
| `sea-forge recall` reads capabilities.jsonl | spec-minimum G6 | `crates/sea-forge-cli/src/commands/recall.rs` | unit + integration tests (transitive V-04) | ✅ | high | — |
| Fail-closed on unclassifiable op | spec-minimum §14 | typed `ForgeError::UnknownIntent` (exit 2) | P3 exit-code assertion | ✅ | high | — |
| LocalWorkspaceSandbox is process-level only (known limitation) | spec-minimum §15, README | `crates/sea-forge-sandbox/src/` (process backend); Landlock/Seatbelt added in M1 | spec-minimum §15 caveat | ✅ (limitation documented honestly) | high | OS jail is M1, not minimum |
| P1-P4b proofs deterministic | spec-minimum §12.2 | `justfile:335-394` | `just proof` V-05 | ✅ | high | — |

---

## M-FULL — Full system milestones (`spec-full.md` status "Draft v0.2")

The full spec lists M0-M8 as the additive evolution. CURRENT_STATUS records M9-M16 as additionally implemented under `spec-adlc-thoth-minimum.md` and `spec-agent-orchestration.md`.

| Milestone | Source | Status per CURRENT_STATUS | Implementation evidence | Verification | Status | Confidence | Notes |
|---|---|---|---|---|---|---|---|
| M0 — integrity ledger + authority fabric + DomainForge adapter + extension ABI + crate graduation | spec-full §17 row M0 | landed | `crates/sea-forge-ledger/`, `crates/sea-forge-authority/`, `crates/sea-forge-domainforge/`, `crates/sea-forge-extension/` | conformance tests (transitive V-04) | 🟢 | medium | DomainForge is the **adapter shell** per ADR-001, not the first-party library binding; M0's "real `.sea` parse" claim depends on DomainForge library availability (not vendored) |
| M1 — jail backend (Landlock/Seatbelt) | spec-full §17 row M1 | landed | `crates/sea-forge-sandbox/src/jail.rs` (Linux Landlock) | `just no-async-kernel`, portable Landlock tests; Seatbelt skipped on Linux (V-23) | 🟢 | high (Linux); ⏸ macOS | — |
| M2 — case engine (CMMN-subset) + templates + criteria provenance | spec-full §17 row M2 | landed | `crates/sea-forge-case-runner/`, `crates/sea-forge-planner/src/templates.rs`, `crates/sea-forge-planner/src/criteria.rs` | conformance_m5, conformance_m10, conformance_m14 | 🟢 | high | — |
| M3 — server + approvals | spec-full §17 row M3 | landed | `crates/sea-forge-server/` (Tokio), approval surfaces in `crates/sea-forge-cli/src/approvals.rs` and `crates/sea-forge-server/src/sfwp/approvals.rs` | conformance M3 tests, V-SMOKE-1..3 | ✅ | high | Live server smoke confirmed |
| M4a — settlement integrity + capability memory | spec-full §17 row M4a | landed | `crates/sea-forge-settlement/`, `crates/sea-forge-capability/` (SQLite) | conformance M4a tests | 🟢 | high | — |
| M4b — governed recall (semantic memory) | spec-full §17 row M4b | landed | `crates/sea-forge-cli/src/commands/memory.rs`, FTS index | M4b proofs | 🟢 | medium | — |
| M5 — spec-to-code + generator pipelines + DomainForge projections | spec-full §17 row M5 | landed | `crates/sea-forge-spec-pipeline/`, `sea-forge project` CLI | conformance_m5 | 🟢 | medium | — |
| M6 — federation prep | spec-full §17 row M6 | landed | `crates/sea-forge-cell/src/bundle.rs` | M6 proofs | 🟢 | medium | `cell.migrate` distinct from `bundle::export/import` (OPEN debt) |
| M7 — environments + evaluators | spec-full §17 row M7 | landed | `sea-forge env list/show` CLI, `crates/sea-forge-planner/src/criteria.rs` | M7 proofs | 🟢 | medium | — |
| M8 — artifact-to-IP | spec-full §17 row M8 | landed | `crates/sea-forge-artifact-ip/`, `sea-forge artifact {synthesize,productize,capitalize,attest}` | M8 proofs | 🟢 | medium | — |

---

## M-ADLC — Thoth / ADLC minimum (`spec-adlc-thoth-minimum.md`, status "Draft v0.1")

CURRENT_STATUS records M9-M11 as landed.

| Milestone | Source | Status | Implementation | Verification | Status | Confidence |
|---|---|---|---|---|---|---|
| M9 — self-model genesis | spec-adlc-thoth E11 | landed | `crates/sea-forge-self-model/`, `sea-forge self-model {validate,rebuild,show}` | conformance_m9 | 🟢 | medium |
| M10 — ODI provenance + seed model resolver | spec-adlc-thoth | landed | `crates/sea-forge-planner/src/criteria.rs::SeedModelResolver` | conformance_m10 | 🟢 | medium |
| M11 — Thoth engine + ask | spec-adlc-thoth E13 | landed | `crates/sea-forge-thoth/`, `sea-forge ask` CLI, `Request::Ask` server variant | conformance_m11_service, conformance_m11_ask | 🟢 | high |

---

## M-ORCH — Agent orchestration (`spec-agent-orchestration.md`, status "Draft v0.1")

CURRENT_STATUS records M12-M16 (Tasks 1-18 of the 2026-07-22 spec-audit-remediation plan) as landed.

| Milestone | Source | Status | Implementation | Verification | Status | Confidence |
|---|---|---|---|---|---|---|
| M12 — AgentProvider seam + `agent_task` operation | spec-agent-orch E14/E15 | landed | `crates/sea-forge-agent/`, `sea-forge agent {list,probe,delegate,cancel}` | conformance_m12, conformance_m13 | 🟢 | medium |
| M13 — delegation + case-runner extraction + ledger replay | spec-agent-orch E15 | landed | `crates/sea-forge-case-runner/`, `crates/sea-forge-server/src/delegation.rs`, `sea-forge ledger replay --case` | conformance_m13 (18 tests) | 🟢 | high | 
| M13 — sealed summarized transcripts | spec-agent-orch (owner decision 2026-07-17) | landed | `crates/sea-forge-server/src/transcript_seal.rs` (XChaCha20Poly1305) | t16_* conformance | 🟢 | high |
| M14 — topology templates (`sequential_agents`, `concurrent_agents`) | spec-agent-orch E16 | landed | `crates/sea-forge-planner/src/templates.rs::store_builtin` | conformance_m14 (T14.1-T14.3) | 🟢 | high |
| M15 — Thoth manager loop + SoD | spec-agent-orch E16 | landed | `crates/sea-forge-thoth/src/manager.rs`, `sea-forge case manager-iterate` | conformance_m15 (T15.1-T15.5) | 🟢 | high |
| M16 — ACP driver + SWE_SEED harvesting | spec-agent-orch E17 | portable coverage green; real-host release ignored | `crates/sea-forge-agent/src/acp/`, `crates/sea-forge-server/src/swe_seed_reconciliation.rs` | conformance_m16 (12 portable tests; 2 real-host `#[ignore]`) | 🟢 portable; ⏸ real-host | high |
| Late SWE_SEED declaration reconciliation (Task 18) | spec-audit-remediation | landed | `crates/sea-forge-server/src/swe_seed_reconciliation.rs::reconcile_all_cases` (startup) + `verify_swe_seed_completion` (read-time) | t16_8_* conformance | 🟢 | high |

---

## FE — Workbench frontend (`.agents/specs/frontend/*`)

The frontend specs are a **family** of v0.1 documents; the canonical method catalog is the YAML. The workbench plan (`.agents/plans/2026-07-24-sea-forge-workbench-frontend-api-implementation.md`) defines 14 vertical-slice tasks; CURRENT_STATUS records Tasks 1-11 complete (Task 11 most recent, 2026-07-27).

| Requirement | Source | Implementation | Verification | Status | Confidence |
|---|---|---|---|---|---|
| Stack: Tauri 2 + Bun + React 19 + Vite + Astryx | ADR-004 | `workbench/apps/desktop/package.json`, `workbench/package.json` | V-07, V-08, V-10 | ✅ | high |
| SFWP transport: envelopes, negotiation, request recovery, events | Task 3 | `crates/sea-forge-server/src/sfwp/{correlation,events,precondition}.rs`, `workbench/apps/desktop/src-tauri/src/{socket,bridge,events}.rs` | conformance_sfwp.rs (hello→subscribe→kill→reconnect→cursor resume scenario) | ✅ | high |
| Application shell + 9 semantic components + G1-G9 route guards + a11y + Storybook | Task 4 | `workbench/apps/desktop/src/shell/`, `packages/sea-forge-ui-components/`, `guards/` | V-08, axe-core in Playwright | ✅ | high |
| Readiness vertical slice (read-only live data) | Task 5 | `readiness.get` (server), `useReadiness` (frontend) | conformance readiness_get_*, Playwright readiness.spec.ts | ✅ | high |
| Case authoring (draft→preflight→commit) | Task 6 | `case.entry_options`/`case.preflight`/`case.commit` (server), `caseAuthoringMachine.ts` (frontend) | conformance_case_authoring.rs (6 tests), XState machine tests (6) | ✅ | high |
| Case overview + horizon + approval list/decide | Task 7 | `case.{list,get_overview,get_horizon}`, `approval.{list,decide}` (server); `CaseHorizonPage`, `ApprovalInboxPage` (frontend) | conformance_case_views (6), conformance_approvals (7) | ✅ | high |
| Run record surface | Task 8 | `run.{list,get}` (server); `RunRecordPage` (frontend) | conformance_run_views | ✅ | high |
| Asset catalog | Task 9 | `asset.list` (server); `AssetsPage` + `AssetCatalogPage` (frontend) | conformance_assets | ✅ | high |
| Delegation preview | Task 10 | `delegation.preview` (server); `DelegationWorkbench` (frontend) | conformance_delegation_preview | ✅ | high |
| Delegation roster | Task 11 | `delegation.list` (server); `DelegationRoster` (frontend) | component tests for DelegationRoster (this commit) | ✅ | high |
| Tasks 12-14 (Thoth/Models/Memory live; Operations; Tauri bundle) | plan | not landed | — | 🔴 | high (absence) | Per CURRENT_STATUS "Still open" note |

**Catalog coverage**: 21 of 123 catalog methods implemented (17%). The catalog file is explicitly `target-unmapped`; the plan graduates methods task-by-task, so the gap is planned, not abandoned.

---

## SEC — Security and secrets (`Shell-SPEC.md §8`, AGENTS.md Safety Boundaries)

| Requirement | Source | Implementation | Verification | Status | Confidence |
|---|---|---|---|---|---|
| Only `*.enc.env` tracked under `secrets/` | Shell-SPEC §8 | `.gitignore:35` `/secrets/*.env` + `!/secrets/*.enc.env` | `git ls-files secrets/` → only `README.md`, `dev.enc.env` | ✅ | high |
| Age private key lives outside repo | Shell-SPEC §8 | `.envrc` reads `$SOPS_AGE_KEY_FILE` (default `~/.config/sops/key.txt`) | inspected this pass | ✅ | high |
| `.envrc` redacted when no key present | Shell-SPEC §8.2 | `.envrc` `load_sops_env` returns 0 with `log_status` | inspected this pass | ✅ | high |
| Network secret allowlist, never ambient | Shell-SPEC §7.2, AGENTS Safety | `SecretRequirement` typed; explicit minimal env in sandbox/runtime | negative tests in conformance suites | 🟢 | medium |
| No `unsafe_code` workspace-wide | AGENTS Rust Conventions | `Cargo.toml:37` `unsafe_code = "deny"` | `cargo check` V-02 (would fail otherwise) | ✅ | high |
| argv-based process exec, no shell | AGENTS Safety | `crates/sea-forge-runtime/`, `crates/sea-forge-agent/src/acp/` | tests + clippy | ✅ | high |
| Workspace path safe-join | AGENTS Safety | `crates/sea-forge-sandbox/src/path.rs` (safe-join) | unit tests | 🟢 | high |
| Sealed-transcript verification before shred | spec-agent-orch (owner decision 2026-07-17) | `transcript_seal::open` + `sealed_verification_failed` basis | t16_* redaction tests | 🟢 | high |

---

## CI — Foundation and CI (`Shell-SPEC.md`, `.github/workflows/`)

| Requirement | Source | Implementation | Verification | Status | Confidence |
|---|---|---|---|---|---|
| `just doctor/check/test/proof` exist | Shell-SPEC §10.2 | `justfile:30-394` | V-05, V-06 | ✅ | high |
| CI gate `CI / gate` requires lint+test+macos | `.github/workflows/ci.yml:119-161` | gate job aggregates `needs.{lint,test,macos}.result`; skipped = failure | inspect | ✅ | high |
| Conventional-commit PR title | `.github/workflows/pr-title.yml` | present | inspect | ✅ | high |
| Trusted-publishing release flow | `.github/workflows/release-please.yml` | present; bootstrap via `just publish-bootstrap` | not executed this pass | ⚪ | medium |
| Security workflow | `.github/workflows/security.yml` | present (2.5KB) | not inspected in detail | ⚪ | medium |
| Devbox pins 8 system tools | Shell-SPEC §3 | `devbox.json` | inspect | ✅ | high |

---

## GAP — Unmapped requirements

| Requirement | Source | Status | Reason unmapped |
|---|---|---|---|
| `.agents/specs/api-routes.md` | (0-byte file) | 🔴 empty | No content to map; the live API spec lives in `.agents/specs/frontend/sea-forge-workbench-api-spec-v0.1.md` (see OPEN-3) |
| `.agents/specs/datamodel.md` | (0-byte file) | 🔴 empty | Same; live datamodel lives in `crates/sea-forge-core/src/types.rs` |
| `cell.migrate` SFWP method | catalog yaml | 🔴 absent | `.agents/OBSERVED_DEBT.md` flags naming collision risk with `bundle::export/import` |
| `thoth.ask` typed response contract | CURRENT_STATUS "Still open" | 🟡 partial | Method is reachable but lacks typed response; `thoth.ask` lives in CLI `ask` and `Request::Ask` |
| `domain_model.list` SFWP method | CURRENT_STATUS "Still open" | 🔴 absent | Catalog target, not yet implemented |
| Real-host ACP/SWE_SEED/Seatbelt release evidence | spec-agent-orch §17 (M16 gate) | ⏸ skipped | Requires operator-supplied config; intentionally `#[ignore]` |
| Playwright e2e against real server | Workbench skill guidance | 🔴 absent (mocked only) | Cross-cutting decision deferred since Task 5 (OBSERVED_DEBT) |
