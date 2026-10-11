# GodSpeed Judgment Plane — Repository-Grounded Adversarial Architecture Review

**Status:** Review complete — no implementation performed, no production code modified, no persisted data altered.
**Date:** 2026-09-16
**Normative input under test:** `.agents/specs/GODSPEED_JUDGMENT_PLANE_SPEC.yaml` (1494 lines, `status: draft`, `version: 0.1.0`) — treated as a **hypothesis**, not truth.
**Template respected:** `.agents/specs/AGENT_SPEC_TEMPLATE.yaml` (1055 lines). Meaning/contract/proof vs implementation plan vs repository context vs mutable execution state vs evidence stay separate — this review is *evidence*, not a plan.

---

## 0. Method, evidence discipline, and the correction that reframes everything

**Target-machine correction.** My first pass was grounded on the development host. The user corrected this: **the deployment target is an NVIDIA Jetson**, and the development host is not the runtime. All machine-grounded reasoning is therefore split:

| | Development host (this box) | Deployment target |
|---|---|---|
| Identity | WSL2, Ubuntu 26.04, 6 vCPU, 7.8 GiB RAM, no CUDA, no `nvidia-smi` | **NVIDIA Jetson AGX Orin 64 GB, SM87, unified CPU/GPU memory** — `edgeai/AGENTS.md:7` |
| Inference | *absent* — no Ollama, no GGUF/safetensors, no vLLM, no llama.cpp server | llama.cpp (GGUF, SM87) + **vLLM AWQ**; default `Qwen3.6-35B-A3B-AWQ-4bit` — `edgeai/README.md:140` |
| Front door | *absent* | **launcher, OpenAI-compatible, `127.0.0.1:18001`**, memory-aware model admission — `edgeai/README.md:83-90` |
| Services | only `gauntlet serve` on `:4000`; no Postgres, no NATS, no Redis, no PGMQ | `nats:2.10-alpine` + `sea-bridge` (`edgeai/deploy/jetson/docker-compose.yml:331,348`); `ruvector-postgres:arm64-local` (`docker-compose.knowledge.yml:6`) |

**Consequence:** any argument of the form "this is too big for the machine" is void. The target has 64 GB of unified memory with GPU-class inference behind an OpenAI-compatible endpoint. The binding constraint is not capacity — it is **the absence of a typed bounded-judgment contract and a loop correlation identity** (§1.3, §3, §7).

**Evidence classes used throughout:**
`[FACT]` repository or machine evidence directly establishes it, citing `path` or `path:line`.
`[INFERENCE]` evidence strongly suggests it; competing explanations are named.
`[ASSUMPTION]` required for the conclusion but not established; recorded as a gap.
`[ROADMAP]` intentionally future work.

**Explicit non-claims.** This review does **not** claim: production readiness from unit tests; provider replaceability from the mere existence of a port trait; learning where no weights change; settlement merely because downstream code consumed a value.

---

## 1. Executive Finding

### 1.1 What GodSpeed actually is today

GodSpeed is **not** an unfinished product missing a judgment layer. It is a **settled, evidence-heavy governed-execution kernel (SEA-Forge) attached to a settled constrained execution harness (Gauntlet), surrounded by five independent, largely unconsumed satellites**: a normative protocol (CEP), a Rust conformance runtime for it (RealityTrace/sxr), a context-assembly service (Context Kernel), a semantic compiler (DomainForge), and two developmental-memory aspirants (godspeed_agent, SWE_SEED).

Load-bearing, verified facts:

- **SEA-Forge is the only component at production scale.** 22 workspace crates, **4,039 `#[test]`**, 4 CI workflows, 193 commits. It owns authority (`PolicyAuthorityEngine`; `Verdict{Allow,Deny,Escalate}`; `NormalizedDisposition{Allow,Deny,Escalate,Boundary,Degraded}`; `Determinism{policy_bundle_hash, action_request_hash, identity_binding_hash}` — `crates/sea-forge-core/src/types.rs:361-397`), settlement (`SettlementClaim`, `BatchEvaluationResult`, `SettlementEvent`, `SettlementDeclaration` — `types.rs:742-1013`), and an integrity ledger with MMR + ed25519 witness (`crates/sea-forge-ledger/src/types.rs`).
- **Gauntlet is the settled execution harness and the accidental owner of the only real judgment corpus.** 5 core crates + **24 adapters**, **no CI workflows at all**, 16 live run databases under `targets/.runs/*/state/*.db`.
- **The frozen runtime contract already allocates every responsibility.** `docs/explanations-and-references/goodspeed-loop.md` names each component's ownership exactly. **Judgment is not in that table.** That absence is the actual finding — not a documentation oversight, but a genuine architectural blank.

### 1.2 Is the Judgment Plane needed?

**Yes — but almost none of the spec's proposed machinery is.** The need is narrow and falsifiable:

- `[FACT]` **No probabilistic, uncertainty-preserving, bounded-question type exists anywhere in the stack.** `sea-forge-core/src/types.rs:1101` carries `confidence: String`. `sea-forge-capability/src/promotion.rs:363` computes a `confidence` that is a **deterministic weighted product** (`reliability × coverage × recovery × burden`, `:345`). Gauntlet's `Outcome` is 3-valued (`PASS|FAIL|INCONCLUSIVE`). CEP's `Question` requires only `question_id` + `question_form` in its machine-enforced schema (`cep/schemas/cep-semantic-envelope.schema.json:1385`); `question_type` is an **unconstrained string** whose 17-value enum exists only in prose (`cep/spec/CEP-0005-question-model.md:336-352`); and there is **no allowed-answer-domain field, no numeric-scale field, and no probability field**.
- `[FACT]` Therefore every judgment in the system today is *deterministic* or *unquantified*. `sea-forge-thoth/src/manager.rs:26` `judge(...)` returns one of four states by pure rule evaluation of observed facts. Gauntlet's model rung emits a 3-valued `Outcome` with **no distribution** (`crates/gauntlet-domain/src/verification.rs:26-33`).
- `[FACT]` The model rung is **last resort, position 10 of 10**, and structurally marked non-deterministic: `RungKind::deterministic()` is `false` only for `ModelJudgment` (`crates/gauntlet-app/src/verify/ladder.rs:100-104`), and "a model record claiming `instrument.deterministic: true` is refused" (`ladder.rs:29-31`).

The missing capability is real and precisely located: **a bounded, typed question with a declared answer domain, answered with preserved uncertainty, recorded as evidence, composed deterministically, and structurally unable to widen authority.** That is a small artifact. It is not a plane.
### 1.3 The most important architectural discovery

**Both halves of the Judgment Plane already exist, in different repositories, and they do not overlap — they fail to meet.**

| Half | Where it exists | What it has | What it lacks |
|---|---|---|---|
| **Judgment contract + orchestration + consequence corpus** | Gauntlet | `AgentRunner` port (`gauntlet-ports/src/agent_runner.rs:440-470`); `RoleInvocationSpec` with **`context_envelope_digest` bound per invocation** (`:267-284`); `ContextEnvelope{body,digest}` (`:240-258`); `RoleId{…,Verifier,Critic,Observer,Optimizer,Custom}` (`gauntlet-domain/src/values.rs:617-625`); `VerificationRecord` binding *run + claim + evidence + outcome + instrument* (`verification.rs:26-33`); 16 run DBs; `promote/` with held-out control/treatment + regression | **no HTTP model provider.** The only real runner is `CliAgentRunner`, driven by `GAUNTLET_CLI_AGENT_CMD`, shell-word parsed, **no default, no fallback** (`gauntlet-adapter-agent-cli/src/config.rs:4-15,39-44`) |
| **HTTP model provider + typed failures + network policy** | SEA-Forge | `AgentProvider` trait (`sea-forge-agent/src/provider.rs:71-83`); **`OpenAiCompatibleProvider`** hitting `POST /v1/chat/completions` (`:154-184`); `AnthropicProvider`; 10-variant typed `AgentError` (`:85-97`); `EndpointSnapshot`; `NetworkPolicy` with explicit loopback allowance; token `Usage` | **no judgment contract.** `provider.rs` is consumed only by `delegation.rs` and `sea-forge-server/src/agent_probe.rs`; nothing produces a bounded typed answer |

`[INFERENCE, high confidence]` The Judgment Plane's real deliverable is therefore **one adapter and one type family** — not a subsystem. Put SEA-Forge's existing HTTP provider behind Gauntlet's existing judgment contract, and give the answer a declared domain and a distribution.

`[FACT]` **Two materially independent provider paths already exist and are already contract-tested**, which is what the spec's own provider-substitution requirement demands: a shared `AgentRunner` contract suite runs against the real adapters (`gauntlet-adapter-agent-cli/src/lib.rs:59`, `gauntlet-adapter-agent-prime/src/lib.rs:31`), and `sea-forge-agent/tests/provider_contract.rs` drives `OpenAiCompatibleProvider` against a real in-process HTTP server, asserting the exact request line `POST /v1/chat/completions HTTP/1.1`.

### 1.4 The minimum delta

Five bounded changes — **zero new services, zero schema breaks, zero new databases, zero `.sea` changes**:

1. **S1 — one adapter.** A single Gauntlet `AgentRunner` implementation speaking HTTP through the existing `sea_forge_agent::OpenAiCompatibleProvider` (or a parallel minimal HTTP transport) targeting the Jetson launcher at `127.0.0.1:18001`. No new port trait. No new process.
2. **S2 — one semantic object.** A bounded question surface carried **inside the existing CEP Question envelope section** (additive; no new envelope kind, no CEP version break), plus a `DecisionSurface` **derived artifact** materialized from DomainForge's existing `ApplicationContract` (`domainforge-core/src/application/contract.rs:45`) and `Reference`/`ReferenceDefinition` (`gauntlet-domain/src/reference.rs:391,412`) — **not a new storage engine, not a compiler, not a grammar change**.
3. **S3 — one versioned rule file.** Composition thresholds move out of code into a versioned, inspectable config consumed by the **existing** `sea-forge-settlement` path (`SettlementClaim.evaluator_scores` → `SettlementEvent`). No new `CaseAssessment` type.
4. **S4 — one correlation field.** `loop_correlation_id` bound once at case admission and echoed by SEA-Forge `run_id`, Gauntlet `run_id`, sxr snapshot/run refs, and CEP envelope scope. This is the single highest-leverage change in the review, and it is **one field plus four echo sites**.
5. **S5 — an ownership decision.** One memory owner per purpose, recorded. Nothing new is built; the five ledgers simply stop growing into each other.

### 1.5 The largest production blocker

`[FACT]` **The component that owns the judgment corpus has no CI.** `gauntlet` has no `.github/workflows` directory at all. It owns `Execution Environment` in the frozen loop contract, hosts the only consequence-grounded calibration data (16 run DBs), and implements the judgment ladder — and **no automated gate protects it**. By contrast SEA-Forge (4 workflows), Context Kernel (2), CEP (2), SWE_SEED (1) and NeatCode have CI; sxr and godspeed_agent do not.

`[FACT]` Second blocker, named by the repository itself: `just crate-test sea-forge-server` currently reaches an **unrelated failing test**, `conformance_run_locator::both_layouts_still_resolve_after_a_restart`, because a second server cannot start while the first holds the cell lock (`.agents/CURRENT_STATUS.md`, 2026-08-31). A red full-suite gate on the authority-owning crate blocks any production claim.

### 1.6 The highest-value existing capability being underused

`[FACT]` **Gauntlet's run databases, and Gauntlet's `promote/` controlled-evaluation machinery.** Across the 16 DBs I measured directly: **37** `role_invocation`, **28** `correction_selection`, **17** `verification_record`, **17** `settlements`, **29** `observation_bindings`. The `records` table already admits 14 typed kinds including `observation`, `comparison`, `residual`, `promotion_decision`, `correction_selection`. And `promote/` already implements **held-out control vs treatment plus regression control vs treatment** (`revision_evaluations` columns `held_out_control, held_out_treatment, regression_control, regression_treatment`) behind a `PromotionGate`.

`[INFERENCE]` This is a **consequence-grounded evaluation harness** that nobody is pointing at judgment. The spec's central ambition — "consequence-grounded evaluation instead of subjective prompt evaluation" — is **already implemented for capability promotion** and merely not applied to judgment quality.

### 1.7 The most dangerous duplication or boundary error

**Two errors, in this order of danger.**

**First (correctness / authority risk): the noun "judgment" already means deterministic classification in SEA-Forge, and the spec reuses it for probabilistic inference.** `sea-forge-core/src/types.rs:819` `ManagerJudgment{Satisfied,Progressing,Stalled,Blocked}`; `sea-forge-thoth/src/manager.rs:21` — "never invents free-form judgment — purely a function of the observed facts"; `sea-forge-cli/src/commands/manager.rs:169` `judge(&view)`; plus a third unrelated use, `judge_snapshots` in `gauntlet-cli/src/improve_cmd.rs:2057`. Introducing `Judgment`/`JudgmentObservation` with *probabilistic* semantics into the same crate tree, under the same word, guarantees that a future reader cannot distinguish a deterministic classification from a model inference. **This is the most likely source of expensive rework**, and it must be settled by an explicit naming decision rather than left to context.

**Second (evidence / provenance risk): five append-only ledgers and three incompatible "evidence" schemas.** `sea-forge-ledger` (MMR + global checkpoint + ed25519 witness) vs `sxr-ledger` (hash chains; **MMR deliberately omitted**, acknowledged at `sxr-core/src/types.rs:71` as mirroring `sea-forge-ledger::LedgerEntry` "so a future export is a projection") vs Gauntlet's SQLite `event_log` + hash-chained records vs `godspeed_agent` `LedgerStore` (newline-JSON + `fsync` + `flock` + checksums, `storage.py:52-66`) vs SWE_SEED's `trace_ledger.sqlite3`.

Separately, the word *evidence* names three incompatible shapes: CEP-0006 question-bound Evidence; `sea_forge_core::types::EvidenceRecord{kind,uri,sha256,source_event_id}` (`sea-rs/crates/sea-forge-core/src/types.rs:595`); and Gauntlet's evidence CAS plus `evidence_productions` table. None is a projection of another; a loop that produces all three cannot answer which one a settlement rested on.

### 1.8 Is a Jev-like capability affordable from the existing stack?

**Yes, and the cost is measured in days, not quarters.** `[INFERENCE, high confidence]`

- The Jetson already serves an OpenAI-compatible endpoint (edgeai launcher `:18001`) with a local Qwen model — Provider B is *running hardware*, not a research task.
- `OpenAiCompatibleProvider` already speaks that protocol and already permits loopback HTTP through `NetworkPolicy{allow_loopback_test}` (`sea-rs/crates/sea-forge-agent/src/network.rs:35-48`); non-loopback production endpoints require HTTPS, which is a correct and already-enforced boundary.
- Gauntlet already has the invocation boundary, the persisted per-invocation **context digest**, the typed judgment record, the deterministic-before-model ladder, and the settlement correlation.
- The only semantic object missing is a **bounded question with a declared answer domain and preserved uncertainty**. That is a type and a rule table.

**What is not affordable and must not be claimed: calibration.** `[FACT]` **17 settled judgments across 16 runs** is enough to prove the mechanism end-to-end; it is nothing like enough to calibrate a model. Proposition 13 of the brief is **falsified** (§11).
---

## 2. Current Architecture — Evidence Grounded

### 2.1 Component map (what each repository actually is)

| Repository | Language / runtime | Verified scale | Verified role | Maturity | Actually used by the loop? |
|---|---|---|---|---|---|
| `sea-rs` (SEA-Forge) | Rust 2021, 22 workspace crates; `sea-forge-server` and `sea-forge-agent` are the 2 async edge crates, the other 20 synchronous | 4,039 `#[test]`; 4 CI workflows; 193 commits | Authority, governed execution, operational evidence, operational settlement, capability promotion, integrity ledger | **production-capable** | yes — kernel of the loop |
| `gauntlet` | Rust, 5 core crates + 24 adapter crates | no CI; 16 live run DBs | Execution Environment (frozen contract); verification ladder; run/consequence corpus | **working-prototype, high internal rigour** | yes — Execution Environment |
| `sxr` (RealityTrace) | Rust, 6 crates (`sxr-core`, `-ledger`, `-verify`, `-git`, `-df`, `-cli`) | 192 `#[test]`; no CI | CEP record-kind runtime + expected-vs-observed binding + SQLite ledger | **working-prototype, partial** | declared, weakly wired |
| `cep` | Python spec + validator + scorer; 11 normative docs | 65 pytest funcs; 2 CI workflows | Normative protocol; **no runtime, no model call** | **working-prototype (spec-quality)** | as protocol only |
| `Context_Kernel` | Rust, 20 crates (`ck-*`) | 221 `#[test]`; 2 CI workflows; embeddings **disabled by default** (`config/context-kernal.example.toml`) | Bounded cited context; MCP stdio/SSE, 8 tools | **working-prototype** | yes — via SWE_SEED MCP |
| `domainforge` | Rust core + Python + TypeScript + WASM | 680 MB checkout | Canonical `.sea` semantics; `ApplicationContract`; projections; `semantic_pack` | **production-capable (compiler)** | yes — semantic authority |
| `SWE_SEED` | Rust, 2 crates | 420 `#[test]`; CI; hard standalone-invariant test | Work contract, context requirement, proof plane; MCP gateway; hooks | **working-prototype** | declared |
| `godspeed_agent` | Python ≥3.11 | 1,092 `def test_`; **no CI** | Navigation/affordance selection, developmental memory, capability formation | **working-prototype** | declared; standalone in code |
| `NeatCode` | Node ESM, **0 runtime deps** | 45 `node:test`; published npm v1.1.0; CI | Repo review/restructure tooling | **production-capable (narrow)** | **no** — not on any runtime path |
| `edgeai` | Rust edge-* crates + Compose + systemd | 5.1 GB; `just edgeai-check` | The **target appliance**: launcher, llama.cpp, vLLM, Hermes, NATS, ruVector | **production-capable (appliance)** | target host |
| `hassos-addon-agent-memory-ledger` | Python add-on: Docker + **Postgres canonical + NATS JetStream bridge** | — | Intended durable developmental persistence | **deployable, not in loop** | no (no Postgres/NATS here) |

### 2.2 Dependency graph — classified, not assumed

**code dependency (compile-time):**
`sea-forge-runtime → sea-forge-authority`; `sea-forge-settlement → sea-forge-ledger`; `sea-forge-server → sea-forge-agent + sea-forge-authority + sea-forge-extension + sea-forge-ledger`.
No code dependency exists between any two *repositories* except `edgeai` crates within `edgeai`. **The stack is 13 repositories joined by contracts and identity strings, not by code.**

**process dependency (verified call sites):** `sxr-df::CliDomainForgeAdapter` invokes DomainForge's envelope CLI; `edgeai` launcher spawns `llama-server` and vLLM; `SWE_SEED/federation/context_client.rs:302-312` spawns the Context Kernel binary.

**MCP dependency (verified):** `SWE_SEED → Context Kernel` over stdio, `tools/call` name `context_required` (`swe-seed-core/src/federation/context_client.rs:366,380-386`), gated by env `SWE_SEED_CONTEXT_KERNEL_BIN` and a **no-op when unset** (`:277`). Context Kernel implements it (`ck-mcp/src/lib.rs:132`).

**wire-only (schema or identity string, no live call site):** CEP schema → sxr (the schema is *embedded and hash-pinned*: `sxr-core/src/embedded_schema.rs:15`, sha256 `4b42bfdd…`); `godspeed_agent` → NATS subject `sea.agent.event.godspeed-agent` (`agentic_capability_loop/publisher.py:30`), opt-in via `GSA_NATS_URL`, **no-op otherwise** (`:71-77`); `godspeed_agent` reads a SEA manifest path only to derive a domain hash with a warning-and-fallback (`adapters.py:26-53`).

**authority dependency:** exactly one — `sea-forge-authority` is consulted before every consequential action, including the model probe (`agent_probe.rs:221`).

**evidence dependency:** declared but **not implemented as a contract** — no repository accepts another repository's evidence as input through a typed interface; correlation is by convention on `run_id` strings.

**optional integration:** `project-topology-architect`, `domainforge-lsp`, `NeatCode` — present in the workspace file, on no runtime path.

`[FACT]` **Sharp negative result:** `godspeed_agent` contains **no import, HTTP call, or subprocess into SEA-Forge, Gauntlet, sxr, CEP, Context Kernel, or DomainForge.** Its integration is contract-level only (a name-mapping table at `developmental_memory.py:85-97` and `evidence_ingest.py:84-93`, plus the one-way NATS publish). The frozen loop doc assigns it navigation, developmental settlement, capability state, and memory writes — and it is **structurally disconnected** from everything that produces the evidence it would consume. It is not the integration façade; nothing is.
### 2.3 Authority map (one owner per stable responsibility, as actually implemented)

| Responsibility | Authoritative owner (implemented) | Evidence |
|---|---|---|
| Canonical semantics / concept identity | DomainForge | `domainforge-core/src/application/contract.rs:45` `ApplicationContract`; `semantic_pack/`; `concept_id.rs` |
| Semantic projection into SEA-Forge | `sea-forge-domainforge` | `DomainModelRef` with `identity_scheme_version` `v2-full-preimage` (`.agents/CURRENT_STATUS.md`, 2026-09-08) |
| Bounded context assembly | Context Kernel | `ck-core/src/entities/retrieval.rs` (`RetrievalQuery`, `Budget`, `RetrievalResult`, `ScoredItem`) |
| Authority verdict | `sea-forge-authority` | `Verdict`, `NormalizedDisposition`, `Determinism` (`sea-forge-core/src/types.rs:373-397`) |
| Execution containment | `sea-forge-runtime` + `sea-forge-sandbox` | landlock jail, workspace path safety (`sea-forge-sandbox/src/environment.rs`) |
| Agent/tool orchestration | Gauntlet | `RoleInvocation`; `BlastRadius` tier to `sandbox_policy_ref` required above `Contained` (`gauntlet-domain/src/role_invocation.rs:161-166`) |
| **Model invocation** | **split, ownerless** | Gauntlet `AgentRunner` (process/CLI) vs SEA-Forge `AgentProvider` (HTTP) — no single owner |
| Evidence capture | Gauntlet CAS; `sea-forge-evidence`; sxr | three incompatible shapes (§1.7) |
| Operational settlement | SEA-Forge | `SettlementClaim`, `BatchEvaluationResult`, `SettlementEvent`, `SettlementDeclaration` (`sea-forge-core/src/types.rs:742-1013`) |
| CEP conformance | sxr (runtime) + CEP (spec) | `sxr-core/src/embedded_schema.rs` pins the CEP envelope schema by sha256 |
| Developmental memory | **five owners, no boundary** | §1.7 |
| Capability promotion | **four implementations** | `sea-forge-capability/src/promotion.rs`; Gauntlet `promote/`; `godspeed_agent/agentic_capability_loop`; `SWE_SEED/learning/candidate.rs` |
| Adaptation / learning | Gauntlet `promote/` (measurement only) | `revision_evaluations` held-out + regression columns |
| Model weight change | **nobody** | `[FACT]` no training dependency, weight artifact, or update path in any of the 11 inspected repositories |

### 2.4 Model-invocation points (exhaustive over the inspected surface)

| # | Site | Transport | Boundary enforcement |
|---|---|---|---|
| 1 | `sea-forge-server/src/agent_probe.rs` | `AgentProvider::complete` to HTTP | `[FACT]` authority is evaluated **first**: `if decision.verdict != Verdict::Allow` (`:221`) and a second secret check (`:270`) both precede dispatch; failure settles `Rejected` (`:427`), never Accepted |
| 2 | `sea-forge-agent/src/delegation.rs:254` | `AgentProvider` | `[FACT]` `CompletionResponse.tool_calls` — every call re-enters authority; single-turn mode denies each and records the denial (`provider.rs:46-49`) |
| 3 | Gauntlet `verify/ladder.rs:431` `ModelRung` | `AgentRunner` via `RoleInvocationSpec` | `[FACT]` result must strict-parse `role_result@v1` carrying `verification_record@v1`; a model cannot wear a deterministic instrument's flag (`ladder.rs:31`) |
| 4 | `gauntlet-adapter-compare-model/src/lib.rs:1119` | `AgentRunner` (in-adapter) | scripted critic; emits a comparison envelope |
| 5 | `godspeed_agent/godspeed_nav/ml.py:12` | **not an LLM** | `PredictiveModelPort` with `heuristic`/`sklearn`/`testing`/`disabled`; default `heuristic` (`config.py:46`); explicit advisory framing (`ml.py:362-365`) |
| 6 | `edgeai` launcher to llama.cpp / vLLM | OpenAI-compatible HTTP | memory-aware admission; refuses a model that will not fit |

`[FACT]` **Sites 1-4 are all "model answers, code decides." Sites 1, 3 and 4 produce no uncertainty at all.** Site 5 is not model invocation despite the name `PredictiveModelPort` — a reader must not mistake it for a judgment provider.

### 2.5 State / evidence flow, frozen state, and replay

- **Where meaning originates:** DomainForge `.sea` to `ApplicationContract` / `ReferenceDefinition`; frozen per run as `reference_digest` (Gauntlet `runs.reference_digest`) and as `criteria_sha256` + `criteria_record_hash` (SEA-Forge `ApprovalRequest`, `types.rs:876-879`).
- **Where state freezes:** Gauntlet `checkpoints(iteration_no, reference_digest, unit_states, budget_spent, in_flight, event_log_offset)`; SEA-Forge `CasePlan` plus an immutable case-file version.
- **Where nondeterministic model judgment occurs:** only at §2.4 sites 1-4, and in every case the *decision function* is code, not the model.
- **Where deterministic logic applies:** `sea-forge-authority` policy evaluation; `sea-forge-settlement` acceptance; `sea-forge-capability` promotion arithmetic; rung ordering in `gauntlet-app/src/verify/ladder.rs`; Gauntlet `aggregate`/`residual`.
- **Who can cause side effects:** SEA-Forge runtime/sandbox only, behind `Verdict::Allow`; Gauntlet beneath that, within a `BlastRadius` tier.
- **What evidence survives:** Gauntlet CAS content-addressed objects plus `evidence_productions` provenance rows plus `observation_bindings`; SEA-Forge JSONL evidence plus MMR-committed ledger entries; sxr ledger rows.
- **How an event is replayed:** Gauntlet `event_log` offsets are recorded on `checkpoints`, `settlements`, and `evidence_productions`, so a run is reconstructible from its own database. A `reference.lock` exists per integrated tree. `[INFERENCE]` Replay is implemented *within* Gauntlet and **not across** repositories.
- **Where consequence becomes distinguishable from expectation:** Gauntlet `comparison` to `Residual` to `correction_selection` to `settlements`. This is the only place in the stack where expected-vs-observed is a typed artifact.
- **Where settlement occurs:** SEA-Forge `SettlementEvent` / `SettlementDeclaration`; Gauntlet `settlements`; sxr `settlement.registered`. Three settlements, one loop.
- **What can currently become developmental memory:** Gauntlet `capability_observations` / `capability_transitions` / `revision_*`; SEA-Forge `CapabilityRecord` and `MemoryItem`; the `godspeed_agent` ledger; SWE_SEED `learning/*` (**written, never read** — `route/`, `context/`, `hooks/`, and `skill/render.rs` never load learning artifacts).

### 2.6 Gauntlet persisted consequence chain (the calibration substrate, table by table)

`[FACT]` Reconstructed from the SQLite schema written by `crates/adapters/gauntlet-adapter-state-sqlite`:

```text
runs(run_id, created_at, reference_id, reference_version, reference_digest,
     intent, config_fingerprint, environment_fingerprint, parent_run, budget)
checkpoints(seq, run_id, iteration_no, reference_digest, unit_states,
            budget_spent, in_flight, event_log_offset)        <- state_before
records(seq, run_id, record_type IN {iteration, role_invocation, reference_version,
        observation, comparison, verification_record, residual, payment, failure,
        recovery_event, candidate_revision, promotion_decision, escalation,
        correction_selection}, payload, at)                  <- judgment + action
observation_bindings(run_id, iteration_no, evidence_ref, observation_digest)
settlements(run_id, unit_id, verification_ref, evidence_ref,
            reference_digest, event_log_offset, settled_at)   <- settlement
evidence_productions(run_id, evidence_ref, producer_name, producer_version,
                     locator, iteration_no, produced_at, event_log_offset,
                     content_digest)                          <- provenance
capability_observations(capability, class, run_id, fact, payload)
capability_transitions(capability, payload)
revision_proposals / revision_evaluations / revision_decisions / revision_restores
```

`[INFERENCE, high confidence]` The chain **state_before to judgment to action to observed consequence to settlement to capability** is already persisted, correlated by `run_id` plus `event_log_offset`, and replayable. The dataset the spec demands in `REQ-SETTLE-003` can be built **without reconstructing history from prose**.

Verified sample `verification_record@v1` payload, read directly from `targets/.runs/demo-calculator.settled-3of3-clean-seed-0730/state/gauntlet-state.db`:

```json
{"schema":"verification_record@v1","verification_ref":"vrf-0W57...","run_id":"XHK2...",
 "iteration_no":3,"unit_id":"AK-9011","claim":"calculator.html exists ...",
 "requirement":{"kind":"test_report","locator":"evidence/markup-check-report.json","producer":"instrument"},
 "instrument":{"name":"gauntlet-verify-test-report","version":"0.1.0","deterministic":true},
 "evidence_ref":"sha256:088421aa...","produced_at":1789437964,"outcome":"PASS","class":null,
 "detail":"decided by the test-report rung"}
```

`[FACT]` **Three gaps this sample exposes, all material:**

1. `outcome` is 3-valued with **no distribution and no confidence** — the judgment is already collapsed to a decision at the moment of recording.
2. `instrument{name,version,deterministic}` is present, but **model identity is not**. The parallel `role_invocation` sample carries `"model":null`, so the historical corpus **cannot attribute a judgment to a model**, which blocks `REQ-PROV-001` retroactively.
3. Across all 16 databases the `records` table contains only **three** kinds in practice — `role_invocation` (37), `correction_selection` (28), `verification_record` (17). In the database I sampled, `iteration`, `observation`, `comparison`, `residual` and `promotion_decision` are **ABSENT**. The *schema* is rich; the *content* is thin.

---

## 3. Spec Adversarial Review

Method: for every entity and requirement group in `GODSPEED_JUDGMENT_PLANE_SPEC.yaml` I searched the eleven repositories for an existing stable noun that already carries the distinction. Classifications: `keep` / `rename` / `merge` / `relocate` / `delete` / `already_exists` / `roadmap` / `unresolved`.

### 3.1 Proposed entities

| Spec entity | Classification | Evidence and reasoning |
|---|---|---|
| `DecisionSurface` | **keep, but demote to a derived artifact — not a first-class compiled zone** | No equivalent exists. But `domainforge-core/src/application/contract.rs` already carries `ApplicationContract{OperationContract, PolicyBinding, EvidenceKind, IdempotencyStrategy, ConcurrencyStrategy, FailureContract}` and `gauntlet-domain/src/reference.rs:391,412` carries `ReferenceDefinition`/`Reference` with `QualityDimension`, `Assumption`, `Unknown`. A surface is therefore **materializable** from two existing typed sources. Spec itself defers compilation to `roadmap` (`:1410-1416`) and says no `.sea` syntax is required now — consistent with evidence. |
| `Question` | **already_exists — extend CEP-0005, do not create a GodSpeed Question** | `cep/spec/CEP-0005-question-model.md` defines Question, Question Identity, Question Scope, Question Types, Answer Shapes, Question-Evidence binding, and question lifecycle; `sxr` enforces the envelope with the schema hash-pinned at `sxr-core/src/embedded_schema.rs:15`; the registry already has a `question.declared` record kind with `authority_class = declared evaluation context`. Creating a second Question noun **duplicates a stable, already-pinned protocol noun**. |
| `JudgmentRequest` | **already_exists under a different name — `RoleInvocationSpec` + `ContextEnvelope`** | `gauntlet-ports/src/agent_runner.rs:267-284`: `invocation_id, run_id, iteration_no, role, prompt, context_envelope, workspace, permissions, result_contract_path, deadline, model_hint, sandbox_policy, budget_slice`. The request is already immutable-by-construction (fields are set at admission) and `context_envelope.digest()` is already the context hash. The **only** spec field with no counterpart is `decision_surface_hash` / `semantic_pack_hash`, which S2 supplies. |
| `JudgmentProvider` | **delete as a GodSpeed port — two provider ports already exist** | `gauntlet_ports::AgentRunner` (with `InvocationHandle`, `InvocationStatus`, `Deadline`, `CancelReason`, `RunnerCapabilities`) and `sea_forge_agent::AgentProvider` (with `AgentError` x10, `Usage`). A third port would be a third abstraction over the same capability and would re-open the question of which one owns cancellation, budgets, and typed failure — all already settled. |
| `Judgment` | **rename — unresolved and urgent** | The noun is occupied twice by deterministic semantics (`ManagerJudgment`, `judge_snapshots`) and once by a prompt-level notion (`VERIFIER_PROMPT`: a verifier answers exactly one question per claim). A probabilistic `Judgment` placed beside `ManagerJudgment` in the same crate tree is a **name collision with opposite semantics**. Recommendation: name the probabilistic artifact explicitly, e.g. `BoundedJudgment` / `Inference`, and **rename or namespace the existing deterministic uses** as `CaseStateClassification`. Recorded as UNRESOLVED (§5, unresolved list) because it is a public-interface naming decision and therefore an ask-first change. |
| `JudgmentObservation` | **merge into the existing judgment record — do not create a new type** | `gauntlet_domain::VerificationRecord` already binds *run + claim(unit) + evidence + outcome + instrument* one-to-one-to-one-to-one, and is already persisted as `verification_record@v1` in a dedicated `record_type` slot. RealityTrace's `claim.made` (`authority_class = asserted, not evidenced`) and `diagnostic.hypothesis` (`authority_class = hypothesis, not verdict`) already express the same epistemic status at the CEP layer. The genuinely missing fields are **distribution, model identity, and semantic hashes** — additive fields on an existing type, not a new type. |
| `JudgmentComposer` | **delete — composition already exists deterministically in two places** | `sea-forge-settlement` composes `SettlementClaim.evaluator_scores: BTreeMap<String,f64>` plus `BatchEvaluationResult{total, passed, pass_ratio, min_pass_ratio, failures}` into `SettlementEvent`; `gauntlet-app` composes observations into `ComparisonRecord`/`Residual`. A `JudgmentComposer` would be a **third aggregator**. The real gap is that thresholds live in code, not in a versioned inspectable file (`REQ-COMP-002`). |
| `CaseAssessment` | **delete as a new type — `SettlementClaim` + `SettlementEvent` already are it** | `SettlementClaim` already carries `criteria`, `execution`, `authority_verdicts`, `evaluator_scores`, `batch`, `write_only`; `SettlementEvent` carries `status`, `basis`, `review_required`. Adding `CaseAssessment` would create a second assessment type with no owner and force every consumer to choose. |
| provider metadata / provenance | **keep as additive fields, not a new entity** | Gauntlet already persists `producer_name`, `producer_version`, `locator`, `content_digest` in `evidence_productions`, and `instrument{name,version,deterministic}` on the record. `sea_forge_agent::Usage{input_tokens,output_tokens,total_tokens}` already gives cost provenance. Missing only: model identity on the record (historically `null`) and the surface/semantic hashes. |
| uncertainty representation | **genuinely missing — this is the one real new requirement** | `[FACT]` No distribution, probability, confidence-interval, or abstention type exists in SEA-Forge, Gauntlet, sxr, or the CEP machine-enforced schema. CEP carries `uncertainty_record` and `confidence` on *Representation* fields only, and both are unconstrained `object`/`string` in the schema. This is the narrow true gap. |
| replay identity | **genuinely missing at the loop level; exists within each component** | Gauntlet has `context_envelope_digest` and `event_log_offset`; SEA-Forge has `Determinism{policy_bundle_hash, action_request_hash, identity_binding_hash}`; sxr has `snapshot_id` + `checkpoint` hash. **No shared value survives the round trip.** This is S4. |
| settlement correlation | **partially exists — inside Gauntlet only** | `settlements(run_id, unit_id, verification_ref, evidence_ref, reference_digest, event_log_offset)` correlates judgment to settlement **within Gauntlet**. Nothing correlates a Gauntlet settlement to a SEA-Forge `SettlementEvent` or a sxr `settlement.registered` row. |

### 3.2 Requirement groups

| Group | Requirement IDs | Classification | Reasoning |
|---|---|---|---|
| `authority_separation` | AUTH-001..005 | **keep — and strengthen with an executable test** | `[FACT]` The code already does what AUTH-001/002/005 demand: `agent_probe.rs:221` checks `Verdict` before dispatch; failure settles `Rejected` (`:427`). But there is **no test in the stack that asserts a model provider cannot widen authority**, so the requirement is satisfied by construction today and unproven tomorrow. |
| semantic bounding | JUDG-001..003 | **keep; JUDG-003 unresolved** | `sema-` bounding requires the surface to be resolvable. `sea-forge-domainforge` already carries `DomainModelRef` with a versioned identity scheme and a `CandidateDisposition`, and the 2026-09-08 remediation fixed an earlier identity defect (§4). What is unresolved is what happens when a surface references a semantic concept that the pinned `semantic_pack_hash` does not contain. |
| deterministic composition | COMP-001..003 | **keep; relocate thresholds to a versioned file** | COMP-001 is already true of `sea-forge-settlement`. COMP-002 is **violated today**: thresholds are code constants (e.g. `score >= 0.5` at `sea-forge-settlement/src/lib.rs:373`), not an inspectable versioned artifact. COMP-003 (explicit contradiction path) has no counterpart in code. |
| provenance and replay | PROV-001..004 | **keep; PROV-001 is retroactively unsatisfiable for historical data** | `[FACT]` Historical `role_invocation` rows carry `model: null`, so `REQ-PROV-001` cannot be satisfied for existing runs. Forward-looking it is achievable with one field. PROV-004 (append-or-supersede, never destructive overwrite) is already the design of every ledger in the stack. |
| settlement linkage | SETTLE-001..003 | **keep; SETTLE-001 requires S4** | Without `loop_correlation_id` there is no reliable join key. SETTLE-002 is a discipline requirement and is already respected. SETTLE-003 is *nearly* satisfied by Gauntlet's schema and blocked only by the absence of pre-consequence uncertainty. |
| safe failure | SAFE-001..005 | **keep; verify against the existing `AgentError` taxonomy** | `AgentError{InvalidRequest, UnsupportedKind, Unreachable, Timeout, Http4xx, Http5xx, Redirect, SchemaInvalid, Oversize, Transport}` already covers the required typed failures. `Unreachable`/`Timeout` already map to no-ALLOW. |
| security / trust boundaries | SEC-001..005 | **keep; SEC boundary partially enforced** | `NetworkPolicy::validate_destination` enforces HTTPS for production and rejects private/reserved destinations unless the loopback test flag is set — a genuinely good existing boundary, and a **deployment note for the Jetson** where the launcher is loopback. Authn/authz to the model endpoint is provider-local (`credential_ref`) and there is **no tenancy model**. |
| verification / proof | VERIFY-001..005 | **keep; the proof surface is the weakest part** | No CI on Gauntlet; no calibration artifact anywhere; `.agents/` evidence exists for SEA-Forge and edgeai but not for a judgment capability. |
| variation / repeatability | VAR-001..008 | **keep; substantially pre-existing** | Gauntlet already models variation (`BudgetDimension`, `correction_selection`, `variation_tags` on `SettlementDeclarationRequest`) and SEA-Forge's capability promotion already requires repeated settled variation (`CapabilityQualifying`, `CapabilityVariation`, `CapabilityRecovery`, `CapabilityOrchestration`). |
| recovery | RECOV-001..004 | **keep; pre-existing** | Gauntlet `recovery_event` record kind, `Replay` via `event_log_offset`, `revision_restores`. |
| capability / composition groups COMP-*, CONFIG-*, GOAL-*, NONGOAL-*, CLAIM-*, OUT-* | — | **keep as written** | `claim_discipline` and `must_report_incomplete_blocked_or_failed_when` are unusually well-constructed and are the reason this review can be adversarial at all. No change recommended. |

---

## 4. Responsibility Matrix

One authoritative owner wherever the evidence permits; where it does not, that is itself the finding.

| Responsibility | Current owner(s) | Proposed owner | Evidence | Duplication | Change required |
|---|---|---|---|---|---|
| Canonical concept identity | DomainForge | DomainForge (unchanged) | `.sea` parser, `semantic_pack`, `concept_id.rs` | none | none |
| Semantic projection + identity scheme | `sea-forge-domainforge`, `sxr-df` | `sea-forge-domainforge` canonical; `sxr-df` a consumer | `DomainModelRef` v2-full-preimage; `CliDomainForgeAdapter` | 2 adapters, 1 meaning | none (remediated 2026-09-08) |
| Bounded context assembly | Context Kernel; SWE_SEED `context/pack.rs`; NeatCode `context.mjs`; godspeed_agent `developmental_memory.ingest` | **Context Kernel** | `ck-core/entities/retrieval.rs`; MCP `context_required` | **4 implementations** | declare CK canonical; do not delete the others (different scopes) |
| Semantic question declaration | CEP `Question` (spec) + sxr `question.declared` | **CEP-0005 + sxr** | `embedded_schema.rs:15`; `record_kinds.csv` | none | **additive** fields: answer domain, scale, probability |
| Pre-consequence judgment record | Gauntlet `VerificationRecord`; sxr `claim.made` / `diagnostic.hypothesis`; sea-rs `EvidenceRecord` | **Gauntlet** (runtime judgment) with sxr as the CEP projection | `verification_record@v1`; `record_kinds.csv` | 3 shapes, 1 meaning | **additive** fields: distribution, model identity, surface hash |
| Judgment invocation | Gauntlet `AgentRunner`; sea-rs `AgentProvider` | **Gauntlet `AgentRunner`** as the judgment path | `agent_runner.rs:440-470`; `provider.rs:71-83` | **2 ports, disjoint transports** | add **one** HTTP `AgentRunner` adapter reusing sea-rs transport code |
| Provider failure taxonomy | `sea_forge_agent::AgentError`; Gauntlet `AgentError` | keep both (different transports) | `provider.rs:85-97` | 2 taxonomies | none — but map them explicitly in the adapter |
| Deterministic composition to assessment | `sea-forge-settlement`; Gauntlet `aggregate`/`residual` | **`sea-forge-settlement`** (operational), Gauntlet (intra-run) | `types.rs:742-800` | 2 aggregators, different scopes | move thresholds to a versioned rule file |
| Authority verdict | `sea-forge-authority` | `sea-forge-authority` (unchanged) | `types.rs:373-397` | none | none |
| Governed execution | `sea-forge-runtime` / `sea-forge-sandbox` | unchanged | landlock jail | none | none |
| Agent/tool orchestration | Gauntlet | Gauntlet (unchanged) | `RoleInvocation`, `BlastRadius` tiers | none | none |
| Evidence capture | Gauntlet CAS; `sea-forge-evidence`; sxr | Gauntlet CAS for run evidence; sea-rs for governed-run evidence; sxr for CEP envelopes | `evidence_productions`; `EvidenceRecord`; `record_kinds.csv` | 3 shapes | **define which one a settlement cites** (no rewrite) |
| Operational settlement | `sea-forge-settlement` | unchanged | `SettlementEvent`, `SettlementDeclaration` | 3 settlement types across repos | none — scope them explicitly |
| Loop correlation identity | **none** | **new field bound at case admission** | no shared type exists | n/a | **S4 — one field, four echo sites** |
| Developmental memory (durability) | `sea-forge-ledger` (MMR + witness) | unchanged | `types.rs` `MmrState`, `WitnessReceipt` | 5 ledgers | ownership decision only |
| Developmental memory (run consequence corpus) | Gauntlet SQLite | **Gauntlet** — promoted to canonical replay substrate | 16 DBs, `event_log` + `settlements` | never exported | declare canonical; add read-only export |
| Developmental memory (navigation/affordance) | `godspeed_agent` LedgerStore | unchanged, but **must consume** settlement | `storage.py`; no consumer of its ledger exists | disconnected | wire a read path or mark it not-in-loop |
| Adaptation (measurement) | Gauntlet `promote/` | **Gauntlet** | `revision_evaluations` held-out + regression | 4 promotion implementations | none — point it at judgment |
| Capability promotion | `sea-forge-capability`; Gauntlet `promote/`; godspeed_agent; SWE_SEED `learning/candidate.rs` | **`sea-forge-capability`** for capability claims; Gauntlet for run-level measurement | `promotion.rs` `CapabilityPromotionPolicy`; `can_promote_candidate` | **4** | declare the boundary; SWE_SEED learning artifacts are currently **inert** |
| Model weight change | **nobody** | nobody (explicit non-goal for now) | no training dep anywhere | n/a | record as explicit non-goal |
| CEP conformance checking | sxr runtime + CEP validator | unchanged | `embedded_schema.rs`; `cep/src/cep/` | none | none |
| Repo/tooling hygiene | NeatCode | unchanged | `lib/`, `bin/`, 0 deps | none | none — **not on the runtime path** |

---

## 5. Missing Capability Matrix

Production classification: `MUST_HAVE_BEFORE_PRODUCTION` / `SHOULD_HAVE_SOON` / `OPTIONAL` / `NOT_RELEVANT`.

| Capability | Current state | Production requirement | Gap | Evidence | Cheapest valid solution | Value | Cost | Uncertainty |
|---|---|---|---|---|---|---|---|---|
| Bounded typed question with declared answer domain | **ABSENT** | Questions must declare allowed values | no field in CEP Question schema or any Rust type | `cep-semantic-envelope.schema.json:1385`; grep for probability/confidence across all repos: 0 typed hits | Extend CEP-0005 additively with `answer_domain` | **critical** | low | low |
| Uncertainty-preserving judgment output | **ABSENT** | Distributions or abstention, never a bare verdict | `VerificationRecord.outcome` is 3-valued; `verification_record@v1` has no probability | `verification.rs:26-33`; live DB sample | Additive `distribution` field on the existing record | **critical** | low | low |
| Loop correlation identity | **ABSENT** | One join key across meaning, judgment, authority, execution, settlement | no shared type; correlation by string convention | §2.5; no `loop_correlation_id` anywhere | **S4**: one field + four echo sites | **critical** | **low** | low |
| HTTP model provider on the judgment path | **ABSENT in Gauntlet; PRESENT in sea-rs** | Reach a local OpenAI-compatible endpoint | `CliAgentRunner` is process-only | `gauntlet-adapter-agent-cli/src/config.rs:4-15`; `provider.rs:154-184` | One adapter reusing `OpenAiCompatibleProvider` | **high** | **low** | low |
| Model identity on judgment records | **ABSENT (historically `null`)** | Every judgment cites provider + model | `role_invocation.model` is `null` in the corpus | live DB sample | Populate on write, forward-only | **high** | low | low |
| Versioned composition thresholds | **ABSENT** | Rules inspectable outside model prompts | thresholds are code constants | `sea-forge-settlement/src/lib.rs:373` (`score >= 0.5`) | Versioned config file consumed by existing path | **high** | low | low |
| Explicit contradiction / uncertainty path | **ABSENT** | Conflicting judgments produce an explicit path | no such branch in composition | `REQ-COMP-003`; no code counterpart | One explicit enum arm | high | low | low |
| Calibration artifact | **ABSENT** | Reliability measured against consequence | no Brier/ECE/reliability output anywhere | no calibration code in any repo | Read-only replay harness + report | **high** | **medium** | **medium** |
| Consequence-grounded judgment evaluation | **PARTIALLY EXISTS** (for capability, not judgment) | Judgment scored against recorded settlement | `promote/` exists but is not pointed at judgment | `revision_evaluations`; 17 settlements | Reuse `promote/`'s control/treatment shape | **high** | medium | low |
| CI on the judgment-corpus owner | **ABSENT** | Required gates protect the corpus | no `.github/workflows` in Gauntlet | directory absent | Add CI invoking existing `just` recipes | **critical** | low | low |
| Green full-suite gate on the authority crate | **RED (unrelated test)** | Server gate must be green | `conformance_run_locator::both_layouts_still_resolve_after_a_restart` fails | `.agents/CURRENT_STATUS.md` 2026-08-31 | Fix cell-lock test isolation | **high** | low | medium |
| Provider-substitution proof | **PARTIALLY EXISTS** | Two materially independent providers must prove the contract | contract suite exists per port; **no cross-port substitution test** | `agent-cli/src/lib.rs:59`; `provider_contract.rs` | One test running the same surface through both adapters | high | low | low |
| Authority non-bypass test for judgment | **ABSENT** | Model output can never widen authority | enforced by construction, unproven by test | `agent_probe.rs:221` (no adversarial test) | Adversarial test: provider claims ALLOW, authority must still DENY | **critical** | low | low |
| Idempotency / retry semantics for judgment | **PARTIALLY EXISTS** | Bounded retries, no duplicate judgments | Gauntlet `InvocationHandle` + idempotent `cancel`; TypeSafe reference shows retry accounting | `agent_runner.rs:458-465`; `system_one_adapter/_response.py:15-22` | One retry-policy config with attempt provenance | high | low | medium |
| Timeout / cancellation provenance | **EXISTS** | Typed timeout failure, cancellable | `AgentError::Timeout`; `cancel(&self, handle, reason, deadline)` | `provider.rs:90`; `agent_runner.rs:458-465` | none | — | — | low |
| Semantic-hash binding of the judgment surface | **ABSENT** | Surface hash in the request | no `decision_surface_hash` field | `RoleInvocationSpec` field list | Additive field (S2) | high | low | low |
| Tenancy / authz for judgment | **NOT_RELEVANT today** | Only if exposed beyond the box | no tenancy model; loopback-only on target | `NetworkPolicy` | none — revisit on exposure | low | — | medium |
| Telemetry / metrics | **PARTIALLY EXISTS** | Operable | `tracing-subscriber` in sea-rs; `GAUNTLET_OTEL_ENDPOINT` env exists but **no OTel dependency found**; no Prometheus anywhere | `sea-rs/Cargo.toml:56`; `gauntlet/.env.example` | Declare the metrics that matter (judgment latency, abstention rate, calibration drift) | medium | medium | medium |
| Retention / correction history | **EXISTS by design** | Never destructively overwrite | append-only across all five ledgers | `record_kinds.csv` `allowed_predecessors`; ledger types | none | — | — | low |
| Migration / schema versioning | **PARTIALLY EXISTS** | Compatible evolution | sea-rs `DomainModelRef` `#[serde(default)]` back-compat; Gauntlet `schema_migrations` table | `schema_migrations(version, name)` in live DB | Reuse the same pattern for additive judgment fields | medium | low | low |
| Restart / recovery / resume | **EXISTS** | No lost work | Gauntlet `checkpoints`; SEA-Forge resume command; server restart settlement | `checkpoints` table; `sea-forge-cli/src/commands/resume.rs` | none | — | — | low |
| Backpressure / admission | **EXISTS (SEA-Forge)** | Bounded concurrency | `max_concurrent_runs` + 8-request waiting room | `.agents/CURRENT_STATUS.md` 2026-08-31 | extend to judgment invocations | medium | low | low |

---

## 6. System Convergence Recommendation

### 6.1 Current to target, derived from evidence

```text
CURRENT
  meaning      : DomainForge .sea -> ApplicationContract / Reference       [KEEP]
  representation: Context Kernel (MCP, embeddings off)                      [KEEP]
  judgment      : Gauntlet ladder rungs 1-4 deterministic, rung 10 = model,
                  3-valued, NO distribution; sea-rs judge() 4-way deterministic
                  2 disjoint provider ports; no cross-repo identity         [EXTEND]
  composition   : sea-forge-settlement evaluator_scores (>0.5, in code)
                  + Gauntlet aggregate/residual                            [KEEP, EXTERNALISE RULES]
  authority     : sea-forge-authority Verdict{Allow,Deny,Escalate}          [KEEP]
  execution     : sea-rs runtime/sandbox + Gauntlet orchestration           [KEEP]
  observation   : Gauntlet CAS + evidence_productions + observation_bindings
                  sea-forge-evidence JSONL; sxr evidence.created            [KEEP, SCOPE]
  settlement    : sea-forge-settlement; Gauntlet settlements; sxr
                  settlement.registered                                     [KEEP, SCOPE]
  memory        : 5 ledgers, no owner boundary                              [DECIDE OWNERSHIP]
  adaptation    : Gauntlet promote/ (capability only), no calibration       [POINT AT JUDGMENT]

TARGET (minimum coherent)
  meaning       DomainForge                         UNCHANGED
  representation Context Kernel                      UNCHANGED
  questions     CEP-0005 Question, additively extended (answer domain, scale)
  surface       DecisionSurface = DERIVED artifact from
                ApplicationContract + Reference      NO NEW STORE, NO COMPILER
  judgment      Gauntlet VerificationRecord@v1 + additive
                {distribution, provider_identity, model_identity, surface_hash}
                via AgentRunner; ONE new HTTP adapter reusing
                sea_forge_agent::OpenAiCompatibleProvider (Jetson :18001)
  composition   sea-forge-settlement, thresholds from a VERSIONED RULE FILE
  authority     sea-forge-authority                    UNCHANGED - judgment is evidence
  execution     UNCHANGED
  observation   UNCHANGED (scope declared per product)
  settlement    UNCHANGED (scope declared per product)
  correlation   loop_correlation_id  <-- THE ONE NEW CROSS-CUTTING FIELD
  memory        one owner per purpose, declared
  adaptation    read-only consequence-grounded replay harness + calibration report
  training      EXPLICITLY NOT BUILT
```

### 6.2 What changes, what does not, what is deliberately not built

**Changes (S1-S5):** one adapter; one additive semantic extension; one versioned rule file; one correlation field; one ownership decision.

**Explicitly NOT built (with reasons):**

| Not built | Why |
|---|---|
| A judgment runtime/service/daemon | The judgment path is a library call inside an existing process. A service adds deployment, auth, tenancy, and failure modes for zero capability. The Jetson is one box. |
| A third provider port | Two ports already exist with typed failures, cancellation, budgets, and contract suites. A third re-opens settled questions. |
| `CaseAssessment` / `JudgmentComposer` / `JudgmentObservation` as new types | `SettlementClaim`+`SettlementEvent`, `sea-forge-settlement`, and `VerificationRecord` already carry these distinctions. New nouns would create a second truth and force consumers to choose. |
| `.sea` grammar changes | Spec defers this to roadmap (`:1410-1416`) and the evidence supports it: `ApplicationContract` + `Reference` are sufficient to materialize a surface. |
| A new event schema or CEP version bump for the first slice | sxr pins the CEP envelope schema by sha256 (`embedded_schema.rs:15`). A version bump breaks a hash-pinned conformance contract for no gain in slice 1. |
| A new database or message broker | Postgres, NATS, PGMQ, ruVector are absent from the judgment path today. Gauntlet's SQLite already holds the consequence chain. Introduce infrastructure only against a proven need. |
| A training pipeline or weight updates | No mechanism exists and none is needed to prove bounded judgment. `[FACT]` nothing in 11 repositories changes weights. Claiming learning today would be false. |
| An "Agent Memory Ledger" service as a fourth ledger | Five ledger implementations already exist. Adding the sixth is the wrong direction; one ownership decision is the right one. |
| Deleting, rewriting, or re-languaging any repository | Every repository inspected is internally coherent. Fragmentation is a *wiring* problem, not a *code* problem. |
| DomainForge DecisionSurface compilation | ROADMAP. Requires evidence that a derived artifact is insufficient first. |

### 6.3 On the fifteen adversarial propositions (falsification attempt)

The full table with `evidence_for`, `evidence_against`, `conclusion`, `confidence`, and `what new evidence would change the conclusion` is the deliverable `2026-09-16-godspeed-spec-delta.yaml` appendix, summarised here:

| # | Proposition | Verdict | Confidence |
|---|---|---|---|
| 1 | GodSpeed needs a new Judgment Plane | **TRUE, but narrow** — needs bounded questions + uncertainty + correlation, not a plane | high |
| 2 | Judgment Plane deserves its own runtime component | **FALSE** | high |
| 3 | `JudgmentObservation` deserves a new type | **FALSE** — merge into `VerificationRecord` | high |
| 4 | `CaseAssessment` deserves a new type | **FALSE** — `SettlementClaim`+`SettlementEvent` | high |
| 5 | DomainForge should compile DecisionSurfaces | **PREMATURE** — derive first | medium-high |
| 6 | SEA-Forge is the correct authority owner | **TRUE** | high |
| 7 | Gauntlet is the correct execution/orchestration owner | **TRUE**, with the caveat that it has no CI | high |
| 8 | RealityTrace is the correct consequence/evidence owner | **TRUE in protocol, UNPROVEN in runtime** — sxr has no live correlation to Gauntlet or sea-rs | medium |
| 9 | Agent Memory Ledger is the correct developmental-memory owner | **FALSE as stated** — the correct answer is one owner per purpose with Gauntlet canonical for the run-consequence corpus | medium-high |
| 10 | Existing event schemas can remain stable | **TRUE for slice 1** (`verification_record@v1` and the CEP envelope both stay) | high |
| 11 | A local Qwen provider approximates the interface sufficiently for development | **TRUE** — the OpenAI-compatible surface already parses, and it is the *same* protocol the Jetson serves | high |
| 12 | Jev can eventually be added as a pure adapter | **TRUE** — because the required contract is a bounded question with a typed answer, and TypeSafe's own primitive is `Choice{criteria}` to `ChoiceAnswer{choice, confidence, probabilities}` | medium-high |
| 13 | Historical Gauntlet data is sufficient for calibration | **FALSE** — 17 settled judgments; sufficient to prove the mechanism, insufficient to calibrate | **high** |
| 14 | The stack is close enough to production that convergence beats simplification | **TRUE**, with three named exceptions: Gauntlet CI, the red `sea-forge-server` gate, and the absence of a correlation identity | medium-high |
| 15 | Separate repositories provide useful modularity rather than accidental fragmentation | **MIXED** — genuinely useful for DomainForge, SEA-Forge, Gauntlet, Context Kernel; **accidental** for the five ledgers, four capability promotors, and the two disjoint provider ports | medium |

---

## 7. Judgment Plane Minimum Slice

**Goal: prove or disprove the bounded-judgment hypothesis using data that already exists, at the lowest irreversible cost. This section is a specification for evidence, not an implementation plan.**

### 7.1 Exact existing types to reuse (do not re-create)

| Need | Reuse | Location |
|---|---|---|
| Invocation boundary + cancellation + budgets | `AgentRunner`, `RoleInvocationSpec`, `InvocationHandle`, `InvocationStatus`, `Deadline`, `CancelReason`, `RunnerCapabilities` | `gauntlet-ports/src/agent_runner.rs:240-470` |
| Context identity of the judgment | `ContextEnvelope{body, digest}`, `RoleInvocationSpec.context_envelope` | `gauntlet-ports/src/agent_runner.rs:240-258,267-284` |
| Persisted per-invocation context digest | `RoleInvocationDefinition.context_envelope_digest` | `gauntlet-domain/src/role_invocation.rs:110-131` |
| Judgment record | `VerificationRecord` (run + claim + evidence + outcome + instrument) | `gauntlet-domain/src/verification.rs:26-33` |
| Persisted record slot | `record_type = verification_record`, schema `verification_record@v1` | live DB; `gauntlet-ports/src/verification_record.rs` |
| Deterministic-before-model ordering | `RungKind`, `Ladder::select_and_verify`, `MODEL_RUNG_NAME` | `gauntlet-app/src/verify/ladder.rs:71-104,276` |
| Role vocabulary (already includes the needed roles) | `RoleId{Orchestrator,Builder,Critic,Verifier,Observer,Optimizer,Custom}` | `gauntlet-domain/src/values.rs:617-625` |
| Deterministic composition into settlement | `SettlementClaim.evaluator_scores`, `BatchEvaluationResult`, `SettlementEvent` | `sea-forge-core/src/types.rs:742-800` |
| Authority boundary the judgment must not cross | `Verdict`, `NormalizedDisposition`, `AuthorityAction` | `sea-forge-core/src/types.rs:176,373-397` |
| HTTP provider transport | `OpenAiCompatibleProvider`, `EndpointSnapshot`, `NetworkPolicy`, `AgentError`, `Usage` | `sea-forge-agent/src/provider.rs`, `network.rs`, `config.rs` |
| CEP question + judgment semantics | `Question` / `claim.made` / `diagnostic.hypothesis` record kinds | `cep/spec/CEP-0005-question-model.md`; `sxr-core/src/record_kinds.csv` |
| Consequence-grounded evaluation shape | `PromotionGate`, `held_out_control/treatment`, `regression_control/treatment` | Gauntlet `promote/gate.rs`, `revision_evaluations` table |
| Historical corpus | 16 run DBs | `gauntlet/targets/.runs/*/state/*.db` |

### 7.2 Exact new types truly required

Only **four** new things, three of which are fields rather than types:

1. **`DecisionSurface`** — derived, not authored: `{id, version, semantic_pack_hash, questions: Vec<QuestionRef>}`. Materialized from `ApplicationContract` + `ReferenceDefinition`. Stored as a file artifact with a content hash; **not** a database, **not** a compiled zone.
2. **`AnswerDomain`** — the missing CEP-0005 additive field on a Question: `Choice{labels: Vec<String>} | Score{min, max, legend} | Proposition{semantics_uri}`. One field, three variants. This is the single genuinely new semantic object in the whole review.
3. **`distribution`** — additive field on `verification_record@v1`: `BTreeMap<label, f64>` plus an explicit abstention arm. Preserves uncertainty instead of collapsing it to `PASS|FAIL|INCONCLUSIVE`.
4. **`loop_correlation_id`** — one string bound at case admission and echoed by sea-rs `run_id`, Gauntlet `run_id`, sxr snapshot/run refs, CEP envelope scope.

`[ASSUMPTION]` I assume the CEP maintainers will accept an **additive** envelope-section change. If they will not, the fallback is to carry `AnswerDomain` inside the Gauntlet `result_contract_path` document for slice 1 and defer the CEP extension — which costs nothing in capability and is strictly reversible.

### 7.3 Exact integration seam

```text
sea-forge-server / gauntlet-app
        |
        | 1. resolve DecisionSurface (derived file, hash-checked against semantic_pack_hash)
        v
RoleInvocationSpec { role: Verifier|Critic|Custom, prompt, context_envelope,
                     result_contract_path, deadline, model_hint, sandbox_policy, budget_slice }
        |
        | 2. AgentRunner::invoke  (NEW: HttpAgentRunner adapter)
        |      -> HTTP POST /v1/chat/completions  on the Jetson launcher 127.0.0.1:18001
        v
role_result@v1  ->  strict-parsed  ->  verification_record@v1
        |            (+ distribution, provider_identity, model_identity, surface_hash)
        | 3. deterministic composition (versioned rule file) on the distribution
        v
verification_record[outcome, distribution, instrument{deterministic:false}]
        | 4. existing persistence
        v
records(record_type='verification_record')   and   settlements(...)
```

**One new crate-level artifact, at most:** an HTTP `AgentRunner` in `gauntlet/crates/adapters/gauntlet-adapter-agent-http`, ~300-500 lines including tests, reusing `sea_forge_agent` transport where the crate graph allows and reimplementing the same 3 HTTP calls where it does not (the wire shape is 40 lines). **No new port trait. No new service. No schema replacement.**

### 7.4 Provider boundary

- **Provider A (existing):** `CliAgentRunner` with `GAUNTLET_CLI_AGENT_CMD` — already real, already contract-suite tested, no default and no fallback.
- **Provider B (new):** HTTP to `http://127.0.0.1:18001/v1` — the Jetson launcher, local Qwen. Reuses `OpenAiCompatibleProvider` semantics; requires `allow_loopback_test` semantics for loopback HTTP, exactly as `NetworkPolicy` already models.
- **Provider C (deterministic replay):** `MockAgentRunner` from scripted fixtures — this is the *calibration* provider and the reason the slice is cheap.
- **Provider D (future, roadmap):** TypeSafe Jev/System One as a pure adapter, **if and only if** the stable contract is `AnswerDomain -> distribution`. Verified externally: TypeSafe's primitive is `ChoiceModel{type:'choice', instructions, criteria: Mapping[label, description|None]}` answering `ChoiceAnswer{choice, confidence, probabilities: dict[label,float]}`, and `ScoreModel` answering `ScoreAnswer{score, confidence, legend: dict[int,str], probabilities: dict[int,float]}`. That is **structurally the same shape** as `AnswerDomain` to `distribution`, which is why proposition 12 survives. `[FACT]` The same shape also exists in `system_one_adapter/_response.py` with `Usage{input_tokens_total, output_tokens_total, n_retries, n_retries_malformed_structure, latency}` — the retry-accounting fields are the useful implementation idea worth copying.
- `[WARNING]` **Do not adopt TypeSafe nouns.** `Noul`, `Choice`, `Score` are provider-local. GodSpeed's neutral expression is *question*, *answer domain*, *distribution*.

### 7.5 Evidence boundary, replay procedure, first decision surface

**Evidence boundary.** Every judgment in the slice must persist: request identity, surface hash, context digest, provider identity, model identity, distribution, the rule version that composed it, and the settlement it is later correlated with. All of these already have homes except provider/model identity and surface hash.

**Replay procedure (read-only, no new infra):**

1. For each of the 16 run DBs, read `checkpoints` to recover state_before and `reference_digest`.
2. Read `verification_record` rows and their `evidence_ref` content from `evidence/objects/`.
3. Re-present each record's `claim` + `requirement` as a bounded Question with `AnswerDomain::Choice{SUFFICIENT, INSUFFICIENT, INCONCLUSIVE, UNDECIDABLE}`.
4. Ask Provider C (scripted) and Provider B (Jetson Qwen) for a **distribution** over that domain.
5. Compose against a versioned threshold to a 3-valued outcome using the existing rule shape.
6. Compare composed outcome against the **recorded** `settlements` row — the ground truth that already exists.
7. Emit a calibration report: agreement rate with the deterministic rung, agreement with the eventual settlement, Brier score, abstention rate, and provider disagreement rate.

**First decision surface (recommended):** *does the evidence at the declared locator satisfy the declared requirement?* Reasons: it is already the exact question `VERIFIER_PROMPT` asks; it already has a declared closed answer set; it already has ground truth in `settlements`; and it is the highest-frequency judgment in the corpus (17 `verification_record` rows). A second surface — *is this residual a genuine expectation violation or noise?* over `residual` rows — is the natural follow-on.

### 7.6 Success and failure criteria (falsifiable)

**The slice succeeds if and only if all of:**

1. **Mechanism:** a bounded Question with a declared `AnswerDomain` is resolved from a hash-pinned surface, answered by two materially independent providers, and persisted without schema replacement.
2. **Composition determinism:** for identical record distributions and rule version, the composed outcome is bit-identical across runs.
3. **Correlation:** every judgment is joinable to its later Gauntlet settlement and, through `loop_correlation_id`, to its SEA-Forge authority decision.
4. **Authority non-bypass:** an adversarial provider returning an allow-shaped answer **cannot** produce `Verdict::Allow` without the authority engine independently agreeing. (This is a new test, and it is the single most important test in the slice.)
5. **Calibration signal:** on the historical corpus, the distribution carries information the deterministic rung does not — measurably better agreement with recorded settlement than a constant answer, **or** a demonstrated failure mode that falsifies the hypothesis.

**The slice fails (and the hypothesis is falsified) if:** the distribution carries no information over the existing 3-valued rung; or provider B and provider C disagree so completely that the surface is not stably answerable; or composition cannot be made deterministic. **A clean falsification is a success of the review.**

**Do not accept as success:** scripted-provider-only results; a distribution that is always uniform; agreement measured against the model's own output rather than against recorded settlement.

---

## 8. Production Readiness Assessment (blockers in dependency order)

Each blocker is classified and ordered so that fixing an earlier one does not require redoing a later one.

| # | Class | Blocker | Evidence | Cheapest valid resolution |
|---|---|---|---|---|
| B1 | **evidence blocker** | Gauntlet — the owner of the judgment corpus and of `Execution Environment` — has **no CI** | no `.github/workflows` directory | Add CI invoking the existing `just` recipes; no new tooling |
| B2 | **correctness blocker** | `sea-forge-server` full-suite gate is **red** on an unrelated test | `.agents/CURRENT_STATUS.md` (2026-08-31): `conformance_run_locator::both_layouts_still_resolve_after_a_restart` — second server cannot start while the first holds the cell lock | Fix cell-lock test isolation (lock is per-cell, test assumes per-process) |
| B3 | **architecture blocker** | **No loop correlation identity** | no shared type; correlation by `run_id` string convention across 4 systems | **S4** — one field, four echo sites |
| B4 | **architecture blocker** | **Two disjoint model-invocation stacks with no owner** | `AgentRunner` (process) vs `AgentProvider` (HTTP); `gauntlet-adapter-agent-cli/config.rs:4-15` vs `provider.rs:154-184` | Declare Gauntlet the judgment path; add one HTTP adapter |
| B5 | **safety blocker** | **No adversarial test that model output cannot widen authority** | authority-before-model is true by construction (`agent_probe.rs:221`) with no test asserting it | Write the non-bypass test (§7.6 criterion 4) |
| B6 | **architecture blocker** | **Name collision on *judgment*** | `ManagerJudgment` + `judge()` + `judge_snapshots` are deterministic; the spec's `Judgment` is probabilistic | Explicit naming decision (§3.1) — an ask-first, public-interface change |
| B7 | **evidence blocker** | **Model identity absent from historical judgment records** | live DB sample: `role_invocation.model` is `null`; `verification_record@v1` has no provider/model field | Populate forward; mark historical rows unattributed rather than fabricating attribution |
| B8 | **evidence blocker** | **No calibration artifact anywhere** | no Brier/ECE/reliability code in any repo; 17 settled judgments | Read-only replay harness (§7.5) |
| B9 | **architecture blocker** | **Five ledgers, four capability promotors, three evidence schemas, four context assemblers** | §1.7, §4 | Ownership declaration per purpose; no deletion |
| B10 | **operational blocker** | `gauntlet` has no sealed/immutable runtime artifact story; `sxr` and `godspeed_agent` also have no CI | `.github` absent in all three | CI in precedence order: sxr, godspeed_agent |
| B11 | **operational blocker** | **Observability is nominal.** `tracing-subscriber` present in sea-rs; `GAUNTLET_OTEL_ENDPOINT` env exists but **no OTel dependency was found** in Gauntlet crates; no Prometheus anywhere | `sea-rs/Cargo.toml:56`; `gauntlet/.env.example` | Declare the four metrics that matter (judgment latency, abstention rate, provider disagreement, calibration drift) — and either wire OTel or delete the env var |
| B12 | **deployment blocker** | Judgment evidence has **no retention/export contract** off the Jetson | Gauntlet writes `.gauntlet/` + `targets/.runs/` locally; no export path | Add a read-only export of the consequence chain (also the S5 substrate) |
| B13 | **deployment blocker** | `NetworkPolicy` requires HTTPS for non-loopback endpoints — correct, but it means any off-box judgment provider needs TLS termination | `network.rs:17-21` | Document; `tailscale serve` already provides TLS on the target |
| B14 | **product/API blocker** | **No external judgment API exists**, and none should exist yet | no HTTP surface for judgment in any repo | None. `[ROADMAP]` Revisit only after calibration evidence |
| B15 | **performance blocker** | **UNVERIFIED** — no judgment latency budget exists; inference on a 64 GB unified-memory Jetson with model swapping is the real cost driver | `edgeai` launcher swaps models with memory-aware admission | Measure on target before designing a budget. Do not assume. |

**Not blockers (explicitly):** containerization (the appliance is Compose-based), Postgres/NATS/PGMQ absence (not on the judgment path), secret handling (SOPS/age configured in sea-rs and edgeai; secrets-check recipes exist), and language heterogeneity (every component is internally coherent).

---

## 9. Functional Competitive Surface

Two distinct things must not be conflated.

### 9.1 A. The Jev-like primitive: state + bounded question -> typed probabilistic judgment

**What must exist for GodSpeed to expose this cleanly:**

| Requirement | Status |
|---|---|
| A declared, versioned, hash-pinned question surface | needs S2 |
| A closed answer domain per question | **ABSENT** — the one real gap |
| A provider-neutral invocation boundary | **EXISTS twice** |
| Uncertainty preserved through to storage | **ABSENT** |
| Deterministic composition to a decision | **EXISTS** (`sea-forge-settlement`) |
| Provenance sufficient for replay | mostly EXISTS; needs model identity + surface hash |
| Abstention / indecision as a first-class answer | **ABSENT** |
| Provider substitution proven | partially EXISTS (two ports, per-port contract suites; no cross-port test) |

`[INFERENCE]` **GodSpeed could expose a defensible Jev-comparable primitive in roughly a single focused slice** — the type family in §7.2 plus the adapter in §7.3. `[FACT]` It would be *narrower* than Jev in model breadth (one local provider + one CLI provider + one scripted provider, versus multiple hosted providers) and *broader* in one specific respect Jev does not have: every judgment it produces is already bound to an authority decision, an execution, and a settlement.

### 9.2 B. The GodSpeed system-level capability beyond the primitive

This is where genuine architectural differentiation exists, and it is **already implemented**, not aspirational:

| Differentiation | Why it is hard to copy, and its evidence |
|---|---|
| **Judgment cannot acquire authority** | `Verdict` evaluation precedes model dispatch and is a separate subsystem with its own policy bundle hash (`agent_probe.rs:221`; `Determinism{policy_bundle_hash,...}`). A competitor must build an authority fabric to match it. |
| **Consequence-grounded judgment evaluation, not prompt evaluation** | `revision_evaluations` with held-out **and** regression control/treatment (`promote/`). Most judgment vendors cannot evaluate against consequence at all because they do not own execution or settlement. |
| **Deterministic-before-model by structure, not by convention** | `RungKind::position()` orders instruments; `deterministic()` is a property of the rung; a model cannot wear a deterministic instrument's flag (`ladder.rs:29-31,85-104`). |
| **Expected-vs-observed as a typed artifact** | Gauntlet `comparison` -> `Residual` -> `correction_selection` -> `settlements`. |
| **Capability earned, not declared** | `CapabilityPromotionPolicy` requires repeated settled variation with a computed confidence (`promotion.rs:345-375`). |
| **Integrity at rest** | MMR + global checkpoint + ed25519 witness (`sea-forge-ledger`). |

### 9.3 Unsupported claims that must not be made

- **No claim of learning.** `[FACT]` Nothing in eleven repositories changes weights, policy, or model behaviour as a result of execution. SWE_SEED writes learning artifacts that **nothing reads**. Gauntlet's promotion changes *capability bookkeeping*, not model behaviour.
- **No claim of production readiness.** B1, B2, B5, B9 are open.
- **No claim of provider replaceability.** Two ports exist with per-port contract suites; **no single test proves that swapping providers preserves semantic meaning** (`REQ` in the spec's own `must_report_incomplete...` list).
- **No claim of calibration.** 17 settled judgments.
- **No claim that the stack is a single system.** It is thirteen repositories with one frozen contract document and no runtime correlation identity.

### 9.4 Smallest external capability, and what could ship before the full stack

`[INFERENCE]` The first externally credible artifact is **not** an API. It is a **calibration report** over Gauntlet's historical runs, reproducible by a third party from the pinned run DBs: *given a bounded question with a declared answer domain, and a recorded consequence, how informative is a model judgment versus a deterministic instrument?* That artifact is honest, cheap, already materially available, and it is the prerequisite for any API claim. It also has direct internal value: it tells the team whether the model rung is worth its cost at position 10.

---

## 10. Revised Normative Spec

The revised specification is delivered as a separate artifact so that the original remains intact and diffable:

- **`2026-09-16-godspeed-judgment-plane-spec-revised.yaml`** — proposed revision. Stable requirement IDs are **preserved verbatim**; rejected requirements are marked rather than silently dropped; every material change carries an `evidence` and `justification` annotation.
- **`2026-09-16-godspeed-spec-delta.yaml`** — the annotated delta: per-requirement disposition (`keep` / `amend` / `relocate` / `reject` / `defer`), the evidence that forces the change, the requirement IDs added, and the unresolved questions.

**Summary of material changes (all justified by §3 evidence):**

| Change | Type | Justification |
|---|---|---|
| `JudgmentProvider` demoted from port to provider-neutral *configuration* | **reject as a new port** | two ports already exist |
| `JudgmentObservation` merged into `VerificationRecord` + additive fields | **merge** | the record already binds run+claim+evidence+outcome+instrument |
| `JudgmentComposer` and `CaseAssessment` removed | **reject** | composition and assessment already exist and are owned |
| `Question` deferred to CEP-0005 with one additive field (`AnswerDomain`) | **relocate** | creating a GodSpeed Question would duplicate a hash-pinned protocol noun |
| `DecisionSurface` reclassified as a derived artifact | **amend** | `ApplicationContract` + `Reference` already supply it |
| NEW: `distribution` on the judgment record | **add** | the only genuinely missing semantic distinction |
| NEW: loop correlation identity | **add** | no shared identity exists |
| NEW: versioned composition rule artifact | **add** | `REQ-COMP-002` is violated today |
| `TypeSafe Jev provider` retained as ROADMAP with a tightened contract clause | **amend** | external evidence shows the shape matches; nouns must not leak |
| `Judgment model training` retained as ROADMAP, with an explicit *nothing changes weights today* statement | **amend** | prevents a false learning claim |

---

## 11. Next Decision

**Exactly one recommended next engineering action.**

> **Build a read-only, replayable measurement harness over Gauntlet's 16 existing run databases that answers one question: does a bounded question with a declared answer domain, answered by two materially independent providers, carry information about the recorded settlement that the existing 3-valued deterministic rung does not?**

This is `PROOF-1` in the unresolved list. It is measurement, not judgment implementation. It writes no production code path, changes no schema, adds no service, and touches no persisted data — it reads `targets/.runs/*/state/*.db` and emits a report.

### Why this action

- It is the **only** action that can falsify the Judgment Plane hypothesis before any of S1-S5 is built. Everything else in §1.4 is either cheap-but-contingent (the adapter is only worth building if the surface is stably answerable) or irreversible (naming, schema).
- It uses the **highest-value underused capability** already established in §1.6.
- It reuses Gauntlet's own `promote/` control/treatment evaluation shape rather than inventing a measurement methodology.
- It has **near-zero reversibility cost**: the deliverable is a report and a harness, not a contract.

### What uncertainty it resolves

| Uncertainty | Resolved by |
|---|---|
| Is the historical corpus sufficient to reconstruct state_before -> judgment -> consequence -> settlement, or are the `records` payloads too thin? (`[FACT]` only three `record_type`s appear in practice) | step 1-3 of §7.5 |
| Is the *first decision surface* stably answerable by a local model at all? | step 4 |
| Does a distribution carry information beyond a 3-valued outcome? | step 7 |
| How often do two materially independent providers disagree on the same bounded question? | step 7 |
| Is the uncertainty representation worth its cost, or is the existing `PASS|FAIL|INCONCLUSIVE` already sufficient? | step 7 |

### What evidence it must produce

1. A machine-readable dataset: one row per recoverable historical judgment with `{run_id, iteration_no, unit_id, question, answer_domain, distribution_provider_B, distribution_provider_C, recorded_outcome, recorded_settlement, reference_digest}`.
2. A **coverage report**: how many of the 16 runs and 17 settled judgments were fully reconstructible, and **which fields were missing** (this is the direct measurement of the B7 provenance gap).
3. A **calibration report**: Brier score and reliability by label, abstention rate, provider disagreement rate, and a comparison against the deterministic rung baseline.
4. An explicit **falsification statement** if the distribution carries no information.

### What decisions become possible afterward

| If the harness shows | Then the decision is |
|---|---|
| The corpus reconstructs and the distribution is informative | Build S1 (the HTTP adapter) and S2 (`AnswerDomain`), because the surface is worth it. |
| The corpus reconstructs but the distribution adds nothing over `PASS|FAIL|INCONCLUSIVE` | **Reject the uncertainty requirement.** Much of the Judgment Plane spec collapses, at near-zero cost, and the effort redirects to B1/B2/B5. This is a *good* outcome. |
| The corpus does not reconstruct (payloads too thin / `model: null` everywhere) | The blocker is B7 and B12, not judgment. Fix provenance first; the Judgment Plane is premature until the corpus is attributable. |
| Providers disagree completely | The first decision surface is wrong. Pick a different surface from `residual` rows before building anything. |

### Explicitly not the next action

- Not the HTTP adapter. It is cheap but **contingent** on the harness result, and building it first would create a commitment before the evidence exists.
- Not the naming decision. Important (B6) but it is an ask-first public-interface change and it does not unblock anything above.
- Not CI for Gauntlet. Genuinely needed (B1), genuinely independent, and it can proceed in parallel without waiting for this harness — but it buys no information about judgment.
- Not a `DecisionSurface` compiler, a new event schema, or a new service.

---

## Appendix A. Unresolved questions, assumptions, and evidence gaps

Honest accounting of what this review could **not** settle. These are the places where closure would be manufactured, not earned.

| ID | Open item | Class | Why it is open | How to close |
|---|---|---|---|---|
| U1 | Can a `distribution` on `verification_record@v1` be added without breaking the CEP envelope hash pin (`sxr-core/src/embedded_schema.rs:15`)? | `[ASSUMPTION]` | Gauntlet's record is not itself a CEP envelope; the pin applies to sxr's envelope schema. I did **not** prove that the two can coexist in one run without a CEP amendment. | Read `sxr-core/src/envelope.rs:195` `validate_record_kind_conformance` and trace whether a GodSpeed run emits both. |
| U2 | Does any existing run emit a `records` row of kind `observation`, `comparison`, `residual`, or `promotion_decision`? | `[FACT: not in the sampled DB]` | I sampled one DB of 16; the aggregate counts show only 3 kinds, but I did not enumerate per-DB. | One read-only query across all 16 DBs. |
| U3 | Does `agent_probe`'s `SettlementStatus::Accepted` path constitute operational settlement or a probe-local result? | `[INFERENCE]` | The code settles `Accepted` when the probe succeeds (`agent_probe.rs:352-373`); whether that is intended to be a loop settlement or a probe outcome is a design question the code does not answer. | Read `sea-forge-server/src/agent_probe.rs` in full plus its caller. |
| U4 | Is the `residual` / `correction_selection` corpus large enough to calibrate a second surface? | `[FACT: 28 correction_selection rows]` | 28 is small; the distribution across runs is unmeasured. | Count per-run in the harness. |
| U5 | What is the actual inference latency and model-swap cost on the Jetson for a bounded question? | `[ASSUMPTION]` | The target box is not this machine; `edgeai` benchmarks exist but I did not read them. | Read `edgeai` benchmark artifacts under `artifacts/` and profiles. |
| U6 | Whether `sea-forge-agent` can be depended on from `gauntlet` at all (crate graph, async runtime, forbidden-dependency policy) | `[ASSUMPTION]` | I did not inspect Gauntlet's `deny.toml` / dependency policy or whether an async runtime is already present. This determines whether the HTTP adapter **reuses** or **reimplements** the transport. | Read `gauntlet/deny.toml`, `gauntlet/Cargo.toml`, and one existing async adapter. |
| U7 | The tenancy/authz requirement if judgment is ever exposed off-box | `[ROADMAP]` | No tenancy model exists anywhere in the stack. | Defer until an exposure decision is made. |
| U8 | Whether `SWE_SEED`'s inert `learning/` artifacts are intended to be consumed or are dead code | `[INFERENCE: nothing reads them]` | Writers exist (`learning_cli.rs:42-48,107-110`); no reader was found in `route/`, `context/`, `hooks/`, `skill/render.rs`. | Either wire a consumer or record them as intentionally inert. |
| U9 | Whether `NeatCode` and `project-topology-architect` belong in the stack scope at all | `[INFERENCE: no runtime path]` | Zero integration found in either direction. | Confirm with the operator; the review's recommendation is to leave both independent. |

## Appendix B. Evidence ledger (how each headline claim was established)

| Claim | Method |
|---|---|
| SEA-Forge scale (22 crates, 4,039 tests, 4 workflows, 193 commits) | `Cargo.toml` `[workspace] members`; static `#[test]` count; `ls .github/workflows`; `git log` |
| Gauntlet has no CI; has 5 crates + 24 adapters | `find crates -maxdepth 3 -name Cargo.toml`; `find .github -type f` (empty) |
| Gauntlet persists a correlated chain | SQLite schema dump of all 16 `gauntlet-state.db` files from `sqlite_master` |
| Judgment corpus is thin | SQL `group by record_type` across the 16 DBs; `select count(*) from settlements / observation_bindings` |
| Model identity is absent historically | First `role_invocation` payload in the sampled DB: `model: null` |
| `VerificationRecord` is 3-valued with no distribution | `gauntlet-domain/src/verification.rs:26-33`; live `verification_record@v1` payload |
| Two disjoint provider ports | `gauntlet-ports/src/agent_runner.rs:440-470` vs `sea-forge-agent/src/provider.rs:71-83`; `impl AgentRunner for` search (real impls in `agent-cli`, `agent-prime`, `agent-mock` only) |
| `CliAgentRunner` has no default and no fallback | `gauntlet-adapter-agent-cli/src/config.rs:39-44` |
| CEP Question lacks answer domain / scale / probability | `cep/schemas/cep-semantic-envelope.schema.json:1385`; `cep/spec/CEP-0005-question-model.md:178-207,336-352` |
| No probabilistic type anywhere | cross-repo grep for `confidence|probability|likelihood|posterior|calibrat` over `*.rs`, restricted to non-test code |
| Authority precedes model dispatch | `sea-forge-server/src/agent_probe.rs:221,270,352-373,423-444` |
| `godspeed_agent` is structurally disconnected | grep for imports/HTTP/subprocess into the other repositories across all `*.py` — only name-mapping tables and one opt-in NATS publish |
| Jetson target and inference stack | `edgeai/AGENTS.md:7`; `edgeai/README.md:105,140`; `edgeai/deploy/jetson/docker-compose.yml:195,331,348`; `docker-compose.knowledge.yml:6` |
| Development host has no inference and no DBs | `ss -tlnp`; `which ollama psql nats-server`; `find` for `*.gguf`/`*.safetensors` |
| Five ledgers / four promotors / three evidence schemas / four context assemblers | §1.7, §4 with per-item `path` citations |

## Appendix C. Deliverables

| Artifact | Purpose |
|---|---|
| `2026-09-16-godspeed-judgment-plane-adversarial-review.md` | This report |
| `2026-09-16-godspeed-current-architecture.mmd` | Current component/authority/dependency graph with evidence-classed edges |
| `2026-09-16-godspeed-target-architecture.mmd` | Minimum-coherent target graph, derived from evidence |
| `2026-09-16-godspeed-capability-gap.yaml` | Machine-readable capability gap + production classification + cost/value/uncertainty/reversibility |
| `2026-09-16-godspeed-spec-delta.yaml` | Annotated spec delta + the fifteen-proposition falsification table |
| `2026-09-16-godspeed-judgment-plane-spec-revised.yaml` | Proposed revised normative spec, stable IDs preserved |
