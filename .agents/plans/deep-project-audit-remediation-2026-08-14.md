# Deep Project Audit Remediation Plan — 2026-08-14

Source of truth: `.agents/reports/deep-project-audit-2026-08-14.md` (base) and
`.agents/reports/deep-project-audit-supplement-2026-08-14.md` (supplement).
Revision audited: `df80d73` (matches current working tree HEAD).

This plan inventories every finding, groups shared root causes, orders the work
by dependency, and records decisions and verification. It is a living record;
status is updated as remediation proceeds.

---

## 1. Inventory and mapping

Legend for disposition columns (filled in as work proceeds):
`R` = REMEDIATED, `AR` = ALREADY RESOLVED, `P` = PARTIALLY REMEDIATED,
`D` = DEFERRED (spec decision), `NA` = NOT APPLICABLE, `T` = TODO.

### Base audit findings

| ID | Severity | Root-cause class | Component(s) | Remediation intent | Status |
| --- | --- | --- | --- | --- | --- |
| F-01 | | High | A: path canonicalization | authority `invalid_relative_path`/`path_denied`/`hard_denied`; sandbox `validate_relative_path`/`safe_join`; planner `valid_relative_path` | Reject `//`/`.`/trailing-`/` spellings; normalize before glob; segment-aware `path_prefix`; anchor built-ins `**/.env*`, `**/.git/**` | R |
| F-02 | High | B: error-path completion | server `case_dispatch.rs` dispatch loop | Pre-validate batch dispatchability before spawn; drain-and-settle on mid-loop error | R |
| F-03 | Med-High | B: state machine | `case_dispatch.rs` escalation; `case-runner` Escalated→ItemFailed; planner required-failed→Terminate | Park (not terminate) escalated required items; approval re-drive; no `item_failed` label | R |
| F-04 | Med-High | C: crash-consistency | ledger `append_under_lock` order; `quarantine_incomplete_tail` | Write entries before mmr; rebuild mmr as derived state on repair | R |
| F-05 | Med | C: panic-on-corruption | ledger `prove_entry` | bounds check vs `entries.len()` | Validate `entries.len()` vs `mmr.leaf_count`; typed error | R |
| F-06 | Med | | C: panic-on-corruption | ledger `signing.rs` `base64_decode` | Reject code points ≥ 256 before table index; typed error | R |
| F-07 | Med | F: async/blocking | server `fire_notify` | Drain pipes concurrently, timeout, `spawn_blocking` | R |
| F-08 | Med | D: role propagation | server identity gate → `Actor` construction sites | Propagate verified `ActorRole` into all `Actor` construction | T |
| F-09 | Med | A: path canonicalization | workbench `drafts.rs` `draft_path` | Validate `draft_id` grammar `^[A-Za-z0-9_-]{1,64}$` | R |
| F-10 | Med | E: evidence integrity | CLI `resume.rs` synthetic `Completed` | No fabricated `ExecutionResult` for write-only items | R |
| F-11 | Med | B: recovery | CLI `resume`/`case reopen` | Recovery verb for stranded `Active` cases | T |
| F-12 | Med | | G: root convention | cell/bundle/self-model/thoth/transcript_seal | Unify to state-root convention; fail-closed export | T |
| F-13 | Med | H: governance metadata | capability `promotion.rs` substring match | Exact `attempted_capability`/plan-item mapping; reject `*` | T |
| F-14 | Med-Low | A: path canonicalization | ledger `LedgerStream::open`; CLI resume/case/adopt ids | Validate id grammar at crate boundary before fs mutation | R |
| F-15 | Med-Low | A: path canonical | server `case.preflight`/`commit` `template_ref` | Validate name/version charset before join | R |
| F-16 | Med-Low | A: trust boundary | server policy/plan path resolution | Constrain to workspace-relative under cell root | T |
| F-17 | Low-Med | A: path canonicalization | settlement `settle()` quarantine `plan_item_id` | Validate `plan_item_id` grammar before write | R |
| F-18 | Low-Med | F: resource bounds | agent `acp.rs` episode | Wall-clock deadline; bound `tool_calls`; count tool-call turns | R |
| F-19 | Low-Med | F: resource bounds | server `SubmitPayload.timeout` | Clamp timeout at protocol boundary | R |
| F-20 | Low | E: evidence classification | sandbox `jail.rs` stderr heuristic | Annotate `suspected` or use exit-code/EACCES | T |
| F-21 | Low | I: operability | server `status` in-memory map | Rebuild map at startup / answer from disk | R |
| F-22 | Low | C: key hygiene | ledger `signing.rs` | 0600-at-create; split verify/sign; symlink-check; no self-witness | P |
| F-23 | Low | D: role propagation | thoth `service.rs` actor id→role | Map verified role before matching grants | T |
| F-24 | Low | F: async/blocking | server/ledger blocking + O(n) scans | `spawn_blocking`; incremental tail; caps | T |
| F-25.a–s | Low | mixed hardening | various | Batched hygiene items (see §4) | T |

### Supplement findings

| ID | Severity | Root-cause class | Component(s) | Remediation intent | Status |
| --- | --- | --- | --- | --- | --- |
| SUP-01 | Med-High | F: resource bounds | domainforge `load_validate` | Pre-parse nesting-depth cap before `parse_source` | R |
| SUP-02 | Med | D: identity | domainforge `parse_options_sha256` | Hash normalized entry spelling (strip `./`) | R |
| SUP-03 | High (chain) | A: path canonicalization | spec-pipeline `is_generated_zone` | Segment-aware zone match + lexical normalization | R |
| SUP-04 | Med | E: evidence integrity | spec-pipeline `build_projection_record`; self-model `verify_projection` | Record `Declared`/`projection_unvalidated`; verify output bytes | T |
| SUP-05 | Med | F: panic/overflow | capability `promotion.rs` + settlement `declaration.rs` `parse_fixed` | Char-boundary truncation; `checked_mul`; dedupe | R |
| SUP-06 | Med | B + E | server `agent_probe.rs` descriptor version | Derive version from config hash; settle failed probe | T |
| SUP-07 | Med | G: root convention | self-model/thoth/readiness double-nested reads | Unify root; absent-registry → degraded not zero | T |
| SUP-08 | Med | J: registry trust | extension `lib.rs` load/replace | Verify registry vs ledger; guard replace path | T |
| SUP-09a–i | Low | mixed | various | Sweep items (see §4) | T |

---

## 2. Root-cause grouping (systemic fixes)

The reports identify these shared mechanisms. Each gets ONE canonical primitive
and consumers are migrated to it, rather than N local patches.

1. **A — Canonical path validation/normalization.** One shared lexical
   normalizer (reuse `sea_forge_sandbox::safe_lexical_join` semantics) applied at
   each *enforcing* function: authority deny/allow, sandbox validate/safe_join,
   planner `valid_relative_path`, spec-pipeline `is_generated_zone`, and the id
   joins (F-09/F-14/F-15/F-17, SUP-09g). Retires F-01, F-09, F-14, F-15, F-17,
   SUP-03, SUP-09g, and the F-25.h/i TOCTOU notes.
2. **B — Error paths bypass completion invariants.** Dispatcher pre-validation +
   drain-and-settle helper; escalation parks instead of terminates; recovery
   verb for stranded `Active`. Retires F-02, F-03, F-11, SUP-06 (unsettled run).
3. **C — Crash-consistency + corruption→typed-error.** Entries-first commit;
   mmr as rebuildable derived state; bounds/char checks before indexing.
   Retires F-04, F-05, F-06, F-22, F-25.q/r.
4. **D — Identity/role propagation.** Thread verified `ActorRole`; map actor id→
   role in thoth; pin identities to normalized content. Retires F-08, F-23,
   SUP-02, F-25.e, SUP-09f.
5. **E — Evidence integrity ("validated" without validators).** No fabricated
   process results or validation stamps; borrow declared/probed/demonstrated
   vocabulary. Retires F-10, SUP-04, SUP-09h, F-20 (classification honesty).
6. **F — Resource bounds + async discipline.** Depth caps, wall-clock deadlines,
   clamped timeouts, `spawn_blocking`, bounded maps. Retires SUP-01, SUP-05,
   F-07, F-18, F-19/SUP-09a, F-24, SUP-09b/c.
7. **G — Root convention.** One state-root convention threaded through
   cell/self-model/thoth/bundle; fail-closed on absent evidence. Retires F-12,
   SUP-07.
8. **H — Governance metadata.** Exact capability matching. Retires F-13.
9. **I — Operability.** Status-map rebuild. Retires F-21.
10. **J — Registry trust.** Ledger-verified registry load. Retires SUP-08.

---

## 3. Ordering dependencies and batches

Ordering (from both reports' "Ordering dependencies"):
- A's normalization primitive first (serves F-14/F-15/F-17/F-09, SUP-03).
- F-04's "mmr as derived state" decision determines F-05.
- F-02/F-11 before F-03's approval re-drive (recovery is where re-driven case resumes).
- F-08 precedes role-keyed policy work.
- SUP-02's decision determines SUP-09f's release-id pinning.
- SUP-08's ledger verification precedes any `import`/`adopt` production caller.

### Batch 1 — Path canonicalization (class A) + generated-zone (SUP-03)
F-01, SUP-03, F-09, F-14, F-15, F-17, SUP-09g.
### Batch 2 — Ledger crash-consistency + corruption panics (class C)
F-04, F-05, F-06, F-22, F-25.q.
### Batch 3 — Dispatcher completion invariants (class B)
F-02, F-03, F-11, SUP-06 (unsettled run).
### Batch 4 — Resource bounds + panic/overflow (class F)
SUP-01, SUP-05, F-07, F-18, F-19/SUP-09a, SUP-09b, SUP-09c.
### Batch 5 — Identity/role propagation (class D)
F-08, F-23, SUP-02, F-25.e, SUP-09f.
### Batch 6 — Evidence integrity (class E)
F-10, SUP-04, SUP-09h, F-20.
### Batch 7 — Root convention (class G) + registry trust (class J)
F-12, SUP-07, SUP-08.
### Batch 8 — Governance metadata + operability (H, I)
F-13, F-21, F-16.
### Batch 9 — Hardening sweep (F-24, F-25.a–s, SUP-09d/e/i)

---

## 4. Decisions and specification questions

1. **SUP-02 (entry spelling identity).** Decision: normalize the entry spelling
   (strip leading `./` and collapse duplicate slashes) before hashing, so
   `model.sea` and `./model.sea` produce equal `semantic_model_sha256`. Aligns
   with systemic observation "identity from normalized content". (Alternative —
   reject non-canonical spellings — is defensible but breaks existing callers
   that pass `./entry.sea`; normalization is less disruptive.)
2. **F-12/SUP-07 (root convention).** Decision: adopt the *state-root*
   convention — cell/self-model/thoth/bundle/transcript_seal join directly under
   the passed root (drop the extra `.sea-forge` prefix), matching pipeline/
   ledger/case-runner. This is a persisted-layout change; it is recorded here as
   a deliberate decision. Export must fail closed when zero requested run files
   resolve.
3. **F-03 (escalation semantics).** Decision: an `Escalated` settlement parks
   the item/case as awaiting approval (not `ItemFailed`/`TerminateCase`), and
   the trace records `ItemTerminated`/a new `ItemAwaitingApproval`-equivalent
   rather than `item_failed`. Approval resolution re-drives the case.
4. **F-04 (mmr as derived state).** Decision: `entries.jsonl` is the single
   source of truth; `mmr.json` is rebuildable. `append` writes entries first,
   then mmr. `quarantine_incomplete_tail` rebuilds mmr from surviving entries.
   `verify()` keeps a read-only comparison but reports a repairable mismatch.
5. **SUP-04 (projection validation).** Decision: until a real validator exists,
   record `status: Declared` / `basis: ["projection_unvalidated"]` and make
   `verify_projection` hash materialized outputs against `output_refs`. (Do not
   stamp `Accepted`/`projection_validated` without performing validation.)
6. **SUP-09h (evaluate_authority).** Decision: reword the trace/basis to
   describe the stem-heuristic approximation rather than asserting full
   "DomainForge evaluated validated model against canonical action", until the
   real evaluator is wired.

---

## 5. Verification commands

Per batch: `cargo test -p <crate> <test>` for focused regression, then
`cargo test -p <crate>`, then the workspace suite.

Final acceptance:
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features --locked --no-fail-fast`
- `just context-check` (agent handoff gate)
- Re-run retained audit-harness reproductions (T1–T7, S1–S8, server_d1) and
  confirm they no longer demonstrate the defects.

---



## 6. Execution log

(Updated as batches complete. Each entry records: finding, invariant restored,
test added, verification run, result.)

### Batch 1 — Path canonicalization (class A) + generated-zone — COMPLETE
- Added `sea_forge_core::path` (`validate_relative_path`, `normalize_relative_path`,
  `path_prefix_matches`, `valid_id_segment`).
- F-01: authority `path_denied` normalizes before glob; built-ins anchored
  `**/.env*`, `**/.git/**`; `matches_rule` uses segment-aware `path_prefix_matches`.
- F-01/SUP-03: sandbox `validate_relative_path` delegates to core (strict);
  planner `valid_relative_path` strict; spec-pipeline `is_generated_zone`
  segment-aware + normalized.
- F-09: workbench `drafts.rs` validates draft_id `^[A-Za-z0-9_-]{1,64}$`.
- F-14: `LedgerStream::open` validates ledger_id; CLI resume/case/ledger validate
  case_id; cell `adopt` validates cell_id.
- F-15: `templates::parse_template_ref` made pub and reused in server
  `case.commit`/`case.preflight`; cell `parse_template_ref` validates charset.
- F-17: `settle()` validates `plan_item_id` before quarantine write.
- SUP-09g: self-model `current_snapshot` validates manifest snapshot id.
- Tests added: core `path` unit tests; authority
  `non_canonical_spellings_cannot_bypass_hard_boundaries`; spec-pipeline
  `generated_zone_spelling_aliases_are_denied`; sandbox
  `ambiguous_spellings_are_rejected`; ledger `open_rejects_traversal_ledger_id`;
  settlement `traversal_plan_item_id_cannot_escape_quarantine_dir`; workbench
  `draft_id_traversal_is_refused`.
- Verified: `cargo test -p` for core/sandbox/planner/authority/spec-pipeline/
  ledger/settlement/self-model/cell — all green. `cargo fmt --all -- --check`
  clean. Workbench `draft_id_traversal_is_refused` NOT run (Tauri host needs
  glib-2.0; devbox unavailable) — code verified by inspection; recorded as
  environmental limitation.

### Batch 2 — Ledger crash-consistency + corruption panics — COMPLETE
- F-04: `append_under_lock` writes `entries.jsonl` before `mmr.json`;
  `quarantine_incomplete_tail` rebuilds the derived MMR from surviving entries.
- F-05: `prove_entry` validates `entries.len() == mmr.leaf_count` before slicing.
- F-06: `base64_decode` rejects code points >= 256 before table indexing.
- F-22 (partial): signing key created 0600 from the start; `load_verifying_key`
  no longer mints keys; removed `.unwrap()` TOCTOU panic.
- Tests added: `m0_crash_recovery_rebuilds_mmr_ahead_of_entries`,
  `m0_prove_entry_under_desync_is_typed_error_not_panic`,
  `signature_verification_rejects_wide_chars_without_panic`.
- Verified: `cargo test -p sea-forge-ledger` green (14+22+3).

### Batch 3 — Dispatcher completion invariants (class B) — COMPLETE
(Work landed in the working tree during the resumed session; verified green.)
- F-02: `case_dispatch::submit` pre-validates the whole ready batch's
  dispatchability *before* any spawn, so a mix of executable and non-executable
  items (e.g. `[SandboxedTask, Stage]`) rejects with a typed
  `non_executable item kind` error while `active` is empty — no activated
  episode is aborted mid-dispatch and left without a terminal settlement.
- F-03: an escalated *required* item parks the case as
  `CaseState::AwaitingApproval` (close_reason `awaiting_approval:<item>`) and
  returns a `DispatchOutcome { state: "awaiting_approval", exit_code: 5 }`
  instead of folding into `ItemFailed`/`TerminateCase`. The escalation opens
  exactly one `approval_request` that stays `pending`, and `case-runner` traces
  `TraceKind::ItemTerminated` (not `item_failed`) for the escalation. Approval
  resolution re-drives the case (CLI `resume` flow).
- Regression tests: `conformance_dispatch_remediation.rs` —
  `f02_mixed_batch_rejects_before_any_episode_spawns` (asserts zero
  `item_activated` events before rejection),
  `f03_escalated_required_item_parks_case_not_terminates` (asserts no
  `item_failed`/`case_terminated`, one pending approval, `awaiting_approval`
  state), `f19_huge_timeout_is_clamped_not_panic` (F-19).
- Also landed in the same dispatch surface:
  - F-19/SUP-09a: `MAX_TIMEOUT_SECS` clamp on `SubmitPayload.timeout` at the
    protocol boundary (pathological u64 can no longer flip negative via
    `as i64`).
  - F-07: `fire_notify` moved to `spawn_blocking`, stdout/stderr to `null`
    (a chatty hook can no longer deadlock the parent on a full pipe), and
    bounded with `wait_timeout` (hung hook is SIGKILLed after 30s). New
    `notify_tests` module covers both.
- Verified: `cargo test -p sea-forge-server --test conformance_dispatch_remediation`
  3/3 green. Full workspace `cargo test --workspace --all-features --locked
  --no-fail-fast` green (107 test-result-ok, 0 failures) with the entire
  working tree applied.
- Environmental note: `workbench ... draft_id_traversal_is_refused` (F-09)
  still not runnable here (Tauri host needs glib-2.0/devbox); code verified by
  inspection, matching plan Batch 1's recorded limitation.

### Additional cross-batch items landed in the working tree (later batches)
These belong to later batches but were implemented together; they are recorded
here once and cross-referenced from their batch when it is finalized.
- SUP-01 (Batch 4): domainforge `MAX_NESTING_DEPTH = 256` lexical pre-parse
  guard (`nesting_depth_exceeded`) bounds bracket depth, consecutive unary `-`,
  and `not` chains before the recursive-descent parser — a crafted deep model
  yields a typed `domain_model_error` instead of a stack-overflow SIGABRT.
  Tests: `deep_nesting_is_rejected_before_parse_not_abort`,
  `modest_nesting_still_parses`.
- SUP-05 (Batch 4): char-boundary-safe truncated parse + saturating/checked
  arithmetic in both `sea_forge_capability::promotion::parse_fixed` and
  `sea_forge_settlement::declaration::parse_fixed`. Tests:
  `parse_fixed_is_char_boundary_and_overflow_safe`.
- F-10 (Batch 6): CLI `resume` no longer fabricates a synthetic
  `Completed`/`exit 0` `ExecutionResult` for write-only items. New
  `SettlementClaim.write_only` flag flows from the CLI to `settle`, which
  records an honest `write_only` basis and accepts only when all
  `required_artifacts` materialize (`None => Rejected` otherwise). Tests:
  `write_only_item_settles_without_fabricated_process_result`.

### Batch 5 — Identity/role propagation (class D) — PARTIAL (SUP-02 landed)
(Work landed during the resumed session; remaining class-D findings still TODO.)
- SUP-02: domainforge hashes the *normalized* entry spelling
  (`normalize_entry_uri`: strip a single leading `./`, collapse duplicate
  `/` segments), so `model.sea` and `./model.sea` resolve to the same
  `semantic_model_sha256` — identity is derived from resolved content, never a
  caller's raw spelling. Test:
  `equivalent_entry_spellings_produce_the_same_semantic_identity`.
- Verified: `cargo test -p sea-forge-domainforge --test conformance_m0_domainforge`
  21/21 green.
- Remaining class-D: F-08 (ActorRole propagation into all `Actor` construction
  sites), F-23 (thoth actor id→role), F-25.e, SUP-09f (release-id pinning).

### Batch 8 — Governance metadata + operability (H, I) — PARTIAL (F-21 landed)
(Work landed during the resumed session; F-13 and F-16 remain TODO.)
- F-21: `ServerState::new` now rebuilds the in-memory `status` `cases` map from
  disk (`sfwp::case_views::list`) at startup, so a restart no longer answers
  `"case not found"` on `status` for pre-restart cases. `exit_code`/`run_dir`
  start `None` (the map is a liveness cache; `case.list` remains disk
  authority). Test: `status_resolves_a_pre_restart_case_from_disk`.
- Remaining class H/I: F-13 (exact capability↔declaration mapping, reject `*`),
  F-16 (constrain server policy/plan path resolution to workspace-relative
  under cell root).

### Batch 4 — Resource bounds + panic/overflow (class F) — PARTIAL (F-18 landed)
- F-18: ACP prompt pumps now have a wall-clock deadline derived from the
  `max_turns × per_turn_timeout` budget and capped at 15 minutes. The deadline
  wins over a receive timeout exactly at budget exhaustion, so a peer streaming
  benign notifications cannot be misclassified as a disconnected peer.
  Tool-call updates now consume turns even when title-less, and retained
  tool-call descriptions cap at 256 while overflow updates continue to consume
  budget. No public termination vocabulary changed: both resource bounds settle
  as the existing `turn_cap_exceeded` outcome. Regressions cover notification
  streaming, title-less tool calls, and retained-map capping.
- Remaining Batch 4: SUP-09b (CLI recursive migration must reject symlink
  traversal/cycles) and SUP-09c (cap untrusted whole-file reads).
