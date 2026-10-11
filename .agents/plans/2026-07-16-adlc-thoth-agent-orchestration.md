# Implementation Plan — ADLC/Thoth Self-Model (M9–M11) and Governed Agent Orchestration (M12–M16)

**Created:** 2026-07-16  
**Revised after adversarial review:** 2026-07-16  
**Source of truth:** `.agents/specs/spec-adlc-thoth-minimum.md`, then `.agents/specs/spec-agent-orchestration.md`, then `spec-full.md`, then `spec-minimum.md`. Quote the specs; do not reconstruct requirements from this plan.  
**Baseline:** branch `full-spec`, commit `b351c95`, with M0–M8 implemented. M9–M16 have not started.

This revision corrects implementation assumptions that were false in the first draft:

- tracked templates cannot live under `.sea-forge/`; that directory is runtime output only;
- E8 templates cannot currently carry sentries, stages, dependencies, all-of rollups, or dynamic item lists;
- the server semaphore currently caps whole CLI subprocesses, not concurrently active plan items;
- the server has no live cancellation handles or suspended ACP approval channel;
- current settlement records do not carry the authorship/proposal provenance needed for Thoth SoD;
- `endpoint_ref` alone is not an exact-action authority boundary;
- desired-outcome provenance lacks its required typed `domain_model_ref`;
- the current projection enum lacks `kg` and `self_model_snapshot`;
- real SWE_SEED, hosted-endpoint, and ACP-host tests cannot be ordinary portable `cargo test` gates.

---

## 0. Operating contract

### 0.1 Approval checkpoints

The executing agent MUST stop for owner approval before:

1. adding or selecting HTTP, async-trait/future, URL, zeroization, or ACP dependencies;
2. changing persisted records, ID grammar, policy-bundle schemas, authority boundaries, exit codes, CLI/server public interfaces, or the `justfile`/CI surface;
3. narrowing ACP support after T16.3;
4. changing the specs to resolve a contradiction.

The approval request must name the exact schema delta or dependency version/features and its compatibility effect. A draft spec is not blanket authorization to bypass this repository rule.

### 0.2 TDD and task size

Every behavior change follows RED → GREEN → REFACTOR. Each numbered slice below is a separate reviewable commit unless two adjacent slices are mechanically inseparable. Do not combine a prerequisite refactor with the feature that consumes it.

Before editing a file:

- read it completely;
- inspect one nearby pattern and its tests;
- confirm the nearest `AGENTS.md`;
- preserve unrelated worktree changes.

### 0.3 Source-owned assets

- Canonical model assets live under `models/`.
- Canonical built-in template assets live under `crates/sea-forge-planner/assets/templates/` or an equivalent source-owned directory approved during implementation.
- Runtime installation materializes immutable pinned copies into `<root>/templates/`.
- Never commit `.sea-forge/**`.

### 0.4 Cumulative milestone gate

Every completed slice runs its focused tests. Every completed milestone runs:

```bash
devbox run -- just context-check
devbox run -- just check
devbox run -- just test
devbox run -- just proof
devbox run -- just no-async-kernel
```

It must also run the new milestone test and every prior M9+ conformance test. Add a repository-owned aggregate recipe or script only after the required CI/config approval. `just ci` alone is insufficient because it does not run `just proof`.

CLI proof commands must:

- build `target/debug/sea-forge` first;
- use a temporary root and explicit policy/config fixtures;
- assert expected exit codes, including governed denials;
- never rely on a globally installed `sea-forge`;
- never mutate tracked fixtures for a teeth-check.

### 0.5 Status and evidence discipline

After every milestone:

- update `.agents/CURRENT_STATUS.md`;
- update an existing `.agents/OBSERVED_DEBT.md` entry or add one only for concrete out-of-scope debt;
- update the applicable spec §5 claim table with actual evidence;
- report skipped platform or real-integration tests as skipped, never passed;
- run `devbox run -- just context-check`.

---

## 1. Dependency graph

```text
Task 0 baseline + approvals
  └─ Task 1 M9 self-model
      ├─ Task 2 M10 E8/ADLC/ODI
      └─ Task 3 M11 Thoth
          └─ Task 4 M12 provider seam (also requires Task 2: M0–M11 must all be green)
              └─ Task 5 M13 scheduler + governed delegation
                  ├─ Task 6 M14 topology templates
                  │   └─ Task 7 M15 manager loop
                  └─ Task 8 M16 ACP + SWE_SEED
```

Tasks 2 and 3 may be developed independently after Task 1 only if their shared core/authority contracts are frozen first. Task 4 waits for both. Task 8 may proceed in parallel with Tasks 6–7 after Task 5.

---

## Task 0 — Freeze the baseline and approve contract changes

**Goal:** Establish a reproducible M0–M8 baseline and obtain approval for the known public-contract and dependency changes before implementation.

### Slice 0.1 — Baseline

1. Run the cumulative gate in §0.4 on commit `b351c95`.
2. Record exact pass counts and any platform skips in `CURRENT_STATUS.md`.
3. Confirm the worktree contains no tracked `.sea-forge/**`.
4. Confirm all M0–M8 proof commands remain available from Devbox.

### Slice 0.2 — Contract inventory and approval

Prepare one compatibility note covering:

- new IDs: `smsnap_`, `thq_`, `tha_`;
- additive persisted types and fields for snapshots, realizations, desired-outcome refs, transcript evidence, delegation control, and manager iterations;
- `ProjectionKind::{Kg, SelfModelSnapshot}`;
- policy surfaces and boundaries for `self_disclosure`, `external_api`, and `secret_access`;
- `Operation::AgentProbe`, `ItemKind::AgentTask`, `Operation::AgentTask`, settlement bases, cancellation records, and authorship/proposal provenance;
- additive CLI/server operations and their exit codes;
- E8 template-schema additions with backward-compatible defaults;
- version-skew fixtures and migration/read compatibility.

Classify enum additions separately from optional-field additions. Closed serde
enum additions are incompatible with old readers. For each new variant, the
compatibility note must choose one explicit policy:

- bump the containing record/schema version and prohibit old-reader
  consumption;
- provide a compatibility view/down-conversion;
- change the boundary to an approved open/unknown representation.

Tests must exercise an old reader against records containing each new variant;
“legacy records still deserialize” is necessary but not sufficient.

Obtain explicit approval before changing these contracts.

### Slice 0.3 — Dependency decision records

Research official documentation and propose exact, pinned dependencies for:

- HTTP/TLS and URL parsing;
- async trait/object strategy;
- secret zeroization, if used;
- ACP protocol/client support.

Prefer existing workspace dependencies and standard library facilities. Record maintenance, license, feature set, transitive async runtime, and security implications. Obtain approval before editing manifests.

### Slice 0.4 — Resolve summarized-transcript verification

The agent spec currently requires:

- summarized mode to omit the full transcript artifact;
- the transcript hash to be verifiable at completion.

A digest cannot be recomputed after its input bytes are discarded. Stop for an
owner/spec decision before M13. Present these options:

1. retain a sealed/encrypted canonical transcript outside the public `full`
   artifact surface and verify before crypto-shredding;
2. define and retain a separately verifiable commitment/proof structure;
3. narrow summarized-mode completion to ledger-integrity verification of the
   recorded digest, while reserving transcript recomputation for `full` mode.

Recommendation: option 1 if post-run auditability is mandatory; option 3 if
data minimization is the stronger requirement. Update the spec and T13.6
expectation before implementation.

**Gate:** baseline cumulative gate green; approvals recorded; no feature code changed.

---

## Task 1 — M9: Genesis self-model, realizations, snapshots, and projections

**Goal:** Validate the two bundled models through the in-process DomainForge adapter, produce immutable ledgered snapshots, and rebuild KG/CALM/JSON self-projections deterministically.

### Slice 1.0 — Installation lifecycle ingress

The repository currently has no unified `init`/`upgrade` command and no single
extension-mutation hook. Define the lifecycle before snapshot code:

1. First-root initialization is detected by an absent installation manifest and
   runs through one shared initialization service used by CLI and server.
2. Upgrade is detected by a release ID/schema version change in that manifest;
   upgrade never rewrites prior snapshots.
3. All extension install/adopt/disable paths call one registry mutation service
   that commits the mutation, marks the current snapshot stale, and requests a
   rebuild.
4. Failure atomicity is explicit: a failed snapshot rebuild preserves the
   committed registry truth, records `self_model_error`, and leaves the old
   snapshot verifiable but stale.
5. Add conformance tests for first initialization, release upgrade, each
   extension mutation, retry, and partial failure.

### Slice 1.1 — Release-owned model assets and reproducible realization

1. Add `models/seaforge-system@0.1.0.sea`.
2. Add `models/adlc-odi-case@0.1.0.sea` from the existing seed, with the namespace required by the spec.
3. Preserve and record both:
   - the original seed hash cited by the spec;
   - the final bundled asset hash after versioning/namespacing.
4. Add a deterministic release-realization generator. It may write only to `OUT_DIR` during Cargo builds; a release recipe may materialize a checked artifact. `generated_at` must derive from explicit release metadata or `SOURCE_DATE_EPOCH`, not wall-clock build time.
5. Verify bundled bytes against release constants before `load_validate`.

Tests:

- pristine bytes validate;
- copied temporary fixture with one changed byte fails before parse;
- a build from identical inputs produces identical realization bytes.

### Slice 1.2 — Canonical projection ABI and persisted types

1. Reuse `sea_forge_core::types::ProjectionRecord` as the persisted record used by `sea-forge-spec-pipeline`.
2. Treat the similarly named adapter-layer record in `sea-forge-extension` as an ABI DTO; do not create a third projection record. If conversion is required, add explicit lossless conversion tests and record the duplicate-type debt.
3. Add `ProjectionKind::Kg` and `ProjectionKind::SelfModelSnapshot`; update exhaustive matches, serialization fixtures, version-skew tests, and rebuild-hash tests.
4. Place `SelfModelSnapshot`, `ReleaseRealization`, and `CellRealization` in the narrowest owning crate unless another crate must persist or consume the type without depending on that owner. Move only true cross-crate ABI types into `sea-forge-core`.
5. Add canonical hash functions that exclude only the documented self-hash/timestamp fields.

### Slice 1.3 — Composed model and typed lookup

1. Add `sea-forge-self-model` as a synchronous crate.
2. Extend `SeaSourceSet` composition through DomainForge namespace semantics.
3. If DomainForge cannot compose the required overlays, generate a deterministic pre-merged source set at release time; do not fork parser or validation semantics.
4. Expose a typed, read-only concept lookup and bounded-region query interface. Do not expose raw graph queries.

### Slice 1.4 — Cell realization and probes

1. Keep probe execution outside `sea-forge-self-model`. CLI/server orchestration performs ordinary authority-checked sandbox runs and passes only ledger-verified probe evidence into snapshot assembly.
2. Build cell realization from:
   - extension-registry state;
   - environment contracts;
   - sandbox availability;
   - verified probe evidence.
3. A missing, failed, stale, or unverifiable probe caps status; it never elevates or crashes snapshot creation.
4. Test V5: remove and re-add a toolchain across snapshots; availability follows fresh evidence and is never cached across snapshots.

### Slice 1.5 — Snapshot lifecycle and projections

1. Create snapshots on init, upgrade, extension install/adopt/disable, and explicit rebuild.
2. Event hooks may mark the current snapshot stale before rebuild; they must not silently mutate an immutable snapshot.
3. Rebuild KG/CALM/JSON through the existing projection pipeline, with authority, evidence, settlement, quarantine, and deterministic rebuild hashes.
4. Verify or regenerate bundled pre-generated projections before use.
5. Add CLI:
   - `self-model validate`;
   - `self-model rebuild [--probe]`;
   - `self-model show [--json]`.
6. `self_model_error` blocks only self-model consumers and `ask`.

### Conformance

Implement T9.1–T9.5 plus V5. Teeth-checks use temporary fixtures and temporary roots only.

**M9 gate:** focused crate/CLI tests, T9.1–T9.5, V5, then the cumulative gate.

---

## Task 2 — M10: E8 control-flow vocabulary and ADLC/ODI templates

**Goal:** Make E8 capable of representing the required case control flow, then deliver deterministic ADLC/ODI templates with typed desired-outcome provenance.

### Slice 2.1 — Template control-flow projection

Add backward-compatible fields to `TemplateItem` for:

- `entry_criteria`;
- `exit_criteria`;
- `parent_stage`;
- `depends_on`.

Instantiation must project these fields exactly into `PlanItem`. Parameter substitution remains forbidden in IDs, names, item kinds, sentry event kinds, sentry source IDs, and sandbox classes unless a later typed expansion explicitly allows it.

Tests prove old templates deserialize unchanged and new templates preserve stages/sentries/dependencies byte-deterministically.

### Slice 2.2 — Correct sentry semantics

The current `SettlementStatus` predicate searches all settlement events instead of the matched source event. Fix it so the predicate evaluates the trigger event for the named source.

Tests:

- a rejection from item A cannot activate a sentry bound to item B;
- repeated A rejection reactivates only A’s intended successor;
- replay produces the same activation sequence;
- existing OR-of-sentries behavior remains unchanged.

### Slice 2.3 — Typed desired-outcome provenance

1. Add `OriginRefKind::DesiredOutcome`.
2. Add the approved additive persisted representation for its target. Prefer:
   - `reference`: canonical concept ref;
   - `domain_model_ref`: typed optional field required for `DesiredOutcome`;
   - existing `sha256`: hash binding of the referenced concept/model tuple.
3. Add a `DesiredOutcomeResolver` trait to planner verification rather than hard-coupling planner internals to the self-model crate.
4. Verify model hash, concept membership, and expected Desired Outcome Criterion class at plan-commit time.
5. Reject unresolved or mismatched refs as `criteria_provenance_error` before authority and before execution side effects.
6. Update canonical hashes, version-skew fixtures, and legacy-record compatibility.

### Slice 2.4 — Built-in assets and installation

1. Add source-owned `adlc_case@0.1.0` and `odi_adlc_case@0.1.0` assets.
2. Extend the existing built-in installer to materialize pinned runtime copies under `<root>/templates/` without overwriting a mismatched same-version file.
3. Encode Frame/Form/Build/Activate, required milestones, discretionary-item slot, and simulation-rejection reactivation.
4. ODI prepends job framing and outcome discovery/selection and binds desired-outcome refs.

### Conformance

Implement T10.1–T10.5 and V3. T10.2 must include two rejections followed by acceptance and prove source-bound sentry evaluation.

**M10 gate:** focused planner/CLI tests, T10.1–T10.5, V3, all M9 tests, then the cumulative gate.

---

## Task 3 — M11: Thoth typed disclosure protocol

**Goal:** Answer the nine typed question kinds through disclosure-before-retrieval, with deterministic grounded claims and enforceable non-self-certification.

### Slice 3.1 — Protocol and claim derivation

1. Add synchronous `sea-forge-thoth` with a `protocol` module.
2. Define `ThothQuestion`, `ThothAnswer`, `DisclosurePlan`, `GroundedClaim`, `ClaimStatus`, and `ClaimClass` exactly from the spec.
3. Keep statements template-generated from typed fields.
4. Add the total question-kind → candidate-claim-class table.
5. Derive status from verified snapshot refs only; limitations propagate into every relevant claim.

### Slice 3.2 — `self_disclosure` authority surface

1. Add the approved backward-compatible policy schema.
2. Treat absent surface as deny-all.
3. Validate unknown classes and high-risk grants as specified.
4. Canonicalize a self-disclosure action with actor, case, question kind, candidate classes, subject digest, snapshot ref, and freshness requirement.
5. Ledger the decision before bounded retrieval.

### Slice 3.3 — Bounded query and zero-leakage denial

1. Apply permitted regions inside the query executor.
2. Commit the executed query plan as evidence.
3. Denial output names classes only and does not vary by restricted model content.
4. Leakage tests place sentinel facts in the protected model. The user-supplied subject may appear in the question record; the protected sentinel must appear in no answer, error, log, trace rendering, or server reply.
5. `ask_why_denied` reads only the recorded denial.

### Slice 3.4 — Authorship provenance and SoD

The current settlement path has no claim-author identity. Add approved immutable provenance:

- `authored_by` on claims/evidence that Thoth authors;
- `proposed_by` on discretionary plan mutations;
- binding of those identities into settlement-declaration and capability-promotion inputs.

Enforce:

- Thoth cannot settle a claim it authored;
- Thoth cannot promote its own capability claim;
- an actor cannot bypass the check by relabeling/replaying the claim or proposal;
- sponsor requirements remain enforced for privileged grants.

Do not describe this as “existing proposer≠approver machinery” until the provenance and comparisons exist.

### Slice 3.5 — CLI, server, replay, and R1

1. Add `ask` CLI and server request through one shared governance service.
2. Build explicit temporary allow and deny policy fixtures for tests.
3. Assert exit 0, 4, 2, and 1 paths separately.
4. Add stale-snapshot behavior and V1/V2/V4.
5. Run R1 from Thoth answers through the normal projection pipeline and record any missing question kind in the spec §5 claim table.

### Conformance

Implement T11.1–T11.10, V1, V2, V4, and R1.

**M11 gate:** focused tests, T11.1–T11.10, variations and R1, all M9–M10 tests, then the cumulative gate.

---

## Task 4 — M12: AgentProvider seam and exact external-call authority

**Goal:** Add two HTTP provider shapes behind an adapter boundary while preventing endpoint substitution, SSRF, credential leakage, and pre-authority I/O.

### Slice 4.1 — Provider contract and dependency boundary

After dependency approval:

1. Add `sea-forge-agent` as the only agent-dialogue HTTP/async crate outside `sea-forge-server`.
2. Define an object-safe `AgentProvider` contract and typed message/tool events.
3. Implement exactly `openai_compatible` and `anthropic`.
4. Pin request/response fixtures and size limits.
5. Expand `no-async-kernel` from a Tokio-only hardcoded list to an explicit kernel-crate inventory and forbidden dependency set covering async runtimes and HTTP clients. Include `sea-forge-domainforge`, `sea-forge-spec-pipeline`, `sea-forge-cell`, `sea-forge-artifact-ip`, `sea-forge-self-model`, and `sea-forge-thoth`; exclude only approved adapter/runtime crates.

### Slice 4.2 — Endpoint declaration versus derived status

1. Parse `AgentEndpoint` declarations from config.
2. Register immutable endpoint descriptors as `runtime_adapter` extensions.
3. Keep `declared` config separate from evidence-derived `probed/demonstrated` status; config cannot assert demonstrated status.
4. Endpoint install/change/disable marks the self-model snapshot stale and triggers the normal rebuild path.

### Slice 4.3 — Exact-action authorization and network hardening

Canonicalize each provider call with:

- endpoint ID;
- endpoint descriptor/config digest;
- provider kind;
- normalized scheme/host/port/path base;
- model;
- request-size, response-size, turn, token, and timeout limits;
- credential reference, never credential value.

Bind the grant to that exact action and revalidate immediately before connecting.

Security requirements:

- production endpoints require HTTPS; plain HTTP is limited to explicit loopback test/development mode;
- resolve and reject private, loopback, link-local, multicast, and metadata-service destinations unless the explicit local-test mode applies;
- pin the authorized destination for connection or use an equivalent DNS-rebinding-safe connector;
- disable redirects, or separately authorize every redirect target without forwarding credentials across origins;
- ignore inherited proxy variables unless explicitly configured and granted;
- config reload cannot repoint an in-flight or already-authorized endpoint.

### Slice 4.4 — Credential authority

1. Authorize the exact external call before reading a credential.
2. Resolve `credential_ref` only under a separate exact `secret_access` grant.
3. Hold the value only in provider memory and zeroize where practical.
4. Never place credentials in ACP child environments unless a separate grant explicitly requires it.
5. Tests prove denied external calls perform zero connection attempts and zero secret reads.

### Slice 4.5 — Probe, errors, and evidence

1. Add the approved executable `Operation::AgentProbe` and a dedicated
   governed probe-run service. This is a one-call diagnostic operation, not an
   `agent_task`.
2. The probe service must create the ordinary intent/plan/authority/evidence/
   settlement chain and settle only on a schema-valid response. It reuses the
   provider adapter but none of M13's multi-turn scheduling.
3. Add `agent list` and `agent probe` over that shared service.
4. Use local stub servers for both provider shapes.
5. Cover unreachable, 4xx, 5xx, schema-invalid, oversize, timeout, redirect, config-swap, and DNS/private-address refusal.
6. Prohibit fallback to another endpoint.
7. Sweep records, logs, errors, traces, and transcripts for credential sentinels, including split streaming chunks.

### Conformance

Implement T12.1–T12.6 plus the exact-action and SSRF abuse cases above.

**M12 gate:** focused tests, T12.1–T12.6, all M9–M11 tests, dependency-boundary gate, then the cumulative gate.

---

## Task 5 — M13: Item-level scheduling, durable control, and governed delegation

**Goal:** Make concurrently active plan items real, then execute `agent_task` as ordinary governed run episodes with cancellation and transcript evidence.

### Slice 5.1 — Scheduler refactor before agent execution

The current server semaphore wraps a whole blocking `sea-forge run` subprocess, while the plan pipeline executes active items serially. Refactor before claiming M13 parallelism:

1. Keep the case reducer synchronous and pure.
2. Move case scheduling and episode dispatch ownership into
   `sea-forge-server`. The server calls synchronous planner/kernel services from
   `spawn_blocking`; it no longer delegates whole-case scheduling to a blocking
   `sea-forge run --plan` subprocess.
3. The server acquires its one existing semaphore per executable episode, for
   both sandboxed and agent items.
4. Define an internal episode-dispatch service shared by server and one-shot
   CLI. The one-shot CLI runs the same reducer/service serially with an
   effective concurrency of one; it does not create another semaphore or claim
   parallel execution.
5. Commit dispatch-start ordering before execution and completion ordering on return so replay is deterministic despite wall-clock races.
6. Preserve existing single-item CLI behavior and M0–M12 proofs.
7. Test mixed sandboxed + agent load, not agent-only load.

No second semaphore, pool, queue, or coordination ledger is permitted.

### Slice 5.2 — Additive `agent_task` vocabulary

1. Add approved `ItemKind::AgentTask` and `Operation::AgentTask` contracts.
2. Do not overload executable `Operation` with authority-only variants; preserve the project lesson separating executable operations from broad `AuthorityAction`.
3. Validate endpoint ref, instruction bounds, turns, token budget, response schema, retention mode, and sandbox/workspace constraints at plan acceptance.
4. Add settlement bases and version-skew fixtures.

### Slice 5.3 — Durable cancellation and crash recovery

1. Add an append-only cancellation/control request record tied to case, run episode, item, requester, authority decision, and timestamp.
2. Maintain in-memory cancellation handles only as a projection of durable control state.
3. Define HTTP abort, stream abort, child termination, timeout, and server-restart recovery.
4. Cancellation always commits transcript-so-far and settles `rejected/cancelled`.
5. A cancellation request racing with normal completion has one deterministic winner recorded by ordinal; it cannot produce two terminal settlements.

Tests include restart during cancellation and cancellation before, during, and between turns.

### Slice 5.4 — Delegation loop and transcript evidence

1. Enforce turns, conservative token accounting, request/response sizes, per-turn timeout, and whole-run timeout inside the loop.
2. If provider usage metadata is missing, use a documented conservative accounting method or stop below the grant boundary; never assume zero.
3. Treat all model output/tool requests as untrusted typed data.
4. Redact credentials and sentinel patterns before canonicalization, hashing, summary creation, logging, or storage.
5. Make the structural summary deterministic and bounded.
6. `summarized` and `full` modes hash the same redacted canonical transcript; only `full` stores the artifact.
7. Criteria evaluation, not agent narration or termination reason, decides settlement.

### Slice 5.5 — Hostile tool requests and no-side-effect preflight

1. Every requested tool/action re-enters normal authority and sandbox mediation.
2. An out-of-grant request records denial and leaves the dialogue able to continue.
3. Preflight failure performs no network, secret read, child spawn, or workspace write.

### Conformance

Implement T13.1–T13.7 plus scheduler mixed-load, restart, race, token-accounting, and chunk-boundary-redaction tests.

**M13 gate:** focused tests, T13.1–T13.7, all M9–M12 tests, then the cumulative gate.

---

## Task 6 — M14: Deterministic sequential and concurrent topology templates

**Goal:** Extend E8 only as much as required to instantiate a variable number of agent tasks and an all-success rollup.

### Slice 6.1 — Typed deterministic item expansion

Scalar substitution cannot produce N items. Add a bounded structured parameter and repeat construct:

- typed list of agent-task descriptors;
- deterministic item IDs derived from list order or stable keys;
- hard maximum item count;
- schema validation before expansion;
- forbidden substitution remains enforced for executable kind, sentry event kind, and other privileged fields;
- duplicate IDs/keys reject.

Old templates remain byte-compatible.

### Slice 6.2 — All-of rollup semantics

Current entry criteria are OR-of-sentries. Add a backward-compatible explicit conjunction mechanism, such as `entry_criteria_mode: all` with default `any`, or one typed `AllSettled` predicate. Choose the smaller approved schema.

The concurrent rollup must require all named branches to settle accepted. A rejected branch parks the case and cannot satisfy the success rollup.

Tests prove:

- any-of legacy behavior unchanged;
- all-of waits for every named source;
- an unrelated settlement cannot satisfy it;
- accepted/rejected status is source-bound;
- replay remains deterministic.

### Slice 6.3 — Source-owned topology assets

Add source-owned `sequential_agents@0.1.0` and `concurrent_agents@0.1.0` assets and install them through the built-in installer. Do not write tracked files under `.sea-forge/`.

### Conformance

Implement T14.1–T14.3. Stub latencies prove scheduler behavior only; they do not upgrade the spec §5 “real agent latencies” claim. Keep that claim unproven until Task 8 real integration supplies evidence.

**M14 gate:** focused tests, T14.1–T14.3, all M9–M13 tests, then the cumulative gate.

---

## Task 7 — M15: Deterministic Thoth manager loop

**Goal:** Advance stalled cases through bounded, governed proposals without giving Thoth settlement authority or an implicit free-form planner.

### Slice 7.1 — Define the deterministic judgment contract

Before code, document and test the minimum rule table:

- `satisfied`: case is completed by existing settlement state;
- `blocked`: unresolved approval, missing required authority, or terminal dependency block;
- `progressing`: at least one item is active/enabled or new settlement progress occurred since the prior iteration;
- `stalled`: unmet outcome with no active/enabled work and no terminal block.

The manager cannot invent free-form tasks. Proposal content comes from an approved, versioned manager proposal catalog or case-declared discretionary-item template. The chosen source and hash become part of `ManagerIteration`.

### Slice 7.2 — Manager iteration record and governed reads

1. Add the approved `ManagerIteration` record.
2. Bind it to case-file version, ledger head, Thoth snapshot, grounded claim refs, proposal-source ref, and actor identity.
3. Use existing case-read authority plus E13 disclosure rules.
4. Reject a judgment whose refs do not resolve at record time.

### Slice 7.3 — Proposal, SoD, and exhaustion

1. Propose only through the existing discretionary-item mutation path.
2. Carry `proposed_by` immutably into the item and all later settlement/promotion checks.
3. A denied proposal is recorded as the iteration outcome.
4. Enforce the grant’s `max_manager_iterations`.
5. Exhaustion parks and escalates through existing approvals; it does not create a background daemon.
6. Test hostile provenance relabeling and replay.

### Conformance

Implement T15.1–T15.5.

**M15 gate:** focused tests, T15.1–T15.5, all M9–M14 tests, then the cumulative gate.

---

## Task 8 — M16: ACP driver and SWE_SEED integration

**Goal:** Drive CLI-resident agents under SEA authority, keep approval and cancellation semantics live across ACP episodes, and harvest SWE_SEED evidence portably.

### Slice 8.1 — Pin the ACP contract

After dependency approval:

1. Pin the ACP protocol/schema version and fixture corpus.
2. Inventory every supported permission-request kind for that version.
3. Document unsupported kinds as typed deny.
4. Use tokenized argv only, validate executable/attach targets, launch with a minimal explicit environment, and never invoke a shell.
5. Derive sandbox posture from the run grant; reject session-proposed escalation.

### Slice 8.2 — Permission mediation with suspended sessions

Existing approve/resume restarts CLI work and cannot keep a session alive. Add:

1. a durable permission-request record bound to session episode, run, item, exact requested action, and approval ID;
2. a suspended-session state with bounded timeout;
3. an in-memory channel keyed only by the durable approval ID;
4. grant/deny delivery back into the same live session when possible;
5. crash/disconnect behavior that rejects the episode with partial transcript;
6. resume as a new episode linked by `continuation_key`, never a hidden continuation of an unrecorded process.

Tests cover grant, deny-with-session-alive, timeout, server restart, disconnect, and duplicate resolution.

### Slice 8.3 — Permission mapping fidelity

Exercise every permission kind in the pinned inventory:

- each maps to a canonical SEA action and recorded decision;
- unmapped or malformed kinds deny;
- no permission is exercised before its decision;
- no ACP sandbox or permission hint widens the SEA grant.

If fidelity is lossy, stop and request approval to narrow E17 to an allow-listed subset. Never weaken SEA authority semantics.

### Slice 8.4 — SWE_SEED evidence harvesting

1. Accept the SWE_SEED repository/host path through explicit test configuration; never hardcode `~/projects/SWE_SEED`.
2. Verify the configured repository commit before relying on adapter shape.
3. Safe-join harvested paths beneath the run workspace and `.agent-harness/`.
4. Treat harvested content as untrusted evidence, validate size/type/hash, and never execute it.
5. Correlate harvested refs and `SweSeedTransport` declarations to the exact delegating run.

### Slice 8.5 — Portable conformance versus real integration

Split tests:

- portable conformance: scripted ACP fixture server for protocol, permissions, sandbox, disconnect, and resume;
- real ACP-host integration: explicitly configured and ignored/skipped when unavailable;
- real SWE_SEED integration: explicitly configured and ignored/skipped when unavailable;
- hosted endpoint release proof: operator-supplied credential, separately authorized.

A skip keeps the integration claim unproven. M16 development may be code-complete with a documented skip, but release acceptance requires one real ACP host, one SWE_SEED-projected host, and one hosted endpoint as required by §17.7.

### Conformance

Implement T16.1–T16.5 portably where the spec permits an equivalent server. T16.1 also requires one real compatible ACP host before claiming protocol integration. T16.6 and §17.7 remain release-integration gates with stored evidence.

**M16 code gate:** portable tests, T16.1–T16.5 fixture coverage, all M9–M15 tests, then the cumulative gate.  
**M16 release gate:** real ACP host + T16.6 + hosted endpoint evidence, no skips, then the cumulative gate.

---

## 9. Cross-cutting security and failure tests

The following abuse cases are mandatory where applicable:

- endpoint config changes after authorization;
- redirect attempts with credentials;
- DNS rebinding/private/metadata destination;
- denied call attempts to read a secret;
- parent environment leakage to ACP child;
- malicious model output used as argv, path, policy, or settlement input;
- split-chunk credential echo;
- oversize/garbage/partial streaming response;
- cancellation/normal-completion race;
- server restart with live cancellation or approval;
- duplicate approval resolution;
- forged `authored_by`/`proposed_by`;
- unrelated settlement satisfying a sentry;
- transcript or harvested-proof path escape;
- missing evidence never elevates status or settlement.

---

## 10. Code-complete acceptance

- [ ] T9.1–T9.5 and V5 pass.
- [ ] T10.1–T10.5 and V3 pass.
- [ ] T11.1–T11.10, V1/V2/V4, and R1 pass.
- [ ] T12.1–T12.6 plus exact-action/SSRF/secret-read tests pass.
- [ ] T13.1–T13.7 plus scheduler/cancellation/restart/token/redaction tests pass.
- [ ] T14.1–T14.3 pass with source-bound all-of semantics.
- [ ] T15.1–T15.5 pass with immutable authorship/proposal provenance.
- [ ] T16.1–T16.5 portable conformance passes.
- [ ] T16.6 and §17.7 are either evidenced or explicitly reported as skipped;
      a skip leaves the release claim unproven.
- [ ] No tracked `.sea-forge/**`.
- [ ] No new async runtime or HTTP client in any kernel crate.
- [ ] Every provider/ACP action is exact-action authorized before network, secret read, or spawn.
- [ ] Every terminated delegation has verifiable transcript evidence.
- [ ] Thoth cannot settle or promote its own claims or proposed work.
- [ ] All prior milestone tests and P1–P4b remain unchanged and green.
- [ ] Spec §5 claim tables reflect evidence, not intent.
- [ ] `CURRENT_STATUS.md`, debt, lessons, and open questions are current.
- [ ] Final cumulative gate in §0.4 passes.

## 11. Release acceptance

Release acceptance is stronger than code-complete acceptance:

- [ ] T16.6 passes against a configured, commit-verified SWE_SEED-projected host.
- [ ] §17.7 passes against one real compatible ACP host and one real hosted endpoint.
- [ ] No required real-integration test is skipped.
- [ ] Evidence refs, hashes, authority decisions, and settlements are recorded in the applicable spec §5 claim table.
- [ ] The final cumulative gate passes after the real integrations.

## 12. Stop/redesign conditions

Stop and surface the conflict instead of improvising when:

- DomainForge cannot express required composition without changing validation semantics;
- a persisted/public schema change lacks approval;
- a new dependency lacks approval or fails license/security review;
- item-level scheduling would require a second concurrency mechanism;
- ACP permission mapping is lossy;
- a topology requires an actor/message runtime;
- a manager proposal requires free-form ungrounded task invention;
- a prior proof must be weakened to make a new milestone pass;
- real integration is unavailable but the claim would otherwise be marked proven.
