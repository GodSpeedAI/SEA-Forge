# Minimum Governed Kernel Implementation Plan

**Goal:** Implement `.agents/specs/spec-minimum.md` as the synchronous two-crate governed lifecycle, including P1–P4b and the required conformance matrix.

**Architecture:** `sea-forge-core` owns typed records and the deterministic one-shot pipeline; `sea-forge-cli` owns argument parsing, diagnostics, exit codes, validation, recall, and inspection. The pipeline decides every authority request before materializing or executing any operation, then writes append-only trace/evidence, settlement, case, and capability records.

**Tech stack:** Stable Rust 2021; serde JSON/YAML; SHA-256; RFC 3339 UTC timestamps; clap; synchronous `std::process`; no network or async runtime.

## Global Constraints

- Preserve `.agents/specs/spec-minimum.md` as the normative contract and Appendix A ordering.
- Use TDD for every behavior slice; keep each dependency-ordered checkpoint compiling.
- Never invoke a shell, inherit the parent environment wholesale, or write runtime data outside the configured `.sea-forge` root.
- Default deny, evaluate all operations before side effects, and retain complete governed records for deny, escalation, and execution failure.
- Do not add full-spec features, network dependencies, Tokio, a server, UI, dynamic plugins, or an OS jail.

## Task 1: Domain records, IDs, and typed errors

**Files:** `crates/sea-forge-core/src/{lib,ids,types,errors}.rs`, `crates/sea-forge-core/Cargo.toml`, workspace manifests.

**Produces:** All §7 record types with serde round trips; ID constructors/validators; canonical hash input support; typed input/config/planner/IO/execution errors.

- [x] Add focused failing tests for ID grammar, enum JSON names, top-level round trips, attribution, extension/projection slots, and path lexical validation.
- [x] Add only the approved dependencies required by the spec and implement the minimum types/errors.
- [x] Run `cargo test -p sea-forge-core ids types errors`; expect all focused tests to pass.
- [x] Run `cargo fmt --all -- --check && cargo check -p sea-forge-core`.
- [x] Commit the independently compiling type foundation.

## Task 2: Flush-safe trace and evidence writers

**Files:** `crates/sea-forge-core/src/{trace,evidence}.rs`, nearby unit tests.

**Consumes:** `TraceEvent`, `EvidenceRecord`, sequenced ID helpers. **Produces:** `JsonlTraceRecorder::append`, `JsonlEvidenceWriter::append_artifact`, SHA-256 and deterministic pre-mint identity helpers.

- [x] Write failing temp-directory tests proving monotonic IDs, one valid JSON object per flushed line, artifact copies, matching hashes, and stable `model.sea` pre-mint identity.
- [x] Implement append-and-flush writers and safe artifact metadata construction.
- [x] Run the focused trace/evidence tests, then `cargo check -p sea-forge-core`.
- [x] Commit the record writers.

## Task 3: Validated policy and fail-closed authority

**Files:** `crates/sea-forge-core/src/authority.rs`, policy fixtures/tests.

**Produces:** `AuthorityPolicyBundle::load`, canonical bundle hash, request construction, identity resolution, and `PolicyAuthorityEngine::evaluate`.

- [x] Write failing tests for all §8.3 config classes, default deny, allow, escalate, unresolved identity, deterministic decisions, audit shape, generated-zone/secret hard boundaries, unclassified input, and reserved surfaces.
- [x] Implement strict YAML validation/defaults, stable canonical hashing, first-match rules, and hard-boundary evaluation.
- [x] Run `cargo test -p sea-forge-core authority` and `cargo check -p sea-forge-core`.
- [x] Commit the authority fabric.

## Task 4: Deterministic domain planner

**Files:** `crates/sea-forge-core/src/{domain,planner}.rs` and focused tests.

**Produces:** `domain::interpret` and `DeterministicPlanner::plan`, including the exact fixed model bytes and test-only conformance variants scoped by the spec.

- [x] Write failing tests for recognized case-insensitive demo intent, unknown/empty/oversized input, exact operations/criteria, and unsafe constructed paths.
- [x] Implement the fixed pattern table and plan construction without using intent text in paths, content, or argv.
- [x] Run focused planner tests and `cargo check -p sea-forge-core`.
- [x] Commit the planner slice.

## Task 5: Workspace safety and synchronous execution

**Files:** `crates/sea-forge-core/src/{sandbox,runtime}.rs` and focused tests.

**Produces:** `safe_join`, `LocalWorkspaceSandbox`, and `ProcessExecutor::execute` with file-backed stdout/stderr and timeout termination.

- [x] Write failing tests for relative/absolute/traversal/bad-character/symlink paths, workspace-only writes, minimal child environment, nonzero completion, spawn failure, and timeout with no surviving child.
- [x] Implement safe workspace materialization and argv-only synchronous execution with explicit PATH/HOME plus requested env.
- [x] Run focused sandbox/runtime tests and `cargo check -p sea-forge-core`.
- [x] Commit the execution boundary.

## Task 6: Settlement and capability memory

**Files:** `crates/sea-forge-core/src/{settlement,capability}.rs` and focused tests.

**Produces:** `RuleBasedSettlementEvaluator::settle`, append-only envelope storage, and read-only newest-first recall filtering.

- [x] Write failing tests for allow/deny/escalate precedence, false success, spawn failure, timeout, stdout checks, append-per-run, malformed-line tolerance, filters, limits, and no-match behavior.
- [x] Implement the exact §16.2 basis vocabulary and line-oriented capability scan/append.
- [x] Run focused settlement/capability tests and `cargo check -p sea-forge-core`.
- [x] Commit settlement and memory.

## Task 7: Governed pipeline

**Files:** `crates/sea-forge-core/src/pipeline.rs`, core integration tests.

**Produces:** `run_intent(RunOptions) -> RunOutcome`, enforcing preflight-before-run-dir and the §9.1 lifecycle order.

- [x] Write failing integration tests for happy path, all-deny, escalation, false success, nonzero, timeout, repeated-run identity, cross-link resolution, case state, and exact accepted/halted trace order.
- [x] Implement decide-all-then-execute orchestration, write-once JSON records, evidence collection, settlement, case close, envelope append, and typed exit classification.
- [x] Run core pipeline tests and `cargo test -p sea-forge-core`.
- [x] Commit the end-to-end library slice.

## Task 8: CLI commands and diagnostics

**Files:** `crates/sea-forge-cli/src/main.rs`, `crates/sea-forge-cli/src/commands/{mod,validate,run,recall,inspect}.rs`, CLI integration tests.

**Produces:** clap command surface and exit codes 0/1/2/3/4; exact validator output; JSONL recall; six-record inspect output; contextual JSON diagnostics.

- [x] Write failing binary tests for validator valid/invalid cases, run flag defaults/overrides and output, typed preflight failures, recall filters/read-only behavior, inspect, and diagnostics fields.
- [x] Implement command dispatch and map core outcomes/errors to the specified stdout/stderr and exit codes.
- [x] Run `cargo test -p sea-forge-cli` and `cargo check --workspace --all-features --locked`.
- [x] Commit the CLI surface.

## Task 9: Conformance proofs, documentation, and handoff

**Files:** integration tests/fixtures as needed, `README.md`, `.agents/{CURRENT_STATUS,OBSERVED_DEBT,LESSONS,OPEN_QUESTIONS}.md`, `justfile` only if the existing proof recipe needs source-driven completion.

- [x] Add/finish §17.1 and §17.2 integration tests, including kill-9 valid-prefix durability where supported; mark genuinely unsupported platform checks skipped with reason.
- [x] Run P1–P4b against fresh temporary roots and inspect all produced records and hashes.
- [x] Update README with the §15 process-level sandbox limitation and working command examples; resolve the runtime-diagnostics debt entry.
- [x] Run `cargo fmt --all -- --check`, focused clippy/checks, `devbox run -- just context-check`, `devbox run -- just check`, and `devbox run -- just test`.
- [x] Perform the canonical multi-axis code review; fix findings and rerun affected gates.
- [x] Refresh `CURRENT_STATUS.md` with exact verification and remaining limitations, then commit the final conformance/documentation increment.

## Risks and Mitigations

- Process timeout/orphan handling is platform-sensitive: use a narrow synchronous mechanism compatible with Linux/macOS and test actual child death; report any skipped platform variant.
- The spec asks authority to accept hand-built reserved/unknown surfaces beyond planner operations: keep this as an explicit authority input representation without adding executors.
- JSON canonicalization must not depend on map insertion order or YAML formatting: hash typed validated values serialized through recursively sorted JSON.
- Existing diagnostics must remain structured while successful commands keep stdout machine-readable: route diagnostics only to stderr and test both streams.

## Definition of Done

Every checklist item in spec §18 is met; P1–P4b and §17 tests pass; required project gates pass; no unapproved full-spec scope is present; the worktree diff passes correctness, security, compatibility, simplicity, and architecture review.
