# SEA Forge Documentation Map

This document is the navigational index and sitemap for the **SEA Forge Technical Knowledge System**. It categorizes every page by reader need, Diátaxis quadrant, audience level, prerequisites, and cross-cutting relationships.

---

## Navigation by Intent

* **Learn the System:** [Orientation (`index.md`)](file:///c:/Users/sprim/projects/sea-rs/docs/index.md) → [Mental Model (`mental-model.md`)](file:///c:/Users/sprim/projects/sea-rs/docs/mental-model.md) → [Architecture Overview (`architecture.md`)](file:///c:/Users/sprim/projects/sea-rs/docs/architecture.md) → [Subsystems](file:///c:/Users/sprim/projects/sea-rs/docs/documentation-map.md#3-subsystems-layer-3)
* **Get Something Done:** [Tutorials](file:///c:/Users/sprim/projects/sea-rs/docs/documentation-map.md#6-tutorials-layer-6) and [How-To Guides](file:///c:/Users/sprim/projects/sea-rs/docs/documentation-map.md#7-how-to-guides-layer-7)
* **Understand Why:** [Explanation & Design Rationale](file:///c:/Users/sprim/projects/sea-rs/docs/documentation-map.md#5-explanation--design-rationale-layer-5)
* **Look Something Up:** [Technical Reference](file:///c:/Users/sprim/projects/sea-rs/docs/documentation-map.md#8-technical-reference-layer-8) and [Glossary](file:///c:/Users/sprim/projects/sea-rs/docs/reference/terminology.md)
* **Follow Execution:** [Workflows & Execution Traces](file:///c:/Users/sprim/projects/sea-rs/docs/documentation-map.md#4-workflows--execution-traces-layer-4)
* **Find the Implementation:** [Source Traceability Map (`source-map.md`)](file:///c:/Users/sprim/projects/sea-rs/docs/source-map.md)

---

## Master Page Catalog

### 0. Orientation (Layer 0)

| Page | Purpose | Primary Audience Need | Diátaxis | Prerequisites | Related Pages |
|---|---|---|---|---|---|
| [`index.md`](file:///c:/Users/sprim/projects/sea-rs/docs/index.md) | High-level entry point and orientation | Newcomer needing rapid system understanding | Orientation | None | `mental-model.md`, `architecture.md` |

### 1. Conceptual Foundation (Layer 1)

| Page | Purpose | Primary Audience Need | Diátaxis | Prerequisites | Related Pages |
|---|---|---|---|---|---|
| [`mental-model.md`](file:///c:/Users/sprim/projects/sea-rs/docs/mental-model.md) | Explains the philosophy and mechanics of governed execution | Engineer wanting conceptual structure before code | Explanation | `index.md` | `architecture.md`, `subsystems/kernel-pipeline.md` |

### 2. Architecture Overview (Layer 2)

| Page | Purpose | Primary Audience Need | Diátaxis | Prerequisites | Related Pages |
|---|---|---|---|---|---|
| [`architecture.md`](file:///c:/Users/sprim/projects/sea-rs/docs/architecture.md) | Canonical high-level architectural specification, topology, and invariants | Systems engineer needing boundaries, layers, and constraints | Reference / Explanation | `index.md`, `mental-model.md` | `source-map.md`, all subsystem guides |

### 3. Subsystems (Layer 3)

| Page | Purpose | Primary Audience Need | Diátaxis | Prerequisites | Related Pages |
|---|---|---|---|---|---|
| [`subsystems/kernel-pipeline.md`](file:///c:/Users/sprim/projects/sea-rs/docs/subsystems/kernel-pipeline.md) | Core types, IDs, path validation, errors, and synchronous lifecycle | Developer modifying kernel crates | Explanation / Reference | `architecture.md` | `subsystems/authority-fabric.md`, `subsystems/settlement-evidence.md` |
| [`subsystems/authority-fabric.md`](file:///c:/Users/sprim/projects/sea-rs/docs/subsystems/authority-fabric.md) | Policy resolution, rules, dispositions, candidate verdicts, and ActionGrants | Security engineer / policy author | Explanation / Reference | `architecture.md` | `explanation/why-authority-precedes-execution.md`, `how-to/write-authority-policies.md` |
| [`subsystems/sandbox-runtime.md`](file:///c:/Users/sprim/projects/sea-rs/docs/subsystems/sandbox-runtime.md) | Landlock/Seatbelt jails, process isolation, argv execution, and NetworkPosture | Systems engineer auditing security bounds | Explanation / Reference | `architecture.md`, `subsystems/authority-fabric.md` | `explanation/landlock-jail-security-model.md` |
| [`subsystems/settlement-evidence.md`](file:///c:/Users/sprim/projects/sea-rs/docs/subsystems/settlement-evidence.md) | Settlement criteria evaluation, machine-readable basis, artifact hashing, pre-mint ID | Engineer evaluating task correctness | Explanation / Reference | `architecture.md`, `subsystems/kernel-pipeline.md` | `explanation/settlement-vs-process-exit.md` |
| [`subsystems/integrity-ledger.md`](file:///c:/Users/sprim/projects/sea-rs/docs/subsystems/integrity-ledger.md) | Append-only streams, ULID ordering, hash chains, MMR, and witness receipts | Cryptographic auditor / backend engineer | Explanation / Reference | `architecture.md` | `explanation/append-only-truth-rebuildable-views.md` |
| [`subsystems/capability-memory.md`](file:///c:/Users/sprim/projects/sea-rs/docs/subsystems/capability-memory.md) | Semantic envelopes, capability promotion policies, SQLite FTS memory | Agent developer / memory engineer | Explanation / Reference | `architecture.md`, `subsystems/settlement-evidence.md` | `how-to/rebuild-projections.md` |
| [`subsystems/domainforge-boundary.md`](file:///c:/Users/sprim/projects/sea-rs/docs/subsystems/domainforge-boundary.md) | First-party adapter to `domainforge-core`, `.sea` parsing, AST bounds, candidate verdicts | Domain modeler / language integrator | Explanation / Reference | `architecture.md` | `subsystems/authority-fabric.md` |
| [`subsystems/server-sfwp.md`](file:///c:/Users/sprim/projects/sea-rs/docs/subsystems/server-sfwp.md) | Unix domain socket server, SFWP v1 NDJSON protocol, request admission, event broadcasting | Backend / frontend integration engineer | Explanation / Reference | `architecture.md` | `reference/sfwp-protocol-reference.md`, `subsystems/workbench-desktop.md` |
| [`subsystems/thoth-agent.md`](file:///c:/Users/sprim/projects/sea-rs/docs/subsystems/thoth-agent.md) | ADLC manager loop, Thoth ask service, ACP driver, permission brokering, sealed transcripts | Agent orchestrator / tool developer | Explanation / Reference | `architecture.md`, `subsystems/authority-fabric.md` | `workflows/agent-delegation-flow.md`, `how-to/configure-agent-endpoints.md` |
| [`subsystems/workbench-desktop.md`](file:///c:/Users/sprim/projects/sea-rs/docs/subsystems/workbench-desktop.md) | Standalone Tauri host, React 19 UI, Astryx theme, TanStack Router, G1–G9 guards | Desktop UI developer | Explanation / Reference | `subsystems/server-sfwp.md` | `reference/sfwp-protocol-reference.md`, `tutorials/03-authoring-first-workbench-case.md` |
| [`subsystems/spec-pipeline-ip.md`](file:///c:/Users/sprim/projects/sea-rs/docs/subsystems/spec-pipeline-ip.md) | Spec-to-code pipelines (ADR/PRD/Code/Test), generated zones, and IP transitions | Release engineer / IP custodian | Explanation / Reference | `architecture.md`, `subsystems/kernel-pipeline.md` | `reference/cli-command-reference.md` |

### 4. Workflows & Execution Traces (Layer 4)

| Page | Purpose | Primary Audience Need | Diátaxis | Prerequisites | Related Pages |
|---|---|---|---|---|---|
| [`workflows/cli-run-lifecycle.md`](file:///c:/Users/sprim/projects/sea-rs/docs/workflows/cli-run-lifecycle.md) | Step-by-step trace of `sea-forge run` from intent to capability envelope | Developer tracing single-episode execution | Explanation / Trace | `architecture.md` | `subsystems/kernel-pipeline.md`, `subsystems/authority-fabric.md` |
| [`workflows/case-orchestration-lifecycle.md`](file:///c:/Users/sprim/projects/sea-rs/docs/workflows/case-orchestration-lifecycle.md) | Multi-stage CMMN case commit, sentry transitions, sub-episodes, and completion | Developer tracing case engine workflows | Explanation / Trace | `architecture.md`, `workflows/cli-run-lifecycle.md` | `subsystems/server-sfwp.md` |
| [`workflows/human-approval-cycle.md`](file:///c:/Users/sprim/projects/sea-rs/docs/workflows/human-approval-cycle.md) | Escalation detection, `approvals.jsonl` fold, Workbench inbox, and resume flow | Developer tracing human-in-the-loop flows | Explanation / Trace | `architecture.md`, `subsystems/authority-fabric.md` | `subsystems/workbench-desktop.md` |
| [`workflows/agent-delegation-flow.md`](file:///c:/Users/sprim/projects/sea-rs/docs/workflows/agent-delegation-flow.md) | Agent task dispatch, ACP tool permission interception, sealed transcript capture | Developer tracing autonomous agent flows | Explanation / Trace | `architecture.md`, `subsystems/thoth-agent.md` | `how-to/configure-agent-endpoints.md` |

### 5. Explanation & Design Rationale (Layer 5)

| Page | Purpose | Primary Audience Need | Diátaxis | Prerequisites | Related Pages |
|---|---|---|---|---|---|
| [`explanation/why-authority-precedes-execution.md`](file:///c:/Users/sprim/projects/sea-rs/docs/explanation/why-authority-precedes-execution.md) | Load-bearing rationale for Invariant AUTH-01 (why directories require Allow) | Security auditor / architect | Explanation | `architecture.md` | `subsystems/authority-fabric.md` |
| [`explanation/settlement-vs-process-exit.md`](file:///c:/Users/sprim/projects/sea-rs/docs/explanation/settlement-vs-process-exit.md) | Rationale for Invariant DOM-01 (why exit 0 is merely evidence) | Systems engineer / test architect | Explanation | `architecture.md` | `subsystems/settlement-evidence.md` |
| [`explanation/synchronous-kernel-boundary.md`](file:///c:/Users/sprim/projects/sea-rs/docs/explanation/synchronous-kernel-boundary.md) | Rationale for Invariant BUILD-01 (19 async-free crates, Tokio isolated to edges) | Rust developer / dependency auditor | Explanation | `architecture.md` | `how-to/run-and-verify-gates.md` |
| [`explanation/append-only-truth-rebuildable-views.md`](file:///c:/Users/sprim/projects/sea-rs/docs/explanation/append-only-truth-rebuildable-views.md) | Rationale for Invariant DATA-01 (append-only ledgers vs disposable projections) | Database architect / storage engineer | Explanation | `architecture.md` | `subsystems/integrity-ledger.md` |
| [`explanation/landlock-jail-security-model.md`](file:///c:/Users/sprim/projects/sea-rs/docs/explanation/landlock-jail-security-model.md) | Linux Landlock ABI v1-v6 filesystem and network isolation security analysis | Platform security engineer | Explanation | `architecture.md` | `subsystems/sandbox-runtime.md` |

### 6. Tutorials (Layer 6)

| Page | Purpose | Primary Audience Need | Diátaxis | Prerequisites | Related Pages |
|---|---|---|---|---|---|
| [`tutorials/01-first-governed-run.md`](file:///c:/Users/sprim/projects/sea-rs/docs/tutorials/01-first-governed-run.md) | Step-by-step beginner guide to executing a governed intent via CLI | Newcomer wanting hands-on success | Tutorial | Cloned repository, installed Rust/Devbox | `reference/cli-command-reference.md` |
| [`tutorials/02-inspecting-evidence-and-proofs.md`](file:///c:/Users/sprim/projects/sea-rs/docs/tutorials/02-inspecting-evidence-and-proofs.md) | Verifying cryptographic traces, artifact hashes, and running `just proof` P1–P4b | Engineer wanting to inspect evidence | Tutorial | `tutorials/01-first-governed-run.md` | `subsystems/settlement-evidence.md` |
| [`tutorials/03-authoring-first-workbench-case.md`](file:///c:/Users/sprim/projects/sea-rs/docs/tutorials/03-authoring-first-workbench-case.md) | Guided walk through Workbench desktop: cell readiness, preflight, commit, settlement | Operator wanting GUI journey | Tutorial | Working desktop build | `subsystems/workbench-desktop.md` |

### 7. How-To Guides (Layer 7)

| Page | Purpose | Primary Audience Need | Diátaxis | Prerequisites | Related Pages |
|---|---|---|---|---|---|
| [`how-to/write-authority-policies.md`](file:///c:/Users/sprim/projects/sea-rs/docs/how-to/write-authority-policies.md) | Authoring YAML policy files with rules, wildcards, dispositions, and boundaries | Security officer / operator | How-To | Basic YAML knowledge | `subsystems/authority-fabric.md` |
| [`how-to/configure-agent-endpoints.md`](file:///c:/Users/sprim/projects/sea-rs/docs/how-to/configure-agent-endpoints.md) | Setting up OpenAI, Anthropic, or ACP endpoints with SOPS secrets and least privilege | Agent developer | How-To | API keys | `subsystems/thoth-agent.md` |
| [`how-to/run-and-verify-gates.md`](file:///c:/Users/sprim/projects/sea-rs/docs/how-to/run-and-verify-gates.md) | Running fast checks, crate tests, proofs, and Workbench contract gates | Contributing developer | How-To | Devbox / just installed | `architecture.md` |
| [`how-to/troubleshoot-cell-failures.md`](file:///c:/Users/sprim/projects/sea-rs/docs/how-to/troubleshoot-cell-failures.md) | Diagnosing lock collisions, stale Unix sockets, Landlock denials, and schema drift | Operator / on-call engineer | How-To | Terminal access | `operations/troubleshooting.md` |
| [`how-to/regenerate-contracts-and-schemas.md`](file:///c:/Users/sprim/projects/sea-rs/docs/how-to/regenerate-contracts-and-schemas.md) | Executing the Rust -> JSON Schema -> TS/AJV contract regeneration pipeline | Full-stack developer | How-To | Bun and Cargo installed | `subsystems/server-sfwp.md` |

### 8. Technical Reference (Layer 8)

| Page | Purpose | Primary Audience Need | Diátaxis | Prerequisites | Related Pages |
|---|---|---|---|---|---|
| [`reference/terminology.md`](file:///c:/Users/sprim/projects/sea-rs/docs/reference/terminology.md) | Complete glossary of project domain vocabulary and non-meanings | Reader needing precise terminology definitions | Reference | None | All documentation |
| [`reference/cli-command-reference.md`](file:///c:/Users/sprim/projects/sea-rs/docs/reference/cli-command-reference.md) | Exhaustive reference of all 24 `sea-forge` CLI subcommands, arguments, and exit codes | Script author / terminal user | Reference | None | `tutorials/01-first-governed-run.md` |
| [`reference/sfwp-protocol-reference.md`](file:///c:/Users/sprim/projects/sea-rs/docs/reference/sfwp-protocol-reference.md) | Complete SFWP v1 method catalog, request/response wire types, and error structures | Client developer / protocol implementer | Reference | `subsystems/server-sfwp.md` | `subsystems/workbench-desktop.md` |
| [`reference/configuration-spec.md`](file:///c:/Users/sprim/projects/sea-rs/docs/reference/configuration-spec.md) | Specifications for `server.yaml`, environment variables (`SEA_FORGE_ROOT`), cell layout | Cell administrator | Reference | `docs/CELL_CONTRACT.md` | `subsystems/server-sfwp.md` |
| [`reference/persisted-record-schemas.md`](file:///c:/Users/sprim/projects/sea-rs/docs/reference/persisted-record-schemas.md) | Complete JSON/JSONL field definitions for all 7 persisted governance record kinds | Auditor / tool developer | Reference | `subsystems/kernel-pipeline.md` | `subsystems/settlement-evidence.md` |
| [`operations/troubleshooting.md`](file:///c:/Users/sprim/projects/sea-rs/docs/operations/troubleshooting.md) | Structured symptom-cause-remediation diagnostic matrix | Debugging engineer | Reference | None | `how-to/troubleshoot-cell-failures.md` |
| [`source-map.md`](file:///c:/Users/sprim/projects/sea-rs/docs/source-map.md) | Concept-to-source traceability matrix connecting ideas to code symbols | Codebase explorer / maintainer | Reference | None | All subsystem guides |
