# T06 independent confirmation — round 2 (re-verification of the F1-F7 fixes)

- Verdict: **APPROVE**
- Verifier: independent critic, round 2 (did not build T06 and did not build/fix the
  F1-F7 changes; builder session = a1129f1, fixes = commit b5fd2c3)
- Date: 2026-09-26 (verification window 21:36-22:20 -0400)
- Re-verifies: round 1's REJECT (T06/critic/confirmation.md, findings F1-F6) against the
  fixes in b5fd2c3, plus the kernel recovery reconciliation (F7) those fixes surfaced.
- Basis: every finding below was closed by THIS critic's own observation — source reads,
  live tests re-run with -v, and a manual production-binary drive over real HTTP against a
  real kernel cell I seeded myself (/tmp/t06c-critic-cell, since removed; production
  gateway binary built with NO build tags serving provenance go:live:sfwp). Raw captures
  under critic2/runtime/, gate logs under critic2/gates/, test transcripts under
  critic2/teeth/.

## 1. Gates re-run by this critic (true exits captured)

| Gate | Result | Evidence |
|---|---|---|
| `go vet ./...` | EXIT=0 | gates/go-vet-critic2.log |
| `go test -race -count=1 ./...` | EXIT=0 (9 pkgs ok) | gates/go-test-race-critic2.log |
| `go test -race -count=1 -tags live -p 1 ./internal/...` | EXIT=0 (full sweep, one pass) | gates/go-test-live-p1-critic2.log |
| `gofmt -l .` | clean (empty, exit 0) | gates/gofmt-critic2.log |
| `cargo test -p sea-forge-cli -p sea-forge-case-runner -p sea-forge-server` | EXIT=0, 58 suites, 0 failed | gates/cargo-three-crate-critic2.log |
| `cargo fmt --all -- --check` | clean (empty, exit 0) | gates/cargo-fmt-check-critic2.log |
| `go test -race -count=1 -tags casework_fixture ./internal/...` | 1 flake in ~10 rounds (pre-existing, see 5.3) | gates/go-test-fixture-tag-critic2.log + gates/fixture-tag-flake-note.md |
| `cargo test -p sea-forge-server --lib correlation` | 13/13 PASS | inline (section 4e) |
| Production build + `go tool nm` fixture probe | 0 fixture symbols | gates/nm-fixture-probe-critic2.log |
| `go test -race -count=1 ./internal/contract/` (T01 goldens) | 5/5 PASS | teeth/contract-goldens-critic2.log |
| Live projection golden (TestLiveGoldenSentryChainSnapshot) | PASS | teeth/golden-live-critic2.log |

No new dependencies: `git show b5fd2c3 --stat` touches no go.mod/go.sum.

### The -p 1 live-gate posture change — SOUND, not a weakening

The change (fixes/summary.md, decision log D-3-followups-2) serializes the live sweep
across packages because each package's TestMain boots its own kernel cell and ~5
concurrent kernels made restart/resume/relay tests load-flaky. Judged sound because:
(1) no coverage is lost — every test still runs -count=1 -race against a real kernel;
(2) there is no `t.Parallel` anywhere in these packages, so within-package order is
unchanged (Go already runs tests sequentially); (3) it matches the repo's existing
determinism discipline — `.cargo/config.toml` pins `jobs = 1` for the Rust side;
(4) I reproduced the claimed benefit myself: my full -p 1 sweep was green, and my
targeted live tests were green both before and after it. The residual cost is wall time
only.

## 2. Per-finding closure (all verified by this critic's own evidence)

### F1 — perspective override against the real kernel: CLOSED

- Code: `projection.LiveSource.VerifyPerspective` (internal/projection/live.go:255-272)
  now sends `Actor: s.gateway` with `OnBehalfOf` = the requested (actor, role), the same
  delegation shape intents use. The empty-actor-block defect is gone.
- Checked-in live test re-run with -v: TestLiveWorldPerspectiveVerifiesAgainstTheKernel
  PASS (teeth/f1-perspective-live-critic2.log).
- My own manual drive (runtime/f1-drive.md and f1-*.json), production binary over real
  HTTP against my own cell whose server.yaml binds an EXTRA actor `operator_b` (uid 3103,
  role operator) that is deliberately NOT in `delegable_actors`:
  - `GET /api/world?actor=operator_local&role=operator` -> HTTP 200; after committing a
    case through the served surface, the same request returns the real operator
    perspective (task_prepare READY_TO_BEGIN offering EXECUTE_ITEM, task_publish WAITING
    with the sentry explanation, milestone) — real standing, not a stub.
  - `GET /api/world?actor=operator_b&role=operator` (BOUND but NOT allowlisted) -> HTTP
    403, typed: `authority_denied ... actor 'operator_b' is not in this cell's
    gateway-delegable allowlist` (runtime/f1-bound-not-allowlisted.json). This is the
    exact negative round 1 required; the checked-in test's negative (operator_c) is
    neither bound nor allowlisted — same allowlist refusal, see 5.4.
  - `GET /api/world?actor=operator_c&role=operator` -> HTTP 403 typed.

### F2 — ADD_DISCRETIONARY_WORK accepted kernel write: CLOSED

- Code: intents.go:303-344 constructs a minimal VALID proposal (kernelItemKind defaults
  to sandboxed_task, one governed write_file op derived from the title,
  RequiredArtifacts naming the written path, DependsOn anchoring to stage_id).
- Checked-in live test re-run with -v: TestLiveDiscretionaryAddProducesAnAcceptedKernelWrite
  PASS (teeth/f2-discretionary-live-critic2.log); it asserts plan_mutated==1 from durable
  state and an item_completed settlement accepted with the write_only basis.
- My own drive over served HTTP (runtime/f2-*.json): add -> `success:true` with a
  strictly-advanced new_cursor and resulting_object item-disc-89880b6a; executed
  task_prepare then the item (one honest STALE_PROJECTION on the way — the staleness
  guard catching a downstream frame, retried with the refusal's own fresh cursor);
  durable truth read off disk afterwards (runtime/f2-durable-verify.md):
  - `plan_mutated` count = 1 (exactly once),
  - `item_completed` for item-disc-89880b6a with `settlement: accepted`,
  - the run's settlement.json: `status: accepted`, basis includes `write_only` and
    `required_artifact_present:discretionary/critic2-augmented-notes.md`.

### F3 — E2E policy escalates exactly the signoff-gate draft write; L5 flow reachable: CLOSED

- fixtures/cells/e2e/policy.yaml: `escalate-e2e-signoff-gate-write` is the FIRST rule
  (first-match-wins documented), verdict escalate, actor_role operator,
  operation_kind write_file, path_prefix `review` — minimal: the only served write under
  review/ is the signoff-gate's task_draft (sentry-chain writes work/, discretionary adds
  write discretionary/, both still under the allow rule). The kernel's matching is
  segment-aware and unit-pinned: sea-forge-core/src/path.rs `path_prefix_matches` +
  `prefix_is_segment_aware` test (`review` matches review/draft.md, not review-private/x).
- The L5 flow, driven by me end-to-end through the served surface of my own cell
  (runtime/f3-*.json, captured in f1-drive.md):
  1. PROPOSE_CASE e2e-signoff-gate@0.1.0 (operator_local) -> success.
  2. EXECUTE_ITEM task_draft -> success; durable case ledger shows
     settlement_recorded status `escalated`, and the cell's approvals.jsonl gains
     apr_0001 status `pending` (the kernel's only approval-opening path, via episode
     dispatch escalation — policy comment cites case_dispatch.rs accurately).
  3. operator_local APPROVE_HUMAN_TASK -> UNAUTHORIZED_ROLE (gateway role guard, no
     kernel call).
  4. rso_local (R-SO) APPROVE_HUMAN_TASK with justification -> success;
     approvals.jsonl shows `apr_0001 approved, resolved_by: rso_local`; the kernel's
     delegation-audit ledger entry carries BOTH principals
     (`subject_refs: ["actor:rso_local","gateway:gateway"]`,
     `effective_actor_id: rso_local`, `verb: approval_decide`).
  5. operator_local COMPLETE_HUMAN_TASK signoff_release (justification in the payload)
     -> success; durable `human_task_completed` lands.
- Regression (round-1 strength): the delegated-approval flow still works end-to-end —
  steps 1-5 above ARE that flow, over real HTTP with a real kernel.

### F4 — new_cursor race: CLOSED

- Code: every mutation path captures `before` via CursorForCase BEFORE the kernel call
  and answers via postMutationCursor (intents.go:408-422) which calls
  Relay.WaitForCaseAdvance (server/relay.go:151-160) — a strict-advance wait (`cursor >
  before`), not the old vacuous current-cursor read.
- Judged genuinely closed: the response can no longer present the PRE-mutation cursor as
  post-mutation unless the kernel frame is lost for >5s, in which case the advisory field
  understates honestly rather than lying (SSE and /api/world carry the truth).
- Checked-in live test re-run with -v: TestLiveSSERelayOfKernelFramesWithResume PASS
  (teeth/f4-sse-resume-critic2.log); the drain loop now SKIPS same-cursor snapshots
  (server_live_test.go:160-168) instead of breaking on the first one.
- Stale-cursor tooth re-proven by my own probe (runtime/f4-stale-*.json): EXECUTE_ITEM
  with an old cursor -> STALE_PROJECTION carrying current_cursor, and the cell's
  requests/ directory unchanged (8 records before and after; no record for the refused
  intent) — no kernel write.
- My manual drives additionally confirmed strictly-advancing new_cursor values on
  PROPOSE_CASE, ADD_DISCRETIONARY_WORK and EXECUTE_ITEM responses.

### F5 — plan_schema_error -> INVALID: CLOSED

- Mapping: mapKernelRefusal (intents.go:521-526) lists plan_schema_error with the
  malformed-shape group returning RefInvalid, with a comment naming the fix.
- Unit: TestKernelRefusalsMapOntoTypedKinds case "plan-schema" pins it; re-run standalone
  PASS (teeth/f5-mapping-unit-critic2.log).
- Live: my own probe — ADD_DISCRETIONARY_WORK anchored to a nonexistent stage over served
  HTTP -> kernel `plan_schema_error: unknown dependency no-such-stage`, surfaced as
  `refusal_kind: INVALID` (runtime/f5-schema-err-resp2.json). Cosmetic nit only: the
  class prefix doubles ("plan_schema_error: plan_schema_error: ...") because the kernel
  message already embeds the class — no behavioral impact.

### F6 — pool IdleTTL: CLOSED

- Mechanism read and verified (client.go): IdleTTL defaults to 8s (client.go:91-92,
  below the kernel's 10s request-line timeout with margin); acquire (client.go:238-281)
  reaps any pooled connection whose idle age is >= IdleTTL before it can be handed out
  and redials fresh; release stamps idleAt. A stale connection can never be returned to
  a caller.
- The OBSERVED_DEBT entry (RESOLVED 2026-09-25) matches this mechanism exactly.
- New unit test TestRequestAfterIdleGapUsesAFreshConnection (client_test.go:376-470) is a
  real mechanism test: fake server closes idle conns after 300ms, IdleTTL=100ms, asserts
  the post-idle mutation succeeds on a FRESH connection (conns==2) with each mutation
  sent exactly once. It ran green inside the unit and live sweeps.
- T05 live tests re-run x2 (teeth/f6-recovery-subscribe-x2-critic2.log):
  TestLiveKillMidCommitRecovery and TestLiveSubscriptionResumeAcrossRestart PASS in both
  rounds.

### F7 — kernel locator reconciliation: CLOSED

- Diff read (git show b5fd2c3 -- crates/sea-forge-server):
  (a) Ordering: in case_dispatch.rs submit's mint closure, `ids::case_id()` mints the
      id, then `mint_store.record_locator(request_id, &case_id)?` runs (fail-closed)
      BEFORE `CaseRunner::initialize_case` performs the case ledger's first write. The
      locator is bound before any case ledger write, as claimed.
  (b) Reconciliation: settle_interrupted_requests (correlation.rs) reconciles a record
      ONLY when method == "case.commit", it carries a locator, and
      case_ledger_landed(locator) is true — the ledger file exists AND contains a
      `"kind":` byte pattern (a real first event, not an empty file); such records settle
      completed with `{"case_id": locator, "recovered": true}`. Everything else (no
      locator, missing/empty ledger, other methods) stays interrupted. The locator is
      validated with valid_id_segment before joining a path.
  (c) The locator never erases: record_locator is first-write-wins and refuses to touch
      settled records; record_outcome now carries the prior locator forward, so it
      survives the terminal write. Unit-pinned by
      `the_locator_is_never_rewritten_and_survives_the_restart_settlement` and
      `a_pending_record_refuses_concurrent_retry_without_erasing_its_locator`.
  (d) RequestRecord schema regenerated additively: the committed
      workbench/packages/contracts/schema/RequestRecord.schema.json adds optional
      `locator` (not required; struct uses serde(default) +
      skip_serializing_if). See 5.1 for the TS-side follow-through.
  (e) Tests I ran myself:
      - `cargo test -p sea-forge-server --lib correlation` -> 13/13 PASS, including
        restart_reconciles_a_located_commit_whose_ledger_landed (completed + Replay
        verdict), restart_still_interrupts_a_located_commit_whose_ledger_never_landed
        (failed/interrupted), and the two locator-preservation tests.
      - TestLiveKillMidCommitRecovery x3 PASS (teeth/f7-recovery-x3-critic2.log) plus a
        fourth full-transcript round (teeth/f7-recovery-x4-full-transcript.log) that hit
        the exact F7 branch live: "commit completed before the kill; outcome recovered
        for case ...", record status=completed, durable truth = exactly 2 CaseCreated
        (warm-up + the killed-but-landed commit) — no failed record over a landed effect,
        no double commit. The kernel binary I used was rebuilt from HEAD before the runs.

## 3. Round-1 strengths re-checked (regressions: none found)

- Fixture exclusion: fresh production build (no tags) succeeds; `go tool nm` finds ZERO
  northstar/fixturestore/fixturerevision/servefixturestack symbols
  (gates/nm-fixture-probe-critic2.log).
- T01 contract conformance: all five golden/exhaustiveness tests PASS; the live
  projection golden (builder output over a real kernel byte-pinned against
  testdata/livesnapshot-sentry-chain.json) PASSes at HEAD.
- Delegated approval flow end-to-end: proven live by my own F3 drive (section F3, steps
  1-5), including both-principals audit.

## 4. Verdict rationale

Every round-1 blocking finding (F1, F2) and every minor one (F4, F5) is fixed and
re-proven against the real kernel by my own observation, not just by the builder's
tests. F3's cell/policy gap is fixed at the source (the policy now opens the approval the
L5 journey resolves) and the journey is live-proven end to end. F6's mechanism matches
its recorded resolution and its new unit test is a genuine one. F7 is a sound, minimal
kernel reconciliation: the locator ordering closes the failed-over-landed-effect window
deterministically, the reconciliation is conservative (only genuinely landed commits),
and it is pinned by deterministic unit tests plus a live kill-recovery run that exercised
the recovered branch. All required gates are green with true exits; the -p 1 posture is
sound. The plan's T06 proves-clause — "For each mapped intent, POST /api/intents produces
the expected kernel TraceKinds, followed by an SSE revision whose snapshot reflects
them" — now holds including ADD_DISCRETIONARY_WORK's accepted path, and the kernel-veri-
fied ?actor=&role= override works as advertised.

## 5. Non-blocking observations (coordinator-owned; nothing here blocks T06)

1. Uncommitted generated-zone follow-through: b5fd2c3 commits the regenerated
   RequestRecord.schema.json (additive locator), but the regenerated TS files
   (workbench/packages/contracts/generated/RequestRecord.ts and .validator.ts) sit
   UNCOMMITTED in the worktree (present, correct, and consistent with the committed
   schema). Per the "change generators, not generated zones" rule these should be
   committed with (or immediately after) the schema change. Left untouched by this
   critic per instructions.
2. mapKernelRefusal message cosmetics: for kernel classes whose message already embeds
   the class (e.g. plan_schema_error), the served message doubles the prefix. Harmless;
   dedupe if touched again.
3. Pre-existing coordinator fixture-tag flake: internal/coordinator's
   casework_fixture suite is order-dependent (once in ~10 rounds at HEAD; also flaked at
   pre-fix a1129f1 with a different test, so not introduced by b5fd2c3). Attributed and
   documented in critic2/gates/fixture-tag-flake-note.md; recommend an OBSERVED_DEBT
   entry by the coordinator.
4. Test-comment nit: TestLiveWorldPerspectiveVerifiesAgainstTheKernel calls its negative
   case "bound-but-not-allowlisted", but operator_c (in the harness cell) is neither
   bound nor allowlisted. The refusal path is the same allowlist check; the truly
   bound-but-not-allowlisted case was proven by this critic's own drive with a bound
   operator_b (section F1). Optional one-line comment fix.
5. Cosmetic stderr noise in TestLiveDiscretionaryAddProducesAnAcceptedKernelWrite (a
   relay "rebuild ... failed: EOF" line at cell teardown); the test's assertions all
   pass. No action needed.

## 6. Evidence index (this critic: .agents/evidence/casework-live-wiring/T06/critic2/)

- gates/: go-vet-critic2.log, go-test-race-critic2.log, go-test-live-p1-critic2.log,
  gofmt-critic2.log, cargo-three-crate-critic2.log, cargo-fmt-check-critic2.log,
  nm-fixture-probe-critic2.log, go-test-fixture-tag-critic2.log,
  fixture-tag-flake-note.md
- teeth/: f1-perspective-live-critic2.log, f2-discretionary-live-critic2.log,
  f4-sse-resume-critic2.log, f5-mapping-unit-critic2.log,
  f6-recovery-subscribe-x2-critic2.log, f7-recovery-x3-critic2.log,
  f7-recovery-x4-full-transcript.log, contract-goldens-critic2.log,
  golden-live-critic2.log
- runtime/: f1-drive.md (full manual-drive transcript), f1-*.json (perspective probes,
  preflight, propose), f2-*.json + f2-durable-verify.md (discretionary add + durable
  settlement), f3-*.json (L5 approval journey), f4-stale-*.json, f5-*.json
- Transient artifacts (cell, binaries) lived under /tmp/t06c-critic-cell and /tmp and
  were removed; nothing outside .agents/evidence/.../T06/critic2/ was created or
  modified in the repository. Not committed, per instructions.
