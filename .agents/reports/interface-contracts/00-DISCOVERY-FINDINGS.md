# Discovery Findings Report: GodSpeed System Front End and Cognitive Environment

**Date:** 2026-09-20  
**Scope:** Complete typed application boundary discovery across SEA-Forge (`sea-rs`), Gauntlet (`gauntlet`), RealityTrace (`sxr`), Go System Front End (`apps/godspeed-casework-go`), and React Cognitive Environment (`apps/godspeed-cognitive-ui`).  
**Status:** Canonical Discovery Baseline (Completed prior to interface contract delivery).  

---

## 1. Project Boundaries & Repository Topology

### 1.1 Repository Locations & Roles
* **SEA-Forge (`/home/sprime01/projects/sea-rs`)**:
  - Rust workspace of 22 crates (19 strictly synchronous kernel crates, 2 async edge crates: `sea-forge-server` and `sea-forge-agent`, 1 CLI `sea-forge-cli`).
  - **Ownership:** Governed casework meaning, lifecycle, sentry evaluation, authority policy decisions, append-only ledger truth, and operational settlement.
  - **Key interfaces:** Unix domain socket NDJSON boundary (`sea-forge-server`), SFWP protocol v1 (`crates/sea-forge-server/src/sfwp/`), typed event log (`sea-forge-ledger`, `sea-forge-trace`), case runner (`crates/sea-forge-case-runner`).
* **Gauntlet (`/home/sprime01/projects/gauntlet`)**:
  - Rust workspace containing `gauntlet-domain`, `gauntlet-ports`, `gauntlet-app`, `gauntlet-cli`, and 26 modular adapters.
  - **Ownership:** Difficult execution, agent harness, sandboxed execution, verification, and transformation. Gauntlet executes authorized work and emits execution observations; it possesses *no* case semantic authority and its completion is *never* settlement.
  - **Key interfaces:** CLI (`gauntlet run`, `gauntlet resume`, `gauntlet verify`), `RunController`, `UnitScheduler`, `PlannedDispatch`, `gauntlet-ports` contracts (`agent_runner`, `observer`, `verifier`, `operator_surface`, `state_store`).
* **RealityTrace / `sxr` (`/home/sprime01/projects/sxr`)**:
  - Rust workspace containing `sxr-core` (pure logic, zero I/O), `sxr-ledger` (SQLite STRICT + WAL append-only ledger with dense SHA-256 hash chaining), `sxr-verify`, `sxr-df`, `sxr-git`, and `sxr-cli`.
  - **Ownership:** Epistemic anti-collapse runtime, recording relationships between expected state ($G_{declared}$) and observed state ($G_{observed}$), questions, thresholds, claims, qualifying evidence, discrepancies (residuals), CAS storage (`cas://sha256:...`), and 9-locus diagnostic attribution.
  - **Key interfaces:** CLI (`sxr snapshot`, `sxr question`, `sxr threshold`, `sxr verify`, `sxr evidence`, `sxr claim`, `sxr settlement`, `sxr query`, `sxr projections`), CEP-0008 envelope validator.
* **Go System Front End (`apps/godspeed-casework-go`)**:
  - Go module under `sea-rs/apps/godspeed-casework-go` (Go 1.27+).
  - **Ownership:** Operational coordination, claim/lease management (`WorkOpportunity`, `ExecutionLease`), reconciliation, preflight validation, projection assembly, SSE event streaming, and interaction intent routing.
  - **Current substrate:** Minimal Northstar fixture provider (`configs/fixture-serve.json`), HTTP handlers in `internal/server/server.go`, minimal port stubs in `internal/ports/ports.go`.
* **React Cognitive Environment (`apps/godspeed-cognitive-ui`)**:
  - Vite + React 19 + TypeScript 5.7+ application under `sea-rs/apps/godspeed-cognitive-ui`.
  - **Ownership:** Human-operable 2.5D spatial/temporal projection, semantic zoom, attention discipline, local selection/focus, progressive artifact presentation, and contextual agent narration choreography.
  - **Current substrate:** Contract definitions in `contracts/` (`world.ts`, `interaction.ts`, `temporal.ts`, `artifact.ts`, `agent.ts`), 2.5D canvas, fixture provider.

---

## 2. Discovery of the COMPLETE SEA-Forge Case Subsystem

### 2.1 Case Lifecycle
Implemented in `crates/sea-forge-core/src/types.rs`, `crates/sea-forge-case-runner/src/lib.rs`, `crates/sea-forge-server/src/sfwp/case.rs`, and `crates/sea-forge-cli/src/commands/case.rs`:
* **Create / Open Case:** `case_dispatch::submit` or SFWP `CaseCommit` (`crates/sea-forge-server/src/lib.rs:764-780`). Commits `Case` record and `CasePlan` to `<root>/cases/<case_id>/case.json` and `<root>/cases/<case_id>/plan.json`, mints monotonic `case_id` via `ids::case_id()` (`case-<ULID>`), appends `TraceKind::CaseCreated` and `TraceKind::PlanCreated`.
* **Identify Case:** Structured ID `case-<ULID>` (alphanumeric with dashes, length <= 128, validated by `valid_case_id` in `case_views.rs:241`).
* **Case Definition / Model Reference:** `CasePlan.template_ref` (format `name@version`, e.g. `audit-pipeline@1.0`), resolved via `<root>/templates/<name>@<version>.yaml` (`sea_forge_planner::templates`).
* **Start / Activate Case:** Case initialized in state `CaseState::Active` (`crates/sea-forge-core/src/types.rs:17`). Initial plan items evaluated via `case_engine::next_case_actions` (`crates/sea-forge-planner/src/case_engine.rs:539`). Items with satisfied entry criteria fire `CaseAction::Activate` (emitting `TraceKind::ItemActivated`) or `CaseAction::Enable` (for manual activation).
* **Suspend / Pause:** Implemented via `TraceKind::ItemTerminated` or setting `CaseState::AwaitingApproval` when an approval or human task is parked (`case_engine.rs:136`, `approvals.rs:1-30`).
* **Resume:** Executed when a pending approval is decided (`approval.decide` / `ApprovalDecide`), reopening item dispatch.
* **Close / Complete:** Evaluated by `can_auto_complete` (`case_engine.rs:434`): all required items must have `ItemStatus::Completed`, and zero items remain `Active` or `Enabled`. Emits `TraceKind::CaseClosed` and sets `CaseState::Completed` with timestamp `closed_at`.
* **Terminate / Cancel:** Triggered when a required item fails (`required_item_failed`, `case_engine.rs:452`). Emits `CaseAction::TerminateCase { blocking_item }`, appending `TraceKind::CaseTerminated` with `blocking_item` payload, setting `CaseState::Terminated` and `close_reason`.
* **Reopen / Reactivate:** Fully implemented in `crates/sea-forge-cli/src/commands/case.rs:75-109` (`reopen`). Requires case in `Completed` or `Terminated`. Authorizes against `AuthorityAction::Reserved { resource_type: "case_reopen", resource_id: case_id }`, appends `TraceKind::CaseReopened`, resets state to `CaseState::Active`, clears `close_reason` and `closed_at`, and re-enables plan evaluation.
* **Current Lifecycle States:** `CaseState` enum:
  - `Active`: Actively running or awaiting next action.
  - `AwaitingApproval`: Blocked on human decision or policy escalation.
  - `Completed`: All required work accepted and settled.
  - `Terminated`: Required work failed or stopped with blocking reason.
* **Historical Lifecycle Transitions:** Recorded sequentially in `<root>/cases/<case_id>/case-events.jsonl` and mirrored in the cell ledger `ledgers/case-<case_id>/`.

### 2.2 Case State
* **Current Case Snapshot:** Available through SFWP `case.get_overview` (`crates/sea-forge-server/src/sfwp/case_views.rs:359`), returning `CaseOverview`: `case_id`, `case_state`, `summary`, `created_at`, `closed_at`, `close_reason`, `plan_ref`, `template_ref`, `item_count`, `stages`, `settlements`, `run_ids`.
* **Case Version / Revision:** Explicit monotonic sequence: `event_id` (`cev-<seq>`), ledger `entry_ulid`, and SFWP `last_event_id` (`CaseHorizon.last_event_id`).
* **Current Plan & Items:** Read from `<root>/cases/<case_id>/plan.json` containing `CasePlan` and `Vec<PlanItem>`.
* **Stages:** `CasePlan.items` filtered where `item_kind == ItemKind::Stage`. Stages group child plan items via `PlanItem.parent_stage`.
* **Milestones:** `PlanItem` with `item_kind == ItemKind::Milestone`. Milestones cannot repeat (`case_engine.rs:290`). Attainment emits `TraceKind::MilestoneAchieved` (`case_engine.rs:526`).
* **Case Evidence & Basis:** Settled runs record `SettlementEvent` (`settlement.json`), detailing `status` (`Accepted`, `Rejected`, `Escalated`), evidence `basis` (list of evidence refs), `review_required`, and `criteria_ref`.

### 2.3 Plan & Plan-Item Behavior
* **Item Kinds (`ItemKind` in `types.rs:51-60`):**
  - `SandboxedTask`: Synchronous sandboxed execution with `Operation::WriteFile` or `Operation::ExecuteCommand`.
  - `HumanTask`: Work requiring human interaction, parked via `CaseAction::ParkHumanTask` until `HumanTaskCompleted`.
  - `Milestone`: Verification gate that achieves once entry criteria are met; cannot repeat.
  - `Stage`: Structural grouping for pipeline stages or sub-workflows.
  - `TimerListener`: Temporal event listener.
  - `UserEventListener`: Human/external event trigger.
  - `AgentTask`: Autonomous agent task carrying `Operation::AgentTask` (endpoint, instruction, max turns, token budget).
* **Plan Item Markers (`ItemMarkers` in `types.rs:62-70`):**
  - `required: bool`: If true, failure causes case termination; auto-completion blocks until completed.
  - `repetition: bool`: If true, failed instances retry up to `max_instances`; if false, `max_instances` must be 1.
  - `manual_activation: bool`: If true, transitions to `Enabled` on satisfied criteria and waits for manual activation; if false, auto-activates.
* **Sentries & Criteria:**
  - `Sentry` (`types.rs:97-101`): Composed of `SentryTrigger` (`source`: item ID or `"case"`, `event`: event name) and optional `if_predicate` (`SentryPredicate`: `ArtifactExists { path }` or `SettlementStatus { status }`).
  - `EntryCriteriaMode` (`types.rs:75-81`): `Any` (OR of sentries, default) or `All` (AND of sentries, used for concurrent-branch rollups).
  - Exit Criteria: Sentries determining when an active item or stage exits.
* **Standing & States (`ItemStatus` & SFWP `ExecutionStanding` / `SettlementStanding` in `case_views.rs:53-84`):**
  - Execution Standing: `Pending` → `Enabled` → `Active` → `Completed` | `Failed` | `Terminated`.
  - Settlement Standing: `Unsettled` → `Accepted` | `Rejected` | `Escalated`.
  - **Inviolable Invariant:** Execution standing and settlement standing are strictly disjoint. Execution exiting with status 0 is `execution: completed`, but remains `settlement: unsettled` until evaluated by authority.

### 2.4 Discretionary Work
* **Representation:** In `crates/sea-forge-cli/src/commands/case.rs:111-170` (`add_task`, `propose_item`) and `crates/sea-forge-core/src/types.rs:133-138` (`PlanItem.proposed_by`).
* **Add-to-Plan Mechanism:**
  - Item loaded or generated.
  - Appended to `CasePlan.items`.
  - Proposal validated with `validate_proposal(&mut plan)`.
  - Authorizes against `AuthorityAction::Reserved { resource_type: "discretionary_task_add", resource_id: format!("{case_id}:{item_id}") }`.
  - Commits `case_plan_mutation` ledger record.
  - Materializes new `plan.json`.
  - Appends `TraceKind::PlanMutated`.
* **Separation of Duties (SoD):** If `proposed_by` is set (e.g. proposed by Thoth manager loop or an agent), policy prevents the proposer from approving or settling its own proposed item.

### 2.5 Dynamic & Adaptive Case Behavior
* **Multiple Episodes:** `PlanItem` retried across multiple runs accumulates `run_ids: Vec<String>` and `instances: u32` (`case_views.rs:107`).
* **Reopening:** Case reopening allows adding new work, modifying plan items, or re-evaluating milestones after contrary evidence.
* **Invalidation & Mutation:** Sentry evaluations re-run dynamically on every trace event (`evaluate_sentries`). If prior assumptions fail, downstream work does not become enabled.
* **Escalation:** When settlement or authority returns `Escalated`, the item terminates without case failure, opening a `PendingApproval` in `approvals.jsonl`.

### 2.6 Case Actions & Operations
* `CaseCommit`: Protected command creating a case from template + params.
* `CaseReopen`: Protected command reopening a completed/terminated case.
* `DiscretionaryTaskAdd`: Protected command adding a plan item.
* `ApprovalDecide`: Protected command approving or rejecting an escalation.
* `Delegate`: Protected command delegating an `AgentTask` to an external agent endpoint.
* `CancelDelegation`: Protected command cancelling in-flight delegation.
* `Ask` (Thoth): Protected disclosure inquiry.

### 2.7 Human Tasks & Interaction Points
* `ItemKind::HumanTask`: Case engine pauses item in `ParkHumanTask` (`case_engine.rs:136`), awaiting `HumanTaskCompleted`.
* Approvals (`PendingApproval` in `sfwp/approvals.rs:76-99`):
  - Enumerated via `approval.list`.
  - Carries `ApprovalGovernanceContext`: `approval_source`, `decision_source`, `reason`, `matched_rule`, `policy_refs`, `boundary_constraints`, `requester`, `operation_kind`, `purpose_context`, `evidence_refs`, `eligible_actors`.
  - Resolved via `approval.decide` (`decision: "approve" | "reject"`).

### 2.8 Role & Authority Boundaries
* Defined in `crates/sea-forge-core/src/types.rs:297-358`:
  - `Operator`: Interactive user executing standard commands.
  - `Agent`: Automated software agent performing bounded tasks.
  - `DataSteward (R-DS)`: Manages data models, datasets, and privacy policies.
  - `AgentGovernor (R-AG)`: Authorizes agent deployments, prompts, and tool access.
  - `LifecycleCustodian (R-LC)`: Reopens cases, manages archiving, case disposal.
  - `SecurityOfficer (R-SO)`: Resolves security escalations and policy violations.
  - `RiskManager (R-RM)`: Resolves financial, compliance, or risk escalations.
  - `Developer (R-DEV)`: Proposes code changes, plans, and technical artifacts.
  - `AutomatedAgent (R-AA)`: Autonomous execution identity.
  - `Service` / `System`: Background orchestrator.
* Authority Engine (`sea-forge-authority`): Default DENY. `PolicyAuthorityEngine.evaluate` returns `AuthorityDecision` with `verdict`: `Allow`, `Deny`, `Escalate`. An `ActionGrant` is minted *only* on `Allow`.

### 2.9 Evidence & Settlement
* `SettlementCriteria`: Declared on each plan item (`SettlementCriteria` in `types.rs:115`).
* `SettlementEvent`: Commits outcome `Accepted`, `Rejected`, or `Escalated` with evidentiary `basis` (hashes, artifact paths, test logs).
* Immutability: Historical settlement records in `settlement.json` and ledger streams can never be overwritten or muted.

### 2.10 Case Events
* Full trace events enumerated in `TraceKind` (`types.rs:501-526`): 24 event kinds spanning creation, activation, execution, artifacts, milestones, approvals, mutations, reopening, and termination.
* Monotonic event sequence: `event_id` and `dispatch_ordinal` / `settlement_ordinal` (`case_runner/src/lib.rs:43-52`).

### 2.11 History & Version Access
* On-disk records: `<root>/cases/<case_id>/case.json`, `plan.json`, `case-events.jsonl`, `ledgers/case-<case_id>/stream.jsonl`.
* SFWP inspect methods: `case.list`, `case.get_overview`, `case.get_horizon`, `events.get_range` (bounded replay with `from_cursor`, `to_cursor`, `limit`).

---

## 3. Discovery of Gauntlet as Governed Execution Subsystem

### 3.1 Architecture & Role
* Gauntlet (`/home/sprime01/projects/gauntlet`) is the **Execution Environment** role in the GodSpeed architecture (`docs/explanations-and-references/goodspeed-loop.md:68`).
* Governed loop:
  $$\text{Reference} \to \text{Transform} \to \text{Observe} \to \text{Compare} \to \text{Verify} \to \text{Select} \to \text{Correct} \to \text{Settle} \to \text{Remember}$$
* Execution Controller: `gauntlet_app::control::controller::RunController`.
* CLI: `gauntlet run "<intent>"`, `gauntlet resume <run_id>`, `gauntlet verify`, `gauntlet deliver`.
* Single Writer Lock: `crates/gauntlet-cli/src/writer_lock.rs` enforces exclusive execution per state directory (`writer.lock`).

### 3.2 Integration Boundary with SEA-Forge (Convergence Edges E5A & E5B)
* **Edge E5A: `AuthorizedInvocation` (`sea_forge` → `execution_environment`)**:
  - Emitted only when SEA-Forge authority issues an `Allow` verdict.
  - Payload fields: `work_request_id`, `invocation_id`, `authority_decision_id`, `operation`, `execution_constraints`, `domain_model_ref`.
  - Invariant: Gauntlet cannot self-assert authority; it only executes work authorized by SEA-Forge.
* **Edge E5B: `ExecutionObservation` (`execution_environment` → `sea_forge`)**:
  - Emitted by Gauntlet upon run completion or failure.
  - Payload fields: `work_request_id`, `invocation_id`, `authority_decision_id`, `execution_status` (`completed`, `spawn_failed`, `timed_out`, `sandbox_violation`), `observed_effects`, `evidence_refs`.
  - Invariant: Gauntlet reporting `execution_status: completed` does **not** settle the case.

### 3.3 Handoff Back to SEA-Forge (Convergence Edge E6)
* **Edge E6: `OperationalSettlement` (`sea_forge` → case consequence)**:
  - SEA-Forge compares the `ExecutionObservation` against the declared `SettlementCriteria`.
  - If all criteria are satisfied by evidence, emits `OperationalSettlement` with `operational_settlement_status: accepted`.
  - Only then does the plan item advance to `SettlementStanding::Accepted` and milestone achieve.

---

## 4. Discovery of RealityTrace (`sxr`) as Observation, Trace & Temporal Subsystem

### 4.1 Architecture & Role
* SXR is the cryptographic audit, observation, and settlement engine in `/home/sprime01/projects/sxr`.
* Implements **Common Event Protocol (CEP-0008)** with 26 closed record kinds.
* Storage: SQLite with `STRICT` tables and `WAL` mode (`ledger.db`) + Content-Addressable Storage (`cas://sha256:<hex>`).
* Cryptographic Hash Chain: Dense SHA-256 link: $h_n = \text{SHA256}(\text{canonical\_json}(\text{row}_n))$.

### 4.2 Projections & Dual Graphs
* $G_{declared}$ (`sxr_decl_*`): The graph of declared concepts, applications, expectations, and questions derived from `.sea` models and pre-execution contracts.
* $G_{observed}$ (`sxr_obs_*`): The graph of evidenced observations, test results, and verified facts.
* Residuals: Discrepancies between $G_{declared}$ and $G_{observed}$ are explicitly computed (`residual.computed`).
* 9-Locus Diagnostic Attribution (`diagnostic_attribution.rs`): Identifies whether a discrepancy stems from:
  1. `verifier` (broken test/linter)
  2. `environment` (flaky OS/dependencies)
  3. `precondition`
  4. `execution`
  5. `observation`
  6. `evidence_derivation`
  7. `threshold`
  8. `settlement`
  9. `declared_model` (upstream model error)

### 4.3 Temporal & History Navigation
* SXR provides exact time-travel: because every row has a gapless `append_ordinal` and dense parent hash, $G_{declared}$ and $G_{observed}$ can be projected at any historical ordinal $k$.
* The Go System Front End correlates SXR's temporal checkpoints with SEA-Forge trace events to project a coherent developmental history.

---

## 5. Cross-System Identity Model

The complete correlation chain linking the four systems is:

| Concept | Owning System | Canonical ID Format | Wire / Schema Field |
|---|---|---|---|
| **Case** | SEA-Forge | `case-<ULID>` | `case_id` |
| **Case Plan** | SEA-Forge | `plan-<ULID>` | `plan_id` |
| **Governed Work Item** | SEA-Forge | `item-<name>` / string | `plan_item_id` |
| **Work Opportunity** | Go Front End | `opp-<ULID>` | `opportunity_id` |
| **Execution Lease** | Go Front End | `lease-<ULID>` | `lease_id` |
| **Authorized Invocation** | SEA-Forge (E5A) | `inv-<ULID>` | `invocation_id` |
| **Gauntlet Run** | Gauntlet | `run-<ULID>` / ULID | `gauntlet_run_id` / `run_id` |
| **Gauntlet Unit** | Gauntlet | `unit-<name>` | `unit_id` |
| **RealityTrace Question** | RealityTrace (`sxr`) | `q-<ULID>` | `question_id` |
| **RealityTrace Claim** | RealityTrace (`sxr`) | `clm-<ULID>` | `claim_id` |
| **RealityTrace Observation** | RealityTrace (`sxr`) | `obs-<ULID>` | `observation_id` |
| **RealityTrace Evidence** | RealityTrace (`sxr`) | `evi-<ULID>` | `evidence_id` / `evidence_ref` |
| **Artifact** | SEA / SXR / CAS | `cas://sha256:<hex>` or `art-<ULID>` | `artifact_ref` |
| **Authority Decision** | SEA-Forge | `dec-<ULID>` | `authority_decision_id` |
| **Settlement** | SEA-Forge (E6) | `set-<ULID>` | `settlement_id` |
| **Projection Cursor** | Go Front End | Monotonic integer (`1, 2, ...`) | `cursor` / `liveCursor` |
| **GitHub PR / Commit** | GitHub | `#123` / `sha256` | `pr_number`, `commit_sha` |

---

## 6. Repository & GitHub Surface

* **Facts vs Meaning:** GitHub owns repository facts (PR open, CI check passed, commit pushed, PR merged). SEA-Forge owns what those facts mean to governed casework.
* **Current Implementation:**
  - `AuthorityAction::GithubPr` in `crates/sea-forge-core/src/types.rs:223`.
  - `GithubPrSurface` in `crates/sea-forge-authority/src/lib.rs:541`.
  - `RepositoryPort` in `apps/godspeed-casework-go/internal/ports/ports.go:118` (`Proposal(ctx, id) (ChangeProposal, error)`).
  - No direct SFWP GitHub method exists; the Go application adapts repository facts through `RepositoryPort`.

---

## 7. Existing Go Boundary & Missing Seams

### Current State in `apps/godspeed-casework-go`:
* Implements a Northstar fixture provider:
  - `internal/ports/ports.go`: Declares `AuthorityPort`, `ExecutionPort`, `RepositoryPort`, `ArtifactStore`.
  - `internal/server/server.go`: HTTP server on `127.0.0.1:4179` with `/api/healthz`, `/api/world`, `/api/time`, `/api/events`, `/api/intents`, `/api/artifacts`.
  - `internal/coordinator/`: Basic in-memory intent dispatch.
  - `internal/projection/`: In-memory fixture projection store.
* **Gaps Identified:**
  1. `AuthorityPort` in Go only exposes `ListCases`, `Settlement`, `History`. It lacks `CreateCase`, `ReopenCase`, `AddDiscretionaryWork`, `GetHorizon`, `ListApprovals`, `DecideApproval`, `GetReadiness`, `GetIdentity`.
  2. `ExecutionPort` in Go only exposes `Execute(ExecutionRequest) (ExecutionObservation, error)`. It lacks `Resume`, `Cancel`, `Status`, `InspectDiagnostics`.
  3. `TraceObservationPort` does not exist in Go: RealityTrace questions, claims, dual-graph comparisons, and 9-locus diagnostic attribution are unmapped.
  4. Role awareness: Go currently ignores caller role and serves a single static fixture projection.

---

## 8. Existing React-Facing Contracts

### Current State in `apps/godspeed-cognitive-ui`:
* Contracts in `contracts/`:
  - `world.ts`: `WorldSnapshot`, `WorldObject` (id, kind, label, position, salience, parentId, note, attention), `WorldRelationship`, `WorldSurface`.
  - `interaction.ts`: `InteractionIntent` (kind: `focus-object`, `resolve-object`, `inspect-artifact`, `request-explanation`, `propose-consequence`, `decide-approval`, `persist-artifact`), `IntentOutcome`.
  - `temporal.ts`: `TemporalPosition`, `TemporalWindow`, `TemporalAdapter`.
  - `artifact.ts`: `ArtifactRef`, `DisclosureLevel` (`minimal`, `summary`, `source`), `ArtifactAdapter`.
  - `agent.ts`: `AgentRequest`, `NarrationBeat`, `NarrationDirective`, `NarrationStream`.
* **Strengths:** Cleanly decoupled from backend CMMN machinery; human-facing language; progressive disclosure.
* **Gaps:** Needs expansion to support the full role-aware action surface, explicit case lifecycle interactions (`reopen`, `add-discretionary`, `escalate`), RealityTrace discrepancy display, and robust reconnect/resync protocols.

---

## 9. Comprehensive System Ownership Matrix

| Surface / Capability | Authoritative Owner | Secondary / Consumer | Forbidden Roles |
|---|---|---|---|
| **Case Lifecycle & State** | **SEA-Forge** (`sea-forge-server`) | Go (Coordinates), React (Renders) | Gauntlet, React cannot mutate directly |
| **Sentry & Plan Evaluation** | **SEA-Forge** (`sea-forge-planner`) | Go (Inspects) | Neither Go nor React evaluates sentries |
| **Role & Authority Decisions** | **SEA-Forge** (`sea-forge-authority`) | Go (Applies to projections) | React never makes authority decisions |
| **Discretionary Work Addition** | **SEA-Forge** (`case_dispatch` / CLI) | Go (Routes intent) | Unrecorded additions forbidden |
| **Execution Allocation & Run** | **Gauntlet** (`gauntlet-app`) | Go (Leases & tracks) | Gauntlet never grants authority |
| **Observation & Evidence Chains**| **RealityTrace** (`sxr-core`/`ledger`)| Go (Assembles history) | Telemetry never equals evidence |
| **Repository Facts** | **GitHub** | Go (`RepositoryPort`) | GitHub never determines case settlement |
| **Operational Coordination** | **Go System Front End** | React (Receives), SEA (Emits to)| Go never overrules SEA Forge authority |
| **Lease & Claim Management** | **Go System Front End** | Gauntlet (Worker), SEA (Informed) | Leases never grant semantic authority |
| **Cognitive Projection Assembly**| **Go System Front End** | React (Presents) | Projections never mutate source records |
| **Scene Layout & Camera** | **React Cognitive Environment** | User (Controls) | Backend never owns camera state |
| **Local Focus & Arrangement** | **React Cognitive Environment** | User / Agent directives | Local focus never mutates case data |
| **Agent Narration Choreography** | **React Cognitive Environment** | Agent Adapter (Streams) | Narration beats never bypass UI actions |

---

## Summary of Completed Discovery

Discovery is complete. All 9 criteria have been thoroughly established against real repository source code across `sea-rs`, `gauntlet`, `sxr`, `apps/godspeed-casework-go`, and `apps/godspeed-cognitive-ui`.

Proceeding to create the complete suite of deliverables in `/home/sprime01/projects/sea-rs/.agents/reports/interface-contracts/`.
