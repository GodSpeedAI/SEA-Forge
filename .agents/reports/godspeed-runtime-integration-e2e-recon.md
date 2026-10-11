# GodSpeed Stack — Runtime Integration & E2E Reconnaissance Report

- **Date:** 2026-08-24
- **Author:** read-only reconnaissance agent (ox-alpha)
- **Workspace:** `/home/sprime01/projects/sea-rs/godspeedai-stack.code-workspace` (13 folders, untracked file in sea-rs worktree)
- **Status basis:** worktree state as found on 2026-08-24 (HEADs in §2). No repository file was modified by this investigation except this report.

**Evidence legend:** FACT = directly demonstrated by code/test inspection. INFERENCE = strongly implied by code. INTENT = documented/propped by docs or types but not demonstrably wired. UNKNOWN = insufficient evidence.

---

## 1. Executive Findings

1. The canonical GodSpeed architecture is the agentic capability loop. Its production implementation currently consists of several mature runtime planes with one critical cross-plane settlement bridge absent. Therefore the complete canonical production runtime has not yet settled end-to-end.
   - **SWE_SEED** (Rust) — work-contract harness: route → trace → proof → gate, plus the *federation* envelope plane (`emit`/`consume`/`sign`/`verify`) and the **MCP stdio client to Context Kernel**.
   - **Context_Kernel** (Rust) — retrieval-only context plane: MCP server exposing the `context_required` tool backed by `AclContextAgent`; emits `ContextPacketCreated` payloads; never decides authority.
   - **godspeed_agent / GSA** (Python) — settlement & developmental-memory authority: `godspeed-nav` CLI + `cybernetic_nav.*` MCP server (21 tools); consumes `EvidenceRecorded` envelopes as *provisional* evidence; only `record_settlement()` promotes; publishes its own 6 event types to NATS.
   - **sea-rs / SEA Forge** (Rust) — governed execution kernel: intent → plan → authority (with DomainForge semantic validation) → sandboxed execution → trace/evidence → settlement → semantic envelope; Unix-socket server adds approvals, delegations (ACP to external coding agents), transcript sealing, and SWE_SEED declaration reconciliation.
   - Optional transport/persistence: **NATS JetStream + hassos-addon-agent-memory-ledger** (Postgres canonical event store via `sea_nats_bridge.py`).

2. **The single strongest implemented cross-component edge today is SWE_SEED → Context Kernel over MCP stdio** (`context_required` → cited `ContextPacketCreated`). It is config-gated, offline-first, and covered by tests on both sides.

3. **Highest-risk architectural finding:** **no component produces `EvidenceRecorded` envelopes in production code.** GSA's settlement-ingestion boundary (`handle_evidence_recorded` → `ingest_execution_evidence`) is implemented, hardened, and tested — but only against synthetic events. The "SEA-Forge emits EvidenceRecorded" step exists in documentation and in the legacy Python reference loop, not in any shipping binary. The full 14-event cross-repo sequence is proven only by the **generation-1 Python test** (`SWE_SEED/tests/test_agentic_capability_hardening.py`) which dynamically loads Python modules from four repos — not through the production binaries.

4. **Two coexisting generations:** the gen-1 Python reference loop (`SEA/libs/agentic_capability_loop/*`, `Context_Kernel/integration/agentic_capability_loop/adapters.py`) and the gen-2 production stack (Rust/Rust/Rust/Python). They disagree on details already (e.g., gen-1 envelope puts `namespace` top-level; the v1 schema forbids it). Tests exist for both; nothing pins them to each other.

5. **Semantic identity anchor is weak:** every envelope carries `domain_model_hash`, but when the SEA manifest is absent the adapters silently fall back to `sha256("agentic_capability_loop")` (a constant), issuing only a Python warning / `HashSource::Fallback`. Identity can degrade silently across the whole stack.

6. **RealityTrace (sxr)** is a parallel, self-contained evidence subsystem (its own hash-chained ledger, its own CEP-record export verified by CEP's anti-collapse evaluator). It consumes DomainForge semantics through a **process boundary** (the `domainforge` CLI `envelope --emit cep`). No GodSpeed-loop component imports or invokes sxr. It is canonical for its own scope, peripheral to the capability loop.

7. **Gauntlet** currently implements only `gauntlet doctor` and `gauntlet reference digest` (full runner surface is spec'd but unlanded). Nothing references it at runtime → development/tooling, not canonical runtime. Consequently `project-topology-architect` (future Gauntlet skill) and `NeatCode` (future SWE_SEED skill) are outside the E2E boundary, matching the mission exclusions.

---

## 2. Investigation Scope and Provenance

### Repositories inspected (HEAD at investigation time)

| Repo | HEAD | Branch | Pre-existing dirty state |
| --- | --- | --- | --- |
| sea-rs | `006daa2` | ultracode/sea-forge-completion | 7 entries (AGENTS.md M; template deletions; untracked `godspeedai-stack.code-workspace`, plan/spec templates, current_status.yml) |
| cep | `adbf35f` | slice-0a-cep-semantic-envelope | 2 (`.serena/project.yml` M; untracked product-marketing.md) |
| Context_Kernel | `aa17b6a` | harden | 43 (heavy .agents churn, staged) |
| domainforge | `5eb6848` | chore/release-please-commit-fix | clean |
| domainforge-lsp | `db79562` | semantic-adapter | 9 (untracked `semantic_*.rs`, modified backend/lib) |
| edgeai | `607fe5f` | dev | clean |
| gauntlet | `dbe529a` | main | 53 (adapters M, Cargo.{toml,lock} M, jolli churn) |
| godspeed_agent | `eeee146` | audit-corrections/neatcode-2026-07-29 | 3 |
| hassos-addon-agent-memory-ledger | `edf9116` | main | 9 |
| NeatCode | `7d4a516` | main | clean |
| project-topology-architect | `42a332b` | main | 5 |
| SWE_SEED | `16dcce4` | deploy-prep | 16 |
| sxr | `560eaf9` | main | 8 |

Not inspected as candidates (outside the declared workspace): `SEA` was inspected **only** as the holder of the authoritative `.sea` domain model + gen-1 Python libs (see §3); `godspeed`, `godspeed-growth` untouched (not in workspace; presumed unrelated — UNKNOWN).

### Method

Static inspection only. No builds were run; no tests were executed. Primary techniques: manifest/workspace inventory, symbol-level grep across repos, targeted file reads, existing-test inventory, and cross-checking a pre-existing prior report (`SWE_SEED/.agent-harness/reports/godspeed_stack_agentic_loop_wiring_report.md`) against current code (several of its claims are now stale — noted inline where relevant).

### Limitations

- Runtime behavior was not exercised (read-only mandate); all dynamic claims rest on code + existing tests.
- `sea-forge-server`'s full request verb set, GSA's full metric math, and sxr's complete command surface were sampled, not exhaustively read.
- The `sfwp` delegation-preview/thoth views and `sea-forge-cell`/`artifact-ip` internals were not deep-dived (P3 for this report's purpose).

---

## 3. Component Classification

| # | Component | Role | Classification | Key evidence | In canonical E2E? |
| --- | --- | --- | --- | --- | --- |
| 1 | **sea-rs (SEA Forge)** | Governed capability-execution kernel (CLI + Unix-socket server) | **Canonical runtime component** | 22-crate workspace `Cargo.toml`; pipeline described in ARCHITECTURE.md §4.1 and implemented across `sea-forge-{core,planner,authority,sandbox,runtime,trace,evidence,settlement,capability}`; entry `crates/sea-forge-cli/src/main.rs` (`Run`, `Delegate`, `InternalTestSweSeed`, …); server `crates/sea-forge-server/src/lib.rs:5` ("listens on a Unix domain socket, accepts NDJSON requests") | **Yes** (authority/execution plane) |
| 2 | **SWE_SEED** | Work-contract harness + federation envelope plane | **Canonical runtime component** | `crates/swe-seed/src/cli.rs:38-145` (`Route`,`Trace`,`Gate`,`Federation`,`Gateway`,…); `swe-seed-core/src/federation/*`; trace ledger `swe-seed-core/src/trace_ledger.rs` | **Yes** (intake/orchestration/proof plane) |
| 3 | **Context_Kernel** | Retrieval-only context MCP service | **Canonical runtime component** (in-loop role) + general context service | `crates/ck-mcp/src/agentic_capability_loop.rs` (module header: "Consumed: ContextRequired — emitted by SWE_SEED; Emitted: ContextPacketCreated"); tool dispatch `ck-mcp/src/lib.rs:132` (`"context_required"`); wiring `ck-bin/src/main.rs:423-435` (`CK_CONTEXT_CORPUS_ROOT`) | **Yes** (context plane) |
| 4 | **godspeed_agent (GSA)** | Settlement / capability / developmental-memory authority | **Canonical runtime component** | `pyproject.toml` script `godspeed-nav=godspeed_nav.cli:main`; `godspeed_nav/runtime.py:348` `record_settlement`, `:440` `ingest_execution_evidence`; `agentic_capability_loop/adapters.py` (emits 6, consumes `EvidenceRecorded`); `mcp/cybernetic_nav_server/server.py` `TOOL_NAMES` (21 tools); HARNESS.md ownership boundary (settlement/capability/MCP owned by GSA; hooks/traces owned by SWE_SEED, F-08) | **Yes** (settlement plane) |
| 5 | **domainforge** | SEA DSL parser/semantic engine; publishes crates + CLI + python module | **Required integration dependency** (+ toolchain projection source) | `sea-forge-domainforge/Cargo.toml:12` depends on crates.io `domainforge-core = "=0.16.0"` (published from this repo); CLI `domainforge-core/src/cli/mod.rs:62` `Envelope` subcommand consumed by sxr-df; `compute_pack_content_hash` in `domainforge-core/src/semantic_pack/canonical_json.rs` referenced by addon docs | Library: yes (via sea-rs). CLI: only for sxr path |
| 6 | **hassos-addon-agent-memory-ledger** | Home-Assistant add-on: Postgres canonical event store + NATS JetStream bridge | **Optional runtime adapter / persistence infrastructure** | `rootfs/usr/bin/sea_nats_bridge.py` (stream `SEA_LEDGER`, durable `agent_memory_ledger_bridge`, `CANONICAL_ROUTES` at lines 158-192); exercised by GSA e2e `tests/e2e/test_settlement_roundtrip.py` (skipped without docker) | Optional slice (transport persistence) |
| 7 | **cep** | Canonical Evaluation Protocol: owns Semantic Envelope schema + anti-collapse semantics; conformance evaluator CLI | **Specification/standards + conformance tooling** (not a loop process) | `schemas/semantic-envelope.schema.json`, `src/cep/conformance/corpus.py:31` `CORPUS_SCHEMA_VERSION="cep/anti-collapse-corpus@1"`; README: "The envelope is not the truth"; evaluates externally supplied corpora incl. sxr exports | Conformance gate only (L0/L5-style) |
| 8 | **sxr (RealityTrace)** | Experiment evidence subsystem: expected→observed→difference, hash-chained ledger, CEP record producer | **Canonical within its own scope; parallel/peripheral to the capability loop** | Own workspace `sxr-{core,cli,df,git,ledger,verify}`; `AGENTS.md` (Evidence and Settlement; Historical Evidence Immutable; Experiment Identity); **zero references from sea-rs/SWE_SEED/GSA/CK code** (grep) | Separate track; propose dedicated slice, not part of capability-loop E2E |
| 9 | **gauntlet** | Evaluation harness (ports-and-adapters runner) | **Development/tooling** today | `GAUNTLET.md` runtime policy contract; `crates/gauntlet-cli/src/main.rs` implements only `doctor` + `reference digest` (full §20.1 surface pending "Task 20"); grep finds **no** references from any other stack repo | **No** |
| 10 | **project-topology-architect** | Intended Gauntlet skill | **Skill/plugin (pre-wiring)** | Mission statement; no runtime references from gauntlet code (checked via grep of gauntlet crates — none) | **No** (excluded per mission) |
| 11 | **NeatCode** | Intended SWE_SEED skill | **Skill/plugin (pre-wiring)** | Mission statement; SWE_SEED has a generic `Skill` CLI + skills dir but nothing importing NeatCode at runtime | **No** (excluded per mission) |
| 12 | **edgeai** | Edge appliance platform (queues, HA/Discord/email adapters, models) | **Unrelated to the canonical loop** (infrastructure pattern donor only) | `publisher.py:7` cites "mirroring edgeai's spool pattern" — pattern reference, not dependency; no import/build references found from stack repos | **No** |
| 13 | **domainforge-lsp** | Language server for the SEA DSL | **Development/tooling** | `Cargo.toml` description "Language Server Protocol implementation"; dirty branch adds semantic pack loaders for editor UX; no runtime consumer in the loop | **No** |
| 14 | **SEA (old Python repo, outside workspace)** | Holds the **authoritative `.sea` domain model + manifest** for the loop; hosts gen-1 Python reference libs | **Persistence-of-domain-model + legacy reference implementation** | `SEA/docs/specs/domains/agentic_capability_loop/agentic_capability_loop.{sea,manifest.json}` (manifest `meta.sea_file_hash=62f4f0cd…`); `SEA/libs/agentic_capability_loop/{authority_service,policies,signing}.py` loaded by SWE_SEED Python cross-repo tests; marker walk in `SWE_SEED` looks for nested `SEA/` with `tools/sea_parse.py`\|`docs/specs` | Domain-model source: **yes** (identity anchor). Gen-1 libs: regression-reference only |

Explicit exclusion accounting: `project-topology-architect` and `NeatCode` are excluded per mission and confirmed unwired (§3 rows 10–11). Gauntlet additionally confirmed non-canonical by implementation status (row 9). `sxr` is in the workspace and is a real subsystem, but no capability-loop component calls it — it forms its own loop with cep and domainforge-CLI (see §4.3).

---

## 4. Canonical Runtime Loop

### 4.1 Primary loop — the agentic capability loop (as implemented today)

Actual stage names follow the code. Entry point for a piece of work is the **SWE_SEED CLI inside a target repository**; the settlement terminal state lives in **GSA ledgers**; the authority/execution terminal state lives in **SEA Forge case/run records**.

```text
[Work request in a repo]
    |  swe-seed route "<task>" [--record]           (crates/swe-seed/src/cli.rs Route)
    v
[SWE_SEED route selection] -- RouteSelected envelope (federation on)      \
    |                                                                       | envelopes:
    |  ContextConfig mode=external + SWE_SEED_CONTEXT_KERNEL_BIN set        | sea.agent.event.v1.json
    v                                                                       | namespace INSIDE payload;
[SWE_SEED spawns Context Kernel binary (MCP stdio)]                         | idempotency_key =
    |  JSON-RPC tools/call "context_required"                                |   sha256(cid|etype|payload)
    |  {work_request_id, context_requirement_id, corpus_id, query,          | provenance {origin, chain}
    |   max_results, authority_decision_id?, authority_reference?}          /
    v
[Context Kernel ck serve  (ck-bin/src/main.rs Serve; corpus root from CK_CONTEXT_CORPUS_ROOT)]
    |  ck-mcp dispatch_tool("context_required") -> AclContextAgent.handle_context_required()
    |     - corpus_id starts_with("learning-proposals") -> read GSA learning-proposals JSONL
    |     - else FilesystemSourceAdapter over <corpus_root>/<corpus_id> (traversal-contained)
    v
[ContextPacketCreated payload]  {domain_model_hash, context_packet_id, context_requirement_id,
    work_request_id, corpus_id, citations[{citation_id, source, content, fetched_at}],
    authority_decision_id?, authority_reference?}         <-- CK passes authority REFS through;
                                                              never decides authority
    |
    |  (back in SWE_SEED) POL-ACL-003 expectation: >=1 citation
    v
[Proof plane]  RouteSelected -> ProofStarted -> run deterministic proof command
    |            -> ProofCompleted {proof_result_id, result, exit_code, output_ref,
    |                               proof_type: live|simulation}
    |  trace events appended to SQLite chain_entries:
    |     hash = SHA256(event_type || payload_json || prev_hash)   (trace_ledger.rs:36)
    v
[trace_chain_root]  --federation sign (Ed25519 over canonical string)-->  signed ProofCompleted envelope
    |
    |  (settlement plane; TODAY: no automated producer — see Gap G1)
    v
[GSA agentic_capability_loop.handle_evidence_recorded(event)]
    |  requires event_type=EvidenceRecorded; payload fields: event_id, affordance_id,
    |  work_request_id, proof_result_id, expected_result, observed_result (non-empty)
    |  affordance must exist with current_status == 'affordance'
    v
[NavigationRuntime.ingest_execution_evidence]  -> PROVISIONAL record in execution_evidence.jsonl
    |  match/mismatch preserved as observed fact; NEVER writes settlements/capabilities;
    |  idempotent on event_id                                        (runtime.py:440-493)
    v
[godspeed-nav record-settlement]  (human/agent promotion step; CLI or cybernetic_nav MCP tool)
    |  _require_executable_affordance -> score_settlement(before,after,...)
    v
[Settlement recorded]  settlements.jsonl + trajectories.jsonl + capabilities update
    |                + repetition_schedules; emits SettlementRecorded/CapabilityUpdated/
    |                 RepetitionPlanned/TwinUpdated/CoherenceBreakDetected/LearningProposalCreated
    v
[SettlementPublisher.drain]  (only if GSA_NATS_URL set; offline-first JSONL buffer stays truth)
    |  NATS subject sea.agent.event.godspeed-agent.<event_type slugged: lower, _->->
    |  header Nats-Msg-Id = idempotency_key
    v
[hassos-addon sea_nats_bridge.py]  JetStream stream SEA_LEDGER, durable agent_memory_ledger_bridge
    |  validate_envelope -> validate_payload(family) -> route by CANONICAL_ROUTES:
    |     sea.agent.event      -> event_log.agent_events   (fail_open=True!)
    |     sea.governance.*     -> governance.action_requests/action_decisions (fail-closed)
    |     sea.memory.*         -> memory.items / event_log.inbox_events     (fail-closed)
    |  invalid + fail-closed -> Nak -> DLQ sea.ledger.deadletter ; duplicates ACKed idempotently
    v
[Postgres agent_memory DB]  terminal durable observation store for the settlement plane

=== parallel authority/execution plane (SEA Forge) =============================

[Intent] sea-forge-cli Run {intent, plan, policy, root, timeout, entity, process}
    |  parse + preflight -> case/run dirs under <root>/.sea-forge/
    v
[Deterministic planner] intent -> typed CasePlan                    (sea-forge-core planner)
    v
[Authority fabric]  PolicyAuthorityEngine; semantic candidate validated through
    |  sea-forge-domainforge::evaluate_authority (domainforge-core =0.16.0)
    |  verdict Allow/Deny/Escalate/Boundary/Degraded; ActionGrant = move-only proof token
    |-- deny/escalate --> governed HALT (records trace/evidence, no execution)
    v
[Sandbox + runtime] materialize workspace, execute argv, capture stdout/stderr/hashes
    v
[trace.jsonl + evidence.jsonl + settlement.json + semantic-envelope.json]
    |  settlement evaluated independently of exit code; envelope appended to capabilities.jsonl
    v
[Server-era additions] sea-forge-server (Unix socket, NDJSON, additive SFWP verbs v1):
    - delegations: DelegationRequest -> AcpSpawn{argv,cwd,env} (external coding agent over stdio)
      with Anthropic/OpenAI-compatible providers + permission mediation -> sealed transcript
    - SWE_SEED declaration ingress + reconciliation:
        swe_seed_claim_manifest_sha256(case_id, run_id, plan_item_id, settlement_id,
                                       transcript_sha256, harvested_refs)   (idempotent join)
```

Terminal observable results of the primary loop:
- **Intake/proof plane:** passing `swe-seed gate <trace-id> --verify` over a hash-chained SQLite trace ledger + route/trace records under `.agent-harness/traces/`.
- **Context plane:** `ContextPacketCreated` payload returned over MCP (and, federation-on, the emitted envelope file/stdout).
- **Settlement plane:** rows in GSA `.godspeed-harness/ledgers/*.jsonl` (optionally replicated to Postgres `event_log.agent_events` via NATS).
- **Authority plane:** `.sea-forge/cases/<case>.json`, `runs/<run>/…`, `capabilities.jsonl`.

Async/feedback behavior: NATS drain is asynchronous and offline-tolerant (buffered JSONL is truth); bridge redelivers via JetStream ack-wait/max-deliver; SEA Forge server adds approval timers and subscriptions (Tokio confined to server edge); GSA repetition scheduling feeds future affordance ranking (feedback into `rank-affordances`).

### 4.2 Secondary loop — SEA Forge governed run (standalone-capable)

`sea-forge-cli run` works with no other component (ARCHITECTURE.md §4.1 "minimum run"). This is the deterministic golden path for the authority plane and needs only a policy file + a fixture intent. DomainForge enters here as an **in-process library** (semantic candidate evaluation inside authority), which is the only DomainForge consumption inside production code today.

### 4.3 Tertiary loop — RealityTrace (sxr) with CEP verification

```text
[Repo enrolled in sxr] -> sxr plan/run/question -> observations vs expected state
    |  semantics resolved via sxr-df CliDomainForgeAdapter:
    |    subprocess: domainforge CLI `envelope --emit cep`
    |    classify BY ENVELOPE SHAPE, never exit code:
    |      validation_status=validated + representations -> SemanticContext::Resolved
    |        (content_hash + semantic_closure_hash identify D)
    |      conformance_status=non_conformant, no representations -> ValidationFailed
    |      anything else -> error (malformed never misclassified)
    v
[sxr ledger: hash-chained envelopes; sxr-ledger/sxr-verify]
    v
[exported corpus: cep/anti-collapse-corpus@1]
    v
[cep conformance anti-collapse / evaluate_corpus] -> PASS/FAIL via detect_reality_gap_collapses
    ("sxr exported corpus -> independent CEP evaluator", cep/src/cep/conformance/corpus.py)
```

No component of the capability loop consumes sxr output today (**FACT**, absence verified by grep). Integration is *through CEP semantics*, not through code.

### 4.4 What is NOT wired despite documentation suggesting it

| Claim source | Claim | Reality |
| --- | --- | --- |
| GSA `adapters.py:13` | "consumes … ProofCompleted (from SWE_SEED, via SEA-Forge evidence chain)" | **No `handle_proof_completed` exists.** Only `handle_evidence_recorded` (adapters.py:125). INTENT, not implemented. |
| Prior wiring report §2 | "SEA-Forge … emits AuthorityChecked, EvidenceRecorded, CoherenceBreakDetected" | True only of the gen-1 Python libs in `SEA/libs/agentic_capability_loop/`. sea-rs contains zero occurrences of `EvidenceRecorded`. |
| Prior wiring report Link 3 | "CK Rust `ContextAgent` trait has no implementation" | **Stale/fixed**: `AclContextAgent` is implemented and served through the real binary (`ck-mcp/src/lib.rs:440-477`, `ck-bin/src/main.rs:423-435`). |
| HARNESS.md F-08 | "GodSpeed-Agent registers no host hooks and runs no live trace recorder" | Consistent: `godspeed_nav.trace_capture` recorder removed (docs/live-harness-trace-capture.md header: "Superseded by F-08"). |

---

## 5. Critical Integration Contract Inventory

Criticality: P0 = system truth/integrity; P1 = breaks primary loop; P2 = supported capability; P3 = peripheral.

| ID | Upstream → Downstream | Producer symbol → Consumer symbol | Contract (the thing that crosses) | Source evidence | Dir / Sync | Criticality |
| --- | --- | --- | --- | --- | --- | --- |
| INT-001 | SWE_SEED → Context Kernel | `ContextKernelClient::spawn/context_required/call_tool` (`swe-seed-core/src/federation/context_client.rs:43,77,110`) → `McpService::dispatch_tool("context_required")` (`ck-mcp/src/lib.rs:132,440-477`) | JSON-RPC `tools/call` over stdio; args `{work_request_id, context_requirement_id, corpus_id, query, max_results, authority_decision_id?, authority_reference?}`; response `result.context_packet` = ContextPacketCreated payload | context_client.rs:1-160; ck-mcp lib.rs:119-145, 435-477 | async process pair, strict req/resp | **P0** |
| INT-002 | CK → corpus storage | `AclContextAgent::resolve_citations` (`ck-mcp/src/agentic_capability_loop.rs:111-165`) → `ck_core::ports::SourceAdapter` / `ck_adapter_filesystem::FilesystemSourceAdapter` | Filesystem units under `<corpus_root>/<corpus_id>`; traversal/symlink/absolute-path containment enforced (canonicalize + starts_with); `learning-proposals*` corpora read GSA JSONL ledger instead | acl module 76-165; ck-core/src/ports/source_adapter.rs | sync in-process | **P1** |
| INT-003 | CK wiring | env `CK_CONTEXT_CORPUS_ROOT` (`ck-bin/src/main.rs:423-435`) → `AclContextAgent.corpus_root` (missing ⇒ `unwrap_or_default()` = empty path ⇒ **empty citations success**) | Environment configuration contract | ck-bin main.rs 423-435 | startup | **P0** (failure semantics) |
| INT-004 | SWE_SEED → (file/stdout sink) → world | `emit_*` builders + `dispatch()` (`swe-seed-core/src/federation/emit.rs:17-158`) | Envelope v1 JSON (`schema_version,event_id,source_agent,event_type,occurred_at,idempotency_key?,payload,provenance?`); `namespace` rides inside payload (schema `additionalProperties:false`); gated by `FederationConfig{enabled, emit_envelope,…}`; Suppressed ⇒ zero filesystem touch | emit.rs 1-158; envelope.rs:21-34,41-56,199-218 | sync file write | **P0** |
| INT-005 | SEA repo manifest → all emitters | `resolve_from_root/resolve_from` (envelope.rs:86-156); Python `_load_hash` (GSA adapters.py:15-24) | `domain_model_hash` resolution order: `SEA_ROOT`→`<root>/docs/specs/domains/agentic_capability_loop/agentic_capability_loop.manifest.json` `meta.sea_file_hash` → `SEA_MANIFEST_PATH` → parent marker walk (`SEA/tools/sea_parse.py` ∨ `SEA/docs/specs`) → **fallback sha256("agentic_capability_loop") + warning** | envelope.rs:86-190; GSA adapters.py 15-24; manifest FACT: `meta.sea_file_hash` field present | sync read | **P0** (identity anchor; weak fallback) |
| INT-006 | SWE_SEED ↔ SEA(-Forge) verifier | `signing.rs` canonical string + Ed25519 (signing.rs:1-40) ⇄ Python `cryptography` verifier (pinned test vector; `swe-seed-core/tests/federation_verify_sea.rs`) | Signed bytes = `{namespace}\n{event_type}\n{occurred_at}\n{compact_sorted_payload_json}`; `signature` and `event_id` excluded; keys: committed `.agent-harness/federation/keys/<id>.pub`, private gitignored `.swe-seed/federation/keys/<id>.key` | signing.rs 1-80; federation_cli.rs Status/Keygen/Sign/Verify | offline files | **P0** (provenance) |
| INT-007 | SWE_SEED trace events → gate | `trace_ledger.rs` SQLite `chain_entries` (DDL :59, insert :115, verify :152-186); `Envelope::with_trace_chain_root` (envelope.rs:258) | Hash chain: `hash = SHA256(event_type‖payload_json‖prev_hash)`, genesis prev = `GENESIS_PREV`; `swe-seed gate <id> --verify` fails closed on broken links/unrouted traces; `gate-merge` uses it as merge gate | trace_ledger.rs 1-190; README gate section | sync sqlite | **P0** |
| INT-008 | World → GSA settlement intake | `handle_evidence_recorded` (adapters.py:125-146) → `NavigationRuntime.ingest_execution_evidence` (runtime.py:440-493) | Canonical event envelope; required payload fields `event_id,affordance_id,work_request_id,proof_result_id,expected_result,observed_result` (non-empty) else `DomainRuleError("incomplete_execution_evidence")`; `_require_executable_affordance`; writes ONLY `execution_evidence` ledger; idempotent re-delivery on `event_id`; match/mismatch kept as observed fact; status always `provisional` | adapters.py 108-146; runtime.py 440-493; HARNESS.md policy bullets | sync function | **P0** |
| INT-009 | GSA operator/agent → promotion | `NavigationRuntime.record_settlement` (runtime.py:348-439) + CLI `record-settlement` (cli.py:154) + MCP tool `cybernetic_nav.record_settlement` | Requires executable affordance (else `settlement_requires_executable_affordance`); computes score/status; appends `settlements`, `trajectories`, updates capability evidence, schedules repetitions; **only promotion path** from provisional | runtime.py 348-493 | sync | **P0** |
| INT-010 | GSA emitter parity | `_event/_idempotency_key/_provenance` (adapters.py:26-105) | Same idempotency derivation as SEA v1 (F-10): `sha256("{cid}|{etype}|{canonical_payload}")`; replay keeps key, freshens event_id; provenance `{origin:"godspeed-agent", chain:[domain_model_hash:, correlation_id:]}`; 6 emitted event types | adapters.py 26-105; test_v1_contract.py:206-228 | sync | **P1** |
| INT-011 | GSA → NATS | `SettlementPublisher` (publisher.py:23-100) | Subject `sea.agent.event.godspeed-agent.<slug>` (slug = lowercase, `_`→`-`); `Nats-Msg-Id` header = idempotency_key; disabled unless `GSA_NATS_URL` (+`nats` pkg); failure never breaks inner loop; JSONL outbox remains authority | publisher.py 1-100 | async fire-drain | **P1** |
| INT-012 | NATS → Postgres bridge | `BridgeConfig`/`CANONICAL_ROUTES`/`validate_envelope/validate_payload/validate_contract` (`sea_nats_bridge.py:64-101,158-192,341-520`) | Stream `SEA_LEDGER`; durable `agent_memory_ledger_bridge`; subjects `sea.agent.event.>`, `sea.governance.request.>`, `sea.governance.decision.>`, `sea.memory.write.>`, `sea.memory.lifecycle.>`; per-family required fields (e.g., governance.request needs `requesting_identity_id,requested_action_type`); fail-closed families → DLQ `sea.ledger.deadletter`; **`sea.agent.event` is fail-open**; duplicate `Nats-Msg-Id` ACKed without reprocessing; message-id precedence incl. sha256(subject‖payload) fallback | sea_nats_bridge.py 64-101,158-192,200-235,341-520 | async pull consumer | **P1** |
| INT-013 | Bridge → Postgres | SQL migrations `001_event_log.sql` (agent_events), `004_inbox_outbox.sql`, `006_governance_events.sql`, `013_security_roles.sql` (under `rootfs/usr/share/agent_memory_ledger/agent_memory/`) | Tables `event_log.agent_events`, `governance.action_requests/action_decisions`, `memory.items`, inbox/outbox; role `bridge_worker` | migration files | transaction per message | **P1** |
| INT-014 | domainforge-core → sea-rs authority | `sea_forge_domainforge::{evaluate_authority, CandidateDisposition, DomainModel, DomainModelRef}` used by `sea-forge-authority/src/lib.rs:1-8` | Crates.io library API pinned `=0.16.0`; semantic candidate disposition feeds authority decisions; `ActionGrant` move-only token cannot be serialized/cloned | authority lib.rs 1-70; adapter Cargo.toml:12 | sync in-process | **P1** |
| INT-015 | clients → sea-forge-server | `Request` enum (verb-tagged) + SFWP additive verbs (`sfwp/mod.rs:5-33`, `SFWP_PROTOCOL_VERSION="1"`) | NDJSON over Unix domain socket; unknown verb ⇒ clean serde rejection; major version `"1"` only; views: approvals, delegations, events, correlation, readiness, thoth, case/run views; record-size ceiling checks | server lib.rs:5,46,263-267,855-863; sfwp/mod.rs | async server / sync kernel | **P1** |
| INT-016 | server → external agents | `DelegationRequest` (delegation.rs:30-60) → `sea_forge_agent::{run_delegation, AcpSession, AcpSpawn, AnthropicProvider, OpenAiCompatibleProvider, AcpPermissionMediator}` (acp.rs:313-360) | argv-spawned ACP agent over stdio with cwd/env; permission mapping (`acp_permission_allowed/denied*`); max_response/transcript bytes; transcript sealed (`transcript_seal.rs`); actor_role verified per connection (F-08 note in delegation.rs) | delegation.rs 1-60; acp.rs 313-360, 958-1030 | multi-turn async | **P1** (needs fake agent for tests) |
| INT-017 | SWE_SEED declarations → SEA Forge ledger | `swe_seed_claim_manifest_sha256(case_id,run_id,plan_item_id,settlement_id,transcript_sha256,harvested_refs)` (`swe_seed_reconciliation.rs:27-52`); `append_declaration_ledgered_once` + pure reconciler | Canonical-hash claim manifest binds declaration to exact run/settlement; forged/relabeled/replayed declarations never correlate; idempotent, callable from server or bare CLI (`InternalTestSweSeed` stdin hook, cli/main.rs:60-67,492-504) | swe_seed_reconciliation.rs 1-90; main.rs 492-504 | sync ledger append | **P1** |
| INT-018 | GSA ledgers | `LedgerStore.append/read/quarantine` (`storage.py:28-96`) | Append-only JSONL per ledger name (alnum-validated); fcntl-locked, fsync'd; secret redaction; corrupt lines quarantined under `cache/quarantine`; storage manifest with `STORAGE_SCHEMA_VERSION=1` | storage.py 12-96 | sync file | **P1** |
| INT-019 | domainforge CLI → sxr | `CliDomainForgeAdapter` (`sxr-df/src/adapter.rs:1-60`) | Subprocess `envelope --emit cep`; classification strictly by envelope shape (`validation_status`/`representations` vs `conformance_status`); `content_hash` + `semantic_closure_hash` required exactly when Resolved | sxr-df adapter.rs 1-60 | subprocess | **P2** |
| INT-020 | sxr → cep | `evaluate_corpus` (`cep/src/cep/conformance/corpus.py:31-80`) | Corpus JSON `cep/anti-collapse-corpus@1`: `{corpus_id, records:[{record_id, record_ref?, accepted_gap_statuses[], valid_evidence_ids[], submitted_answer}]}`; verdict from `detect_reality_gap_collapses` (single source of truth) | corpus.py 1-80 | CLI batch | **P2** |
| INT-021 | SWE_SEED python cross-repo test rig | `tests/workspace_roots.py` resolver (`GODSPEED_WORKSPACE_ROOT`, `SEA_ROOT`, `SWE_SEED_ROOT`, `GODSPEED_AGENT_ROOT`, `CONTEXT_KERNEL_ROOT` + folder-name fallbacks) → `test_agentic_capability_hardening.py` loads modules from 4 repos | Test-infrastructure contract: env-var-driven root discovery; dynamic module loading of gen-1 Python loop | workspace_roots.py 1-30; hardening test head 1-40 | pytest | **P2** |
| INT-022 | GSA → CK (learning proposals) | `AclContextAgent.learning_proposals_ledger` reading GSA-written JSONL for `learning-proposals*` corpora (acl module 76-118) | One-directional file handoff (GSA writes ledger; CK reads) | acl module 76-118 | file | **P2** |
| INT-023 | Addon semantic packs ↔ clients | `expectedHash` pin computed by `sea_core::semantic_pack::compute_pack_content_hash` (addon README; fn in `domainforge-core/src/semantic_pack/canonical_json.rs`) | Content-addressed semantic packs; load-time tamper detection. Where the loader itself runs (client-side via domainforge python module) — **UNKNOWN; no loader code found in addon repo** (grep) | addon README Tamper-Protection; domainforge canonical_json.rs:98 | HTTP/file | **P3** |

Contract kinds worth distinguishing per row (structural/semantic/temporal/authority/identity/provenance/failure/persistence) are called out inline in §6–§9 and in the test catalog.

---

## 6. Semantic Transformation Map

| # | Representation A | transformation | Representation B | Must survive |
| --- | --- | --- | --- | --- |
| T1 | `.sea` DSL text (`SEA/docs/specs/domains/agentic_capability_loop/agentic_capability_loop.sea`) | domainforge parse/project | `Ast`/`Ir`/`.manifest.json` with `meta.sea_file_hash=62f4f0cd…` | identity of the domain model; determinism (hash covers `.sea` bytes) |
| T2 | manifest `meta.sea_file_hash` | resolve_from_root / `_load_hash` | `domain_model_hash` string in **every** envelope payload + provenance chain | **identity** — currently degradable to constant fallback (G-2); provenance chain records origin only when set |
| T3 | free-text work request | swe-seed route cards | Route card JSON `{job_type, route_card, required_context, required_skills, work_loop, required_artifacts, proof, done_when, next_action}` | intent → obligations mapping stability (golden-tested: `route_golden.rs`) |
| T4 | `ContextRequired` payload | CK FilesystemSourceAdapter chunk/discover | `citations[]{citation_id, source, content, fetched_at}` | source referenceability; containment; **≥1 citation expectation (POL-ACL-003) NOT enforced by CK** (G-3) |
| T5 | structured payload dict | canonical signing string (`{ns}\n{etype}\n{occurred_at}\n{sorted_compact_payload}`) | Ed25519 signature bytes | byte-exact cross-language parity (Rust dalek ⇄ Python cryptography); `event_id` intentionally excluded |
| T6 | trace event stream | `SHA256(etype‖payload_json‖prev_hash)` chain | `trace_chain_root` | tamper-evidence of execution history; genesis anchoring |
| T7 | `EvidenceRecorded` payload | GSA ingest | `execution_evidence` record `ee-*` with `status=provisional`, match/mismatch fact | semantic rule "software proof ≠ settlement"; idempotency by event_id |
| T8 | before/after states + payment | score_settlement | `settlements` record `{score,status,evidence,…}` + trajectory θ/radius | outcome independence from exit codes; evidence list carried |
| T9 | GSA event_type (e.g. `SettlementRecorded`) | `_slug` (lower, `_`→`-`) | NATS subject suffix (`settlement-recorded`) | reversible mapping is **lossy-by-design**; bridge does not invert it — event consumers must not rely on original casing/underscores |
| T10 | NATS JSON msg | validate_envelope→payload(family) | Postgres row in routed table | required-field completeness; fail-closed vs fail-open semantics differ **per family** |
| T11 | intent string | deterministic planner | `CasePlan` ops | same input ⇒ same plan (authority precedes side effects) |
| T12 | semantic candidate | `evaluate_authority` | `CandidateDisposition` feeding GovernanceVerdict | policy meaning anchored to loaded domain model, not vibes |
| T13 | delegation turns | transcript_seal | `transcript_sha256` inside evidence + claim manifest | execution-result linkage to declarations |
| T14 | model text | domainforge CLI envelope | D envelope (`content_hash`,`semantic_closure_hash`) → SemanticContext::Resolved \| ValidationFailed | malformed output must never be misclassified (sxr §10.4a) |
| T15 | sxr records | corpus adapter | `(answer, known_evidence_ids, accepted_gap_statuses)` triple | "adapts transport, never reinterprets CEP meaning" (corpus.py docstring) |

Silent-corruption candidates (structurally valid, semantically wrong): T2 fallback, T4 zero-citations, T9 slug ambiguity (two event types differing only by case/underscore collide), plus gen-1↔gen-2 namespace placement (top-level vs in-payload) — see G-4.

---

## 7. Identity / Provenance / Evidence Flow

Identifier chain (who creates, where it crosses):

| Identifier | Created by | Crosses | Replaceable? | Binding strength |
| --- | --- | --- | --- | --- |
| `work_request_id` | SWE_SEED (caller-supplied in emit builders) | every loop event, CK packet, GSA evidence | caller could substitute — correlation is convention | structural |
| `context_requirement_id` / `context_packet_id` | SWE_SEED / CK packet builder | ContextRequired→Packet→downstream | CK regenerates per call | structural |
| `event_id` | uuid4 at emission (SWE_SEED `make_event`, GSA `_event`) | envelope-level; excluded from signature | fresh on replay **by design**; dedup via idempotency_key | weak alone |
| `idempotency_key` | derived sha256(cid\|etype\|canonical payload) | NATS `Nats-Msg-Id`, GSA dedup | no — content-derived | content-bound |
| `trace_id` + `chain_entries.hash` chain + `trace_chain_root` | SWE_SEED trace ledger | gate, signed envelopes | no — hash-chained | crypto (sha256 chain), signature optional |
| Ed25519 signature | `federation sign` (SWE_SEED) | envelope → any verifier with committed pubkey | no | crypto (covers payload+type+ts+ns) |
| `claim_manifest_sha256` | sea-rs reconciler & production submitter | declaration ↔ harvested-evidence join | no — canonical hash of {case,run,plan_item,settlement,transcript_sha256,refs} | crypto (structural bind) |
| `transcript_sha256` | sea-forge-server transcript_seal | delegation evidence, claim manifest | no | crypto |
| `domain_model_hash` | resolved from SEA manifest (or fallback) | every payload + provenance chain | **silently replaceable by constant** when manifest absent (warning only) | weak → strong only with manifest present |
| `authority_decision_id` / `authority_reference` | SEA Forge decision event | passed through CK packet verbatim | pass-through only; CK never mints | structural reference |
| `ee-*` / `set-*` / `traj-*` record ids | GSA uuid hex | internal ledgers | internal only | structural |
| sxr `content_hash`/`semantic_closure_hash` | sxr-df over D envelope | sxr ledger + CEP corpus | no | crypto over semantic closure |

Provenance chains are **structural + selective cryptographic**: hashes cover payloads/chains, signatures cover envelopes, but there is no end-to-end Merkle binding across planes; the joins are id-based (`work_request_id`…) plus two crypto anchors (`trace_chain_root`, `claim_manifest_sha256`). Where hash coverage matters, tests must pin the exact hashed byte string (INT-006, INT-017).

RealityTrace association: within sxr's own loop, expected-vs-observed association is via its ledger's record kinds + checkpoint hashes (sxr-core `types.rs`, `record_kinds.csv`); the CEP gate then judges anti-collapse independently. There is no sxr attachment point inside the capability loop (**FACT** — would be an architectural addition, not a test).

---

## 8. State and Persistence Model

| Store | Owner | Format/location | Write path | Read path | Recovery | Source of truth? |
| --- | --- | --- | --- | --- | --- | --- |
| Trace ledger | SWE_SEED | SQLite `chain_entries` + `.agent-harness/traces/records/*.json` | `trace start/finish`, append-only seq per trace_id | `gate --verify`, `reflect` | chain verify detects severance | yes (execution history) |
| Route/harness projections | SWE_SEED | `.agent-harness/routes/*.json`, host projections | `sync` (regenerate idempotent — `regenerate_idempotent.rs`) | route, doctor | regenerate | projection (canonical = harness def) |
| CK store | Context_Kernel | SQLite `kernel.db` (`serve --database`), artifacts under `--data-dir` | MCP `store_context`/`ingest_source`; retention GC | `recall_context`,`query_reference`,`resolve_reference`,`context_required` | reopen DB; retention GC deletes orphan payloads | yes (context corpus cache+index) |
| GSA ledgers | godspeed_agent | `.godspeed-harness/ledgers/*.jsonl` (+indexes/config/cache) | append w/ fcntl+fsync | runtime reads, `search-developmental-memory` (RuVector bridge optional) | corrupt-line quarantine; storage manifest | yes (developmental memory) |
| SEA Forge records | sea-rs | `<root>/.sea-forge/cases|runs|capabilities.jsonl|memory/items.jsonl` | pipeline appends after each stage | inspect/recall/replay/rebuild | rebuild projections from JSONL; migrations hash-verified | yes (append-only) |
| Postgres agent_memory | hassos addon | `event_log.agent_events`, `governance.*`, `memory.items`, inbox/outbox | bridge inserts post-validation | addon queries/validators | JetStream redelivery until ACK; DLQ for poison | canonical for **replicated governance/memory view**; GSA JSONL remains upstream truth (publisher docstring: "Postgres remains canonical" refers to bridge-side routing; GSA docstring says JSONL buffer stays authority — **naming tension documented in open questions**) |
| NATS JetStream `SEA_LEDGER` | infra | stream w/ per-subject caps | publishers + bridge outbound poll | bridge pull consumer | ack_wait 30s, max_deliver 10, DLQ | transport only |
| sxr ledger | sxr | `.sxr` ledger files via `ledger_io` | append envelopes | verify/query/projections | historical immutability rule; recovery/replay cmds | yes (experiment evidence) |

Cross-store consistency requirements an E2E suite must enforce:
1. GSA JSONL settlement exists ⇒ optionally replicated Postgres row with same `idempotency_key` (when NATS path exercised); never duplicated (Nats-Msg-Id).
2. Provisional `execution_evidence` must exist **without** any `settlements`/`capabilities` write (negative invariant).
3. SEA Forge `runs/<id>/settlement.json` ⇄ declaration correlation only via exact `claim_manifest_sha256` recomputation.
4. CK `context_packet_id` appearing downstream must be traceable to a stored citation set (or, if packets aren't persisted by CK — they are return values only — the test must treat packet as ephemeral; **FACT: CK returns packets; persistence is the caller's job**).

---

## 9. Failure and Recovery Model

| Surface | Behavior today (FACT unless noted) | Danger class | Test needed |
| --- | --- | --- | --- |
| SWE_SEED federation off | `Dispatch::Suppressed` — no fs touch, no external call ("standalone invariant", tested `standalone_invariant.rs`) | safe default | FAIL-011 pin |
| SWE_SEED sink write fails | logged to stderr, returns Suppressed, inner loop continues ("ponytail" comment emit.rs:140-155) | silent transport loss | FAIL-012 |
| CK binary absent (`SWE_SEED_CONTEXT_KERNEL_BIN` unset) | client `from_env()` → None; hot path logs and proceeds (context_client.rs:16-20) | context silently skipped — completion may proceed w/o citations unless something checks POL-ACL-003 | FAIL-001 |
| CK configured but corpus root empty/mismatched | `Ok(vec![])` ⇒ **success packet with 0 citations** | semantic-loss: violates POL-ACL-003 downstream but nothing enforces in CK | FAIL-009 (pin + flag) |
| GSA incomplete evidence | typed `DomainRuleError("incomplete_execution_evidence")`, nothing written | good | FAIL-003 |
| GSA unknown affordance | typed rejection before any write | good | FAIL-004 |
| GSA duplicate delivery | returns existing record; no duplicate write | good | FAIL-005 |
| GSA ledger corruption mid-file | corrupt lines quarantined, reads continue | partial-read semantics | RESTART-001 |
| Publisher NATS down / pkg missing | no-op, logs once, buffer retained | eventual consistency | FAIL-006 |
| Bridge fail-closed invalid msg | reject → retry (ack_wait/max_deliver) → DLQ `sea.ledger.deadletter` | poison isolation | INT-006/FAIL-007 |
| Bridge fail-open family invalid | ingested with warnings (**documented deviation from plan assumption** — see e2e test docstring) | silent acceptance of garbage on agent events | FAIL-008 pin |
| sea-rs authority deny/escalate | halt before execution; trace+evidence still recorded; settlement-specific exit codes | good | E2E-002 |
| sea-rs unknown operation | deny (unknown means deny) | good | L0/L1 |
| Delegation child misbehaves | max_response/transcript byte caps; unmapped permission ⇒ `acp_permission_denied_unmapped:*` | resource/permission safety | INT-009 variants |
| Declaration forgery/replay | claim-manifest hash mismatch ⇒ never correlates | integrity | PROV-004 |
| `domain_model_hash` fallback | constant hash + warning only | silent identity downgrade | FAIL-010 |

Report-success-after-partial-failure candidates: CK zero-citation packet (G-3); fail-open agent-family ingestion of malformed events (G-5); ProofCompleted "consumption" that doesn't exist (G-6) — a ProofStarted/Completed pair alone will never reach settlement because no producer bridges proof → `EvidenceRecorded` (G-1).

---

## 10. Existing Test Coverage

### Already proves (do not duplicate)

| Area | Tests | What they prove |
| --- | --- | --- |
| SWE_SEED core (Rust) | 51 files in `crates/swe-seed-core/tests/` — notably `context_kernel_client.rs`, `federation_parity.rs` (Rust⇄Python `_load_hash`/`_event` parity), `federation_signing.rs`, `federation_verify_sea.rs` (verifies Python-signed envelopes), `standalone_invariant.rs`, `trace_ledger.rs`, `chain_integrity.rs`, `v1_contract.rs`, `gateway_*` (7), `host_projection_determinism.rs`, `route_golden.rs` | envelope/schema/hash/signing parity; CK-client protocol against a stub/spawned binary; gateway routing/concurrency; host projection determinism |
| SWE_SEED CLI (Rust) | `cli_golden.rs`, `trace_cli.rs`, `cr_review_*`, `learning_*`, `host_cli.rs` | command surface goldens |
| SWE_SEED Python | `tests/test_agentic_capability_hardening.py` (**incl. `test_full_fourteen_event_sequence_carries_domain_hash_and_trace_ids` at :392**), `tests/test_agentic_capability_contracts.py` | gen-1 14-event cross-repo sequence carrying domain hash + trace ids, using modules loaded from 4 repos via `workspace_roots.py` |
| Context_Kernel | `ck-mcp/tests/acl_context_agent.rs` (citation resolution from fs corpus; authority-ref pass-through; traversal containment implied by unit), plus broad crate tests (`ck-bin/tests/cli.rs`, auth, retention, SSE reload) | ACL adapter behavior incl. refs pass-through |
| godspeed_agent | `agentic_capability_loop/tests/test_v1_contract.py` (15 tests: v1 conformance, provisional-only ingestion, idempotency, replay key semantics, teeth: top-level namespace rejected, missing-field rejected, **vendored schema matches contracts pin**), `test_publisher.py`, `tests/test_runtime_workflow.py`, `tests/test_swe_seed_trace_ingestion.py` (path-confinement reader), `tests/e2e/test_settlement_roundtrip.py` (GSA→NATS→bridge→Postgres + fail-closed DLQ; docker-gated) | GSA-side contracts incl. real transport roundtrip |
| sea-rs | `conformance_m0_authority.rs`, `m4a/m4b capability`, `criteria_provenance.rs`, `swe_seed_cli.rs` + `conformance_m8_artifact.rs`, extensive server tests (`handle_request`-level, socket-budget logic) | authority fabric, capability/envelope append+recall, settlement criteria provenance, SWE_SEED CLI hook + reconciliation, server protocol |
| hassos addon | `tests/test_bridge.py`, `validate_sea_bridge.sh` | bridge validation stages/routing |
| cep | own pytest suite + schemas/fixtures (`schemas/fixtures`, `vendored/`) | schema validation, benchmark scorers, anti-collapse |
| sxr | embedded crate tests + `proof/`, `ablation/` | ledger/verify/adapter classification |
| gauntlet | `gauntlet-arch-tests` (`just boundaries`), doctor tests | layering only |
| domainforge | 60+ parser integration tests (`sea-core/tests/`), CLI tests | parse/validate/pack hashing |

### Not proved anywhere (the actual gaps)

1. **G-1:** No production-path producer of `EvidenceRecorded`; hence no binary-level test of SEA-Forge/proof-plane → GSA settlement flow.
2. **G-2:** No test asserts `domain_model_hash` equals the real manifest hash end-to-end through *production binaries* (parity tests compare Rust⇄Python resolution logic, including fallback — but nothing fails the *stack* when fallback fires).
3. **G-3:** Zero-citation success packets: nobody asserts ≥1 citation (POL-ACL-003) at the seam.
4. **G-4:** No cross-generation compatibility pin (gen-1 top-level `namespace` vs v1 in-payload; GSA explicitly rejects top-level namespace — a gen-1 envelope fed to GSA fails).
5. **G-6:** GSA claims to consume `ProofCompleted` but no handler exists; untested/unimplementable as documented.
6. **G-7:** No test drives `swe-seed route → CK → proof → gate → sign` as one continuous production-binary flow (pieces are covered separately).
7. **G-8:** NATS subject slug collision/ambiguity untested; bridge family-routing for `sea.agent.event.godspeed-agent.*` fail-open behavior only covered by the docker-gated e2e (skipped in CI without services).

---

## 11. Testability Assessment

**Deterministic seams already present (use them; do not invent new ones):**
- Federation master switch default-off ⇒ hermetic standalone runs; `--federation on` flips emission with stdout sink (`Dispatch::Written(PathBuf::from("-"))`) — perfect for capture.
- `SWE_SEED_CONTEXT_KERNEL_BIN` lets tests point at a locally built `ck` binary; CK gets temp `--database`/`--data-dir` (client creates them) — fully isolated CK instances.
- `InternalTestSweSeed` CLI verb reads a server-request JSON from stdin — deterministic reconciliation testing without sockets.
- `sea-forge-agent` supports in-memory transports for tests (acp.rs:362 comment) — delegation loops can run against a scripted fake agent; alternatively a tiny echo-ACP binary via `AcpSpawn.argv`.
- GSA is plain Python with injectable storage root; `SettlementPublisher(nats_url=None)` = offline no-op; e2e compose file provided (`tests/e2e/docker-compose.yml`, postgres-init.sh).
- cep evaluator is a pure function over a JSON corpus file.
- sxr operates on temp repos; `sxr-df` takes any binary path conforming to the envelope CLI contract (can point at domainforge CLI or a fixture-emitting fake).

**Dependencies that should remain real:** SWE_SEED trace SQLite, CK SQLite, GSA JSONL ledgers, Postgres+NATS in the L4 slice (they are the system under test for persistence semantics).

**Substitutable:** external coding agents in delegation (fake ACP binary/in-memory transport); Anthropic/OpenAI endpoints (provider trait + fake); NATS/Postgres in L0–L3 (absent ⇒ offline-first paths).

**Obstacles to reliable E2E (findings, not to fix):**
1. `EvidenceRecorded` producer absent — the canonical cross-plane E2E requires a thin *test-owned* producer shim (explicitly a test fixture, not production code) or driving GSA handlers directly.
2. Docker-gated e2e skips silently — CI placement must provision services or explicitly mark tier.
3. Clock/uuid nondeterminism (`occurred_at`, `event_id`, ULIDs) — assertions must be relational/idempotency-based, not value-equal.
4. Two-generation drift: the strongest existing cross-repo test exercises Python modules directly, bypassing production binaries — green ≠ integrated.

---

## 12. Canonical Golden E2E Scenario

**Name:** `golden-capability-loop-local` — smallest flow crossing intake → context → proof → gate → sign → settlement → promotion with zero external services and no network.

**Initial state (fixtures, all under one pytest tmp root):**
1. `sea-root/docs/specs/domains/agentic_capability_loop/agentic_capability_loop.manifest.json` — copy the real manifest (or synthesize with known `meta.sea_file_hash=62f4f0cd84d08682f0e9dc34a4d4166466b9c1c09fd10c181897ba909bbcb565`); sibling `agentic_capability_loop.sea` present so the hash is honest.
2. `repo/.agent-harness/routes/bugfix.json` + minimal harness files (copy from SWE_SEED fixtures used by `route_golden.rs`).
3. `corpus/acme/doc1.md` containing sentinel text `"Hello world from doc1"`.
4. GSA storage root pre-seeded via `godspeed-nav` with one **executable affordance** (`current_status=='affordance'`) — created through `start-direction`/rank flow or direct LedgerStore fixture matching `storage.py` schema.
5. Federation keys: `swe-seed federation keygen ci-key` executed in the temp repo (writes committed pub + gitignored priv).

**Configuration/environment:**
- `SEA_ROOT=<tmp>/sea-root` (so `resolve_from_root` hits HashSource::SeaRoot — no fallback).
- `SWE_SEED_CONTEXT_KERNEL_BIN=<target>/ck` ; corpus root passed via `SWE_SEED_CONTEXT_CORPUS_ROOT` (client maps to `CK_CONTEXT_CORPUS_ROOT`).
- `GSA_NATS_URL` unset (offline variant) — NATS/Postgres variant is E2E-003 (L4).
- Binaries prebuilt: `swe-seed`, `ck` (ck-bin), `godspeed-nav` (uv-installed), optional `sea-forge` for the authority slice.

**Steps:**
1. `swe-seed route "fix the failing checkout test" --record` → capture `trace_id`, `route_decision_record`.
2. Context stage: invoke the harness path that constructs `ContextKernelClient.from_env()` and calls `context_required(work_request_id, context_requirement_id, "acme", query=None, authority_decision_id="sea-decision-001", authority_reference="AuthorityChecked#evt_abc")` — or drive `swe-seed` run with federation on so `ContextRequired` envelope is emitted to stdout while the client fetches the packet.
3. `swe-seed trace-start` / execute trivial deterministic proof (`sh -c 'printf ok > proof.out; exit 0'`) / `trace-finish <trace-id> … just-ci-placeholder pass` with `proof_result_id` captured.
4. `swe-seed gate <trace-id> --verify` → expect exit 0.
5. `swe-seed federation sign <trace-id> --key ci-key` → capture signed ProofCompleted envelope JSON.
6. **Producer shim (test fixture only):** wrap the signed envelope + harvested proof facts into a canonical `EvidenceRecorded` envelope (v1 shape: namespace inside payload, `domain_model_hash`, `idempotency_key` per F-10) and deliver via `agentic_capability_loop.handle_evidence_recorded(event, runtime)` with the seeded runtime.
7. `godspeed-nav record-settlement …` for the affordance with before/after states and observed consequence.

**Expected terminal state / assertions:**
- Gate exit 0; chain length ≥ route+context+proof events; `prev_hash` linkage intact.
- Signed envelope verifies against committed pubkey (`swe-seed federation verify`); mutating one payload byte breaks it; re-signing with different `event_id` still verifies (event_id excluded).
- Packet: exactly 1 citation; `citations[0].content` contains sentinel; `authority_decision_id=="sea-decision-001"` and `authority_reference=="AuthorityChecked#evt_abc"` passed through verbatim; `domain_model_hash == manifest.meta.sea_file_hash` (NOT the fallback constant).
- GSA ledgers after step 6: exactly one `execution_evidence` record, `status=="provisional"`, match fact preserved, **no** `settlements`/`capabilities` rows yet.
- After step 7: one `settlements` record referencing affordance + evidence list; `trajectories` row appended; capability evidence count incremented; `repetition_schedules` entry exists.
- Correlation: `work_request_id` identical in route record, packet, evidence, settlement inputs.
- Negative effects that must NOT occur: second delivery of the same event (step 6 rerun) creates no second `ee-*`; no envelope with top-level `namespace` accepted anywhere (GSA teeth); no `settlements` write triggered by evidence ingestion alone.

**Why this scenario:** crosses 4 production components + 3 persisted stores + the two crypto anchors, ~seconds runtime, fully offline, and directly encodes the discovered contracts (INT-001/004/005/006/007/008/009/010).

---

## 13. Proposed Test Architecture

| Layer | Proves | Deliberately does NOT prove | Cost | Environment | Time | Determinism | CI placement |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **L0 — Contract invariants** | Pure functions: hash formulas, canonical strings, slug derivation, schema conformance of fixture envelopes (vendored pins), manifest-hash resolution order | Any process behavior | trivial | none (unit) | ms | full | every commit |
| **L1 — Component integration** | One repo, real internals: CK tool dispatch w/ temp DB; GSA runtime ledgers; sea-rs pipeline run; SWE_SEED trace+gate | Cross-repo wiring | low | cargo/pytest + tmpdirs | sec | high | every PR |
| **L2 — Pairwise cross-repo** | Real binary pairs: SWE_SEED⇄CK (spawn), GSA⇄bridge w/o docker (handler-level), sea-rs reconcile via InternalTestSweSeed, sxr⇄domainforge CLI, sxr⇄cep | Full loop continuity | medium | built binaries + venvs | sec–min | high | nightly + merge-critical jobs |
| **L3 — Canonical E2E (golden)** | §12 scenario end-to-end offline; plus authority-plane golden (`sea-forge-cli run` on demo.sea) | Transport persistence, real agents | medium | prebuilt binaries + uv venv | min | high (relational asserts) | merge gate |
| **L4 — Failure/recovery + transport E2E** | NATS/Postgres roundtrip, DLQ, restarts, concurrency, tamper/gate failures, fallback degradation | Real LLM behavior | high | docker compose (provided) + binaries | min | high given fixed services | nightly |
| **L5 — Real-adapter smoke** | Actual delegation to a real coding-agent binary behind `AcpSpawn`; real Home Assistant add-on container | Deterministic outcomes (smoke only) | high | secrets + network/hosts | min+ | low | weekly/manual |

Derivation note: this mirrors the discovered seams rather than imposing a template — L2 pairs correspond 1:1 to INT-001…INT-020; L3 encodes §12; L4 targets §9 rows.

---

## 14. Proposed Test Catalog

Format: ID — purpose / path / components / contracts / setup / execution / assertions / failure signal / mocking / evidence.

### L0 contract tests

**CONTRACT-001 envelope-v1-shape-parity**
- Purpose: prevent structural drift of the canonical envelope across producers/consumers (G-4).
- Path/contracts: INT-004, INT-010, INT-012.
- Components: SWE_SEED `Envelope` serde, GSA `_event`, bridge `validate_envelope`, vendored schema (`cep/schemas/vendored` + GSA contracts pin).
- Setup: generate one envelope per producer with fixed inputs.
- Execution: validate each against `sea.agent.event.v1.json`; assert `namespace` ∈ payload only; assert GSA rejects top-level `namespace` (teeth test exists — reuse shape).
- Assertions: identical required field sets; `additionalProperties:false` honored; `schema_version=="v1"`.
- Failure signal: any producer emitting gen-1 top-level namespace or extra fields.
- Mocking: none.
- Evidence: envelope.rs:41-56,199-218; adapters.py:66-105; sea_nats_bridge.py:361-430; test_v1_contract.py:230-248.

**CONTRACT-002 idempotency-key-derivation-parity**
- Purpose: dedup correctness across Rust/Python (F-10).
- Execution: compute key for identical logical content with fresh event_id in both implementations; assert equal; assert distinct for changed payload/correlation.
- Assertions: equality/non-malleability; replay keeps key, refreshes event_id.
- Evidence: adapters.py:31-45; test_v1_contract.py:206-228; envelope.rs make_event.

**CONTRACT-003 signing-canonical-string-parity**
- Purpose: byte-exact Ed25519 signing input across languages.
- Execution: build canonical string for a fixed envelope; compare Rust bytes to Python `cryptography` path via the committed test vector; sign with dalek, verify with Python (and vice versa if available).
- Assertions: byte equality; signature verifies both directions; mutation of payload/type/ts/ns breaks it; changing `event_id` does not.
- Evidence: signing.rs:1-40; federation_verify_sea.rs.

**CONTRACT-004 domain-model-hash-resolution-order**
- Purpose: pin INT-005 ordering + expose fallback (G-2).
- Execution: matrix — SEA_ROOT valid / stale path / SEA_MANIFEST_PATH / marker walk / nothing.
- Assertions: correct `hash` + `HashSource`; fallback yields `sha256("agentic_capability_loop")` AND `warned=true` (Rust) / `UserWarning` (Python).
- Evidence: envelope.rs:83-190; adapters.py:15-24.

**CONTRACT-005 trace-chain-hash-formula**
- Purpose: tamper evidence.
- Execution: build chain, recompute hashes, flip one payload → verify fails listing severed seq.
- Assertions: genesis prev == GENESIS_PREV; formula `SHA256(etype‖payload_json‖prev_hash)`.
- Evidence: trace_ledger.rs:4-36,152-190; chain_integrity.rs (exists — extend only if gap).

**CONTRACT-006 claim-manifest-hash-stability**
- Purpose: reconciliation join correctness.
- Execution: recompute `swe_seed_claim_manifest_sha256` for identical inputs (different orders of `harvested_refs` sorted canonically?) — assert stable; altered any field → different hash.
- Assertions: canonical serialization; mismatch never correlates (pairs with INT-010).
- Evidence: swe_seed_reconciliation.rs:27-52.

**CONTRACT-007 context-packet-payload-schema**
- Purpose: freeze ContextPacketCreated shape incl. optional authority refs.
- Assertions: required keys; citations element keys; optional refs serialize only when present.
- Evidence: acl module 47-69.

**CONTRACT-008 subject-slug-derivation**
- Purpose: pin T9 transform + document collision hazard.
- Assertions: `SettlementRecorded`→`settlement-recorded`; `CoherenceBreakDetected`→`coherence-break-detected`; two synthetic types differing only by case/underscore map to same subject (documenting lossy nature).
- Evidence: publisher.py:_slug.

**CONTRACT-009 bridge-route-table-contract**
- Purpose: freeze CANONICAL_ROUTES (family→schema/table/required/fail-openness).
- Execution: table-driven validation of sample payloads per family.
- Assertions: required-field enforcement matches table; agent family fail-open, governance/memory fail-closed.
- Evidence: sea_nats_bridge.py:158-192,431-520; pairs with addon test_bridge.py.

**CONTRACT-010 sea-policy-to-code-trace**
- Purpose: keep POL-ACL names/policies in `.sea`/manifest aligned with code constants that reference them (SWE_SEED POL-ACL-001/003 comments; GSA validation).
- Execution: parse manifest policies list; assert referenced codes exist and rationale strings unchanged (golden).
- Evidence: ACL manifest policies block; context_client.rs:4; validation.py.

### L1 component integration

**INT-L1-001 ck-context-required-dispatch** — real `McpService` + temp corpus: happy path, traversal corpus_id rejected (error), missing corpus → 0-citation packet (assert current behavior; flagged gap G-3), learning-proposals corpus reads GSA-format JSONL. Evidence: acl module 111-225; ck-bin main.rs:423-435.

**INT-L1-002 gsa-ledger-semantics** — ingest/promote/idempotency/quarantine/redaction against real `LedgerStore` (extends existing runtime tests with restart-in-between). Evidence: storage.py, runtime.py.

**INT-L1-003 sea-forge-minimum-run** — `sea-forge-cli run` with allow-policy on fixture intent → assert `.sea-forge` tree contents (plan.json determinism, authority.json decisions, evidence hashes verify, settlement.json independent of exit code, semantic-envelope appended to capabilities.jsonl). Evidence: ARCHITECTURE.md §4.1/4.2; conformance_m* tests.

**INT-L1-004 swe-seed-route-gate-local** — route→trace→gate on temp repo with fixture route card; unrouted trace fails gate. Evidence: gate_cli.rs; route_golden.rs.

### L2 pairwise cross-repo

**INT-001 swe-seed-to-ck-real-binaries** — spawn real `ck` via `ContextKernelClient::spawn` with temp db/data-dir + corpus fixture; assert packet citations + authority-ref pass-through + latency-bounded single resp line protocol. Failure signal: malformed NDJSON handling, stderr noise on stdout. Mocking: none (this is the integration). Evidence: context_client.rs:43-160; complements existing `context_kernel_client.rs`.

**INT-005 gsa-ingests-signed-envelope-file** — take CONTRACT-003-signed ProofCompleted + hand-built EvidenceRecorded (fixture shim) from file → `handle_evidence_recorded` → provisional record; tampered envelope rejected by signature verify before ingestion. Evidence: INT-006/008.

**INT-006 nats-bridge-roundtrip (docker)** — formalize existing `test_settlement_roundtrip.py` expectations: SettlementRecorded lands in `event_log.agent_events`; governance-family malformed → DLQ; duplicate Nats-Msg-Id ingested once. Keep skip-guard. Evidence: tests/e2e/*; sea_nats_bridge.py.

**INT-009 delegation-fake-acp-agent** — `AcpSpawn` a scripted echo-ACP binary (fixture script speaking minimal ACP over stdio); drive one delegation turn; assert transcript sealed (sha256 recorded), permission denial mapping for unmapped tool kind, budget cap enforcement. Evidence: acp.rs:313-360,958-1030; delegation.rs.

**INT-010 declaration-reconciliation** — via `sea-forge InternalTestSweSeed` stdin request: append declaration with correct claim hash → correlation committed once; repeat → idempotent; forged hash → no correlation. Evidence: main.rs:492-504; swe_seed_reconciliation.rs; existing swe_seed_cli.rs extended.

**INT-013 sxr-domainforge-cli-classification** — point `CliDomainForgeAdapter` at real domainforge CLI with (a) valid mini `.sea`, (b) invalid model, (c) CLI emitting garbage; assert Resolved(+hashes)/ValidationFailed/error classification-by-shape. Evidence: sxr-df/adapter.rs:1-60; domainforge cli/envelope.rs.

**INT-014 sxr-export-to-cep-gate** — produce corpus export from a small sxr ledger; run `cep conformance` corpus evaluation; assert PASS on honest records; inject fabricated answer w/o evidence ids → FAIL. Evidence: corpus.py:31-80.

**INT-015 gen1-hardening-drift-detector** — run existing `SWE_SEED/tests/test_agentic_capability_hardening.py` with `workspace_roots` env pointing at the workspace; track (not necessarily block) failures as generation drift signal. Evidence: workspace_roots.py; hardening:392.

### L3 canonical E2E

**E2E-001 golden-capability-loop-local** — exactly §12. PASS criteria enumerated there. Components: swe-seed, ck, GSA. Contracts: INT-001,004,005,006,007,008,009,010.

**E2E-002 authority-deny-halt-path** — `sea-forge-cli run` with policy denying the planned op: expect governed-halt exit code, **no** workspace/artifact writes beyond records, trace+evidence contain denial, no semantic-envelope success appended. Evidence: ARCHITECTURE §3 invariant 1; authority lib.

### L4 failure/recovery E2E

**FAIL-001 ck-unconfigured-hot-path** — full route→proof with `SWE_SEED_CONTEXT_KERNEL_BIN` unset: completes standalone; assert NO packet anywhere and (flag) no POL-ACL-003 enforcement fires (documents G-3 exposure).
**FAIL-003/004/005 gsa-rejections** — as §9 (typed errors, zero ledger writes, idempotent redelivery).
**FAIL-006 publisher-offline-buffering** — GSA_NATS_URL unreachable: drain no-ops; JSONL buffer retains; later successful drain flushes exactly once (Nats-Msg-Id dedup at stream).
**FAIL-007 bridge-poison-to-dlq** — fail-closed family invalid envelope → DLQ subject receives; original acked; stream healthy.
**FAIL-008 agent-family-fail-open-pin** — malformed agent event → ingested w/ warning row (pins actual semantics; guards against accidental tightening/loosening).
**FAIL-009 zero-citation-exposure** — E2E-001 variant with empty corpus: packet succeeds with 0 citations; assert downstream gate/settlement does NOT block (documents G-3) — test carries `xfail`-style annotation intent: it encodes a defect contract until fixed.
**FAIL-010 hash-fallback-degradation** — run E2E-001 without SEA_ROOT: all envelopes carry fallback constant; assert warning surfaced (stderr/log) and `HashSource::Fallback` reported by `federation status` (G-2 visibility).
**FAIL-011 standalone-invariant-sweep** — with federation off, run entire golden flow: assert zero files created outside repo-scoped stores (no sinks touched).
**FAIL-012 sink-write-failure** — read-only sink dir: dispatch returns Suppressed, stderr log written, flow continues.
**RESTART-001 gsa-restart-quarantine** — append garbage byte-line to a ledger; restart runtime; reads skip+quarantine; appends still work; storage manifest intact.
**RESTART-002 bridge-redelivery** — kill bridge mid-batch (before ACK): JetStream redelivers; no duplicates in Postgres (idempotency by Nats-Msg-Id/message-id derivation).
**CONC-001 gsa-concurrent-append** — N threads appending distinct records: no interleaving corruption (fcntl), all present, order total per file.
**CONC-002 publisher-dedup-race** — two drains of same buffered envelope: single Postgres row.
**PROV-001 transcript-seal-binding** — delegation transcript hash equals recorded `transcript_sha256`; claim manifest incorporating it verifies.
**PROV-002 identifier-lineage** — golden scenario: extract {work_request_id, context_packet_id, proof_result_id, evidence_event_id, settlement id} and assert each downstream artifact references its upstream id exactly.
**PROV-003 signature-coverage** — mutate each signed field class (payload/type/ts/ns vs event_id): verify fails for former, passes for latter.
**PROV-004 forged-declaration-rejection** — declaration with recomputed-but-wrong run/settlement binding never correlates (INT-010 inverse).

### L5 smoke (optional)

**SMOKE-001 real-agent-delegation** — delegate trivial task to installed real agent binary via server `delegate` (requires local credentials; manual/weekly).
**SMOKE-002 addon-container** — add-on image up; bridge + validators green (`validate_sea_bridge.sh`).

---

## 15. Runtime Coverage Matrix

Stages (actual): Intake(route) · SemRep(domain hash/envelopes) · Orch(proof gating) · Ctx(CK) · Auth(SEA authority) · Exec(sandbox/delegation) · Obs(trace/evidence) · Settle(GSA) · Persist(JSONL/SQLite/PG) · Trans(NATS/MCP/socket) · Verify(gate/cep) · Sign(provenance)

| Test | Intake | SemRep | Orch | Ctx | Auth | Exec | Obs | Settle | Persist | Trans | Verify | Sign |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| CONTRACT-001 | | ✓ | | | | | ✓ | ✓ | | ✓ | ✓ | |
| CONTRACT-002/003 | | ✓ | | | | | | ✓ | | | | ✓ |
| CONTRACT-004 | | ✓ | | | | | | | | | | ✓ |
| CONTRACT-005/006 | | | ✓ | | | | ✓ | | ✓ | | ✓ | ✓ |
| CONTRACT-007 | | | | ✓ | | | | | | ✓ | | |
| CONTRACT-008/009 | | | | | | | ✓ | ✓ | ✓ | ✓ | | |
| CONTRACT-010 | | ✓ | | | ✓ | | | | | | | |
| INT-L1-001 | | | | ✓ | | | | | | ✓ | | |
| INT-L1-002 | | | | | | | | ✓ | ✓ | | | |
| INT-L1-003 | | ✓ | ✓ | | ✓ | ✓ | ✓ | ✓ | ✓ | | | |
| INT-L1-004 | ✓ | | ✓ | | | | ✓ | | ✓ | | ✓ | |
| INT-001 | | | | ✓ | | | | | | ✓ | | |
| INT-005/006 | | ✓ | | | | | | ✓ | ✓ | ✓ | | ✓ |
| INT-009/010 | | | ✓ | | ✓ | ✓ | ✓ | | ✓ | | | ✓ |
| INT-013/014 | | ✓ | | | | | ✓ | | ✓ | | ✓ | |
| INT-015 | ✓ | ✓ | ✓ | ✓ | | | ✓ | ✓ | | | ✓ | |
| E2E-001 | ✓ | ✓ | ✓ | ✓ | | | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| E2E-002 | ✓ | | ✓ | | ✓ | | ✓ | ✓ | ✓ | | | |
| FAIL-001..012 | ✓ | ✓ | ✓ | ✓ | | | ✓ | ✓ | ✓ | ✓ | ✓ | |
| RESTART/CONC | | | | | | | | ✓ | ✓ | ✓ | | |
| PROV-001..004 | | ✓ | ✓ | | | ✓ | ✓ | ✓ | ✓ | | ✓ | ✓ |

Untested-stage exposure after catalog: none uncovered outright; weakest cells are **Auth** (covered only by L1-003/E2E-002 + existing sea-rs conformance) and **Exec** delegation realism (fake-agent only until L5).

---

## 16. Contract Coverage Matrix

| Integration | Contract crux | Existing coverage | Proposed coverage | Remaining gap |
|---|---|---|---|---|
| INT-001 SWE_SEED→CK | MCP stdio tool call/response | `context_kernel_client.rs`, `acl_context_agent.rs` | INT-001 (real both-sides), CONTRACT-007 | stderr/stdout discipline under load |
| INT-002/003 CK corpus | adapter port + env wiring | `acl_context_agent.rs` partial | INT-L1-001, FAIL-009 | zero-citation policy enforcement (arch change needed to *fix*) |
| INT-004 envelopes | v1 schema + gated dispatch | `v1_contract.rs`(SEED), GSA teeth | CONTRACT-001, FAIL-011/012 | cross-producer single-schema test |
| INT-005 domain hash | resolution order/fallback | `federation_parity.rs` | CONTRACT-004, FAIL-010 | stack-level downgrade alarm |
| INT-006 signing | canonical bytes | `federation_signing.rs`, `federation_verify_sea.rs` | CONTRACT-003 | — (extend only) |
| INT-007 chain | hash chain + gate | `trace_ledger.rs`, `chain_integrity.rs` | CONTRACT-005, INT-L1-004 | gate-as-merge-gate wiring |
| INT-008/009 GSA settlement | provisional/promotion | `test_v1_contract.py`, runtime tests | INT-L1-002, E2E-001 steps 6-7 | producer absence (G-1) |
| INT-010/011 GSA emit/publish | key derivation + subjects | `test_v1_contract.py`, `test_publisher.py` | CONTRACT-002/008, FAIL-006, CONC-002 | slug collision semantics |
| INT-012/013 bridge/PG | routing/validation/DLQ | `test_bridge.py`, docker e2e | CONTRACT-009, INT-006, FAIL-007/008, RESTART-002 | always-on CI provisioning |
| INT-014 authority⇄domainforge | semantic candidate eval | `conformance_m0_authority.rs` | INT-L1-003 (uses it) | version-pin drift watch (=0.16.0) |
| INT-015 server/SFWP | additive verbs/versioning | server tests | INT-009/010 use socket-less hooks | full verb-matrix golden (P2) |
| INT-016 delegation/ACP | spawn/permission/seal | in-memory transport tests | INT-009, PROV-001, SMOKE-001 | realistic agent variance (L5 only) |
| INT-017 reconciliation | claim hash join | `swe_seed_cli.rs` | INT-010, CONTRACT-006, PROV-004 | out-of-process actor path |
| INT-018 GSA ledgers | append/quarantine | runtime/storage tests | RESTART-001, CONC-001 | torn-write under crash (fsync assumed) |
| INT-019/020 sxr⇄DF⇄CEP | shape classification, corpus gate | sxr/cep internal tests | INT-013/014 | cross-repo CI glue |
| INT-021 py cross-repo rig | roots discovery | hardening suite itself | INT-015 | keeping gen-1 modules importable as repos evolve |
| INT-022 GSA→CK learning packs | file handoff | `acl_context_agent.rs` (ledger branch) | INT-L1-001 variant | format pin for learning-proposals JSONL |
| INT-023 semantic packs | expectedHash pin | none found end-to-end | none proposed (UNKNOWN loader) | loader location unidentified — open question |

---

## 17. Test Fixtures and Harness Requirements

**Binaries to build (cargo):** `swe-seed` (SWE_SEED), `ck` (Context_Kernel `ck-bin`), `sea-forge` (sea-rs `sea-forge-cli`), optionally `sea-forge-server` for L2 socket tests. Python: `godspeed-nav` via `uv sync` in godspeed_agent; `cep` via its pyproject; bridge needs the **addon repo's venv** (psycopg, nats) — existing e2e runs it as subprocess from `LEDGER_ROOT`.

**Working-directory assumptions:** one tmp root per test holding `sea-root/` (manifest+.sea), `repo/` (with `.agent-harness/`), `corpus/`, GSA `storage/`. Cross-repo Python rig needs `GODSPEED_WORKSPACE_ROOT` or the four `*_ROOT` vars (workspace_roots.py).

**Environment variables (complete list):** `SEA_ROOT`, `SEA_MANIFEST_PATH` (alt), `SWE_SEED_CONTEXT_KERNEL_BIN`, `SWE_SEED_CONTEXT_CORPUS_ROOT` (mapped to CK's `CK_CONTEXT_CORPUS_ROOT`), `GSA_NATS_URL` (unset for offline tiers), `E2E_NATS_URL`/`E2E_PG_DSN` (docker tier), `GODSPEED_WORKSPACE_ROOT`/`CONTEXT_KERNEL_ROOT`/`GODSPEED_AGENT_ROOT` (py rig). No secrets required below L5.

**Ports/services (L4 only):** NATS 4222, Postgres 5432, bridge health 8099 — provisioned by `godspeed_agent/tests/e2e/docker-compose.yml` + `postgres-init.sh`.

**Databases/stores:** temp SQLite for CK (`--database`, `--data-dir` auto-created by client), SWE_SEED SQLite ledger in temp repo, GSA JSONL root, sea-rs `.sea-forge` under temp root, Postgres schema from addon migrations.

**Canonical fixtures to author:**
1. `fixtures/sea-root/…/agentic_capability_loop.manifest.json` (+ `.sea`) — real copy pinned by hash.
2. `fixtures/corpus/acme/doc1.md` sentinel corpus.
3. `fixtures/route-card/bugfix.json` (mirror of SWE_SEED route golden).
4. `fixtures/keys/` — generated per-run via `federation keygen` (never commit private keys).
5. `fixtures/evidence_recording_shim.py` — test-only producer converting a signed ProofCompleted + proof facts into a v1 `EvidenceRecorded` envelope (exists solely because of G-1; clearly labeled).
6. `fixtures/fake-acp-agent.(rs|py)` — scripted ACP responder for INT-009.
7. Reuse existing: `sea-forge-domainforge/tests/fixtures/demo.sea`, addon compose files, SWE_SEED route/host fixtures.

**Determinism controls:** no fixed clocks possible (`occurred_at` RFC3339 — assert format + monotonicity); ids are uuid4/ULID — assert relationally; hash assertions use explicit expected digests (constants above); sort JSON canonically before comparison; single-threaded proof command (`sh -c`) for Exec.

**Startup/shutdown ordering (L4):** compose up → wait 4222+5432 reachable (existing `_reachable()` pattern) → apply migrations (postgres-init) → run bridge subprocess with env from `BridgeConfig.from_env` → run flow → assert → SIGTERM bridge → compose down. Readiness criterion: bridge health endpoint 8099 OK + pg SELECT 1.

**Cleanup:** all state inside pytest tmp_path; docker volumes pruned by compose down; no writes into real repos (tests must chdir into tmp repo for `swe-seed` invocations).

**PASS/FAIL definition:** PASS = every asserted observable (state rows, hashes, signatures, correlations, absence-of-forbidden-effects) holds; process exit codes only where the contract IS the code (gate, CLI errors). FAIL = any contract violation, including "succeeded but wrong semantics" cases (FAIL-009/010 encode known defects deliberately).

---

## 18. Recommended Implementation Sequence

1. **Phase 0 — Foundations (prereq: none):** build binaries once into a shared target dir; author fixtures §17.1-5; L0 CONTRACT-001…010 (pure, fast, immediately valuable).
2. **Phase 1 — Single-repo integration (prereq: Phase 0):** INT-L1-001…004. Extends existing suites; no cross-repo risk.
3. **Phase 2 — Pairwise (prereq: 1):** INT-001 (SWE_SEED⇄CK), INT-005, INT-010, INT-013, INT-014. Highest-value seams first (INT-001 is P0).
4. **Phase 3 — Canonical E2E (prereq: 2):** E2E-001 golden; then E2E-002 authority-deny.
5. **Phase 4 — Failure/recovery (prereq: 3):** FAIL-001…012, RESTART-001/002, CONC-001/002, PROV-001…004 (many are parameterized variants of golden).
6. **Phase 5 — Transport persistence (prereq: 4 + docker availability):** INT-006 formalization, FAIL-007/008, CONC-002 in CI with services.
7. **Phase 6 — Drift & optional (prereq: any):** INT-015 gen-1 drift tracker (report-only), SMOKE-001/002 (scheduled/manual).

Rationale: every phase depends only on earlier phases' fixtures/binaries; P0 contracts land before expensive orchestration.

---

## 19. Architectural / Test Gaps (cannot be covered by tests without architecture changes — do NOT change)

1. **G-1 EvidenceRecorded producer absence.** Until a production component emits settlement-plane evidence, the cross-plane loop is completed only by test shims. Tests can prove each edge and the shim-mediated whole, but not autonomous system behavior.
2. **G-2 Silent domain_model_hash fallback.** A "stack refuses to run on fallback identity" policy does not exist; tests can only observe the downgrade (FAIL-010).
3. **G-3 Zero-citation packets succeed.** Enforcing POL-ACL-003 at CK or at SWE_SEED consumption is an architecture/product decision; today only downstream conventions reference it.
4. **G-4 Generation split.** gen-1 (top-level namespace, Python loop) and gen-2 (in-payload) are mutually incompatible at the envelope teeth; convergence or explicit deprecation is an owner decision (open question Q-3).
5. **G-6 ProofCompleted consumption undocumented-impossible.** Either implement a handler or amend HARNESS/adapters docstring; tests can only pin current absence.
6. **G-8 CI service dependency.** Transport-persistence truths stay docker-gated; making them first-class requires CI provisioning (infra change).
7. **INT-023 loader location unknown.** If semantic-pack loading is expected inside the addon, the loading code path could not be found in-repo (grep for `sea_core`/`semantic_pack` empty); clarifying ownership is prerequisite to any pack-integrity E2E.

---

## 20. Open Questions and Uncertainties

- **Q-1:** Is the gen-1 Python loop (`SEA/libs/agentic_capability_loop`, `Context_Kernel/integration/agentic_capability_loop/adapters.py`) still a supported surface, or is it frozen reference material? (Its fourteen-event test is the only full-sequence proof that exists.)
- **Q-2:** Who is the intended production emitter of `EvidenceRecorded` — SEA Forge server post-settlement, SWE_SEED gate, or the addon bridge outbox? No code today. (Owner decision recorded nowhere findable.)
- **Q-3:** Which envelope generation is normative for new consumers — v1 in-payload namespace (schema+vendored pin) appears normative; confirm gen-1 deprecation.
- **Q-4:** `sea_nats_bridge.py` header says "Postgres remains canonical" while GSA treats JSONL as authority and NATS as drain. Which is normative for conflict resolution? (Both statements found; possibly scoped differently: canonical *for governance/memory views*.)
- **Q-5:** Does any component consume CK's SSE transport in the loop? Only stdio observed (context_client.rs §14 note "never NATS"; SSE used by general CK clients — UNKNOWN for stack).
- **Q-6:** `sea-forge-cell`, `sea-forge-thoth`, `sea-forge-self-model`, `artifact-ip` roles sampled only; their E2E relevance unassessed beyond server views (thoth view exists in SFWP).
- **Q-7:** Whether `harvested_refs` ordering affects `claim_manifest_sha256` (canonicalization assumed via `hash_canonical`; not individually verified).
- **Q-8:** edgeai relationship limited to pattern citation — confirm no hidden build-time coupling (none found by grep/import scan).
- **Q-9:** `godspeed` / `godspeed-growth` repos (outside workspace file) — presumed unrelated; not inspected (UNKNOWN).
- **Q-10:** Whether CK persists `ContextPacketCreated` anywhere (treated as ephemeral return value here; no persistence code seen).

---

## 21. Source Evidence Index

Most important symbols/files for the implementing agent:

**SWE_SEED**
- `crates/swe-seed/src/cli.rs:38-145` — full command enum (Route/Trace/Gate/Federation/Gateway…)
- `crates/swe-seed/src/federation_cli.rs:8-80` — Federation actions; `run --federation on` smoke
- `crates/swe-seed/src/gateway_cli.rs:14-60` — Gateway skeleton (stage-1)
- `crates/swe-seed-core/src/federation/envelope.rs:21-56,86-190,199-218,258` — Envelope, hash resolution, make_event, trace_chain_root
- `crates/swe-seed-core/src/federation/emit.rs:17-158` — event builders + gated dispatch
- `crates/swe-seed-core/src/federation/context_client.rs:1-160` — MCP stdio client (spawn/env/protocol)
- `crates/swe-seed-core/src/federation/signing.rs:1-80` — Ed25519 canonical-string contract
- `crates/swe-seed-core/src/federation/flags.rs:1-120` — FederationConfig/plane modes/standalone default
- `crates/swe-seed-core/src/trace_ledger.rs:1-190` — chain DDL/hash/verify
- `crates/swe-seed-core/tests/` — 51 integration files (see §10)
- `tests/workspace_roots.py`, `tests/test_agentic_capability_hardening.py:392`

**Context_Kernel**
- `crates/ck-mcp/src/agentic_capability_loop.rs:1-225` — module doc (roles), EventEnvelope(latent), ContextRequiredPayload, Citation, ContextPacketCreatedPayload, AclContextAgent (containment, learning-proposals branch)
- `crates/ck-mcp/src/lib.rs:119-145,435-477` — tool dispatch + `handle_context_required`
- `crates/ck-bin/src/main.rs:32-54,138-175,423-435` — Serve, SSE option, corpus-root env wiring
- `crates/ck-mcp/tests/acl_context_agent.rs` — existing behavior pins
- `context-kernal.toml.example`

**godspeed_agent**
- `pyproject.toml` (entry point), `bin/godspeed-nav`
- `godspeed_nav/cli.py:62-184` — 18 subcommands
- `godspeed_nav/runtime.py:348-493` — record_settlement / ingest_execution_evidence / _require_executable_affordance
- `godspeed_nav/storage.py:12-96` — LedgerStore
- `agentic_capability_loop/adapters.py:1-146` — emitters, idempotency/provenance, handle_evidence_recorded
- `agentic_capability_loop/publisher.py:1-100,191` — SettlementPublisher + drain
- `mcp/cybernetic_nav_server/server.py` — 21 MCP tools
- `tests/e2e/test_settlement_roundtrip.py` + `docker-compose.yml`, `agentic_capability_loop/tests/test_v1_contract.py`

**sea-rs**
- `Cargo.toml` — 22-crate workspace
- `crates/sea-forge-cli/src/main.rs:26-67,492-504` — Commands incl. InternalTestSweSeed stdin hook
- `crates/sea-forge-server/src/lib.rs:5,46,263-267,855-863` — socket server essentials
- `crates/sea-forge-server/src/sfwp/mod.rs:1-40` — SFWP additive protocol v1
- `crates/sea-forge-server/src/delegation.rs:1-60` — DelegationRequest/actor_role
- `crates/sea-forge-server/src/swe_seed_reconciliation.rs:1-90` — claim-manifest join
- `crates/sea-forge-agent/src/acp.rs:313-360,958-1030` — spawn/permission mapping
- `crates/sea-forge-authority/src/lib.rs:1-70` — GovernanceVerdict/ActionGrant/evaluate_authority import
- `crates/sea-forge-domainforge/Cargo.toml:12` — `domainforge-core = "=0.16.0"`
- `ARCHITECTURE.md` §3/§4 (invariants, minimum run, records layout)

**hassos-addon-agent-memory-ledger**
- `agent_memory_ledger/rootfs/usr/bin/sea_nats_bridge.py:64-101,158-192,200-235,341-520`
- `rootfs/etc/s6-overlay/s6-rc.d/sea-nats-bridge/run` — enable gates
- `rootfs/usr/share/agent_memory_ledger/agent_memory/001_event_log.sql`, `004_inbox_outbox.sql`, `006_governance_events.sql`
- `tests/test_bridge.py`

**domainforge / sxr / cep / gauntlet**
- `domainforge-core/src/cli/mod.rs:62` + `cli/envelope.rs` — CLI envelope subcommand
- `domainforge-core/src/semantic_pack/canonical_json.rs:98+` — compute_pack_content_hash
- `sxr/sxr-df/src/adapter.rs:1-60`; `sxr/sxr-cli/src/main.rs:36-104`; `sxr/sxr-core/src/{chain,types,record_kinds}.rs`
- `cep/src/cep/conformance/corpus.py:1-80`; `cep/schemas/semantic-envelope.schema.json`, `schemas/vendored/`
- `gauntlet/crates/gauntlet-cli/src/main.rs:17-50` — doctor + reference digest only
- `gauntlet/GAUNTLET.md`, `justfile` — policy contract + gates

**Domain model / gen-1**
- `SEA/docs/specs/domains/agentic_capability_loop/agentic_capability_loop.sea` (+ `.manifest.json` `meta.sea_file_hash`)
- `SEA/libs/agentic_capability_loop/{authority_service.py,policies/,signing.py}`
- `Context_Kernel/integration/agentic_capability_loop/adapters.py`
- Prior art (stale in parts): `SWE_SEED/.agent-harness/reports/godspeed_stack_agentic_loop_wiring_report.md`

---

### Final verification checklist (mission §Final Verification)

- [x] Canonical boundary established from implementation evidence (§3)
- [x] Canonical entry points identified (§4.1: swe-seed CLI; §4.2 sea-forge-cli; terminals listed)
- [x] Terminal results identified (§4.1 end of section)
- [x] E2E loop reconstructed with branches/async (§4.1–4.4)
- [x] P0/P1 integrations have producer/contract/consumer detail (§5 INT-001…017)
- [x] Semantic transformation boundaries identified (§6)
- [x] Identity/provenance traced (§7)
- [x] RealityTrace role traced from implementation (§4.3, §3 row 8)
- [x] State/persistence boundaries (§8)
- [x] Failure semantics inspected (§9)
- [x] Existing tests inventoried (§10)
- [x] Canonical deterministic E2E specified (§12)
- [x] Test catalog with stable IDs (§14)
- [x] Runtime + contract coverage matrices (§15, §16)
- [x] Fixtures/harness requirements (§17)
- [x] Implementation sequence (§18)
- [x] Uncertainties recorded (§19, §20)
- [x] Major claims carry source evidence (inline citations throughout)
- [x] No implementation/test code written; no repo files changed except this report
