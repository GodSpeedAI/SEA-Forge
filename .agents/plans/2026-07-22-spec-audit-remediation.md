# Implementation Plan — Four-Spec Audit Remediation

**Created:** 2026-07-22
**Source of truth:** `.agents/reports/2026-07-22-spec-implementation-audit-independent-validation.md`, then `.agents/specs/spec-minimum.md`, `.agents/specs/spec-full.md`, `.agents/specs/spec-adlc-thoth-minimum.md`, and `.agents/specs/spec-agent-orchestration.md` in their declared prerequisite order.
**Originating context:** Independent executable audit of the four active specifications found minimum-contract drift, missing full-system enforcement, incomplete M9-M11 production paths, and M13-M16 security and settlement defects. The prior `devbox run -- just test` SIGSEGV was stale build output and was resolved by the operator with `cargo clean`; it is not implementation scope.
**Status of the work today:** Focused suites cover substantial behavior, but they omit or encode the audited divergences. This plan closes every confirmed recommendation and material omission without treating documentation status, test names, or fixture-only behavior as proof.

---

## 0. How to use this plan (agent operating instructions)

- Execute tasks in order unless the dependency graph below explicitly permits parallel work. Tasks 1-3 may proceed independently after Task 0. Tasks 5-7 may proceed independently after Task 0, but Task 8 requires Tasks 6-7. Tasks 9, 10A, and 10B are sequential. Tasks 11, 12, 13, 13B, 14A, and 14B are sequential. Tasks 15-18 require the M11 substrate from Task 14B; Task 16 also requires Task 2, and Task 17 requires Tasks 15-16.
- **Every task ends with a verification gate.** Do not mark a task done until its gate command exits 0.
- **One authority fabric, one ledgered source of truth, one settled outcome: compatibility views and adapters never bypass, duplicate, or weaken that path.**
- Match surrounding code style; mirror `crates/sea-forge-sandbox/src/lib.rs:145-190` for safe paths, `crates/sea-forge-cli/src/commands/mediated.rs:130-186` for mediated ingress, `crates/sea-forge-ledger/src/lib.rs` for commits/views, and existing `conformance_m*.rs` suites for proof shape.
- Specifications define required behavior. The independent audit locates gaps; it does not replace the specifications.
- Use RED → GREEN → REFACTOR for every behavior change. Keep each task in a separate reviewable commit unless an adjacent task is mechanically inseparable.
- Stop for owner approval before adding/upgrading dependencies; changing persisted schemas, policy fields, public interfaces, or exit codes; or editing CI/deployment configuration.

### Dependency graph

```text
Task 0 baseline + approvals
  ├─ Task 1 cell import path safety
  ├─ Task 2 canonical transcript/redaction primitive
  ├─ Task 3 jail network isolation
  ├─ Task 4 minimum command/lifecycle compatibility
  ├─ Task 5 DomainForge source-set boundary
  ├─ Task 6 memory authority scope
  ├─ Task 7 SQLite FTS + stale detection
  │    └─ Task 8 recall compatibility + governed evidence (also Task 6)
  └─ Task 9 spec-stage prerequisites
       └─ Task 10A ordinary governed M5 case path (also Tasks 5 and 8)
            └─ Task 10B M5 CLI/project/projection integration
                 └─ Task 11 ledgered self-model + real realization inputs
                 └─ Task 12 ODI provenance production wiring
                      └─ Task 13 Thoth query/disclosure semantics
                           └─ Task 13B Thoth authorship SoD
                                └─ Task 14A real Thoth joins + mediated service
                                     └─ Task 14B CLI/server ask adapters
                                ├─ Task 15 delegation settlement/schema fidelity
Task 2 ─────────────────────────└─ Task 16 transcript retention/storage
Tasks 14B-16 ───────────────────── Task 17 endpoint/topology/manager grants
Tasks 11,15 ────────────────────── Task 18 SWE_SEED reconciliation
Tasks 1-18 (including lettered tasks) ─ Task 19 final conformance and handoff
```

### Global verification gates (must stay green after EVERY task)

```bash
cargo fmt --all -- --check
devbox run -- just check
devbox run -- just test
devbox run -- just proof
devbox run -- just no-async-kernel
```

If `just test` reports the previously observed pre-test SIGSEGV, run `cargo clean`
once and rerun the unchanged command. If it fails again, stop and diagnose it as
a new blocker; never replace the workspace gate with focused passes.

---

## Key facts already discovered (do not re-derive)

| Thing | Location |
|---|---|
| Canonical workspace-relative safe join already rejects traversal and symlink-parent escape | `crates/sea-forge-sandbox/src/lib.rs:145-190` |
| Cell import joins manifest-controlled IDs and paths before validation | `crates/sea-forge-cell/src/bundle.rs:219-248` |
| Jail configures Landlock filesystem rights only, using ABI V1 | `crates/sea-forge-sandbox/src/jail.rs:260-295` |
| Standalone validator currently requires mediated policy | `crates/sea-forge-cli/src/commands/validate.rs:3-16` |
| Demo planner injects root and policy into validator argv | `crates/sea-forge-planner/src/lib.rs:95-108` |
| Minimum pipeline writes `plan.json` twice | `crates/sea-forge-cli/src/pipeline.rs:300-301,350-359` |
| DomainForge validates and parses only the entry source | `crates/sea-forge-domainforge/src/lib.rs:100-163` |
| Memory projection is JSON and has no freshness commitment | `crates/sea-forge-capability/src/memory.rs:196-277` |
| Memory scope helper has no production caller | `crates/sea-forge-capability/src/memory.rs:179-194` |
| Generic reserved authority matching ignores `memory_scope` | `crates/sea-forge-authority/src/lib.rs:2480-2505` |
| Recall authority request omits requested entity/process and result IDs | `crates/sea-forge-cli/src/commands/recall.rs:89-127` |
| Compatibility recall mutates governance state and augments envelopes | `crates/sea-forge-cli/src/commands/recall.rs:37-84` |
| Spec-pipeline validation checks ordering only | `crates/sea-forge-spec-pipeline/src/lib.rs:73-95,268-280` |
| `process_pipeline` has no production caller | `crates/sea-forge-spec-pipeline/src/lib.rs:268-280` |
| Self-model snapshots/projections are direct files with empty governance refs | `crates/sea-forge-self-model/src/store.rs:162-253`, `crates/sea-forge-self-model/src/projections.rs:71-91` |
| ODI built-in contains placeholder provenance | `crates/sea-forge-planner/src/templates.rs:910-929` |
| Resolver-aware criteria verification exists but production uses the no-model wrapper | `crates/sea-forge-planner/src/criteria.rs:263-292`, `crates/sea-forge-cli/src/pipeline.rs:287-300` |
| CLI Thoth view returns no capability or environment state | `crates/sea-forge-cli/src/commands/ask.rs:48-75` |
| Thoth disclosure can emit a stronger class than the permitted class | `crates/sea-forge-thoth/src/engine.rs:174-198,261-289` |
| Thoth SoD helper is unit-test-only and claims omit authorship | `crates/sea-forge-thoth/src/engine.rs:4-21,185-198,277-289` |
| ACP transcript reaches hashing/storage without redaction | `crates/sea-forge-server/src/delegation.rs:488-513,1133-1182` |
| HTTP redaction handles only exact credential and literal Bearer prefix | `crates/sea-forge-agent/src/delegation.rs:89-100` |
| Case dispatch drops response schema/retention and replaces settlement basis | `crates/sea-forge-server/src/case_dispatch.rs:437-477` |
| Delegation always rejects turn-cap termination | `crates/sea-forge-server/src/delegation.rs:595-629` |
| Generated topology and manager endpoint refs violate endpoint ID grammar | `crates/sea-forge-planner/src/templates.rs:938-947`, `crates/sea-forge-cli/src/commands/manager.rs:258-269`, `crates/sea-forge-agent/src/config.rs:124-138` |
| Manager authorization does not bind the iteration cap | `crates/sea-forge-cli/src/commands/manager.rs:47-78` |
| SWE_SEED declarations are sampled only during initial settlement | `crates/sea-forge-server/src/delegation.rs:569-592,666-676` |

---

## Task 0 — Freeze the clean baseline and approve contract choices  (foundation · blocking)

**Goal:** The repaired clean build is reproducible, and every dependency/public-contract decision needed by later tasks has explicit owner approval before code changes.

**Why this shape:** The plan requires SQLite FTS, enforceable network isolation, and sealed transcript retention. Repository rules prohibit guessing dependencies or public schema changes.

### Steps

1. Run the global gates on the current tree after the operator's `cargo clean`; record exact counts and platform skips in `.agents/CURRENT_STATUS.md`.
2. Inspect official documentation and propose the exact `rusqlite` version/features for `sea-forge-capability`, including bundled/system SQLite choice, license, and supply-chain impact.
3. Inspect the installed Landlock crate/kernel support and propose either an exact Landlock upgrade/features or another enforceable Linux egress-denial mechanism; specify the network-grant projection and state macOS Seatbelt behavior separately.
4. Specify the selected sealed summarized-transcript format, key source/lifecycle, crypto dependency/version/features, public-vs-sealed storage paths, and crypto-shredding semantics required by `.agents/specs/spec-agent-orchestration.md:15-20,186-198`.
5. Inventory additive policy/config/public-type changes: DomainForge limits/version, pipeline predecessor evidence, memory grant scope, transcript retention, response-schema evidence, topology endpoint selection, manager grant cap, Thoth ask server request, and self-model ledger records.
6. Record the exact approved dependency choices in `docs/decisions/ADR-002-audit-remediation-dependencies.md` and approved public/persisted contract deltas in `docs/decisions/ADR-003-audit-remediation-contracts.md`. Record unresolved choices in `.agents/OPEN_QUESTIONS.md`; remove them only after the owner decision is captured in the ADR. Link both ADRs and baseline results from `.agents/CURRENT_STATUS.md`.

### Gate

```bash
devbox run -- just context-check && devbox run -- just check && devbox run -- just test && devbox run -- just proof && devbox run -- just no-async-kernel
```

**Done when:** The clean baseline exits 0, both ADRs identify exact approved versions/contracts, `CURRENT_STATUS.md` links them, and any unresolved item remains in `OPEN_QUESTIONS.md` and blocks its dependent task.

**Redesign trigger:** If no available backend can enforce default network denial or no approved key source can seal summarized transcripts, stop and revise the applicable spec/assurance claim with owner approval; do not ship a weaker implicit fallback.

---

## Task 1 — Make cell import paths fail closed  (M6 security · critical)

**Goal:** A bundle cannot write, remove, stage, or rename outside `<root>/.sea-forge/imported/`, and adversarial tests prove zero escaped side effects.

**Why this shape:** Hash-valid archive content is still untrusted path input. Validate every manifest-controlled path before the first filesystem mutation and reuse the canonical safe-join implementation.

### Steps

1. Add RED tests for absolute paths, `..`, malicious `bundle_id`, malicious `cell_id`, duplicate/ambiguous normalized paths, and symlink-parent escape in `crates/sea-forge-cell/tests/conformance_m6.rs`.
2. Reuse `sea_forge_sandbox::safe_join` for staging, final cell directory, and each manifest file in `crates/sea-forge-cell/src/bundle.rs`; validate the complete manifest before `remove_dir_all`, `create_dir_all`, `File::create`, or `rename`.
3. Add the internal workspace dependency in `crates/sea-forge-cell/Cargo.toml` if needed; do not create a second safe-path algorithm.
4. Assert rejection leaves the import root and an outside sentinel byte-identical.

### Gate

```bash
cargo test -p sea-forge-cell --test conformance_m6 traversal -- --nocapture && cargo test -p sea-forge-cell --test conformance_m6
```

**Done when:** Every hostile path returns `bundle_integrity_error`, no outside path changes, and replacing safe-join with raw `join` makes at least one test fail.

**Redesign trigger:** If `safe_join` cannot validate a not-yet-created staging subtree, add a canonical lexical-validation entry point beside it in `sea-forge-sandbox`; do not duplicate validation in `sea-forge-cell`.

---

## Task 2 — Canonicalize and redact transcripts at one choke point  (M13/M16 security · critical)

**Goal:** HTTP and ACP transcripts use the same sentinel-aware redaction and the exact same canonical bytes for summary input, hashing, storage, and verification.

**Why this shape:** Redaction after hashing or storage is ineffective. One producer prevents HTTP/ACP divergence and eliminates a second serialization path.

### Steps

1. Add RED tests in `crates/sea-forge-agent/src/delegation.rs` for credentials, private-key/password/token sentinels, unrelated secrets, UTF-8, field ordering, and streaming chunk-boundary reconstruction.
2. Expose one small API in `crates/sea-forge-agent/src/delegation.rs` that accepts transcript entries plus known secrets, returns redacted entries and canonical JSONL bytes, and computes the digest from those bytes.
3. Use the repository's canonical JSON rules rather than assuming struct field declaration order is JCS-equivalent.
4. Route HTTP production through this API and route ACP conversion in `crates/sea-forge-server/src/delegation.rs:1133-1182` through the same API before summary or artifact creation.
5. Make artifact storage write the canonical bytes returned by the producer; delete duplicate serialization in `crates/sea-forge-server/src/delegation.rs:504-513`.

### Gate

```bash
cargo test -p sea-forge-agent transcript -- --nocapture && cargo test -p sea-forge-server --test conformance_m16 redaction -- --nocapture
```

**Done when:** Sentinel values are absent from transcript entries, summaries, hashes' source bytes, artifacts, evidence, and errors; changing redaction order to after hashing fails the tests.

**Redesign trigger:** If current ledger sentinel rules cannot be reused without introducing an inverse dependency, keep the shared redactor in `sea-forge-agent` and test it against the ledger sentinel corpus; do not move HTTP/async dependencies into a kernel crate.

---

## Task 3 — Enforce default network denial in jail backends  (M1 isolation · critical)

**Goal:** A jail-class run cannot open ungranted outbound or listening sockets, while an explicitly granted network boundary permits only its declared scope.

**Why this shape:** Filesystem Landlock is not network isolation. Backend setup must fail closed when the host cannot enforce the granted sandbox class.

### Steps

1. Add instrumented TCP/UDP RED tests to `crates/sea-forge-sandbox/tests/conformance_m1.rs` for outbound connect and bind/listen under default denial, explicit grant, and unsupported-kernel behavior; avoid external Internet dependencies by using loopback fixtures.
2. Extend `SandboxSpec`/grant projection only as approved in Task 0 so network posture is derived from authority, never child input.
3. Implement the approved Linux egress restriction in `crates/sea-forge-sandbox/src/jail.rs`; negotiate the required ABI instead of pinning `ABI::V1`.
4. Return `jail_unavailable` before child spawn when the requested/default posture cannot be enforced; never fall back to `local`.
5. Add or preserve a separately reported macOS test/skip for Seatbelt behavior; never count an unsupported platform as passing.

### Gate

```bash
cargo test -p sea-forge-sandbox --test conformance_m1 network -- --nocapture && cargo test -p sea-forge-sandbox --test conformance_m1
```

**Done when:** Default-denied connect and bind/listen attempts fail, fixture servers receive zero connections, explicit grants behave within their boundary, and disabling network rules makes the denial tests fail.

**Redesign trigger:** If Landlock cannot express the required destination semantics, enforce no-network jail now and defer scoped egress to an approved proxy/backend; do not claim per-destination enforcement from port-only rules.

---

## Task 4 — Restore minimum validator and lifecycle compatibility  (minimum kernel · high)

**Goal:** The hidden validator is config-free, the demo plan uses exact argv, and `plan.json` is written exactly once after `run_started`.

**Why this shape:** The three defects share one self-hosted validator contract. Fixing only argv or only mediation leaves the demo pipeline inconsistent.

### Steps

1. Add RED assertions in `crates/sea-forge-cli/tests/lifecycle.rs` that `sea-forge validate <file>` creates no `.sea-forge` state and that persisted demo argv is exactly `[current_exe, "validate", "model.sea"]`.
2. Remove root/policy mediation from `crates/sea-forge-cli/src/commands/validate.rs`; keep deterministic shape validation and exact output/exit behavior.
3. Remove injected root/policy tokens from `crates/sea-forge-planner/src/lib.rs:95-108`.
4. Remove the first `plan.json` write at `crates/sea-forge-cli/src/pipeline.rs:300-301`; retain one write after `case_created`/`run_started` and before `plan_created`.
5. Write `plan.json` through the existing create-new/write-once helper so a second write is an executable error. The accepted lifecycle test plus trace-order assertion then proves the file was created once at the declared position without adding test-only instrumentation.

### Gate

```bash
cargo test -p sea-forge-cli --test lifecycle validator -- --nocapture && cargo test -p sea-forge-cli --test lifecycle intent_to_settlement_produces_complete_accepted_run -- --exact
```

**Done when:** Validation succeeds without policy/state, exact argv is persisted, one plan write follows `run_started`, and reintroducing policy arguments or the early write fails a focused test.

**Redesign trigger:** None plausible; the minimum and full specs explicitly preserve the self-hosted validator compatibility contract.

---

## Task 5 — Complete the DomainForge source-set trust boundary  (M0 prerequisite · high)

**Goal:** `load_validate` verifies every source URI/hash, resolves imports from the supplied set, enforces finite limits, and rejects unsupported versions before returning a DomainModelRef.

**Why this shape:** M9 self-model composition and M10 desired-outcome resolution cannot be trustworthy over an entry-only parser path.

### Steps

1. Add multi-file fixtures and RED tests in `crates/sea-forge-domainforge/tests/conformance_m0_domainforge.rs` for resolved import, unresolved import, non-entry hash drift, duplicate URI, absolute/traversal URI, unsupported version, and each configured resource limit.
2. Add the approved finite limits and version expectation to the adapter/source-set contract in `crates/sea-forge-domainforge/src/lib.rs`; validate all metadata and bytes before parsing.
3. Use the pinned DomainForge namespace-aware library API to resolve imports from `SeaSourceSet.files`; do not invoke a CLI or write temporary files.
4. Build `source_refs` only from verified sources and include parser options/version/limits in the stable model identity as required by the spec.
5. Preserve side-effect-free failure and add a test that no source/version error reaches planning or authority.

### Gate

```bash
cargo test -p sea-forge-domainforge --test conformance_m0_domainforge -- --nocapture && cargo test -p sea-forge-domainforge --test projection
```

**Done when:** A real multi-file model passes, every malformed/drifted/oversized variant fails before side effects, and skipping non-entry hash verification breaks a test.

**Redesign trigger:** If pinned `domainforge-core` cannot resolve an in-memory source set, stop and request approval for the smallest supported adapter/version change; do not stage files or call the DomainForge CLI as a hidden side effect.

---

## Task 6 — Bind memory scope into authority grants  (M4b authority · high)

**Goal:** Recall authorization binds requester, target entity/process, kinds, and limit, and the returned set cannot exceed the granted `memory_scope`.

**Why this shape:** Filtering after a generic allow decision is not authority enforcement. The exact requested scope must be part of the canonical action and decision.

### Steps

1. Add RED authority tests in `crates/sea-forge-authority/tests/conformance_m0_authority.rs` for `own`, `entity:<id>`, `any`, missing scope, cross-entity denial, and request/decision hash changes when target scope changes.
2. Implement approved `recall_memory` parameter matching in `crates/sea-forge-authority/src/lib.rs`; default deny when scope is absent or insufficient.
3. Move or expose the scope predicate so authority/grant enforcement owns it; remove the production-orphan helper from `crates/sea-forge-capability/src/memory.rs` if no longer needed there.
4. Return a typed grant boundary that the recall executor can apply without rereading policy.

### Gate

```bash
cargo test -p sea-forge-authority --test conformance_m0_authority memory_scope -- --nocapture
```

**Done when:** Cross-entity recall is denied before data access without an explicit grant, authorized scopes intersect exactly, and removing target entity from action parameters fails hash/boundary tests.

**Redesign trigger:** If the existing generic `PolicyRule` cannot represent exact memory scope without ambiguous optional fields, add one approved typed rule payload rather than encoding scope in strings at the executor.

---

## Task 7 — Replace JSON memory index with SQLite FTS and freshness proof  (M4b storage · high)

**Goal:** `memory/index.sqlite` is a rebuildable FTS projection of `items.jsonl`, and missing, corrupt, or stale indexes return exactly the linear-scan result.

**Why this shape:** The spec selects SQLite FTS explicitly. Freshness must be proven by a source commitment, not inferred from successful deserialization.

### Steps

1. After Task 0 approval, add the exact `rusqlite` dependency to workspace and `crates/sea-forge-capability/Cargo.toml`; keep it confined to the memory projection.
2. Add RED tests in `crates/sea-forge-capability/tests/conformance_m4b.rs` for rebuild, FTS query equivalence, appended-item staleness, truncated/corrupt DB, missing DB, deterministic ordering, and source-digest mismatch.
3. Replace `MemoryIndex` JSON serialization in `crates/sea-forge-capability/src/memory.rs` with a minimal schema containing FTS rows plus source hash/count or ledger ordinal.
4. Rebuild atomically to a temporary database and rename only after integrity checks; keep `items.jsonl` authoritative.
5. Update `crates/sea-forge-cli/src/commands/memory.rs` to use `memory/index.sqlite`; do not silently retain `index.json` compatibility unless persisted user data demonstrates a migration need.

### Gate

```bash
cargo test -p sea-forge-capability --test conformance_m4b index -- --nocapture && cargo test -p sea-forge-capability --test conformance_m4b
```

**Done when:** Indexed and fallback results are identical for present/missing/corrupt/stale cases, appending an item forces fallback, and removing the source commitment makes the stale-index test fail.

**Redesign trigger:** If approved SQLite features cannot provide FTS on a supported platform, stop and obtain a spec/portability decision; do not substitute another JSON index again.

---

## Task 8 — Separate compatibility recall from governed memory recall  (minimum + M4b · high)

**Goal:** Capability compatibility recall prints stored envelopes unchanged and read-only, while memory recall is authority-scoped and emits evidence naming every returned memory item.

**Why this shape:** One command surface has two declared contracts. Explicit modes preserve minimum compatibility without weakening full-spec memory governance.

### Steps

1. Add filesystem-snapshot RED tests in `crates/sea-forge-cli/tests/lifecycle.rs` proving compatibility capability recall changes no file and output JSON equals matched `capabilities.jsonl` envelopes byte-for-structure without injected `assurance`.
2. Add governed memory recall tests in a focused CLI M4b suite for own/cross-entity/any scopes, exact action parameters, returned IDs in evidence, limit, and SQLite/fallback equivalence.
3. Refactor `crates/sea-forge-cli/src/commands/recall.rs`: the legacy capability path performs the required read-only scan; the `--kind` memory path constructs the exact scoped authority action from Task 6 and queries Task 7's projection.
4. Commit a recall evidence record containing request scope and returned memory IDs before exposing results; ensure denial reads no memory data.
5. Expose assurance through the governed response/evidence path, not by mutating compatibility envelope JSON.

### Gate

```bash
cargo test -p sea-forge-cli --test lifecycle recall -- --nocapture && cargo test -p sea-forge-cli --test conformance_m4b -- --nocapture
```

**Done when:** Compatibility recall is byte-for-structure read-only, governed recall leaves a resolvable evidence chain, cross-entity denial reads nothing, and deleting result IDs from evidence fails a test.

**Redesign trigger:** If one CLI invocation cannot distinguish the contracts without ambiguity, add an approved explicit submode while preserving the minimum argv; do not infer governance mode from filesystem contents.

---

## Task 9 — Validate complete spec-pipeline prerequisites  (M5 library · high)

**Goal:** A stage advances only when every required predecessor is present or externally supplied with verified status, hash, schema, DomainModelRef, and pinned DomainForge version.

**Why this shape:** Enum ordering is not a proof chain. Validation must consume persisted predecessor evidence and reject downstream use of quarantined output.

### Steps

1. Add RED cases in `crates/sea-forge-spec-pipeline/tests/conformance_m5.rs` for missing, skipped, rejected, quarantined, unhashed, schema-mismatched, model-mismatched, and version-mismatched predecessors.
2. Extend the stage/input contract only as approved in Task 0 so later-start inputs name immutable predecessor records and validation evidence.
3. Replace order-only validation in `crates/sea-forge-spec-pipeline/src/lib.rs:73-95` with canonical-chain validation; retain a small order helper if useful.
4. Prevent `compute_proof_classification` from counting a stage whose prerequisite chain has not passed.
5. Remove or reverse the unit fixture at `crates/sea-forge-spec-pipeline/src/lib.rs:357-365` that currently grants focused-slice classification without earlier stages.

### Gate

```bash
cargo test -p sea-forge-spec-pipeline --test conformance_m5 predecessor -- --nocapture && cargo test -p sea-forge-spec-pipeline
```

**Done when:** Every invalid predecessor variant fails before downstream processing, a valid later-start chain passes, and changing one predecessor digest breaks the gate.

**Redesign trigger:** If historical stage records lack enough immutable metadata, add an explicit legacy-unverified result that cannot advance proof classification; do not fabricate predecessor evidence.

---

## Task 10A — Express M5 stages as ordinary governed case episodes  (M5 runner · high)

**Goal:** M5 stages execute as ordinary CasePlan episodes whose authority, evidence, quarantine, and settlement records are owned by the shared case runner.

**Why this shape:** The spec forbids a separate pipeline scheduler. Establish the runner contract before adding CLI/project and projection concerns.

### Steps

1. Add RED runner tests in `crates/sea-forge-case-runner/tests/conformance_m5.rs` for an accepted stage, a denied generated-zone edit, a quarantined invalid stage, and a downstream sentry blocked by invalid predecessor output.
2. Add the smallest approved typed stage operation/metadata to `crates/sea-forge-core/src/types.rs` only if existing operations cannot carry Task 9's immutable predecessor refs; cover old-reader behavior.
3. Build the canonical stage CasePlan in `crates/sea-forge-planner/src/lib.rs` using existing sentries and settlement criteria.
4. Add synchronous stage execution to `crates/sea-forge-case-runner/src/lib.rs`, reusing normal authority, trace, evidence, settlement, and case-event services. Call Task 9's validator before activation.
5. Keep `crates/sea-forge-spec-pipeline/src/lib.rs` pure: it validates records and proof classification but owns no scheduling, filesystem writes, or ledger commits.

### Gate

```bash
cargo test -p sea-forge-case-runner --test conformance_m5 -- --nocapture && cargo test -p sea-forge-spec-pipeline --test conformance_m5
```

**Done when:** Stage episodes produce the ordinary governed record chain, rejected/quarantined output cannot activate successors, and bypassing the runner authority call fails the denial test.

**Redesign trigger:** If existing plan item vocabulary cannot carry a required pure stage, use the Task 0-approved typed addition; do not create a second scheduler or truth store.

---

## Task 10B — Wire CLI project and DomainForge projections through M5 cases  (M5 integration · high)

**Goal:** A real CLI project invocation runs the Task 10A case path and produces independently settled CALM/RDF ProjectionRecords from one validated DomainModelRef.

**Why this shape:** CLI ingress and projection fan-out consume the runner contract; they should not be designed inside it.

### Steps

1. Add a RED end-to-end `crates/sea-forge-cli/tests/conformance_m5.rs` that runs a small ADR→PRD→SDS→SEA→AST/IR→manifest→generated-contract chain and verifies ledger links, classification, quarantine, and no DomainForge-owned side effects.
2. Create `crates/sea-forge-cli/src/commands/project.rs` as a thin adapter over Task 10A's case-runner API; register it in `crates/sea-forge-cli/src/commands/mod.rs` and add/dispatch the approved clap variant in `crates/sea-forge-cli/src/main.rs`.
3. Use Task 5's validated DomainModelRef and the in-memory projection API in `crates/sea-forge-domainforge/src/lib.rs`; SEA Forge owns all authorized output writes.
4. Commit separate validated ProjectionRecords and settlements for CALM and RDF even though they share one model.
5. Add a negative CLI proof that a projection validation failure quarantines that target without accepting or silently dropping it.

### Gate

```bash
cargo test -p sea-forge-cli --test conformance_m5 -- --nocapture && cargo test -p sea-forge-domainforge --test projection -- --nocapture
```

**Done when:** The CLI reconstructs every M5 stage from ledgered case records, projections share the exact validated model but settle independently, and direct DomainForge filesystem activity fails the test.

**Redesign trigger:** If projection execution needs state absent from the runner result, extend the synchronous runner result with immutable refs; do not let CLI reread mutable intermediate files as truth.

---

## Task 11 — Ledger self-model rebuilds from real installation state  (M9 · high)

**Goal:** Self-model snapshots and projections derive from verified registry/environment/probe/capability inputs and are ledgered, evidenced, settled, and materialized as rebuildable views.

**Why this shape:** M10/M11 consume self-model truth. Direct accepted files with empty governance refs cannot support disclosure or provenance.

### Steps

1. Add RED M9 tests in `crates/sea-forge-self-model/tests/conformance_m9.rs` and `crates/sea-forge-cli/tests/self_model_cli.rs` requiring ledger entries/envelopes, non-empty authority/evidence/settlement refs, stale mutation handling, and byte-identical rebuild.
2. Replace caller-supplied empty/placeholder inputs in `crates/sea-forge-cli/src/commands/self_model.rs` with reads from verified extension registry, environments/probes, sandbox availability, and capability projection records.
3. Refactor `crates/sea-forge-self-model/src/store.rs` so pure assembly remains library code and one governed service commits validation evidence, projection records, settlement, snapshot envelope, then materializes views.
4. Populate `ProjectionRecord` governance refs in `crates/sea-forge-self-model/src/projections.rs`; a projection cannot be `Accepted` without validation evidence and settlement.
5. Preserve immutable snapshots and failure atomicity; a failed rebuild leaves the prior snapshot verifiable and stale, never silently current.

### Gate

```bash
cargo test -p sea-forge-self-model --test conformance_m9 -- --nocapture && cargo test -p sea-forge-cli --test self_model_cli -- --nocapture
```

**Done when:** Every current snapshot resolves to ledgered source/authority/evidence/settlement records, real installation changes alter the snapshot, and clearing governance refs fails T9.1/T9.3.

**Redesign trigger:** If capability projection records cannot be consumed without a dependency cycle, pass a verified ledger-owned projection interface into self-model assembly; do not let CLI fabricate a digest.

---

## Task 12 — Resolve ODI desired-outcome provenance before authority  (M10 · high)

**Goal:** Instantiated ODI plans carry real desired-outcome concept/model hashes, and unresolvable or drifted provenance fails before authority or side effects.

**Why this shape:** Placeholder template metadata and isolated resolver tests do not prove production plan commitment.

### Steps

1. Add RED production-plan tests to `crates/sea-forge-planner/tests/conformance_m10.rs` and `crates/sea-forge-cli/tests/conformance_m10.rs` for valid, missing, wrong-class, unknown-concept, and model-drift desired outcomes.
2. Replace `sha256:placeholder` and fixed `outcome:primary` in `crates/sea-forge-planner/src/templates.rs` with validated template parameters that remain forbidden from privileged IDs/kinds.
3. Ensure built-in template installation is called from the source-owned installer and criteria are derived with `derive_from_template`, not intent-only provenance.
4. Construct a resolver over Task 11's validated seed DomainModelRef and invoke `verify_plan_criteria_with_resolver` in both normal and externally supplied plan ingress before authority.
5. Assert criteria/model hash changes invalidate prior approval/declaration bindings.

### Gate

```bash
cargo test -p sea-forge-planner --test conformance_m10 -- --nocapture && cargo test -p sea-forge-planner --test criteria_provenance m10_ -- --nocapture && cargo test -p sea-forge-cli --test conformance_m10 -- --nocapture
```

**Done when:** Real template instantiation resolves desired outcomes against the pinned seed, drift fails before authority with zero side effects, and restoring the placeholder breaks T10.3.

**Redesign trigger:** If the seed lacks the required concept class, correct the source model and regenerate through Task 11's governed path; do not weaken class validation.

---

## Task 13 — Make Thoth claim derivation bounded and total  (M11 library · critical)

**Goal:** Every typed question has a bounded handler, emitted claims never exceed permitted classes/evidence, and a required-fresh policy refuses stale snapshots before query execution.

**Why this shape:** Pure protocol semantics must be correct before CLI/server persistence. Fixing mediation first would ledger incorrect disclosure.

### Steps

1. Add RED unit tests in `crates/sea-forge-thoth/src/engine.rs` for declared-only/installed-only partial grants, unsupported capability under each grant, every QuestionKind, empty-evidence responses, recorded-decision `ask_why_denied`, and required-fresh refusal before any SnapshotView query method is called.
2. Refactor `derive_claims` so each emitted `claim_class` is in `permitted`, status is capped to that class's evidence rung, and unsupported/unknown is represented without leaking a forbidden class.
3. Implement bounded typed handlers for operation requirements, authority requirements, failure explanation, evidence-for-claim, and why-denied; never return `Answered` with an unexplained empty claim set.
4. Make `ask_why_denied` require and resolve a verified authority-decision reference, using only its allowed reason codes/classes.
5. Replace `freshness_of`'s stale-return placeholder with an explicit pre-query denied/refused outcome when `DisclosurePolicy::requires_fresh()` is true; retain disclosed stale answers only for grants that permit them.

### Gate

```bash
cargo test -p sea-forge-thoth -- --nocapture
```

**Done when:** Partial grants cannot reveal higher classes, every question has a deterministic typed outcome, required-fresh refusal invokes no snapshot query, and removing the class cap or pre-query freshness guard fails a test.

**Redesign trigger:** If a question cannot be answered from current typed snapshot APIs, extend the typed interface minimally; never expose raw graph queries or broadly retrieve then redact.

---

## Task 13B — Enforce immutable Thoth authorship at settlement and promotion  (M11 SoD · critical)

**Goal:** Every Thoth-authored claim carries immutable provenance, and the same actor cannot declare its settlement or promote its capability through copied or replayed records.

**Why this shape:** SoD belongs at the two authority-bearing boundaries, not only in the answer engine or unit tests.

### Steps

1. Add RED tests in `crates/sea-forge-settlement/tests/criteria_provenance.rs` and `crates/sea-forge-capability/tests/conformance_m4a.rs` for same-author deny, different-author allow, copied claim, relabeled claim, and replayed claim.
2. Set `authored_by` from the immutable Thoth actor identity when `crates/sea-forge-thoth/src/engine.rs` constructs claims; include it in canonical claim hashing.
3. Move the shared SoD predicate to `sea_forge_core::types::validate_claim_authorship_sod` in `crates/sea-forge-core/src/types.rs`, as approved in Task 0. Remove `check_sod` from `sea-forge-thoth` after all callers use the core helper.
4. Invoke the check in `crates/sea-forge-settlement/src/declaration.rs` before accepting a declaration and in `crates/sea-forge-capability/src/promotion.rs` before applying promotion weight.
5. Persist typed `sod_violation` evidence without stripping original authorship.

### Gate

```bash
cargo test -p sea-forge-settlement thoth_sod -- --nocapture && cargo test -p sea-forge-capability thoth_sod -- --nocapture && cargo test -p sea-forge-thoth t11_7 -- --nocapture
```

**Done when:** Same-author settlement/promotion denies at both boundaries, copied/replayed provenance still denies, a different authorized actor can proceed, and removing either boundary check fails its test.

**Redesign trigger:** If moving the predicate would create a public core contract not approved in Task 0, duplicate no logic: stop and obtain approval for the smallest shared typed helper.

---

## Task 14A — Build the real snapshot view and mediated ask service  (M11 service · critical)

**Goal:** One synchronous Thoth service joins verified installation state, obtains an exact disclosure grant, enforces its freshness requirement, and commits the complete question-to-answer evidence chain.

**Why this shape:** State joins and governance are one kernel-side service. CLI and server become adapters only after this contract is proven.

### Steps

1. Add `crates/sea-forge-thoth/tests/conformance_m11_service.rs` with RED cases for demonstrated, attempted-only, unavailable, absent-policy denial, partial grants, stale-required refusal, and replay.
2. Implement `LedgerSnapshotView` in a new `crates/sea-forge-thoth/src/service.rs` over Task 11 snapshots plus ledger-verified capability, projection, settlement, and probe records; remove any need for caller-supplied status.
3. In the same service, construct the canonical `self_disclosure` action, call the existing authority mediator, and derive `DisclosurePolicy` from the matched grant, including `require_fresh_snapshot` and permitted regions/classes.
4. Commit question, authority decision, DisclosurePlan, bounded query plan, answer, and evidence to the acting ledger stream before materializing compatibility views.
5. Validate purpose length, typed subject, ULID-backed IDs, actor identity/sponsor, and case-read scope before authority.

### Gate

```bash
cargo test -p sea-forge-thoth --test conformance_m11_service -- --nocapture && cargo test -p sea-forge-thoth
```

**Done when:** Real verified records determine claim status, stale-required grants refuse before query, every terminal answer resolves the complete ledger chain, and bypassing mediation fails the service test.

**Redesign trigger:** If the service creates a dependency cycle, pass narrow ledger-verified reader/committer traits from the existing owner crate; do not return joins to CLI or duplicate mediation.

---

## Task 14B — Expose the one Thoth service through CLI and server  (M11 adapters · high)

**Goal:** CLI and server use one ask service that derives real capability/environment state and commits question, authority decision, disclosure/query evidence, and answer.

**Why this shape:** One shared service prevents CLI/server governance divergence and replaces fixture-injected capability truth.

### Steps

1. Add RED CLI/server adapter tests for allowed/denied calls and identical governance record IDs/parents across both ingresses.
2. Delete `SelfModelSnapshotView` and direct policy loading from `crates/sea-forge-cli/src/commands/ask.rs`; call Task 14A's service and retain only output formatting and exit-code mapping.
3. Add the Task 0-approved `ask` request variant to `crates/sea-forge-server/src/lib.rs`; invoke the same service through `spawn_blocking` and return its typed answer.
4. Add version-skew/protocol tests for clients that do not understand the new request variant.
5. Prove both adapters leave the same question, decision, disclosure/query evidence, and answer lineage for equivalent inputs.

### Gate

```bash
cargo test -p sea-forge-cli --test ask_cli -- --nocapture && cargo test -p sea-forge-server ask -- --nocapture && cargo test -p sea-forge-thoth
```

**Done when:** Permitted production asks return evidence-backed states, denied asks leak no restricted facts, CLI/server ledger shapes match, and bypassing the authority service fails both ingress tests.

**Redesign trigger:** If server protocol compatibility requires a versioned request variant, obtain the Task 0-approved schema change; do not add a separate server answer engine.

---

## Task 15 — Preserve delegation schema, termination, and settlement semantics  (M13 · high)

**Goal:** Response schemas are enforced, turn-cap output still reaches criteria evaluation, and case replay preserves the original detailed settlement basis.

**Why this shape:** Termination describes why dialogue stopped; settlement evaluates outcome evidence. One original SettlementEvent must flow through delegation and case completion.

### Steps

1. Add RED tests in `crates/sea-forge-server/tests/conformance_m13.rs` for schema-valid/invalid output, turn-cap with criteria accepted/rejected, and case-level preservation of cancellation/cap/endpoint/criteria bases.
2. Carry `response_schema` from `Operation::AgentTask` through `crates/sea-forge-server/src/case_dispatch.rs` and `DelegationRequest`; validate final output with the approved existing/minimal JSON Schema mechanism.
3. Evaluate available final output/artifacts for `TurnCapExceeded` in `crates/sea-forge-server/src/delegation.rs`; retain `turn_cap_exceeded` in basis without forcing rejection.
4. Return or reuse the committed `SettlementEvent` from delegation instead of constructing `basis: ["delegation_completed"]` in case dispatch.
5. Persist validated schema output as named evidence; schema failure returns typed `schema_invalid` without raw sensitive payload leakage.

### Gate

```bash
cargo test -p sea-forge-server --test conformance_m13 schema -- --nocapture && cargo test -p sea-forge-server --test conformance_m13 turn_cap -- --nocapture && cargo test -p sea-forge-server --test conformance_m13
```

**Done when:** Cap termination can accept or reject solely by criteria, schema failures are typed/evidenced, case replay retains exact basis, and restoring the unconditional rejection fails tests.

**Redesign trigger:** If full JSON Schema requires a new dependency not approved in Task 0, stop for approval or narrow the supported schema subset explicitly in the spec; do not pretend arbitrary JSON Schema is validated.

---

## Task 16 — Implement retention precedence and sealed summarized storage  (M13 security · critical)

**Goal:** Retention resolves plan item → endpoint → global → summarized default, full mode stores public canonical transcripts, and summarized mode stores only the approved sealed canonical transcript plus summary/digest.

**Why this shape:** Task 2 owns bytes/redaction; this task owns policy selection and storage. Keeping them separate avoids embedding retention policy in serialization.

### Steps

1. After Task 0 approval, add retention fields/defaults to `crates/sea-forge-agent/src/config.rs`, server config, and delegation request with version-skew tests.
2. Add RED T13.6 tests using one transcript across every precedence level and both modes; assert identical redacted digest and mode-specific visibility.
3. Resolve retention once in `crates/sea-forge-server/src/case_dispatch.rs` and pass the typed mode into delegation; remove ignored `transcript_retention` destructuring.
4. In full mode, store Task 2 canonical bytes at the specified content-addressed public path. In summarized mode, seal the same bytes using the approved format/key source outside the public transcript surface and verify before completion.
5. Add tamper, wrong-key, missing-sealed-payload, crypto-shred, restart, and no-plaintext-sentinel tests. Missing verification settles rejected; it never degrades to summary-only success.

### Gate

```bash
cargo test -p sea-forge-server --test conformance_m13 retention -- --nocapture && cargo test -p sea-forge-server --test conformance_m13 redaction -- --nocapture
```

**Done when:** Precedence is deterministic, both modes commit the same redacted digest, summarized plaintext is absent, sealed tamper fails settlement, and forcing full retention makes the default-mode test fail.

**Redesign trigger:** If approved sealing cannot be made restart-verifiable without ambient secrets, stop and revisit the owner decision; never store plaintext under a renamed path or claim unverifiable sealing.

---

## Task 17 — Resolve real endpoints and authority-bound manager caps  (M14/M15 · high)

**Goal:** Built-in topology and manager proposals resolve valid configured endpoints, dispatch end to end, and manager iteration cannot exceed the authority grant.

**Why this shape:** Topology correctness is not proven by sentry-only tests when generated tasks cannot pass endpoint preflight. Manager limits belong in the same exact-action boundary used to propose work.

### Steps

1. Add a validated endpoint parameter/default-resolution contract to topology templates and manager catalog; never substitute endpoint IDs through untrusted free-form text.
2. Replace `agent:builtin` and `agent:default` in `crates/sea-forge-planner/src/templates.rs` and `crates/sea-forge-cli/src/commands/manager.rs` with resolved IDs that satisfy `[a-z0-9_-]{1,64}` and registry lookup.
3. Extend M14 tests to configure stub endpoints and dispatch sequential/concurrent plans through the server, including rejected branch rollup behavior.
4. Bind requested `max_manager_iterations` into the canonical manager authority action and decision; enforce the minimum of CLI, config default, and grant in `crates/sea-forge-cli/src/commands/manager.rs`.
5. Extend M15 tests for caller-above-grant, config-above-grant, hash changes, exact exhaustion, and no proposal after cap.

### Gate

```bash
cargo test -p sea-forge-planner --test conformance_m14 -- --nocapture && cargo test -p sea-forge-cli --test conformance_m15 -- --nocapture && cargo test -p sea-forge-server topology -- --nocapture
```

**Done when:** Generated plans dispatch against registered endpoints, all rollups settle from real episode results, grant caps override larger caller/config values, and restoring colon IDs fails preflight tests.

**Redesign trigger:** If built-ins cannot select a safe default endpoint, require an explicit endpoint parameter and reject omission; do not invent or auto-route among endpoints.

---

## Task 18 — Reconcile late SWE_SEED declarations  (M16 · high)

**Goal:** A qualifying SWE_SEED declaration arriving after delegation settlement becomes correlated to the exact run through an idempotent rebuildable projection.

**Why this shape:** Append-only source records cannot be rewritten. Late joins belong in a projection/reconciler keyed by immutable run identity.

### Steps

1. Add RED M16 tests in `crates/sea-forge-server/tests/conformance_m16.rs` that settle a SWE_SEED-harnessed delegation with harvested proofs, receive a declaration through a real server-owned settlement-authority path, and verify immediate correlation. Add a second test that appends directly through `sea_forge_settlement::append_declaration_ledgered_once` while the server is absent, then proves startup/read-time reconciliation. Cover duplicate runs, wrong run/verifier/hash, and declaration-before-artifact.
2. Add `submit_swe_seed_declaration` beside delegation settlement in `crates/sea-forge-server/src/delegation.rs`. After the settlement and harvested evidence are committed, load the approved settlement-authority descriptor from the active policy, construct the declaration request only from persisted criteria/evidence, call `CommandSweSeedTransport`/`SweSeedSettlementAuthority` through `spawn_blocking`, and retain `settlement_authority_unavailable` without local fallback. This is the production declaration ingress that is currently absent.
3. Create `crates/sea-forge-server/src/swe_seed_reconciliation.rs` with `append_and_reconcile_swe_seed_declaration`, `reconcile_swe_seed_declarations(root, case_id)`, and `verify_swe_seed_completion(root, run_id)`. The wrapper commits the declaration through `append_declaration_ledgered_once` and then invokes the same pure reconciler used by replay. The builder reads ledger-verified harvested refs and declarations and binds run, case, plan item, declaration ID/hash, and proof hashes.
4. Register the module in `crates/sea-forge-server/src/lib.rs`; make `submit_swe_seed_declaration` call the wrapper. Also invoke reconciliation during server startup/case-ledger replay and at the start of `verify_swe_seed_completion`, so declarations appended out of process are joined before a completion claim is read.
5. Materialize `runs/<run_id>/swe-seed-correlation.json` only from the committed correlation entry. Replace the one-time correlation construction in `crates/sea-forge-server/src/delegation.rs:569-592` with the shared reconciler, and make the M16 completion/release assertion call `verify_swe_seed_completion`. Preserve real SWE_SEED-host tests as ignored/skipped unless operator configuration exists and report them honestly.

### Gate

```bash
cargo test -p sea-forge-server --test conformance_m16 swe_seed -- --nocapture && cargo test -p sea-forge-settlement swe_seed -- --nocapture && cargo test -p sea-forge-server swe_seed_reconciliation -- --nocapture
```

**Done when:** Server-owned declaration append correlates immediately, out-of-process append correlates on startup/read, matching declarations appear exactly once, mismatches never correlate, rebuild is byte-identical, and removing either trigger fails its dedicated late-arrival test.

**Redesign trigger:** If settlement declaration transport cannot emit a local append notification, make reconciliation an explicit deterministic command plus read-time freshness check; do not mutate the original delegation settlement.

---

## Task 19 — Re-run all specification gates and align claims  (closeout · blocking)

**Goal:** Every repaired requirement has executable proof, all prerequisite gates pass in order, and repository status documents only claims supported by those results.

**Why this shape:** Focused passes cannot establish cumulative conformance. Final acceptance must include minimum proofs, milestone suites, security boundaries, and real-integration skips.

### Steps

1. Run every focused command from Tasks 1-18, including Tasks 10A/10B, 13B, and 14A/14B, then all global gates without substituting earlier output.
2. Run the proof commands in `.agents/specs/spec-minimum.md:846-891`, the full-spec milestone gate in `.agents/specs/spec-full.md:1620-1744`, M9-M11 at `.agents/specs/spec-adlc-thoth-minimum.md:439-470`, and M12-M16 at `.agents/specs/spec-agent-orchestration.md:402-475`.
3. Run real ACP/SWE_SEED/macOS checks only where configured; record each unsupported test as skipped with reason and leave its claim unproven.
4. Update `.agents/CURRENT_STATUS.md`, resolve matching `.agents/OBSERVED_DEBT.md` entries, and update spec status/claim tables only for gates that actually passed.
5. Perform an independent code review for correctness, fail-closed behavior, schema compatibility, dependency boundaries, and test teeth; run `devbox run -- just context-check` after documentation changes.

### Gate

```bash
devbox run -- just context-check && devbox run -- just check && devbox run -- just test && devbox run -- just proof && devbox run -- just no-async-kernel
```

**Done when:** All portable gates exit 0, every acceptance item below resolves to executable evidence, intentionally breaking each repaired boundary fails its focused test, and skipped platform/real-host claims remain explicitly unproved.

**Redesign trigger:** Any failed prerequisite gate reopens its owning task; do not revise a status claim, weaken a test, or skip a portable check to close the plan.

---

## Final acceptance checklist (whole plan)

- [x] Clean baseline and exact dependency/contract approvals are captured in ADR-002/ADR-003; unresolved choices block dependent tasks. *(Task 0)*
- [x] Cell import rejects every traversal/absolute/symlink escape before mutation, with an outside-sentinel teeth-check. *(Task 1)*
- [x] HTTP and ACP share redaction and canonical bytes; redaction-after-hash fails tests. *(Task 2)*
- [x] Jail default connect and bind/listen denial is OS-enforced or setup fails closed; disabling the rule fails socket tests. *(Task 3)*
- [x] Validator is config-free, demo argv exact, and plan written once in lifecycle order. *(Task 4)*
- [x] Multi-file DomainForge validation covers all sources, imports, versions, URIs, and limits. *(Task 5)*
- [x] Memory grants bind and enforce exact requester/target scope before reads. *(Task 6)*
- [x] SQLite FTS rebuild and stale/missing/corrupt fallback are result-equivalent. *(Task 7)*
- [x] Compatibility recall is unchanged/read-only; governed recall emits result-linked evidence. *(Task 8)*
- [x] Spec stages cannot advance from absent, invalid, rejected, or mismatched predecessors. *(Task 9)*
- [x] M5 stages execute as ordinary governed case episodes and invalid output blocks successors. *(Task 10A)*
- [x] A real CLI project invocation proves the complete M5 stage/projection chain. *(Task 10B)*
- [x] Self-model snapshots and projections derive from real state and resolve full ledger governance refs. *(Task 11)*
- [x] ODI desired outcomes resolve against the pinned seed before authority; placeholders are absent. *(Task 12)*
- [x] Thoth disclosure is class-bounded and total, and required-fresh grants refuse before query. *(Task 13)*
- [x] Authored claims cannot self-settle or self-promote through copied/replayed records. *(Task 13B)*
- [x] One mediated service joins real capability/environment state and commits the complete ask chain. *(Task 14A)*
- [x] CLI/server ask are thin adapters over that service with identical ledger lineage. *(Task 14B)*
- [x] Delegation preserves schema, criteria, termination, and exact settlement basis. *(Task 15)*
- [x] Retention precedence and sealed summarized verification pass with no plaintext leakage. *(Task 16)*
- [x] Topologies dispatch valid endpoints and manager iterations stop at the grant boundary. *(Task 17)*
- [x] Late SWE_SEED declarations reconcile idempotently to exact runs. *(Task 18)*
- [x] `devbox run -- just check` exits 0.
- [x] `devbox run -- just test` exits 0 after the clean baseline and at final acceptance.
- [x] `devbox run -- just proof` preserves P1-P4b.
- [x] `devbox run -- just no-async-kernel` preserves the kernel dependency boundary.
- [x] `.agents/CURRENT_STATUS.md` and applicable spec claim tables reflect only executable results; platform/real-host skips remain explicit.

## Guardrails (do not violate)

- Do not add or upgrade dependencies, alter persisted schemas, policy precedence, ID grammar, exit codes, public interfaces, or CI without explicit owner approval.
- Do not create a second authority gate, scheduler, ledger, memory truth store, transcript serializer, safe-path algorithm, or self-model source of truth.
- Do not permit side effects before all exact authority decisions exist; isolation never substitutes for authority, and authority never substitutes for isolation.
- Do not fall back from jail to local, strong settlement to local, sealed retention to plaintext/full, SQLite freshness to a stale result, or DomainForge validation to entry-only parsing.
- Do not expose restricted data and attempt post-retrieval redaction. Bound Thoth queries and transcript redaction before materialization.
- Do not hand-edit generated zones or commit `.sea-forge/**` runtime output.
- Do not weaken, rename around, ignore, or delete a failing conformance test. Add teeth by proving absence of forbidden side effects.
- Keep kernel crates synchronous. Tokio/HTTP remain confined to `sea-forge-server`, `sea-forge-agent`, or an explicitly approved runtime adapter.
- Preserve unrelated worktree changes. Keep each task's implementation, tests, and required contract documentation in one focused commit; keep unrelated cleanup separate.
