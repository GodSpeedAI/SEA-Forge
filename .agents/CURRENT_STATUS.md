# Current Status

Updated: 2026-07-17

## Objective

Implement Tasks 2–3 (M10 E8/ADLC/ODI templates + M11 Thoth) from
`.agents/plans/2026-07-16-adlc-thoth-agent-orchestration.md`. M9 is complete
and gated (313 tests, P1–P4b, no-async-kernel). M10 slice 2.1 (E8 control-flow
vocabulary) is in progress.
tag; proven by 11 tests across core serde, ledger replay, and E6 import).
Task 0.3 needs no new dependencies for M9. Task 0.4 is decided (sealed
canonical transcript; gates M13). Committing per the plan §0.2
(slice-per-commit); M9 proceeds straight through to the cumulative gate.

## Worktree State

On branch `full-spec` at `b351c95`. The M0–M8 implementation and closeout are
committed. Current worktree changes are the untracked, revised M9–M16 plan and
updates to this status, `OPEN_QUESTIONS.md`, and the two companion M9–M16
specifications. The last recorded M0–M8
workspace tests, strict static checks, minimum proofs, and context-coupled
Devbox gates passed; Task 0 of the new plan must re-run them before relying on
that baseline. The shared worktree also contains uncommitted M9-related source
and test edits that this documentation pass did not create or verify; preserve
and review them against the now-normative M9 contract before treating them as
an implementation result.
Accepted continuation steps 2–4 and 7–8 are implemented: approval-required
authority remains escalated until an exact ledgered resolution is consumed;
artifact transitions park as one canonical pending record; and approved strong
transitions resume through SWE_SEED to exactly one manifest, declaration, and token.

A code-review pass over `.tmp/cr.md` (27 findings) was applied: 14 fixed in
source/tests (plan_item_id propagation, read/no-assurance authorization, per-episode
approval sequence, required-role enforcement, derived_from canonicalization,
resumed-token proposal-hash check, artifact_id path-traversal guard, attestation rebuild
identity check, terminal retry idempotency, attestation degraded_controls binding, governed
capitalize seeding, + 4 conformance-test fixes), 1 partial (policy identity_bindings + R-SO;
transition SOD rule blocked structurally — SodRule can't scope to transition_kind), 10
verified already-fixed/invalid. Pre-existing M8 CI debt was also cleared to green the gate:
settlement_id is now unique per run, the approve-resolution sequence is derived from the
case-ledger decision count, the capitalize double-grant in the artifact-ip test helpers was
removed, and resume-retry approval grants are idempotent via grant_after_approval_idempotent
(strict double-spend rejection preserved). The workspace is CI-green (fmt + clippy +
289 tests). Remaining open debt: SodRule transition_kind scoping (.agents/OBSERVED_DEBT.md).

## Changed Files

- `.agents/plans/2026-07-16-adlc-thoth-agent-orchestration.md` — revised after
  adversarial review to add approval gates, source-owned template assets, E8
  vocabulary prerequisites, source-bound sentries, item-level scheduling,
  durable cancellation/approval control, exact endpoint authorization,
  credential authority, SoD provenance, and portable/real integration gates.
- `.agents/OPEN_QUESTIONS.md` — records the unresolved contradiction between
  summarized transcript disposal and later hash recomputation.
- `.agents/specs/spec-adlc-thoth-minimum.md` — makes the M9 `self_model.v1`
  additive-record compatibility contract, source-owned templates, lifecycle
  triggers, provenance, and source-bound sentry requirements normative.
- `.agents/specs/spec-agent-orchestration.md` — makes server-owned episode
  scheduling, exact external/secret authorization, durable control/approval,
  source-bound topology semantics, manager SoD, and the M13 transcript-design
  gate normative.
- `crates/sea-forge-sandbox/src/lib.rs` — SandboxClass, ExecutionSandbox trait,
  select_sandbox, SandboxSpec/Handle/Error/RelPath types.
- `crates/sea-forge-sandbox/src/local.rs` — LocalSandbox backend (existing behavior).
- `crates/sea-forge-sandbox/src/jail.rs` — JailSandbox backend (Linux Landlock).
- `crates/sea-forge-sandbox/tests/conformance_m1.rs` — M1 conformance tests.
- `crates/sea-forge-runtime/src/lib.rs` — uses sandbox backend from grant's class.
- `crates/sea-forge-core/src/types.rs` — added ExecutionStatus::SandboxViolation.
- `crates/sea-forge-settlement/src/lib.rs` — settlement basis `jail_violation`.
- `crates/sea-forge-authority/src/lib.rs` — ActionGrant exposes sandbox_class(),
  relaxed hardcoded local-only check to allow any granted class.
- `crates/sea-forge-cli/src/commands/migrate.rs` — new `sea-forge migrate` command.
- `crates/sea-forge-cli/src/commands/inspect.rs` — finds run dirs in both v0.1 flat
  and v0.2 case-nested layouts.
- `crates/sea-forge-cli/src/commands/mediated.rs` — migrated roots report
  `legacy_digest_only` assurance without requiring a signer.
- `crates/sea-forge-ledger/src/types.rs` — `LedgerStream::verify` checks
  `legacy_import` files against recorded sha256/size.
- `crates/sea-forge-cli/tests/conformance_m0_migrate.rs` — M0 migration gate tests.
- `Cargo.toml` — added 10 new kernel crate members to workspace.
- Task 17: `sea-forge-core/tests/version_skew.rs`; CLI `runs --unsettled`,
  parked-run durability/resume reuse, CEP fixture and produced-envelope check;
  final witnessed envelope checkpoint; §18 evidence links and status/debt updates.
- `crates/sea-forge-artifact-ip/src/lib.rs` — M8 registration, transition,
  projection rebuild, strict caller proposal, pending/terminal/claim-manifest
  records, and exact typed authority/approval resolution.
- `crates/sea-forge-artifact-ip/tests/conformance_m8.rs` — M8 conformance and
  hostile rebuild tests for substituted actions, detached approvals, metadata,
  semantic anchors, derivation identity mismatches, and lifecycle view forgery.
- `crates/sea-forge-domainforge/src/lib.rs` — additive typed class references in
  `DomainModelRef`; existing concept membership remains unchanged.
- `justfile` — added `no-async-kernel` recipe; wired into `ci`.
- `Cargo.lock` — refreshed by the workspace expansion.
- `crates/sea-forge-core/src/lib.rs` — reduced to ids/types/errors + `RECORD_VERSION`.
- `crates/sea-forge-cli/src/main.rs` — added `mod pipeline` and `Migrate` command.
- `crates/sea-forge-cli/src/pipeline.rs` — moved from `sea-forge-core`.
- `crates/sea-forge-cli/src/commands/{run,recall}.rs` — updated imports.
- `crates/sea-forge-cli/src/tests/lifecycle.rs` — updated evidence imports.
- `crates/sea-forge-cli/Cargo.toml` — added kernel crate dependencies.
- New crates: `sea-forge-domain`, `sea-forge-authority`, `sea-forge-planner`,
  `sea-forge-sandbox`, `sea-forge-runtime`, `sea-forge-trace`, `sea-forge-evidence`,
  `sea-forge-settlement`, `sea-forge-capability`, `sea-forge-extension`,
  `sea-forge-ledger` (foundation).
- `Cargo.toml` / `Cargo.lock` — added 11 new kernel crate members, added
  `ed25519-dalek` to workspace dependencies.
- Task 9.5 additions:
  - `crates/sea-forge-core/src/types.rs` — added `OriginRef`, `OriginRefKind`,
    `OriginRole`, `CriteriaDerivation`, `DerivationMethod`, `JobContract`,
    `DirectionKind`, `SettlementCriteriaRecord`, `PlanItem.settlement_criteria_ref`,
    `CasePlan.job_contract_ref`, `SettlementClaim.criteria_ref`,
    `SettlementEvent.criteria_ref`.
  - `crates/sea-forge-planner/src/criteria.rs` — derivation, hashing, and
    verification of settlement-criteria records.
  - `crates/sea-forge-planner/src/lib.rs` — re-exports criteria helpers.
  - `crates/sea-forge-planner/src/templates.rs` — `PlanTemplate` gains
    `origin_refs` and `job_contract`.
  - `crates/sea-forge-planner/tests/criteria_provenance.rs` — M2c planner
    conformance tests.
  - `crates/sea-forge-planner/tests/conformance_m2.rs` and
    `crates/sea-forge-planner/tests/template_conformance.rs` — updated struct
    literals for new fields.
  - `crates/sea-forge-settlement/src/lib.rs` — emits `legacy_unattributed_criteria`
    basis and records `criteria_ref` on settlement events.
  - `crates/sea-forge-settlement/tests/criteria_provenance.rs` — M2c settlement
    conformance tests.
  - `crates/sea-forge-cli/src/pipeline.rs` — derives/commits criteria records
    from intent before authority for built-in `run`.
  - `crates/sea-forge-cli/src/plan_pipeline.rs` — derives/commits criteria
    records for `run --plan` proposals.
  - `crates/sea-forge-cli/tests/conformance_m2.rs` — added committed criteria
    record verification.
  - `Cargo.toml` — added `tempfile` to workspace dependencies; planner and
    settlement crates gained required test dependencies.

## Completed

- Repaired the `full-spec` pre-push license gate: workspace crates now use the
  valid custom SPDX reference `LicenseRef-SEA-Forge`, cargo-deny explicitly
  allows that reference, and README license links resolve to the checked-in
  `LICENSE` and `COMMERCIAL-LICENSE.md` files.
- Merged `ci-cd` into `main` and pushed to `origin/main`.
- Task 1 — M0a mechanical crate graduation: moved 14 slice modules into 10 new
  kernel crates plus `pipeline.rs` into `sea-forge-cli`, fixed cross-crate imports,
  added required dependencies, added `no-async-kernel` check, and verified the
  gate: `cargo fmt`, `cargo clippy -D warnings`, `cargo test --workspace`,
  `just proof`, `just no-async-kernel` all pass.
- Noted and fixed one test-path issue: `runtime::tests::timeout_child_helper` became
  `tests::timeout_child_helper` after the move; this is a path reference update,
  not a logic change.
- Task 2 — M0b sea-forge-ledger: complete. All §12 M0 ledger conformance fixtures
  pass: 1000-record multi-stream append with ULID/ordinal/chain/MMR verification;
  one-byte alteration / truncate / reorder / duplicate detection with typed
  `ledger_integrity_error`; Ed25519 signed checkpoints with chain verification;
  MMR inclusion proofs; global checkpoints committing all stream roots;
  independent witness receipts detecting fork substitution (and rejecting
  self-witnessing); secret sentinel redaction rejecting plaintext private keys
  and API keys while accepting approved ciphertext commitments; key rotation
  with old checkpoints verifying under snapshotted key refs; crash recovery
  quarantining incomplete tails. CLI `ledger verify|prove` subcommands added.
- Task 3 — M0c DomainForge semantic adapter: `crates/sea-forge-domainforge`
  created with `domainforge-core = "=0.13.0"`, default features off. Implements
  `load_validate(SeaSourceSet) -> DomainModel` using DomainForge's parser → graph →
  validation pipeline; `DomainModelRef` with `semantic_model_sha256` over canonical
  4-tuple; authority normalization table (Reject/Deny→deny, Escalate→escalate,
  Allow→allow, NotApplicable→deny-if-required); real `.sea` fixture; conformance
  tests covering valid parse → stable ref, invalid syntax → domain_model_error,
  source-hash drift rejection, no-side-effects-on-invalid-input, and normalization.
  pass: 1000-record multi-stream append with ULID/ordinal/chain/MMR verification;
  one-byte alteration / truncate / reorder / duplicate detection with typed
  `ledger_integrity_error`; Ed25519 signed checkpoints with chain verification;
  MMR inclusion proofs; global checkpoints committing all stream roots;
  independent witness receipts detecting fork substitution (and rejecting
  self-witnessing); secret sentinel redaction rejecting plaintext private keys
  and API keys while accepting approved ciphertext commitments; key rotation
  with old checkpoints verifying under snapshotted key refs; crash recovery
  quarantining incomplete tails. CLI `ledger verify|prove` subcommands added.

## Verification

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`: passed.
- `cargo test --workspace --all-features --locked`: passed on the current Task 9 worktree.
- `just proof`: P1–P4b passed.
- `just no-async-kernel`: passed.
- `cargo build --workspace --all-targets --locked`: passed.
- `cargo test -p sea-forge-artifact-ip`: 22 passed, 0 failed.
- `cargo test -p sea-forge-authority`: 36 passed, 0 failed.
- `cargo test -p sea-forge-cli --test conformance_m8_artifact --locked`: 3 passed,
  0 failed. The evaluator/token hostile test was observed red before implementation
  because no evaluator score was ledgered, then green after the M7 path was wired.
- `cargo test -p sea-forge-sandbox --test conformance_m7 --locked`: 10 passed,
  0 failed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `git diff --check` plus untracked M8 file checks: passed.
- `cargo test -p sea-forge-cli conformance_m0_migrate --locked`: passed.
- Task 6 migration gate: lossless genesis import, idempotence guard, ledger
  verify corruption detection, and `legacy_digest_only` inspect assurance pass.
- Task 7 — M1 jail sandbox backend: `SandboxClass` enum (`local|jail|microvm`),
  `ExecutionSandbox` trait (§11.2), `LocalSandbox` (existing behavior), and
  `JailSandbox` (Linux Landlock via the `landlock` crate). Landlock ruleset
  allows read-write to workspace+artifacts, read-only to `/`, denies all other
  writes. Thread-based restriction (no `unsafe`/`pre_exec`) keeps the main
  thread unrestricted. Runtime selects backend from `grant.sandbox_class()`;
  unavailable class returns `unsupported_sandbox_class_error`. Settlement adds
  `jail_violation` basis when `ExecutionStatus::SandboxViolation` is detected.
  Conformance tests: jail blocks write outside workspace, schema_error for
  untrusted argv0 on local, class identity, unavailable-platform refusal.
- `cargo test -p sea-forge-planner --test conformance_m2 --test template_conformance`: 13 passed, 0 failed.
- `cargo test -p sea-forge-cli --test conformance_m2`: 3 passed, 0 failed.
- `devbox run -- just check`: passed on the current worktree; cargo-deny emitted only non-fatal duplicate/unmatched-allowance warnings.
- Task 5 resolver slice: 12 `sea-forge-authority` tests passed, including typed
  deny/escalate/boundary/degraded/allow resolution and order independence.
- Task 5 execution-boundary slice: authority no longer depends on sandbox;
  move-only, non-serializable grants bind the exact action/run/item/workspace;
  public sandbox materialization and runtime execution consume a matching grant.
- Task 5 canonical-decision slice: `LedgerStream::commit_typed` returns an
  opaque committed-record reference; the CLI commits every authority decision
  before writing `authority.json` or issuing its exact-action grant.
- Task 5 review fixes: boundary dimensions now intersect and incompatible
  boundaries deny; grants require a one-use decision issued by the same engine
  plus a non-deserializable committed ref; authority evidence commits before
  the decision that cites it.
- Task 5 candidate slice: authority decisions now persist candidate verdicts,
  winning source, resolution reason, sandbox grant, boundaries, and controls;
  v0.2 engine declarations reject fail-open modes and unavailable required
  engines deny through the typed resolver.
- Task 5 view/extension slice: authority compatibility output materializes only
  after its committed decision and records current/failed freshness; failed
  materialization preserves verifiable ledger truth. Extension registry saves
  are ledger-first, and imported adoption consumes an exact-action grant plus a
  committed authority reference.
- Task 5 ingress slice: run commits intent, plan, identity, policy, request,
  evidence, and decision before effects; validate, recall, and inspect now pass
  through the same authority engine and ledger-backed exact-action check before
  reading protected data. Minimum v0.1 read behavior remains compatible; v0.2
  policies require explicit read rules.
- Task 5 hardening: v0.2 identity maps fail unresolved identities closed and
  require sponsors for automated agents; complete protected operation names and
  fail-closed engine declarations parse in one schema; DomainForge candidates
  compose through the typed resolver; grants bind timeout, environment keys,
  workspace, artifacts, sandbox class, boundaries, controls, and expiry.
- Required-integrity policies now produce signed stream/global checkpoints and
  independently signed witness receipts before any command-start event. Missing
  or duplicate witnesses fail closed before workspace effects. Authority decision,
  audit, and opaque-constraint mirrors rebuild from ledger records; inspect and
  recall surface ledger assurance.
- Task 5 / M0-G3 complete: v0.2 policy snapshots require all authority surfaces,
  RBAC permissions, SoD rules, source hashes, and canonical bundle hashes;
  configured DomainForge evaluation runs through real CLI ingresses; opaque
  constraints preempt matching work; per-record assurance proves inclusion in
  the exact signed/witnessed checkpoint; authority audit records preserve the
  resolved disposition, canonical resource subject, and case linkage.
- Independent final review: approved with no findings.
- Task 6 — M0f `sea-forge migrate`: lossless v0.1 → v0.2 case layout migration.
  `sea-forge migrate` enumerates legacy source files (excluding views/append-only
  `authority/` and `capabilities.jsonl`), emits `legacy_import` genesis ledger
  entries with byte sha256/path/size/legacy record version, signs an initial
  global checkpoint, and relocates run directories under
  `.sea-forge/cases/<case_id>/runs/<run_id>/` and case files to
  `.sea-forge/cases/<case_id>/case.json` without rewriting record bytes. Migration
  is idempotence-guarded by `.sea-forge/migration.json`. `LedgerStream::verify`
  verifies each `legacy_import` file against its recorded hash and size, so a
  corrupted legacy file fails `ledger verify`. `inspect` finds runs in both v0.1
  flat and v0.2 nested layouts and reports `legacy_digest_only` for migrated
  records. Conformance tests verify byte hashes committed, IDs resolvable, ledger
  verify green, re-migration refused, corruption detected, and inspect assurance
  labeling.
- Task 7 — M1 jail sandbox backend: `SandboxClass` enum (`local|jail|microvm`),
  `ExecutionSandbox` trait (§11.2), `LocalSandbox` (existing behavior), and
  `JailSandbox` (Linux Landlock via the `landlock` crate). Landlock ruleset
  allows read-write to workspace+artifacts, read-only to `/`, denies all other
  writes. Thread-based restriction (no `unsafe`/`pre_exec`) keeps the main
  thread unrestricted. Runtime selects backend from `grant.sandbox_class()`;
  unavailable class returns `unsupported_sandbox_class_error`. Settlement adds
  `jail_violation` basis when `ExecutionStatus::SandboxViolation` is detected.
  Conformance tests: jail blocks write outside workspace, schema_error for
  untrusted argv0 on local, class identity, unavailable-platform refusal.
- Task 8 — M2a CMMN-subset case engine: persisted `PlanItem`/`Sentry`/`Case`/
  `TraceKind`/`SettlementCriteria` types; sentry evaluator as a pure function of
  trace events and workspace file set; static `plan_cycle_error` satisfiability
  check on the entry-criteria dependency graph; `validate_proposal` normalization
  (safe IDs, relative paths, no empty plans); deterministic case reducer with
  enable/activate/complete/park/terminate actions; required-item failure
  terminates the case with `terminated rejected`; `parked` is a normal state, not
  a failure; `sea-forge run --plan` plan-proposal driver; `sea-forge case` and
  `sea-forge task` subcommands (reopen, add-task, complete). Conformance tests:
  A/B(rep×2)/C/M scenario replay reproduces activation order; empty entry
  criteria activate immediately; unsatisfiable sentries rejected; required-item
  failure terminates the case; reducer retries then terminates required items;
  parked case is not failure; proposal validation rejects bad paths and cycles.
- Task 9 — M2b plan templates: `PlanTemplate`/`ParameterDef` types with typed
  parameters (`string`, `int`, `bool`, `path`) stored at
  `.sea-forge/templates/<name>@<version>.yaml`; load-time forbidden-substitution
  checks (`kind`, `plan_item_id`, `name`, `sandbox_class`, `argv[0]`);
  deterministic instantiation yielding byte-identical `CasePlan` for same template
  - params; `template_ref` provenance recorded in `CasePlan` and semantic envelope;
  `load_pinned` with per-version SHA-256 pin that rejects content changes without
  a version bump; built-in `sea_model_demo@0.1.0` template. Conformance tests:
  instantiation byte-identity; forbidden `argv[0]` substitution rejected at load;
  missing required parameter is input error; path parameter rejects
  parent/absolute/prefix escape; pin rejects same-version byte change.

- Task 9.5 — M2c settlement-criteria origin and provenance: `OriginRef`,
  `SettlementCriteriaRecord`, `JobContract`, and `CriteriaDerivation` types in
  `sea-forge-core`; `PlanItem.settlement_criteria_ref` and `CasePlan.job_contract_ref`;
  `sea-forge-planner/src/criteria.rs` with `derive_from_intent`,
  `derive_from_template`, deterministic `criteria_sha256`/`criteria_record_hash`, and
  `verify_item_criteria`/`verify_plan_criteria`; `settlement_criteria` records
  committed to the ledger before authority in both `run_intent` and `run --plan`
  pipelines; embedded criteria snapshot/hash agreement enforced; legacy claims
  without `criteria_ref` marked `legacy_unattributed_criteria` in settlement basis;
  no new crate, database, or independent criteria store added; JobContract not
  synthesized for current paths because existing Intent and PlanTemplate substrate
  already satisfies §7.1a. Conformance tests: every new PlanItem resolves to one
  committed criteria record; origin refs resolve and hash-verify; missing ref and
  hash-mismatch fail with `criteria_provenance_error`; same template+params yields
  identical criteria content, origin refs, and criteria_sha256; changing criteria
  changes the hash; legacy items are skipped by verification; built-in demo does
  not create a JobContract.

- Task 10 — M3 server, approvals, operator loop: `ApprovalRequest`/`ApprovalStatus` types; `approvals.jsonl` append-only store with latest-line-wins resolution; escalate→ApprovalRequest→exit 5 in plan_pipeline; `sea-forge approve|reject` CLI with TTL expiry check and no-re-resolution; `sea-forge resume` re-enters the case loop after approval resolution; `sea-forge-server` crate with Tokio runtime, Unix socket NDJSON protocol (submit/status/approve/reject), `spawn_blocking` dispatch via subprocess, `max_concurrent_runs` semaphore, dynamic config reload (last-known-good on invalid), `notify_command` execution (failure logged and ignored). Conformance tests: escalate→exit 5→approve→resume→completed; double-approve refused; reject→resume→terminated (exit 4); expired approval refuses resolution.

- Task 11 — M4a settlement declarations + capability promotion: `SettlementDeclarationRequest`/`SettlementDeclaration`/`Declarer`/`DeclarationIndependence`/`DeclarationReliability`/`SettlementStrength`/`DeclarationStatus` types in `sea-forge-core`; `CapabilityPromotionPolicy`/`CapabilityRecord`/`CapabilityStatus`/`CapabilityQualifying`/`CapabilityVariation`/`CapabilityRecovery`/`CapabilityOrchestration` types in `sea-forge-core`; `crates/sea-forge-settlement/src/declaration.rs` with `SettlementAuthority` trait, `LocalSettlementAuthority` adapter (strength=local always, qualifies_for_capability=false), `SweSeedSettlementAuthority` adapter with pluggable `SweSeedTransport` trait (test-double in tests), `check_integrity` (post-hoc criteria, self-declaration, missing criteria_ref/origin_refs), `compute_declaration_hash`, `append_declaration`/`load_declarations` JSONL store; `crates/sea-forge-capability/src/promotion.rs` with fixed-point decimal arithmetic (6 places, i64 millionths, clamp, zero-denominator rule), `default_v02_policy`, `compute_policy_hash`, `save_policy`/`load_policy` snapshot store, `declaration_qualifies` predicate (status=accepted, strength=strong, qualifies_for_capability, independent, weight>=min, criteria_ref non-empty, evidence-backed tags), `build_capability_record` pure projection (counts from envelopes, qualifying from declarations+policy, variation coverage dedup, recovery tracking, orchestration burden reduction, confidence = reliability_ratio *coverage_ratio* recovery_ratio * burden_factor, status determination attempted<demonstrated<proven, contraction reasons), `rebuild_capability` (byte-identical modulo rebuilt_at), `require_proven` (denies with citation unless status>=proven). 13 conformance tests: raw counts match 5 mixed runs; local declaration zero qualifying weight; post-hoc criteria integrity failure; self-declaration integrity failure; gameable feedback weight below threshold; low attribution weight below threshold; three qualifying declarations promotion to proven; repeated variation no coverage increase; regression contraction; rebuild byte-identity; require_proven denial with citation; require_proven allows when proven; policy change contraction.

- Task 12 — M4b governed semantic memory: `MemoryKind`/`MemoryItemProvenance`/`MemoryItem` types in `sea-forge-core`; `EvidenceKind::Recall` variant added; `memory_scope: Option<String>` added to `PolicyRule` in `sea-forge-authority`; `crates/sea-forge-capability/src/memory.rs` with `compute_dedup_key` (sha256 of kind + normalized statement + entity_id), `extract_from_envelope` (deterministic: one `outcome` item per envelope, statement capped at 1000 chars, provenance from envelope run_id + evidence_refs), `append_memory_items` (append-only JSONL), `load_memory_items` (dedup-at-read: merge by dedup_key, union run_ids/evidence_refs, earliest created_at, latest last_confirmed_at), `recall_memory` (linear scan, scope filter, kind filter, limit capped at 50), `scope_allows` (own/entity:X/any/default-deny), `rebuild_index`/`query_index`/`recall_with_fallback` (pure JSON projection, identical results to fallback scan); pipeline extraction wired after envelope append in `pipeline.rs` (never fails run, errors logged); CLI `sea-forge memory rebuild` command + `sea-forge recall --kind` flag (memory-item mode, contract preserved without --kind). 8 conformance tests: two-entity dedup + provenance, own-scope isolation, cross-entity denial, index-delete equivalence, extraction safety, dedup-key determinism, limit cap, kind filter. ponytail: JSON index instead of rusqlite/SQLite — achieves same outcome (rebuildable projection, fallback-equivalent) without C compilation dependency; switch to rusqlite if linear scan becomes measured bottleneck.

- Task 13 — M5 spec-to-code pipeline + DomainForge projections: `PipelineRoute`/`ProofClassification`/`StageKind`/`StageStatus`/`StageFile`/`SpecPipelineStage`/`SpecPipelineRun`/`ProjectionRecord`/`ProjectionValidation` types in `sea-forge-core`; `run_spec_pipeline`/`run_projection` added to authority operation_kind list; `ProjectionKind` gained Ord/PartialOrd; `project()` function added to `sea-forge-domainforge` (CALM via `calm::export`, RDF via `KnowledgeGraph::from_graph`→`to_turtle`/`to_rdf_xml`); CALM export's non-deterministic `sea:timestamp` stripped for byte-identical regeneration (§10.7); new crate `sea-forge-spec-pipeline` with `compute_stage_hash`/`compute_chain_hash` (linked SHA-256 chain), `validate_stage_order` (canonical stage ordering), `compute_proof_classification` (authority-only → generated-contract → focused-slice ceiling), `quarantine_stage` (sets status + quarantine_ref + basis), `check_generated_zone_edit`/`is_generated_zone` (src/gen, .ast.json, .ir.json, .manifest.json, semantic fixtures), `project_model` (in-memory CALM+RDF via adapter), `compute_rebuild_hash`/`build_projection_record` (ProjectionRecord with rebuild hash), `process_pipeline` (validate + quarantine + classify), `verify_byte_identity` (regeneration determinism), `compute_input_hash`/`hash_content`. 10 conformance tests + 5 domainforge projection tests. ponytail: no `sea-forge project` CLI command yet (gate is crate-level tests only); no `.sea` synthesis adapter (DomainForge validation test covers the contract); declarative Evaluator deferred to M7 (per Appendix A, command form is Task 15).

- Task 14 — M6 SeaCell federation prep: `cell_id` field added (Option<String>, absent = legacy valid) to `TraceEvent`, `EvidenceRecord`, `SemanticEnvelope` in `sea-forge-core`; `BundleFile`/`BundleManifest` types in `sea-forge-core`; `ids::cell_id()` (`cell_<8hex>`)/`ids::bundle_id()` helpers. New crate `sea-forge-cell`: `cell::ensure`/`read` (load-or-create `.sea-forge/cell.json`, idempotent, schema `cell.v1`); `bundle::export` (tar with `manifest.json`, sha256 per file, deterministic header mode, excludes `workspace/` scratch, includes `artifacts/`); `bundle::import` (atomic-reject per §14.8: stage to `.staging-<bundle_id>`, recompute+verify all sha256/size, reject whole bundle on any mismatch — missing/extra/tampered — clean staging on err, atomic rename into `imported/<exporter_cell_id>/`, never touches `capabilities.jsonl`); `bundle::read_manifest`; `template::adopt` (copy from `imported/<cell_id>/templates/` into active `templates/`, leaves imported provenance trail). `EventSink` trait + `SinkEvent` (8 contract fields: `event_id, trace_id, correlation_id, causation_id, idempotency_key, source_agent, occurred_at, schema_version, subject, payload`) + `JsonlEventSink` (append-only JSONL) + `trace_to_sink` (maps `TraceEvent` → `SinkEvent`) + `subject_for` (maps 24 `TraceKind` variants to `sea.{domain}.{action}.{qualifier}` 4-segment subject per ecosystem map) in `sea-forge-trace`. Authority operation_kind allow-list extended: `import_bundle`, `export_bundle`, `adopt_template`. `pipeline.rs` stamps `cell_id` on envelope. CLI: `sea-forge export`/`import`/`adopt` commands with authority mediation. 11 conformance tests: export/import hash-verify, capability-count-unchanged, tampered-bundle atomic reject (teeth: one flipped byte → whole import fails, no leftover dir), extra-entry rejection, missing-manifest rejection, unknown-schema rejection, re-import replaces prior, read-manifest inspection, cell-id stability, absent-cell legacy valid, imported-template-not-instantiable-pre-adopt. New dep: `tar = "0.4"` (spec mandates tar format; integrity-boundary correctness; MIT/Apache-2.0). ponytail: not wiring EventSink into live pipeline (M6 = seam + roundtrip test only); bundles exclude `workspace/` scratch (only evidence files + artifacts); adopt is copy-not-move (preserves imported provenance trail); no environment bundles (M7/Task 15).

- Task 15 — M7 environment contracts + command evaluators: `PlanItem.environment` plus optional `SettlementCriteria.evaluator`, `records`, `per_record_evaluator`, `min_pass_ratio`; `BatchEvaluationResult`/`BatchFailure` and evaluator scores carried on `SettlementClaim`. `sea-forge-sandbox/environment.rs`: YAML `EnvironmentSpec` (`base`, `provides.commands`, command evaluators), first-use SHA-256 pinning at `environments/.pins/`, missing/tampered specs fail `environment_unavailable`, base materialization, score parsing, and built-in `demo_env@0.1.0`. Authority gains `PolicyRule.environment`, an environment context on `AuthorityEvaluation`, and `command_allowed` intersection enforcement: command basename must be provided by the item environment and satisfy any `argv0` rule. Pipeline loads/materializes the environment before authority, executes every authorized command sequentially, executes declared evaluator commands under their own authority decision/grant, and runs per-record evaluator command pairs through authority with each record materialized as `record.json`; settlement records evaluator scores and writes failing batch records to `quarantine/<plan_item_id>.jsonl`. CLI: `sea-forge env list|show`. 10 `environment_*` conformance tests: evaluator basis, score parsing, 10-record ratio 0.8 with 2 failures accepted + 2 quarantined, 3 failures rejected + 3 quarantined (teeth), three-axis independence, hash pinning, missing/tampered fail-before-materialization, demo fixture, base materialization, YAML round-trip. ponytail: declarative predicate evaluators deferred (M7 proves command form); Endpoint/NetworkFlow remain deny-by-default and credentials remain authority contracts, preserving a future DomainForge Cell projection seam.
- Task 16 M8 authoritative rebuild hardening: transition decisions deserialize as
  `AuthorityDecision` and must allow the reconstructed canonical action with all
  source licenses plus exact run/case/`transition` context and payload hash.
  Capital approvals deserialize as `ApprovalRequest` and bind approved,
  pre-expiry resolution to the token's run, case, criteria, decision, plan item,
  requester, and approver. Hostile substituted-action and unrelated-approval
  rebuilds fail closed.
- Task 16 transition cases now derive their intent/criteria provenance and source
  evidence from the resolved `TransitionInput`, copy the profile's single M7
  evaluator and approval requirement into settlement criteria, derive the item
  environment from `<environment-ref>.<evaluator-name>`, and execute the evaluator
  through the existing environment/authority/runtime path. Settlement events
  ledger evaluator scores; the artifact resolver enforces the profile threshold
  before token append. The detached `transition.ok` marker was removed. Profiles
  requiring approval or strong external declarations park before case creation;
  no approval or declaration is synthesized.
- Task 16 review blockers fixed: `quality` is never qualifying capital value even
  when listed by a gate profile; value-source records resolve external-case
  ledgered run evidence plus an accepted linked settlement before append, with
  aggregate acceptance derived from those records; capitalization approvals bind
  both criteria hashes to the resolved `SettlementCriteriaRecord`. Hostile tests
  cover fabricated value sources and missing/wrong approval hashes, and the
  quality-only path proves accepted quality evidence creates no token/projection.
- Task 16 capital hardening now deserializes every strong reference as a complete
  `SettlementDeclaration`, verifies its embedded hash and exact settlement,
  criteria, run, case, and plan-item links, and reuses the default v0.2 capability
  qualification policy. Strong declarations also require ledger-resolved source
  evidence plus explicit standing, independence, reliability, and adapter
  attestation data. Value evidence requires `canonical_value_evidence_kind` on
  each underlying `EvidenceRecord`, with exact source/wrapper agreement.
- Continuation dependency step 2 removes the caller `approval_ref`/`approver_id`
  authority shortcut. `PolicyAuthorityEngine::grant_after_approval` now requires
  exact committed authority-decision, approval-resolution, and criteria records;
  validates all decision/case/run/item/criteria hashes, action/context, expiry,
  and separation of duties; and yields one exact one-use grant. CLI approval now
  resolves and revalidates ledger truth before committing an idempotent resolution,
  then updates `approvals.jsonl` as a compatibility view.
- Continuation steps 3–4 add strict `TransitionProposal` ingress, canonical
  `PendingArtifactTransition`, typed terminal, and immutable claim-manifest
  records. The artifact-only plan extension commits the exact transition
  escalation and standard approval, then `commit_typed_once` commits pending in
  `case-<case_id>` before exposing `awaiting_approval`/exit 5. Strong-only gates
  park identically; no approval resolution or declaration is fabricated.
- Continuation steps 7–8 add validated `settlement_authorities[]` descriptors and
  a real tokenized-argv SWE_SEED command transport with JSON stdin/stdout,
  bounded output, timeout, minimal environment, and fail-closed behavior. Resume
  detects artifact pending/terminal records before generic M3 handling, trusts
  only ledgered approval resolutions, terminalizes reject/expiry, persists and
  reuses evaluator execution/settlement, commits one immutable manifest and
  qualifying strong declaration, consumes the exact approval grant, and commits
  one token plus terminal. Outage remains `awaiting_approval` with no declaration
  or token; retry/replay returns the existing matching records.
- Final Task 16 domain blockers are closed: required metadata uses an explicit
  `product_contract.*` selector vocabulary over ledgered result evidence;
  semantic anchors are typed as concept/class and resolve against a ledgered
  `DomainModelRef`; derive validates exact source/identity pairs and new result
  identities; append-only lifecycle records carry exact authority bindings and
  rebuild independently from maturity, so retired capital remains capital.
- Final Task 16 review blocker fixed: artifact resume now converts an expired,
  unresolved exact pending approval into one ledgered `expired` resolution before
  terminalizing the matching transition (exit 4, no token). Hostile replay
  coverage confirms terminal/no-token behavior and ledger idempotence.
- Task 17 Definition-of-Done sweep: additive v0.2 fields are deserialized by
  exact v0.1 reader snapshots from actual current record serialization; `just ci`
  retains the no-Tokio kernel check; produced semantic envelopes are checked
  against the copied CEP-0008 fixture with the divergence recorded as debt;
  `runs --unsettled` reports run IDs lacking settlement; approval-parked runs
  persist plan/authority snapshots and resume into the same run; required-integrity
  runs append a final signed/witnessed checkpoint covering the semantic envelope.
- Task 17 crash-recovery drill (2026-07-16): submitted real case
  `case_20260716T175502Z_dcd297` through `sea-forge-server`, reaching approval hold
  for `run_20260716T175502Z_5e7bcc`; terminated the server; verified the run kept
  `plan.json` and `authority.json`; restarted the server and verified status was
  absent from volatile memory (no auto-resume); `sea-forge runs --unsettled`
  discovered the persisted run; explicit security-officer approval plus
  `sea-forge resume` completed the same run with accepted settlement.
- Task 17 substrate reconciliation: inspected and reused existing serde record
  types, case files, ledger records, approval/resume path, integrity checkpoint
  writer, CI recipe, and milestone conformance suites. Extended only the CLI
  read path and parked/final checkpoint persistence; added compatibility and CEP
  checks plus the copied schema. Deliberately added no database, recovery daemon,
  second run store, JSON-schema dependency, or alternate envelope format.
- Task 17 final review hardening: generic resume now derives approval status,
  run identity, and criteria binding from unique case-ledger request/resolution
  records rather than `approvals.jsonl`; `runs --unsettled` requires a parseable
  settlement matching the run; value-source settlement lookup matches both
  `set_01` and run ID; compatibility tests use exact v0.1 minimum-record and
  authority shapes against actual current serialization.
- Generic resume verifies the case ledger hash chain before consuming approval
  truth; a hostile edited approval-resolution payload now fails closed.

## Remaining

- Commit Tasks 16–17 when requested.
- Stale stash `stash@{0}` remains from the initial workspace cleanup; will drop
  once the log-file reset is no longer a safety-net concern.

## Tasks 1–4 Specification Reconciliation

- Substrate map, reconciliation matrix, M0 gate evidence, and deferrals:
  `.agents/reports/2026-07-12-tasks-1-4-spec-reconciliation.md`.
- The standalone proposed patch, complete patched specification, and patch guide
  were not found in the repository or nearby project tree, so no `git apply` or
  `git apply --check` was possible. The proposals in the user request were
  evaluated manually against the code.
- `spec-full.md` now defines deterministic verdict resolution with `allow` as
  least restrictive, an opaque exact-action/context authorization boundary,
  canonical ledger-before-view failure semantics, M0-G1–G6, completion-claim
  levels, and cumulative release boundaries.
- The implementation plan assigns those implementation and proof obligations to
  Task 5 without prescribing an `AuthorizedAction` type or a parallel authority
  or persistence system.
- Public runtime execution and sandbox materialization now consume opaque,
  one-use, context-bound authority grants; direct ungranted effects do not compile.

- `cargo test -p sea-forge-planner --test criteria_provenance --locked`: 13 passed, 0 failed.
- `cargo test -p sea-forge-settlement --test criteria_provenance --locked`: 4 passed, 0 failed.
- `cargo test -p sea-forge-cli --test conformance_m2 --locked`: 4 passed, 0 failed.
- `cargo test --workspace --all-features --locked`: passed; no regressions in P1–P4b or earlier milestones.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo test -p sea-forge-artifact-ip --locked`: 27 passed, 0 failed. Partial
  declaration, metadata-free execution result, and relabeled value-evidence tests
  were observed red before implementation and green afterward.
- `cargo test -p sea-forge-capability --locked`: 22 passed, 0 failed.
- `cargo test -p sea-forge-cli --test conformance_m8_artifact --locked`: 3 passed,
  0 failed.
- `cargo test -p sea-forge-authority --locked`: 36 passed, 0 failed.
- `git diff --check`: passed.
- `just proof`: P1–P4b passed.
- `just no-async-kernel`: passed.
- `just context-check`: passed.
- `devbox run -- just check`: all gates green.
- Continuation dependency step 2: `cargo test -p sea-forge-authority --locked`
  passed (39 tests); `cargo test -p sea-forge-cli --test conformance_m3 --locked`
  passed (5 tests); targeted authority/CLI clippy with all targets/features and
  `-D warnings` passed; `cargo fmt --all -- --check` passed.
- Continuation steps 3–4: `cargo test -p sea-forge-artifact-ip --locked` passed
  (29 tests); CLI M3 and M8 passed (5 tests each); planner passed (28 tests);
  authority passed (39 tests); workspace all-target/all-feature clippy with
  `-D warnings` and `cargo fmt --all -- --check` passed.
- Continuation steps 7–8: settlement passed (9 tests), authority passed (40 tests),
  artifact passed (29 tests), CLI M3 passed (5 tests), and CLI M8 passed (5 tests).
  Strict workspace all-target/all-feature clippy with `-D warnings` and formatting
  check passed. RED was observed first for missing continuation compilation, then
  for a run-workspace grant mismatch; both became green after implementation.
- Final repository gates: `devbox run -- just context-check`, `devbox run -- just
  check`, and `devbox run -- just test` all passed. Cargo-deny reported only the
  existing non-fatal duplicate/unmatched-allowance warnings.
- Final domain-blocker RED/GREEN: `cargo test -p sea-forge-artifact-ip --locked`
  first failed on the missing typed-anchor/lifecycle API, then passed 39 tests.
- `cargo test -p sea-forge-cli --test conformance_m8_artifact --locked`: passed 5
  tests after diagnosing and fixing the fixture's missing declared concept class.
- `cargo test -p sea-forge-authority --locked`: passed 40 tests.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`,
  `cargo fmt --all -- --check`, and `git diff --check`: passed.
- Final requested `just proof` was run once: P1–P4b passed.
- Task 17 full gate through the context check: `cargo fmt --all -- --check`,
  strict workspace clippy, workspace all-feature tests, and `just proof` passed;
  `devbox run -- just check` then stopped only because this status update was due.
- After the status refresh, `devbox run -- just context-check`,
  `devbox run -- just check`, and `devbox run -- just test` all passed.
- After final review hardening, formatting, strict workspace clippy, workspace
  all-feature tests, and `just proof` passed again.
- Final `devbox run -- just context-check`, `devbox run -- just check`, and
  `devbox run -- just test` passed after review hardening.
- After ledger-verification hardening, formatting, strict workspace clippy,
  workspace all-feature tests, and `just proof` passed again.
- Final `devbox run -- just context-check`, `devbox run -- just check`, and
  `devbox run -- just test` passed after ledger-verification hardening.

## Blockers

- **Task 0.2 (M9 contract delta) — awaiting owner approval.** Seven additive
  changes; the only one with a real compatibility tradeoff is the closed
  `ProjectionKind` enum gaining `Kg` + `SelfModelSnapshot` (old readers reject
  records carrying the new variants; recommended policy: schema-tag the new
  self-model records `self_model.v1`, no global 0.2→0.3 bump). See the approval
  package in chat / this section.
- M10–M16 contract items (OriginRefKind::DesiredOutcome, ItemKind::AgentTask,
  Operation::AgentTask, self_disclosure/external_api/secret_access surfaces,
  ManagerIteration, settlement bases, cancellation records, authorship
  provenance) are inventoried but do NOT block Task 1; approval requested per
  task when each milestone enters scope.
- M12/M16 dependency selection (HTTP client, async strategy, URL, zeroization,
  ACP client) is blocked on explicit approval of exact versions/features; not
  needed for Task 1 (M9 declares no new dependencies).
- M13 transcript evidence uses the Task 0.4 decision (owner-accepted
  2026-07-17): retain a sealed, encrypted canonical transcript for `summarized`
  mode, verify it before crypto-shredding, expose only the deterministic summary
  by default. Canonically recorded in `spec-agent-orchestration.md` "Resolved
  decisions"; `OPEN_QUESTIONS.md` entry retired. Gates M13, not M9.

## Task 0.1 — Baseline re-run (2026-07-16)

Cumulative gate on `b351c95`, branch `full-spec`, fresh worktree:
- `devbox run -- just context-check` — passed.
- `devbox run -- just check` — passed (fmt-check, clippy `-D warnings`
  workspace/all-targets/all-features, typecheck, security). cargo-deny emitted
  only the known non-fatal getrandom 0.2/0.3 duplicate (transitive via
  domainforge-core 0.13.0); no fatal advisories.
- `devbox run -- just test` — passed (full `cargo test --workspace
  --all-features --locked`, exit 0; prior CI-green record = 289 tests; no
  platform skips).
- `devbox run -- just proof` — P1–P4b passed.
- `devbox run -- just no-async-kernel` — "ok: no tokio in kernel crates".
- Tracked `.sea-forge/**`: 0 files (confirmed via `git ls-files`).

## M9 progress (Task 1)

- Slice 1.2a (commit): added `ProjectionKind::{Kg, SelfModelSnapshot}` (appended
  to preserve existing Ord), `ForgeError::SelfModel` (class `self_model_error`),
  and `ids::snapshot_id()` (`smsnap_`). Compatibility boundary proven by 11 new
  tests: `crates/sea-forge-core/tests/projection_kind_boundary.rs` (serde, 6),
  `crates/sea-forge-ledger/tests/projection_boundary.rs` (record_kind skip +
  verify-immune, 3), `crates/sea-forge-cell/tests/projection_bundle_boundary.rs`
  (E6 hash import, 2). No global schema bump; M0–M8 records/readers unchanged.
  `cargo fmt`, clippy (`-D warnings`, 5 affected crates), workspace check, and
  affected-crate tests all green.
- Slice 1.1 (commit): added release-owned model assets `models/seaforge-system@0.1.0.sea`
  (canonical Genesis self-model, namespace `godspeed.seaforge.system`, 90
  concepts) and `models/adlc-odi-case@0.1.0.sea` (from the seed, re-versioned
  to 0.1.0, 131 concepts). Both validate through `load_validate`. Asset sha256
  (pinned release constants for slice 1.3): system=
  `09ead9ac9514009d60de0da18d936b82aa6a94a86db94c00dde7f3da54b77cfa`; adlc=
  `8c891cff33fe7bb012ebb0fda6232bc8e996d5a1af47e9c5a675f2cc7a143613`; original
  seed (spec-cited) =   `2ea06fc9c59d28fac9b9d47f18c4787b2ffb740b0527513a70556cf91dcbffc5`.
- Slice 1.2b+1.3 (commit): added synchronous kernel crate
  `sea-forge-self-model` (workspace member; added to `no-async-kernel`
  inventory). Provides `BundledModels` + `verify_bundled` (byte-check against
  pinned release sha256 before validation), `load_composed` (validates both
  models through DomainForge), `ComposedModel` (typed read-only concept lookup,
  no raw graph), `ReleaseRealization` (deterministic via SOURCE_DATE_EPOCH),
  `CellRealization` (+ `ToolchainProbe`/`ProbeResult`), `SelfModelSnapshot`
  (five digest fields + `snapshot_hash`), and canonical (jcs-nfc-v1-aligned)
  hashing. T9.1 + T9.2 green (lib 4 + conformance 2); clippy/fmt clean;
  no-async-kernel green.
- Slice 1.4 (commit): added `build_cell_realization` — assembles a cell
  realization from registry state, environment contracts, and evidenced probe
  results; a missing/failed/unverifiable probe records `Unavailable` + evidence
  ref and lists the tool under `degraded_components` (caps status, never
  elevates, never crashes snapshot creation). No probe state is cached between
  builds (V5). Probe *execution* stays in the CLI/server layer (slice 1.5); the
  crate only consumes evidenced results. T9.5 + V5 green.
- Slice 1.5a (commit): KG/CALM/JSON self-projections + persistence/lifecycle.
  `domainforge::project` gained a `Kg` arm (Turtle KG). `sea-forge-self-model`
  gained `projections` (project_self → 3 ProjectionRecords with deterministic
  rebuild_hash + verify_projection; byte-identical across rebuilds modulo
  created_at) and `store` (manifest-gated init/upgrade/rebuild, immutable
  snapshot files, rebuildable projections, mark_current_stale without mutating
  snapshots, validate). T9.3 (projection determinism + drift rejection) and
  T9.4 (extension-disable rebuild keeps prior snapshot verifiable) green; init
  idempotency green. CLI wiring (validate/rebuild/show) is the next sub-slice.
- Slice 1.5b (commit): CLI `sea-forge self-model validate|rebuild [--probe]
  [--capability-hash]|show [--json]` wired through `commands::self_model` over
  the store. domainforge Kg output renamed to `model.ttl` (no double nesting).
  CLI integration test (binary spawn) green: rebuild→validate→show--json
  round-trip, distinct snapshot on re-rebuild, and corrupt-snapshot ⇒ exit 1
  `self_model_error`.

## M9 gate (2026-07-17) — GREEN

Cumulative gate on `full-spec` after all M9 slices:
- `devbox run -- just context-check` — passed.
- `devbox run -- just check` — passed (fmt, clippy `-D warnings`
  workspace/all-targets/all-features, typecheck, security).
- `devbox run -- just test` — passed; **313 tests, 0 failed, 0 platform skips**
  (baseline was 289; +24 new M9 tests: 11 ProjectionKind boundary + 4
  self-model lib + 7 conformance_m9 [T9.1–T9.5, V5] + 2 CLI integration).
- `devbox run -- just proof` — P1–P4b passed (unchanged).
- `devbox run -- just no-async-kernel` — "ok: no tokio in kernel crates"
  (sea-forge-self-model included in the inventory).
- Tracked `.sea-forge/**`: still 0 files.
- T9.1–T9.5 + V5 all green. M9 is code-complete and gated. Remaining: M10–M16.

## Decisions

- Pipeline moved to `sea-forge-cli` (not kept in `sea-forge-core`) because keeping
  it in core would create a circular dependency once authority/runtime/sandbox
  moved to their own crates. This matches the plan’s allowance: “pipeline.rs stays
  in core or moves to cli — keep wherever the diff is smallest.”
- `sea-forge-extension` starts with a minimal lib.rs re-exporting descriptor
  types from `sea-forge-core`; it will gain registry logic in Task 4 (M0d).
- The `timed_out_child_pid_is_no_longer_alive` test’s child-process test-name
  argument was updated to match the new crate-local test path; this is a required
  mechanical reference update, not a logic or proof change.
