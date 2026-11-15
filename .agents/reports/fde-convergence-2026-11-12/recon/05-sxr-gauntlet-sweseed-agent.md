# Recon 05: SXR (RealityTrace), Gauntlet, SWE_SEED, GodSpeed-Agent

Read-only, 2026-11-12. No fetch: "behind" uses local `origin/main` refs (may be stale). OBS = seen in code/test/doc; INF = inferred; DOC = doc claim, not re-verified.

## Summary

| System | Role | Owns | Does not own |
|---|---|---|---|
| RealityTrace (SXR) | Frozen expectation -> observation -> comparison -> evidence; hash-chained ledger | Record graph, residuals, exports | Execution, authority, reward |
| Gauntlet | Constrained execution harness: frozen Reference, blind compare, ladder verify, settle | Reference freeze, run loop, verdicts, promotion gate | Authority (external), containment |
| SWE_SEED | Pre-work contract for AI coding: route, context, proof declared up front, trace | Route cards, proof contracts, trace chain, ProofCompleted | Code writing, model, CI, settlement |
| GodSpeed-Agent | Python developmental layer: settlement history, policy mining, calibration | `state/*.jsonl`; WorkRequested producer (E1) | Runtime authority |

## A) RealityTrace / SXR

### 1. Identity
- `/home/sprime01/projects/sxr`, `GodSpeedAI/RealityTrace`, branch `feat/native-binary-distribution`, HEAD `c239d5c`.
- Dirty: 1 file, `opencode.json` (`$schema` line).
- vs `origin/main`: **8 ahead / 2 behind** (behind: `4d154b1` native packaging #12, likely squash-merged; `b616335` agent-event ingest #13).
- Worktree: `/home/sprime01/projects/gsr-heads/realitytrace` @ `4d05cc0` (detached).

### 2. Role and ownership
Owns the ledger and record kinds (`sxr-core/src/record_kinds.csv`, 28 kinds), CAS (`sxr-ledger/src/cas.rs:1-8`), comparison and residuals. `sxr-core` is declared I/O-free (`sxr-core/src/lib.rs:1-10`).

### 3. Capabilities

| Capability | Evidence | Tests | Status |
|---|---|---|---|
| Repo enrollment, ULID identity via git `rev-parse` | `sxr-git/src/discovery.rs:94`, `identity.rs` | `sxr-git/tests/ac_identity.rs` | IMPLEMENTED |
| DomainForge CLI adapter, pinned 0.21.2 | `sxr-df/src/lib.rs:8`, `adapter.rs:65,135` | `sxr-df/tests/real_binary_tests.rs`, `fixture_tests.rs` | IMPLEMENTED (real-binary tests not run) |
| `semantic.snapshot`, Representation D in CAS, sha256 recompute | `adapter.rs:325,414,565` | `snapshot_and_threshold_tests.rs` | IMPLEMENTED |
| Record graph with allowed predecessors | `record_kinds.csv`, `threshold.rs` | `evidence_record_tests.rs` | IMPLEMENTED |
| cargo-test JSON ingest as observation | `sxr-verify/src/cargo_json.rs:18` | `sxr-cli/tests/command_verifier.rs` | IMPLEMENTED (cargo-test only) |
| Hash-chained append-only SQLite ledger, verify | `sxr-core/src/chain.rs`, `sxr-ledger/src/ledger.rs` | `ac_ledger_001/002.rs`, `ledger_read_only.rs` | IMPLEMENTED |
| Generations, destructive recovery, quarantine | `sxr-ledger/src/generation.rs`, `quarantine.rs` | none seen | IMPLEMENTED (code); tests UNVERIFIED |
| Writer lock, durable queue, outbox | `writer_lock.rs`, `queue.rs`, `outbox.rs` | `sxr-cli/tests/slice*.rs` | IMPLEMENTED |
| ProofCompleted fail-closed ingest (E7) | `sxr-core/src/proof_ingest.rs:1-25` | `convergence_t07_proof_ingestion.rs` | IMPLEMENTED |
| `evidence.recorded`: expected vs observed (E8) | `sxr-core/src/evidence_emit.rs:17,68` | `convergence_t08_evidence_emission.rs` | IMPLEMENTED |
| Residuals (`delta`, `residual`, `G_declared`) | `sxr-cli/src/projections.rs:1731-1754` | not traced | PARTIAL |
| Semantic diff / identity drift records | `record_kinds.csv`; `adapter.rs` | - | PARTIAL |
| Canonical JSON export, anti-collapse corpus, deterministic rebuild | `projections.rs:1567,2703,5`; `main.rs:417` | `slice6.rs` (likely) | IMPLEMENTED |
| Process mining, sequence reconstruction, behavior replay | none (grep) | - | ABSENT |
| OpenTelemetry / spans | none (grep) | - | ABSENT |

### 4. Interfaces
- CLI `realitytrace` (`sxr-cli/src/main.rs:40-108`): repo, semantic, snapshot, question, threshold, run, plan, contract, verify, evidence, claim, settlement, ingest, ledger, query, projections, spool, await, queue, daemon, delta, diagnostic, residual, explain, context, doctor.
- Ingest: `ingest --run <id> --verifier cargo-test --stdin-format cargo-test-json [--extensions JSON]`.
- No HTTP API. Daemon uses a Unix socket.
- Schemas: `schemas/sxr-record-v1.json`, `sxr-plan-v1.json`, `sxr-output-v1.json`, `cep-semantic-envelope.schema.json` (CEP-0008).
- IDs: ULID repo/checkout/run; declaration keys `concept:<uuid>`, `occurrence:<kind>:<hash>:<idx>`, `application:` (`projections.rs:650,761`); `snapshot:`, `checkpoint:`.
- Storage: SQLite (rusqlite bundled); CAS `cas/sha256/<aa>/<bb>/<hash>`. Env `SXR_DOMAINFORGE_BIN`.

### 5. Cross-system
| Peer | Mechanism | Direction | Status |
|---|---|---|---|
| SWE_SEED | ProofCompleted (E7) consumed; mirrored vocabulary, no code dependency | SWE_SEED -> SXR | OBS |
| GSA | `evidence.recorded` (E8) consumed in hop 6 | SXR -> GSA | OBS (`godspeed_agent/tests/test_e2e_world_loop.py:62`) |
| SEA-Forge | Reads SXR ledger read-only to corroborate evidence | SXR -> SEA-Forge | DOC (`sea-rs/docs/how-to/corroborate-evidence-with-realitytrace.md`) |
| DomainForge | CLI binary, snapshot and diff | DF -> SXR | OBS |
| Gauntlet | Named as an executor in README; no code | - | DOC |

E2E: `sea-rs/scripts/e2e-world-loop.sh` hop 5 (`cargo test -p sxr-core --test e2e_world_loop`). Not executed. Gate scripts `sxr/proof/*.sh`. CI evidence `.agents/reports/native-binary-ci-evidence-40875037852.md` (DOC). UX: none.

### 6. Unfinished
- `.agents/current_status.yml`: `in_progress`. ARM64 glibc 2.35 install check and `realitytrace-v*` publish path not confirmed.
- `.agents/CURRENT_STATUS.md` covers an ablation workstream (Evidence Revision 005), not SXR core: stale or cross-branch.
- `ablation/` is an LLM research harness, not core.

### 7. Risks
- Semantic anchoring is by declaration key; `world_ref.rs:12-13` checks syntax only. Digest check is upstream (INF).
- No sequence replay or process mining: any behavior reconstruction is new work.

## B) Gauntlet

### 1. Identity
- `/home/sprime01/projects/gauntlet`, `GodSpeedAI/Gauntlet`, branch `release/native-distribution`, HEAD `5842777`. Clean.
- vs `origin/main`: **12 ahead / 0 behind** (native-distribution packaging fixes).
- Worktree: `/home/sprime01/projects/gsr-heads/gauntlet` @ `1543cbf` (detached).
- DOC: PR #7 `settled_pending_operator_merge` (`.agents/CURRENT_STATUS.yml`).

### 2. Role and ownership
Owns Reference freeze, run loop, blind comparison, verification ladder and VerifiedClaim (`gauntlet-app/src/verify/authority.rs:1-30`), settlement (`src/settle/`), governance gate (`src/govern/`), promotion gate (`improve promote`, 8 conditions, `gauntlet-cli/src/improve_cmd.rs:758`). Authority is external by design (README "Authority stays outside the executor").

### 3. Capabilities

| Capability | Evidence | Tests | Status |
|---|---|---|---|
| Digest-verified frozen Reference | `gauntlet-app/src/verify/authority.rs`, `gauntlet-cli/src/reference_lock.rs` | `p1_reference_integrity.rs` | IMPLEMENTED |
| PASS / FAIL / INCONCLUSIVE, never collapsed | `gauntlet-domain::Outcome`; `verify_cmd.rs:4-20,420` | CLI tests | IMPLEMENTED |
| Ladder: FileDigest(1), SchemaCheck(2), CommandExitCode(3), TestReport(4), ModelJudgment(10); deterministic first | `gauntlet-app/src/verify/ladder.rs:71-100` | in crate | IMPLEMENTED |
| VerificationRecord (harness, probe_argv, observed, harness_version, date) | `adapters/gauntlet-adapter-harness-catalog/src/verification.rs:85-96` | adapter tests | IMPLEMENTED |
| `verify --all` read-only conformance sweep | `verify_cmd.rs:1-30` | `cli_surface.rs` (DOC) | IMPLEMENTED |
| Observation, blind comparison, settlement, budget | CONFORMANCE:84-91; GAUNTLET.md | p-series proofs | IMPLEMENTED (DOC) |
| Anti-gaming G1-G16; mutation 73 examined / 63 caught / 10 unviable / 0 missed | CONFORMANCE:94; `.agents/reports/mutation.json` | `gauntlet-cli/tests/antigaming.rs` (19) | IMPLEMENTED (DOC; G9 advisory) |
| Proofs P1-P12 | CONFORMANCE:93 | 83 integration files; ~2226 test fns | IMPLEMENTED (DOC) |
| Agent fabric (`fabric admit-root`, `serve --caller`) | `gauntlet-cli/src/fabric_cmd.rs:1-12` | `fabric_cli.rs`, `fabric_acceptance.rs` | IMPLEMENTED (OBS header) |
| Adaptive critic fan-out | CONFORMANCE:112 | partial | PARTIAL |
| TUI / dashboard | CONFORMANCE:115 says absent; `workbench/` exists | - | CONTRADICTED |
| Worktree workspaces; observability adapter | CONFORMANCE:116-117 | - | ABSENT |
| Real runner adapters | GAUNTLET.md: deliberately unconfigured (UNK-003) | - | ABSENT by config |

### 4. Interfaces
- CLI `gauntlet`: `run`, `verify [--unit|--run|--all]`, `improve propose|evaluate|promote|rollback|baseline`, `fabric admit-root|serve`, `escalations`, `doctor`, `compare-diff`. Verify exits 0 iff zero FAIL (`verify_cmd.rs:15-20`).
- Config: `GAUNTLET.md` front matter (budget 2M tokens, $25, 15% recovery reserve; state postgres; sandbox default `none`).
- Ports (README:1058-1113): ReferenceProvider, AgentRunner, Observer, Comparator, Verifier, PolicyProvider, StateStore, EvidenceStore, EventLog, Sandbox, BudgetMeter.
- Result contract `role_result@v1` (CONFORMANCE:83).
- "Evidence bundle" and "claim ledger" do not exist by name in Rust (0 hits). Closest: `EvidenceStore`, `CapabilityLedger`.

### 5. Cross-system
- No Rust reference to realitytrace, sxr, swe_seed, sea-forge, domainforge (0 hits in `crates/`). Docs only.
- Callers: operator (CLI, TUI), fabric caller. No SWE_SEED or SXR caller observed.
- Not in the sea-rs world loop. `targets/w9-*.log` are cargo check logs, not analyzed.

### 6. UX
Bun + OpenTUI workbench in `workbench/`. `PRODUCT.md` (terminal-native, "ruler outside executor", "router not authority"). `DESIGN.md` not read.

### 7. Unfinished
- PR #7 merge pending operator. Canonical handoff `.agents/reports/handoff.md` (not read).
- UNK-001 (`docs/CONFORMANCE.md:170`), UNK-002 (LICENSE), UNK-003 (default runner, `docs/CONFIGURATION.md:101`).
- ~20 TODO/FIXME markers in `crates/**/*.rs`. Duplicate `.bak` status files.

### 8. Risks
- CONFORMANCE says TUI absent; `workbench/` exists. Verify before citing either.
- Runners unconfigured: real-agent path and `ModelJudgment` rung not proven in this tree.
- Sandbox default `none`: containment not enforced by default.

## C) SWE_SEED

### 1. Identity
- `/home/sprime01/projects/SWE_SEED`, `GodSpeedAI/swe_seed`, branch `feat/native-distribution`, HEAD `dac8eb7`.
- Dirty: 1 file, `.jolli/jollimemory/debug.log` (noise).
- vs `origin/main`: **2 ahead / 0 behind**: `f215f01` (`swe-seed init`, embedded harness bundle), `dac8eb7` (native candidate workflow, clean-room smoke).
- Worktree: `/home/sprime01/projects/gsr-heads/swe-seed` @ `232e02d` (detached).

### 2. Role and ownership
README:3-40: "lightweight verification harness for AI-assisted software work". Pipeline: work request -> route -> context -> artifact -> proof declared up front -> agent -> trace -> evidence-backed claim. Layers SweSeed > Harness > Fabricator (`SWE_SEED_SPEC_v0.2.0.md`). Owns route cards, proof contracts, traces, trace ledger (`trace_ledger.rs:620,765`), ProofCompleted emission ("exclusive authoritative producer", `federation/proof_completed.rs:1-2`). Accepts WorkRequested only from GSA (`federation/work_ingress.rs:1-6`). Not CI, not settlement (README:1209-1257).

### 3. Capabilities

| Capability | Evidence | Tests | Status |
|---|---|---|---|
| Routing, route cards, context planning, trace lifecycle | README (DOC); `swe-seed-core/src/trace/lifecycle.rs` | `crates/swe-seed-core/tests/*` | IMPLEMENTED (module presence) |
| Hash-chained SQLite trace ledger, signed anchors, verify | `trace_ledger.rs:245,620,765,778` | ledger tests | IMPLEMENTED |
| ProofCompleted: 7 required fields; passed/failed | `federation/proof_completed.rs:39-58` | `convergence_t07_proof_completed.rs` | IMPLEMENTED |
| WorkRequested ingress; placeholder ids refused | `federation/work_ingress.rs` | `convergence_t03_work_ingress.rs` | IMPLEMENTED |
| Governed submission to Context Kernel (E4), context slice (E3) | `federation/governed_submission.rs`, `context_client.rs` | `convergence_t02`, `t04` | IMPLEMENTED (federation off by default, `federation/mod.rs:1-6`) |
| OperationalSettlement adjudication (E6) | `federation/operational_settlement.rs` | `convergence_t06_*` | IMPLEMENTED |
| Ed25519 federation signing | `federation/signing.rs` | `federation_signing.rs`, `federation_parity.rs`, `federation_verify_sea.rs` | IMPLEMENTED |
| Host projection (Claude, Codex, OpenCode, Copilot, Antigravity, CI) | README (DOC); `swe-seed hosts` | - | IMPLEMENTED, narrower guarantees (README:1353) |
| OTel and JUnit export | README (DOC) | not enumerated | IMPLEMENTED (DOC) |
| MCP gateway; federation envelope exchange; SEA-Forge verification | README:1367 | - | EXPERIMENTAL |
| Context Kernel deep integration; GSA settlement handoff | README:1373 | - | SPEC-ONLY |

### 4. Interfaces
- CLI `swe-seed` (`crates/swe-seed/src/cli.rs`, `federation_cli.rs`, `provenance_cli.rs`). Contracts as data: `.agent-harness/baml/baml_src/swe_seed.baml`.
- Events: `WorkRequested`, `ProofCompleted`, `OperationalSettlement`; mirrored from Python `agentic_capability_loop/adapters.py` (`federation/mod.rs:6-8`).
- Traces: JSON under `.agent-harness/traces/` (`seed/capability.rs:118`).

### 5. Cross-system
- GSA -> SWE_SEED: WorkRequested (E1). SWE_SEED -> Context Kernel: E4, E3. SEA-Forge -> SWE_SEED: E6. SWE_SEED -> SXR: E7.
- E2E: `sea-rs/scripts/e2e-world-loop.sh` hops 2 and 4 (`E2E_DIR`, `E2E_WORLD_REF`; `e2e_world_loop.rs:1-10`). Not executed.

### 6. Unfinished and risks
- Experimental and spec-only items above. Native candidate workflow added; no run result seen.
- Dual implementation: Python `agentic_capability_loop/` and Rust `swe-seed-core` (intentional 1:1 port). `tests/test_cross_repo_hash_consistency.py` mitigates drift.
- Federation off by default; cross-system path runs only under the e2e driver.
- Python tree (`pyproject.toml`, `uv.lock`, `node_modules`) coexists with the Rust workspace; README does not explain why.

## D) GodSpeed-Agent

### 1. Identity
- `/home/sprime01/projects/godspeed_agent`, `GodSpeedAI/GodSpeed-Agent`, branch `migration/cep-world-required-evidence`, HEAD `4bfed12`.
- Dirty: 2 fixtures regenerated by a test run (`tests/fixtures/t03_work_requested.json`, `t11_canonical_golden.json`: new event_id, timestamp). Do not commit as-is.
- vs `origin/main`: **33 ahead / 0 behind**. No extra worktrees.

### 2. Role and ownership
Developmental agent (README:1-30): settlement history (`state/milestones.jsonl`), lessons, missing, decisions (`CURRENT_STATE.md`), policy mining, calibration experiments. WorkRequested producer (E1). No runtime authority; v008 "No runtime deployment".

### 3. Capabilities

| Capability | Evidence | Tests | Status |
|---|---|---|---|
| WorkRequested and loop events pin world_ref | commit `baf6005`; `godspeed_nav/runtime.py`, `canonical_events.py` | `tests/test_e2e_world_loop.py:36` | IMPLEMENTED (per commit) |
| world_ref syntax and equality gate; no digest check | `godspeed_nav/world_ref.py:1-10` | `agentic_capability_loop/tests/conftest.py` | IMPLEMENTED |
| Inbound evidence checked against pinned world; refuse EvidenceRecorded without world_ref when pinned | `godspeed_nav/evidence_ingest.py:186-206,331-338`; commit `ca05f3d` | `tests/test_e2e_world_loop.py:62,83` | IMPLEMENTED (the "required evidence" step) |
| Settlement datasets, policy mining, shadow evaluation | `scripts/settlement_datasets/mine_policies.py` | `tests/` | IMPLEMENTED (research) |
| v006-v008 calibration experiments | `CURRENT_STATE.md` Last Settlement; `external-proxy-calibration` | - | EXPERIMENT (DOC) |

### 4. Interfaces
Events `WorkRequested` (E1), `EvidenceRecorded` (E8, consumed). `world:<name>@sha256:<64 hex>` (`world_ref.py`). JSONL under `state/` and `ledgers/`. Source labels `context-kernel`, `sea-forge` (`developmental_memory.py:88-91`).

### 5. Cross-system
- sea-rs world loop hops 1 and 6 (`e2e-world-loop.sh:55,73`). Not executed.
- Context Kernel and SEA-Forge: only as memory source labels.
- Cognate: zero references in this repo; mentioned only in sea-rs (`just sea-forge-live`). Relation UNVERIFIED.

### 6. Migration state (cep-world-required-evidence)
- Stage 9 (`CURRENT_STATE.md`, DOC): work requests and loop events pin world_ref; inbound evidence checked against deployment pin; agent-local settlement labelled. 1,131 tests pass (DOC, not re-run). The doc names branch `migration/cep-world-ref` (merged as PR #1, `83d7d5f`).
- On this branch: `baf6005` (pin), `ca05f3d` (refuse unbound EvidenceRecorded when pinned), `4bfed12` (world-loop hop tests). Code present; PR status not observed.

### 7. Unfinished and risks
- Next move: N4 swe_smith ingestion fix and calibration re-check; SWE-rebench as 4th environment. Known limit: 3-environment evidence.
- `state/missing.jsonl`: two "no known gaps" entries.
- CURRENT_STATE names the wrong branch and test counts differ across stages (1,131 vs 1,099).
- `world_ref` logic duplicated in GSA and SXR; both syntax-only.
- Terminology collision: GSA "settlement" (developmental milestone) vs SXR `settlement.registered` (runtime record).

## Cross-cutting

1. Only SXR has a hash-chained declared-vs-observed ledger (Question and Threshold before Observation). Gauntlet has blind comparison and a three-state verdict but no sequence ledger. SWE_SEED has declared proof contracts and a trace chain. GSA holds developmental milestones.
2. No code link between Gauntlet and SXR or SWE_SEED. Convergence needs a new adapter.
3. Real chain (driver-run, not executed): GSA hop 1 -> Context Kernel -> SWE_SEED E4 -> SEA-Forge E5B (simulated) -> SWE_SEED E7 -> SXR E8 -> GSA hop 6. No recorded run output found in these four repos.
4. Reproduction primitives present: SXR `ledger verify`, AC-REPLAY-001 sha256 recompute, canonical JSON export; Gauntlet reference digest check and read-side `verify`; SWE_SEED `verify_chain` and cross-repo hash test. None executed.
5. Missing for FDE: process mining or sequence reconstruction (SXR), OTel spans (SXR), claim ledger and evidence bundle (Gauntlet), one shared verdict schema.
6. Mirrored contracts (ProofCompleted, world_ref) across SXR, SWE_SEED, GSA with no shared package: main drift risk.
7. Stale status docs: SXR `.agents/CURRENT_STATUS.md` (ablation); Gauntlet CONFORMANCE (TUI "absent" but `workbench/` exists); GSA CURRENT_STATE (wrong branch).

## Unknowns
- Fetch (operator decision) to confirm behind counts and PR #12 / PR #7 merge state.
- SXR `slice5.rs`, `slice6.rs`: whether residuals are tested.
- Gauntlet `.agents/reports/handoff.md`, `conformance.json`.
- GSA: Cognate relationship; test count at HEAD.
