# T06 independent confirmation — Live projection builder and intent translation

- Verdict: **REJECT** (narrow, remediable; see F1-F6 — the plan's proves-clause is not met for
  one mapped intent, and one advertised live feature is broken against the real kernel; all
  gates are green and the core machinery is independently verified, so the re-verification
  surface after a fix is small)
- Verifier: independent critic (did not build T06; builder session = commit a1129f1)
- Date: 2026-09-25 (verification window 18:48-19:20 -0400)
- Plan: `.agents/plans/2026-09-23-casework-live-wiring-production.plan.yaml` T06 (proof level P2,
  confirmation: independent)
- Claim under test: "For each mapped intent, POST /api/intents produces the expected kernel
  TraceKinds, followed by an SSE revision whose snapshot reflects them."

## 1. What was independently verified (evidence-backed)

### 1.1 Gates re-run by this critic (true exits captured under critic/gates/)

| Gate | Result | Evidence |
|---|---|---|
| `go vet ./...` | EXIT=0 | gates/go-vet.log |
| `go test -race -count=1 ./...` | EXIT=0 (13 pkgs ok) | gates/go-test-race.log |
| `go test -race -count=1 -tags live ./internal/...` | EXIT=0 | gates/go-test-live-fixture.log |
| `go test -count=1 -tags casework_fixture ./internal/...` | EXIT=0 | gates/go-test-live-fixture.log |
| `gofmt -l .` | clean (empty output, exit 0) | inline |
| `cargo test -p sea-forge-cli -p sea-forge-case-runner -p sea-forge-server` | EXIT=0, 523 tests ok, 0 fail | gates/cargo-three-crate.log |
| `bun test src/ports` (T01 contract, TS side) | 22 pass, 0 fail | gates/bun-ports.log |

No new dependencies: `git show a1129f1 --stat` touches no `go.mod`/`go.sum` and no `crates/`
files (guardrail respected; see 4.4).

### 1.2 Fixture exclusion (plan guardrail "no production code path can load fixtures")

- `go build -o /tmp/t06-prod ./cmd/godspeed-casework` (NO tags) succeeds;
  `go tool nm /tmp/t06-prod | rg -i "northstar|fixturestore|fixturerevision|servefixturestack"`
  is EMPTY. All fixture files carry `//go:build casework_fixture`
  (internal/projection/fixture.go, fixture_store.go, internal/server/fixture_server.go +
  _test.go, cmd/godspeed-casework/main_fixture.go; the untagged stubs are
  main_fixture_off.go and internal/config/fixture_enabled.go with `fixtureBuildEnabled=false`,
  which makes `Validate` refuse adapter "fixture" in production).
- justfile: `casework-go-up` builds with `-tags casework_fixture`; the live recipe builds
  without tags. Fixture recipes' names/semantics untouched (correction C-1 respected).

### 1.3 Contract conformance

- `go test ./internal/contract/ -v` (count=1): TestGoldenRoundTrip,
  TestGoldenCoversEveryIntentKind, TestGoldenCoversEveryRefusalKind,
  TestStreamEventKindsExhaustive, TestKindListsAreWellFormed — all PASS (teeth/contract-test-critic.log).
- `go test -race -tags live -run TestLiveGoldenSentryChainSnapshot ./internal/projection/ -v`
  — PASS (teeth/golden-live-critic.log): the LIVE builder output over a real kernel is
  byte-pinned against testdata/livesnapshot-sentry-chain.json.
- Shape diff of the served snapshot vs the T01 golden
  `.agents/reports/interface-contracts/golden/world-snapshot.json`: field names and nesting
  agree (world_id, case_id, cursor, timestamp, perspective{actor_id,role}, summary{headline,
  phase,status_phrase,progress_percent}, visible_objects[{id,kind,name,status,badge,
  explanation,salience,parent_id,depends_on,actions[]}], available_actions, attention_focus
  {primary_object_id,salience_rank,narration}); action descriptors carry
  id/label/intent/variant/consequential/requires_justification. Differences are honest
  omissions, not drift: the live snapshot has no `display_name` (the kernel identity view
  carries none) and `available_actions` includes the case-level lifecycle offer (the
  spec-04 vocabulary has no "case" object kind; the T06 builder documents this).
- The 10 SCREAMING_SNAKE intent kinds map 1:1 onto the T01 consequential set; refusals use
  the typed `refusal_kind` envelope with the legacy error_code/error_message mirror.

### 1.4 Permanent teeth re-run (all PASS, -v, under critic/teeth/)

- intents live teeth, 5/5 PASS (teeth/intents-live-critic.log): accepted EXECUTE_ITEM with
  exactly one durable item_activated; idempotent replay verbatim; operator APPROVE_HUMAN_TASK
  -> UNAUTHORIZED_ROLE with NO kernel correlation record and no side effect; older cursor ->
  STALE_PROJECTION carrying the fresh cursor with no kernel write; kernel-refused stale
  digest (precondition_failed, no case dir created) with a real-digest control committing
  exactly one case; kernel request_id dedup across a restarted gateway.
- server live tests, 14/14 PASS incl. TestLiveSSERelayOfKernelFramesWithResume,
  TestLiveTemplateEndpoints, healthz provenance, per-case cursor world, history-at-cursor +
  documented 404, empty-world, Last-Event-ID header resume, resync_required, relay cursor
  bookkeeping and monotonic dedup (teeth/sse-live-critic.log).

### 1.5 This critic's own runtime attack (real cell, real kernel, PRODUCTION gateway binary
over real HTTP; raw captures under critic/runtime/)

Stack: sea-forge-server on a temp cell seeded exactly like the checked-in E2E cell (T03
templates, E2E policy, server.yaml binding the invoking uid to the gateway principal with
operator_local/operator and rso_local/R-SO delegable), plus /tmp/t06-prod (no build tags)
serving http://127.0.0.1:4179 with provenance `go:live:sfwp`.

- GET /api/templates -> both E2E templates with typed parameters. POST preflight -> passed
  with digest (runtime/templates.txt, preflight.txt).
- PROPOSE_CASE (actor operator_local, echoed digest) -> success, case created; kernel
  correlation record written (requests/critic-propose-3.json).
- GET /api/world -> task_prepare READY_TO_BEGIN offering EXECUTE_ITEM; task_publish WAITING
  with explanation "No work has been recorded for this item yet: its entry sentries have not
  fired."; operator perspective; summary "1 item(s) ready to execute."
- EXECUTE_ITEM over HTTP with the current cursor -> success; the case's DURABLE
  case-events.jsonl gained item_enabled, item_activated, settlement_recorded, item_completed,
  item_enabled (downstream), i.e. the expected TraceKinds.
- SSE: a concurrent `curl -N /api/events?last=<cursor>` captured 5 snapshot revisions with
  `id:` = kernel event cursor; the FIRST post-mutation snapshot shows task_prepare COMPLETED
  ("Settled accepted") and task_publish READY_TO_BEGIN — the downstream sentry unlock relayed
  through SSE without reload (runtime/sse-capture.txt). This is the claim's second half,
  demonstrated.
- Exact replay of the same intent id + body -> recorded outcome verbatim, no duplicate kernel
  effect. Fresh intent id carrying the OLD cursor -> STALE_PROJECTION with `current_cursor`
  set; NO kernel correlation record (no write) (runtime/replay.txt, stale.txt).
- GET /api/world?cursor=<old stored> -> the true historical snapshot (task_prepare still
  READY_TO_BEGIN at the pre-execution cursor). Unknown cursor -> HTTP 404 with the documented
  note. SSE resume via the Last-Event-ID header -> hello + replay strictly newer, no gaps.
- Delegated flow (user B approves user A's work): after changing ONLY my temp cell's policy
  write_file verdict allow -> escalate (see F3), EXECUTE_ITEM as operator_local opened a real
  approval (apr_0001, approvals.jsonl, status pending); APPROVE_HUMAN_TASK as rso_local (R-SO)
  through the gateway with mandatory justification -> success; approvals.jsonl decision record
  `resolved_by: rso_local` with the note; the kernel's delegation-audit ledger carries BOTH
  principals for every delegated mutation (subject_refs ["actor:rso_local","gateway:gateway"],
  payload with effective_actor_id/effective_role/gateway_actor_id/gateway_uid/request_id/verb).
- A-approves-A: (a) through the gateway, operator_local's APPROVE_HUMAN_TASK -> UNAUTHORIZED_ROLE,
  approvals.jsonl unchanged, zero kernel calls (no correlation record); (b) bypassing the
  gateway's projection guard with a raw kernel probe (approval_decide, on_behalf_of the
  submitter) -> `"error_class":"separation_of_duty", "no_side_effect":true`, approvals.jsonl
  unchanged — and the gateway maps that class to SOD_VIOLATION (unit-pinned). The SoD
  backstop remains intact behind the gateway.
- PROPOSE_CASE with a mismatched digest over HTTP -> STALE_PROJECTION, cases dir unchanged.
- TERMINATE_CASE -> durable case_terminated; REOPEN_CASE -> durable case_reopened.
- OPEN_ARTIFACT without digest -> INVALID (honest message); with a real content digest ->
  success (artifact fetched through the governed path).
- COMPLETE_HUMAN_TASK with a wrong-shaped `result` (string instead of object) -> INVALID via
  the strict decoder; correct shape -> success with durable human_task_completed (+case_closed).
- ESCALATE_OR_OVERRIDE -> UNAVAILABLE (honest, pre-T13).

## 2. Findings (basis for the REJECT)

- **F1 (blocking, T06 code): the kernel-verified `?actor=&role=` perspective override is
  broken against the real kernel.** `projection.LiveSource.VerifyPerspective`
  (internal/projection/live.go:244-260) sends `ports.Governance{OnBehalfOf: ...}` with an
  EMPTY `Actor`; the Go client's `Governance.apply` always writes an `actor` block, the
  kernel parses the empty claim as absent, and `resolve_delegated`
  (crates/sea-forge-server/src/identity.rs:522 `claim.ok_or(IdentityRefusal::Missing)`)
  refuses with `Missing`. Observed live: every `/api/world?actor=...&role=...` returns 403
  with the kernel's message "this verb causes a side effect and requires an `actor` block..."
  for BOTH operator and R-SO actors. The builder's T06 summary advertises "kernel-verified
  ?actor=&role= perspective overrides"; that feature does not work. Test coverage
  (TestWorldExplicitPerspectiveIsVerified) exercises only a FAKE verifier (the 403 plumbing),
  so this was never live-proven. Fix is small: send the gateway's own claim as `Actor`
  (mirroring intents.execute, which DOES work — proven by every delegated mutation above).
- **F2 (blocking for the proves-clause): ADD_DISCRETIONARY_WORK cannot produce an accepted
  kernel write as wired.** The claim says "For each mapped intent, POST /api/intents produces
  the expected kernel TraceKinds". Observed live: without `parameters.stage_id` -> INVALID
  (intents.go:201 requires it unconditionally); with any stage id on the checked-in E2E
  templates -> kernel `plan_schema_error` ("sandboxed task item-disc-... must declare at
  least one operation" for kind=work_item/sandboxed_task; "unknown parent stage" otherwise),
  because the gateway's proposal (itemWire, adapters/sfwp/authority.go) carries NO operations.
  The kernel itself accepts an operation-less, stage-less milestone: my raw `case_add_item`
  probe returned `{"ok":true,"proposed_by":"operator_local"}` and the case ledger gained
  plan_mutated + item_enabled — so the kernel verb is healthy (T04), the GATEWAY translation
  cannot reach the accepted path. Root attribution is partly T01: the contract marks
  `stage_id` required (typescript/types.ts:217) and the golden's ADD_DISCRETIONARY_WORK
  success example is not reproducible against the real kernel as wired. This is a
  correction_protocol fact (contract vs kernel) that the T06 builder should have recorded in
  the decision log; it is not recorded, and the accepted path for this intent is neither
  live-proven nor reachable. The intent table's unit "accepted" test runs against a FAKE
  authority, which is why the suite stayed green.
- **F3 (coverage gap, cross-task): no approval can arise through the served surface on the
  checked-in E2E cell.** The signoff-gate template's human-task completion settles the case
  directly (human_task_completed -> case_closed, no approvals.jsonl) — approvals open only on
  an episode settled under a policy verdict `escalate` (T04 semantics; the E2E policy has
  none). Consequently APPROVE/REJECT_HUMAN_TASK had no live trigger on the shipped cell: the
  builder's live teeth never exercised approval.decide against the real kernel, and my own
  live proof of the delegated approve path required changing MY temp cell's policy verdict
  (allow -> escalate). The plan's L5 journey ("the operator is denied and a second user
  (R-SO) approves, approvals.jsonl shows both principals") needs the E2E cell/template/policy
  to actually open an approval — a T03/T04-scope gap that T10 will hit; recording it here so
  it is not discovered mid-ladder.
- **F4 (minor, T06 code): IntentResponse.new_cursor is race-prone.** intents.execute waits
  via `WaitForCaseAdvance(ctx, caseID, "")`; relay.go returns the CURRENT observed cursor
  when `before == ""` and the case was already observed, so for existing cases the wait is
  vacuous. Observed: EXECUTE_ITEM's response returned the PRE-mutation cursor while the SSE
  revisions arrived right after. Advisory field only (SSE and /api/world carry the truth),
  but TestLiveAcceptedExecuteAndIdempotentReplay's "must answer with the post-mutation kernel
  cursor" assertion passes by timing. Fix: pass the pre-call cursor as `before`.
- **F5 (minor, T06 code): kernel class `plan_schema_error` maps to AUTHORITY_DENIED.**
  mapKernelRefusal's switch has no case for it, so it falls to the default; it is a malformed
  proposal and belongs with INVALID (T01 semantics: "malformed shape ... kernel input_error").
  Observed in the F2 probes.
- **F6 (non-blocking, T05 scope, surfaced by T06's production wiring): the SFWP client pool
  has no idle-connection hygiene.** The kernel closes connections whose next request line
  exceeds its 10s line timeout (repeating "request line exceeded the 10s server timeout"
  warnings in server.log). The first request after >10s idle then fails: for mutations the
  client honestly refuses UNAVAILABLE ("never crossed admission; retry only with a NEW
  request id", no kernel record, no side effect — controlled reproduction: sleep 12s ->
  fail 0.4s -> immediate retry succeeds); for reads the single retry can also hit a stale
  pooled conn and surface as broken-pipe authority_denied. Failure CONTAINMENT is correct
  (no duplicates, no fabricated success, typed refusals); AVAILABILITY is not: any human
  pause >10s between steps breaks the next served request. T05 owns the fix (idle TTL below
  10s, a health ping on acquire, or a warm-up on idle); recording it here because T06's
  served experience inherits it and the builder's summary does not mention it.

## 3. Judgment calls requested by the coordinator

- **(a) Composed sentry reasons — honest, with a caveat.** The kernel's view verbs carry no
  sentry predicates (HorizonItemView: plan_item_id/name/item_kind/execution/settlement/
  parent_stage/depends_on/last_event_at/run_ids — verified against frame.go and live DTOs),
  and T06 was forbidden to add a server view verb. The builder composes reasons from standing
  data only, and the live output supports the honesty claim: a dependent-but-unblocked item
  says "No work has been recorded for this item yet: its entry sentries have not fired." —
  it does NOT invent a sentry name or claim knowledge it lacks; an item with open deps names
  them with standings ("name (execution, settlement x)" via openDependencies); an
  approval-gated item names the approval id. This satisfies the plan's golden-test wording
  ("the dependent item shows its sentry reason") without fabrication. Caveat for the operator
  (the builder flagged it too): naming the sentry SOURCE (e.g. "unlocks when task_prepare's
  settlement is accepted") would need a kernel view extension — if CJ journeys need it, that
  is a T13/kernel request, not a T06 defect.
- **(b) The relay pump bug — no remaining stranding path.** relay.go captures
  `events := r.feed.Events()` ONCE before the select loop (Run, lines 81-96), with the
  rationale in the comment; the previous per-iteration re-invocation (each call spawning a
  fresh pump goroutine and stranding frames in unread channels) is gone. `sfwpFeed.Events()`
  is itself a fresh-channel-per-call footgun, but in all wirings (main.go serveLive,
  livestack.AssembleStack, tests) Run is started exactly once per feed; the unit fakes return
  a stable channel. Store.Append broadcasts with per-subscriber buffers and evicts stalled
  subscribers (closed channel -> client reconnects with Last-Event-ID) rather than stalling;
  the relay still advances cursors when a rebuild fails, so staleness guards stay correct
  during gaps. I could not construct a remaining frame-stranding path under the current
  wiring.
- **(c) PROPOSE_CASE skipping the cursor staleness check — sound.** Its target case does not
  exist, so no per-case cursor can be stale against; the freshness bound is the kernel's own
  preflight-digest precondition on the TEMPLATE record, which I verified live in both
  directions (fresh digest commits; mismatched digest -> precondition_failed, no case,
  surfaced as STALE_PROJECTION per the T01 golden). Two concurrent commits from one template
  are two legal cases, not a staleness miss. The gateway additionally refuses known-case
  mutations whose cursor it has not yet observed via a real case.list existence check
  (UNAVAILABLE while syncing / INVALID for unknown case) rather than guessing.
- **(d) run.list Go-side extension — legitimate.** The kernel verb predates T06
  (crates/sea-forge-server/src/sfwp/run_views.rs and Request::RunList exist well before
  a1129f1; the builder says "since T04", my git archaeology says earlier — either way,
  pre-existing). a1129f1 touches NO crates/ files and no go.mod/go.sum: NewRunList,
  RunSummaryView, Authority.RunsList and ports.CaseAuthorityPort.RunsList are pure Go-side
  additions inside the task's surface.

## 4. Deviations (builder-flagged) — judged

1. **Composed sentry reasons** — accepted (judgment 3a); operator flag noted.
2. **OPEN_ARTIFACT requires parameters.digest** — accepted: artifact.get is content-addressed;
   the kernel attaches artifacts to runs, not object ids; the refusal message says exactly
   that, and the accepted path works live. T08/T09 resolve by digest as planned.
3. **justfile `casework-go-up` builds with `-tags casework_fixture`** — accepted and correct:
   without it the dev fixture stack would be impossible in a world where the default build
   cannot select fixtures (the guardrail working as intended). Fixture recipe names and
   semantics otherwise untouched per correction C-1; the live banner no longer lies about
   FIXTURE-LABELED providers.
4. **run.list Go-side extension** — accepted (judgment 3d).
5. **Status files left to the coordinator** — noted: a1129f1 touches neither
   .agents/CURRENT_STATUS.md nor current_status.yml (single-writer discipline while
   go_gateway tasks run in parallel). Acceptable, but the handoff must land before T06 can be
   marked settled.
6. **NEW (builder did not flag): the F2 contract-vs-kernel mismatch and F6 idle-pool defect
   are unrecorded** — per the plan's correction_protocol and the decision-log practice
   (D-2-impl-notes/D-3-followups), both belong in the decision log; F2 also invalidates the
   ADD_DISCRETIONARY_WORK golden success example as a live-reproducible shape.

## 5. What REJECT does and does not ask for

All gates are green and re-verified; the projection fold, revision store, SSE relay, role
guard, staleness guard, idempotency, template endpoints, delegated identity (T02) wiring,
fixture exclusion and 8 of 10 intent kinds (plus ESCALATE_OR_OVERRIDE's honest refusal) are
independently proven against the real kernel. The REJECT rests on:

- F1 (perspective verification broken against the real kernel; fix in live.go),
- F2 (ADD_DISCRETIONARY_WORK accepted path unreachable as wired; gateway fix plus a recorded
  contract correction; re-prove with a live accepted add producing plan_mutated),
- F4/F5 (small correctness fixes in intents/relay),
- F3 (record the approval-journey cell gap; surface it to T03/T04/T10 owners),
- F6 (record; T05-owned idle-hygiene fix; not a T06 code change).

Per the S2-1 precedent: a REJECT is never self-reconcilable by the builder; re-verification
must be an independent re-run of the affected proofs (live: perspective override, accepted
ADD_DISCRETIONARY_WORK, new_cursor, plan_schema_error mapping) plus the standing gates.

## 6. Evidence index (this critic)

- gates/: go-vet.log, go-test-race.log, go-test-live-fixture.log, cargo-three-crate.log,
  bun-ports.log
- teeth/: intents-live-critic.log, sse-live-critic.log, golden-live-critic.log,
  contract-test-critic.log
- runtime/: templates.txt, preflight.txt, propose.txt, propose-retry.txt, execute.txt,
  replay.txt, stale.txt, signoff-preflight.txt, sse-capture.txt, ledger-sod-case.jsonl,
  runtime-transcript-2.md (phases 2-4 incl. delegated flow, SoD probes, idle reproduction)
- Transient runtime artifacts (cell, binaries, config) live under /tmp/t06-critic-cell and
  /tmp/t06-prod; nothing outside .agents/evidence/.../T06/critic/ was created or modified in
  the repository.
