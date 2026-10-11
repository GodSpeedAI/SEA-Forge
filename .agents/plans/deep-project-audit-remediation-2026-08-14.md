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
| F-08 | Med | D: role propagation | server identity gate → `Actor` construction sites | Propagate verified `ActorRole` into all `Actor` construction | R |
| F-09 | Med | A: path canonicalization | workbench `drafts.rs` `draft_path` | Validate `draft_id` grammar `^[A-Za-z0-9_-]{1,64}$` | R |
| F-10 | Med | E: evidence integrity | CLI `resume.rs` synthetic `Completed` | No fabricated `ExecutionResult` for write-only items | R |
| F-11 | Med | B: recovery | CLI `resume`/`case reopen` | Recovery verb for stranded `Active` cases | R |
| F-12 | Med | | G: root convention | cell/bundle/self-model/thoth/transcript_seal | Unify to state-root convention; fail-closed export | R |
| F-13 | Med | H: governance metadata | capability `promotion.rs` substring match | Exact `attempted_capability`/plan-item mapping; reject `*` | R |
| F-14 | Med-Low | A: path canonicalization | ledger `LedgerStream::open`; CLI resume/case/adopt ids | Validate id grammar at crate boundary before fs mutation | R |
| F-15 | Med-Low | A: path canonical | server `case.preflight`/`commit` `template_ref` | Validate name/version charset before join | R |
| F-16 | Med-Low | A: trust boundary | server policy/plan path resolution | Constrain to workspace-relative under cell root | R |
| F-17 | Low-Med | A: path canonicalization | settlement `settle()` quarantine `plan_item_id` | Validate `plan_item_id` grammar before write | R |
| F-18 | Low-Med | F: resource bounds | agent `acp.rs` episode | Wall-clock deadline; bound `tool_calls`; count tool-call turns | R |
| F-19 | Low-Med | F: resource bounds | server `SubmitPayload.timeout` | Clamp timeout at protocol boundary | R |
| F-20 | Low | E: evidence classification | sandbox `jail.rs` stderr heuristic | Annotate `suspected` or use exit-code/EACCES | R |
| F-21 | Low | I: operability | server `status` in-memory map | Rebuild map at startup / answer from disk | R |
| F-22 | Low | C: key hygiene | ledger `signing.rs` | 0600-at-create; split verify/sign; symlink-check; no self-witness | P (0600 + no-mint landed Batch 2; rest tracked in debt) |
| F-23 | Low | D: role propagation | thoth `service.rs` actor id→role | Map verified role before matching grants | R |
| F-24 | Low | F: async/blocking | server/ledger blocking + O(n) scans | `spawn_blocking`; incremental tail; caps | P (MUST items landed Batch 9; tail-cache/ledger-internals deferred with triggers) |
| F-25.a–s | Low | mixed hardening | various | Batched hygiene items (see §4) | P (e landed Batch 5; a–d/f–h/j–r landed or dispositioned in Batch 9; g narrowed per owner; n → OBSERVED_DEBT) |

### Supplement findings

| ID | Severity | Root-cause class | Component(s) | Remediation intent | Status |
| --- | --- | --- | --- | --- | --- |
| SUP-01 | Med-High | F: resource bounds | domainforge `load_validate` | Pre-parse nesting-depth cap before `parse_source` | R |
| SUP-02 | Med | D: identity | domainforge `parse_options_sha256` | Hash normalized entry spelling (strip `./`) | R |
| SUP-03 | High (chain) | A: path canonicalization | spec-pipeline `is_generated_zone` | Segment-aware zone match + lexical normalization | R |
| SUP-04 | Med | E: evidence integrity | spec-pipeline `build_projection_record`; self-model `verify_projection` | Record `Declared`/`projection_unvalidated`; verify output bytes | R |
| SUP-05 | Med | F: panic/overflow | capability `promotion.rs` + settlement `declaration.rs` `parse_fixed` | Char-boundary truncation; `checked_mul`; dedupe | R |
| SUP-06 | Med | B + E | server `agent_probe.rs` descriptor version | Derive version from config hash; settle failed probe | R |
| SUP-07 | Med | G: root convention | self-model/thoth/readiness double-nested reads | Unify root; absent-registry → degraded not zero | R |
| SUP-08 | Med | J: registry trust | extension `lib.rs` load/replace | Verify registry vs ledger; guard replace path | R |
| SUP-09a–i | Low | mixed | various | Sweep items (see §4) | P (a/b/c/f/g/h landed earlier; d/e/i landed Batch 9; causation_id + dead types → OBSERVED_DEBT) |

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

### Batch 4 — Resource bounds + panic/overflow (class F) — COMPLETE
- F-18: ACP prompt pumps now have a wall-clock deadline derived from the
  `max_turns × per_turn_timeout` budget and capped at 15 minutes. The deadline
  wins over a receive timeout exactly at budget exhaustion, so a peer streaming
  benign notifications cannot be misclassified as a disconnected peer.
  Tool-call updates now consume turns even when title-less, and retained
  tool-call descriptions cap at 256 while overflow updates continue to consume
  budget. No public termination vocabulary changed: both resource bounds settle
  as the existing `turn_cap_exceeded` outcome. Regressions cover notification
  streaming, title-less tool calls, and retained-map capping.
- SUP-09b: `sea-forge migrate` traversal is symlink-safe and bounded. Both
  recursive walkers (`enumerate_files_recursive`,
  `remove_empty_dirs_recursive`) type every directory entry with
  `DirEntry::file_type()`/`fs::symlink_metadata` (lstat semantics — never
  follow), reject any symlink in the enumeration tree with a typed
  `ForgeError::Input` (`refusing to migrate through symlink: …`), and enforce
  `MAX_MIGRATION_DEPTH = 32` (deeper trees are rejected, not recursed).
  `execute()` now traverses and validates the legacy tree *before*
  `resolve_key_config`/`load_or_create_signing_key`, so a rejection happens
  with zero filesystem side effects (no key material, no `ledgers/`, no
  `migration.json`). En route finding: the pre-fix cycle fixture did not
  stack-overflow but silently collected ~800 PATH_MAX-bounded junk entries
  (`runs/loop/loop/…/plan.json`) into the migration Vec — both that silent
  junk-ingest and the deeper-layout overflow are closed by the same rejection.
  Tests: `crates/sea-forge-cli/tests/migrate_safety.rs` — symlink-directory
  cycle, symlinked-file, and depth-cap rejections, each asserting nonzero
  exit, the typed message on stderr, and zero side effects including the
  default key dir. The full migrate happy path
  (`conformance_m0_migrate`, 2 tests) still passes unchanged.
- SUP-09c: every audited untrusted whole-file read is bounded.
  - Server views: `sfwp/mod.rs` defines `MAX_RECORD_BYTES = 4 MiB` (single
    JSON records) and `MAX_JOURNAL_BYTES = 64 MiB` (append-only JSONL
    journals) with one shared `size_within_cap` enforcement predicate.
    `case_views::read_case` stats first and reports oversized records as
    `Unreadable` (present-but-unreadable is the integrity signal) while stat
    failures keep the absent/read-failure mapping; `read_plan`,
    `read_case_events`, `read_settlement` (a site the audit's line list
    missed), `run_views::read_json` (shared `pub(crate)` helper, so
    `sfwp::assets`/`sfwp::delegations` inherit the cap), and `read_jsonl`
    degrade oversized inputs exactly as unreadable ones already did. Tests:
    5 new conformance tests (`conformance_case_views` 3, `conformance_run_views`
    2) using valid-JSON(L) fixtures padded past each cap so the *only* changed
    property is size (a cap on malformed bytes would prove nothing) —
    differential red runs confirmed the padded records rendered normally
    pre-fix.
  - `sea-forge-trace::append_internal_error` streams the journal
    (`BufReader::read_until`) instead of slurping it — memory bounded by one
    line on the internal-error path — while preserving every observable
    semantic: torn tail (no trailing newline) refuses, blank lines are skipped
    not counted, the first malformed line refuses, an empty journal refuses
    (a naive streaming rewrite would have silently begun appending; pinned by
    test), and the appended event keeps `seq_id("tev", 4, n+1)`. 7 new
    in-crate tests (10/10 total).
  - `sea-forge-sandbox::jail` permission-denied heuristic reads at most
    `MAX_STDERR_SCAN_BYTES = 65_537` bytes (mirroring settlement's
    `declaration.rs` cap idiom) with lossy UTF-8 decoding — strictly better
    than the old `read_to_string().unwrap_or_default()`, which discarded *all*
    stderr when any byte was non-UTF-8 and blinded the heuristic. 3 new
    real-jailed-spawn conformance tests: within-cap marker still classifies
    `SandboxViolation`; marker past the cap does not (fixture asserts the
    on-disk offset really is ≥ the cap); non-UTF-8 prefix no longer blinds.
  - `sea-forge-cli` `internal-test-swe-seed` caps stdin at 1 MiB
    (`take(1_048_577)`, typed `Input` error past the cap), mirroring the
    settlement transport idiom. Tests: `tests/swe_seed_cli.rs` — oversize
    rejects with the cap message; a small valid `SettlementDeclarationRequest`
    still succeeds.
  - Out-of-scope follow-up filed in `OBSERVED_DEBT.md`: further unbounded
    whole-file reads exist elsewhere in `sea-forge-server`
    (`correlation.rs`, `delegation.rs`, `transcript_seal.rs`, `lib.rs`,
    `case_dispatch.rs`, `sfwp/assets.rs`, `sfwp/case.rs`) — deferred to the
    Batch 9 hardening sweep, not part of the audited site list.
- Verified: `cargo fmt --all -- --check` clean; `cargo clippy -p sea-forge-cli
  -p sea-forge-server -p sea-forge-trace -p sea-forge-sandbox --all-targets --
  -D warnings` clean; per-crate suites green (`sea-forge-cli` migrate_safety 3,
  swe_seed_cli 2, conformance_m0_migrate 2; `sea-forge-server`
  conformance_case_views 9, conformance_run_views 11; `sea-forge-trace` 10;
  `sea-forge-sandbox` full crate incl. conformance_m1 15/15); full workspace
  `cargo test --workspace --all-features --locked --no-fail-fast` green.

### Batch 5 — Identity/role propagation (class D) — COMPLETE
(SUP-02 landed earlier; this entry closes the batch.)
- F-08: the identity gate now carries the whole `ResolvedActor` — id *and*
  verified role — from `dispatch_bounded` through `handle_request_as` into
  every authority evaluation. `handle_request_as` derives one `verified_role`
  (the gate's verified `ActorRole`; in-process callers without a socket
  identity keep the historical local-operator shape, spelled once at that
  single fallback site) and threads it explicitly into all five construction
  sites: `record_cancellation`, `execute_sandbox` (incl. its literal
  `resolve_identity(entity, ActorRole::Operator)`), `delegation::DelegationRequest`
  → `execute_with_permission_broker` + `AcpAuthorityMediator` (new required
  `actor_role` field; manual `Default` documents the operator fallback for
  test fixtures), and `agent_probe::ProbeRequest`. Role-keyed policy rules now
  distinguish principals server-side; renderer-authored roles remain
  fail-closed at the gate.
  Test: `conformance_role_propagation.rs`
  `f08_verified_role_reaches_the_authority_evaluation` — one policy whose only
  allow rule targets `R-SO`: a `ResolvedActor` resolved through real
  `IdentityBindings` (uid-bound SecurityOfficer) gets verdict `allow`; the same
  submit with no verified identity evaluates as the operator fallback and is
  denied.
- F-23: thoth `SurfacePolicy` no longer stores the actor id as
  `actor_role`. New `grant_keys_for` builds the asker's grant keys from the
  raw actor id plus the serde wire spelling of every role the authority
  bundle binds to that principal (`operator`, `R-SO`, …), so role-authored
  grants match bound actors while id-keyed grants stay working;
  `requires_fresh` uses the same key set. Tests:
  `t23_role_authored_grant_permits_a_bound_actor` (bound asker ⇒ `Partial`
  with the granted class disclosed, out-of-grant classes withheld),
  `t23_unbound_actor_cannot_use_another_principals_role_grant` (⇒ `Denied`).
- F-25.e: two bundle-hash producers disagreed —
  `refresh_policy_bundle_hash` excluded `source_base` while
  `PolicyAuthorityEngine::new` hashed the bundle *including* it, so the same
  policy loaded from different absolute paths evaluated under different
  recorded hashes. One private `content_hash` primitive (canonical hash over
  governed fields, excluding the self-describing `policy_bundle_hash` field
  and location-bound `source_base`) is now the single source used by
  `refresh_policy_bundle_hash`, schema validation's mismatch check, and
  `PolicyAuthorityEngine::new`. Tests:
  `policy_bundle_hash_is_independent_of_source_base_path` (two cells, two
  paths ⇒ identical bundle stamps and engine hashes).
- SUP-09f: self-model rebuild keyed its release-realization idempotency on
  the constant `RELEASE_ID` while the payload varies with SOURCE_DATE_EPOCH,
  so any rebuild from a different build environment (or edited realization)
  hit the ledger payload conflict and every later rebuild failed. The
  idempotency key is now the realization's canonical content hash
  (`realization_sha256`) — identical content dedupes to the existing record,
  differing content commits cleanly. Tests:
  `conformance_rebuild_identity.rs` — a different build epoch rebuilds
  cleanly (verified to reproduce the exact `idempotency key payload
  conflict` against the pre-fix code) and identical rebuilds dedupe to
  exactly one release record.
- Verified: `cargo fmt --all -- --check` clean; `cargo clippy -p
  sea-forge-authority -p sea-forge-thoth -p sea-forge-self-model -p
  sea-forge-server --all-targets --all-features --locked -- -D warnings`
  clean; per-crate suites green (authority 42+17, thoth 39+9, self-model incl.
  new suite, server full incl. m16 16 after building the CLI binary); full
  workspace `cargo test --workspace --all-features --locked --no-fail-fast`:
  111 suites, 912 passed, 4 ignored (documented real-host release gates). One
  failure in the parallel workspace run — `sea-forge-agent`
  `acp::tests::notifications_cannot_outlive_episode_wall_clock_budget` — does
  not track this diff (crate untouched): it passes in isolation and with its
  module, same host-load class as the previously documented `kill_9` flake in
  OBSERVED_DEBT.md.

### Batch 6 — Evidence integrity (class E) — COMPLETE
(F-10 landed cross-batch earlier; this entry closes the batch.)
- SUP-04 (plan §4.5): projection records no longer assert validation that
  never ran. Core gained `ProjectionStatus::Declared`; spec-pipeline's
  `build_projection_record` (also the CLI `project` success path via the same
  function) now stamps `Declared` / basis `["projection_unvalidated"]` /
  `validator_ref: "none"` instead of the unconditional
  `Accepted`/`projection_validated` over the descriptor sentinel.
  `verify_projection` now takes the projections directory and hashes every
  *materialized* output file against its recorded `output_refs` ref after the
  rebuild-hash check — a replaced or corrupted view file is a
  `self_model_error`, closing the doc-promise gap (`store::validate` passes
  the real dir). Tests: conformance_m5 asserts the Declared/unvalidated/
  none triple; conformance_m9 t93 materializes all three projections,
  verifies them, and proves a replaced output file fails with "does not match
  its recorded ref".
- SUP-09h (plan §4.6): domainforge `evaluate_authority`'s trace reason now
  describes what actually runs — a filename-stem approximation ("write_file
  allowed iff the resource_id file stem ASCII-case-insensitively matches a
  declared entity/resource name; full policy evaluation not yet wired") — so
  durable authority evidence no longer claims "evaluated validated model
  against canonical action" for the unwired evaluator. No decision vocabulary
  changed.
- F-20: the jail's child-controlled stderr heuristic can no longer record a
  definite violation. Core gained `ExecutionStatus::SuspectedSandboxViolation`
  (wire `suspected_sandbox_violation`); jail classification of nonzero-exit +
  "permission denied" stderr lands there, settlement records basis
  `suspected_jail_violation` (still Rejected), CLI label
  `suspected_sandbox_violation`. The definite `SandboxViolation` remains
  reserved for observed violations. SUP-09c's bounded-scan tests re-anchored
  to the suspected variant plus a new distinction test
  (`jail_stderr_heuristic_never_asserts_a_definite_violation`).
- En route: fixed a pre-existing clippy `bool_comparison` in core `path.rs`
  surfaced by widening clippy scope to core.
- Verified: `cargo fmt --all -- --check` clean; `cargo clippy -p sea-forge-core
  -p sea-forge-sandbox -p sea-forge-settlement -p sea-forge-cli -p
  sea-forge-domainforge -p sea-forge-spec-pipeline -p sea-forge-self-model
  --all-targets --all-features --locked -- -D warnings` clean; full workspace
  `cargo test --workspace --all-features --locked --no-fail-fast`: 111 suites,
  914 passed, 0 failed, 4 ignored (documented real-host release gates).

### Batch 7 — Root convention (class G) + registry trust (class J) — COMPLETE
Per the recorded §4.2 decision (state-root layout; persisted-layout change).
- F-12: the `.sea-forge` prefix is gone everywhere it was prepended —
  `cell.rs` (`<root>/cell.json`), `bundle.rs` export/import
  (`<root>/runs`, `<root>/templates`, `<root>/imported/...`),
  `self-model/store.rs` (`<root>/self-model/...`, ledger at `<root>/ledgers`),
  `thoth/service.rs` (`<root>/capabilities.jsonl`,
  `<root>/settlement/declarations.jsonl`), server `transcript_seal.rs`
  (`<root>/sealed`), `cell/template.rs` adopt (`<root>/templates`), and the
  capability promotion readers/writers (`<root>/capabilities.jsonl`,
  `<root>/settlement/declarations.jsonl`, `<root>/capabilities/policies`) —
  the last group being a SUP-07 coherent-wrong-reader instance the audit's
  crate list missed. The CLI's default root directory is still named
  `.sea-forge`; it is now simply the state root every crate joins directly.
  Export fails closed: requested runs that resolve to zero evidence files
  refuse the bundle with a typed error instead of exporting silently empty
  (templates-only exports stay lawful). Tests:
  `export_with_unresolvable_runs_fails_closed_not_silently_empty`; the m6
  escape test re-anchored as `legacy_sea_forge_symlink_is_inert_to_import`
  (a hostile legacy `.sea-forge` symlink neither breaks an import nor gets
  written through); all double-nested fixtures across cell/thoth/self-model/
  cli/server tests moved to state-root paths.
- SUP-07: an *absent* extension registry no longer fabricates a clean
  "zero extensions" cell — `ExtensionRegistry::exists()` lets the CLI rebuild
  disclose it by marking the snapshot stale with reason
  `extension_registry_absent` (degraded, never silently zero); the readiness
  projection reads the unified state-root ledger path.
- SUP-08: new `ExtensionRegistry::load_verified(root, stream)` proves the
  registry bytes against the newest `extension_registry` ledger record
  (payload-hash match); forged or rolled-back files and unattested registries
  are refused. `agent_probe`'s registration read goes through it. The
  immutable-replace path can no longer resurrect a quarantined entry as
  FirstParty+Active (versioned replace refused; idempotent re-registration
  leaves quarantine intact), and duplicate `(id, version)` entries are
  rejected at load. Tests:
  `load_verified_refuses_registry_bytes_the_ledger_never_committed`,
  `quarantined_runtime_adapter_cannot_be_replaced_by_registration`.
- Verified: `cargo fmt --all -- --check` clean; clippy `-D warnings` clean on
  cell/self-model/thoth/extension/server/cli/capability; full workspace
  `cargo test --workspace --all-features --locked --no-fail-fast`: 111 suites,
  917 passed, 0 failed, 4 ignored (documented real-host release gates).

### Batch 8 remainder — governance metadata + trust boundary — COMPLETE
- F-13: capability promotion matches declarations by *exact* `plan_item_id`
  equality (the identity rule envelopes already used for
  `attempted_capability`); `"*"` is rejected outright with a typed
  `ForgeError::Input` (`build_capability_record` now returns `Result`;
  thoth's disclosure path maps the error to fail-closed `None`). The
  substring/wildcard aggregation could inflate provenance and flip a
  capability to `Proven` on unrelated evidence, gating `require_proven`
  side-effect authority. Tests: conformance_m4a `f13_substring_plan_items_*`,
  `f13_wildcard_capability_identity_is_rejected`,
  `f13_exact_plan_item_mapping_still_aggregates`; thoth m11 fixture re-anchored
  to exact mapping.
- F-16: request/config-supplied policy and plan references resolve strictly
  workspace-relative under the cell state root
  (`agent_probe::resolve_policy_path` → lexical validation via
  `sea_forge_core::path::validate_relative_path`; absolute/traversal spellings
  are a typed `UnsafePath` before any read). Wired through submit (plan),
  execute_sandbox, delegation (incl. SWE_SEED declaration), agent_probe,
  record_cancellation; `case.commit`'s draft is returned as its cell-relative
  spelling; the CLI delegate client now sends cell-relative spellings
  (`cell_relative_reference`, typed error for outside-the-cell references).
  Test updates across nine server suites + CLI m13 convert fixtures to the new
  boundary; every converted test now also proves the constraint.

### Batch 9 — hardening sweep — COMPLETE (with recorded dispositions)
- F-11 (recovery): CLI `resume` accepts stranded `Active` cases. A recovery
  pre-pass terminal-settles activated-but-unsettled episodes as
  rejected/interrupted (`ItemFailed` with basis `interrupted`) while leaving
  genuinely parked human tasks untouched, then re-drives the loop from events:
  required item → lawful termination, repeating item → re-drive, otherwise →
  completion. The approval-expiry termination no longer fires for recovered
  Active cases. Tests: `tests/resume_recovery.rs` (5) incl. parked-human-task
  preservation and gate text.
- SUP-06 (probe descriptor): adapter version is *derived* from config identity
  (`cfg-<12 hex of descriptor_config_sha256>`) instead of hardcoded `0.1.0`,
  so a config edit takes the registry's replace-in-place path instead of
  permanently failing immutability; the fabricated `sha256:b…b` output
  contract digest was replaced with a hash over the declared empty-completion
  schema; registration failures after authority commits now settle the run
  Rejected (`agent_endpoint_registration_failed`) via the existing
  `finish_rejected` evidence+settlement path, never leaving an
  allowed-but-unsettled run, with the provider never contacted.
  En-route defect found by the new tests: registry attestation lived in each
  probe's ephemeral case ledger, so any later probe failed `load_verified`
  against bytes a different ledger committed — attestation moved to a
  dedicated cell-scoped `extension-registry` ledger. Tests: m12
  `t12_7_*` (4): derived version + grammar safety, identical-config
  idempotence, config-edit re-probe succeeds and marks self-model stale,
  registration failure settles fail-closed with quarantine intact.
- F-24 (availability): MUST items landed per the survey —
  `MAX_PRECONDITION_RECORDS = 16` cap enforced pre-resolution (typed Input);
  `LedgerRecordResolver` opens/reads the case ledger once per bundle, not per
  record; delegation's jail spawn replaced thread+blocking-recv with
  `spawn_blocking`; SWE_SEED harvest (git subprocess + tree walk) and the ACP
  continuation ledger scan moved off tokio workers; `record_cancellation`
  wrapped; events ledger switched from `Mutex<LedgerStream>` held across
  blocking appends to `Arc<LedgerStream>` with publish/get_range/replay on
  blocking threads (the ledger's own flock remains the serializer);
  case_dispatch mints the case (read/validate/init/first writes) in one
  blocking unit and settlement recording runs through
  `record_completion_blocking`. Deferred with triggers (OBSERVED_DEBT-style):
  events tail-cache/paged storage, broad commit_view wraps, ledger-internal
  incremental tip. Test: `oversized_precondition_records_fail_with_typed_error_
  before_any_side_effect`; ordering pinned by existing sfwp e2e + ledger
  multi-writer tests.
- F-25.f: pinning test `misspelled_top_level_bundle_key_is_rejected`
  (attribute already had `deny_unknown_fields`).
- F-25.g (owner decision: narrow): the v0.1 legacy implicit allow for
  `recall_memory` now authorizes only the acting entity's own memory;
  cross-entity recall requires an explicit `memory_scope` rule (typed error
  naming it). Self-recall stays byte-compatible. Test: m4b
  `f25g_legacy_silent_policy_refuses_cross_entity_recall`.
- F-25.h: grants carry the decision-time canonical executable identity
  (`ActionGrant::resolved_executable`, non-serialized); `sea-forge-runtime`
  re-canonicalizes argv[0] immediately before spawn and refuses mismatch or
  missing binding — shrinking the decision→spawn swap window to a same-instant
  race. Residual openat2-class closure documented. Test: authority
  `f25h_grant_binds_the_decision_time_executable_identity`.
- F-25.i: shared `safe_write` (unix `O_NOFOLLOW`, ELOOP → typed UnsafePath)
  routed through plan materialization, environment base materialization, and
  artifact capture. Test: sandbox
  `destination_swapped_to_symlink_between_check_and_write_is_refused`
  (models the true check→swap→write window).
- F-25.j: mutation-class reserved actions (`delete_file`,
  `generated_zone_mutation`, `spec_mutation`, `settlement_authority_mutation`,
  `policy_mutation`, `evidence_mutation`) hitting the built-in generated-zone/
  .git/.env/secret boundaries deny unconditionally before any rule — the wall
  exists before their first executor lands. Test: authority
  `reserved_mutator_into_generated_zone_denies_even_under_allow_all_policy`.
- F-25.k: both sandbox backends' `collect_artifacts` stubs fail closed
  (`artifact_collection_unsupported`) instead of returning silent empty
  success. Test: sandbox `collect_artifacts_refuses_instead_of_returning_empty`.
- F-25.l: one owner for the approvals journal write — core `approvals::append`
  emits record+newline in a single O_APPEND write; the server's drifted copy
  delegates to it. No concurrent reader can observe a torn line.
- F-25.m (owner decision: 256 + reject empty ops): `MAX_PLAN_ITEMS = 256`
  enforced at `validate_proposal`; zero-operation SandboxedTasks rejected as
  `plan_schema_error`; `has_cycle` rewritten as iterative DFS (100k-node chain
  validates without overflow — abort-class DoS closed). Existing planner
  fixture corpus updated to carry real operations. Tests:
  `deep_linear_dependency_chain_validates_without_stack_overflow`,
  `sandboxed_task_without_operations_is_rejected`.
- F-25.q: append-time predecessor verification — the ledger recomputes the
  stored tail's content hash and refuses to chain on mismatch
  (`ledger_integrity_error`), so forged/rewritten tails surface at the next
  append rather than only at verify(). Consistently-rewritten tails remain
  verify()/checkpoint territory (documented). Test:
  `appending_onto_a_forged_tail_is_refused` (both tamper variants + clean
  control).
- F-25.r: evidence/trace/capability writers `sync_data` after flush
  (power-loss-safe appends matching ledger discipline; the best-effort
  internal-error path intentionally stays unsynced); recall scans are
  byte-oriented (`read_until`) so one non-UTF-8 byte degrades to the malformed
  counter instead of aborting the scan — approvals.jsonl keeps its strict
  contract deliberately. Test: capability
  `non_utf8_line_degrades_to_skipped_not_scan_failure`.
- F-25.n: filed in OBSERVED_DEBT (wire-contract change required for a real
  fix; trigger = first external multi-client SFWP consumer).
- SUP-09d: one canonical primitive (`sea_forge_core::canonical`) behind all
  four former copies (ledger/evidence/self-model/authority), explicit sort +
  NFC values, honest docs ("not RFC 8785 JCS"; keys not NFC'd; label kept —
  it is hashed into committed records). Golden-vector tests pin the exact
  bytes. En-route discovery: the four copies were NOT equivalent — two
  normalized NFC only at top level. The shared primitive recurses at all
  depths; kernel-committed data is ASCII today, so historical hashes are
  stable, and the F-25.q check would surface any hypothetical old divergent
  entry loudly. Full JCS compliance → OBSERVED_DEBT (owner spec decision).
- SUP-09e: `ComposedModel::dual_declared_concepts()` discloses the overlay's
  dual-declared names (30, allowlist-pinned against the bundled models) so
  definition divergence cannot silently collapse. Composition precedence
  itself → OBSERVED_DEBT (owner decision).
- SUP-09i: CEP-0008 projection validates identity grammar (26-char ULID,
  sha256:<64 lowercase hex>, safe id segment); `validate_descriptor` enforces
  extension_id/version grammar (shared id-segment predicate + version
  predicate allowing dots); `register_built_in` treats same-(id,version)
  different-hash as a loud build bug instead of silent idempotent success;
  `import` refuses duplicate/terminal-standing targets (closing the
  registry-brick vector the load-time duplicate guard created). causation_id
  deserialization asymmetry and dead `Compatibility`/`ExtensionInstallRecord`
  types → OBSERVED_DEBT. Tests: cep0008 malformed-input trio; extension lib
  grammar/registration/import tests.
- Workbench (separate workspace): F-25.a `frozenLockfile = true`; F-25.b/c
  sidecar env allowlist (env_clear + HOME/PATH/RUST_LOG/TMPDIR) and removal of
  the PATH fallback (two-rung resolution: SEA_FORGE_SERVER_BIN override →
  sibling sidecar; absent ⇒ fail-closed `server_binary_not_found`);
  F-25.d dev/storybook down recipes kill only PID-file-recorded sessions
  (warn-and-exit when absent/stale; fuser port-killing removed).
  Verified once the operator installed the Tauri system prerequisites:
  src-tauri `cargo fmt --check` clean (scoped fmt applied to pre-existing
  bridge.rs/drafts.rs drift), `cargo clippy --all-targets -- -D warnings`
  clean, `cargo test` green (27 lib + 4 bridge + 4 packaged_stack); the F-16
  boundary additionally caught the host bridge test's absolute plan/policy
  spellings, now cell-relative. docs/execution/OPERATIONS_AND_STARTUP.md
  lookup-order section updated to describe the two-rung fail-closed
  resolution and the env allowlist.

### Verification for Batch 8 remainder + Batch 9
Per-crate suites green during the batch: core, capability, thoth, planner,
ledger, sandbox, authority (lib+m0), extension (lib+cep0008), cli (incl.
resume_recovery 5, m4b, m13), server (all 24 binaries green post-F-16/SUP-06,
post-F-24). Final gates recorded below.

