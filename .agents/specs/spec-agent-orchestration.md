# SEA Forge — Governed Agent Connectivity, Delegation, and Thoth Orchestration

Status: Draft v0.1

Scope: Rust (stable, edition 2021), Linux primary / macOS secondary. Extends the `spec-full.md` workspace with one new adapter crate and four new extension capabilities. Kernel crates stay synchronous; async lives only in the server/adapter layer (the `no-async-kernel` invariant of `spec-full.md` is preserved).

Purpose: Give SEA Forge the ability to employ model-backed agents as governed labor — any OpenAI-API- or Anthropic-API-compatible agent over HTTP, and any ACP-speaking CLI agent (including SWE_SEED-harnessed hosts such as Claude Code and Codex) — with every delegation authority-gated, evidenced, cancellable, and settlement-judged; and give Thoth a bounded manager loop so it can orchestrate many such agents at once through the existing governed plan-mutation path.

Owner: SEA Forge core team

Prerequisite: **`spec-adlc-thoth-minimum.md` implemented and green through M11** (which itself requires `spec-full.md` green through M8). This spec never redefines kernel types, the ledger, the authority fabric, record formats, or fail-closed rules — it extends them. Where this spec is silent, `spec-adlc-thoth-minimum.md` governs; then `spec-full.md`; then `spec-minimum.md`.

Companion inputs: `.agents/reports/2026-07-14-sk-goose-t3code-capability-delta-audit.md` (capability delta audit; Dossiers 1–4 are the acquisition rationale for E14–E17), the reserved `external_api` authority surface (`spec-minimum.md` §7.3.4), plan templates (E8), the case engine (M2a), approvals and `max_concurrent_runs` (M3), settlement declarations and `SweSeedTransport` (M4a), Thoth and the `self_disclosure` surface (E13).

Resolved decisions (owner-supplied, 2026-07-14):

- **ACP is adopted** as the protocol for CLI-resident agents. E17 is a committed capability, not a spike; its first gate still validates permission-mapping fidelity before broad rollout.
- **SWE_SEED's interface shape** (inspected at `~/projects/SWE_SEED`, commit `a55d14b`, branch `deploy-prep`): SWE_SEED is a CLI harness (`swe-seed` clap CLI, no network surface of its own) that projects policy/hooks/skills into host coding agents (Claude, Codex, OpenCode, GitHub Copilot, Antigravity, CI) via `swe-seed sync --host`, and emits route/trace/proof artifacts under `.agent-harness/`. Therefore an "SWE_SEED instance" is a **host coding agent driven via ACP (E17) with the harness projected in**; its trace/proof artifacts are harvested as evidence, and its settlement declarations continue to arrive through the existing M4a `SweSeedTransport`. SWE_SEED is unlocked by E17, not E14.
- **Transcript retention** is a config flag: `summarized` (default — bounded summary evidence plus transcript hash) or `full` (complete transcript as a content-addressed artifact). Crypto-shredding rules from `spec-full.md` §7.0c apply in both modes.

## 0. Spec Frame

1. What should be built? — Four extension capabilities in milestone order: **E14 AgentProvider seam** (a `complete`/`stream` provider trait with OpenAI-compatible and Anthropic-compatible implementations in a new adapter crate, activating the reserved `external_api` authority surface), **E15 governed `agent_task` delegation** (a new plan-item/operation kind whose execution is a dialogue with an agent endpoint, run as an ordinary governed run: turn-capped, cancellable, parallel under the existing server semaphore, transcript-evidenced, independently settled), **E16 orchestration topologies** (`sequential_agents`, `concurrent_agents` E8 templates plus the Thoth manager loop: bounded iterations of read-case → propose-discretionary-`agent_task` → await settlement), and **E17 ACP driver** (an ACP client adapter that drives CLI agents, mapping ACP permission requests onto SEA approvals and ACP sandbox modes onto `SandboxClass`, with continuation identity linking successive episodes to one external session).
2. What result should it produce? — A case plan can delegate work to N heterogeneous agents (HTTP-API agents and ACP CLI agents, including SWE_SEED-harnessed hosts) concurrently; each delegation leaves inspectable transcript evidence and settles on criteria, never on agent narration; Thoth can drive a whole case forward by proposing delegations under authority.
3. How will we know the result is real? — Milestone gates in §17. A milestone that cannot pass P1–P4b and all prior milestone gates (M0–M11) unchanged has broken the kernel and MUST be rejected.
4. What capability should get stronger after repeated use? — The installation's demonstrated ability to complete cases using delegated agent labor: capability records (E4) for `agent_task`-backed operations climb the attempted < demonstrated < proven ladder exactly like sandboxed work, and Thoth's answers about "what agents this installation can employ, and with what track record" grow from the same evidence.
5. What evidence proves the capability claim? — Transcript evidence (or content-addressed full transcripts) hash-linked into runs; settlement records with the new bases; ledger-replayable ordering of parallel delegations; ACP permission grants recorded as approval decisions; SWE_SEED proof artifacts cross-linked to the delegating run.
6. What fails safely? — A denied `agent_task` performs no network I/O and no child-process spawn; an unreachable or misbehaving endpoint yields a typed error and settlement `rejected` with transcript-so-far preserved; cancellation settles `rejected` with basis `cancelled`, never vanishes; an ACP disconnect settles the run rejected; credentials never enter ledger payloads, evidence, or errors; there is never provider fallback (an endpoint failure is a fact to record, not a routing decision to hide).
7. What must repeat until reliable? — §13: parallel dispatch under the semaphore, cancellation mid-dialogue, turn-cap breach, denial-without-side-effects, redaction, ACP permission→approval mediation, and manager-loop stall handling.
8. What changes when evidence disagrees with the design? — §5 claim table. Notably: if the ACP permission model proves lossy against SEA authority semantics (gate T16.3), E17 narrows to an allow-listed subset of ACP request kinds rather than weakening authority; if sentry-chain coordination proves insufficient for a real topology, the template vocabulary grows — an actor/message runtime is never introduced.

## Normative Language

RFC 2119 keywords as in `spec-minimum.md`. `Implementation-defined` likewise.

## 1. Problem Statement

SEA Forge solves governed execution — plan, authority, sandbox, trace, evidence, settlement — but through M11 it cannot *employ* a model-backed agent: no crate can speak to an LLM endpoint or drive a coding-agent CLI. The capability delta audit verified this absence workspace-wide; the only external-agent seam is `SweSeedTransport` (settlement declarations inbound, no task dialogue outbound).

Job-to-be-done:

- When an operator (or Thoth, acting as a sponsored R-AA actor) needs multi-agent labor applied to a case, they need SEA Forge to delegate instruction packets to SWE_SEED instances and/or any OpenAI-/Anthropic-API-compatible or ACP-speaking agent, in parallel and under authority, so that the case advances on settled evidence instead of on agent self-reports.

Current failure mode:

- Agent work happens outside the fabric entirely (a human runs Claude Code by hand), so it produces no trace, no evidence, and no settlement.
- There is no way to run N agents concurrently against one case, cancel one, and keep the rest.
- Thoth (M11) can answer questions but cannot act on a stalled case; there is no governed loop that turns its reading of the case file into proposed work.

The system is needed because ungoverned orchestrators exist (goose, semantic-kernel, t3code) but all three treat agent narration as progress; SEA Forge's corrective — settlement as the sole success arbiter — must extend to delegated work or the governance boundary ends where the agents begin.

Important boundary:

- This system is a governed delegation and orchestration layer over existing kernel machinery.
- It is not an agent framework: no conversation store as truth, no actor/message runtime, no LLM participation in kernel decisions, no prompt-layer safety in place of authority.
- Successful execution means a delegation's run settled on its criteria with transcript evidence preserved, not that the agent said it finished.

## 2. Goals and Non-Goals

### 2.1 Goals

- G1 **AgentProvider seam (E14)**: a new adapter crate `sea-forge-agent` MUST define a provider contract (`complete`, `stream`) over a typed message/tool vocabulary, with exactly two built-in implementations: `openai_compatible` (base URL + credential ref + request-shape mapping) and `anthropic`. Every provider call is a protected operation gated by the `external_api` authority surface (reserved since v0.1, activated here, deny-by-default). Typed errors; no inter-provider fallback.
- G2 **Endpoint registry (E14)**: agent endpoints are declared in configuration as **AgentEndpoint** entries and surfaced to the fabric as `ExtensionDescriptor`s of kind `runtime_adapter`, so registration, compatibility, and disclosure (via Thoth `AskAvailableAffordances`) reuse existing machinery.
- G3 **Governed delegation (E15)**: `agent_task` MUST be an additive `PlanItem` kind and `Operation` kind. Executing one is an ordinary run: authority-checked per delegation, turn-capped by a grant boundary, executed through E14 (or E17), transcript-evidenced, and settled on the item's criteria. Agent output is untrusted input (`spec-full.md` §8.6); an agent's claim of success is never a settlement input with standing (extending the M4a self-declaration rejection to delegated work).
- G4 **Parallelism and cancellation (E15)**: N `agent_task` runs MUST dispatch concurrently under the existing server `max_concurrent_runs` semaphore — no new pool. Any in-flight delegation MUST be cancellable (operator command or sentry decision) without affecting sibling runs; cancellation settles `rejected` with basis `cancelled`.
- G5 **Transcript evidence (E15)**: every delegation MUST preserve dialogue evidence per the retention mode: `summarized` (default) stores a bounded summary record plus the SHA-256 of the canonical full transcript; `full` additionally stores the transcript as a content-addressed artifact file, hash-committed to the ledger. Mode is set globally and overridable per endpoint and per plan item (most specific wins).
- G6 **Topology templates (E16)**: built-in E8 templates `sequential_agents@0.1.0` (chain of `agent_task` items linked by sentries on each predecessor's settlement) and `concurrent_agents@0.1.0` (N parallel `agent_task` items plus a rollup milestone whose sentry requires all N settlements) MUST instantiate to ordinary CasePlans with no new kernel types.
- G7 **Thoth manager loop (E16)**: Thoth MUST be able to run a bounded manager iteration against a case: read the case file and trace ledger (its progress ledger — no second ledger), decide whether the outcome is satisfied / progress is being made / what should run next, and propose the next `agent_task` as a **discretionary item** through the existing governed plan-mutation path. Iteration count is a grant boundary; exhaustion parks the case and escalates through existing approval machinery. Thoth remains an R-AA actor: it proposes, it never settles, and it MUST NOT settle or promote work it proposed (structural separation of duties, enforced as in E13).
- G8 **ACP driver (E17)**: an ACP client adapter MUST drive CLI-resident agents (Claude Code, Codex, and SWE_SEED-harnessed hosts) as `agent_task` executors: spawn/attach with continuation identity, surface ACP permission requests as SEA approval requests (never auto-grant beyond the run's grant), map the session's sandbox posture from the run's `SandboxClass`, and treat session transcripts identically to E15 evidence.
- G9 **SWE_SEED integration (E17)**: when the delegate is an SWE_SEED-harnessed host, the adapter MUST harvest SWE_SEED trace/proof artifacts (route decision, `ProofStarted`/`ProofCompleted` outputs) from the workspace as evidence cross-linked to the delegating run, and settlement declarations arriving via the M4a `SweSeedTransport` MUST be correlatable to that run.

### 2.2 Non-Goals

- No actor or message-passing runtime; the case engine (sentries, milestones, discretionary items) is the coordination substrate.
- No conversation store as a source of truth; transcripts are evidence, the ledger is truth.
- No LLM participation in kernel decisions (authority, settlement, classification); the model only ever produces untrusted payloads.
- No provider catalog beyond the two API shapes plus ACP; no OAuth flows, local inference, or provider fallback/routing logic.
- No group-chat topology, MCP surface import, cron scheduler, or checkpoint/diff review UX (audit §8 rejected/deferred list); revisit only when a real case needs them.
- No autonomous unbounded Thoth: every manager loop is sponsored, iteration-capped, and case-scoped.

## 3. Outcome Contract

### 3.1 Output Produced

- `crates/sea-forge-agent/` — the adapter crate: provider trait, `openai_compatible` + `anthropic` implementations, ACP client driver, endpoint registry glue. The only workspace location where an HTTP client and async executor for agent dialogue may appear outside `sea-forge-server`.
- Activated `external_api` authority surface rules in `sea-forge-authority` with grant boundaries: `endpoint_ref` (which endpoint), `max_turns` (dialogue cap), `token_budget` (optional cumulative cap), `max_manager_iterations` (E16 loops).
- Additive kernel vocabulary in `sea-forge-core`: `Operation`/`PlanItem` kind `agent_task`; settlement bases `cancelled`, `turn_cap_exceeded`, `agent_endpoint_error`.
- `.sea-forge/templates/sequential_agents@0.1.0.yaml` and `.sea-forge/templates/concurrent_agents@0.1.0.yaml`.
- Per delegation: a `TranscriptEvidence` record (and, in `full` mode, `.sea-forge/artifacts/transcripts/<sha256>.jsonl`), ledger-committed.
- CLI: `sea-forge agent probe <endpoint>`, `sea-forge agent list`, `sea-forge run cancel <run-id>`, `sea-forge case manage <case-id> --iterations N` (Thoth manager loop).

### 3.2 Outcome Verified

The output counts as a verified outcome only when a delegation's run settles on its declared criteria with transcript evidence whose hash verifies against the ledger commitment, and replaying the ledger reproduces the dispatch/settlement ordering of any parallel batch.

A run MUST NOT be reported as complete when the agent asserted success but criteria evaluation did not pass, when transcript evidence is missing or hash-mismatched, or when an ACP permission was exercised without a recorded approval decision.

### 3.3 Consumer and Handoff

Consumed by: case operators (via CLI/server), Thoth (manager loop reads settlements; `AskAvailableAffordances`/`AskCapability` disclose endpoints and their track record), capability promotion (E4), and downstream plan items gated by sentries on delegation settlements. Handoff is complete when the settlement record, transcript evidence ref, and (for SWE_SEED) harvested proof artifacts are all reachable from the run record. Unverifiable outcomes park the item in the existing blocked path with a typed error.

## 4. Capability Claim

After repeated use, an installation should be better able to complete multi-agent cases with predictable cost and audit: operators reuse topology templates instead of hand-built plans, and Thoth's manager loop resolves stalls without human replanning.

Proven only by: (a) the same topology template instantiated across ≥3 distinct cases with settled rollups; (b) capability records for `agent_task` operations reaching `demonstrated` through the ordinary promotion rules; (c) a stalled case advanced to settlement by manager-loop proposals alone.

Not proven by: transcript volume, agent self-reports, or successful probes without settled case work.

## 5. Evidence and Claim Discipline

| Claim | Level | Required evidence | Current evidence | Gap |
|---|---|---|---|---|
| The server semaphore suffices as the delegation concurrency cap | Evidence-backed (design) / unproven (this use) | M13 gate: N parallel `agent_task` runs respect `max_concurrent_runs` | `max_concurrent_runs` implemented and tested for sandboxed runs (M3) | prove under mixed sandboxed + agent load |
| Sentry chains suffice for sequential/concurrent topologies (no actor runtime needed) | Partially proven | M14 gate: both templates settle end-to-end incl. rollup | Case engine + E8 templates green (M2a–M2c); claim inherited from spec-full §5 | prove with real agent latencies and a parked-item path |
| ACP's permission model maps losslessly onto SEA authority/approvals | Assumption (decision: adopt ACP) | T16.3: every ACP request kind exercised in a session maps to a recorded SEA decision; no unmapped grant | goose + t3code both ship working ACP endpoints (audit §5); no SEA-side mapping exists | first E17 gate; lossy mapping ⇒ narrow to allow-listed request kinds (§0.8) |
| SWE_SEED instances are reachable as ACP-driven hosts with harvestable proofs | Partially proven | M16 gate: one SWE_SEED-harnessed host completes a delegation with proof artifacts cross-linked | SWE_SEED inspected: host projection for Claude/Codex/OpenCode/Copilot confirmed (`crates/swe-seed-core/src/adapters/`), trace/proof artifacts confirmed; no end-to-end run yet | run the M16 slice |
| Summarized transcripts are sufficient evidence for settlement audit | Assumption | variation case: a disputed settlement audited from summary + hash alone | none | if insufficient in practice, flip the default to `full` (config change, no code change) |

## 6. System Overview

### 6.1 Architecture Pattern

- Pattern: adapter crate + orchestrator extension over an existing governed run pipeline. Delegations are ordinary runs whose executor is an agent dialogue instead of an argv command.
- Reason: every invariant needed (authority, concurrency, evidence, settlement, approvals, cancellation-adjacent parking) already exists for runs; reusing the run lifecycle means E14–E17 add vocabulary, not machinery.
- Patterns intentionally not used: actor runtime (SK), session-store-as-truth (goose), event-sourcing stack (t3code — the ledger already is one, stronger), background worker pool (server semaphore suffices).

### 6.2 Main Components

1. `sea-forge-agent::provider` — the `AgentProvider` trait + `openai_compatible`/`anthropic` implementations. Inputs: instruction packet, dialogue state, endpoint config, credential ref. Outputs: typed completion/stream events, typed errors.
2. `sea-forge-agent::acp` — ACP client driver. Inputs: endpoint config (argv or attach target), run grant (sandbox class, boundaries), approval channel. Outputs: session events, permission requests (surfaced as approvals), continuation identity, transcript.
3. `sea-forge-agent::delegation` — executes an `agent_task` run: builds the instruction packet from PlanItem params, drives the provider/ACP loop under turn/token caps and the cancellation signal, emits `TranscriptEvidence`, returns a typed result for criteria evaluation. Never judges success itself.
4. `sea-forge-thoth::manager` — the manager loop (E16): reads case file + ledger, produces a `ManagerIteration` decision, proposes discretionary `agent_task` items via the existing planner path. R-AA constraints enforced structurally.

```mermaid
flowchart LR
  Op["Operator / Thoth (sponsored)"] --> Srv["sea-forge-server (semaphore, approvals)"]
  Srv --> Del["sea-forge-agent::delegation"]
  Del --> Prov["provider (openai_compatible | anthropic)"]
  Del --> ACP["acp driver (CLI agents incl. SWE_SEED hosts)"]
  Del --> Ev["TranscriptEvidence → ledger"]
  Ev --> Settle["settlement (criteria, existing pipeline)"]
  Thoth["sea-forge-thoth::manager"] -. reads case+ledger, proposes discretionary items .-> Srv
```

### 6.3 External Dependencies

- HTTP client (`reqwest` or equivalent) — E14 provider calls. Confined to `sea-forge-agent`; failure ⇒ typed `agent_endpoint_error`, run rejected.
- ACP protocol implementation (Rust ACP crate if suitable, else a minimal client over the published schema — implementation-defined, documented at M16) — E17 sessions. Failure/disconnect ⇒ run rejected, transcript-so-far preserved.
- Child-process management (existing sandbox machinery) — spawning CLI agents under a jail-class grant. Failure ⇒ existing sandbox failure semantics.

## 7. Core Domain Model — extensions only

All kernel types from prior specs are unchanged; everything here is additive under v0.2 compatibility rules.

### 7.1 AgentEndpoint (E14/E17)

Purpose: a declared, registry-visible way to reach one agent. Used by: config loader, extension registry, authority checks, delegation executor, Thoth disclosure.

- `id` (string) — stable key, `[a-z0-9_-]{1,64}`; doubles as the `endpoint_ref` grant boundary value.
- `kind` (enum) — `openai_compatible | anthropic | acp`.
- `descriptor_ref` (string) — the `ExtensionDescriptor` (kind `runtime_adapter`) registered for this endpoint.
- `base_url` (string, required for HTTP kinds) — validated absolute https/http URL. `argv` (list, required for `acp` kind) — tokenized, never shell-invoked.
- `credential_ref` (string or null) — name of an environment variable or secret-store key; the resolved value is `credential_bearing` and MUST never appear in records, logs, or errors.
- `default_model` (string, optional), `transcript_retention` (enum override, optional), `status` (enum: `declared | probed | demonstrated`) — status climbs only on settled evidence, mirroring the E13 claim ladder.

### 7.2 agent_task PlanItem parameters (E15)

Purpose: what a delegation says to do. Used by: planner validation, delegation executor, criteria evaluation.

- `endpoint_ref` (string) — must resolve to a registered AgentEndpoint; authority-checked against the grant's `endpoint_ref` boundary.
- `instruction` (string) — the task packet; rendered into the initial user message (HTTP kinds) or session prompt (ACP). Untrusted content rules apply to anything interpolated in.
- `max_turns` (integer ≥1) — required; also bounded above by the grant's `max_turns`.
- `token_budget` (integer, optional), `response_schema` (JSON schema, optional — schema-valid final output becomes a named evidence field), `transcript_retention` (enum, optional override).

### 7.3 TranscriptEvidence (E15)

Purpose: the audit record of one delegation dialogue. Used by: evidence pipeline, settlement audit, Thoth `AskEvidenceForClaim`.

- `run_id`, `endpoint_ref`, `turns_used` (integer), `termination` (enum: `completed | turn_cap_exceeded | cancelled | endpoint_error | acp_disconnect`).
- `transcript_sha256` (string) — SHA-256 of the canonical JSONL transcript (JCS-canonicalized messages, one per line), always present regardless of retention mode.
- `summary` (string, bounded ≤ implementation-defined size) — always present; deterministic structural summary (turn count, tool calls made, final-message excerpt), not model-generated.
- `artifact_ref` (string, only in `full` mode) — content-addressed transcript artifact path.
- `harvested_refs` (list, E17/SWE_SEED) — hashes+paths of harvested proof/trace artifacts.

Redaction rule: credential values and any string matching the M0 sentinel-redaction patterns are redacted from transcript content *before* hashing and storage; the hash commits to the redacted canonical form.

### 7.4 DelegationRun state (E15)

A delegation reuses the existing run lifecycle; additive fields on the run record: `agent_endpoint_ref`, `cancellation_requested_at` (timestamp or null), `continuation_key` (string or null, E17 — links successive episodes to one external ACP session).

### 7.5 ManagerIteration (E16)

Purpose: one auditable step of the Thoth manager loop. Used by: `sea-forge-thoth::manager`, ledger, escalation.

- `case_id`, `iteration` (integer, 1-based), `snapshot_refs` (case-file version + ledger head consulted).
- `judgment` (enum: `satisfied | progressing | stalled | blocked`) with `rationale_claim_refs` (grounded-claim refs per E13 — the judgment must cite evidence, not narrate).
- `action` (enum: `none | propose_item | escalate`) and `proposed_item_ref` (discretionary item id, when applicable).
- Recorded to the ledger whether or not the proposal is granted; a denied proposal is an outcome, not an error.

## 8. Configuration and Input Contract

### 8.1 Sources and Resolution

Precedence: per-plan-item params → per-endpoint config → `[agent]` section of the existing server/CLI config file → built-in defaults. Environment variables are read only through `credential_ref` indirection. Relative paths resolve against the `.sea-forge/` root. `argv` fields are tokenized argv, never shell.

### 8.2 Required Config Fields

| Field | Type | Required | Default | Validation |
|---|---|---:|---|---|
| `agent.endpoints[].id` | string | yes | none | `[a-z0-9_-]{1,64}`, unique |
| `agent.endpoints[].kind` | enum | yes | none | `openai_compatible \| anthropic \| acp` |
| `agent.endpoints[].base_url` / `argv` | string / list | kind-dependent | none | absolute URL / non-empty argv; exactly the one matching `kind` |
| `agent.endpoints[].credential_ref` | string | no | none | resolvable at preflight for HTTP kinds |
| `agent.transcript_retention` | enum | no | `summarized` | `summarized \| full` |
| `agent.default_max_turns` | integer | no | 16 | ≥1; per-item value may not exceed grant boundary |
| `thoth.manager.max_iterations_default` | integer | no | 8 | ≥1; bounded above by grant |

### 8.3 Config Error Classes

Standard taxonomy from `spec-minimum.md` §8.3 applies: `missing_config_error`, `parse_error`, `schema_error`, `unsupported_kind_error` (unknown endpoint kind), `missing_credential_error` (unresolvable `credential_ref` at preflight). Blast radius: a broken *endpoint entry* fails only work referencing that endpoint (typed, operator-visible) — other endpoints and all non-agent work continue; an unparseable `[agent]` section blocks only `agent_task` dispatch, never sandboxed runs.

### 8.4 Dynamic Reload

Follows the server's existing reload posture: reloaded endpoint config applies to future dispatches; in-flight delegations keep the config they launched with. Invalid reload keeps last-known-good with `invalid_reload_error`.

### 8.5 Startup and Preflight

Startup validates the `[agent]` section shape if present (absence is valid — installations without agent connectivity lose nothing). Per-dispatch preflight: endpoint exists and is registered, credential resolvable (HTTP kinds), grant covers `external_api` with a matching `endpoint_ref`, `max_turns` within boundary. Preflight failure parks the item with a typed error; no network I/O or spawn occurs.

### 8.6 Primary Input Contract

Inputs are `agent_task` plan items (validated at plan acceptance: §7.2 fields) and manager-loop invocations (`case_id`, iteration cap). Invalid items are rejected at planning time, not at dispatch. Duplicate dispatch of the same item follows existing run-idempotency rules. Agent responses are untrusted: schema-validated when `response_schema` is set, size-bounded (implementation-defined cap, typed `agent_endpoint_error` on breach), and never interpolated into privileged operations.

## 9. Operational Flow and State Model

### 9.1 Flow Summary

```text
plan accepted (agent_task items validated)
  → sentry activates item → preflight + authority check (external_api, endpoint_ref, max_turns)
  → server semaphore slot → delegation loop (provider or ACP; turns counted; cancellation polled;
     ACP permission requests → approval requests)
  → termination (completed | turn_cap_exceeded | cancelled | endpoint_error | acp_disconnect)
  → TranscriptEvidence committed (+ harvested SWE_SEED proofs, E17)
  → criteria evaluation → settlement (accepted | rejected + basis)
  → downstream sentries fire / manager iteration observes
```

Branches: denial ⇒ parked, zero side effects. Endpoint error mid-dialogue ⇒ transcript-so-far committed, settled rejected (`agent_endpoint_error`). Cancellation ⇒ best-effort abort (HTTP: drop request; ACP: session cancel then SIGTERM per sandbox rules), settled rejected (`cancelled`). Approval awaited (ACP permission) ⇒ existing `awaiting_approval` state, exit 5 semantics unchanged.

### 9.2–9.3 States and Transitions

Delegations reuse the existing run state machine; the only additive terminal distinctions are the settlement bases `cancelled`, `turn_cap_exceeded`, `agent_endpoint_error` (each a distinct named outcome, not collapsed into generic failure). Manager loop states: `iterating → (proposal granted | proposal denied | judgment satisfied) → done`, or `iteration cap reached → case parked + escalation`. Idempotency: re-dispatching a settled item follows existing rules (new run, prior settlement immutable); replaying the ledger reproduces dispatch order.

### 9.4 Transition Triggers

- Sentry fired — activates an `agent_task` item exactly like any other kind.
- Cancellation requested (CLI/server op) — sets `cancellation_requested_at`; the delegation loop observes it between turns and during streaming.
- ACP permission request — creates an approval request; grant/deny recorded; deny ⇒ the agent's action is refused inside the session (session continues; the run settles on criteria as usual).
- Turn/token boundary reached — terminates the dialogue, `turn_cap_exceeded`.
- Manager iteration tick — one read→judge→act cycle; never a background daemon, always an invoked, granted operation.

### 9.5 Important Nuances

- An agent that says "done" has produced *input to criteria evaluation*, nothing more; settlement may still reject.
- `turn_cap_exceeded` is not necessarily failure of the underlying goal — criteria may still evaluate the produced artifacts and settle accepted; the basis records *why the dialogue ended*, settlement records *whether the outcome held*.
- ACP deny-inside-session differs from run cancellation: a denied permission leaves the session alive; only cancellation or termination ends it.
- For SWE_SEED hosts, the *harness's* proof artifacts (not the host agent's chat) are the primary evidence; the transcript is corroboration.
- Thoth's manager judgment `satisfied` does not settle anything — it merely stops proposing; settlement authority remains where M4a put it.

## 10. Core Behavior Requirements

### 10.1 Authority and Isolation (E14/E15)

- Every provider call and ACP session MUST execute under a grant covering `external_api` with a matching `endpoint_ref`; denial MUST prevent all network I/O and process spawn (proven by test, not by review).
- The HTTP client and async executor MUST NOT appear in kernel crates; `cargo deny`-style checks (existing dependency-boundary gate mechanism) MUST enforce the crate boundary.
- Credentials MUST be resolved at call time, held only in memory, and redacted from every persisted or logged surface.

### 10.2 Delegation Execution (E15)

- The implementation MUST count turns and enforce `max_turns`/`token_budget` inside the loop, not post-hoc.
- The implementation MUST commit `TranscriptEvidence` for every terminated delegation, including failures and cancellations (transcript-so-far).
- The implementation MUST dispatch parallel `agent_task` runs through the existing semaphore and MUST NOT introduce a second concurrency mechanism.
- The implementation MUST NOT retry a failed endpoint call against a different endpoint (no fallback).

### 10.3 Topologies and Manager Loop (E16)

- Templates MUST instantiate deterministically (existing E8 determinism gate style) to plain CasePlans.
- The manager loop MUST record a `ManagerIteration` for every cycle, cite grounded claims for its judgment, propose only through the discretionary-item path, and stop at the iteration boundary with escalation.
- Thoth MUST NOT hold settlement authority over items it proposed; the SoD check is structural (same mechanism as E13's self-settlement bar).

### 10.4 ACP Sessions (E17)

- Every ACP permission request MUST map to a recorded SEA decision (approval or policy rule); an unmapped request kind MUST be denied with a typed error, never passed through.
- Session sandbox posture MUST derive from the run's `SandboxClass`; the adapter MUST NOT accept a session-proposed escalation.
- `continuation_key` MUST link resumed sessions to the same case lineage in the ledger.

### 10.5 Completion Rules

Complete only when: run settled, `TranscriptEvidence` committed and hash-verifiable, (E17) all exercised permissions have recorded decisions, (SWE_SEED) harvested proof refs resolve. Incomplete/blocked/failed whenever any of these is absent — regardless of what the agent reported.

## 11. Execution / Integration Contract

- Invocation: HTTP kinds — JSON request per the endpoint's API shape (OpenAI chat-completions or Anthropic messages), pinned request shapes with contract tests against recorded fixtures; ACP — child process (tokenized argv) or attach, ACP handshake, session per delegation episode. Timeout: per-turn and whole-run timeouts are grant/config boundaries (implementation-defined defaults, documented).
- Result: typed `DelegationOutcome { termination, turns_used, final_output?, evidence_ref }`; failure carries `error.code` (`agent_endpoint_error` subcodes: `unreachable | http_4xx | http_5xx | schema_invalid | oversize | acp_disconnect`), `error.message` (redacted), `error.recoverable`.
- Side effects allowed: network I/O to the declared endpoint; child processes and file writes only inside the run's sandbox/workspace; approval requests. Forbidden: writes outside the workspace, credential persistence, ledger writes from inside `sea-forge-agent` (evidence goes through the existing pipeline).

## 12. Evidence, Proof, and Observability

Required artifacts per delegation: `TranscriptEvidence` (always), transcript artifact (`full` mode), harvested SWE_SEED proofs (when applicable), settlement record with basis. Storage/retention: existing evidence store and ledger rules; `full` transcripts are content-addressed under `.sea-forge/artifacts/transcripts/` and subject to §7.0c crypto-shredding. Proof commands:

```text
cargo test -p sea-forge-agent                         # contract, denial, redaction, cancellation suites
sea-forge agent probe <endpoint> && sea-forge verify  # probe evidence chains + ledger verification
sea-forge ledger replay --case <id>                   # reproduces parallel dispatch/settlement ordering
```

A proof passes when the gate's expected records exist, hashes verify, and replay is deterministic; it fails on any missing record, hash mismatch, or nondeterministic replay. Required log context: `run_id`, `endpoint_ref`, `turns_used`, `termination`. Required metrics: delegations by termination basis; approval-mediated permissions count; manager iterations per case. Required events: dispatch, each approval decision, termination, settlement — all already ledger events; no new telemetry channel.

## 13. Repeatability and Variation Requirements

Variation: (1) N=5 parallel delegations with `max_concurrent_runs`=2 — ordering replayable, cap never exceeded; (2) same template against both an HTTP endpoint and an ACP endpoint; (3) `summarized` vs `full` retention on the same delegation — identical `transcript_sha256`; (4) oversize/garbage agent output — typed rejection, no crash, no unredacted spill.

Recovery: (1) endpoint dies mid-stream — transcript-so-far committed, rejected `agent_endpoint_error`, siblings unaffected; (2) cancel one of three parallel runs — cancelled settles `rejected/cancelled`, other two settle normally, rollup sentry behaves per its condition; (3) ACP disconnect and resume — `continuation_key` links the successor episode; (4) manager loop hits iteration cap on an unresolvable case — case parks, escalation recorded, no runaway.

## 14. Failure Model and Recovery Strategy

1. `agent_endpoint_error` (unreachable/4xx/5xx/schema/oversize) — fail the affected run only; typed, recoverable=true for transport, false for schema; no fallback, no automatic retry (retry is a plan-level decision via existing mechanisms).
2. `authority_denied` — park before any side effect; existing denial semantics; recovery = new grant, not code path.
3. `acp_session_failure` (spawn failure, protocol error, disconnect) — run rejected, transcript-so-far preserved, child process reaped per sandbox teardown rules (teardown failures logged, never blocking).
4. `manager_loop_exhaustion` — case parked + escalation; explicitly not an error state of Thoth.

Phase separation: preflight failures abort with zero side effects; mid-dialogue failures preserve evidence; teardown/harvest failures are logged and reduce the run to `rejected` with whatever evidence was captured — they never destroy captured evidence. On failure the system MUST NOT: silently retry against another endpoint, discard partial transcripts, or leave an untracked child process.

## 15. Security, Safety, and Trust Boundaries

- Trusted: config file, authority policy, ledger, grants, approval decisions. Untrusted: everything an agent produces (messages, tool-call requests, ACP permission requests, files written in its workspace), everything SWE_SEED artifacts contain (evidence to verify, not instructions to obey).
- Privileged operations: activating `external_api` rules, approving ACP permissions, settlement. The model/agent can request; only the fabric grants.
- Secrets: via `credential_ref` indirection only; never logged, never in evidence, never in transcripts (redaction-before-hash, §7.3); a leaked-looking string in agent output is itself redacted by the M0 sentinel pass.
- Prompt injection stance: the prompt is never the security boundary (inherited from E13). A hostile agent may say anything; it cannot exceed its grant, its sandbox class, or its approval decisions. Tests MUST include a hostile-transcript case (agent requests out-of-grant action; nothing happens beyond a recorded denial).
- Thoth SoD: manager-loop proposals are attributed to Thoth's actor identity; the settlement path structurally excludes that identity for those items.

## 16. Reference Algorithms

### 16.1 Delegation loop (E15)

```text
function run_delegation(item, grant, endpoint, cancel):
  preflight(endpoint, grant)                    # no side effects on failure
  transcript = []; turns = 0
  msg = build_instruction_packet(item)
  loop:
    if cancel.requested: return terminate(cancelled, transcript)
    if turns >= min(item.max_turns, grant.max_turns): return terminate(turn_cap_exceeded, transcript)
    resp = provider.complete_or_stream(endpoint, transcript + [msg])   # authority-gated call
    on error e: return terminate(agent_endpoint_error(e), transcript)
    transcript += [msg, resp]; turns += 1
    if resp.is_final or schema_satisfied(item.response_schema, resp): return terminate(completed, transcript)
    msg = next_turn_input(resp)                 # tool results, continuation — all sandbox-mediated

function terminate(reason, transcript):
  t = redact(canonicalize(transcript))
  commit TranscriptEvidence{ sha256(t), summary(t), artifact if retention==full, reason }
  return DelegationOutcome{ reason, evidence_ref }   # settlement happens downstream, on criteria
```

### 16.2 Manager iteration (E16)

```text
function manager_iterate(case_id, grant, i):
  require i <= grant.max_manager_iterations else park_and_escalate(case_id)
  view = read(case_file(case_id), ledger_head())          # governed reads (E13 disclosure rules apply)
  j = judge(view)                                          # cites grounded claims; deterministic given view
  record ManagerIteration{case_id, i, view.refs, j}
  match j:
    satisfied → return done
    stalled | progressing_with_gap → propose_discretionary_agent_task(...)  # may be denied; denial recorded
    blocked → escalate(case_id)
```

## 17. Test and Validation Matrix

### 17.1 Core Conformance — M12 (E14, AgentProvider seam)

| # | Test | Expected |
|---|---|---|
| T12.1 | `agent probe` against local stub (both API shapes) | schema-valid response, evidence committed, settled accepted |
| T12.2 | probe without `external_api` grant | denied; instrumented stub proves zero connections opened |
| T12.3 | credential redaction sweep | key absent from every record, log, error, transcript |
| T12.4 | contract fixtures (recorded request/response per shape) | pinned request shapes match byte-for-byte |
| T12.5 | dependency boundary | HTTP client/async absent from all kernel crates (automated check) |
| T12.6 | endpoint error taxonomy (unreachable, 4xx, 5xx, oversize) | typed subcodes; settled rejected; no fallback attempted |

### 17.2 Core Conformance — M13 (E15, governed delegation)

| # | Test | Expected |
|---|---|---|
| T13.1 | two-item case: `agent_task` → sentry-gated `sandboxed_task` | chain settles end-to-end; downstream saw only settled evidence |
| T13.2 | 5 parallel delegations, semaphore=2 | cap respected; ledger replay reproduces ordering |
| T13.3 | cancel one of three in flight | cancelled ⇒ `rejected/cancelled` with partial transcript; siblings settle normally |
| T13.4 | turn-cap breach | `turn_cap_exceeded`; criteria still evaluated against produced artifacts |
| T13.5 | agent asserts success, criteria fail | settled rejected (narration has no standing) |
| T13.6 | retention modes | same `transcript_sha256` both modes; artifact exists only in `full` |
| T13.7 | hostile transcript (out-of-grant request) | recorded denial; no side effect; run continues/settles on criteria |

### 17.3 Core Conformance — M14 (E16a, topology templates)

| # | Test | Expected |
|---|---|---|
| T14.1 | `sequential_agents@0.1.0` instantiation ×2 | deterministic identical plans; chain settles in order |
| T14.2 | `concurrent_agents@0.1.0` with rollup | rollup milestone fires only when all N settled |
| T14.3 | one branch rejected | rollup does not fire; case parks per sentry condition; no partial-success leak |

### 17.4 Core Conformance — M15 (E16b, Thoth manager loop)

| # | Test | Expected |
|---|---|---|
| T15.1 | stalled case, one iteration | `ManagerIteration` recorded, discretionary `agent_task` proposed and granted, runs, settles |
| T15.2 | proposal without authority | denied; denial recorded as the iteration's outcome; loop continues or escalates |
| T15.3 | iteration cap reached | case parked; escalation through approvals; no further proposals |
| T15.4 | SoD | Thoth identity structurally excluded from settling items it proposed |
| T15.5 | judgment grounding | every judgment cites resolvable claim refs; a judgment without evidence refs is rejected at record time |

### 17.5 Core Conformance — M16 (E17, ACP driver)

| # | Test | Expected |
|---|---|---|
| T16.1 | ACP session vs local goose (or equivalent ACP server), read-only task, jail grant | task completes; transcript evidence committed |
| T16.2 | ACP permission request | surfaced as SEA approval; grant and deny both exercised and recorded; deny leaves session alive |
| T16.3 | permission-mapping fidelity sweep | every request kind observed maps to a recorded decision; unmapped kind ⇒ typed denial (this gate is the ACP-adoption validation; lossy ⇒ §0.8 narrowing) |
| T16.4 | sandbox posture | session runs under the run's `SandboxClass`; escalation attempt refused |
| T16.5 | disconnect + resume | first episode rejected with partial transcript; resumed episode linked by `continuation_key` |
| T16.6 | SWE_SEED end-to-end | delegation to an SWE_SEED-projected host; route/proof artifacts harvested and cross-linked; `SweSeedTransport` declaration correlated to the run |

### 17.6 Regression (every milestone)

P1–P4b and all M0–M11 gates unchanged; `cargo test --workspace` green; T12.5 dependency boundary re-run.

### 17.7 Real Integration

Required once per release against one real hosted endpoint (operator-supplied credential) and one real CLI agent: skipped runs reported skipped, never passed.

## Milestone order and dependencies

| Milestone | Delivers | Depends on |
|---|---|---|
| M12 | E14 provider seam, `external_api` activation, `agent probe` | M0–M11 green |
| M13 | E15 `agent_task` delegation: parallel, cancellable, transcript-evidenced | M12 |
| M14 | E16a topology templates | M13 |
| M15 | E16b Thoth manager loop | M14 + E13 (Thoth) |
| M16 | E17 ACP driver + SWE_SEED integration | M13 (may proceed parallel to M14–M15) |

## 18. Implementation Checklist / Definition of Done

- [ ] `sea-forge-agent` crate exists; HTTP/async confined to it (T12.5 automated).
- [ ] `external_api` surface activated deny-by-default with `endpoint_ref`/`max_turns`/`token_budget` boundaries.
- [ ] `agent_task` kind additive; settlement bases `cancelled`/`turn_cap_exceeded`/`agent_endpoint_error` present.
- [ ] Transcript retention flag implemented (`summarized` default, `full` content-addressed), redaction-before-hash proven.
- [ ] Parallelism via existing semaphore only; cancellation settles, never vanishes.
- [ ] Both topology templates deterministic; rollup gate proven.
- [ ] Manager loop bounded, evidence-grounded, SoD-enforced, escalating on exhaustion.
- [ ] ACP permission→approval mapping validated (T16.3) or scope narrowed per §0.8.
- [ ] SWE_SEED slice green (T16.6) with proof harvest + settlement-transport correlation.
- [ ] All M12–M16 gates pass; P1–P4b and M0–M11 unchanged; claims table (§5) updated with actual evidence.
