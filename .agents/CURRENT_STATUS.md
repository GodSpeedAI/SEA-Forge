# Current Status

Updated: 2026-07-24

> **2026-07-24 Workbench implementation skill + plan created (no production
> code).** New reusable skill `.agents/skills/building-sea-forge-workbench/`
> (SKILL.md + 8 reference files + deterministic `scripts/validate-skill.py`,
> passing, teeth-checked + 4 evaluations) and repository-grounded plan
> `.agents/plans/2026-07-24-sea-forge-workbench-frontend-api-implementation.md`
> (14 vertical-settlement tasks, capability-delta table, 12 proof scenarios).
> Grounding findings: server = Unix-socket NDJSON `verb`-tagged enum
> (`sea-forge-server/src/lib.rs:396`); no JS workspace/Tauri/schema-gen
> anywhere; SFWP envelopes, request recovery, and `events.subscribe` (cursor
> = ledger `entry_ulid`) are additive work. Frontend spec package staleness
> fixed in place: `css.txt` → `colors_and_type.css` (matches all refs),
> absent `preview/`, `assets/README.md`, `context/provenance.md` references
> corrected in README/SKILL/DESIGN/app README; generated
> `ui_kits/DESIGN-MANIFEST.json` screen-misclassification documented (not
> hand-edited) in the skill's `reference/source-map.md`. Next step: plan
> Task 1 (SFWP method-grounding report).

> **2026-07-24 spec-audit-remediation Task 19 portable closeout complete**
> (`.agents/plans/2026-07-22-spec-audit-remediation.md`): every focused gate
> from Tasks 1–18 was re-run in dependency order against `ca11dc2`; all
> selected portable tests passed. The Task 9 gate initially selected zero
> tests because its `prerequisite` filter matched no current test name, so
> the plan now uses `predecessor`; the corrected gate selects 10 predecessor
> tests and the complete `sea-forge-spec-pipeline` suite remains green.
> Fresh cumulative results: `devbox run -- just test` passed the workspace
> all-features suite; `devbox run -- just check` passed context, formatting,
> clippy `-D warnings`, typecheck, dependency-policy, and secret-scan gates;
> `devbox run -- just proof` passed minimum P1–P4b; and `devbox run -- just
> no-async-kernel` passed for all 19 kernel crates.
> Linux Landlock connect/bind denial and explicit-grant tests passed. The
> Seatbelt/macOS case was skipped on Linux, and the real ACP and real
> SWE_SEED release tests were not run because their operator-supplied
> environment/host configuration is absent; those three platform/real-host
> claims remain unproved. No matching open entry exists in
> `.agents/OBSERVED_DEBT.md`. The independent correctness/fail-closed/schema/
> dependency/test-teeth review found no remediation blocker; it recorded one
> unrelated historical Markdown-whitespace issue in `OBSERVED_DEBT.md`.

> **2026-07-24 spec-audit-remediation Task 18 landed**
> (`.agents/plans/2026-07-22-spec-audit-remediation.md`): late SWE_SEED
> declaration reconciliation closes the last documented M16 gap. New
> `sea-forge-server::swe_seed_reconciliation` is a single pure, idempotent
> join: `reconcile_swe_seed_declarations(root, case_id)` reads every
> `agent_task_evidence` record carrying non-empty `harvested_refs` in a case
> ledger, joins it against every ledger-verified `settlement_declaration`
> whose `claim_manifest_sha256` matches an independently-recomputed manifest
> hash over `(case_id, run_id, plan_item_id, settlement_id,
> transcript_sha256, harvested_refs)` (`swe_seed_claim_manifest_sha256` —
> wrong run/verifier/hash therefore never correlates), and commits one
> `swe_seed_correlation` ledger record plus the `runs/<run_id>/swe-seed-
> correlation.json` view per *distinct* resulting declaration set
> (idempotency-keyed on the declaration-id set itself, so an unchanged run
> commits nothing new and matching declarations appear exactly once).
> `delegation.rs`'s inline one-shot correlation construction and its
> snapshot-only `swe_seed_declarations_for_run` helper are replaced by calls
> into this shared reconciler. A new `submit_swe_seed_declaration` in
> `delegation.rs` is the previously-absent production declaration ingress:
> after settlement and harvested evidence are committed, it loads the
> policy's `strong_settlement_authority()` descriptor, builds the
> `SettlementDeclarationRequest` only from already-persisted criteria/
> evidence, and calls `CommandSweSeedTransport`/`SweSeedSettlementAuthority`
> through `tokio::task::spawn_blocking`; an unavailable authority surfaces as
> `settlement_authority_unavailable` and is logged, never locally
> faked — the already-committed settlement is never mutated, and a later
> out-of-process declaration still reconciles. `append_and_reconcile_swe_seed_declaration`
> wraps `sea_forge_settlement::append_declaration_ledgered_once` with an
> immediate reconcile call, used both by the production path and by an
> out-of-process actor appending directly while the server is absent.
> `reconcile_all_cases` runs at `ServerState::new` (alongside the existing
> `recover_cancelled_delegations`), and `verify_swe_seed_completion(root,
> run_id)` reconciles before reporting a run's harvested/declared status —
> the two read-time/startup triggers required by the plan. New tests:
> `sea-forge-server` `swe_seed_reconciliation.rs` unit suite (mismatched
> run/verifier/hash never correlate, declaration-before-evidence correlates
> once evidence lands, repeated reconcile is idempotent, two runs in one case
> correlate independently); `conformance_m16.rs` `t16_8_*` (server-owned
> declaration correlates immediately through a real `CommandSweSeedTransport`
> subprocess — `sea-forge-cli`'s existing hidden `internal-test-swe-seed`
> double, located next to the test binary rather than duplicating a second
> transport fake; a declaration appended directly via
> `append_declaration_ledgered_once` while no server is running reconciles
> on the next `ServerState::new` startup; the same late declaration
> reconciles at read time via `verify_swe_seed_completion` with no restart);
> `sea-forge-settlement` `declaration.rs`
> (`swe_seed_duplicate_declare_for_same_claim_conflicts_on_append`: two
> independent `declare()` calls for the same claim mint different
> `declaration_id`s, and appending both is rejected by
> `append_declaration_ledgered_once`'s idempotency-key conflict check, not
> silently duplicated). Gates green: `cargo test -p sea-forge-server --test
> conformance_m16 swe_seed -- --nocapture`, `cargo test -p sea-forge-settlement
> swe_seed -- --nocapture`, `cargo test -p sea-forge-server
> swe_seed_reconciliation -- --nocapture`. `cargo fmt --all -- --check` and
> `cargo clippy --workspace --all-targets --all-features --locked -- -D
> warnings` are clean; `cargo test --workspace --all-features --locked` (91
> test binaries, 0 failures), `devbox run -- just check`, `devbox run -- just
> proof` (P1-P4b), and `devbox run -- just no-async-kernel` (19 kernel
> crates — `sea-forge-server` is not one) are all green. Real SWE_SEED/ACP
> host release tests remain intentionally ignored pending operator
> configuration.

> **2026-07-24 spec-audit-remediation Task 17 landed**
> (`.agents/plans/2026-07-22-spec-audit-remediation.md`): built-in topology
> templates and the Thoth manager loop no longer name unregisterable
> endpoints. `sequential_agents_template`/`concurrent_agents_template`
> (`sea-forge-planner::templates`) now take a caller-supplied `endpoint_ref`
> and return `Result<PlanTemplate, ForgeError>`, validating it against the
> same `^[a-z0-9_-]{1,64}$` grammar `AgentEndpointConfig::validate` enforces
> — the old literal `agent:builtin`/`agent:default` values fail this check
> (they contain `:`), so a restored colon ID now fails template construction
> instead of only failing dispatch preflight. `store_builtin` threads a
> `default_endpoint_ref` parameter through to both templates (a new
> `DEFAULT_TOPOLOGY_ENDPOINT_REF` constant covers the CLI's criteria-only
> materialization call site, which never dispatches). Both templates' generated
> `AgentTask` branches/steps and the concurrent rollup milestone are now
> `markers.required = true` — previously `ItemMarkers::default()` left every
> item optional, so `can_auto_complete` completed the case before any branch
> ever dispatched, which is why no test had ever proven real end-to-end
> dispatch through these templates. The Thoth manager loop
> (`sea-forge-cli::commands::manager`) now requires an explicit `--endpoint`
> (never invented/auto-routed) for its synthesized proposal, and binds the
> caller-requested `max_manager_iterations` into the canonical
> `manager_iteration` authority action's parameters so the ledgered decision
> reflects what was actually asked (differing requests now produce differing
> `action_request_hash` values). A new `ActionGrant::max_manager_iterations`
> accessor (`sea-forge-authority`, mirroring the existing
> `network_tcp_ports` fail-narrow pattern) reads an authority-granted
> `max_manager_iterations` boundary constraint (added to the boundary
> dimension allowlist); `manager::iterate` now enforces
> `min(requested, grant_cap)` — a policy-granted cap always wins over a
> larger caller or default-value request, never the reverse. New/extended
> tests: `sea-forge-planner` `conformance_m14.rs` (`t17_0_*`: colon/empty/
> oversized endpoint_ref rejection, caller-supplied endpoint_ref binding);
> `sea-forge-cli` `conformance_m15.rs` (`t17_1`-`t17_5`: caller-above-grant,
> config-default-above-grant, authority decision hash changes with the
> requested cap, exact exhaustion at the grant cap, no further proposal
> after park); a new `sea-forge-server` `conformance_topology.rs`
> (`topology_*`: sequential steps dispatch through the real server in order
> against a stub endpoint and settle accepted; concurrent branches all
> accept and the rollup milestone fires; one required branch's real episode
> settling rejected terminates the case with `blocking_item` and the rollup
> never fires). Gate commands (`cargo test -p sea-forge-planner --test
> conformance_m14`, `-p sea-forge-cli --test conformance_m15`, `-p
> sea-forge-server topology`) all green, plus full workspace
> `cargo fmt --all -- --check` / `cargo clippy --workspace --all-targets -- -D
> warnings` / `cargo test --workspace --all-features` (91 green test
> binaries) / `devbox run -- just check` / `just proof` /
> `just no-async-kernel`.

> **2026-07-23 spec-audit-remediation Tasks 11-12 landed**
> (`.agents/plans/2026-07-22-spec-audit-remediation.md`): self-model rebuild
> is now ledgered end-to-end — `sea-forge-self-model::store::rebuild` opens a
> `self-model` `LedgerStream`, commits validation evidence
> (`self_model_verification_evidence`), a real `AuthorityDecision` for the
> `self_model_rebuild` reserved action, and a `SettlementEvent`, then threads
> those refs into every `ProjectionRecord` (`authority_refs`/`evidence_refs`/
> `settlement_ref` non-empty; a projection missing either evidence or
> settlement is `Quarantined`, never `Accepted`); release/cell realizations
> and the immutable snapshot are committed+materialized through the ledger
> instead of raw file writes, and `store::validate` verifies the ledger's hash
> chain when one exists. The CLI (`sea-forge self-model rebuild`) now reads
> real installation state — the extension registry, host-probed sandbox
> classes, and a real hash over `capabilities.jsonl` — instead of
> caller-supplied empty/placeholder inputs; `--capability-hash` was replaced
> by `--actor` (default `operator_local`), matching other governed commands.
> ODI provenance (M10) no longer ships `sha256:placeholder`/`outcome:primary`:
> `odi_adlc_case_template` takes real `seed_domain_model_ref`/
> `seed_model_sha256` parameters, its origin ref now names the real
> `"Desired Outcome Criterion"` concept, and a new `SeedModelResolver`
> (`sea-forge-planner::criteria`) verifies domain_model_ref/hash/concept
> membership/desired-outcome-class before authority. Both production plan
> ingresses (`pipeline.rs`'s intent path and `plan_pipeline.rs`'s externally
> supplied `run --plan`) now call `verify_plan_criteria_with_resolver` with a
> resolver built from Task 11's real bundled self-model seed instead of the
> fail-closed `NoModelResolver` wrapper; a submitted plan naming a known
> built-in template (`is_built_in_template_ref`) installs it through the
> existing source-owned installer (`store_builtin`, pinned under
> `<root>/templates/`) and derives criteria via `derive_from_template` —
> previously unreachable from production — instead of intent-only
> provenance. New/extended tests: `sea-forge-self-model` conformance_m9 (T9.1,
> T9.3 governance-ref assertions), `sea-forge-cli` `self_model_cli.rs`
> (ledgered-record-kinds + governance-ref assertions),
> `sea-forge-planner` `criteria_provenance.rs` (`m10_seed_resolver_*`: valid,
> missing, wrong-class, unknown-concept, model-drift, unrecognized-model),
> `sea-forge-planner` `conformance_m10.rs` (T10.3 placeholder-absence
> assertions), and a new `sea-forge-cli` `conformance_m10.rs` (T10.6 real
> production plan resolving the real seed hash end-to-end; T10.4 a tampered
> pinned built-in template rejected before authority with no allocated run).
> `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets`
> are clean; full affected-crate test suites
> (`sea-forge-self-model`, `sea-forge-planner`, `sea-forge-cli`,
> `sea-forge-server --test conformance_m12`) are green.

> **2026-07-23 spec-audit-remediation Tasks 13-13B landed**
> (`.agents/plans/2026-07-22-spec-audit-remediation.md`): Thoth claim
> derivation (`sea-forge-thoth::engine`) is now bounded and total. A partial
> disclosure grant (e.g. only `DeclaredCapability` permitted) no longer lets
> `derive_claims` emit a stronger natural class/status than granted —
> `capped_capability_claim` picks the natural class when it's permitted, else
> downgrades to the highest permitted class *below* it and caps `status` to
> that class's evidence rung (never elevates). Every `QuestionKind` now has a
> deterministic typed handler (`AskOperationRequirements`,
> `AskAuthorityRequirements`, `AskFailureExplanation`, `AskEvidenceForClaim`
> previously fell through to an empty `_ => {}` arm); each always emits a
> claim (real or a typed `Unsupported` claim) so `Answered` never carries an
> unexplained empty claim set. `ask_why_denied` now requires and resolves a
> verified `RecordedAuthorityDecision` (new minimal `SnapshotView` method,
> default `None`) — unresolvable ⇒ `Denied`; resolved ⇒ discloses only the
> recorded `denied_classes`/`reason_code`, never fresh claims.
> `freshness_of`'s broken placeholder (both branches returned `Stale`
> regardless of `requires_fresh()`) is replaced by an explicit pre-query gate
> in `answer_question`: a required-fresh policy against a stale snapshot
> returns `Denied` before `derive_claims` calls any bounded query method
> (`capability`/`declared_capabilities`/`environment_status` — verified by a
> `CountingSnapshot` test double asserting zero calls). Task 13B adds
> immutable Thoth-claim authorship SoD at both authority-bearing boundaries:
> `GroundedClaim.authored_by` is now stamped `Some("thoth")`
> (`engine::THOTH_ACTOR_ID`) on every claim Thoth constructs; the shared
> predicate moved from `sea-forge-thoth`'s unit-test-only `check_sod` to a new
> public `sea_forge_core::types::validate_claim_authorship_sod` (approved per
> ADR-003); `SettlementDeclarationRequest`/`SettlementDeclaration` gained an
> `authored_by` field (input carried through to the persisted record, part of
> `declaration_hash`) and `sea-forge-settlement`'s `check_integrity` denies
> before acceptance when the declarer's `actor_id` matches the claim's
> `authored_by`; `sea-forge-capability::promotion::declaration_qualifies`
> independently re-checks the same invariant reading straight from the
> persisted `SettlementDeclaration` — so a declaration record copied or
> replayed directly into the promotion pipeline (bypassing `declare()`
> entirely) still can't qualify. New tests: `sea-forge-thoth` `engine.rs`
> (`t13_1_*` capping, `t13_2_*` unsupported-under-each-grant, `t13_3_*`
> per-kind totality, `t13_4_*` ask_why_denied resolve/deny,
> `t13_5_*` pre-query freshness refusal, `t13b_claims_are_stamped_with_thoth_authorship`),
> `sea-forge-settlement` `criteria_provenance.rs` (`thoth_sod_*`: same-author
> deny, different-author allow, copied/relabeled/replayed claim),
> `sea-forge-capability` `conformance_m4a.rs` (`thoth_sod_*`: same at the
> promotion boundary). Gates green:
> `cargo test -p sea-forge-thoth -- --nocapture`,
> `cargo test -p sea-forge-settlement thoth_sod -- --nocapture`,
> `cargo test -p sea-forge-capability thoth_sod -- --nocapture`,
> `cargo test -p sea-forge-thoth t11_7 -- --nocapture`. `cargo fmt --all` and
> `cargo clippy --workspace --all-targets -- -D warnings` are clean; full
> `cargo test --workspace --all-features` is green workspace-wide.

> **2026-07-24 spec-audit-remediation Tasks 14A-14B landed**
> (`.agents/plans/2026-07-22-spec-audit-remediation.md`): Thoth now has one
> real, joined, mediated ask service instead of caller-supplied status. New
> `sea-forge-thoth::service` (`LedgerSnapshotView` + `pub fn ask`) implements
> `SnapshotView` over real state: `capability()` returns `None` only when the
> composed self-model (`ComposedModel::concept_exists`) doesn't declare the
> concept at all, otherwise rebuilds a `CapabilityRecord` purely from ledgered
> `capabilities.jsonl`/`settlement/declarations.jsonl` compatibility views
> (`build_capability_record`, tolerant of either file being absent — unlike
> `rebuild_capability`, this never writes a materialized record as a side
> effect of a read); `CapabilityStatus` maps monotonically to Thoth's
> `ClaimStatus` (`Proven`/`Metabolized → Demonstrated`, `Demonstrated →
> Validated`, `Attempted → Declared` — never over-claims). `environment_status`
> reads the real cell realization's evidenced toolchain probes;
> `declared_capabilities` is the real composed model's concept list.
> `recorded_authority_decision` resolves a verified, previously committed
> `self_disclosure_decision` ledger entry (new `Serialize`/`Deserialize` on
> `RecordedAuthorityDecision`) instead of a test double. The `DisclosurePolicy`
> is derived from the authority bundle's `self_disclosure` surface via a new
> `SelfDisclosureSurface::matching_grant` (additive helper, no schema change);
> `requires_fresh()` is true if any grant this actor holds sets
> `require_fresh_snapshot: true` (conservative — never widens disclosure).
> `ask()` validates purpose length (≤500), non-empty typed subject/actor,
> non-empty case_id-when-given, generates ULID-backed question/answer IDs
> (`sea_forge_core::ids::random_id`), and commits the complete evidence chain
> to one `thoth-asks` ledger stream in order: `self_disclosure_question` →
> `self_disclosure_plan` → `self_disclosure_decision` → `self_disclosure_answer`
> (each linked via `authority_refs` to its parent). Task 14B made CLI and
> server thin adapters over this one service: `sea-forge-cli`'s `ask.rs` no
> longer defines `SelfModelSnapshotView`/`SurfacePolicy` or loads policy
> directly — it only parses `QuestionKind` (via new shared
> `sea_forge_thoth::protocol::parse_question_kind`) and formats output.
> `sea-forge-server` gained an additive `Request::Ask` variant (ADR-003
> shape (1)/(3)) dispatching through `tokio::task::spawn_blocking` to the same
> `service::ask`, mirroring the existing sandboxed-task `spawn_blocking`
> pattern; an old server sees an unrecognized `verb` tag and fails
> deserialization cleanly (never panics, never misroutes). New tests:
> `sea-forge-thoth` `conformance_m11_service.rs` (`t14a_*`: demonstrated,
> attempted-only, unavailable-environment, absent-policy-denies,
> partial-grant-caps, stale-required-refuses-before-any-query, replay-stable);
> `sea-forge-cli` `ask_cli.rs` (`ask_with_granted_policy_answers_real_capability`,
> alongside the 3 pre-existing exit-code tests, all still green);
> `sea-forge-server` `conformance_m11_ask.rs` (allowed/denied/unknown-kind,
> full ledgered lineage assertion, wire-tag round-trip, and
> `unknown_request_verb_fails_clean_not_panic` version-skew guard). Gates
> green: `cargo test -p sea-forge-thoth --test conformance_m11_service
> -- --nocapture && cargo test -p sea-forge-thoth`; `cargo test -p
> sea-forge-cli --test ask_cli -- --nocapture && cargo test -p sea-forge-server
> ask -- --nocapture && cargo test -p sea-forge-thoth`. `cargo fmt --all --
> --check` and `cargo clippy --workspace --all-targets -- -D warnings` are
> clean; `cargo test --workspace --all-features` (90 suites), `devbox run --
> just check`, `devbox run -- just proof` (P1-P4b), and `devbox run -- just
> no-async-kernel` (19 kernel crates, still synchronous) are all green.

> **2026-07-24 spec-audit-remediation Tasks 15-16 landed**
> (`.agents/plans/2026-07-22-spec-audit-remediation.md`): delegation
> settlement/schema/termination (Task 15) and retention precedence/sealed
> summarized storage (Task 16). `Operation::AgentTask.response_schema` is now
> carried end to end: `DelegationRequest` gained `response_schema: Option<&
> serde_json::Value>`; `case_dispatch.rs::execute_agent` passes the item's
> field through instead of dropping it via `..`. A minimal, explicitly-scoped
> JSON Schema subset validator
> (`sea_forge_settlement::validate_response_schema` — `type`/`enum`/
> `properties`/`required`/`items`; no full-JSON-Schema dependency is
> approved, so unrecognized keywords are not enforced rather than pretended)
> gates settlement in `delegation.rs`: a schema-invalid final output always
> settles rejected with a typed `schema_invalid` basis; a schema-valid one is
> committed as new named evidence (`response_schema_evidence` — schema hash +
> `valid` flag only, never the raw non-conforming payload). `TurnCapExceeded`
> no longer forces rejection: when the item declares a criterion (schema
> and/or `agent_output_must_contain`) and the available final output
> satisfies it, the episode settles accepted while `turn_cap_exceeded`
> always stays in the basis (no criteria declared ⇒ unchanged prior
> behavior, rejected). `case_dispatch.rs`'s case-level completion record no
> longer constructs a synthetic `basis: ["delegation_completed"]` — it reuses
> `DelegationResult.basis` (new field), the exact basis delegation already
> committed, so cancelled/turn-capped/endpoint-error/criteria-mismatch
> episodes are never mislabeled at the case level. Task 16 added
> `sea_forge_agent::TranscriptRetentionMode` (`Summarized` default/`Full`)
> with `AgentEndpointConfig.transcript_retention: Option<_>` (endpoint level)
> and `AgentConfig.transcript_retention` (global `[agent]` default), plus
> `TranscriptRetentionMode::resolve(item_override, endpoint, agent_config)`
> implementing the full spec §8.1 precedence (plan item → endpoint → global →
> summarized default), typed-erroring on an invalid item override string
> rather than silently falling back. `case_dispatch.rs::execute_agent`
> resolves this once and passes the typed mode into `DelegationRequest`
> (new `transcript_retention` field, replacing the previously-ignored
> destructure). `delegation.rs` now branches on the resolved mode: full mode
> keeps the existing public plaintext `transcript-<hash>.jsonl` artifact;
> summarized mode seals the same canonical redacted bytes with
> `XChaCha20Poly1305` (new `sea-forge-server::transcript_seal` module, ADR-002
> — fresh random key at `.sea-forge/sealed/<run_id>.key` mode 0600, `nonce ||
> ciphertext` at `transcript-<hash>.sealed`) and verifies by decrypting
> immediately; a verification failure is captured and unconditionally forces
> the settlement rejected with a typed `sealed_verification_failed` basis
> (checked before termination/criteria — it never degrades to a
> summary-only success). The redacted `transcript_sha256` is identical
> across both modes since both seal/store the same `produce_transcript`
> output. Random key/nonce bytes use `getrandom` directly (already a
> workspace dependency at the exact pinned version via `sea_forge_core::ids`)
> rather than `chacha20poly1305`'s own `aead`/`rand_core` re-export chain,
> whose `OsRng`/`RngCore` surface has churned incompatibly across versions
> and was not going to be guessed. New tests: `sea-forge-settlement`
> `response_schema_tests` (valid/enum-mismatch/missing-required/malformed-
> json/array-items); `sea-forge-agent` `config::tests` (`retention_*`:
> full precedence chain, invalid-override-typed-error, parse rejects
> unknown/empty, absent-field version-skew defaulting); `sea-forge-server`
> `transcript_seal::tests` (round-trip, ciphertext-never-contains-plaintext-
> marker, tampered/wrong-key/missing-key/missing-ciphertext all fail
> verification, crypto-shred permanently unrecoverable, restart reads durable
> disk state not memory); `sea-forge-server` `conformance_m13.rs` `t15_*`
> (schema valid/invalid with named evidence, turn-cap-with-satisfied-
> criteria accepts and retains basis, case-dispatch reuses real basis not
> synthetic — via a genuine `Request::Submit` end-to-end dispatch, the first
> in this test file) and `t16_*` (redaction digest identical across full/
> summarized modes with mode-specific artifact visibility, case-dispatch
> resolves the full item/endpoint/global/default precedence chain end to
> end, sealed-verification-failure settles rejected never summary-only
> success). Pre-existing M13/M16 tests that read the plaintext transcript
> artifact (`t13_transcript_artifact_hash_verifies` and three ACP `t16_*`
> tests sharing the `execute_fixture` helper) now explicitly request
> `TranscriptRetentionMode::Full`, since the default changed from
> unconditional-full to spec-correct summarized. Gates green: `cargo test -p
> sea-forge-server --test conformance_m13 schema -- --nocapture && cargo
> test -p sea-forge-server --test conformance_m13 turn_cap -- --nocapture &&
> cargo test -p sea-forge-server --test conformance_m13`; `cargo test -p
> sea-forge-server --test conformance_m13 retention -- --nocapture && cargo
> test -p sea-forge-server --test conformance_m13 redaction -- --nocapture`.
> `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets
> -- -D warnings` are clean; `cargo test --workspace --all-features` (90
> suites, 0 failures), `devbox run -- just check`, `devbox run -- just proof`
> (P1-P4b), and `devbox run -- just no-async-kernel` (19 kernel crates, still
> synchronous) are all green.

> **2026-07-23 review remediation landed** (commit 47608a0): addressed 30/32
> full-spec review findings across `sea-forge-cell`, `sea-forge-case-runner`,
> `sea-forge-agent`, `sea-forge-capability`, `sea-forge-domainforge`, and the
> frontend spec/ADR/CURRENT_STATUS docs. **Follow-up resolution:** review
> findings #6 and #16 are now implemented on branch
> `fix/domain-model-validation`: `load_validate` reports the documented
> `ForgeError::Plan { class: "domain_model_error" }`, post-start
> `ForgeError::Run` failures retain their existing `internal_error` trace
> class, and `MAX_IMPORT_DEPTH=16` is enforced
> from DomainForge's existing public canonical semantic-envelope import
> graph (no DomainForge API or release change). The depth traversal derives
> the unique zero-inbound canonical closure root, so normalized entry spellings
> such as `./entry.sea` cannot bypass the limit. `devbox run -- just
> context-check`, `check`, and `test` are green; the two configured real-host
> release tests remain intentionally ignored. fmt + clippy `-D warnings` +
> `test --workspace --all-features --locked` were green for the original
> 30/32 remediation.
> **Gitleaks fix:** added `.gitleaks.toml` (path allowlist for `.entire/`
> and numbered checkpoint transcript dirs) + inline `// gitleaks:allow`
> on test-fixture lines to stop false-positive regeneration under new
> commit hashes; `gitleaks detect` now reports "no leaks found".

## Objective

Close out `.agents/plans/2026-07-22-spec-audit-remediation.md` with fresh,
portable cumulative conformance evidence and claim tables that preserve
platform/real-host skips. M9–M11 + CEP-0008 adapter complete. M12 (Task 4)
COMPLETE and gated. M13 (Task 5)
COMPLETE and gated: all T13.1–T13.7 conformance rows green, including T13.2
mixed sandboxed/agent dispatch with replayable ordinals (`sea-forge-case-runner`
extraction, server-owned per-episode dispatcher, `sea-forge ledger replay`).
M14 (Task 6) COMPLETE and gated (see the M14 entry below): typed deterministic
item expansion, all-of entry-criteria rollup mode, and the built-in
`sequential_agents@0.1.0`/`concurrent_agents@0.1.0` topology templates.
M15 (Task 7) COMPLETE and gated (see the M15 entry below): deterministic
manager-loop judgment (satisfied/blocked/progressing/stalled), `ManagerIteration`
ledger record, discretionary `agent_task` proposal through the existing
add-task path, iteration-cap park+escalate through the existing approval
mechanism, and a structural SoD gate on approval resolution. M16 ACP portable
coverage is green: ACP v1 session driver, durable permission records/approvals,
planned-run cancellation, Landlock jail support, bounded transport/transcript,
continuation recovery, and SWE_SEED proof harvesting. Late SWE_SEED
declaration reconciliation (Task 18, `sea-forge-server::swe_seed_reconciliation`)
is now COMPLETE and gated: a declaration submitted by the server's own
production ingress, or appended out-of-process at any later time, correlates
to its exact run idempotently, at settlement, at server startup, and at read
time (see the Task 18 entry above). Real ACP/SWE_SEED-host release tests
remain intentionally ignored until operator configuration is available.

Historical M13 progress log (kept for context, superseded by "COMPLETE" above):
T13.2 (Task 5) in progress:
slices 5.2, 5.4, server delegation service, CLI delegate command, and
T13.4 token-budget test all landed. 420 tests pass. The M13 conformance
table now records T13.4 green and the remaining tests pending. Next:
slice 5.3's first vertical path is landed: a caller-supplied run ID can be
cancelled through the server only after a `run_cancel` authority decision and
append-only `control_request`; the provider loop makes a cancellation that
races with a response settle rejected. T13.3 remains pending its three-run,
restart, and successor-episode conformance path. Slice 5.5/T13.5 output
criterion landed: an additive `agent_output_must_contain` field on
`SettlementCriteria` rejects an agent's narrated success when the final
response lacks the required literal. T13.6 retention design landed: full
mode persists the redacted canonical transcript artifact and a test
recomputes its SHA-256 to match the recorded `transcript_sha256`. T13.1
plan-acceptance half landed: the case engine accepts `agent_task` items
and a downstream sentry gates on the agent's settlement status; full
execution routing still pending. T13.2 semaphore cap landed: 4 concurrent
agent delegations with `max_concurrent_runs=2` never exceed 2 in-flight
connections. T13.7 hostile tool requests landed: `AgentToolCall` type
models tool/function calls in provider responses; the delegation loop
unconditionally records `tool_request_denied` per call, no side effects,
dialogue settles on criteria. T13.3 cancel-one-of-three landed: three
concurrent delegations, cancelling the middle one mid-flight settles it
rejected/cancelled while siblings settle accepted/completed. 433 tests pass. T13.3 HTTP
restart recovery landed: startup scans verified durable cancellation controls,
writes one rejected/cancelled terminal settlement and evidence, and a second
restart appends nothing. T13.1 full routing landed: an end-to-end CLI integration test starts
the server, routes `agent_task` to it, emits `SettlementRecorded`, unlocks
the downstream sandboxed task, and completes the case. Next: M13 review and
any deferred ACP continuation work belongs to M16.

Repository search and scoped Understand Anything guidance is staged for its
own documentation commit. Next implementation gap: T13.2 mixed sandboxed and
agent load with replayable dispatch/settlement ordering.

T13.2 design approved: replace the whole-case `Request::Submit` subprocess
path with server-owned, per-episode dispatch under the existing semaphore;
persist dispatch and settlement ordinals and add ledger replay. Implementation
 plan: `.agents/plans/2026-07-20-m13-mixed-dispatch.md`.

T13.2 implementation complete through Task 4: case-runner extraction (Task 1),
delegation episode context (Task 2), server-owned per-episode dispatcher under
the sole shared semaphore with dispatched-failure settlement, human-task
drain, and stale-action re-derivation (Task 3), and additive
dispatch/settlement ordinals plus `sea-forge ledger replay --case` (Task 4).
CLI and server conformance pass. Task 4 review nit fixed (replay rustdoc).
Next: full workspace proof, final whole-branch review, and graph refresh.

T13.2 Task 1 extracted synchronous case lifecycle primitives into the approved
`sea-forge-case-runner` workspace crate; the CLI is a compatibility facade and
focused CLI conformance plus runner check pass. Task 2 binds delegation to an
episode context: planned settlements now retain their submitted case/item/run,
while direct `Request::Delegate` continues to create its standalone context.
Focused planned-context and cancellation regressions pass. Next: add the
server-owned episode dispatcher. Task 2 review follow-up preserves planned
case/item/run identity in restart-recovered evidence and settlement, validates
the cancellation control against the persisted plan, and proves one target
settlement despite unrelated prior settlement records. The final Task 2 review
fix also serializes planned case/item/run identity in normal and rejected
delegation evidence; planned success and rejection conformance assertions pass.

Task 3 dispatcher landed: `Request::Submit` now creates the case in-server and
dispatches sandboxed and agent episodes under the existing `ServerState.semaphore`.
Each dispatch is committed before its episode starts; a completed episode is
settled and applied before the reducer derives more ready work. T13.2's five-item
mixed-load conformance path passes with `max_concurrent_runs=2`, alongside all 13
server M13 conformance tests. `devbox run -- just context-check` passed after this
status update. Commit: recorded in git history.

Task 3 review follow-up: the dispatcher now waits on the sole shared permit,
prioritizes recording returned active completions, converts post-dispatch errors
into terminal rejected settlements, and drains active siblings before case
termination. Two focused regressions plus all 15 server M13 conformance tests,
formatting, and server check pass. Commit: pending.

Task 3 human-task review follow-up: ready human tasks now persist their normal
activation event and drain already-dispatched episodes to terminal settlement
before `Submit` returns active. All 16 server M13 conformance tests, formatting,
and server check pass. Commit: pending.

Task 3 final review follow-up: human holds now process every equally-ready
executable action in either plan order before draining settlements, and a
completion during permit wait restarts case reduction rather than processing
stale actions. All 17 server M13 conformance tests, formatting, and server check
pass. Commit: pending.

Task 3 non-executable activation follow-up: the dispatcher now rejects
`CaseAction::Activate` for item kinds it cannot dispatch (Stage,
TimerListener, UserEventListener) with a typed `ForgeError::Input` before
any side effect, instead of panicking in the activation payload. New
`t13_2_non_executable_activation_returns_typed_error` regression; all 18
server M13 conformance tests, formatting, and server check pass.
Commit: pending.

Task 4 review follow-up: dropped the completion-theater reducer invocation
in `commands::ledger::replay_case` that silently swallowed plan-load and
reducer errors (replay now trusts `case-events.jsonl`); added three
focused unit tests for `validate_ordinals` rejection paths (missing
dispatch_ordinal on ItemActivated, duplicate equal dispatch_ordinal,
non-monotonic lower settlement_ordinal) plus an accepts-monotonic
control, each asserting `ForgeError::Input`; spec-agent-orchestration.md
T13.2 status row flipped from "partial" to green, citing
`t13_2_mixed_episodes_share_server_cap`, `t13_2_replay_matches_persisted_order`,
and `sea-forge ledger replay --case`, with a note that replay applies
only to cases created after the additive ordinal change (no migration).
All 3 CLI M13 conformance tests, 18 server M13 conformance tests, the
4 new ledger unit tests, fmt, and `cargo check` on CLI+server+case-runner
pass. Commit: pending.

T13.2 final whole-branch review follow-up (three fixes):

- `sea-forge-case-runner` added to the `no-async-kernel` `kernel_crates`
  array; `just no-async-kernel` now covers 19 kernel crates (was 18).
- `SettlementRecorded` payload in `case_dispatch::record_completion`
  now carries `run_id`, so `ledger replay --case` correlates dispatch
  to settlement by run_id; `t13_2_mixed_episodes_share_server_cap`
  extended to assert each settlement's run_id maps to a same-item
  dispatch.
- `SubmitPayload.intent` documented as ignored by server-owned dispatch
  (intent-only submit is no longer supported via `Submit`); field
  retained for deserialization compatibility.
All 18 server M13 conformance tests, 3 CLI M13 conformance tests, fmt,
and `just no-async-kernel` (19 crates) pass. Commit: pending.

## M14 topology templates (2026-07-21)

Implementation plan: `.agents/plans/2026-07-21-m14-topology-templates.md`.
M14 (Task 6 of the ADLC/Thoth orchestration plan) is COMPLETE and gated:

- Additive `EntryCriteriaMode::{Any,All}` on `PlanItem`/`TemplateItem`
  (`Any` default, byte-compatible with every pre-existing template).
  `case_engine::entry_criteria_satisfied` now supports all-of rollup gating
  alongside the unchanged OR-of-sentries default.
- Typed deterministic item expansion: `TemplatePlan.repeated: Vec<RepeatedItem>`
  expands a shared item body into N items with IDs derived from
  `{id_prefix}_{key}`, bounded by `MAX_REPEATED_ENTRIES` (32), duplicate
  entry keys rejected, forbidden-substitution sites (including the new
  `TemplateOperation::AgentTask.endpoint_ref`) enforced on the shared body
  before expansion.
- `TemplateOperation::AgentTask` closes a real prior gap — templates could
  not express `Operation::AgentTask` at all before this change.
- Built-in `sequential_agents@0.1.0` (steps chained via per-entry
  settlement-accepted sentries referencing the prior step's deterministic
  ID) and `concurrent_agents@0.1.0` (independent branches plus a flat
  `Milestone` rollup with `entry_criteria_mode: All`) registered through
  the existing `store_builtin` installer, no new CLI wiring.
- `conformance_m14.rs`: T14.1 (deterministic ×2 instantiation, chained
  gating), T14.2 (rollup fires only when all N branches settle accepted),
  T14.3 (one branch rejected plus an unrelated rejection never fires the
  rollup — proves source-binding, not leakage), plus mechanism-level unit
  tests for expansion validation and all-of semantics — 9/9 pass.
- Stub agent endpoints in the built-in templates prove scheduler/rollup
  behavior only; they do not upgrade the spec §5 "real agent latencies"
  claim, which stays open for M16 real integration.

456 workspace tests pass (`cargo test --workspace --offline`), fmt clean,
`just no-async-kernel` still covers 19 kernel crates (no new crate added —
`sea-forge-core`/`sea-forge-planner` remain sync/no-HTTP). `.agents/specs/spec-agent-orchestration.md`
§17.3 T14.1–T14.3 rows flipped to green.

## M15 Thoth manager loop (2026-07-21)

Implementation plan: `.agents/plans/2026-07-16-adlc-thoth-agent-orchestration.md`
Task 7. M15 (E16b) is COMPLETE and gated:

- `ManagerJudgment`/`ManagerAction`/`ManagerIteration` additive types on
  `sea-forge-core`; `sea_forge_thoth::manager::judge` is a pure, IO-free
  function classifying `satisfied | blocked | progressing | stalled`
  deterministically from caller-supplied facts (§9.5's rule table exactly —
  completed beats blocked beats ready-work/settlement-progress beats
  stalled), mirroring `engine.rs`'s existing `SnapshotView`-driven pattern
  rather than adding a new crate dependency.
- `crates/sea-forge-cli/src/commands/manager.rs` (`case manager-iterate`
  subcommand): builds the view from `next_case_actions` plus a
  `replay_case` scan for items already `Enabled`/`Active` from a prior
  tick (an item transitions out of `next_case_actions`'s output once
  enabled, but per §9.5 "active or enabled" still counts as progressing —
  a real gap the first test pass caught); tracks settlement progress via
  a `settlement_events_observed` counter compared against the prior
  iteration's recorded count; commits one `ManagerIteration` ledger record
  per invocation (`record_kind: "manager_iteration"`, same `case-<id>`
  ledger stream as everything else about the case).
- Additive `PlanItem.proposed_by: Option<String>` — part of the plan's
  canonical hash, so relabeling/replaying a proposed item cannot strip the
  provenance. `case.rs::add_task` refactored into a thin file-reading
  wrapper around a new `propose_item(root, policy, actor, case_id, item)`,
  reused directly by the manager loop for in-memory synthesized items
  (`item_kind: agent_task`, `proposed_by: Some(actor)`) — no new mutation
  path, no duplicated authority/ledger-commit logic.
- Denied proposals (T15.2) are caught and recorded as the iteration's
  `granted: false` outcome, never a hard error — matches §7.6 "recorded
  to the ledger whether or not the proposal is granted."
- Iteration-cap exhaustion (T15.3) is a guard *before* judging (per §16.2's
  pseudocode) — no `ManagerIteration` record for the cap-exceeded call
  itself, reuses the exact `ApprovalRequest` + `CaseState::AwaitingApproval`
  pattern `plan_pipeline.rs` already uses for settlement escalation.
- SoD (T15.4): `approve.rs::resolve()` gained a `proposed_by`-based check,
  independent of and prior to the existing requester-based check — an
  actor cannot resolve an approval for a plan item it proposed.
- Two real, previously-latent authority-layer gaps found and fixed along
  the way (both are additive allowlist entries, not behavior changes for
  existing kinds): `sea-forge-authority`'s `PolicyRule.operation_kind`
  validation allowlist and its separate `malformed_action` `RESERVED`
  allowlist were both missing `manager_iteration` — every `Reserved`
  resource_type needs an entry in *both* lists or every policy decision
  for it defaults to hard deny regardless of matching rules. This also
  means `discretionary_task_add` (M10, `case add-task`) was reachable via
  CLI for the first time here — nothing else in the workspace exercised
  it end-to-end before this milestone.
- `crates/sea-forge-cli/tests/conformance_m15.rs`: T15.1–T15.5, all
  subprocess-driven through the real `sea-forge` binary (`run --plan`,
  `case manager-iterate`, `approve`) rather than in-process calls, since
  `sea-forge-cli` is bin-only (no `lib.rs`) — 5/5 pass.

465 workspace tests pass (`cargo test --workspace --offline`), fmt clean,
`just no-async-kernel` still covers 19 kernel crates (`sea-forge-cli`/
`sea-forge-authority` are not kernel crates). `.agents/specs/spec-agent-orchestration.md`
§17.4 T15.1–T15.5 rows flipped to green; the "Manager loop bounded,
evidence-grounded, SoD-enforced, escalating on exhaustion" checklist item
closed.

## M16 ACP + SWE_SEED (2026-07-21, partial)

ACP code gate is complete; SWE_SEED proof harvesting is complete; declaration
reconciliation and real-host evidence remain:

- `sea-forge-agent::acp` speaks ACP v1 JSON-RPC/NDJSON with official
  `mcpServers`, `prompt`, `sessionUpdate`, stop-reason, and capability-gated
  `session/load` shapes. Spawn uses tokenized argv, an explicit minimal env,
  shell rejection, process-group cleanup, bounded stderr drain, bounded input
  lines, and bounded cumulative transcript.
- ACP permissions commit exact hashed request data, an authority decision, a
  durable `permission_request`, and (when escalated) an ordinary
  `approval_request`. Broker wake-up only causes a ledger reload; permission
  allow requires an independently committed approved resolution and the
  existing `grant_after_approval` exact-action validation. Deny/timeout stays
  inside the session. Duplicate wake-ups are rejected.
- Server-dispatched agent tasks now register cancellation handles too. Restart
  recovery turns orphaned ACP approval episodes into one rejected
  `acp_disconnect` settlement with a continuation record; successor episodes
  call capability-gated `session/load` under the same key.
- `sandbox_class: jail` ACP children are spawned on a Landlock-restricted
  thread; portable proof denies an outside `/tmp` write while allowing the run
  workspace, and rejects a session-mode escalation.
- SWE_SEED config requires an explicit repo+commit pair. Commit verification,
  safe `.agent-harness` traversal, symlink/type/count/size/hash validation,
  run-bound harvested refs, and a `swe_seed_correlation` ledger record are
  portable-tested. M4a declarations are resolved by immutable run ID when
  present. This is not yet a reconciliation path for declarations arriving
  after episode settlement; real SWE_SEED/transport evidence remains a release
  gate.
- `crates/sea-forge-server/tests/conformance_m16.rs`: 12 portable tests pass;
  two real-host tests are ignored and reported skipped by design.

Verification: isolated full workspace `cargo test --workspace --all-features
--locked` passed (the normal target had corrupted incremental linker objects
after an interrupted build; no source artifact was deleted). Focused ACP,
M12, M13, M16 tests and clippy all passed.

## Spec-audit remediation Tasks 7-8: SQLite FTS memory index + governed recall (2026-07-22)

Implementation plan: `.agents/plans/2026-07-22-spec-audit-remediation.md`.
Task 7 and Task 8 are COMPLETE and gated:

- Task 7 replaces the prior JSON `memory/index.json` projection (a documented
  `ponytail:` compromise) with `memory/index.sqlite`: an FTS5 virtual table
  (`memory_fts`) plus a `meta` table committing a SHA-256 digest of
  `items.jsonl` at rebuild time. `crates/sea-forge-capability/src/memory.rs`
  rebuilds atomically (temp file → `PRAGMA integrity_check` → rename) and
  `query_index` returns `None` (forcing linear-scan fallback) whenever the
  index is missing, corrupt/truncated, or its stored digest no longer matches
  the current `items.jsonl` bytes — freshness is proven by a source
  commitment, never inferred from successful deserialization. Filtering stays
  identical to the linear scan (exact-match SQL pushdown for entity/process/
  kind plus a shared Rust substring filter) rather than relying on FTS5
  `MATCH` token semantics, so indexed and fallback results are provably
  identical, including mid-word substrings that a tokenizer would miss.
  14 conformance tests in `crates/sea-forge-capability/tests/conformance_m4b.rs`.
  **Dependency note:** ADR-002 originally approved `rusqlite = "0.40"`
  (**superseded**), but `libsqlite3-sys 0.38.1`'s `build.rs` unconditionally
  invokes the still-unstable `cfg_select!` macro (rust-lang/rust#115585) and
  fails to compile on this repo's pinned `rustc 1.92.0`. ADR-002 was amended
  in place to pin `rusqlite = "0.32"` (→ `libsqlite3-sys 0.30.1`, still
  FTS5-bundled, same license) — the **final approved dependency version**,
  not a technology substitution.
- Task 8 splits `crates/sea-forge-cli/src/commands/recall.rs` into two
  contracts: the legacy capability-envelope path now prints matched
  `capabilities.jsonl` envelopes completely unchanged (the prior
  `record_assurance`-injected `"assurance"` field is removed — that helper
  is now dead code and was deleted from `mediated.rs`); the `--kind`
  memory-recall path binds exact `entity_id`/`requester_entity`/`process_id`/
  `kinds`/`limit` into the `recall_memory` authority action (an omitted
  `--entity` defaults the target to the requester's own identity, so `own`
  scope authorizes implicit self-recall without an extra flag), re-applies
  the granted `memory_scope` at the executor via `sea_forge_authority::
  scope_allows` independent of the query filter (defense in depth against a
  query-layer bug), and commits a `recall_evidence` ledger record (new
  `ledgers/memory-recalls/` stream) naming the request scope and every
  returned memory ID *before* printing any result. A denial short-circuits
  inside `PolicyAuthorityEngine::grant()` before the callback that would read
  `memory/items.jsonl` ever runs, so cross-entity denial reads nothing and
  commits no evidence. Assurance moved from the removed envelope mutation to
  the governed path: computed once via `mediated::assurance()` and exposed
  through the evidence record plus a structured `tracing::info!` line (the
  pre-existing `required_integrity_checkpoint_precedes_command_start_and_
  witness_outage_halts` lifecycle test was updated to assert it there
  instead of in compat-recall stdout). 1 new lifecycle.rs test (byte-for-
  structure, no injected assurance, capabilities.jsonl untouched) plus 7
  tests in new `crates/sea-forge-cli/tests/conformance_m4b.rs` (own/cross-
  entity/`entity:<id>`/any scope, evidence-ID exactness, limit, SQLite/
  fallback CLI-level equivalence).

**Task 7-8 gate results (historical context):**
- **Pre-fix result** (`just check` snapshot immediately after Tasks 7-8 code
  landed, before the gitleaks `.gitleaksignore` baseline was refreshed):
  `cargo fmt --all -- --check` ✅; `cargo clippy --workspace --all-targets
  --all-features --locked -- -D warnings` ✅; `cargo test --workspace
  --all-features --locked` ✅; `cargo deny check` (advisories/bans/licenses/
  sources) ✅; `devbox run -- just proof` (P1-P4b) ✅; `devbox run -- just
  no-async-kernel` (19 kernel crates, `rusqlite` confined to
  `sea-forge-capability`, synchronous) ✅; but `devbox run -- just check`'s
  `security` sub-recipe FAILED on 3 pre-existing `gitleaks` findings from
  commits `63a746c3`/`aa9d65bb2` (test-fixture fake secrets in
  `sea-forge-agent`/`sea-forge-server` and a `.entire/metadata/**` session
  artifact) that predate this plan and are unrelated to Tasks 7-8; a
  fmt-only whitespace fix was also applied to the already-modified,
  unrelated `sea-forge-domainforge/tests/conformance_m0_domainforge.rs` to
  unblock `fmt-check`, with no semantic change.
- **Post-fix result** (the frozen clean baseline captured under Task 0 at
  line 1223+: `just context-check && just check && just test && just proof
  && just no-async-kernel` — all green, `gitleaks detect` reports "no leaks
  found" after the four fingerprints were re-added to `.gitleaksignore`).
  The Task 7-8 gate is therefore considered PASSED on the post-fix
  baseline; the pre-fix `security` sub-recipe failure is historical and was
  not caused by Tasks 7-8 code.

## Spec-audit remediation Tasks 9, 10A, 10B: M5 governed stage episodes + CLI project (2026-07-23)

Implementation plan: `.agents/plans/2026-07-22-spec-audit-remediation.md`.
Tasks 9, 10A, and 10B are COMPLETE and gated:

- Task 9 replaces order-only stage validation in `sea-forge-spec-pipeline`
  with canonical-chain validation: `validate_stage_prerequisites` checks
  that a stage's declared input resolves either to an earlier stage's
  matching-path output (requiring that predecessor's status to be
  `Accepted`, and its hash/`schema_ref`/`domain_model_ref`/
  `domainforge_version` to match exactly) or, when no local predecessor
  exists, to a fully self-verified externally supplied file (its own
  `schema_ref` present). Three additive `Option<String>` fields
  (`schema_ref`, `domain_model_ref`, `domainforge_version`) were added to
  `StageFile` in `sea-forge-core::types` (ADR-003-approved shape (1)
  addition; `StageFile` gained `Default` to keep every existing struct
  literal compiling). `process_pipeline` now quarantines any stage —
  including one an executor self-reports `Accepted` — whose prerequisite
  chain fails, before classification runs. `compute_proof_classification`
  no longer grants `generated-contract`/`focused-slice` from a sparse stage
  list: it requires the *entire* canonical-order prefix (`Adr..
  GeneratedContract`, then `LastMileAdapter`/`RuntimeWiring`/
  `AcceptanceProof`) to be present and `Accepted`, closing the audited
  defect where four sparse stages could claim focused-slice. 15 new tests in
  `crates/sea-forge-spec-pipeline/tests/conformance_m5.rs` (missing/
  skipped/rejected/quarantined/unhashed/schema-mismatched/model-mismatched/
  version-mismatched predecessors, valid chain, externally-supplied input,
  process_pipeline cascade) plus a reversed internal unit test proving the
  sparse-focused-slice defect is closed.
- Task 10A adds `run_stage_case`/`run_stage_episode` to
  `sea-forge-case-runner`: a synchronous driver that builds the case
  (ledger stream, case/plan JSON, `CaseCreated`), then loops
  `CaseRunner::next_ready_actions` exactly like the existing server
  dispatcher, dispatching `Activate` through the same
  authority-evaluate → ledger-commit → grant → `sea_forge_runtime::execute`
  → settle pattern used elsewhere (mirroring
  `sea-forge-server::case_dispatch::execute_sandbox`). Two structural gates
  run before authority: Task 9's `validate_stage_prerequisites` (a failure
  quarantines the stage with basis `prerequisite_invalid`, no authority
  call), and a generated-zone guard (§10.7): a non-generator-kind stage
  (i.e. not `Ast`/`Ir`/`Manifest`/`GeneratedContract`/`SemanticFixture`)
  declaring an output path under a generated zone is quarantined with basis
  `generated_zone_direct_edit`, also before authority. `sea_forge_planner`
  gained `stage_case_plan`, converting `Vec<SpecPipelineStage>` into a
  `CasePlan` of `SandboxedTask` `PlanItem`s chained by the same
  `reactivation_sentry` helper `sequential_agents_template` already uses
  (settlement-accepted sentry referencing the prior stage), each
  `markers.required: true` (an unrequired item can be silently skipped by
  `can_auto_complete` before ever activating — a real gap the first test
  pass caught) and `sandbox_class: "jail"` (a stage's command is arbitrary
  tooling, not necessarily the trusted `sea-forge` binary, so it cannot
  rely on the `local` class's trusted-argv0 policy exemption). `sea-forge-spec-pipeline`
  stays pure — no scheduling, filesystem writes, or ledger commits live
  there. 4 conformance tests in `crates/sea-forge-case-runner/tests/conformance_m5.rs`
  (accepted stage completes the case; denied generated-zone edit quarantines
  before authority; broken-prerequisite stage quarantines; a rejected
  predecessor's downstream settlement-accepted sentry never fires, leaving
  the successor `Pending` and the case `terminated`) using a self-invocation
  idiom (mirroring `sea-forge-sandbox`'s `net_probe_helper`) since
  `ExecuteCommand`'s hard `untrusted_executable` invariant only trusts the
  exact running process's own executable — no test may shell out to `sh`.
- Task 10B adds `sea-forge project <entry.sea>`: builds the ADR/PRD/SDS/SEA/
  AST/IR/Manifest/GeneratedContract stage chain with real content (authored
  doc text; the actual `.sea` source; `{:#?}` of DomainForge's parsed graph;
  the validated `DomainModelRef`; a manifest JSON; the CALM projection as
  the generated contract), each stage hash-linked to its predecessor's
  output, runs it through Task 10A's `run_stage_case`, then commits a
  `SpecPipelineRun` (via Task 9's `process_pipeline`, proving the achieved
  `proof_classification`) and independently settles a `ProjectionRecord` +
  `SettlementEvent` pair per requested `ProjectionKind` (default `calm`,
  `rdf`) from the one validated model — an unsupported kind (e.g. `sbvr`,
  which `sea_forge_domainforge::project` already rejects) is quarantined
  with a `Rejected`-status `ProjectionRecord` recording the failure reason,
  never silently dropped or accepted. New hidden `sea-forge stage-check
  <file> <sha256>` subcommand (config-free, same shape as the existing
  hidden `validate`) is the only thing a stage's `ExecuteCommand` ever runs,
  since `sea_forge_authority::untrusted_executable` requires argv[0] to be
  this exact running binary. SEA Forge (this command) performs every
  authorized filesystem write; `sea_forge_domainforge::{load_validate,
  project}` remain pure/in-memory. 2 conformance tests in
  `crates/sea-forge-cli/tests/conformance_m5.rs`: the full chain settles
  with `proof_classification=GeneratedContract` and both projections
  accepted, every expected ledger record kind present, and no unexpected
  top-level filesystem entries under root; the negative case shows a
  non-zero exit, `projections_quarantined=1`, and a `Rejected` `sbvr`
  `projection_record` in the ledger.

Full workspace gate passed: `cargo fmt --all -- --check`, `cargo clippy
--workspace --all-targets --all-features --locked -- -D warnings`, `cargo
test --workspace --all-features --locked` (all crates green), `devbox run
-- just proof` (P1-P4b), `devbox run -- just no-async-kernel` (still 19
kernel crates — no new kernel crate added; `sea-forge-spec-pipeline` and
`sea-forge-domainforge` are used only by non-kernel `sea-forge-cli` and by
already-kernel `sea-forge-planner`/`sea-forge-case-runner`, both of which
remain synchronous).

## SodRule transition scope closeout (2026-07-17)

- Added additive `SodRule.transition_kind: Option<String>` with omitted-None
  serialization for policy-hash compatibility. Validation now rejects unscoped
  or unknown `transition_artifact_stage` selectors and selectors on other
  operations.
- A single fail-closed action matcher enforces requester role, canonical action
  operation, and transition selector in both policy evaluation and
  post-approval grants. The v0.2 capitalization SOD rule now targets
  `transition_artifact_stage` / `capitalize`; non-R-SO resolution remains
  rejected.
- Proof passed: `cargo fmt --all -- --check`; `cargo check -p
  sea-forge-authority`; `cargo test -p sea-forge-authority --locked`; M8 CLI
  and artifact-IP tests; `devbox run -- just context-check`, `just check`, and
  `just test`. No tests skipped.

## Worktree State

On 2026-07-24, `full-spec` was merged into `main` as `74dc8ab` after a
fast-forward update from `origin/main`. The integrated tree passed
`devbox run -- just ci`; publication is pending the pre-push context gate after
this final status refresh. The remaining unstaged documentation correction is
the `prove_entry` source-line reference in the Workbench repository map; it
tracks the current ledger source and is intentionally included in the pending
handoff commit.

On branch `full-spec` at `ca11dc2` before the Task 19 documentation closeout.
The worktree was clean at Task 19 start. Task 19's changes are limited to the
remediation plan's corrected Task 9 test filter and Task 19 status/spec/debt
updates; no product code, dependency, persisted schema, public interface, CI,
or runtime output changed. Unrelated untracked frontend design/API files
appeared while the final gates were running; they were neither inspected nor
modified and remain preserved in the worktree.
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

- Merge handoff: this status refresh records the `main` integration and the
  current source reference in
  `.agents/skills/building-sea-forge-workbench/reference/repository-integration.md`.

- Task 19 closeout: `.agents/plans/2026-07-22-spec-audit-remediation.md`
  corrects the Task 9 zero-match focused filter and records final acceptance;
  `.agents/specs/spec-agent-orchestration.md` aligns the M12–M16 claim table
  with fresh portable evidence and explicit real-host skips;
  `.agents/OBSERVED_DEBT.md` records unrelated historical Markdown whitespace;
  this status file records the final verification and remaining release gates.

- `.agents/reports/2026-07-22-spec-implementation-audit.md` — executable-code and test-evidence audit of all four `spec-*.md` specifications.

- `spec/CEP-0008-semantic-envelope.md` — authoritative CEP-0008 source copied
  from `/home/sprime01/projects/cep/spec/` to support the SemanticEnvelope
  compatibility-debt refactor.
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

- Audited all four `spec-*.md` documents against source and executable tests; the report identifies conformance blockers in every specification and does not use documentation as evidence.

- Copied the authoritative CEP-0008 Semantic Envelope specification into the
  repository; its text matches the source, apart from adding the conventional
  trailing newline.
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

- 2026-07-22 audit: `devbox run -- just check` passed; focused conformance suites and `just proof` passed as recorded in `.agents/reports/2026-07-22-spec-implementation-audit.md`.
- 2026-07-22 audit: `devbox run -- just test` failed twice with a suite-context `SIGSEGV` before `sea-forge-cli` main-unit test output. Its isolated binary test passed (5 tests); the fault remains unresolved.

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

- Run the ignored real ACP release gate when
  `SEA_FORGE_REAL_ACP_ARGV` (and any required `SEA_FORGE_REAL_ACP_ENV`) is
  supplied by an operator.
- Run the ignored real SWE_SEED release gate when the real ACP argv/env plus
  `SEA_FORGE_REAL_SWE_SEED_REPO` and
  `SEA_FORGE_REAL_SWE_SEED_COMMIT` are supplied.
- Run the Seatbelt network conformance case on macOS. None of these skipped
  release/platform checks is claimed by the portable Task 19 closeout.
- `.agents/plans/TODO.md` defines the stronger evidence required before the
  three claims can move from unproved to proven: a jailed real ACP host, a
  real SWE_SEED host plus correlated authority declaration, and an implemented
  macOS Seatbelt backend with non-skipping conformance tests.

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

## M12 progress (Task 4 — E14 AgentProvider seam)

**M12 COMPLETE and GATED (408 tests, P1–P4b, no-async-kernel, cargo-deny).**
Base landed in af94ff0; gap closure in c6aebb6; license/spec/status update
in this change.

M12 gate (2026-07-20): `just context-check`, `just check` (fmt + clippy
`-D warnings` workspace/all-targets/all-features + cargo-deny
licenses/bans/sources), `just test` (408 tests, 0 failed, 0 platform
skips), `just proof` (P1–P4b), `just no-async-kernel` (18 kernel crates)
all green. T12.1–T12.6 green. Tracked `.sea-forge/**` still 0 files.

deny.toml: added `CDLA-Permissive-2.0` to the license allow-list —
carried by `webpki-roots` (Mozilla root CA bundle), a transitive dep of
the approved `reqwest` rustls-tls feature. Permissive license, not
copyleft; mechanical consequence of the approved M12 dependency.

spec-agent-orchestration.md: §5 claim table records M12 evidence
(AgentProvider seam, declared-config-not-status, endpoint failure
taxonomy); §17.1 T12 table annotated with status; §17.6 records the
GREEN gate; §18 checklist M12 items checked; summarized-transcript row
updated to the resolved sealed-transcript decision. `sea-forge-agent` adapter crate with
OpenAI-compatible + Anthropic providers (object-safe `AgentProvider` via
`BoxFuture`, no async-trait dep), `Operation::AgentProbe` + exact-action
`AuthorityAction::AgentProbe` (binds endpoint_ref +
descriptor_config_sha256 + normalized scheme/host/port/path/model/limits

- credential_ref + prompt_sha256 so a config reload cannot repoint an
authorized call), `external_api` surface `allow_hosts` enforcement,
DNS-rebinding-safe client pinning (`resolve_to_addrs`), `no_proxy` +
`redirect::Policy::none()` + HTTPS-only (explicit loopback test mode),
private/loopback/link-local/multicast/metadata-address rejection, separate
`secret_access` mediation before credential resolution, `Zeroizing<String>`
credential handling, immutable `runtime_adapter` endpoint registration
(descriptor change requires a new version), governed `agent_probe` service
creating intent→plan→authority→evidence→settlement with typed error
classes, and CLI `agent list|probe` over the server socket. T12.1–T12.3
green (4 server conformance tests); provider-contract tests pin request
shapes, paths, and auth headers for both provider kinds.

Verification (worktree, pre-commit of base): `cargo fmt --all -- --check`
clean; `cargo clippy --workspace --all-targets --all-features --locked
-D warnings` clean; `cargo test --workspace --all-features --locked`
**394 tests, 0 failed, 0 platform skips** (was 372 after CEP-0008; +22:
8 agent lib + 3 provider-contract + 4 server conformance_m12 + 1
authority m12 exact-action + 3 core/extension/case_engine AgentProbe +
3 agent config/network tests).

Dependencies (owner-approved per plan slice 0.3, confined to adapter
crates): `reqwest 0.12` (default-features=false, features
json+rustls-tls+stream), `url 2.5`, `zeroize 1.8`. Cargo.lock refreshed.

Remaining M12 gaps (before cumulative gate):

- T12.5 dependency-boundary gate: rewrite justfile `no-async-kernel` to an
  explicit kernel-crate inventory (add sea-forge-domainforge,
  sea-forge-spec-pipeline, sea-forge-cell, sea-forge-artifact-ip) and a
  forbidden-dependency set covering async runtimes AND HTTP clients
  (tokio, reqwest, hyper, async-std, …), excluding only approved adapter
  crates (sea-forge-agent, sea-forge-server).
- Slice 4.2 finish: `ServerConfig::load` must call `agent.validate()`
  (last-known-good on invalid); endpoint registration must mark the
  self-model snapshot stale when one exists
  (`sea_forge_self_model::store::mark_current_stale`).
- Slice 4.5 finish / T12.6: probe-level error-taxonomy tests for
  unreachable/4xx/5xx/oversize/redirect/schema-invalid → rejected
  settlement + typed `error_class` + no fallback; CLI exit code for
  rejected probe aligned to repo convention (exit 3).
- Streaming (spec §7.4 redaction + split-chunk sweep) is E15/M13 scope;
  M12 probe is non-streaming (hash-only persistence ⇒ sweep trivially
  holds). Recorded in spec §5 claim table.
- Spec §5 claim table update + cumulative gate + this status refresh.

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

## Audit Remediation Plan — Task 0 (2026-07-22)

Source: `.agents/plans/2026-07-22-spec-audit-remediation.md`, built from
`.agents/reports/2026-07-22-spec-implementation-audit-independent-validation.md`.
Task 0 freezes the clean baseline and records owner-approved dependency and
contract choices for Tasks 1-18 before any remediation code lands.

- **ADR-002** (`docs/decisions/ADR-002-audit-remediation-dependencies.md`):
  approves `rusqlite 0.32` (`bundled`, for M4b SQLite FTS — confirmed FTS5 is
  always compiled into bundled builds, no separate feature flag exists).
  **Note:** ADR-002 originally recorded `rusqlite 0.40`, but that version is
  **superseded** — `libsqlite3-sys 0.38.1`'s `build.rs` unconditionally
  invokes the still-unstable `cfg_select!` macro (rust-lang/rust#115585) and
  fails to compile on this repo's pinned `rustc 1.92.0`. ADR-002 was amended
  in place to pin `rusqlite 0.32` (→ `libsqlite3-sys 0.30.1`, still
  FTS5-bundled, same license) as the **final approved dependency version**;
  `rusqlite 0.40` must not be reintroduced. Also approves
  `landlock 0.4` (for M1 jail TCP bind/connect denial via `AccessNet`,
  replacing the hardcoded `ABI::V1`; UDP/raw-socket coverage remains outside
  Landlock's scope in every ABI through V6 — this matches `spec-full.md:1434`
  verbatim, which already scopes the requirement to "Landlock v4+ TCP
  restrictions where available, else document the gap"), and
  `chacha20poly1305 0.11` (`zeroize` feature, for M13 sealed summarized-mode
  transcripts — per-run random key at `.sea-forge/sealed/<run_id>.key`,
  XChaCha20Poly1305, crypto-shredding = deleting that key file). All three
  versions were confirmed live against Context7 and the crates.io API on
  2026-07-22, not recalled from training data. All three were presented to
  the owner as `AskUserQuestion` choices and approved as the recommended
  option in each case.
- **ADR-003** (`docs/decisions/ADR-003-audit-remediation-contracts.md`):
  inventories additive contract deltas per task. Notably, several deltas the
  plan's steps describe as "add a field" turned out to already exist in
  source (`Operation::AgentTask.{response_schema,transcript_retention}`,
  `PolicyRule.memory_scope`, the M9 self-model `ProjectionKind` variants,
  `OriginRefKind::DesiredOutcome`) — those tasks (6, 11, 12, 15) are
  behavior/wiring fixes, not contract changes, and are not blocked on this
  ADR. Genuinely new deltas (Tasks 5, 9 conditional, 10A conditional, 13/13B,
  14A/14B, 16, 17) all follow the repository's existing additive-compatibility
  convention (optional defaulted fields; new enum variants gated by
  record_kind or accepted as a clean old-reader `serde` error) — no
  `RECORD_VERSION`/global schema bump is required for this plan.
- **Baseline maintenance**: `just check`'s `gitleaks detect` step was failing
  on two long-known false positives — the `SECRET_SENTINELS` pattern-string
  constant in `sea-forge-ledger/src/types.rs` (contains the literal string
  `"-----begin private key-----"` as a redaction pattern, not a real key) and
  `conformance_m0_ledger.rs`'s test asserting a fake `sk-1234567890abcdef`
  payload is rejected by that same redaction mechanism (both confirmed benign
  by reading the actual lines, not just trusting the prior `.gitleaksignore`).
  Root cause: `gitleaks detect` scans full `git log -p` history, so the same
  line can be re-flagged under a *different* commit hash whenever an
  unrelated nearby edit shifts diff context — the repo's history has two such
  commits (`b4464a91…` original, `effc01c455…` a later shift) for each of the
  two lines. Fix: re-added all four resulting fingerprints to
  `.gitleaksignore` (confirmed via `gitleaks detect --redact -v`, now `no
  leaks found`) and added inline `// gitleaks:allow` comments on both lines
  so future commits never regenerate a fifth fingerprint for the same
  content.
- Global gate (`just context-check && just check && just test && just proof
  && just no-async-kernel`) run clean after the operator's `cargo clean` and
  the gitleaks fix above — **all green**: `context-check` passed; `check`
  (fmt, clippy `-D warnings` workspace/all-targets/all-features, `cargo
  check --locked`, `cargo deny check` — advisories/bans/licenses/sources ok,
  `gitleaks detect` — no leaks found) passed; `test` — **488 passed, 0
  failed, 2 ignored** (the two ignored are the documented real-host release
  gates, `t16_1_real_acp_host_release_gate` and
  `t16_6_real_swe_seed_release_gate`, both requiring operator-configured
  external hosts per their own skip messages — not a portable-gate gap);
  `proof` — P1-P4b passed; `no-async-kernel` — "ok: no async runtime or HTTP
  client in 19 kernel crates". This is the frozen clean baseline Tasks 1-18
  build on.
- `.agents/OPEN_QUESTIONS.md` has no unresolved entries after Task 0 — all
  three dependency choices and the network-isolation scope question were
  resolved by owner confirmation in-session rather than left open.
