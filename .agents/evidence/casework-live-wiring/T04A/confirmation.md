# T04 UNIT A — Independent Confirmation

- Verdict: **APPROVE**
- Scope verified: plan T04 steps 1 and 3 (extract CLI case mutations into a library; route `approval.decide` in-process and delete the `run_cli` path), files under `crates/sea-forge-case-runner`, `crates/sea-forge-cli`, `crates/sea-forge-server`, `.agents/evidence/casework-live-wiring/T04A/`.
- Date: 2026-09-23. Branch `casework/live-wiring`. Independent verifier re-ran every gate and the teeth personally; nothing below is taken from builder claims.
- Out of scope (parallel T01, ignored per instructions): `apps/godspeed-cognitive-ui`, `apps/godspeed-casework-go/internal/contract`, `.agents/reports/interface-contracts`.

## Findings (all evidence personally observed)

1. **The library is real, logic-owning, and pure.** `crates/sea-forge-case-runner/src/case_ops/mod.rs` (489 lines) contains the verbatim bodies of `append_case_event`, `reopen`, `propose_item` (with `validate_proposal` cycle check, `proposed_by` carried on `PlanItem`, authority mediation, ledger commit, `PlanMutated` event) and `resolve_approval` (ledger replay, double-resolution guard, SoD on `proposed_by`, requester-inequality, criteria/hash cross-checks, `authorize_resolution` grant mint via `commit_typed("authority_decision", ...)` with verdict-Allow enforcement, TTL expiry leg, `approvals.jsonl` append via `sea_forge_core::approvals::append`). `mediation.rs` (243 lines) and `authority_views.rs` (134 lines) carry the mediation core and mirror rebuilds.
   - Purity: `rg -n "println!|print!|eprintln!|eprint!|clap|async |tokio|await|std::process|Command" crates/sea-forge-case-runner/src/case_ops/` → no matches.
   - No new dependencies: `git diff -- '**/Cargo.toml' Cargo.toml Cargo.lock` → empty. Every crate `case_ops` uses (`sea-forge-core`, `sea-forge-planner`, `sea-forge-ledger`, `sea-forge-authority`, `chrono`, `serde`, `serde_json`; dev `tempfile`) was already in `crates/sea-forge-case-runner/Cargo.toml`, and `sea-forge-cli/Cargo.toml:14` / `sea-forge-server/Cargo.toml:16` already depended on `sea-forge-case-runner`.
   - The CLI no longer holds the moved bodies: `rg -n "sod_violation|approval_resolution_authority_request|ingress_authority_request" crates/sea-forge-cli/src/` → no matches. Remaining `validate_proposal` references are in `plan_pipeline.rs` / `artifact.rs` (pre-existing, different pipelines, not in this diff). Remaining `commit_typed` sites in the CLI are other commands' records (migrate/recall/federation/manager/resume/project), untouched.

2. **CLI is a thin caller; behavior, output and exit codes preserved; CLI tests unmodified and green.**
   - `git diff --stat -- crates/sea-forge-cli/tests/` → empty (unmodified).
   - `cargo test -p sea-forge-cli` → 18 test binaries, **86 passed, 0 failed** (including `conformance_m15.rs` t15.4 SoD suites, 10 passed).
   - `approve.rs` prints the same three lines as before (`approval_id=…`, `status={:?}`, `resolved_by=…` — diff shows the old `println!` trio retained verbatim at the wrapper); `case.rs`/`mediated.rs`/`pipeline.rs` are 1:1 delegations or `pub use` re-exports.
   - Two non-observable micro-differences noted (judged in "Drift review" below): fail-fast policy check in the approve wrapper, and inlined `read_json` in `add_task` with identical error mapping (`ForgeError::io("read case state", …)`).

3. **`run_cli` deleted entirely; it had exactly one caller.** `git show HEAD:crates/sea-forge-server/src/lib.rs | grep -n run_cli` → only `2680: let result = run_cli(...)` (the call) and `2995: async fn run_cli(...)` (the definition). `rg -n "run_cli" crates/` in the working tree → no matches. Deleting the whole function (not just the approval leg) was therefore correct, as the builder reported.

4. **`decide()` calls the library in-process with the identity-verified actor and the same policy path the spawned CLI used.** `crates/sea-forge-server/src/lib.rs:2669` `let actor = actor_id.unwrap_or("operator_local")` — `actor_id` is the gate-verified actor threaded from `handle_request_as` (verified-identity block at lib.rs:1899/1926/1947), and `operator_local` is exactly the CLI's clap default (`crates/sea-forge-cli/src/main.rs:150` `#[arg(long, default_value = "operator_local")]`). Policy: `lib.rs:2674` `state.root.join("authority/active-policy.json")`, identical to the CLI default the old spawned command relied on (`crates/sea-forge-cli/src/commands/mediated.rs:20-24` `policy_path` → `root.join("authority/active-policy.json")`; the old `run_cli` args had no `--policy`). The call is wrapped in `tokio::task::spawn_blocking` (lib.rs:2682). The pre-existing server gate in front of `decide` (identity_not_bound / separation_of_duty / separation_of_duty_unverifiable, lib.rs:1455-1500) is unchanged.

5. **Response shape preserved.** lib.rs:2708-2712 renders `format!("approval_id={approval_id}\nstatus={:?}\nresolved_by={actor}\n", resolved.status)` into `{"ok": true, "output": …}` — byte-equivalent to the CLI stdout the deleted `run_cli` captured (same three `println!` lines). Note: no test in the repo asserts the exact `output` string (searched all `crates/sea-forge-server/tests/` and the workbench bridge, which treats output as opaque); the `ok:true` leg is asserted by the new test, and the format string was verified against the CLI printlns by reading both. Recorded as a minor gap, not a defect — no prior test covered it either.

6. **Five focused library tests exist and assert durable state.** `crates/sea-forge-case-runner/tests/case_ops.rs`: `reopen_reactivates_a_closed_case_and_records_the_event` (case.json → Active, one `case_reopened` event with actor, `case_event` in the case ledger), `reopen_refuses_a_case_that_is_not_closed`, `propose_item_refuses_a_cycle_and_leaves_the_plan_untouched` (`plan_cycle_error`, **plan.json byte-identical**, no `PlanMutated`), `the_proposer_cannot_resolve_its_own_items_approval` (`sod_violation`, no `approval_resolution` ledger record, **approvals.jsonl not created**), `an_uninvolved_actor_resolves_the_approval_and_mints_the_authority_decision` (journal line with `resolved_by`, **grant mint**: exactly 2 authority decisions, second is `verdict=allow` on `resource_type=approval_resolution`; re-resolve refused). `cargo test -p sea-forge-case-runner` → **5 passed + 4 passed (conformance_m5; 2 pre-existing ignored), 0 failed**.

7. **New server identity test exists and is the only test added.** `approval_decide_resolves_as_the_verified_actor_and_refuses_unbound_ones` in `crates/sea-forge-server/tests/conformance_identity.rs` (unbound `mallory` refused with `identity_not_bound` and approvals.jsonl byte-identical to before; `operator_b` resolves in-process and the journal's last line names `resolved_by=operator_b`, `status=approved`, matching `approval_id`). `#[tokio::test]` count in that file: 14 at HEAD → 15 now. Filtered run: `cargo test -p sea-forge-server --test conformance_identity approval_decide_resolves_…` → **1 passed**.

8. **All five required gates green (run by me):**
   - `cargo test -p sea-forge-cli` → 86 passed / 0 failed.
   - `cargo test -p sea-forge-case-runner` → 9 passed / 0 failed (2 pre-existing ignored).
   - `cargo test -p sea-forge-server` → full suite green (lib 83 passed; every integration binary "test result: ok", incl. conformance_identity 15, conformance_approvals 7, conformance_sfwp 19).
   - `cargo fmt --all -- --check` → exit 0, no output.
   - `just no-async-kernel` → `ok: no async runtime or HTTP client in 19 kernel crates`, exit 0.

9. **Teeth re-run (personally executed, verbatim):** backed up `crates/sea-forge-server/src/lib.rs` (md5 `dee61c…`), changed lib.rs:2669 to `let actor = "operator_a".to_string();` (fixed submitter actor). Filtered run of the new test **FAILED** exactly as required:
   ```
   thread 'approval_decide_resolves_as_the_verified_actor_and_refuses_unbound_ones' panicked at
   crates/sea-forge-server/tests/conformance_identity.rs:464:5:
   assertion `left == right` failed: {"error":"approval resolver must differ from requester"}
     left: Null
    right: true
   test result: FAILED. 0 passed; 1 failed; ...
   ```
   (the library's requester-SoD check fired on the substituted actor and the resolution was refused). Reverted by restoring the backup: md5 identical to the pre-probe state, `diff` against the backup empty, `git diff --stat` identical to the pre-probe stat (7 files, 207 insertions, 865 deletions — no residue). Filtered test re-run → **1 passed**. The test has genuine teeth against exactly the fixed-actor attack.

10. **Operating-contract rules respected.** ADR-003 additive rule: no `Request` variant was added or reshaped (the lib.rs diff touches no enum definition; Approve/Reject/Decide parse identically; only internal handling moved) — conformance_sfwp green, no workbench schema regeneration required. TraceKind rule: no TraceKind invented; the library uses only existing enum members (`CaseReopened`, `PlanMutated`); the real enum (`crates/sea-forge-core/src/types.rs:501-523`) was read and contains **no** approval-related kind. Kernel-sync boundary: see judgment (c).

## Judgment of the builder's reported deviations

- **(1) No trace append added for the approval path — SOUND.** `git show HEAD:crates/sea-forge-cli/src/commands/approve.rs | grep -n "append_case_event\|case-events\|TraceKind\|case_events"` → no matches: the old CLI approval path never wrote `case-events.jsonl`; its only journal write was `approvals::append` (lines 209/232), itself a re-export of `sea_forge_core::approvals::append` (`crates/sea-forge-cli/src/approvals.rs:11`), which is exactly what the library calls. Adding a case-event append would have been a behavior change requiring an invented TraceKind — the opposite of both the extraction mandate and the plan's TraceKind rule. The instruction's "trace append" capability is owned by the library (`case_ops::append_case_event`, used by reopen/propose_item). Deviation accepted.
- **(2) Stale-comment amendment in `a_submitter_cannot_approve_their_own_work_but_another_actor_can` — SOUND.** The diff changes only the comment block (the old text described the `run_cli` limitation that no longer exists); both assertions (`error_class == "separation_of_duty"`, `no_side_effect == true`, the not-separation_of_duty second-actor check, and the criteria-message guard) are byte-identical, verified in the diff and by reading the full test. Necessary and honest: the old comment described deleted machinery.
- **(3) `pipeline.rs` keeps `pub(crate)` thin wrappers for two internal call sites — SOUND.** Real call sites confirmed: `pipeline.rs:294` (`load_opaque_constraints`) and `:499` (`rebuild_authority_mirrors`), plus tests at `:836`/`:840`. The wrappers delegate 1:1 with zero logic. Slightly inconsistent with mediated.rs's `pub use` re-export stylistically, but behavior-neutral.

## Protocol judgment calls

- **(a)** covered by deviation (1) above — sound.
- **(b) Extraction into `sea-forge-case-runner` rather than a new crate — dependency-sound.** `case_ops` needs only `sea-forge-core/planner/ledger/authority`, all pre-existing dependencies of case-runner; both consumers (CLI, server) already depended on case-runner, so no manifest moved. A new crate would have been an architecture/dependency-graph change requiring ask-first under root AGENTS.md; the plan's T04 text itself names "sea-forge-case-runner, or a new case-ops module" as the home — both conditions satisfied. `just no-async-kernel` green.
- **(c) `spawn_blocking` in the server — respects the kernel-sync boundary.** crates/AGENTS.md §1 makes `sea-forge-server` one of the two async edge crates; the library stays strictly synchronous (purity check in Finding 1) and the server offloads the blocking ledger work to the blocking pool instead of blocking the runtime (also the plan's own T04 redesign trigger avoidance). `just no-async-kernel` confirms all 19 kernel crates clean.
- **(d) Behavioral drift review — two differences found, both judged immaterial:**
  1. Server error **text** for refused resolutions changed from the old `run_cli` wrapper (`"exit Some(N): <stderr>"`) to the direct `ForgeError` string inside the same `{"error": string}` shape. This is inherent to "call the library in-process" (the instruction mandates the in-process route); the underlying message is the same one the CLI rendered, and no test relied on the old wrapper text (full server suite green).
  2. The CLI approve wrapper now fails fast on a missing policy before ledger replay (old code surfaced the identical `"approval policy is missing"` error only later). Unreachable through the binary: `main.rs` always resolves `policy_path` and passes `Some(...)` (main.rs:694/713). No observable drift.
  3. Everything else checked identical: argument order and defaults (actor `operator_local`, policy `active-policy.json`, note passthrough), the three-line output, the requester-inequality and SoD ordering inside the library (verbatim move), the TTL-expiry leg, `permission_broker.resolve` + `publish_event` on success only (as before), and the unknown-verdict refusal in `Request::Decide`.

## Minor observations (non-blocking, for the record)

- No test pins the exact `output` string of an approval response (Finding 5); if a future task consumes that field's bytes (gateway/UT), a golden assertion would be cheap insurance.
- `gate-fmt.log` in this directory is empty (fmt produced no output — consistent with its exit 0).

Confirmation note only; no other file was modified by the verifier, and nothing was committed. The teeth probe was reverted byte-exactly (md5-verified) before this note was written.
