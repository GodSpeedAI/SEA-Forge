# Capability Delta Audit — semantic-kernel, goose, t3code → SEA Forge (Thoth orchestrator lens)

Consolidated audit of three target repositories against SEA Forge (`sea-rs`), scoped to one strategic question: **what do these repositories reliably do that SEA Forge cannot, that matters for Thoth acting as an orchestrator of many agents (SWE_SEED instances and any OpenAI-/Anthropic-API-compatible agents), and what is the smallest architecture-compatible acquisition path?**

Pared to one report by owner instruction (2026-07-14): shared baseline, per-target capability maps, one delta matrix, dossiers only for recommended acquisitions.

## 1. Executive Verdict

- **Most consequential delta**: SEA Forge has *zero* LLM/agent connectivity. It can govern, sandbox, evidence, and settle work, but it cannot *converse with or delegate to* a model or an external agent. Goose proves the exact missing layer in Rust: a small `Provider` trait (`complete`/`stream`) with `openai_compatible` and `anthropic` implementations, and recipe-driven sub-agent delegation with bounded concurrency and cancellation.
- **Acquire first**: an `AgentProvider` seam (pattern from goose, reimplemented against SEA Forge abstractions) plus a governed `agent_task` delegation path where every external-agent invocation is an ordinary sandboxed, authority-gated, settlement-judged run. Everything else layers on that.
- **Do not copy**: semantic-kernel code (Python/.NET, and SK itself is officially superseded by Microsoft Agent Framework — its README says so at the inspected commit); goose's ungoverned execution semantics (tool return = done, session store as truth); t3code's Effect-TS event-sourcing stack (SEA Forge's ledger already does this with stronger guarantees).
- **Strategic value by target**:
  - *goose* — **architecture + code-adjacent patterns** (Rust, same license family, directly mappable traits). Highest transfer value.
  - *semantic-kernel* — **patterns only** (orchestration topologies: sequential, concurrent, handoff, magentic ProgressLedger; each unit-tested). Valuable as a topology vocabulary that compiles onto the CMMN case engine.
  - *t3code* — **protocol + contract discipline** (uniform `ProviderDriver` over heterogeneous third-party agent CLIs, per-provider approval policy / sandbox mode / continuation identity; ACP as the interop protocol). Valuable as the design for driving CLI-based agents (Claude Code, Codex) as opposed to raw APIs.
- Both goose and t3code independently converge on **ACP (Agent Client Protocol)** for agent interop. That convergence is the strongest external signal in this audit.

## 2. Inspection Substrate

| Item | Value |
|---|---|
| Current project | SEA Forge (`sea-rs`), branch `full-spec`, commit `13c3b19f8f5fd49e0dbbdf7588df38b7acd11c41` |
| Worktree | dirty: untracked spec/report files only (no production changes) |
| Target 1 | https://github.com/microsoft/semantic-kernel @ `c781da134e38acbee57616efc8662d2cfd8130d5` (main, 2026-07-09), MIT |
| Target 2 | https://github.com/aaif-goose/goose @ `743609d014833abf77657f36ca0a5ba0a3ae0887` (main, 2026-07-14), Apache-2.0. Self-referential README (fork lineage of block/goose; audited as-is at this SHA) |
| Target 3 | https://github.com/pingdotgg/t3code @ `3513fa04fbf12c1d4fa2b8d07cfc7f0905714d31` (main, tag `v0.0.29-nightly.20260714.809`), MIT |
| Clones | shallow, under `.tmp/ref/<name>/` (not vendored) |
| Date | 2026-07-14 |
| Baseline sources | `.agents/specs/spec-minimum.md` (implemented v0.1), `.agents/specs/spec-full.md` (M0–M4b implemented per `.agents/CURRENT_STATUS.md`), `.agents/specs/spec-adlc-thoth.md` (draft, M9–M11), workspace tree at the commit above |

## 3. Method and Evidence Standard

Current project inspected first (baseline below), then targets, tracing entry point → mechanism → state → outcome → test/fixture evidence. Classifications: **Proven** = implementation + tests/fixtures exercising the user-facing outcome; **Implemented-unproven** = code path exists, no located end-to-end evidence within audit scope; **Declared** = docs/README only. Audit depth was deliberately bounded (owner instruction); files cited in the Evidence Appendix were opened or grepped directly; nothing cited is inferred from README alone except where marked Declared.

## 4. Current-Project Capability Baseline (delta-relevant subset)

| Capability | Outcome | Entry point | Mechanism | Evidence | Maturity |
|---|---|---|---|---|---|
| Governed run lifecycle | intent → authority → sandbox → trace/evidence → settlement, fail-closed | `sea-forge run` | 15 kernel crates, `pipeline.rs` | P1–P4b, §17 conformance suites, all green | Proven |
| Tamper-evident history | alteration/fork/truncation detection, signed+witnessed checkpoints | `sea-forge ledger verify\|prove` | `sea-forge-ledger` (chain+MMR, Ed25519, witnesses) | M0 gate tests | Proven |
| CMMN case engine | sentry-activated non-linear plans, milestones, discretionary items, reactivation | `sea-forge run --plan`, `case`, `task` | `sea-forge-planner` reducer + sentry evaluator | M2a tests (A/B/C/M replay) | Proven |
| Plan templates | deterministic versioned process assets | `run --template` | `templates.rs`, SHA-pinned | M2b tests | Proven |
| Criteria provenance | settlement criteria trace to origin (intent/template) | pipeline-committed | M2c `OriginRef`/`SettlementCriteriaRecord` | M2c tests | Proven |
| Concurrent runs + approvals | N concurrent governed runs; escalate → human approve/reject/resume | `sea-forge-server` Unix socket; `approve\|reject\|resume` | Tokio, `max_concurrent_runs` semaphore (`crates/sea-forge-server/src/config.rs:7`), subprocess dispatch | M3 tests | Proven |
| Independent settlement + capability promotion | reliability-weighted declarations; attempted<demonstrated<proven | settlement adapters | `SettlementAuthority` trait, `SweSeedTransport` trait (`crates/sea-forge-settlement/src/declaration.rs:53,153`) | M4a 13 tests | Proven (SWE_SEED transport: trait + test double only — no real network impl) |
| Governed memory/recall | scope-gated recall with evidence | `sea-forge recall` | `sea-forge-capability/src/memory.rs` | M4b 8 tests | Proven |
| Jail sandbox | untrusted argv0 under Landlock | per-grant sandbox class | `sea-forge-sandbox/src/jail.rs` | M1 tests | Proven (Linux) |
| **LLM / model provider connectivity** | — | — | none anywhere in workspace | — | **Absent** |
| **Agent conversation/session model** | — | — | none (runs are one-shot command episodes) | — | **Absent** |
| **Delegation to external agents** | — | — | only seam: `SweSeedTransport` (settlement declarations, not task execution) | — | **Absent** |
| **Orchestration topologies over agents** | — | — | sentries can express DAGs of *commands*, not agent dialogues | — | **Absent** |
| Thoth epistemic interface | governed self-knowledge Q&A | spec only | `.agents/specs/spec-adlc-thoth.md` M9–M11; agent loop explicitly deferred | — | Planned |

**Target inspection checklist derived from baseline**: (a) model/agent provider abstraction and its OpenAI/Anthropic-compatible impls; (b) sub-agent delegation: dispatch, concurrency bounds, cancellation, result capture; (c) orchestration topologies and their coordination state; (d) durable session/conversation state and recovery; (e) heterogeneous agent adapter contracts (CLI agents vs API agents); (f) protocols (MCP/ACP/A2A); (g) evidence/test maturity of each; (h) licenses.

## 5. Target Capability Maps

### 5.1 goose (Rust, Apache-2.0)

| Capability | Outcome | Entry/mechanism | Evidence | Maturity |
|---|---|---|---|---|
| Provider abstraction | one agent loop drives ~20 model backends incl. OpenAI-compatible/Anthropic/Bedrock/Ollama | `Provider` trait: `complete`, `stream` (`crates/goose-provider-types/src/base.rs:381–394`); impls in `crates/goose-providers/src/` incl. `openai_compatible.rs:28,73`, `anthropic.rs` | `tests/providers.rs`; provider-types split into own crate | Proven |
| Sub-agent delegation | a parent agent runs a bounded child agent task from a recipe, with cancellation and streamed notifications | `run_subagent_task(SubagentRunParams)` (`crates/goose/src/agents/subagent_handler.rs:37–63`): recipe → child `Agent` → `TaskConfig.max_turns` cap → text/final-output extraction | called from `platform_extensions/summon.rs`; self-test recipe (`goose-self-test.yaml`) | Proven |
| Background parallel delegation | delegate N research/build tasks to background subagents with a concurrency cap; collect completed results | `summon.rs`: `DelegateParams`, `BackgroundTask`/`CompletedTask`, `tokio::spawn` + max-concurrent-background-tasks (`summon.rs:432,521,1858`) | in-file tests (`summon.rs:2772+`) | Proven |
| Durable sessions | resume any conversation after process death; usage/insight queries | `SessionManager` over SQLite/sqlx (`crates/goose/src/session/session_manager.rs:29,317`): `create_session`, `add_message`, `replace_conversation`, insights | session tests, `acp_fork_session_test.rs` (session forking) | Proven |
| Multi-session agent manager | one process hosts many concurrent agent sessions with per-session cancel tokens | `AgentManager` (`crates/goose/src/execution/manager.rs:35`): `get_or_create_agent(session_id)`, `max_sessions`, cancel-token registry | exercised via server/ACP tests | Proven |
| Recipes | versioned declarative task packets (instructions, prompt, extensions, settings) | `Recipe` struct (`crates/goose/src/recipe/mod.rs:43`) | recipe tests + `workflow_recipes/` | Proven |
| ACP server | any ACP client can drive goose as an agent; sessions forkable | `crates/goose/src/acp/` (server, provider, fs); `goose-acp-macros` | 8+ dedicated ACP integration tests (`crates/goose/tests/acp_*`) | Proven |
| Turn context / budget | injected per-turn operational context incl. autonomous-work budget signal | `moim.rs` (`TURN_CONTEXT_TAG`, turn-budget prompt block) | snapshot tests in `agents/snapshots/` | Implemented-proven at unit level |
| Cron scheduling | recurring recipe execution | `scheduler.rs` (tokio-cron-scheduler) | unit tests | Proven |
| Safety rails | tool permissioning, extension malware check, adversary/repetition inspectors | `permission/`, `security/`, `tool_inspection.rs` | `adversary_inspector_tests.rs`, `repetition_inspector_tests.rs` | Proven |

Limitations vs SEA Forge semantics: authority is per-tool confirmation (UI-ish), not fail-closed policy; session DB is mutable truth (no hash chain, no evidence/settlement separation); "task completed" = agent returned text (no settlement).

### 5.2 semantic-kernel (Python/.NET, MIT)

| Capability | Outcome | Entry/mechanism | Evidence | Maturity |
|---|---|---|---|---|
| Orchestration topologies | run a fleet of agents as sequential pipeline, concurrent fan-out/fan-in, group chat, LLM-routed handoffs, or magentic (manager-planned) | `python/semantic_kernel/agents/orchestration/`: `OrchestrationBase[TIn,TOut]` (`orchestration_base.py:85`), `sequential.py`, `concurrent.py`, `group_chat.py`, `handoffs.py` (`OrchestrationHandoffs:57`, `HandoffAgentActor:156`), `magentic.py` | one unit-test file per topology (`tests/unit/agents/orchestration/`) | Proven (in-process) |
| Magentic manager pattern | an orchestrator LLM maintains task+progress ledgers, re-plans on stall, and dispatches to specialist agents | `magentic.py`: `ProgressLedger`/`ProgressLedgerItem` (`:84–92`), reset/replan messages | `test_magentic.py` | Proven (pattern of Magentic-One) |
| Actor runtime | agents as actors with typed message routing; runtime pluggable | `agents/runtime/{core,in_process}` | orchestration tests run on it | Proven (in-process only) |
| Durable process framework | long-running processes with steps/edges, resumable on Dapr | `processes/` (`process_builder.py`, `dapr_runtime/`, `local_runtime/`) | samples + unit tests; Dapr path needs infra | Implemented; Dapr path unproven here |
| Multi-provider connectors | many chat-model backends behind one interface | `connectors/` | broad test suite | Proven |
| **Strategic status** | — | README (inspected commit): *"Semantic Kernel is now Microsoft Agent Framework"* — SK is the superseded predecessor | — | Declared, load-bearing |

### 5.3 t3code (TypeScript/Effect, MIT)

| Capability | Outcome | Entry/mechanism | Evidence | Maturity |
|---|---|---|---|---|
| Heterogeneous agent drivers | one server drives Claude Code, Codex, Cursor, OpenCode, Grok CLIs as interchangeable providers | `apps/server/src/provider/ProviderDriver.ts` (`ProviderInstance:64` w/ `continuationIdentity`, `adapter`, `textGeneration`), `Drivers/{Claude,Codex,Cursor,OpenCode,Grok}Driver.ts` | co-located tests throughout (`makeManagedServerProvider.test.ts`, driver home-layout tests) | Proven |
| Per-provider governance knobs | uniform approval policy, sandbox mode, interaction mode (`default`/`plan`), request kinds (`command`/`file-read`/`file-change`) across heterogeneous agents | `packages/contracts/src/orchestration.ts:35–131` (`ProviderApprovalPolicy`, `ProviderSandboxMode`, `ProviderRequestKind`, `ProviderApprovalDecision`) | `orchestration.test.ts` | Proven (contract level) |
| Event-sourced orchestration state | commands → decider → events → projector; invariants tested | `apps/server/src/orchestration/{decider,projector,Normalizer}.ts`, `commandInvariants.ts` | `decider.*.test.ts`, `projector.test.ts` | Proven |
| Workspace checkpointing | snapshot/diff of agent-modified workspace; reviewable diffs per checkpoint | `apps/server/src/checkpointing/{CheckpointStore,CheckpointDiffQuery,Diffs}.ts` | co-located tests | Proven |
| ACP adapter layer | ACP as common wire for agent CLIs | `apps/server/src/provider/acp/`, `packages/effect-acp/` | `AcpAdapterSupport.test.ts` etc. | Proven |
| Remote relay | drive local agents from web/mobile via relay/ssh/tailscale | `packages/{ssh,tailscale}`, `client-runtime/src/relay` | tests present; infra-dependent | Implemented |
| Provider maintenance/status | provider health snapshots, status cache, maintenance runner | `provider/provider{Snapshot,StatusCache,Maintenance*}.ts` | co-located tests | Proven |

## 6. Comparative Delta Matrix

| Target outcome | Current state | Delta class | Target mechanism | Evidence | Strategic value | Burden | Disposition |
|---|---|---|---|---|---|---|---|
| Drive any OpenAI-/Anthropic-compatible model/agent from Rust | Absent | **MISSING** | goose `Provider` trait + `openai_compatible`/`anthropic` impls | strong | critical (blocks all Thoth orchestration) | low–medium (small trait, 2 impls, reqwest) | **Acquire now** (pattern reimplementation) |
| Delegate bounded sub-tasks to child agents, in parallel, with cancellation | Absent (server semaphore governs *runs*, not agent tasks) | **MISSING** | goose `run_subagent_task` + summon background tasks; SK `concurrent.py` | strong | critical | medium (must be governed: each delegation = a run) | **Acquire now** (pattern, SEA semantics) |
| Name and reuse multi-agent topologies (sequential/concurrent/handoff/magentic) | Partial (sentries express command DAGs; no agent-dialogue topologies) | **PARTIAL / MECHANISM ADVANTAGE** | SK orchestration classes over actor runtime | strong (unit-tested per topology) | high | low as *templates*; high as runtime | **Acquire after prerequisite** — compile topologies to CasePlan templates, not a new runtime |
| Manager agent re-plans from a progress ledger (magentic) | Absent; but SEA's case file + ledger already *is* a superior task ledger | **PARTIAL** | SK `ProgressLedger` loop | strong | high (this is Thoth's orchestrator loop) | medium | **Acquire after prerequisite** (pattern; ledger-backed) |
| Resume agent conversation after process death | Partial (runs resumable via approvals/ledger; no conversation model at all) | **PARTIAL** | goose SQLite `SessionManager`; t3code decider/projector | strong | medium | low if transcripts = evidence records; high if new DB | **Acquire now, minimal form** (transcript-as-evidence, no second store) |
| Drive CLI-based agents (Claude Code, Codex) uniformly with approval/sandbox negotiation | Absent | **MISSING** | t3code `ProviderDriver` + ACP; goose ACP server | strong (two independent codebases converge on ACP) | high (SWE_SEED instances + coding agents are the stated fleet) | medium–high (process mgmt, protocol) | **Run an experiment first** (ACP adapter spike) |
| Uniform per-agent governance knobs (approval policy, sandbox mode, request kinds) | Superior substrate exists (authority fabric, SandboxClass, approvals) | **CURRENT PROJECT SUPERIOR** (semantics) / vocabulary worth adopting | t3code contracts | strong | medium | low | Adopt the *mapping table* only |
| Event-sourced orchestration truth | Present and stronger (signed+witnessed ledger vs unsigned event store) | **CURRENT PROJECT SUPERIOR** | t3code decider/projector | strong | — | — | Reject import; validates SEA design |
| Durable distributed processes (Dapr) | Case engine covers the outcome single-host | **INCOMPATIBLE** (infra: Dapr sidecars; overlaps case engine) | SK `processes/dapr_runtime` | medium | low | critical | Reject |
| Workspace checkpoint/diff review | Latent (workspaces retained per run; evidence hashes exist; no diff query) | **LATENT** | t3code CheckpointStore | strong | medium | low–medium | Observe; revisit for M5+ review UX |
| Turn budget/context injection for autonomous agents | Absent | MISSING (minor) | goose `moim.rs` | medium | medium | low | Observe (fold into delegation params later) |
| Cron-scheduled recurring agent work | Partial (`timer_listener` plan items; no cron surface) | PARTIAL | goose `scheduler.rs` | strong | low | low | Defer |
| Tool-call safety inspectors (adversary/repetition) | Different mechanism (authority + sandbox are stronger gates) | EQUIVALENT-ish | goose inspectors | strong | low | medium | Reject for now |
| SK connector breadth (Python/.NET) | — | **NON-TRANSFERABLE** (language; SK superseded by MAF) | — | — | — | — | Reject code; keep patterns |

## 7. Capability Acquisition Dossiers

### Dossier 1 — `AgentProvider` seam (model/agent API connectivity)

- **A. Outcome**: SEA Forge (specifically Thoth and the delegation path) can send a conversation to any OpenAI-compatible or Anthropic-compatible endpoint (incl. a SWE_SEED instance exposing either) and receive completions/streams, under authority.
- **B. Current limitation**: no HTTP/LLM client exists in any crate; the only external-agent seam is `SweSeedTransport` (`crates/sea-forge-settlement/src/declaration.rs:153`), which carries settlement declarations, not task dialogue.
- **C. Target implementation**: goose `Provider` trait — `async fn complete(...)`, `async fn stream(...)` (`crates/goose-provider-types/src/base.rs:381–394`); `OpenAiCompatibleProvider` (`goose-providers/src/openai_compatible.rs:28`) is base-URL + key + request-shape mapping; `anthropic.rs` likewise. Provider metadata via `ProviderDescriptor::metadata()` (`base.rs:278`).
- **D. Essential vs incidental**: *Essential*: one small trait (complete/stream over a typed message/tool vocabulary), openai-compatible + anthropic request/response mapping, typed errors, no fallback between providers. *Incidental*: goose's 20 providers, config UI, OAuth flows, local inference, catalog.
- **E. Acquisition option**: **(5) reimplement against current abstractions** (pattern adoption; consult goose code, Apache-2.0 permits copying with NOTICE if any literal code moves — prefer clean reimplementation so no NOTICE obligation attaches). Rejected alternatives: depending on goose crates (huge dependency surface, ungoverned semantics), copying wholesale (drags conversation/session types).
- **F. Integration points** (confirmed): new crate `sea-forge-agent` (or module in `sea-forge-thoth` per spec-adlc-thoth crate map); authority surface addition in `sea-forge-authority` (`external_api` resource class already reserved since v0.1 — `spec-minimum.md` §7.3.4); evidence kinds in `sea-forge-evidence`. Probable: server config for endpoint/credential refs.
- **G. Semantic translation**: goose "provider" → SEA Forge **AgentEndpoint** (an `ExtensionDescriptor` of kind `runtime_adapter`); goose "session/conversation" → **interaction transcript evidence** within a run; do **not** import "session as truth". API keys are `credential_bearing` — never in ledger payloads (spec-full §7.0c privacy rules).
- **H. Prerequisites**: available — authority fabric, `external_api` deny-by-default surface, evidence pipeline, server. Missing — none. External — `reqwest` (or similar) enters the workspace: first network client in the kernel's orbit; keep it out of kernel crates (async lives at server/adapter layer, preserving `no-async-kernel`).
- **I. Risks/debt**: async creep into kernel (mitigate: adapter crate only); provider API drift (mitigate: pin request shapes, contract tests with recorded fixtures like goose's `mcp_replays`); secret handling (existing sentinel redaction from M0 applies); every call is a paid side effect — must be an authority-gated operation, not ambient.
- **J. Minimum vertical slice**: `sea-forge agent probe <endpoint-ref>` — sends a fixed prompt to a configured openai-compatible endpoint under an `external_api` allow rule, records request-hash/response evidence, settles accepted on schema-valid response. Denied endpoint ⇒ no network I/O, settlement rejected.
- **K. Acceptance/proof**: contract tests against a local stub server (both API shapes); denial test proves no socket opened; evidence cross-linkage per P2 style; secret-redaction test (key never in any record). Failure behavior: unreachable/4xx/5xx ⇒ typed `agent_endpoint_error`, settlement `rejected`, no provider fallback.

### Dossier 2 — Governed delegation (`agent_task` plan items, parallel + cancellable)

- **A. Outcome**: a case plan item can be "delegate this instruction packet to agent X"; N such items run concurrently under the server semaphore; each delegation produces transcript evidence and settles independently; Thoth (or an operator) can cancel one without killing the case.
- **B. Current limitation**: `PlanItem.item_kind` has `sandboxed_task | human_task | milestone | …` (spec-full §7.1) — no agent-dialogue kind; runs execute argv commands only.
- **C. Target implementation**: goose `SubagentRunParams` (recipe + `TaskConfig{max_turns, provider}` + `CancellationToken` + notification channel) → child agent loop → bounded result extraction (`subagent_handler.rs:37–63,122+`); parallel dispatch with a concurrency cap in `summon.rs` (`:432,521`); SK `concurrent.py` fan-out/fan-in with typed input/output transforms.
- **D. Essential vs incidental**: *Essential*: instruction packet (maps to PlanTemplate/PlanItem params) + turn cap + cancellation + captured transcript + independent result judgment. *Incidental*: goose's recipe YAML shape, notification UI events, text-extraction heuristics.
- **E. Acquisition option**: **(5) reimplement using current abstractions.** A delegation is executed as an ordinary run whose "operation" is `agent_task` (new Operation kind gated by a new authority surface rule), driven by the Dossier-1 provider; `max_turns` is a grant boundary (grants already bind timeout/env/sandbox per Task 5 hardening). The server's existing `max_concurrent_runs` is the concurrency cap — no new pool.
- **F. Integration points** (confirmed): `sea-forge-core` types (`Operation`/`AuthorityAction` additive variant), `sea-forge-planner` (item kind + validation), `sea-forge-authority` (surface + grant boundaries), pipeline/`sea-forge-server` dispatch, settlement basis vocabulary (e.g. `turn_cap_exceeded`). Probable: `sea-forge-cli` `task`/`case` display.
- **G. Semantic translation**: goose "subagent task" → **execution episode (run) of an `agent_task` PlanItem**; goose "recipe" → existing **PlanTemplate** (do not import "recipe"); "task completed" ⇒ only *transcript captured*; **settlement remains the sole success arbiter** (agent narration never settles — this is SEA Forge's core corrective to all three targets). Agent output is untrusted input per spec-full §8.6.
- **H. Prerequisites**: Dossier 1. Everything else (case engine, criteria provenance, approvals, server concurrency) is proven substrate.
- **I. Risks/debt**: transcript volume in ledger (mitigate: content-addressed artifact files, ledger commits hashes — existing artifact rule); runaway cost (turn cap + `max_concurrent_runs` + authority per delegation); cancellation semantics must settle `rejected` with basis `cancelled`, never vanish.
- **J. Minimum vertical slice**: a two-item case plan — item A `agent_task` (ask stub agent to produce `answer.txt` content), item B `sandboxed_task` gated by a sentry on A's settlement; run with two A-instances in parallel; cancel one. Proof: cancelled item settles rejected/cancelled; surviving chain completes; ledger replay reproduces ordering.
- **K. Acceptance/proof**: conformance tests for parallel dispatch under semaphore, cancellation, turn-cap breach ⇒ rejected, transcript evidence hash-verified, denied `agent_task` ⇒ no provider call. Failure behavior: provider error mid-dialogue ⇒ transcript-so-far preserved as evidence, settlement rejected.

### Dossier 3 — Orchestration topology templates (sequential / concurrent / handoff-as-discretionary / magentic loop for Thoth)

- **A. Outcome**: operators/Thoth can instantiate named multi-agent topologies (`sequential_agents@v`, `concurrent_agents@v`, `manager_loop@v`) as plan templates, and Thoth can run a magentic-style loop: read case file + ledger (the progress ledger), decide next delegation, propose it as a governed discretionary item.
- **B. Current limitation**: nothing names or reuses agent coordination shapes; Thoth spec (M9–M11) deliberately ships Q&A only, no orchestrator loop.
- **C. Target implementation**: SK `sequential.py`/`concurrent.py` (typed in/out, fan-in collection), `handoffs.py` (`OrchestrationHandoffs` routing table, LLM picks the next agent), `magentic.py` (`ProgressLedger` — manager evaluates "is request satisfied / is progress being made / next speaker + instruction", replans on stall, `MagenticResetMessage`).
- **D. Essential vs incidental**: *Essential*: the topology vocabulary; magentic's stall-detection/replan loop over an explicit progress record. *Incidental*: Python actor runtime, message classes, group-chat manager strategies. **Key insight**: SEA Forge should *not* build an actor runtime — sequential = sentry chain, concurrent = parallel items + rollup milestone, and the progress ledger already exists as the case file + trace ledger (stronger: tamper-evident).
- **E. Acquisition option**: **(8) adopt the pattern, not the code.** Sequential/concurrent as E8 templates (pure config). Handoff: SK's LLM-chooses-next-agent conflicts with pre-declared plans — translate as *manager proposes a discretionary `agent_task`*, which is exactly the governed plan-mutation path (authority-checked, evidenced). Magentic manager = Thoth loop: bounded, each iteration = read (governed recall/self-model) → propose (discretionary item) → await settlement.
- **F. Integration points**: `.sea-forge/templates/` (confirmed mechanism), `sea-forge-thoth` (loop — probable, per spec-adlc-thoth roadmap "Hermes-derived agent loop behind the typed protocol"), planner discretionary-item path (confirmed, M2a implemented).
- **G. Semantic translation**: SK "handoff" → **discretionary-item proposal** (never runtime control-flow mutation outside authority); "group chat" → deferred (no current outcome needs it); "progress ledger" → **case file + trace ledger** (do not add a second ledger); "manager" → Thoth acting as an `R-AA` actor with sponsor.
- **H. Prerequisites**: Dossiers 1–2; spec-adlc-thoth M9–M11 for the Thoth-driven variant (templates alone need only M10 machinery, already proven).
- **I. Risks/debt**: manager-loop cost runaway (bound iterations as grant boundary); Thoth self-certification (already structurally barred: SoD, cannot settle own claims); temptation to build an actor/message runtime (reject — the case engine is the coordination substrate; spec-full §5 "sentries suffice" claim extends here and should be tested, not assumed).
- **J. Minimum vertical slice**: `concurrent_agents@0.1.0` template — 3 parallel `agent_task` items + rollup milestone whose sentry requires all three settlements; then one manager iteration: Thoth reads the case, proposes 1 discretionary `agent_task` via the existing `case add-task` path, which runs and settles.
- **K. Acceptance/proof**: template determinism (existing gate style); rollup fires only on all-settled; discretionary proposal denied without authority ⇒ no item added; stalled case (item parked) ⇒ manager iteration observable in ledger proposing remediation. Failure behavior: manager iteration cap reached ⇒ case parks, escalation to operator (existing approval machinery).

### Dossier 4 — ACP driver experiment (heterogeneous CLI agents)

- **A. Outcome**: SEA Forge can delegate to CLI-resident agents (Claude Code, Codex, SWE_SEED-as-CLI) through one adapter contract, with continuation identity (resume the same agent session) and negotiated approval/sandbox posture.
- **B. Current limitation**: Dossiers 1–2 cover only HTTP-API agents; CLI agents hold their own loops/state and cannot be driven by `complete/stream`.
- **C. Target implementation**: t3code `ProviderDriver`/`ProviderInstance` (continuationIdentity, adapter, status snapshot — `ProviderDriver.ts:64–103`) with per-CLI drivers (`Drivers/*.ts`) and ACP adapters (`provider/acp/`); goose's ACP server side (`crates/goose/src/acp/`, 8+ integration tests) proves a Rust implementation of the same protocol.
- **D. Essential vs incidental**: *Essential*: the driver contract (spawn/attach, continuation key, typed permission requests surfaced outward, status snapshot) and ACP as the wire. *Incidental*: Effect-TS layers, Electron/relay/UI, provider maintenance UX.
- **E. Acquisition option**: **(6)/(7) integrate through an adapter + adopt the protocol** — but **run an experiment first**: spike one ACP client adapter (goose as the counterpart agent, since it ships an ACP server and is locally runnable) mapping ACP permission requests → SEA Forge authority decisions. The mapping's fidelity (t3code's `ProviderRequestKind` `command|file-read|file-change` → SEA operations; `ProviderSandboxMode` → `SandboxClass`; approval decisions → approvals machinery) is the experiment's success criterion.
- **F. Integration points** (probable): new adapter under the extension ABI (`ExtensionDescriptor.kind: runtime_adapter`), server dispatch, approvals. None confirmed until the spike.
- **G. Semantic translation**: t3code "approval policy" → authority policy rules (never a per-agent local override that bypasses the fabric); "checkpointing" → run workspace + evidence (defer); "continuation identity" → case/run linkage of successive episodes against one external session.
- **H–K**: prerequisite Dossiers 1–2; risk: protocol churn (ACP is young) and child-process lifecycle management; slice = one ACP session with goose executing a read-only task under a jail-class grant, permission request surfaced as a SEA approval; proof = approval-mediated permission grant recorded in ledger; failure = ACP disconnect ⇒ run settles rejected with transcript-so-far evidence.

## 8. Rejected or Deferred Capabilities

| Capability | Reason |
|---|---|
| SK code (any language) | Non-transferable (Python/.NET) and SK is officially superseded by Microsoft Agent Framework — building on it imports a deprecated lineage. Patterns retained. |
| SK Dapr process framework | Architecture conflict: duplicates the case engine with weaker guarantees; drags Dapr infrastructure (spec-full non-goals: no distributed transactions/infra). |
| goose session store as truth | Semantic conflict: mutable SQLite conversation state vs ledger-first truth. Transcripts become evidence artifacts instead. |
| goose extension/MCP surface wholesale | SEA Forge's extension ABI + authority fabric already governs this seam; importing goose's would create a second gate. MCP support, when wanted, should arrive as an adapter through the existing ABI. |
| t3code event-sourcing stack | Duplicative — SEA Forge's ledger is a stronger implementation of the same idea (signed, witnessed, fork-detecting). Its existence *validates* the architecture. |
| t3code relay/remote-access, UI, desktop | Product surface; spec-full non-goal (UI is a future client of E3 contracts). |
| Group-chat topology | No current outcome requires multi-agent free-form dialogue; revisit if a real case needs it. |
| goose cron scheduler | `timer_listener` plan items + server cover the near-term outcome; defer. |
| goose safety inspectors | Authority + jail are the stronger gate for SEA's threat model; inspectors are heuristic prompt-layer defenses — reconsider only for the future LLM presenter. |
| Workspace checkpoint/diff review (t3code) | Latent locally (retained workspaces + hashed artifacts); acquire when a review UX consumer exists (M5+). |

## 9. Recommended Acquisition Sequence

| # | Capability | Prerequisite | Slice | Proof | Unlocks |
|---|---|---|---|---|---|
| 1 | `AgentProvider` seam (Dossier 1) | none (post-M8 recommended) | `agent probe` against stub endpoint | contract + denial + redaction tests | everything below |
| 2 | Governed `agent_task` delegation (Dossier 2) | #1 | 2-item case, parallel + cancel | parallelism/cancel/turn-cap conformance | fleets of SWE_SEED/API agents as case labor |
| 3 | Topology templates (Dossier 3a: sequential/concurrent) | #2 | `concurrent_agents@0.1.0` + rollup | template determinism + rollup gate | reusable orchestration vocabulary |
| 4 | Thoth manager loop (Dossier 3b, magentic-as-discretionary) | #3 + spec-adlc-thoth M9–M11 | one bounded manager iteration | proposal-under-authority + stall handling | Thoth as orchestrator (the stated goal) |
| 5 | ACP driver spike (Dossier 4) | #2 (parallel to #3–4) | one ACP session vs goose, approval-mediated | permission→approval mapping demonstrated | CLI agents (Claude Code/Codex) in the fleet |

## 10. Implementation Specification Inputs

- **Problem statement**: SEA Forge governs work but cannot employ model-backed agents as labor; Thoth (spec-adlc-thoth) needs a delegation and orchestration layer whose every step remains authority-gated, evidenced, and settlement-judged.
- **Scope boundary**: HTTP-API agents (OpenAI/Anthropic-compatible, incl. SWE_SEED) first; CLI agents via ACP behind an experiment gate; no actor runtime; no second conversation store; no LLM in kernel decisions.
- **Components affected**: new adapter crate (`sea-forge-agent` or thoth module), `sea-forge-core` (additive `agent_task` operation kind), `sea-forge-planner`, `sea-forge-authority` (activate reserved `external_api`; grant boundaries: turn cap, endpoint ref, token budget), pipeline/server dispatch, settlement basis vocabulary, templates, CLI (`agent probe`, display).
- **Data model changes** (all additive, v0.2 rules): `AgentEndpoint` descriptor (extension registry), transcript evidence kind, `agent_task` operation/plan-item kind, new settlement bases (`cancelled`, `turn_cap_exceeded`, `agent_endpoint_error`).
- **Contract changes**: provider trait (`complete`/`stream`) private to the adapter crate; server socket gains no new ops for #1–3.
- **State transitions**: delegation runs use the existing run lifecycle; cancellation ⇒ settled `rejected` basis `cancelled`; manager iterations are ordinary discretionary-item proposals.
- **Migration**: none (additive).
- **Dependencies**: HTTP client in adapter crate only; `no-async-kernel` invariant preserved.
- **Compatibility**: P1–P4b and all M0–M8 gates unchanged; spec-adlc-thoth M9–M11 unchanged (this slots as its roadmap "agent loop" made concrete, or as M12+).
- **Acceptance/proofs and failure cases**: per dossier §K; global rule — an agent's own claim of success is never a settlement input with standing (extends M4a self-declaration rejection to delegated work).
- **Unresolved decisions**: see §11.
- **Work packages**: WP1 provider seam; WP2 delegation; WP3 templates; WP4 Thoth loop; WP5 ACP spike — matching §9.

## 11. Open Questions

1. **ACP adoption vs bespoke protocol** for CLI agents: both goose and t3code converge on ACP, but ACP's permission model must map losslessly onto SEA authority (spike in §9 #5 answers this; a lossy mapping would force bespoke).
2. **SWE_SEED's actual interface**: is a SWE_SEED instance reachable as an OpenAI-/Anthropic-compatible endpoint, an ACP agent, or only via the existing settlement transport? Determines whether WP1 or WP5 unlocks it. Not answerable from any inspected repository.
3. **Transcript retention policy**: full transcripts as content-addressed artifacts vs summarized evidence — cost/privacy trade-off needing an owner decision (crypto-shredding rules from spec-full §7.0c apply either way).

## 12. Evidence Appendix

**Current project**: `crates/` (15 kernel crates); `crates/sea-forge-server/src/config.rs:7` (`max_concurrent_runs`); `crates/sea-forge-settlement/src/declaration.rs:53,153` (`SettlementAuthority`, `SweSeedTransport`); `.agents/CURRENT_STATUS.md` (M0–M4b gate evidence); `.agents/specs/spec-full.md` §7.1 (PlanItem kinds), §7.0b (extension ABI), §8.6 (untrusted plan proposals); `.agents/specs/spec-adlc-thoth.md` (Thoth protocol, roadmap seams). Absence of LLM/HTTP connectivity verified by workspace-wide inspection (no reqwest/hyper/provider modules in any kernel crate).

**goose** (`.tmp/ref/goose/`): `crates/goose-provider-types/src/base.rs:278,381–394`; `crates/goose-providers/src/openai_compatible.rs:28,73`; `crates/goose/src/agents/subagent_handler.rs:37–63,122+`; `crates/goose/src/agents/subagent_task_config.rs:13–19`; `crates/goose/src/agents/platform_extensions/summon.rs:52,67,432,521,1858,2772+`; `crates/goose/src/session/session_manager.rs:29,317,413–478`; `crates/goose/src/execution/manager.rs:35,52,114,355`; `crates/goose/src/recipe/mod.rs:43`; `crates/goose/src/scheduler.rs:106,152`; `crates/goose/src/agents/moim.rs:9–24`; `crates/goose/src/acp/`; tests: `crates/goose/tests/{acp_*,providers.rs,agent.rs,compaction.rs,adversary_inspector_tests.rs}`; `goose-self-test.yaml`.

**semantic-kernel** (`.tmp/ref/semantic-kernel/`): `README.md` (MAF supersession notice); `python/semantic_kernel/agents/orchestration/{orchestration_base.py:34,85,181, sequential.py, concurrent.py, group_chat.py, handoffs.py:57,127–156, magentic.py:56–92}`; `python/semantic_kernel/agents/runtime/{core,in_process}`; `python/semantic_kernel/processes/{process_builder.py,dapr_runtime,local_runtime}`; tests: `python/tests/unit/agents/orchestration/test_{sequential,concurrent,group_chat,handoff,magentic,orchestration_base}.py`.

**t3code** (`.tmp/ref/t3code/`): `apps/server/src/provider/ProviderDriver.ts:43–103`; `apps/server/src/provider/Drivers/{Claude,Codex,Cursor,Grok,OpenCode}Driver.ts`; `apps/server/src/provider/acp/`; `packages/contracts/src/orchestration.ts:25–131`; `apps/server/src/orchestration/{decider.ts,projector.ts,commandInvariants.ts}` (+ co-located tests); `apps/server/src/checkpointing/{CheckpointStore,CheckpointDiffQuery}.ts`; `packages/{effect-acp,ssh,tailscale}`; `README.md` (supported agents).

---

## Conclusion

**READY AFTER LISTED DECISIONS**

The investigation is sufficient to write an implementation specification for WP1–WP4 (provider seam, governed delegation, topology templates, Thoth manager loop) without revisiting the target repositories — mechanisms, semantic translations, integration points, slices, and proofs are specified above against confirmed substrate. Two decisions gate parts of the design and are the reason for this verdict rather than READY FOR IMPLEMENTATION SPEC: (1) SWE_SEED's actual interface shape (§11.2) determines which work package first connects the stated fleet, and (2) the ACP-vs-bespoke decision (§11.1) gates WP5 and should be settled by the small spike defined in Dossier 4 rather than by more reading. Neither decision blocks writing the WP1–WP3 spec immediately.
