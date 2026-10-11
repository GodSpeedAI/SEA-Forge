# T04 independent confirmation — units B and C (2026-09-23)

Critic: t04 independent critic (second round). Scope: unit B (commit 9c153b2,
SFWP case mutation/execution verbs + event publishing) and unit C (commit
0fb4703, opt-in case-advance supervisor) of plan
`.agents/plans/2026-09-23-casework-live-wiring-production.plan.yaml` (T04,
proof P2, confirmation: independent). Unit A (1b73724) already has a valid
independent confirmation at `../T04A/confirmation.md` and was not re-judged.
The critic built none of this. Commits were reviewed with `git show`; gates
and attacks were run on the working tree at 85545a6 (unit C plus the
independent T02 change, branch `casework/live-wiring`).

## VERDICT: APPROVE

Every claim in the unit B and unit C specs is implemented and independently
re-proven, all required gates are green with captured exit codes, the three
permanent plan teeth pass filtered re-runs, and an original runtime attack
over a real server socket on a fresh cell passed. No code defect found. The
deviations listed below are boundary/documentation notes, none of which
breaks a T04 requirement; two of them (1, 2) should be carried into T05/T06
as known edges.

## Findings (each personally observed)

1. **ADR-003 additivity holds.** `git show 9c153b2 -- crates/sea-forge-server/src/lib.rs`:
   the Request enum diff adds exactly seven new variants (CaseAddItem,
   CaseReopen, CaseTerminate, CaseAdvance, ItemExecute, HumanTaskComplete,
   ArtifactGet; lib.rs:846-914 in the post-commit file) and touches no
   existing variant or its serde attributes — existing variants appear in the
   diff only as new arms added to exhaustive matches (`request_id`,
   `requires_durable_locator`, `is_protected`). Commit 0fb4703 adds no Request
   variants at all (its lib.rs diff is the supervisor spawn, `case_locks`, and
   the `AdvanceCaller` threading).
2. **Registration complete.** `crates/sea-forge-server/src/sfwp/mod.rs:256-281`
   adds all seven methods to `IMPLEMENTED_METHODS` (six `Command`, `artifact.get`
   `Inspect`); `mod.rs:403-409` adds the seven result types to `SCHEMA_TYPES`;
   `src/bin/gen_sfwp_schema.rs:123-129` emits their schemas; the drift test
   (`tests/conformance_sfwp.rs:740-768`, regenerates into a temp dir and diffs
   the committed tree — non-destructive) passed inside the green 3-crate gate,
   and commit 9c153b2 carries the regenerated workbench schema/validator files.
3. **identity.rs classification is as specced.** `crates/sea-forge-server/src/identity.rs:815-820`:
   `Request::CaseAddItem { .. } | Request::CaseReopen { .. } |
   Request::CaseTerminate { .. } | Request::CaseAdvance { .. } |
   Request::ItemExecute { .. } | Request::HumanTaskComplete { .. } => true`
   (protected), and `identity.rs:849-850`:
   `// Content-addressed read over already-committed evidence records.`
   `| Request::ArtifactGet { .. } => false` (inspect). The match is exhaustive
   with no `_` arm, so the classification cannot silently rot.
4. **Authority/SoD bind in the shared library, same as the CLI.** Binding
   points, all `mediation::authorize_read` over `AuthorityAction::Reserved`
   (a real authority evaluation: policy bundle load, identity resolve,
   grant.authorize — `case_ops/mediation.rs:113-125`):
   - case.reopen → `case_reopen`, `crates/sea-forge-case-runner/src/case_ops/mod.rs:154-163`
   - case.terminate → `case_terminate`, `mod.rs:215-224`
   - case.add_item → `discretionary_task_add`, `mod.rs:273-282`, *after*
     `validate_proposal` (cycle + schema checks, `mod.rs:272`;
     DFS cycle detection at `sea-forge-planner/src/case_engine.rs:161-182,401`)
     and *before* the ledger commit and plan.json materialization
     (`mod.rs:283-293`) — a refusal leaves nothing behind.
   - human_task.complete → `human_task_completion`, `mod.rs:354-363`.
   - advance episodes: `execute_sandbox` evaluates the episode's own authority
     decision for the identity-gate-verified role (F-08,
     `case_dispatch.rs:743-749`), and `case_mutations.rs:411-425` passes that
     verified role into the shared executor — no second authority path.
   - approval SoD (the tooth): proposer-vs-resolver refusal at
     `case_ops/mod.rs:497-512` (`item.proposed_by == Some(actor)` →
     `ForgeError::Plan { class: "sod_violation", .. }`), plus
     resolver≠requester at `mod.rs:555-559`.
   The CLI is a genuine thin caller (`git show 9c153b2 -- crates/sea-forge-cli`:
   `task.rs` deletes its private copy and delegates; `case.rs` re-export removed).
5. **proposed_by overwrite.** `case_mutations.rs:197`:
   `item.proposed_by = Some(actor.clone());` before the library call; the
   result reports the verified actor (`CaseAddItemResult.proposed_by`,
   `case_mutations.rs:44-52`). Confirmed live in my attack (below): a legal
   add over the socket landed in durable plan.json with
   `proposed_by == "operator_local"`.
6. **Mandatory justification.** `case_mutations.rs:297-301`: a blank-trimmed
   justification is refused `ForgeError::Input` before any resolution
   (likewise terminate's reason, `case_mutations.rs:255-259`). Test asserts
   refusal + zero writes (`tests/sfwp_case_mutations.rs:813-834`); my attack
   reproduced it against a real server process.
7. **artifact_get bounded, content-addressed, read-only.** Bound:
   `case_mutations.rs:41` `pub(crate) const MAX_ARTIFACT_BYTES: u64 = 1024 * 1024;`
   enforced by `fs::metadata` *before* the read (`:478-487`, typed refusal over
   the ceiling). Digest grammar (64-char hex) checked before touching disk
   (`:455-459`); returned bytes re-hashed and must equal the digest
   (`:490-495`, integrity error otherwise); evidence journals over the 64 MiB
   read cap are skipped (`sfwp/mod.rs:60,70-74`). Read-only: `is_protected`
   false, no ledger/trace write anywhere in the function.
8. **TraceKind discipline.** Every `TraceKind::` emitted by the two commits
   (grepped: PlanMutated, CaseReopened, CaseTerminated, HumanTaskCompleted,
   CaseClosed, ItemEnabled, ItemActivated, MilestoneAchieved,
   SettlementRecorded, ArtifactCaptured) is a member of the real enum at
   `crates/sea-forge-core/src/types.rs:501-526`. No invented kinds. The extra
   kinds in the advance sequence (SettlementRecorded, MilestoneAchieved,
   CaseClosed) are engine semantics via `apply_episode_completion`, asserted
   as exact jsonl vectors by the tests.
9. **Cannot-miss holds for everything the two units add, and the supervisor
   shares the choke point.** `run_case_mutation` (`case_mutations.rs:133-179`)
   hands every verb a notify sink; the blocking closure owns the channel
   sender, the single publisher task awaits each durable events-ledger append
   (`publish_event`, flock+fsync, `lib.rs:208-234`) before the next, and the
   response returns only after `publisher.await` drains (`:177`) — so the set
   of published frames equals the set of appended case events by construction.
   Audit of every appender reachable from the verbs: the `case_ops::*_with`
   functions notify after each append (`case_ops/mod.rs:128,171,232,301,371,394,416`);
   the advance engine appends only via `append_notify`
   (`case_ops/advance.rs:83-97`) and captures `apply_episode_completion`'s
   appends by delta (`advance.rs:350-363`); `execute_sandbox` never writes
   case-events.jsonl (it creates the run's own `trace.jsonl`,
   `case_dispatch.rs:737`); the supervisor calls the same `advance()` helper
   (`supervisor.rs:166-177`), so there is no unsinked append path in units
   B/C. The documented tradeoff — a failed events-ledger append is logged
   loudly and the frame consumed (`case_mutations.rs:151-163`), case jsonl
   remaining the source of truth and `events.get_range` reading exactly what
   landed — is honest and acceptable: the alternative (failing or un-appending
   a durable case mutation because a *projection* ledger write failed) would
   be worse, the case journal stays authoritative, and the failure mode
   requires an events-ledger disk fault. The kill/resume tooth passed.
10. **Unit C fail-closed default.** `SupervisorConfig` serde defaults to
    `enabled: false` and the absent section parses to the struct default
    (unit test `config.rs` `an_absent_supervisor_section_parses_to_disabled_bounded_defaults`);
    spawn condition is exactly `if supervisor_settings.enabled { supervisor::spawn(..) }`
    (`lib.rs:1221-1224`, 0fb4703 diff); bounds validated even while disabled —
    poll 1..=3600s, cases 1..=8, actor `valid_id_segment` ≤128 chars
    (`config.rs:213-247`), each refusal covered by `load_rejects_out_of_range_supervisor_settings`.
    Runtime proof: test (b) — default config, ready enabled SandboxedTask
    untouched for 6.5 s (> the 5 s default poll + boot pass),
    case-events.jsonl byte-identical, zero run directories
    (`tests/sfwp_supervisor.rs:346-368`).
11. **Bounded concurrency and lock order.** Dedicated supervisor slot
    semaphore (`supervisor.rs:84`, acquired per case `:126`) **and** one
    shared run-pool permit per pass (`supervisor.rs:159`) — the same
    `state.semaphore` the verb takes (`lib.rs` `advance_response`). Per-case
    keyed lock acquired inside the shared `advance()` before any append
    (`case_mutations.rs:409`, released `:437`, pruned `:444`). Acyclicity
    argued from the code: the only three locks are slot → run-pool →
    per-case; the supervisor takes them strictly outer-to-inner; the verb
    enters at run-pool → per-case; no code path acquires the run pool while
    holding a case lock, and none acquires a slot while holding anything;
    `acquire_case_lock` releases the key-map mutex before awaiting the
    per-case mutex, so the map guard is a leaf. No cycle is constructible.
12. **Service-role attribution.** `ActorRole::Service` is a real variant
    (`types.rs:302`) and is the role every supervisor authority evaluation
    sees (`supervisor.rs:169-171`); actor is `supervisor.actor` from config,
    never an end user, never `operator_local`. Durable assertion quoted:
    `assert!(trace.contains("cell_supervisor"), "supervisor episodes must be
    attributed to supervisor.actor")` and
    `assert!(!trace.contains("operator_local"), ...)` over the episode's
    run `trace.jsonl` (`tests/sfwp_supervisor.rs:441-448`). An operator-only
    policy fails closed for service-role episodes (the test must install an
    explicit `actor_role: service` rule to let episodes run,
    `sfwp_supervisor.rs:95-108`).
13. **SandboxedTask-only scope (double filter).** `advance.rs:140-148` drops
    `ParkHumanTask` and any non-SandboxedTask `Activate` before the idle
    check, and `advance.rs:181-187` retains only SandboxedTask in the
    activation set — both under `matches!(scope, AdvanceScope::Supervisor)`;
    the episode loop independently skips non-SandboxedTask (`:276-278`).
    Idle breaks *before* `write_json` (`:188-190` vs `:370`), so a
    human-only case is zero writes: test (d) shows byte-identical jsonl
    across three polls and `approvals.jsonl` never created
    (`tests/sfwp_supervisor.rs:477-509`).
14. **Missing-policy skip.** `supervisor.rs:97-104`: absent
    `authority/active-policy.json` → one `warn`, sleep, `continue` — no case
    is enumerated, nothing is written.
15. **Gates (all first attempt, TRUE exit codes in `critic/gates-exit-codes.txt`).**
    - `cargo test -p sea-forge-cli -p sea-forge-case-runner -p sea-forge-server`
      → EXIT=0; 58 `test result: ok` suites, 0 failed
      (`gate-3crate-attempt1.log`).
    - `cargo test -p sea-forge-server --test sfwp_case_mutations` → EXIT=0,
      10 passed / 0 failed (`gate-mutations-attempt1.log`).
    - `cargo test -p sea-forge-server --test sfwp_supervisor` → EXIT=0,
      4 passed / 0 failed, 6.52 s — the timing windows really executed
      (`gate-supervisor-attempt1.log`).
    - `cargo test -p sea-forge-server --test case_templates_live` (T03
      regression) → EXIT=0, 7 passed / 0 failed (`gate-templates-attempt1.log`).
    - `cargo fmt --all -- --check` → EXIT=0.
    - `just no-async-kernel` → EXIT=0, `ok: no async runtime or HTTP client
      in 19 kernel crates`.
16. **Teeth re-run (filtered, `--nocapture`, transcripts `critic/tooth{1,2,3}.log`).**
    - cycle refusal (`case_add_item_cycle_refusal_leaves_the_plan_byte_identical`):
      1 passed / 0 failed — plan.json byte-identical, no PlanMutated.
    - proposer self-approval (`the_proposer_cannot_resolve_its_own_discretionary_items_approval`):
      1 passed / 0 failed — `sod_violation`, approvals.jsonl unchanged.
    - subscriber kill/resume (`subscriber_kill_and_resume_replays_without_gaps_or_duplicates`):
      1 passed / 0 failed — replay cross-checked against `events.get_range`,
      no gaps/duplicates, strictly monotonic cursors.
17. **Original runtime attack (not a re-run of the suite).** Driver
    `critic/attack.py` + `critic/attack.sh`, transcript
    `critic/attack-run.log`: booted the real `sea-forge-server` binary
    (zero-arg, env-configured) on a fresh cell `/tmp/sf-t04critic-H8bj` with a
    uid-bound operator identity and an allow policy; spoke NDJSON over the
    real socket. Results:
    - cycle `case_add_item` → `{"error_class": "plan_cycle_error"}`;
      plan.json sha256 `7f6f4495…ede7a8` **identical** before/after;
      case-events.jsonl 2 → 2 lines; no `plan_mutated`.
    - `case_terminate` with blank reason → refused `input_error`, zero
      writes, case still `active`.
    - positive control: legal add → ok, `proposed_by: "operator_local"`,
      durable plan.json carries the item with the proposer overwrite, last
      jsonl kind `plan_mutated` (proves the driver, not a connection failure,
      produced the refusals above). ATTACK RESULT: PASS.
18. **Test bodies assert durable state, not response shapes.**
    `tests/sfwp_case_mutations.rs` (all 10) and `tests/sfwp_supervisor.rs`
    (all 4) read `case-events.jsonl` kind vectors, plan.json bytes/hashes,
    case.json state, run `trace.jsonl` kinds, evidence records, run-directory
    counts; refusals assert event-count equality or byte-identity. Weakest
    assertions (still adequate): the supervisor attribution check is a
    substring containment on the run trace rather than a field-level actor_id
    comparison (paired with a `!contains("operator_local")` negative), and
    the resume tooth observes only one live frame before the kill (mitigated
    by the `events.get_range` durable cross-check).

## Deviations from the two specs (all judged non-blocking; recorded for T05/T06)

1. **Cannot-miss is scoped to the T04 verbs, not to every case-events.jsonl
   writer.** The pre-existing commit/dispatch surfaces (`submit` and
   `case.commit`, both funnelling into `case_dispatch::submit`) append
   CaseCreated/PlanCreated/ItemEnabled/… to case-events.jsonl via
   `CaseRunner::append_event` without a sink (`case_dispatch.rs:107-327`) and
   publish only one coarse `case.submitted` frame afterwards
   (`lib.rs:2807-2813`). The module doc's opening sentence
   (`case_mutations.rs:14-16`, "Every append to a case's case-events.jsonl
   flows through exactly one of two choke points … and both notify a host
   sink") is over-broad as literally worded; it is true only for the paths
   units B/C add. Judgment: acceptable for T04 — the unit B spec sentence
   lives in the paragraph defining the new verbs, the tooth proves the verbs,
   and retrofitting the legacy dispatch loop was not a T04 step — but T05/T06
   must know that a case driven through `submit`/dispatch (rather than the
   new verbs) surfaces only the coarse frame on the bus.
2. **CommandStarted/CommandFinished/ArtifactCaptured of an advance episode
   are run-journal events, not bus frames.** `execute_sandbox` writes them to
   the run's `trace.jsonl` (`case_dispatch.rs:737,876-954`), not
   case-events.jsonl, so they are not published as `case.trace.*` frames; the
   case-level bus sequence for an episode is item_activated →
   settlement_recorded → item_completed/terminated/failed. The unit B spec's
   "emits ItemActivated, CommandStarted, …" is satisfied (the test asserts
   the run trace, `sfwp_case_mutations.rs:694-705`), but "every trace append
   publishes an EventFrame" does not cover run-journal appends. Judgment:
   acceptable for T04 (case progress — including downstream ItemEnabled — is
   on the bus, which is what the plan's teeth and T10 L4's SSE requirement
   need); flagged for SEAM-5/T06/T09, whose ExecutionPill expects "command
   frames" over SSE — those will need a bridge or an explicit design decision.
3. **approval.decide error responses gained an additive `error_class` field**
   (`lib.rs` `decide`, 9c153b2). Responses only; ADR-003 (request variants
   never reshaped) is untouched. Note only.
4. **Legacy dispatched episodes now capture stdout/stderr as
   ArtifactCaptured + evidence records** (`case_dispatch.rs:927-948`, 9c153b2)
   for parity with the CLI pipeline — a behaviour change to the pre-existing
   dispatch surface beyond the strict verb scope, justified in-code (sentry
   `ArtifactExists` predicates could otherwise never fire on server-side
   episodes), and green across all gates. Note only.
5. **Supervisor episode timeout is fixed (600 s, `supervisor.rs:73`)** rather
   than configurable; the spec set no timeout requirement, and the bound
   protects the shared pool. Note only.
6. **Engine names differ cosmetically from the spec.** The advance engine
   calls `next_case_actions` (of which `CaseRunner::next_ready_actions` is a
   verbatim alias, `case-runner/src/lib.rs:27-29`), `CaseRunner::append_event`
   and `apply_episode_completion`; `run_sandboxed_episode` (a no-op tracing
   seam, `lib.rs:104-108`) is not used — execution flows through the
   `EpisodeExecutor` closure into the same `execute_sandbox` the dispatcher
   uses, which is the substantive requirement ("no second execution path").
   Note only.

## Note on the earlier builder-written `T04/confirmation.md`

`../confirmation.md` (commit 0fb4703) was written by the building agent after
an earlier critic's procedural REJECT; that critic's report is not preserved
in this repository, so its claims cannot be audited and cannot count as the
independent confirmation T04's `confirmation: independent` requires — a
builder-authored settlement note is builder verification, whatever it
reconciles. The file also self-discloses gate reruns without captured exit
codes ("No EXIT= trailer"). This document, produced by an agent that built
none of T04, with its own gate logs, teeth transcripts and runtime attack
under `critic/`, is the operative independent verdict; `confirmation.md`
stands untouched as a historical artifact. With this APPROVE, T04's
independent-confirmation requirement for units B and C is now genuinely
satisfied (unit A already satisfied by `../T04A/confirmation.md`).

## Evidence index (`.agents/evidence/casework-live-wiring/T04/critic/`)

- `gates.sh`, `gates-exit-codes.txt`, `gate-*-attempt1.log` — six gates, all EXIT=0
- `tooth1.log`, `tooth2.log`, `tooth3.log` — teeth re-runs, 1 passed / 0 failed each
- `attack.py`, `attack.sh`, `attack-run.log`, `attack-server.log` — runtime attack, PASS
