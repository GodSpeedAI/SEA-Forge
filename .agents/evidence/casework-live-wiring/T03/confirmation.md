# T03 independent verification — real plan templates and seeded E2E cell (GAP-D)

Verdict: **APPROVE**

Verifier: independent critic/verifier session (not the builder), 2026-09-23/24.
Scope judged: `fixtures/cells/e2e/**`, the `casework-cell-init` justfile hunk,
`crates/sea-forge-server/tests/case_templates_live.rs`, and this evidence dir.
Every command and runtime observation below was executed/observed personally by
the verifier. The parallel T04 unit's files (`lib.rs`, `sfwp_case_mutations.rs`,
`case_ops/`, CLI task commands, `case_mutations.rs`) were excluded from judgment.

## Findings (each with personally-observed evidence)

1. **Both templates are real `PlanTemplate` YAMLs and load through the real
   loader.** Every construct maps to the actual schema: `PlanTemplate`
   (templates.rs:30-42: name/version/description/parameters/plan),
   `ParameterDef { type, required, default }` (templates.rs:19-26),
   `TemplateItem` incl. `item_kind`, `sandbox_class`, `markers`,
   `entry_criteria`, `entry_criteria_mode` (templates.rs:87-115),
   `TemplateOperation::WriteFile { path, content_hint }` tagged
   `kind: write_file` (templates.rs:126-131), `ItemKind` snake_case renames
   `sandboxed_task`/`milestone`/`human_task` (types.rs:49-60), `ItemMarkers
   { required, manual_activation }` (types.rs:63-70), `EntryCriteriaMode::All`
   → `all` (types.rs:75-81), `Sentry { on: {source,event}, if: {kind: settlement_status,
   status} }` (types.rs:83-101, tag `kind` snake_case), `SettlementCriteria.
   required_artifacts / require_approval` (types.rs:611-635). Runtime proof: over
   the live socket, `case_entry_options` returned both templates with their real
   parameter projections (param_type/required/default), and `case_preflight`
   instantiated them (see finding 8).
2. **Template A (e2e-sentry-chain) meets the spec**: two `SandboxedTask` items,
   item 2 gated on item 1 by a real `settlement_status`/`accepted` sentry, a
   required rollup `milestone` with `entry_criteria_mode: all` naming both
   tasks, and 4 parameters with type/default/constraint (`int` constraint
   enforced by `resolve_params` `parse::<i64>`, templates.rs:249-253; `path`
   safety at templates.rs:261-276). Comment header documents purpose. The live
   preflight response listed the three items in template order as
   SandboxedTask/SandboxedTask/Milestone with a `sha256:`-prefixed template
   digest precondition.
3. **Template B (e2e-signoff-gate) models the gate the way the planner really
   does — no invented approval TraceKind.** `ItemKind::HumanTask` with
   `settlement_criteria.require_approval: true`; the case engine parks it
   (`ItemKind::HumanTask => CaseAction::ParkHumanTask`, case_engine.rs:571) and
   the dispatch path appends `ItemActivated` with payload `{"human_task": true}`
   (case_dispatch.rs:261-269), leaving the case `active`
   (case_dispatch.rs:444-458). The test asserts exactly this event and standing,
   and it passed.
4. **`casework-cell-init` seeds correctly and is idempotent + non-clobbering.**
   I removed the runtime cell, ran the recipe fresh (server.yaml written with
   uid 1000 → `operator_local`/operator, both templates installed, policy
   installed), hashed all files (`/tmp/t03-verify/seed1.txt`), ran the recipe
   again — output "cell already initialized … not modified", hashes byte-identical
   (`diff` empty). Clobber probe: I appended a marker line to `server.yaml` and
   dropped a sentinel file in `templates/`, re-ran — both survived verbatim.
   Installed template/policy sha256s equal the checked-in fixtures exactly
   (133e1fe5…, 1e65ceed…, 4816032…). Seeded `server.yaml` matches the real
   `IdentityBinding { uid, actor_id, roles }` schema (identity.rs:246-256,
   `deny_unknown_fields`).
5. **Boot proof**: `just casework-server-up` built and started the server
   (listening, pid 999845); the socket answered a real `case_entry_options`
   call (both templates returned). `just casework-server-down` stopped it;
   afterwards no `sea-forge-server` process, no `server.sock`, no `server.pid`.
6. **Test gate green**: `cargo test -p sea-forge-server --test case_templates_live`
   → 7 passed / 0 failed. I read all seven bodies: each boots a real server
   (`run(config)` spawned on a temp cell, socket awaited) and drives it through
   a real `UnixStream` line-JSON client; assertions cover durable state
   (`case-events.jsonl` first event `CaseCreated` with the case_id; `case_plan`
   ledger record via `LedgerStream`; `plan.json` template_ref + item ids;
   horizon standing; zero cases created on identity refusal with
   `no_side_effect: true`). The seed helper enumerates the SAME fixtures dir
   the recipe globs (`CARGO_MANIFEST_DIR/../../fixtures/cells/e2e` vs the
   recipe's `fixtures/cells/e2e`), dynamically via `read_dir` — no forked
   template list — and `boot()` loads the seeded `server.yaml` through the real
   `ServerConfig::load`.
7. **Other gates**: `cargo test -p sea-forge-planner` all green (8 test
   binaries, 0 failures). `cargo test -p sea-forge-server` exit 0, 391 tests
   passed across 34 green binaries — including the parallel unit's
   `sfwp_case_mutations` (10/10) at the moment I ran it; no retry was needed
   because the parallel unit's tree happened to compile and pass then.
   `cargo fmt --all -- --check` exit 0 (fully clean now). The builder's
   gate-fmt.log residuals were all in the parallel unit's in-flight files
   (`case_ops/*`, `cli/commands/task.rs`, `lib.rs`) — none in T03's files; that
   attribution claim checked out, and the parallel unit has since cleaned them.
8. **Teeth re-run by me, independently, against the live server.** I authored my
   own cyclic template (`e2e-verifier-cyclic@0.1.0`: loop_first ⇄ loop_second,
   each `settlement_status: accepted` on the other), installed it into
   `<cell>/templates/`: `entry_options` listed it; `case_preflight` returned
   `ok: false` with error `plan_cycle_error: sentry dependency cycle detected`;
   `case_commit` (with actor + request_id) returned the same error; `cases/`
   did not exist — no case created. Removed it; the cell is back to exactly the
   two E2E templates. Code path confirmed: cycle detection is
   `check_satisfiability` (case_engine.rs:162-184), invoked from
   `validate_proposal` (case_engine.rs:394), which both preflight
   (sfwp/case.rs:220) and commit (`case_dispatch::submit`, case_dispatch.rs:65)
   run. The builder's own teeth transcript + request record
   (`requests/req-t03-teeth-cyclic-1.json`, status failed) show the same result.
9. **Constraints honored**: `sfwp/case.rs` is NOT in the working-tree diff
   (logic unchanged); the justfile has exactly one hunk (casework-cell-init);
   no `Cargo.toml` changes — `tempfile` is a pre-existing workspace
   dev-dependency (sea-forge-server Cargo.toml:45); no `apps/` files touched;
   nothing committed (fixtures/, the test file, and this evidence dir are all
   untracked, as instructed).

## Deviation judgments

- **(a) `PlanCreated` assertion dropped — SOUND.** The SFWP commit path never
  appends it, at HEAD or in the working tree: `case_dispatch::submit` appends
  only `TraceKind::CaseCreated` (case_dispatch.rs:107-114), commits a
  `case_plan` ledger record (line 79) and writes `plan.json` (line 105).
  `PlanCreated` is appended only by the CLI plan pipeline
  (crates/sea-forge-cli/src/pipeline.rs:287; trace helper
  crates/sea-forge-trace/src/lib.rs:325). The plan's `proves` string
  ("CaseCreated and PlanCreated") was factually wrong about the kernel's live
  path; asserting it would have required changing kernel behavior (forbidden)
  or fabricating evidence. The test instead asserts the durable plan truth that
  does exist (case_plan ledger record + plan.json naming template_ref and the
  item ids) and documents the reason in its header. Material difference from
  the plan text, correctly disclosed and correctly handled.
- **(b) `manual_activation` on both templates' tasks — SOUND for T03's scope.**
  Verified limitation: `LocalSandbox::execute` accepts only
  `ExecuteCommand` with non-empty argv and errors
  "execution request requires non-empty execute_command" for anything else
  (crates/sea-forge-sandbox/src/local.rs:40-48), and `execute_sandbox`
  hardcodes `write_only: false` (case_dispatch.rs:977) while dispatching only
  the first operation (717-721). A `write_file`-only SandboxedTask that
  auto-activated at commit would therefore fail in the sandbox during the
  commit journey. With `manual_activation`, `next_ready_actions` yields
  `Enable` (case_engine.rs:566-568) → `ItemEnabled` (case_dispatch.rs:194-201)
  → `enabled` standing in case_views — exactly the required post-commit
  sentry-chain standing (head enabled, dependent + milestone pending-blocked,
  nothing settled), with execution correctly deferred to T04's
  `case.advance`/`item.execute`. The sentry dependency itself is real and
  unweakened. The choice is documented in the template header.
- **(c) `request_id` included on commits — not a deviation.** The kernel
  requires it: `requires_durable_locator` includes `CaseCommit`
  (lib.rs:1387-1399) with `durable_locator_required` refusal (1411-1416). The
  test complies with the contract.
- **(d) Removal of the T00-era runtime `server.yaml` — ACCEPTABLE.**
  `.sea-forge/` is gitignored (`.gitignore` line 21 `/.sea-forge/`); the file
  was comment-only, untracked runtime state over empty ledgers. Nothing tracked
  was deleted (git log for `fixtures/` is empty — never committed). I re-proved
  the fresh-seed path myself by removing the whole cell and re-seeding to
  byte-identical hashes.
- **(policy scope) The policy genuinely allows only what the E2E cell needs.**
  `fixtures/cells/e2e/policy.yaml` matches the real `PolicyRule` schema
  (authority lib.rs:775-786) with supported kinds `write_file` (lib.rs:1186)
  and `approval_resolution` (lib.rs:1205); `default` is absent →
  `default_deny()` = "deny" (lib.rs:509-511). Only the `operator` role is
  granted anything. `write_file`'s `path_prefix: ""` is bounded by the
  evaluation context: the action is evaluated with `workspace_root` = the
  case's own run workspace (case_dispatch.rs:722-727, 777). Approval
  resolutions authorize against this same installed file
  (`<root>/authority/active-policy.json`, lib.rs:2948). The policy is
  cell-local and weakens nothing outside the temp cell.

## Minor observations (non-blocking)

- A refused cyclic commit still leaves a `drafts/<id>.json` scratch plan and a
  `requests/<id>.json` correlation record (status failed) in the cell —
  observed after my own teeth. No case, no case ledger, no template pin
  (`templates/.pins` absent; preflight deliberately does not pin,
  sfwp/case.rs:163-176). Benign, cell-local runtime state.
- `entry_options` lists a well-formed-but-cyclic template (listing is
  best-effort by design, sfwp/case.rs:66-69); preflight is the authoritative
  refusal point, which is exactly what the teeth exercises.

## Verdict rationale

All five deliverables exist, are real (no fabricated catalogs, no invented
TraceKinds, no forked seed), and are proven by gates I ran myself: 7/7 live
integration tests, planner green, full server suite exit 0 (391 passed), fmt
clean, twice-idempotent non-clobbering seed verified by hashes, boot/teardown
proof with a socket that answered, and an independently reproduced cyclic-sentry
teeth refusing at preflight and commit with no case created. The four reported
deviations are all verified as sound/acceptable with quoted code evidence.
