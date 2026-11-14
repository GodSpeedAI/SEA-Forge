# Recon 01: SEA-Forge / Casework

Read-only, 2026-11-12. Tags: OBSERVED (saw code/test/doc) or INFERRED. Paths relative to `/home/sprime01/projects/sea-rs` unless absolute.

## 1. Identity

- Primary: `/home/sprime01/projects/sea-rs`, branch `docs/egm-bootstrap-readmes`, HEAD `b072ec50`, remote GodSpeedAI/SEA-Forge. OBSERVED.
- Dirty (3): `M AGENTS.md` (uncommitted 42-line graft block), `M .agents/skills/graft/SKILL.md`, `?? .agents/reports/fde-convergence-2026-11-12/` (orchestrator output).
- origin/main = `595b5270` (release 0.4.1, #90). Local `main` = `c81f6307`, 15 behind / 0 ahead.
- Counts: `origin/main...<branch>` (behind/ahead). Squash merges distort "ahead"; verify on main first.

| Group | Branches (behind/ahead vs origin/main) | Worktree |
|---|---|---|
| Primary | docs/egm-bootstrap-readmes 15/2 (b072ec50) | primary, 3 dirty |
| Clean, ready-ish | feat/horizon-desk-support 0/5 (6e8b6aff) ; fix/server-ambiguous-declare-keeps-lease 7/2 (961dd478) | sea-rs-horizon ; sea-rs-fix |
| Clean, detached | 4e67dfca (ancestor of origin/main, OBSERVED) | gsr-heads/sea-forge |
| Cross-platform sandbox/CI (predate #67) | feat/xplat-consolidate 17/48; feat/xplat-win-sandbox-tests 17/49; feat/xplat-hardening 17/25; feat/openshell-capability 17/16; feat/macos-app-sandbox-jail 17/16; feat/native-packages 17/19; feat/arm64-jetson-packaging 15/10; feat/hardening-capability-harvest 15/36; ci/openshell-live 17/9; ci/openshell-windows 17/4; ci/macos-app-sandbox-probe 17/3; ci/release-provenance 22/1 | _wt/* (all clean) |
| Review remediation (sandbox/Windows) | fix/review1-sandbox 17/83; fix/review1-windows 17/79; fix/review2-sandbox 17/99; fix/review2-windows 17/101 | _wt/r1osb, r1win, r2s2, r2w2 (clean) |
| Settlement/identity fixes | fix/settlement-attempt-claim-settlement 11/22; fix/settlement-attempt-claim-settlement-main 10/1 (dup subject); fix/settlement-orphan-guard 2/2; fix/restore-release-attestation 20/1; security/identity-binding-version 13/1; test/settlement-class-fail-closed 3/2; fix/b12-seed-cell-and-backup 19/1; fix/b08-domainforge-engine-fail-closed 18/3; fix/go-jose-4.1.6 12/1 | none |
| Case task approval rename | t1/case-task-approval 6/1; t1/rename-case-task-approval 7/2 (dup); t1/contracts-home 7/1 | none |
| main | main 15/0 (c81f6307, release 0.3.0) | none |

- Worktrees: all 16 `_wt/*` plus gsr-heads and sea-rs-fix/horizon are clean. OBSERVED.
- Open PR hints: CEP Stage 11 PRs (SEA-Forge #9 base `casework/live-wiring`, RealityTrace #5) were OPEN on 2026-10-05 (`.agents/reports/cep-world-migration/final-report.md`). Current state unknown; no `gh` run.

## 2. Role and authoritative ownership

Owns: "authorize every side effect before execution, then trace, evidence, settle, and record it. Process exit is not success unless settlement accepts the declared outcome" (`AGENTS.md` s1).

| Owns | Where |
|---|---|
| Case state, plan, ledger per case (`cases/<case_id>/case.json`, plan.json, evidence.jsonl, settlement.json, authority.json) | `crates/sea-forge-case-runner/src/lib.rs:366`, `case_ops/mod.rs:48,114` |
| Authority decisions, approvals, CEP verbs | `crates/sea-forge-authority/src/cep.rs:1-14`, `lib.rs` |
| Settlement (Accepted/Rejected/Escalated), criteria, declarations | `crates/sea-forge-settlement/src/declaration.rs`; `crates/sea-forge-core/src/types.rs:836` |
| Evidence | `crates/sea-forge-core/src/types.rs:644` |
| World registry, world_ref binding (recomputes from source) | `crates/sea-forge-domainforge/src/world.rs:102`; cep.rs:186,259 |
| ACP delegation, sessions; sandbox backends | `crates/sea-forge-agent/src/`, `crates/sea-forge-server/src/delegation.rs`, `crates/sea-forge-sandbox/src/` |

Does not own: semantic meaning and settlement are SEA Forge's, Gauntlet executes, the Go gateway "coordinates and projects" and "never becomes the semantic authority" (`apps/godspeed-casework-go/README.md`). OBSERVED.

## 3. Implemented capabilities

| Capability | Evidence | Tests | Status |
|---|---|---|---|
| Case verbs: submit, add_item, advance, terminate, reopen, commit, get_overview | `crates/sea-forge-server/src/sfwp/case_mutations.rs`; `case-runner/src/case_ops/` | `crates/sea-forge-server/tests/{sfwp_case_mutations,conformance_lifecycle,conformance_case_views}.rs` | IMPLEMENTED (kernel) |
| Live casework UI -> Go gateway -> SFWP -> kernel, full lifecycle, 3 runs | `.agents/reports/casework-live-wiring/delivery-summary.md`; `.agents/evidence/casework-live-wiring/T12/` | gate-live-run{1,2,3}.log; `independent-confirmation.md` = CONFIRM WITH CONDITIONS | IMPLEMENTED, E2E evidence (2026-10-05) |
| CaseState {Active, AwaitingApproval, Completed, Terminated} | `crates/sea-forge-core/src/types.rs:16-23` | planner tests | IMPLEMENTED |
| CaseAction {Enable, Activate, AchieveMilestone, ParkHumanTask, CompleteCase, TerminateCase} | `crates/sea-forge-planner/src/case_engine.rs:132` | planner tests | IMPLEMENTED |
| ApprovalState {Pending, Approved, Rejected, Expired, Used}; two-person sign-off | `crates/sea-forge-authority/src/cep.rs:689` | `conformance_approvals.rs` | IMPLEMENTED; approved escalation does not resume case (CW-45) |
| CEP authority loop (request -> world check -> PolicyAuthorityEngine -> committed decision) | cep.rs:1-14; `transitions()` cep.rs:859-905 | `conformance_identity.rs`; cep stage tests | IMPLEMENTED (Stage 6); Stage 11 PR open at last report |
| World transitions (Synthesize/Productize/Capitalize; approval-gated) | cep.rs:859; `crates/sea-forge-artifact-ip/src/lib.rs:32` | `conformance_m8_artifact.rs` | IMPLEMENTED (Stage 10) |
| Settlement + criteria provenance | `types.rs:836`; `settlement/src/declaration.rs` | `crates/sea-forge-settlement/tests/criteria_provenance.rs` | IMPLEMENTED; claim-settle fix only on branches |
| Escalation durable wait (Stage 7) | `.agents/DEBT.md` M-10 | none cited | IMPLEMENTED, "mostly closed" |
| Identity/delegation (on_behalf_of, identity_* refusals) | `sea-forge-server` identity gate; `apps/godspeed-casework-go/internal/adapters/sfwp/authority.go` | `conformance_identity.rs`, `live_identity_test.go` | IMPLEMENTED |
| Delegation list/preview/cancel, agent_run.delegate | `sea-forge-server/src/delegation.rs` | `conformance_delegations.rs`, `conformance_delegation_preview.rs` | PARTIAL: preview reports authority, cannot dry-run (OBSERVED_DEBT) |
| ACP agent sessions: permission mediation, transcript sealing, deny-all default | `crates/sea-forge-agent/src/{acp,network}.rs` | in-crate tests (in-memory transports) | IMPLEMENTED; interrupt/cancel in `server/src/{delegation,case_dispatch}.rs` |
| Resume and failed-resume settlement | `server/src/delegation.rs` (LedgerReadResume, ResumeOutcome) | `fix/settlement-*` branches | PARTIAL: fix not on main |
| Sandbox: jail, isolated, local, OpenShell, workload identity | `crates/sea-forge-sandbox/src/*.rs` | per-platform CI branches | PARTIAL: Linux-first; Windows/macOS/OpenShell live on unmerged branches |
| Thoth (ADLC Q&A, grounded claims, recorded authority decisions) | `crates/sea-forge-thoth/src/{engine,service,protocol}.rs` | thoth tests | IMPLEMENTED (kernel); `/thoth` UI page |
| Full ADLC orchestrator / agent orchestration | `.agents/specs/spec-agent-orchestration.md`, `spec-adlc-thoth-minimum.md` | none | SPEC-ONLY (M13/M14 plans only) |

## 4. Interfaces and contracts

Identifiers:
- Case stream id `case-{case_id}` (`case-runner/src/case_ops/mod.rs:114`; `cli/src/commands/resume.rs:180`). case_id comes from caller-supplied `plan.case_id`; no generator found, so grammar is UNKNOWN (ID grammar changes need approval per AGENTS.md).
- Approval id `approval_{sequence}` (`authority/src/lib.rs:2693`).
- CEP keys: `cep:{operation_id}`, `cep-approval-resolve:{id}`, `cep-approval-use:{id}`, `cep-evidence:{envelope_id}`, `cep-settle:{operation_id}:{set_hash}` (cep.rs:749-757, 1622, 1888). Evidence refs `cep:{envelope_id}`, `settlement-criteria:{sha256}` (cep.rs:988,1029).
- decision_id unique only within one operation (cep.rs ~889 comment).
- world_ref = `world:<name>@sha256:<digest>`, digest = DomainModelIdentity::canonical_digest(); changes on every source edit and release (`cep-world-migration/final-report.md`; DEBT M-04).
- ActorRole (ledger-serialized): Operator, Agent, Service, System, R-DS, R-AG, R-LC, R-SO (`crates/sea-forge-core/src/types.rs:355`). No Gateway variant (`.agents/OPEN_QUESTIONS.md`).

SFWP verbs (string literals in `crates/sea-forge-server/src/`): system.hello/describe/get_schema; readiness.get; identity.get; case.list/json/submit/commit/add_item/advance/terminate/reopen/preflight/get_overview/get_horizon/entry_options; run.list/get/begun; approval.list/decide/approve/reject; evidence.json/jsonl/created; settlement.json; authority.request/settle/evidence/approval/closure/json; delegation.list/preview; agent_run.delegate/cancel_delegation; agent.probe; thoth.ask; events.subscribe/unsubscribe/get_range; artifact.get; asset.list. Go mirror: `apps/godspeed-casework-go/internal/adapters/sfwp/{client,authority}.go`, golden frames `testdata/frames.json`.

Go gateway HTTP (`internal/server/`): /api/{healthz,readyz,metrics,session,auth/*,intents,events (SSE),ask,trajectory,artifacts,templates,world,time}. Session required except health and auth; POSTs need CSRF.

Config: JSON, unknown keys rejected; env `GODSPEED_CELL_ROOT`, `GODSPEED_EVIDENCE_ROOT`, `GODSPEED_CAPABILITY_<NAME>_{ENDPOINT,CREDENTIAL,REQUIRED}`; credentials only `env:`/`file:` indirections (`internal/config/README.md`). Serve exits 2 if a required capability is not ready. I did not open config values.

## 5. Cross-system integrations

Source map: `.agents/reports/godspeed-runtime-integration-e2e-recon.md` (s1, s4.4, s10) and cep-world final report.

| System | Mechanism (direction) | E2E evidence |
|---|---|---|
| DomainForge (.sea) | crate `sea-forge-domainforge =0.19.0` (in) ; sxr uses CLI `envelope --emit cep` | unit/conformance only; goldens embed version (M-03); b08 engine fail-closed unmerged |
| CEP | envelopes authority_request/decision | in-repo Stage 6 tests; Stage 11 PR open |
| Cognate | operation choke point speaks CEP to SEA-Forge | live loop tests skip without prerequisites (M-14); flaky (M-15) |
| RealityTrace (sxr) | observes, consumed via CEP records | sxr proof/ablation; not invoked by GodSpeed loop |
| Gauntlet | ExecutionPort target; only `gauntlet doctor` implemented | none at runtime |
| SWE_SEED | SEA-Forge CLI hook `swe_seed_reconciliation.rs`; federation envelopes | partial |
| Context Kernel | SWE_SEED <-> CK over MCP stdio | strongest edge, tests both sides (recon s10) |
| Horizon | `CaseHorizon`, `case.get_horizon` | see recon 02 (not read) |
| godspeed_agent (GSA) | settlement authority, consumes EvidenceRecorded | G-1: no production producer of EvidenceRecorded in sea-rs; synthetic only |

## 6. UX surfaces

A. SEA Forge Workbench: `workbench/` (Bun), app `workbench/apps/desktop` (React 19, TanStack Router/Query, XState, Astryx `@astryxdesign/*`, `@sea-forge/{astryx-theme,ui-components,ui-tokens,contracts}`, three, ajv).
- Routes (`src/router.tsx`): `/`, `/admin`, `/artifacts`, `/assets`, `/capabilities`, `/cases`, `/cases/new`, `/delegate`, `/evidence`, `/federation`, `/inbox`, `/memory`, `/models`, `/operations`, `/readiness`, `/runs/$runId`, `/thoth`.
- Tokens from `.agents/specs/frontend/colors_and_type.css` (drift-checked); SFWP contracts generated.
- AG-UI: zero code references; 11 docs only. SPEC-ONLY.
- Tauri: README says shell removed 2026-10-13, but 28 desktop src files reference tauri and `@tauri-apps/api` is a dependency. CONTRADICTION.

B. GodSpeed Casework cognitive UI: `apps/godspeed-cognitive-ui` (React, three, Inter, IBM Plex). DOM + WebGL scene, narrative beats, artifacts, camera. Adapter `src/adapters/http/httpCaseworkAdapter.ts`. Playwright e2e (9 files in `e2e/`). `dist/` exists locally, not tracked.

## 7. Unfinished work

- Primary: CURRENT_STATUS rev 183 next move: commit drift fixes; triage 3 items (Vite boilerplate in workbench desktop README; crates/AGENTS.md says 22 crates vs 25; stray empty dir under godspeed-casework-go).
- fix/review1-*, fix/review2-* (79-101 ahead): sandbox/Windows review remediation; likely superseded by feat/xplat-*. Verify on main first.
- Cross-platform branches predate PR #67 (merged 0fc236f3 per memory). Verify overlap.
- Duplicates: settlement-attempt-claim (two branches, 749a2173 vs ccac67a2); case-task-approval rename (t1/*). Pick one.
- feat/horizon-desk-support (5 ahead, 0 behind, clean): closes blind-critic findings; likely PR-ready.
- fix/server-ambiguous-declare-keeps-lease: uncertain-claim hold; status unknown.
- fix/b08: ADR-001 status for unwired DomainForge authority; ancestor-related to merged #66.
- OBSERVED_DEBT open items: kill -9 JSONL prefix test fails on loaded host; accept-loop resilience untested; Readiness Playwright never hits real server; events.get_range has no tail read; `ReadinessView.overall: "stale"` never derived; `agent_list` hardcoded `declared`; delegation.preview costs full asset scan.
- DEBT.md (CEP): M-02 Dependabot 826 alerts; M-06 world registry in-memory; M-07 world not required outside CEP path; M-13 governance off by default; M-14/M-15 test gaps.
- OPEN_QUESTIONS: WSL host lacks newuidmap/newgidmap; Stage A launcher/credential integration cannot qualify. Selected-cell question has an "approved" note but entry still present.
- TODO/FIXME: zero in Rust crates and Go app. Not a completeness signal.

## 8. Contradictions, duplication, risks

1. Tauri removed (docs) vs live Tauri references in desktop (section 6A).
2. Two casework UIs (Workbench desktop, cognitive UI) plus Go gateway; product surface unclear from repo alone.
3. Docs claim unwired links: GSA ProofCompleted handler absent; SEA-Forge emits no EvidenceRecorded (recon s4.4). Settlement across the stack tested only with synthetic events.
4. Gen-1 Python loop vs gen-2 with incompatible envelopes (recon item 4).
5. world_ref churns per release/edit (M-04): persisted refs break on upgrade.
6. Identity fallback: domain_model_hash falls back to a constant when manifest absent (recon item 5). Fail-open in an identity path.
7. Duplicate branches: merge-order and migration-collision risk.
8. Do not delete branches without a content check (memory rule).
9. Sandbox isolation not proven on host: OPEN_QUESTIONS says neither NoNewPrivs nor namespace probes prove distinct host UIDs or credential isolation.
10. CURRENT_STATUS.yaml (1.98 MB) latest rev 183 (2026-10-30) predates the 2026-11 branches.

## 9. Unknowns

- PR and CI state for 2026-11 branches (no `gh`).
- Whether Workbench Tauri references are dead or live (`src/machines/caseAuthoringMachine.ts`, `e2e/tauriMock.ts`).
- Full SFWP payload schemas (`sfwp/testdata/frames.json`, `sea-forge-server/src/sfwp/mod.rs`).
- Concrete case_id/run_id generator (see planner `case_engine.rs`, CLI case commands).
- Cognate Workbench repo: not in this checkout (see recon 02). Whether fix/b12 and fix/restore-release-attestation landed.
