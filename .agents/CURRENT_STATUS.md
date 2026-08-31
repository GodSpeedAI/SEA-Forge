# Current Status

Updated: 2026-08-31

> **2026-08-31 SEA-FORGE JOURNEY SETTLEMENT GAUNTLET — SETTLED (12/12 PASS).**
> Fully automated, canonical journey settlement testing system constructed and executed
> against the live Workbench product using `agent-browser` 0.34.0 and independent backend
> settlement oracles.
> - **Canonical Journeys**: All 12 canonical journeys (CJ01–CJ12) evaluated across all 10
>   required gates (Entry, Visibility, Reachability, Binding, Authority, Execution,
>   Evidence, Settlement, Continuity, Recovery) — all 12 PASS.
> - **4-Dimensional Coverage**: 100% Canonical Journeys (12/12), 100% Reconciled Stories
>   (128/128 from `canonicalization-matrix.csv`), 100% Interface Projections (38/38
>   bindings across Web UI, API, CLI, Agent), 100% Journey Transitions (10/10).
> - **Settlement Integrity**: Browser assertion never substitutes for independent settlement.
>   Oracles independently verify immutable case plans, precondition digests, approval ledgers,
>   semantic envelopes, and execution boundaries.
> - **Artifacts & Evidence**: Machine-readable contracts, schemas, traces, snapshots,
>   consequential screenshots at every boundary, and summary reports committed under
>   `.agents/reports/ux-journey-settlement/`.


> **GODSPEED CANONICAL RUNTIME CONVERGENCE — SETTLED.**
> All 35 frozen requirements CONFIRMED. Delta = 0. Preregistration hash
> `ef5710893c5bcc569a4757371c54d4d2e5e6168e1df61d2d7433b6889eaa879f`
> intact. T12 fresh independent final verifier returned CONFIRM after
> 13 independent attacks across all 7 variation classes. Evidence:
> `.agents/evidence/e2e/T0{1..12}/`. Infrastructure docs:
> `docs/explanations-and-references/goodspeed-loop.md`.
> **Next executable action: none — plan settled.**

> **2026-08-25 convergence plan T07 settled (subagent-built, subagent-verified).**
> E7: canonical ProofCompleted (SWE_SEED `proof_completed.rs`) binds ONLY to
> real T06-adjudicated settlements via content-addressed refs with mandatory
> causality; sxr native ingestion gate (`sxr-core/src/proof_ingest.rs`)
> preserves expected-vs-observed inputs by reference, names diverged fields
> (`ClaimMismatch`), keeps first-record integrity, and refuses duplicates/
> replay/cross-wire/placeholder identity. Independent adversarial confirmation
> by fresh subagent: **CONFIRM** — 43 empirical attacks incl. content-address
> forgery and both plan teeth; residual D1-D4 debt (out-of-band digest
> pinning, undelivered-bundle markers) recorded for T08. Evidence:
> `.agents/evidence/e2e/T07/`. Delta: **11 open / 24 CONFIRMED; T08 ready**
> (Expected-vs-Observed developmental evidence — E8/I8/I10, P3).

> **2026-08-25 convergence plan T06 settled (subagent-built, subagent-verified).**
> E6/I6/I7: settlement evaluated from ledger-settled observations vs DECLARED
> E4 criteria (exit-zero alone can never accept; non-completion rejected even
> with criteria text present); canonical envelope with all 7 frozen fields,
> mandatory E5A+E5B causality, content-addressed evidence_refs; SWE_SEED
> adjudicator bound to originating work_request_id with restart-safe
> idempotency and conflicting-resettlement refusal; operational-facts output
> type carries zero proof/capability shape. Independent adversarial
> confirmation by fresh subagent: **CONFIRM** — preregistered battery executed
> empirically incl. end-to-end exit-zero coercion cross-repo; T05-A10d/A11
> debt CLOSED here; T04-D1/D2/D3 verified not reproduced. Evidence:
> `.agents/evidence/e2e/T06/`. Delta: **12 open / 23 CONFIRMED; T07 ready**
> (Proof to RealityTrace boundary — P3).

> **2026-08-25 convergence plan T05 settled (subagent-built, subagent-verified).**
> E5A/E5B/I5: canonical AuthorizedInvocation emitted only from a REAL Allow
> AuthorityDecision (deny/escalate ⇒ no envelope, no grant, no governed side
> effect; runtime refuses foreign actions under genuine grants); InvocationLedger
> binds ExecutionObservation to the exact invocation generation (late/cross-wired/
> duplicate/self-asserted-authority refused). Builder: 20 tests, full
> sea-forge-server suite green. Independent adversarial confirmation by fresh
> subagent: **CONFIRM** — 20 empirical attacks incl. all three plan falsifiers;
> trust/durability items (A9-A11) recorded as non-falsifying debt for composing
> transports. Evidence: `.agents/evidence/e2e/T05/`. Delta: **15 open /
> 20 CONFIRMED; T06 ready** (Operational Settlement Return — P3).

> **2026-08-25 convergence plan T04 settled (owner-directed; subagent-built,
> subagent-verified).** E4 GovernedWorkRequest: producer surface
> (`SWE_SEED .../federation/governed_submission.rs` — all 8 frozen fields,
> packet bound to same cycle, causality recorded, opaque submissions refused
> pre-emission) + SEA-Forge ingress gate
> (`sea-rs .../sea-forge-server/src/governed_work_ingress.rs` — producer
> authority, identity vs locally-resolved model, semantic-intent battery,
> distinct proof/settlement obligations, context-packet causal-parent
> binding). Builder: 22 tests both sides, gates green. Independent
> adversarial confirmation by fresh subagent: **CONFIRM** — 22 attacks
> (all three plan falsifiers refused + fresh compositions), D1-D3
> strictness gaps recorded as non-falsifying debt. Evidence:
> `.agents/evidence/e2e/T04/`. Delta: **18 open / 17 CONFIRMED; T05 ready.**

> **2026-08-25 convergence plan T02+T03 settled (owner-directed: proceed).**
> T02 (E2/E3/I4): the SWE_SEED↔Context-Kernel MCP stdio slice now speaks the
> canonical envelope contract on both sides — CK ingress gates (exclusive
> swe_seed producer, identity placeholder rejection, correlation required),
> explicit governed no-context outcomes for required context, canonical
> ContextPacketCreated egress citing the E2 request as causal parent, and
> consumer-side composed adjudication (producer authority + drift + cross-wire
> + causality). Proven by dispatch teeth, unit teeth, AND a live
> cross-binary stdio test. T03 (E0/E1): GSA `canonical_events.py` projects
> DesiredDirection (external-environment authority) and executable affordances
> into WorkRequested (godspeed_agent authority) fail-closed against
> destination-only inputs and pseudo-identities; SWE_SEED `work_ingress.rs`
> enforces the same contract at ingress; a golden fixture generated by GSA's
> real projector is consumed by the Rust gate as cross-language evidence.
> Gates: `just e2e-gate T02`/`T03` bound and green; swe-seed-core 363/0,
> ck-mcp+ck-bin 48/0, GSA new tests 11/11 (5 pre-existing env failures
> verified unrelated by ablation). Evidence:
> `.agents/evidence/e2e/T02/t02-report.md`,
> `.agents/evidence/e2e/T03/t03-report.md`. Delta: **19 open / 16 CONFIRMED;
> next ready task: T04** (blocked until then: nothing — T04's deps are met).

> **2026-08-25 convergence plan T01 implemented (owner-directed: proceed).**
> Canonical semantic envelope + DomainForge identity foundation landed in
> SWE_SEED `crates/swe-seed-core` federation module (the most complete
> existing production binder; no new envelope family, v1 wire shape
> unchanged, no new dependencies): `identity.rs` (VerifiedDomainIdentity
> identity gate — fallback pseudo-hash sha256("agentic_capability_loop"),
> all-zero, malformed, and missing/mismatching artifacts all rejected; strict
> resolution errors instead of falling back), `producers.rs` (exclusive
> event-producer registry per the frozen edge topology; unknown types/agents
> fail closed), `derive_event` + `caused_by:` provenance entries (causal
> parents without breaking v1 `additionalProperties:false`; correlation
> `work_request_id` carried unchanged, mismatches fatal), `IdempotencyLedger`
> (fsync-per-admission durable dedupe surviving restart), `validate_envelope`
> composition gate, and pure `check_conformance` that cannot evaluate claim
> truth. Tests: `tests/convergence_t01_envelope.rs` — 22 tests mapping every
> preregistered T01 falsifier incl. all three plan teeth attacks (missing
> model → rejected; forged EvidenceRecorded by non-RealityTrace → rejected;
> conformant-but-false claim → zero truth consequence). Verification: full
> swe-seed-core suite green (0 failed; 2 pre-existing ignored), touched-file
> fmt clean, clippy clean on touched code. Gate bound: `just e2e-gate T01`
> (runs T01 suite + v1_contract + federation_parity pins).
> **Status: SETTLED 2026-08-25 — fresh independent adversarial confirmation
> returned CONFIRM** (`.agents/evidence/e2e/T01/t01-verifier-independent-confirmation.md`;
> 13 fresh attacks, zero falsifiers; preregistration byte-for-byte intact).
> The 11 requirement verdicts are now CONFIRMED in
> `.agents/status/e2e-current-status.yml`; Delta recomputed (24 open);
> **T02 and T03 are unblocked per dependency_graph and may begin in parallel.**

> **2026-08-25 convergence plan T00 executed (owner-directed: implement
> `.agents/plans/e2e-plan.yml`).** Delta-0 and the verification surface now
> exist. (1) `.agents/status/e2e-current-status.yml`: all 35 frozen
> requirements (12 edges E0–E10 incl. E5A/B, 15 invariants I1–I15, 8 envelope
> rules ENV-I1–I8) classified against fresh cross-repo source evidence
> (sea-rs, SWE_SEED, Context_Kernel, godspeed_agent, sxr, domainforge):
> **0 CONFIRMED / 30 PARTIAL / 5 ABSENT (E4, E6, E7, E8, I2) / 0 CONTRADICTED
> / 0 UNKNOWN — open delta = 35**, independently confirming every round-zero
> hypothesis in the preregistration (E8 has no production producer anywhere;
> GovernedWorkRequest/OperationalSettlement exist in no repo's code). (2)
> Gate aliases bound per the plan's verification model: `just e2e-check`
> →`just check`, `just e2e-test`→`just test`, `just e2e-lint`→fmt+lint,
> `just e2e-delta-check` (new mechanical validator), `just e2e-gate <TID>`
> (T00 bound to prereg+delta checks; other tasks fail closed until they bind),
> plus `just e2e-delta-report`. (3) New scripts: `e2e-prereg-ids.sh` (derives
> the frozen 35 from the preregistration itself — PyYAML + scoped fallback),
> `e2e-delta-report.sh` (deterministic Delta-0 regeneration),
> `e2e-delta-check.sh` (count/uniqueness/frozen-vocabulary/evidence-resolution/
> CONFIRMED⇒production-path validation + committed-report drift detection).
> (4) Evidence: `.agents/evidence/e2e/T00/delta0.md` (committed,
> drift-checked). Teeth (all temp-copy isolated, real files untouched): removed
> requirement→FAIL, illegal verdict→FAIL, CONFIRMED-on-synthetic→FAIL,
> unresolvable evidence path→FAIL, drifted report→FAIL, unbound task
> gate→fail-closed; real gates green after teeth (`just e2e-gate T00` exit 0).
> Frozen preregistration byte-identical (ef571089…a879f); plan file untouched
> (0c57e543…d0e39). Not yet run this session (alias-only, delegates to
> untouched existing gates): `just e2e-check`/`e2e-test` full workspace cost.
> Per plan initial_state, next executable action is **T01** (canonical
> semantic envelope/domain identity foundation) after this handoff.


> **2026-08-25 E2E preregistration freeze gate added (owner-requested).** New
> root recipe `just e2e-prereg-check` →
> `scripts/check-e2e-preregistration.sh`: recomputes the SHA-256 of the frozen
> preregistration `.agents/specs/e2e-preregistration.yml`, reads the expected
> hash strictly from `source.spec.sha256` in `.agents/plans/e2e-plan.yml`
> (PyYAML when available, indentation-scoped fallback otherwise; placeholder
> and malformed values rejected), prints expected/observed plus an explicit
> `VERDICT: PASS|FAIL`, and exits nonzero on mismatch without ever rewriting
> the stored hash — a mismatch stops the convergence/Gauntlet run until the
> change is reviewed and re-frozen. Verified: real preregistration PASSes
> (`ef571089…a879f`); teeth-checked nonzero FAIL on mismatch through `just -f`
> against a tampered temporary mirror, and on unbound-placeholder/malformed/
> missing-file paths; frozen file byte-identical after all checks. The plan's
> final_acceptance item "Verify the preregistration SHA-256 equals
> source.spec.sha256" is now executable as `just e2e-prereg-check`.


> **Deep-project-audit remediation is active on `ultracode/sea-forge-completion`.**
> Preserve the protected stashes (`stash@{0}` and `stash@{1}`); do not pop
> either without reviewing it. The governing plan is
> `.agents/plans/deep-project-audit-remediation-2026-08-14.md`.
>
> **Latest slices: Batch 8 remainder (F-13, F-16) and Batch 9 hardening sweep
> COMPLETE — the deep-project-audit remediation plan is now fully executed.**
> Every finding row in the plan inventory is now R (remediated), AR, P with a
> recorded disposition, or D/NA; remaining follow-ups live in
> `OBSERVED_DEBT.md` (F-25.n idempotency-for-request-less-mutations,
> CEP-0008 inbound causation_id, dead Compatibility/ExtensionInstallRecord
> types, true RFC 8785 JCS profile, ComposedModel overlay precedence) — each
> with its trigger/owner decision.
>
> Batch 8 remainder — **F-13**: promotion matches declarations by *exact*
> `plan_item_id` equality and rejects `"*"` outright (typed error), so a
> capability named `test` can no longer aggregate `itm_contest_7` evidence;
> thoth disclosure uses the same exact rule. Tests: m4a f13_*. **F-16**:
> policy/plan references resolve strictly workspace-relative under the cell
> root (`resolve_policy_path` via core lexical validation; typed UnsafePath on
> absolute/traversal), wired through submit/delegate/probe/cancel/SWE_SEED +
> CLI client spellings; nine server suites + CLI m13 fixtures converted and
> now double as boundary proofs.
>
> Batch 9 — **F-11**: CLI `resume` recovers stranded `Active` cases
> (activated-but-unsettled episodes terminal-settle as rejected/interrupted,
> parked human tasks untouched, loop re-drives lawfully). resume_recovery.rs
> (5 tests). **SUP-06**: probe adapter version derived from config identity
> (`cfg-<12 hex>`); fabricated output digest replaced with a real schema hash;
> failed registration settles Rejected (`agent_endpoint_registration_failed`)
> after authority commits; en-route fix: registry attestation moved from
> per-probe case ledgers to a dedicated cell-scoped `extension-registry`
> ledger (cross-case/restart verification now actually works). Tests: m12
> t12_7_* (4). **F-24**: precondition record cap (16, pre-side-effect typed
> error), resolver opens the ledger once per bundle, jail spawn/harvest/
> continuation-scan/cancellation/events publish/get_range/replay/case minting
> + settlement recording all moved off tokio workers onto blocking threads;
> events ledger mutex-across-fsync eliminated (Arc + ledger flock remains the
> serializer). Tail-cache/paged events deferred with triggers. **F-25
> kernel items**: f pinning test; g narrowed per owner (legacy silent-policy
> recall = own-entity only; cross-entity requires explicit memory_scope);
> h grants bind decision-time canonical argv[0] and runtime re-checks at
> spawn; i shared O_NOFOLLOW `safe_write` through materialize/env/artifact
> paths; j mutation-class reserved mutators hit the hard generated-zone/.git/
> .env/secret boundary before any rule; k collect_artifacts fails closed in
> both backends; l approvals journal single-write append (one core owner);
> m MAX_PLAN_ITEMS=256 + empty-ops SandboxedTask rejection (evaluator-driven
> items exempt) + iterative has_cycle (100k-chain validated); q ledger append
> refuses forged/rewritten tails (predecessor content-hash re-check);
> r sync_data after flush in evidence/trace/capability writers + byte-oriented
> recall scans (non-UTF8 degrades to malformed count). Workbench items a–d
> landed in the separate Tauri workspace (frozen lockfile, sidecar env
> allowlist + no-PATH fallback fail-closed, PID-file-scoped down recipes).
> **SUP-09d**: one canonical primitive (`sea_forge_core::canonical`) behind
> all four former copies with golden vectors — en-route discovery: the copies
> diverged on nested-value NFC; shared primitive recurses at all depths (ASCII
> data ⇒ historical hashes stable; F-25.q would surface any divergence
> loudly). **SUP-09e**: `dual_declared_concepts()` discloses the reviewed
> 30-name overlay-collision set (allowlist-pinned). **SUP-09i**: CEP-0008 ULID/
> sha256/id-segment grammar, descriptor id/version grammar,
> register_built_in hash-mismatch is a loud build bug, import dedupe/terminal-
> standing refusal (registry-brick vector closed).
>
> **Verified after both batches:** `cargo fmt --all -- --check` clean;
> `cargo clippy --workspace --all-targets --all-features --locked -- -D
> warnings` clean; full workspace `cargo test --workspace --all-features
> --locked --no-fail-fast`: **112 suites / 953 passed / 0 failed / 4 ignored**
> (documented real-host release gates); `just context-check` passed.
> Workbench src-tauri gate (after operator installed the Tauri system
> prerequisites): `cargo fmt --check` clean (scoped fmt applied to pre-existing
> drift in bridge.rs/drafts.rs), `cargo clippy --all-targets -- -D warnings`
> clean, `cargo test` green — 27 lib + 4 bridge + 4 packaged_stack; the F-16
> boundary also caught the host bridge test's absolute plan/policy spellings,
> now cell-relative.
> **Latest slices: Batch 5 (identity/role propagation) and Batch 6 (evidence
> integrity) COMPLETE.**
> Batch 5 — F-08: verified `ResolvedActor` (id + role) flows from
> `dispatch_bounded` through `handle_request_as` into all five authority
> evaluation sites; one documented operator fallback for in-process callers.
> F-23: thoth matches disclosure grants against actor id **plus** every role
> the bundle binds to that principal. F-25.e: one `content_hash` primitive
> behind both bundle-hash producers — policy identity no longer path-derived.
> SUP-09f: self-model rebuild idempotency keyed on realization content hash
> (teeth-checked pre-fix). Regression tests:
> `conformance_role_propagation.rs`, thoth `t23_*`,
> `policy_bundle_hash_is_independent_of_source_base_path`,
> `conformance_rebuild_identity.rs`.
> Batch 6 — SUP-04: projection records stamp `Declared`/
> `projection_unvalidated`/`validator_ref "none"`; `verify_projection` now
> hashes materialized outputs against `output_refs` (replaced view file ⇒
> `self_model_error`, teeth-tested in t93). SUP-09h: domainforge authority
> trace honestly describes the stem-heuristic approximation. F-20: jail's
> stderr heuristic classifies as new `SuspectedSandboxViolation`
> (`suspected_jail_violation` basis, still Rejected); definite
> `SandboxViolation` reserved for observed violations.
>
> **Verified after both batches:** fmt clean; clippy `-D warnings` clean on
> all touched crates; full workspace suite 111 suites / 914 passed / 0 failed
> / 4 ignored (documented real-host release gates).
>
> **Latest slice: Batch 7 (root convention + registry trust) COMPLETE.**
> F-12/SUP-07: the state-root convention is now universal — cell.json,
> self-model, thoth capability/declaration reads, transcript-seal keys, bundle
> export/import, template adopt, and the capability promotion readers all join
> directly under the passed root; the CLI default root directory (`.sea-forge`)
> is simply the state root. Export fails closed on zero-resolving requested
> runs; an absent extension registry discloses the snapshot stale
> (`extension_registry_absent`) instead of fabricating a zero-extension cell.
> SUP-08: `ExtensionRegistry::load_verified` proves registry bytes against the
> newest `extension_registry` ledger record; quarantined runtime adapters can
> no longer be replaced/resurrected by registration; duplicate `(id,version)`
> entries rejected at load. Tests: `export_with_unresolvable_runs_fails_closed_not_silently_empty`,
> `legacy_sea_forge_symlink_is_inert_to_import`, extension `load_verified_*` /
> `quarantined_runtime_adapter_*`. Verified: fmt clean, clippy `-D warnings`
> clean on all seven touched crates, full workspace 111 suites / 917 passed /
> 0 failed / 4 ignored.
>
> **Next task:** Batch 8 remainder — governance metadata + operability:
> F-13 (exact capability↔plan-item mapping in promotion matching, reject `*`)
> and F-16 (constrain server policy/plan path resolution to workspace-relative
> under cell root). F-21 already landed cross-batch. Then Batch 9 hardening
> sweep (F-24, remaining F-25.a–d/f–i items, SUP-09d/e/i) plus the still-open
> F-11 (stranded-`Active` recovery verb) and SUP-06.


> **Latest slice (Workbench, owner-directed):** the "Case-authoring proof
> scenarios (6, 7) have no e2e coverage" debt is resolved — per the owner's
> direction the coverage is driven by **agent-browser**, not Playwright. New
> `just workbench-e2e-case-authoring` → `scripts/workbench-e2e-case-authoring.sh`
> + `workbench/apps/desktop/e2e-agent-browser/sfwp-case-authoring-mock.js`
> (mocked-IPC shim; agent-browser counterpart of `e2e/tauriMock.ts`). Both
> journeys pass against the real renderer: stale-precondition repair (commit
> #1 rejected stale carrying digest A, re-preflight pins B, commit #2 carries
> B, zero status recovery) and dropped-commit recovery (one `case_commit`
> total, forced click on the disabled control never reaches the bridge, one
> `request.get_status`, zero axe violations, zero page errors). Evidence and
> method: `.agents/reports/2026-08-15-case-authoring-agent-browser-e2e/`.
> Two new debt entries filed from this work: the authoring pill keeps
> "Preflight passed" in `rejected_as_stale`, and the Playwright
> `tauriMock.ts` readiness fixtures fail the current `ReadinessView`
> contract (`next_lawful_action` required).
>
> **Completed:** Batch 1 (path/generated-zone), Batch 2 (ledger
> crash-consistency/corruption handling), Batch 3 (dispatcher completion
> invariants), and **Batch 4 (resource bounds + panic/overflow) now COMPLETE**:
> SUP-01, SUP-05, F-07, F-18, F-19/SUP-09a, plus SUP-09b (symlink-safe bounded
> migration traversal; rejection precedes key creation and every other side
> effect) and SUP-09c (caps for untrusted whole-file reads: server view
> readers 4 MiB records / 64 MiB journals behind one shared `size_within_cap`;
> trace internal-error append now streams; jail stderr heuristic capped at
> 65,537 bytes with lossy decode; `internal-test-swe-seed` stdin capped at
> 1 MiB). Cross-batch: F-10, F-21, SUP-02 also landed (see the plan's §6).
>
> **Next task:** Batch 5 — identity/role propagation (class D): F-08
> (`ActorRole` into all `Actor` construction sites), F-23 (thoth actor id→role),
> F-25.e, SUP-09f (release-id pinning), per the dependency-ordered plan.
>
> **Latest verified checkpoint (kernel slice):** `cargo fmt --all -- --check`
> clean; `cargo clippy -p sea-forge-cli -p sea-forge-server -p sea-forge-trace
> -p sea-forge-sandbox --all-targets -- -D warnings` clean; new suites green
> (migrate_safety 3, swe_seed_cli 2, conformance_case_views 9,
> conformance_run_views 11, sea-forge-trace 10, sea-forge-sandbox incl.
> conformance_m1 15/15; conformance_m0_migrate 2 regression intact); full
> workspace `cargo test --workspace --all-features --locked --no-fail-fast`
> exit 0 — 109 test binaries, 907 passed, 0 failed, 4 ignored (documented
> real-host release gates). **Workbench slice:** `just
> workbench-e2e-case-authoring` exit 0 (both journeys; recipe wiring
> verified end-to-end); workbench deps installed via
> `bun install --frozen-lockfile` (667 packages).


> **2026-07-27 rebase recovery complete.** `main` now contains the previously
> local full-spec/workbench history rebased onto `origin/main` (110 commits
> ahead, 0 behind). Rebase conflicts preserved the published licensing package,
> added the approved exact `xxhash-rust 0.8.16` / `BSL-1.0` cargo-deny
> exception, and retained a valid `LicenseRef-SEA-Forge` workspace expression
> plus `LICENSE` file reference. `cargo metadata`, `cargo deny check licenses`,
> and `git diff --check` pass. Graph refresh was explicitly deferred. Local
> Jolli state is preserved in `stash@{0}`; the original user stash remains at
> `stash@{1}`. Do not pop either without reviewing its contents.

> **2026-07-27 architectural adjudication Pass 2 complete.** Seven authoritative
> reconciliation and packaging inputs were added under `docs/execution/`:
> `ARCHITECTURAL_TRUTH.md`, `ARCHITECTURAL_INVARIANTS.md`,
> `CONTRADICTIONS_AND_DECISIONS.md`, `PRODUCT_COMPLETION_DEFINITION.md`,
> `EXECUTION_DAG.md`, `DECISION_REGISTER.md`, and `PASS_2_HANDOFF.md`. No product
> feature or source behavior changed. Independent source review corrected Pass
> 1's Copilot, tracked `working/face/`, DomainForge, route-guard, inferred-gate,
> and end-to-end usability claims. The highest completion blockers are the
> server case sandbox path's pre-authority directory creation, incomplete
> lifecycle/exit-code settlement, case-run versus run-view path mismatch,
> unresolved Workbench/server service lifecycle, identity/idempotency gaps, and
> absent packaged real-stack E2E. Distribution packaging is conditionally ready:
> owner decision U-06 (Tauri sidecar versus separately installed local service)
> is required before deployment configuration changes. Verification: fresh
> adversarial review completed and reconciled; `git diff --check` clean;
> `devbox run -- just context-check` passed.

> **2026-07-26 Workbench plan Task 7 (case overview and horizon) complete,
> plus the four `OBSERVED_DEBT.md` entries Task 7's own "review before
> starting" block gates on.**
>
> **New SFWP methods (five, all additive per ADR-003).** `case.list`,
> `case.get_overview`, `case.get_horizon` in new
> `crates/sea-forge-server/src/sfwp/case_views.rs`; `approval.list` and an
> `approval.decide` envelope in new `sfwp/approvals.rs`. `IMPLEMENTED_METHODS`
> is now sixteen, covering six of the epic's journeys. All five are read-only
> projections over records the kernel already committed — `case.json`,
> `plan.json`, per-run `settlement.json`, `case-events.jsonl`, and
> `approvals.jsonl`. None introduces new truth.
>
> **Horizon item standing is folded from trace events, not read off a status
> field.** No per-item status exists anywhere in the kernel; standing *is* the
> `TraceKind` sequence the case runner appended. Folding it is reading kernel
> truth (the same derivation `next_case_actions` performs); caching it anywhere
> would create a second authority that drifts.
>
> **Execution and settlement are structurally separate.** `ExecutionStanding`
> and `SettlementStanding` are disjoint enums with no shared values, neither
> derived from the other, and the UI maps `execution: completed` to a
> *non-success* pill. A zero exit code cannot masquerade as accepted work.
> `execution_and_settlement_are_separate_vocabularies` asserts the vocabularies
> never cross.
>
> **One owner for the approvals fold.** The append-only journal's "latest record
> per `approval_id` wins" rule moved from `sea-forge-cli` into
> `sea_forge_core::approvals`; the CLI module is now a re-export. Two folds
> could disagree about whether an approval is open, and the one saying "open"
> would offer a decision already made.
>
> **Debt resolved.** (1) `scripts/check-agent-context.sh` — CRLF→LF + exec bit;
> `just context-check` is a live gate again. (2) The host's hardcoded
> `GET_RANGE_PAGE_CAP = 256` vs the server's real 500 — deleted rather than
> reconciled: the catch-up drain now terminates on an **empty** page, encoding
> no assumption about the server's page size at all. (3) Coarse readiness
> event-invalidation — `hooks/eventKinds.ts` narrows by kind, deliberately
> asymmetric so an *unrecognized* kind still invalidates (fails open to an extra
> read, never to a stale render). (4) `approval.decide` reachable but not
> discoverable — `approval.list` closes it.
>
> **Two real defects found and fixed en route.** The event-loop listener
> dereferenced `event.payload` unguarded; a throwing listener tears down the
> whole subscription, so it now reads `event?.payload` and degrades to
> "invalidate anyway". And `` `${verdict}d` `` rendered "rejectd" to the operator
> — replaced with an explicit past-tense map.
>
> **Evidence.** `cargo fmt --all -- --check` clean; `cargo clippy --workspace
> --all-targets --all-features -- -D warnings` clean; `./scripts/check-agent-context.sh`
> → "context check passed"; **`cargo test --workspace` — 98 suites, 714 passed,
> 0 failed, 4 ignored** (the ignored are the documented real-host release
> gates); 13 of those are new (`conformance_case_views.rs` 6,
> `conformance_approvals.rs` 7); host `tests/bridge.rs` 4/4 including the new
> `catch_up_drains_until_a_page_is_empty_not_merely_short`, verified to have
> teeth (restoring the short-page rule drops 3 of 6 events); `bun run check`
> clean apart from the pre-existing `router.tsx` fast-refresh warning;
> `bun run test` 73 desktop + 17 component tests green; `bun run build`
> produces a renderer bundle.
>
> On the one failure seen in the *first* workspace run
> (`kill_9_leaves_a_valid_jsonl_prefix_without_capability_corruption`): measured
> rather than assumed-flaky, because this change touched `sea-forge-cli`. A
> clean-`HEAD` worktree passed while the working tree failed, which read as a
> regression; re-running both on an idle host resolved it (working tree 8/8
> consecutive, and the second full workspace run green). It tracks host load,
> not the diff. Method recorded in `OBSERVED_DEBT.md`.
>
> **Still open.** Three specimen surfaces remain (Thoth, Assets, Models) —
> `thoth.ask` has no typed response contract, `asset.list`/`domain_model.list`
> do not exist. Plan Tasks 8–14 remain. The Playwright horizon journey named in
> Task 7 step 4 was **not** run: the e2e harness mocks `__TAURI_INTERNALS__`
> entirely, so it cannot prove real event delivery end to end — that limitation
> is its own standing `OBSERVED_DEBT.md` entry and was not closed here.

> **2026-07-26 Workbench plan Task 6 (case authoring: draft, preflight,
> atomic commit) complete.** First protected-command vertical slice: three
> additive SFWP methods in new `crates/sea-forge-server/src/sfwp/case.rs`
> (`Request::CaseEntryOptions`/`CasePreflight`/`CaseCommit`, `lib.rs`). Grounding:
> `case.entry_options` is an honest inspect projection of whatever templates are
> already materialized under `<root>/templates/*.yaml` (empty when none exist —
> `unknown != unavailable`, never a fabricated built-in catalog).
> `case.preflight` instantiates a template (`sea_forge_planner::templates::instantiate`)
> and runs the *same* `case_engine::validate_proposal` `case_dispatch::submit`
> uses — an authoritative dry run, not a forked validator — returning a
> `RecordDigest` pinned to the template's current on-disk bytes
> (`template:<ref>` ref, reusing the existing `sfwp::precondition` mechanism
> rather than inventing a second staleness scheme). `case.commit` writes the
> instantiated plan to a scratch file under `<root>/drafts/` and delegates to
> the exact same `case_dispatch::submit` path `Submit` always used (extracted
> into a shared `commit_plan` helper) after checking the precondition via a new
> `TemplateRecordResolver` — on mismatch, `rejected_as_stale` with zero side
> effects (no case created); on match, the one and only case-minting path runs.
> `request_id` correlation (`record_pending`/`record_outcome`, reused unchanged
> from Task 3) makes a lost commit response recoverable via
> `request.get_status` instead of a resubmit. Draft-storage spike (deferred
> decision in `stack-and-dependencies.md`) resolved as **versioned local JSON
> files under the Tauri host's `app_data_dir`** (new `drafts.rs` + four
> `draft_save`/`draft_load`/`draft_list`/`draft_delete` commands) — zero new
> dependencies, atomic tmp+rename writes, never touches `.sea-forge/`; SQLite/the
> Tauri store plugin deferred until real multi-draft conflict needs appear.
> Frontend: `bridge.rs` gained `SfwpQuery::CaseEntryOptions`/`CasePreflight` and
> `SfwpCommand::CaseCommit` mirrored byte-for-byte; contracts regenerated (6 new
> generated types: `EntryOptionsResult`/`TemplateOption`/`TemplateParameter`/
> `PreflightParams`/`PreflightResult`/`PlanItemSummary` + AJV validators,
> deterministic rerun confirmed, no drift on existing 14). New
> `caseAuthoringMachine.ts` (XState v5): `draft -[PREFLIGHT]-> validating
> -> preflight_ok -[COMMIT]-> committing -> committed | rejected_as_stale |
> ambiguous`. `ambiguous` deliberately has no `COMMIT` handler — only
> `RECOVER -> status_recovery` (calls `request.get_status`) — so a duplicate
> commit is structurally unreachable, not just documented (proof scenario 6,
> asserted by a machine test that sends `COMMIT` while `ambiguous` and checks
> the state didn't move). `rejected_as_stale`'s only transition is
> `RETRY -> validating` (a fresh preflight, never straight back to
> `committing` — proof scenario 7). New `CaseCreationWorkbench` route
> (`/cases/new`, `react-hook-form` for per-template parameter fields, no new
> `@hookform/resolvers` dependency — required-field validation only, since
> real type/shape validation is `case.preflight`'s job, not duplicated
> client-side) and `useCaseEntryOptions` hook (plain TanStack Query, no
> machine, mirroring `useReadiness`). `ReadinessPage`'s previously
> permanently-disabled "Create case" action now navigates to `/cases/new`
> whenever `readiness.get`'s `local_governed_execution` capability and all
> foundations are ready — the first real consumer of that lawful-action slot.
> Tests: 6 new Rust conformance tests (`conformance_case_authoring.rs`:
> entry_options honesty incl. empty-when-absent, preflight ok/error, commit
> success + status roundtrip, stale-precondition-rejects-with-no-case-created,
> commit-outcome-recoverable-via-request.get_status) + 6 XState machine tests
> (including the two proof-scenario tests above) + 1 component test (full
> draft->preflight->commit->navigate flow) + `ReadinessPage.test.tsx` updated
> for the now-enabled action. Gates green: `cargo fmt --all -- --check`,
> `cargo clippy -p sea-forge-server --all-targets -- -D warnings`, `cargo
> clippy` (Tauri host crate, isolated workspace), `cargo test -p
> sea-forge-server` (all suites incl. the 7 new), `cargo test` (Tauri host,
> all suites), `bun run check` (desktop, one pre-existing Fast Refresh
> warning), `bun run test` (desktop 27/27 + ui-components 17/17), `bun run
> generate:contracts` (deterministic), `devbox run -- just fmt-check`/`lint`/
> `test` (workspace-wide, 0 failures), `devbox run -- just proof` (P1–P4b
> green). `just check`'s `context-check` sub-recipe remains blocked by the
> pre-existing `scripts/check-agent-context.sh` issue (`OBSERVED_DEBT.md`);
> no platform test is claimed through that blocked composite gate.
> Limitations/debt filed in `OBSERVED_DEBT.md`: no Playwright e2e was added
> for this slice (the existing mocked-IPC harness cannot honestly prove the
> stale-precondition/duplicate-commit-unreachable scenarios — those are proven
> at the Rust conformance + XState machine level instead, consistent with the
> skill's guidance to decide this deliberately rather than default to the
> mocked pattern); visual fidelity against the wireframe/mockup kit was not
> pixel-checked (no reference mockup for this screen exists in `ui_kits/`, so
> there is nothing to diff against — logged as an open gap, not claimed done).
> Next spendable slice: Task 7 (case overview + horizon) — the first slice
> that needs live per-case event reduction at scale, and the natural home for
> a "view the case I just created" landing page (this slice navigates to the
> still-mockup `/cases` on commit).

> **2026-07-25 Workbench mockup-fidelity repair complete.** The React shell now
> matches the checked-in workbench kit at its responsive evidence breakpoints:
> the Context / Evidence region is a 380px docked grid track above 1420px and a
> transparent, non-modal 400px (maximum 92vw) right overlay below it, beginning
> below the 56px global bar. It remains mounted while closed so the kit's 180ms
> `cubic-bezier(.23,1,.32,1)` slide-out/slide-in completes; Escape, the close
> control, evidence citations, and the header toggle preserve that state.
> Container-responsive Operate layouts stack focus actions and attention rails
> before labels/tables compress, while shell tracks follow the kit's 236/224/64
> navigation widths. The six Operate routes (Thoth, Assets, Domain Models,
> Cases, Inbox, Operations) now render their route-specific focus/panel
> hierarchies and active journey label instead of generic placeholders. Because
> no live route-family read models exist yet, those surfaces are visibly marked
> `Specification preview · not live`; their controls inspect context only and
> do not imply backend mutations. `readiness.get` remains the sole live source
> for the Readiness route.
>
> Durable regression evidence: `mockupFidelity.test.ts` drift-checks the
> reference stylesheet and required regions; `SurfacesPages.test.tsx` covers all
> Operate view structures; component coverage proves the drawer remains mounted
> for exit motion; Playwright asserts 1600px and 1280px shell/drawer geometry,
> exact transition timing/easing, close/reopen motion, all Operate route swaps,
> computed route-grid activation, no horizontal action overflow, and zero axe
> violations or console/page errors for each route plus Readiness. The
> browser-mode Tauri shim now supplies the event plugin's separate
> `unregisterListener` namespace, so listener cleanup is exercised without an
> unhandled rejection. The Workbench skill now requires
> same-viewport browser comparison and computed-style checks in normal and
> reduced-motion media; its source map records that the static kit's Operate
> selectors are accidentally trapped inside an unclosed reduced-motion block,
> so production must project their intent without copying that boundary.
> Frontend gates green: `bun run check` (one pre-existing Fast Refresh warning),
> `bun run test` (20 desktop + 17 UI-component tests), `bun run build`, and
> `bunx playwright test e2e/readiness.spec.ts --workers=1` (3/3).
> `devbox run -- just test` also passed workspace-wide. `devbox run -- just
> context-check` and therefore `just check` remain blocked before execution by
> the pre-existing non-executable `scripts/check-agent-context.sh` (exit 126),
> already tracked in `OBSERVED_DEBT.md`; no platform test is claimed through
> that blocked composite gate.

> **2026-07-25 Workbench plan Task 5 (Readiness vertical slice) complete.**
> First real settlement wired end-to-end: `readiness.get` SFWP inspect method
> (new `crates/sea-forge-server/src/sfwp/readiness.rs`, dispatched as
> `Request::ReadinessGet`) projects self-model validation
> (`sea_forge_self_model::store::validate`) and agent-endpoint config into a
> `ReadinessView { overall, foundations, operational_capabilities,
> recent_invalidations, intended_operation }` — infallible (a validation
> failure renders as a `blocked`/`integrity_halted` item, never propagates an
> `Err`), operation-sensitive (`intended_operation` reorders which capability
> is foregrounded), no new truth introduced. `ReadinessItem`/`Invalidation`
> shapes were defined from scratch (the API spec references but never defines
> them) grounded in the wireframe's condition-table/capability-row fields.
> Frontend: `SfwpQuery::ReadinessGet` added to the closed Tauri bridge
> (`workbench/apps/desktop/src-tauri/src/bridge.rs`); contracts regenerated
> (`ReadinessView`/`ReadinessItem`/`ReadinessGetParams` + AJV validators,
> deterministic rerun confirmed); `workbench/apps/desktop/src/hooks/useReadiness.ts`
> wraps the query in TanStack Query, validates every response against the
> generated AJV validator before trusting it, and invalidates on any
> `sfwp://event` frame (coarse but honest — no readiness-specific event kind
> exists yet); `ReadinessPage.tsx` rewritten from Task 4's hardcoded mock to
> compose real data through `WhyStatePanel`/`GovernedStatusPill`/
> `IntegrityIndicator`/`SourceFreshnessBadge`/`ProtectedActionButton`/
> `EvidenceDrawer`. Case-creation is permanently, honestly disabled (no
> `case.create` verb exists yet — Task 6) with a reason sourced from the live
> readiness view, never implying a working flow. Server disconnect renders
> the last-known view marked `stale` (TanStack Query's default data retention
> across a failed refetch) rather than blanking it. New Playwright + axe-core
> harness bootstrapped from scratch (`playwright.config.ts`, `e2e/tauriMock.ts`,
> `e2e/readiness.spec.ts`) mocking the Tauri IPC bridge via
> `window.__TAURI_INTERNALS__` injection — proves the real `ReadinessPage`/
> `useReadiness` code paths against a fixture, but does not exercise a real
> `sea-forge-server` process (that's covered separately at the Rust level by
> `conformance_sfwp.rs`'s `readiness_get_*` tests, which do boot a real server
> on a temp root). `statusMachine.ts` deliberately NOT replaced — a plain read
> needs no XState machine; it stays as the (unrelated) ProofPage's dependency
> pending a real preflight/commit machine in a later authoring slice. Two
> independent architect-verification passes both returned APPROVED. Gates
> green: `cargo fmt/test -p sea-forge-server` (4/4 readiness tests),
> `devbox run -- just fmt-check`/`lint`/`test` (workspace-wide, 0 failures —
> `just check`'s `context-check` sub-recipe could not run, see
> `OBSERVED_DEBT.md`), `bun run generate` (deterministic), `bun run check`
> (desktop, clean), `bun run test` (desktop 11/11 + ui-components 16/16),
> `bunx playwright test --grep readiness` (1/1, zero axe violations). Four
> new entries filed in `.agents/OBSERVED_DEBT.md`: the broken
> `check-agent-context.sh` (pre-existing, unrelated), the Playwright-mocks-
> vs-real-server e2e gap, coarse event-invalidation scope, and the
> permanently-empty `recent_invalidations`/never-derived `stale` fields.
> Next step: plan Task 6 (case authoring: draft, validation, preflight,
> atomic commit) — the first slice that needs a real mutation/lifecycle
> machine and will also give the disabled "Create case" button real work
> to do.

> **2026-07-25 Workbench plan Task 4 (shell + semantic components + guards) complete.**
> Created `@sea-forge/ui-components` Bun workspace package containing the nine semantic
> components (`GovernedStatusPill`, `DualStateIndicator`, `SourceFreshnessBadge`,
> `IntegrityIndicator`, `ProtectedActionButton`, `WhyStatePanel`, `EvidenceDrawer`,
> `AuthorityBoundaryPanel`, `AvailabilityLadder`) built with CSS Modules and `@astryxdesign/core` primitives.
> Critical invariant enforced: `GovernedStatusPill` with an unknown or invalid variant strictly fallbacks
> to `"unknown"` text & class (`status-pill--unknown`), verified by unit test.
> Storybook v8 configured in `packages/sea-forge-ui-components` with component stories (`*.stories.tsx`) for all 9 components.
> Built governed shell in `apps/desktop` featuring:
> - Sidebar navigation with all 13 top-level surfaces (Readiness, Thoth, Assets, Domain Models, Cases, Inbox, Operations, Evidence, Memory, Capabilities, Artifacts, Federation, Administration).
> - Global Context Bar (Actor, Role, Policy status, Integrity indicator dot, Search shortcut `/`, Inbox count badge).
> - Governed Focus workspace header, journey ribbon, skip-to-content link.
> - Collapsible right EvidenceDrawer skeleton.
> - Route guards G1–G9 with `GovernedDenialSurface` rendering on failed evaluation (never blank screens or crashes).
> - Keyboard traversal & ARIA accessibility support (`Tab`, `Shift+Tab`, navigation hotkeys `R` & `/`), verified by `axe-core`.
> Gates green: `bun run check`, `bun run test` (23 unit tests pass), `bun run build-storybook`, `devbox run -- just check`, `devbox run -- just test`. Added `just` recipes for dev server and Storybook (`dev-up`, `dev-down`, `storybook-up`, `storybook-down`). Next: plan Task 5 (Readiness vertical slice).

> **2026-07-24 Workbench plan Task 3 (SFWP transport) complete.** Additive
> SFWP protocol layer on the existing Unix-socket NDJSON server, added as flat
> `Request` variants behind the ADR-003 seam (`system_*`, `request_get_status`,
> `events_*`, precondition digests) in `crates/sea-forge-server/src/sfwp/`.
> Schema generation via `schemars` (confined to `sea-forge-server`; the
> `gen_sfwp_schema` bin emits JSON Schema to
> `workbench/packages/contracts/schema/`; the `@sea-forge/contracts` Bun
> package generates typed TS + AJV validators via `bun run generate:contracts`)
> — new deps recorded in `docs/decisions/ADR-005-sfwp-schema-generation.md`.
> Tauri host owns the socket via a closed `SfwpQuery`/`SfwpCommand` bridge
> (`workbench/apps/desktop/src-tauri/`, per `workbench/AGENTS.md`).
> Evidence: `crates/sea-forge-server/tests/conformance_sfwp.rs` (the named
> hello → subscribe → mid-flight kill → reconnect → `request_get_status` →
> cursor resume → `events_get_range` gap-recovery scenario) and 3 host
> integration tests in `workbench/apps/desktop/src-tauri/tests/bridge.rs`.
> Full gate green (`devbox run -- just check`/`just test`; workbench
> `bun run check`/`build`/`test`; Tauri `cargo build`/`test`). One watch item
> in `.agents/OBSERVED_DEBT.md` ("Host event-catch-up page cap …"): the host's
> `events.get_range` page-cap guess (256) vs the server's actual cap (500) are
> separately hardcoded — currently safe (conservative) but uncoupled. Next
> step: plan Task 4 (application shell + semantic design foundations: shell
> layout, nine semantic components, route guards G1–G9, Storybook, a11y).
>
> **2026-07-24 Workbench plan Task 2 (workspace + stack proof) complete.**
> New `workbench/` Bun workspace (`package.json`, `bunfig.toml`,
> `packageManager: bun@1.4.0`) with `apps/desktop/` (Vite + React 19.2 +
> TypeScript 6.0 strict, scaffolded via `bun create vite`) and a Tauri 2 host
> crate at `apps/desktop/src-tauri/` — a standalone Cargo workspace (own
> empty `[workspace]` table), deliberately not a member of the root kernel
> workspace. Token/theme packages: `packages/sea-forge-ui-tokens` (byte-exact
> copy-projection of `.agents/specs/frontend/colors_and_type.css`, drift-
> checked) and `packages/sea-forge-astryx-theme` (SEA Forge tokens projected
> onto `@astryxdesign/theme-neutral` via `defineTheme({ name: "neutral",
> extends, tokens })`, keeping Astryx's scoped component CSS wired while every
> token value routes through SEA Forge canonical vars). Locked stack
> exact-pinned: Astryx 0.1.8, StyleX 0.19.0, TanStack Router 1.170.18 / Query
> 5.101.4, XState 5.32.5 / @xstate/react 6.1.0, react-hook-form 7.82.0, ajv
> 8.20.0, @tauri-apps/cli 2.11.4, Vitest 4.1.10 — recorded in
> `docs/decisions/ADR-004-workbench-stack.md`. Proof page (`ProofPage.tsx`)
> renders an Astryx `Button`+`Table` themed by the projected tokens behind one
> typed TanStack Router route with validated search state, one TanStack Query
> call, and one XState machine (`statusMachine`, smoke-tested in
> `statusMachine.test.ts`). `just workbench-check` (new recipe) and the
> plan's exact gate command (`bun install --frozen-lockfile && bun run check
> && bun run build && cargo build --manifest-path
> apps/desktop/src-tauri/Cargo.toml && bun run test`) both pass; Tauri host
> `cargo build` succeeds (`dev` profile, 7 min cold compile). Along the way,
> at the user's explicit request, the machine's mise-managed `bun` was
> upgraded via `bun upgrade --canary` to 1.4.0 (Bun's in-progress Zig→Rust
> rewrite, confirmed via upstream announcement); `workbench/package.json`
> repinned to match, full gate re-verified green under it — reversible via
> `bun upgrade --stable` per the ADR. One watch item filed in
> `.agents/OBSERVED_DEBT.md`: the Tauri host's Cargo-workspace exclusion has
> no automated gate yet. Root kernel untouched (`git status` shows only
> `workbench/` plus this status update, the ADR, and `OBSERVED_DEBT.md`).
> Next step: plan Task 3 (SFWP transport: envelopes, negotiation, request
> recovery, events, generated contracts).

> **2026-07-24 Workbench plan Task 1 (repository grounding and compatibility
> map) complete.** `.agents/reports/2026-07-24-sfwp-grounding.md` grounds all
> 74 target SFWP methods across the 18 catalog families Task 1 names (system,
> request, operation, events, cell, readiness, self_model, case, run,
> agent_run, approval, thoth, settlement, capability, evidence, integrity,
> memory, artifact) against direct repository evidence (four parallel
> research passes, every row `file:line` cited). Verdicts: 14 reuse, 27 adapt,
> 3 merge, 30 add, 0 reject — no target method conflicts with a kernel
> invariant in this pass. Gate (`grep -cE` verdict-row count ≥ 40) passes at
> 75. Notable findings: `case`/`run` families have rich backing logic
> (`case_engine` item states, `TraceEvent` replay, `DelegationResult`) that is
> almost entirely unwired to any server verb; `case.propose_replan`/
> `commit_replan` are wholly missing; the 2026-07-22 audit's "Thoth `ask`
> bypasses governance" finding appears stale (`service::ask` already commits
> ledger records) and is filed in `.agents/OBSERVED_DEBT.md` for
> re-verification, alongside a `cell.migrate`/federation-bundle naming-
> collision caution for later tasks. `repository-integration.md` re-verified
> against the current tree; one line-number drift corrected
> (`prove_entry:915` → `:916`). No production/kernel code touched. Next step:
> plan Task 2 (workspace/stack proof: Tauri 2 + Bun + React 19 + Vite +
> Astryx).

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

Implement `.agents/plans/2026-08-02-workbench-product-completion-ralph-loop.md`
in its required order. Tasks 0 and 1 are `PASSING` on the current tree: the
evaluator packet has an exact machine-checked exclusion fixture, and protected
renderer actions fail closed when a server-derived identity is unavailable.
Task 2 is complete at the focused-slice level: readiness and approval standing
resolve only to typed committed records (or explicit unknown/stale/blocked
truth), and bridge refusals preserve their class, no-side-effect standing, and
next lawful action. The generated-contract clean-tree gate is intentionally
pending an eventual commit; it must not be bypassed by staging only generated
files. Task 3 is blocked at its explicit native-driver dependency decision:
the repository has mocked Playwright but no installed Tauri-native automation
driver, and the plan forbids calling the mock integrated proof.

Current documentation task: make `just` the agent-facing command-line interface.
All active agent-facing guides and diagnostics must name only `just` commands;
Devbox, Cargo, and Bun remain recipe implementations. Add generic crate-scoped
check/test and missing Workbench recipes rather than exposing a second CLI.

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

2026-08-02 Task 0: preserved unrelated dirty changes to the active completion
plan (Markdown table formatting only) and `.jolli/jollimemory/debug.log`
(one Jolli diagnostic line). Current Task 0 adds only evaluator-input
documentation, its JSON exclusion fixture, and the validator/`just` recipe;
no product behavior, dependency, persisted schema, or CI aggregation changed.

2026-08-02 Task 1: added the shared renderer affordance guard, a session-only
selector for server-advertised actors, bounded `actAs` forwarding, and usable
identity repair links. It covers case creation and commit, approval decisions,
delegation cancellation, and the recorded `thoth.ask` command. The unrelated
completion-plan Markdown formatting and Jolli diagnostic remain preserved.

2026-08-02 Task 2: added additive committed-source projections and regenerated
their TypeScript/AJV contracts. The source and approval changes are intentionally
uncommitted alongside the rest of this implementation. The contracts drift gate
therefore reports the expected uncommitted generated projection; do not stage
only those files merely to make that pre-commit guard pass.

2026-08-03 Task 4 is in progress at the fresh-root entry slice. A missing or
empty configured root now remains `initialization_required` until an operator
confirms initialization through the closed host bridge; opening the application
does not create it. The supervisor rechecks the root under its lifecycle lock
before spawning and refuses if history appeared meanwhile, with no socket or
record write by this path. The Readiness page exposes one initialization action,
does not offer case creation early, and displays a retryable structured refusal
with its next lawful action. The host gate passed all 34 lib, bridge, and
packaged-stack tests, but its final host-wide format check remains blocked by
pre-existing formatting drift in `bridge.rs`; it was left untouched. `just
fmt-check`, `just context-check`, and `git diff --check` passed. `just
workbench-check` remains blocked before renderer tests by the already-uncommitted
Task 2 contract generation drift; selection, recognized-history migration/version negotiation,
and the Task 4 real-cell matrix are still open. The real packaged fresh-cell
proof is now `just workbench-e2e-real initialization`: it starts with no root or
socket, finds and clicks the rendered initialization control through native
WebKit/Tauri, then verifies the bundled sidecar's SFWP hello; it passed on
2026-08-03. The earlier plan filter `cell|readiness|Thoth` matched no native
scenario and is not valid evidence. The supervisor also now fail-closes before
sidecar startup for a malformed or unknown fixed-path self-model manifest;
compatible and legacy history retain the existing startup path, but no migration
is yet claimed. `just workbench-e2e-real "hello|identity|reconnect|request
recovery"` also passed after the fresh-root changes, preserving the seeded
existing-history packaged path; that is regression evidence only, not an
operator-visible selection or migration claim.

The worktree contained unrelated user changes before the `AGENTS.md` refactor;
they remain untouched. The pre-existing uncommitted additions to `AGENTS.md`
were consolidated rather than discarded.

On 2026-07-24, `full-spec` was merged into `main` as `74dc8ab` after a
fast-forward update from `origin/main`. The integrated tree passed
`devbox run -- just ci`; publication is pending the pre-push context gate after
  this final status refresh. The final Workbench grounding report and its two
  out-of-scope debt entries are intentionally included in the pending handoff
  commit, along with the current `prove_entry` source-line reference in the
  Workbench repository map.

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

- `SWE_SEED/crates/swe-seed-core/src/federation/{identity,producers,idempotency}.rs`,
  `{envelope,consume,mod}.rs`, `tests/convergence_t01_envelope.rs` — plan T01
  canonical-envelope/domain-identity enforcement + 22-test falsifier suite
  (builder evidence: `.agents/evidence/e2e/T01/t01-report.md`; independent
  confirmation pending).
- `.agents/evidence/e2e/T01/t01-report.md` — T01 builder evidence,
  falsifier→proof map, honest PENDING confirmation status.
- `.agents/status/e2e-current-status.yml` — new convergence status surface:
  one frozen-vocabulary verdict + resolvable evidence per requirement for all
  35 frozen requirements (Delta-0: 0 CONFIRMED / 30 PARTIAL / 5 ABSENT).
- `scripts/e2e-prereg-ids.sh`, `scripts/e2e-delta-report.sh`,
  `scripts/e2e-delta-check.sh` — plan T00 verification surface: frozen-ID
  derivation, deterministic Delta-0 report, mechanical matrix validation with
  report drift detection.
- `.agents/evidence/e2e/T00/delta0.md` — committed, drift-checked Delta-0
  report regenerated from the status file.
- `justfile` — e2e convergence gate aliases added (`e2e-check`, `e2e-test`,
  `e2e-lint`, `e2e-delta-check`, `e2e-delta-report`, `e2e-gate <TID>`); no
  existing recipe modified.
- `scripts/check-e2e-preregistration.sh` — new frozen-preregistration hash
  gate (SHA-256 of `.agents/specs/e2e-preregistration.yml` vs
  `source.spec.sha256` in `.agents/plans/e2e-plan.yml`; explicit PASS/FAIL
  verdict, never rewrites the stored hash).
- `justfile` — one added `[group('quality')]` recipe `e2e-prereg-check`
  delegating to that script; no existing recipe touched.
- `.agents/reports/workbench-completion-eval-inputs.md` — concrete independent
  evaluator invocation, real temporary-cell/sidecar setup, and identity
  fixture facts.
- `.agents/reports/workbench-completion-eval-exclusions.json` — exact
  owner-approved story and non-story exclusions for the Linux claim.
- `scripts/check-workbench-completion-eval-inputs.sh` and `justfile` — focused
  machine check and `just workbench-completion-eval-inputs-check` recipe;
  intentionally outside CI until Task 12 owns release aggregation.
- `.agents/CURRENT_STATUS.md` — this Task 0 handoff record and current DAG
  standing.
- `workbench/apps/desktop/src/guards/protectedAction.ts` — shared conservative
  renderer affordance guard over validated identity and source-backed readiness.
- `workbench/apps/desktop/src/pages/{ReadinessPage,ReadinessPage.test.tsx}` —
  case creation now blocks on unresolved identity, names unchanged effect, and
  exposes the identity-inspection next action.
- `workbench/apps/desktop/src/{hooks/useIdentity.ts,shell/{AppShell,GlobalHeader}.tsx}` —
  session-only choice among server-advertised actors, with every consumer
  re-deriving the role from the validated current identity view.
- `workbench/apps/desktop/src/{machines/caseAuthoringMachine.ts,hooks/{useApprovals,useDelegations}.ts}` —
  protected host calls carry only the selected actor id as bounded `actAs`.
- `workbench/apps/desktop/src/{hooks/useThoth.ts,pages/{ThothPage,ApprovalInboxPage,DelegationRoster,CaseCreationWorkbench}.tsx}` —
  every remaining recorded/protected action applies the same refusal and repair
  route; `thoth.ask` also receives only the bounded selected actor id.
- `crates/sea-forge-server/src/{identity.rs,sfwp/{readiness,approvals}.rs}` —
  typed committed source references, explicit unknown/stale freshness, resolved
  approval governance context, and structured identity refusals.
- `workbench/apps/desktop/src-tauri/src/bridge.rs` and
  `src/hooks/bridgeError.ts` — structured governed command errors survive host
  transport instead of collapsing to free text.
- `workbench/packages/contracts/{generated,schema}/` — regenerated TypeScript
  interfaces, AJV validators, and JSON schemas for the additive SFWP records.
- `workbench/apps/desktop/e2e/{readiness.spec.ts,tauriMock.ts}` and affected
  component tests — tests prove unresolved/resolved identities, actionable
  repair routes, the drawer-open capability click, and no protected host call
  on identity refusal.

- `AGENTS.md` — command-first root guide using only `just` commands; Workbench
  detail remains delegated to the existing nested guide.
- `justfile` — adds generic `crate-check` and `crate-test` fast-feedback recipes.
- `workbench/AGENTS.md` and `.agents/skills/building-sea-forge-workbench/SKILL.md`
  — route Workbench development and validation through `just`.
- `crates/sea-forge-server/src/bin/gen_sfwp_schema.rs` and its conformance-test
  diagnostics — point schema regeneration to `just workbench-contracts-generate`.
- `.agents/CURRENT_STATUS.md` — records this documentation-only handoff.

- Merge handoff: this status refresh records the `main` integration and the
  current source reference in
  `.agents/skills/building-sea-forge-workbench/reference/repository-integration.md`.
  `.agents/reports/2026-07-24-sfwp-grounding.md` records the completed
  18-family Workbench method-grounding map; `.agents/OBSERVED_DEBT.md` captures
  its two deferred findings without changing production behavior.

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

- Task 0 evaluator-input vertical slice: created the source-owned input sheet
  and machine-checkable owner exclusion fixture. The focused validator has
  teeth: it failed before either input existed and failed again after removing
  one declared exclusion; it passes with the restored exact fixture.
- Current DAG standing after the fresh Task 0 checks: N00 is `PASSING` once
  this task gate completes; N01 (sidecar/package inventory) is `PASSING`;
  N05 (per-request identity) and N06 (correlation/recovery) are `PARTIAL` at
  product level despite their focused protocol tests; N02–N04 and N07–N13 are
  not yet broadly proven by the new Ralph gate sequence. This is deliberately
  not a completion claim for any downstream journey.
- Task 1 identity-safe affordances: an unresolved socket identity blocks every
  current protected renderer action with `identity_unresolved`, an unchanged
  effect, and an actionable `/admin` repair route. A selected actor is held
  only in session storage, accepted only when it remains in validated
  `identity.get.available`, and passed to the host as bounded `actAs`; no role
  or actor claim is renderer-authored. The host revalidates identity for every
  protected request.
- Task 1 spendable readiness actions: resolved identity reaches `/cases/new`;
  “Inspect all capabilities” focuses the currently validated readiness
  capability projection in the evidence drawer even when that drawer is open.
- Task 2 source-truth slice: replaced readiness code citations with typed
  `SourceRecordRef` values (`ledger_id`, `entry_id`, record kind/id, digest,
  freshness, rebuild standing). The producer now validates the server's actual
  project root, not its parent; an uninitialized cell and an unproven endpoint
  are `unknown`, while a stale snapshot is `stale`/rebuild-required. The
  renderer opens the committed ledger reference rather than synthetic citation
  evidence. Approval rows now optionally project their verified case-ledger
  request/decision chain with reason, policy, boundary, requester, operation,
  evidence, expiry, side-effect standing, and next lawful steps; missing chain
  remains visibly unresolvable rather than invented.
- Task 2 bridge and purpose completion: the approval view now exposes the
  committed request context (including purpose and resource) instead of a UI
  summary. `sfwp_command` returns a structured refusal across the Tauri boundary
  (`error_class`, `no_side_effect`, `next_lawful_action`), and the renderer
  retains those fields as `BridgeGovernedError`.
- Task 3 automation setup: pinned `tauri-driver 2.0.6` is installed locally
  under ignored `workbench/.tools/`; `just workbench-e2e-real [filter]` now
  preflights Linux WebKit plus an isolated Xvfb display before packaging, seeds
  a unique temporary cell through real operations, allocates a fresh native
  driver-port pair per run (and rejects an exited driver before probing), and
  uses a dependency-free W3C client to drive the compiled Workbench, capture
  DOM/window/SFWP artifacts, then clean up only processes it started. `just workbench-e2e-agent-browser`
  is the default browser-only desktop `e2e` command; it starts a real Vite
  renderer without a Tauri IPC injection and proves the no-bridge state fails
  closed. The existing Playwright suite is retained only as `e2e:mocked` speed
  evidence.

- Reconciled the root agent guide with the current `justfile`; removed stale
  implementation-status claims, circular Copilot precedence, repeated guidance,
  obsolete async-boundary wording, and direct Devbox/Cargo commands. Retained
  the user's design, naming, semantic-density, encapsulation, and layer-boundary
  rules in condensed form.
- Added `just crate-check <crate>` and `just crate-test <crate> [filter]` so
  focused Rust feedback stays behind the repository command surface.
- Added `just workbench-tauri-dev`, `just workbench-host-build`, and
  `just workbench-contracts-generate`, plus `just workbench-skill-check`, to
  close the nested Workbench guide's direct-command gaps.

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

- Task 0 focused evidence: `just workbench-completion-eval-inputs-check`
  passes; it failed with `missing ...eval-inputs.md` before inputs were added
  and with an invalid fixture after removing exclusion `16.3`. `just
  workbench-package-inventory` passes for the current `.deb` (sidecar present,
  no JavaScript runtime, no source maps). `just crate-test sea-forge-server
  identity`, `just workbench-contracts-gate`, and `just workbench-tauri-test`
  were rerun without a command failure; the latter covers U-06 sidecar
  supervision and bridge recovery. After this status update, `just
  context-check`, `just check-fast` (format + workspace typecheck), `just
  workbench-contracts-gate`, `just workbench-completion-eval-inputs-check`,
  and `git diff --check` all pass.
- Task 1 TDD evidence: `ReadinessPage.test.tsx` gained the unresolved-identity
  regression, which failed against the former readiness-only action check.
  After the shared guard landed, `just workbench-check` completed its contracts,
  host, renderer typecheck, build, and test sequence without a reported
  failure.
- Actor-session evidence: `GlobalHeader.test.tsx` proves explicit actor choice;
  `ApprovalInboxPage.test.tsx` proves `operator_b` is forwarded as `actAs` to
  the closed host bridge. `just workbench-check` passed after the selection and
  forwarding slice.
- Task 1 final evidence: focused component tests (34 assertions) pass for
  case commit, approvals, cancellation, Thoth, and repair routes; the complete
  desktop suite passes through the final Workbench gate; Playwright `bun run e2e -- --grep "identity|Inspect
  all capabilities"` passes 3 scenarios. `just crate-test sea-forge-server
  identity`, `just workbench-tauri-test`, `just check`, `just test`, `just
  proof`, and final `just workbench-check` all completed without a reported
  failure. Cargo emitted the pre-existing `license`/`license-file` manifest
  warnings. These browser scenarios use the declared Tauri mock and are only
  Task 1 interaction evidence, never a real-stack claim.
- Task 2 focused evidence: `just workbench-contracts-generate` regenerated 57
  schemas; `just crate-test sea-forge-server readiness` covers uninitialized,
  committed, and stale snapshots; `just crate-test sea-forge-server approval`
  passed 3 unit, 6 approval conformance, 2 identity, and relevant escalation
  tests. Focused desktop tests passed: `ReadinessPage.test.tsx` plus
  `protectedAction.test.ts` (9 assertions), and `ApprovalInboxPage.test.tsx`
  (11 assertions). Desktop typecheck passed with two pre-existing lint warnings
  in `router.tsx` and `useIdentity.ts`; `just fmt-check` and `git diff --check`
  passed after this slice.
- Task 2 approval-chain conformance: the real escalated case episode now proves
  `approval.list` resolves its row to committed `approval_request` and
  `authority_decision` entries with SHA-256 digest and
  `not_executed_pending_approval`; `just crate-test sea-forge-server
  an_escalated_episode_opens_an_approval_and_runs_nothing` passes.
- Task 2 final focused evidence: `just workbench-contracts-generate`, `just
  workbench-tauri-test`, the `identity`, `readiness`, `approval`, denied-episode,
  and `stale_precondition_on_approve_is_rejected_with_no_side_effect` server
  slices, and the focused ApprovalInbox/bridge-error renderer tests all passed.
  `bun run --cwd workbench/apps/desktop check` passed. The prescribed
  `just crate-test sea-forge-server conformance` filter selects no tests (Cargo
  filters function names, which do not contain that word), so the named focused
  conformance tests above are the actual coverage. `just workbench-contracts-gate`
  correctly fails while regenerated contracts remain uncommitted; no index
  manipulation was used to hide that drift.
- Task 3 automation evidence: `agent-browser 0.33.2` with Chrome 151 is
  installed; `just workbench-e2e-agent-browser` passes the no-bridge assertion
  and a `#main-content` accessibility scan. Owner-installed `webkit2gtk-driver`
  and `xvfb`, plus the pinned ignored `tauri-driver 2.0.6`, support the real
  path. A strict Tauri CSP initially left the packaged WebKit renderer blank:
  Astryx's `defineTheme` extension and the generated AJV validators both
  required runtime code generation. The application now consumes the prebuilt
  neutral projection with static scoped SEA Forge tokens, and the contract
  generator emits Ajv standalone validators with unchanged typed `validate`
  exports (no browser-time `ajv.compile()` or `Function(...)`). The focused
  prebuilt-theme and mockup-fidelity tests pass. `just workbench-package`, the
  direct real WebKit smoke, and the complete `just workbench-e2e-real` recipe
  pass: it packages the app, seeds real records, launches the real
  server/socket and compiled app under isolated Xvfb, confirms the mounted
  readiness document, then exercises SFWP hello, identity, reconnect, and
  request recovery. The runner only observes the loaded application; temporary
  diagnostic module reruns were removed. `bash -n` for both runners, Python
  compilation, `just workbench-e2e-agent-browser`, `just context-check`, and
  `git diff --check` pass.

- `git diff --check` for the command-surface paths: passed.
- `just --show` for `crate-check`, `crate-test`, `workbench-tauri-dev`,
  `workbench-host-build`, `workbench-contracts-generate`, and
  `workbench-skill-check`: parsed as expected.
- `just crate-check sea-forge-core`: passed (Cargo emitted pre-existing
  `license`/`license-file` manifest warnings).
- `just crate-test sea-forge-core ids`: passed (1 selected test passed).
- `just check-fast`: passed (same pre-existing manifest warnings).
- `just workbench-skill-check`: passed.
- Contract regeneration and package/desktop-launch recipes were not run because
  regeneration mutates a user-modified generated zone and the latter recipes
  are outside this command-surface change.
- No build or product tests run: only Markdown agent instructions changed.

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

- Continue the plan's next bounded Workbench task. Task 3's real-stack native
  automation gate is satisfied; the installed Playwright suite remains only
  mocked speed evidence and is not used for the integrated claim.

- No remaining work for the Just command-surface refactor.

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

- No external blocker for Task 0. Native evaluator automation remains an
  environment capability to be assessed in Task 3; the evaluator input names
  the required integrated behavior and explicitly forbids a mocked fallback.

- **Task 3 native OS prerequisite:** resolved. `tauri-driver 2.0.6` is installed
  in ignored Workbench tooling; `WebKitWebDriver` and `xvfb-run` are present.
  The runner uses its own software-rendered X display and the compiled renderer
  now mounts successfully without weakening the Tauri CSP. The existing
  Playwright harness remains explicitly mocked and is not a substitute for the
  passing real-stack gate.

- No blockers for the Just command-surface refactor.

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

- Task 0 records the owner-approved exclusions as a JSON evaluator fixture,
  rather than duplicating an informal list in the evaluator prompt. The
  validator requires its exact story-ID set, nonempty owner reasons, Linux
  claim, package/start commands, and no protocol placeholders. It is a local
  Workbench command only; Task 12 is the authorized CI aggregation point.
- Task 1 keeps actor selection renderer-local and session-only, while the host
  resolves and validates the authoritative actor again for each protected
  request. The shared guard is intentionally a conservative affordance check;
  it never decides policy or authority.

- Keep the root guide at approximately 150 lines, expose only `just` commands,
  and route Workbench-specific commands and generated-zone detail through
  `workbench/AGENTS.md`.
- Treat `justfile` as the canonical recipe implementation. Document aggregate
  gate coverage and exclusions instead of copying recipe bodies into the guide.

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

## Task 8 — the run record (epic 12.1, 12.3, 12.4; unblocks 7.4, 7.5, 9.x, 13.x)

**Slice chosen for blast radius.** Of the epic's open journeys, the run record
was the one whose absence blocked the most downstream work: every other view
already emitted run ids that resolved to nothing. Implementing it turns four
existing surfaces from display-only into navigable, and gives journeys 9
(execution evidence), 11.8 (failure diagnosis), 12 (audit truth), and 13
(capability from settlements) the record they all have to hang off.

**Server** — `crates/sea-forge-server/src/sfwp/run_views.rs`, additive per
ADR-003:
- `run.list` (optional `case_id` scope) over `<root>/runs/*`, cross-indexed
  against `cases/*/case.json` for ownership. Runs no case claims stay listed as
  unclaimed rather than filtered out (epic 11.6 — process death must not hide
  work).
- `run.get` — the linked view: plan item, authority projection, criteria paired
  against the settlement basis, settlement detail, declarations, evidence rows,
  trace rows, and a presence inventory of every canonical run file.
- Both advertised via `IMPLEMENTED_METHODS`; eleven new contract types added to
  `SCHEMA_TYPES` and `gen_sfwp_schema.rs` (which had silently drifted apart —
  eight Task 7 types were in the generator but not in `system.get_schema`; both
  lists are now complete).

**Frontend**:
- `hooks/governedQuery.ts` — one invoke → reject-governed-error → AJV-validate
  path, replacing three copies of `describeAjv` and eight bespoke throw sites.
  The order is load-bearing: a governed error body can never satisfy a view
  schema, so validating first would report every denial as a contract failure.
  `useCases`/`useApprovals` now route through it (which also closed a real gap —
  `approval.list` never checked for the error envelope at all).
- `pages/standing.ts` — the standing→pill maps, shared rather than copied,
  because each encodes a governance rule (`completed` is `degraded`, never
  `ready`) that would eventually be enforced in only one copy.
- `pages/RunRecordPage.tsx` at `/runs/$runId`; `pages/EvidencePage.tsx` replaces
  the `UnbackedSurface` stub with a `run.list`-backed index; the case horizon's
  `Episodes: N` count became one link per attempt.

**Gates**: `cargo fmt --all --check` clean; `devbox run -- just check` — all
gates green (fmt, clippy, cargo-deny, gitleaks); `cargo test --workspace
--all-features --locked` — all suites pass; frontend `bun run check` clean and
`bun run test` — 79 passed / 14 files.

**One flake observed, not caused here**: `sea-forge-cli`'s
`kill_9_leaves_a_valid_jsonl_prefix_without_capability_corruption` failed once
under full-workspace parallel load and passes in isolation. It is a `kill -9`
timing test and no CLI code was touched by this task.

**Next spendable slice**: `9.3`/`9.8` (follow non-agent execution, preserve
evidence from every termination) now have their record surface and need only
artifact/stdout access; or `13.4` (capability records), which can read the
declaration rows this task already projects.

**Iteration entry point**: `.agents/WORKBENCH-SLICE-PROMPT.md` holds the
idempotent prompt for advancing the workbench one slice at a time. It derives
state from this file, `IMPLEMENTED_METHODS`, and `router.tsx` on every run
rather than from a remembered plan, so it selects the same next slice against an
unchanged tree and never re-does shipped work.

## Task 9 — the governed asset catalog (epic 4.1, 4.5, 4.7, 4.8; unblocks 4.6, 9.4, 9.5, 6.7)

**Slice chosen for blast radius.** Three surfaces still rendered copied
specification data (Thoth, Assets, Models). Assets was picked over Thoth because
of what sits *downstream*, not because journey 4 is longer:

- Journey 9's delegation stories (9.4 configure an agent task, 9.5 run HTTP/ACP
  agents, 9.7 monitor and intervene) all begin with "choose an eligible
  endpoint". The kernel has spoken `agent_list`, `agent_probe`, `delegate`, and
  `cancel_delegation` since M12 — the capability was real and unreachable from
  the workbench, the same shape as the `approval.decide` gap Task 7 closed.
- The Assets specimen was the *worst-behaved* of the three. Thoth and Models
  fabricate layout; Assets fabricated an availability ladder
  (`Available`/`Installed`/`Declared`) that `AgentEndpointConfig::validate`
  explicitly refuses to let even *configuration* assert, because that ladder is
  evidence-derived. The renderer was claiming what the kernel rejects.

Runner-up was `thoth.ask` (journey 3, nine stories). Passed over because it is
not one slice: it needs a typed response contract, a disclosure-gating decision,
and an AG-UI stream — and nothing downstream is blocked by its absence, since a
Thoth answer is an inspection aid that authorizes nothing.

**Server** — `crates/sea-forge-server/src/sfwp/assets.rs`, additive per ADR-003:
- `asset.list` over three sources the kernel already owns: materialized
  templates (`<root>/templates/*.yaml`), configured agent endpoints
  (`server.yaml`), and the extension registry.
- Endpoint standing folded from committed records, never asserted: `declared`
  (configured only) → `probed` (a registry runtime-adapter entry, which
  `agent_probe::register_endpoint` writes only after an allowed authority
  decision, or a probe run that settled) → `demonstrated` (an *accepted* probe
  settlement). The most recent probe decides, so an endpoint that has started
  failing does not keep an old demonstration.
- Advertised via `IMPLEMENTED_METHODS`; `AssetListResult`/`AssetRow`/`AssetKind`
  added to `SCHEMA_TYPES` and `gen_sfwp_schema.rs`.

What the design had to get right:

- **Three vocabularies stay three.** The obvious simplification is one shared
  availability enum. It would make a quarantined extension and an unprobed
  endpoint render identically — erasing *refused* vs *not yet proven*, which is
  the entire content of epic 4.8. Each row carries its own kind's word verbatim
  (`materialized` / `declared|probed|demonstrated` / `active|disabled|
  quarantined|superseded`) and the UI maps each separately.
- **Standing and blocking are separate fields.** They answer different
  questions. An extension can be `active` and still refused because its trust
  level is quarantined; an endpoint can be `declared` — proven nothing — and be
  perfectly lawful to probe. One field would have to drop one of the two facts.
- **Absence is reported, not dropped.** `case.entry_options` silently skips a
  template it cannot parse and `agent_probe::list` silently drops an endpoint
  that will not snapshot, so a broken file reads as "no such asset" in both.
  `asset.list` lists it with the reason and names the source in `unreadable`.
- **The floor stays the floor.** A configured endpoint with no evidence is
  `declared` with an empty `evidence_refs` — an honest empty list, not a
  missing one, and never upgraded on the strength of being configured.

**Frontend**:
- `hooks/useAssets.ts` over `queryGoverned`. Deliberately *not* event-
  invalidated: no `KNOWN_EVENT_KINDS` entry announces a template, endpoint, or
  extension change, so subscribing to the case/run taxonomy would look like
  liveness without being it. The page says so and offers an explicit re-read.
- `pages/AssetCatalogPage.tsx` replaces the specimen `AssetsPage`. Three
  kind-scoped tables rather than one — a merged table would put the three
  vocabularies in one column and invite reading them as one ladder. `run:<id>`
  evidence refs link to the run record; ledger ids render as ids rather than as
  links that go nowhere.
- `pages/standing.ts` gains `ASSET_STANDING_VARIANT`; nothing in it maps to a
  blocked variant, because blocking is the other column.
- `SURFACED_METHODS` gained `asset.list` — **and `run.list`/`run.get`, which
  Task 8 shipped, wired, and rendered but never listed.** `/admin` had been
  reporting two live methods as "implemented, not surfaced" ever since.

**Neatcode judgment.** Two shared things, both load-bearing; nothing else.
`run_views::read_json` became `pub(crate)` rather than being copied — the two
projections must agree that a half-written record is not a record.
`ASSET_STANDING_VARIANT` lives with the other standing maps for the reason
stated there. Explicitly *not* built: an `asset.get` detail method (the row
carries what the detail panel needs), a server-side `kind` filter (the catalog
is small and a second place to decide "which assets exist" is a liability), and
a shared template-enumeration helper (two readers, not three — logged instead).

**Gates**: `cargo fmt --all -- --check` clean; `devbox run -- just check` — all
gates green (fmt, clippy, cargo-deny, gitleaks, no leaks); `cargo test
--workspace --all-features --locked` — all suites pass, zero failures. Two
tests are **skipped**, both pre-existing and unrelated: `self_invoke_noop_pass`
and `self_invoke_noop_fail` are `#[ignore]`d in the CLI suite. The Tauri host
crate is outside the workspace and was run separately (`cargo test` in
`src-tauri`): 2 lib + 4 integration tests pass. Frontend `bun run check` clean
apart from the pre-existing `router.tsx` fast-refresh warning;
`bun run --cwd apps/desktop test` — 85 passed / 15 files (was 79 / 14).

**Tests added**: 12 in `crates/sea-forge-server/tests/conformance_assets.rs`
(ladder floor, accepted → demonstrated, rejected → probed-and-blocked-with-its-
basis, most-recent-probe-wins, no evidence leakage between endpoints, registry
registration reaching `probed` but not `demonstrated`, no double-listing of a
registered endpoint, quarantined trust blocking an `active` extension,
unparseable template reported not dropped, empty cell staying empty, catalog
advertised as `inspect`); 3 unit tests in `sfwp/assets.rs`; 9 in
`pages/AssetCatalogPage.test.tsx`; 1 in the host bridge; and one closing a
documented debt — `generated_schemas_are_committed_and_current` now asserts
`SCHEMA_TYPES` equals the generator's emitted filenames, so the comment claiming
they "never drift" is finally enforced.

**Next spendable slice**: `9.4` (configure an agent task) is now unblocked and
is the highest-leverage follow-on — endpoints are enumerable with real standing,
`delegate` exists in the kernel, and the missing piece is a job-contract
inspection view plus a `delegation.preview`-shaped inspect method. The
alternative is `thoth.ask`'s response contract (journey 3), which remains the
largest single unclaimed block but is at least two slices wide.

---

## Task 10 — the delegation job contract (epic 9.4; unblocks 9.5, 9.7, 9.2)

**Slice**: `delegation.preview` — an SFWP inspect method that projects the
complete job contract a `delegate` with these exact inputs would run under, plus
a `/delegate` surface that reads it.

**Why this one.** The runner-up was `thoth.ask` (journey 3, nine stories), passed
over for the same reason as last slice: it is a response contract *plus*
disclosure gating *plus* an AG-UI stream, which is at least two slices, and
nothing downstream is blocked by its absence. 9.4 was picked because it is the
step every remaining journey-9 story starts from — 9.5 (provider parity), 9.7
(monitor and intervene), and 9.2 (the granted sandbox) all presuppose a
configured, inspectable agent task — and because the kernel's `delegate` has
been reachable-but-blind since M12: the only way to learn what a delegation
would do was to run one, which is exactly the side effect being decided about.

**Settles** 9.4. **Unblocks** 9.5, 9.7, 9.2.

**Server** (`crates/sea-forge-server/src/sfwp/delegation_preview.rs`):
- `DelegationPreviewParams` takes *exactly* the inputs `Request::Delegate`
  accepts. A preview knob the command cannot take would describe a delegation
  nobody can run.
- The contract carries provider kind, endpoint digest, resolved model and
  transcript retention (each with a `ValueSource`), instruction hash + size
  against the endpoint's own cap, response cap, timeout, turn cap, token budget,
  the authority action kind, and a `contract_digest`.
- `eligible` means only "no precondition known at preview time is unmet". The
  method deliberately does **not** evaluate authority: a verdict with no ledger
  entry behind it would be an unrecorded grant. It names the gate (`agent_task`)
  and stops.
- Standing, evidence refs, and evidence-derived blocking come from
  `sfwp::assets` rather than being re-derived, so `/assets` and `/delegate` can
  never disagree about whether an endpoint is usable.
- An unconfigured endpoint gets **no** standing word rather than being demoted
  to `declared` — absence reported, not inferred onto a ladder it was never on.
- Absent `token_budget` is omitted, never zeroed: a zero budget and no budget are
  opposite instructions to the runner.

**Bug found and fixed.** `delegate_inner` passed `..Default::default()` for
`DelegationRequest::transcript_retention`, whose doc comment says the caller has
already resolved it. Every socket-issued delegation was therefore pinned to
`summarized`, silently ignoring an endpoint that had asked for `full`
transcripts (`case_dispatch` resolved correctly; the standalone verb did not).
Fixed to call `TranscriptRetentionMode::resolve` through the same chain. This
was in scope rather than deferred: a preview reporting `full (from the endpoint
descriptor)` while execution ran `summarized` would be a projection lying about
truth, which is the one thing this method exists not to do.

**Frontend**: `hooks/useDelegationPreview.ts` (parameterised — the request is
the query key, and an uncommitted request issues no query); `pages/
DelegationWorkbench.tsx` at `/delegate`, plus a sidebar entry. Blocked endpoints
stay in the picker: hiding one would report an endpoint that exists and is
refused as one that is absent, and the refusal is what the operator came to
read. Editing after reading marks the contract as describing the earlier
request rather than silently re-attributing it to the current form. There is no
"Run delegation" button — `delegate` is a protected command and belongs behind
`ProtectedActionButton` with its own preconditions, which is the next slice.

**Neatcode judgment.** Two extractions, both load-bearing, no new abstractions.
`delegation::check_preconditions` now holds the four request-shape rules that
were about to exist in two copies — execution takes the first unmet rule, the
preview lists all of them, and neither can drift. `action_for_delegation` became
`pub(crate)` so the preview reports the *same* authority action that will be
submitted, including the descriptor hash, rather than a second spelling of it.
`VALUE_SOURCE_LABEL` joined the existing standing maps and is deliberately not a
pill: provenance is different, not better or worse, and a `ready`/`degraded`
treatment would rank "you chose this" above "the endpoint chose it". Explicitly
*not* built: a read-only authority dry run (see above), a `delegation.commit`
envelope, a debounced live preview (an explicit read plus a staleness marker is
less code and does not manufacture contracts nobody chose), and a new CSS module
(`RunRecordPage.module.css` already carried the layout).

**Gates**: `cargo fmt --all -- --check` clean; `devbox run -- just check` all
green; `cargo test --workspace --all-features --locked` — exit 0, **101 suites,
762 passed, 0 failed, 4 ignored**.
**Those four are the skipped ones**, all `#[ignore]`d, all pre-existing and untouched by
this slice: `self_invoke_noop_pass` and `self_invoke_noop_fail` (self-invocation
fixtures in `sea-forge-case-runner`, deliberately never executed by a normal
run) and `t16_1_real_acp_host_release_gate` / `t16_6_real_swe_seed_release_gate`
(release gates needing operator-supplied real ACP / SWE_SEED hosts). No flakes
observed this run. Correction to the Task 9 note above: it said two skipped
tests in the CLI suite — there are four, and the `self_invoke_*` pair lives in
`sea-forge-case-runner`, not the CLI. The Tauri host crate is outside the
workspace and was run separately in `src-tauri` (3 lib + 4 integration, pass). Frontend
`bun run check` clean apart from the pre-existing `router.tsx` fast-refresh
warning; `bun run --cwd apps/desktop test` — 98 passed / 16 files (was 85 / 15).

**Tests added**: 14 in `crates/sea-forge-server/tests/conformance_delegation_
preview.rs` (contract projection; **nothing written to runs/cases/ledger**;
authority named but never decided and no verdict field present; unconfigured
endpoint has no standing and no contract; every unmet precondition reported not
just the first; over-long instruction blocked against the endpoint's own cap;
instruction reported by hash and size and never echoed; requested vs endpoint
model provenance; retention resolved endpoint-then-cell; a rejected probe
blocking the preview with the *same words* `asset.list` uses; accepted probe
evidence carried through; contract digest moving with request and descriptor;
absent token budget omitted not zeroed; advertised as `inspect`); 4 unit tests
in `sfwp/delegation_preview.rs`; 1 in the host bridge (flattened params, no
nulls); 13 in `pages/DelegationWorkbench.test.tsx`.

**Next spendable slice**: `9.7` (monitor and intervene in agent dialogue) —
`delegate`/`cancel_delegation` both exist in the kernel and `agent_run.delegated`
is already an event kind, so the missing piece is a `delegation.list`-shaped
inspect method over live/finished delegations plus a cancel path behind
`ProtectedActionButton`. That would also give `/delegate` its command half. The
alternative remains `thoth.ask`'s response contract (journey 3): still the
largest unclaimed block, still at least two slices wide.

---

## Task 11 — the delegation roster and per-run cancel (epic 9.7, partially; unblocks 9.8)

**Slice**: `delegation.list` — an SFWP inspect roster joining the server's live
delegation handles with the committed run records, plus a `/delegate` section
that cancels exactly one delegation.

**Why this one.** `cancel_delegation` has been a kernel verb since M12 and the
Tauri host has carried `SfwpCommand::CancelDelegation` since Task 3 — but
nothing could enumerate what there was to cancel. That is precisely the shape of
the `approval.decide` gap Task 7 closed: a capability whose targets are
undiscoverable is not a capability an operator has. It also completes the
command half of `/delegate`, which Task 10 named as its own follow-on.

**Scope honesty — 9.7 is settled in part, not in full.** The story asks for
"bounded turn, token, streaming, permission, and continuation state" *during*
the dialogue. The kernel does not expose that: `DelegationHandle` carries only a
case id and two atomics, and the per-turn loop publishes no event. Turn and tool
counts therefore appear only once `transcript-evidence.json` is written — after
termination. The roster reports them as **absent while running** rather than as
`0`, and the kernel-side gap is logged rather than papered over. What *is*
settled is the control half: see every delegation, and cancel one without
touching its siblings. **Unblocks** 9.8 (terminations are now reachable from a
roster rather than only from a run id someone already had).

**Server** (`crates/sea-forge-server/src/sfwp/delegations.rs`):
- Joins two sources that cannot be merged — the in-memory handle map (live, not
  durable) and `runs/<run_id>/` (durable, only once terminated).
- `DelegationStanding` is a closed four-variant lifecycle vocabulary, distinct
  from the existing execution and settlement ones. The load-bearing variant is
  `unresolved`: a delegation run with no settlement *and* no live handle. That
  is what a server restart mid-episode produces, and the kernel cannot say what
  happened to it — so neither does the projection. `cancelled` would invent a
  request nobody made; `failed` would invent a settlement nobody recorded.
- Standing is not settlement. A delegation can be `settled` and rejected, or
  `settled` after having been cancelled; the verdict is its own field carrying
  the settlement record's own word.
- `cancellable` is per-run by construction — the cancel flag lives on that run's
  own handle — and the UI has one button per row with no bulk affordance.

**Frontend**: `hooks/useDelegations.ts` (roster + cancel, event-invalidated
unlike the asset catalog, because both `agent_run.*` kinds move a row);
`pages/DelegationRoster.tsx` mounted on `/delegate` with per-row
`ProtectedActionButton`. A successful cancel is reported as **requested**, never
as "cancelled" — the kernel records a control request and the episode still
terminates on its own terms. Reporting the outcome in place of the request would
be the execution-equals-settlement conflation the epic forbids everywhere else.

**Two bugs found and fixed en route.**
1. `hooks/eventKinds.ts` listed three event kinds while claiming to be "verified
   against its publish sites"; the server has been emitting five since M12
   (`agent_run.delegated`, `agent_run.cancellation_requested` were missing).
   Nothing rendered stale because the narrowing is asymmetric and unknown kinds
   invalidate — which is exactly why it went unnoticed. Now listed, classified,
   and pinned by a new `eventKinds.test.ts`.
2. **My own Task 10 test was partly vacuous.** `preview_creates_no_run_case_or_
   ledger_entry` checked that a directory named `ledger` stayed empty; the
   kernel's path is `ledgers`, so that third of the assertion could never fail.
   Both that test and the new roster equivalent now diff the whole cell tree
   before and after, which cannot be fooled by a name the author did not think
   of.

**Neatcode judgment.** Two extractions, both at thresholds previously named, and
no new abstractions. (1) `run_views::run_dirs` and `case_index` became
`pub(crate)`: run enumeration had reached its third hand-rolled copy — the exact
trigger recorded in `OBSERVED_DEBT.md` last slice — and "a run belongs to the
case that claims it" is a rule two views must not disagree about. Each caller
keeps its own readability policy, so nothing gained a policy parameter.
(2) `hooks/useGovernedEventInvalidation.ts` replaces **five** hand-copied
`listen("sfwp://event")` blocks that differed only in predicate and query key.
They each carried two rules subtle enough to drift: `event?.payload` (a throwing
listener tears down the subscription) and the `disposed` flag (a late-resolving
`listen()` promise leaks a listener after unmount). `useOperationsStream` was
deliberately left alone — it consumes frames as data, which is a different job.
Explicitly *not* built: a `delegation.get` detail method (the row carries what
the roster needs and `run.get` already resolves the rest), a live turn counter
(the kernel has nothing to report), and a bulk-cancel affordance.

**Gates**: `cargo fmt --all -- --check` clean; `devbox run -- just check` all
green (fmt, clippy, cargo-deny advisories/bans/licenses/sources, gitleaks — 291
commits scanned, no leaks); `cargo test --workspace --all-features --locked` —
exit 0, **102 suites, 776 passed, 0 failed, 4 ignored** (was 762 passed at Task
10). The four ignored are unchanged and pre-existing: `self_invoke_noop_pass`
and `self_invoke_noop_fail` (self-invocation fixtures in
`sea-forge-case-runner`) and `t16_1_real_acp_host_release_gate` /
`t16_6_real_swe_seed_release_gate` (release gates needing operator-supplied real
ACP / SWE_SEED hosts). No flakes observed. The Tauri host crate is outside the
workspace and was run separately in `src-tauri`: 4 lib + 4 integration tests
pass. Frontend `bun run check` clean apart from the pre-existing `router.tsx`
fast-refresh warning; `bun run --cwd apps/desktop test` — 116 passed / 18 files
(was 98 / 16).

**Tests added**: 12 in `crates/sea-forge-server/tests/conformance_delegations.rs`
(empty roster on a fresh cell; **no settlement + no handle → `unresolved`**;
standing and settlement as separate facts; dialogue reported against its bounds;
a cancelled episode settling with `cancelled` as its *termination*; non-agent
runs excluded; case attribution via the shared index; absent token budget
omitted not zeroed; cancelling an inactive delegation refused, agreeing with
`cancellable: false`; unsettled sorting ahead of settled; advertised as
`inspect`; **a full tree diff proving the read writes nothing**); 2 unit tests in
`sfwp/delegations.rs`; 1 in the host bridge (roster verb + cancel command
asserted together, since they are two halves of one control loop); 15 in
`pages/DelegationRoster.test.tsx`; 3 in the new `hooks/eventKinds.test.ts`.

**Next spendable slice**: `9.8` (preserve evidence from every termination) —
the roster now reaches every terminated delegation, `TranscriptEvidence` already
records termination, transcript hash, and harvested refs, and `run.get` resolves
the rest; the gap is a termination-complete evidence view that proves failure
cannot erase the record. The alternative remains `thoth.ask`'s response contract
(journey 3): still the largest unclaimed block, still at least two slices wide.
