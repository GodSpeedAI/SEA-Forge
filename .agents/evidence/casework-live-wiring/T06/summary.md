# T06 evidence summary — Live projection builder and intent translation

Task: plan `.agents/plans/2026-09-23-casework-live-wiring-production.plan.yaml` T06
(branch `casework/live-wiring`, baseline 1513bb0). Proof level P2, builder-verified.

## What was built

### internal/projection (live)
- `builder.go` — the pure fold: `CaseFacts` (ports DTOs from case.list, case.get_overview,
  case.get_horizon, approval.list, run.list + perspective + kernel cursor) ->
  `contract.CognitiveWorldSnapshot`. No I/O, no globals. Disjoint execution/settlement standing
  rendered side by side (a completed-but-unsettled item is "Executed; settlement pending", never
  "done"). Sentry reasons in plain language composed from the real standing data only — the
  kernel's view verbs do not expose template sentry predicates (verified live; see "Sentry data
  note" below), so a pending item explains itself from what the standing factually says, and a
  dependent item names its open dependencies and their standings.
- `roles.go` — the projection's role-filter vocabulary (kernel role spellings): approver roles
  (R-SO/R-RM/R-LC/R-AG/R-DS) see APPROVE/REJECT; executor roles (operator/R-DEV/agent/R-AA) see
  EXECUTE_ITEM/COMPLETE_HUMAN_TASK; proposer and lifecycle roles for commit/discretionary and
  reopen/terminate; machine roles (service/system) get no consequential offers. The intent
  translator's UNAUTHORIZED_ROLE guard calls the SAME predicates, so offer and guard cannot drift.
- `store.go` (new) — the revision store keyed by the KERNEL event cursor (the events-ledger
  entry_ulid the bus hands out). Bounded retention (512, eviction answers the documented
  404/ErrUnknownCursor), monotonic-append enforcement, replay+live subscriptions.
- `live.go` — LiveSource: fetches facts through ports.CaseAuthorityPort and calls the pure
  builder; template/preflight DTO mapping; kernel-verified perspective checks (identity.get with
  the delegation block).
- Fixture files (`fixture*.go`, `fixturedata/`) moved behind the `casework_fixture` build tag
  with the old store/types renamed (FixtureStore/FixtureRevision). The production build (no tags)
  contains no Northstar fixture code; the dev/demo stack (`just casework-go-up`) builds with the
  tag and stays green.

### internal/intents (new)
- One canonical intent kind -> exactly one governed SFWP verb: PROPOSE_CASE->case.commit,
  ADD_DISCRETIONARY_WORK->case.add_item, EXECUTE_ITEM->item.execute,
  COMPLETE_HUMAN_TASK->human_task.complete, APPROVE/REJECT_HUMAN_TASK->approval.decide,
  REOPEN_CASE->case.reopen, TERMINATE_CASE->case.terminate, OPEN_ARTIFACT->artifact.get,
  ESCALATE_OR_OVERRIDE->honest UNAVAILABLE (no kernel verb routes it before T13).
- Effective actor: every mutation sends Governance{Actor: gateway principal,
  OnBehalfOf: intent actor} per T02/D-2; request_id = intent_id (the durable locator; the kernel
  correlates, dedups, and replays recorded outcomes for same id+payload).
- Typed refusals per the T01 envelope, mapped from the kernel's error classes through the new
  ports.ClassRefusal seam (no adapter import in the application layer).
- Staleness: mutating intents must carry client_cursor; the relay's per-case kernel-cursor view
  is compared and mismatches refuse STALE_PROJECTION with the fresh cursor BEFORE any kernel
  write. PROPOSE_CASE is exempt (no target case; the kernel's own preflight-digest precondition
  binds freshness). Untracked-but-existing case -> UNAVAILABLE (projection syncing); unknown case
  -> INVALID. Idempotent replay: same intent id + byte-identical body replays the recorded
  outcome in-process; payload drift -> INVALID.

### internal/server (live)
- `server.go` — spec-04 routes: GET /api/healthz (provenance go:live:sfwp), GET /api/world
  (live build at the per-case kernel cursor; ?cursor= serves true kernel history with documented
  404 for evicted/unknown cursors; kernel-verified ?actor=&role= perspective overrides), GET
  /api/templates + POST /api/templates/preflight, POST /api/intents (refusals are HTTP-200 typed
  outcomes).
- `relay.go` — kernel-frame relay: per-case kernel-cursor tracking (the staleness source), one
  rebuilt snapshot revision per frame appended to the store, broadcast to SSE subscribers.
  Found and fixed a real bug here during the live SSE test: the Run loop re-invoked
  feed.Events() inside the select, spawning a fresh pump goroutine per event and stranding
  frames in unread channels — the live test caught frames being lost.
- `feed.go` — sfwp.Subscription -> EventFeed adapter. Kernel frames are `case.submitted` and
  `case.trace.<snake_kind>` (D-3-followups: there are NO command-level frames on the bus, so no
  execution_progress events are fabricated; the snapshot revisions carry the standing).
- SSE: hello, resync_required when the requested position predates retention, replay of stored
  revisions after Last-Event-ID (header) or ?last= (both honoured), live revisions with
  id=kernel cursor, heartbeat comments.
- The old fixture server moved to `fixture_server.go` behind `casework_fixture` (tests + dev).

### Kernel-side facts verified live during this task (probes + tests, not assumed)
- The kernel verifies the commit precondition itself: a stale preflight digest returns a normal
  (non-error) frame with code "precondition_failed" / outcome "rejected_as_stale" and NO case is
  created. The adapter detects it; the gateway surfaces STALE_PROJECTION (the T01 golden pins
  exactly this semantic and message for template drift).
- Same request_id + same payload replay returns the RECORDED OUTCOME (durable idempotency);
  same id + different payload -> request_id_reused (no_side_effect). Proven in
  teeth/intents-teeth-live.log.
- on_behalf_of role-widening is refused by the kernel (identity_delegation_refused,
  no_side_effect) — the gateway's own UNAUTHORIZED_ROLE guard fires before that.
- The kernel's `case.get_horizon` does NOT expose template entry criteria (sentry predicates) —
  only execution/settlement standing, depends_on, run ids. Sentry reasons are therefore composed
  from standing data (documented decision, not a redesign trigger: the plan's T06 golden test
  wording — "the dependent item shows its sentry reason" — is met by the honest standing-derived
  reason, and naming the specific sentry source would require a NEW kernel view verb, which this
  task was forbidden to touch. Flagged for the operator: if CJ-journeys need sentry SOURCE
  names, that is a kernel extension request.)

## Deviations / notes
1. `run.list` was missing from the Go client surface (kernel verb exists since T04): added
   NewRunList + RunSummaryView + Authority.RunsList + ports.CaseAuthorityPort.RunsList (client-
   side extension only; kernel untouched), per the task's "extend the Go side" rule.
2. Config schema: additive optional `serve` section (gateway principal claim, policy ref,
   perspective) — Document.version stays "1". SEA_FORGE_SOCKET keeps the T05 precedence (an
   explicit endpoint in the file wins over the ambient env; the T05 test pinning this is intact).
3. justfile: `casework-go-up` build line now passes `-tags casework_fixture` (the dev fixture
   stack keeps working; the default build cannot select fixtures — plan guardrail) and the
   `casework-live-go-up` banner no longer claims FIXTURE-LABELED providers. Recipe names and
   semantics otherwise untouched (correction C-1 respected).
4. OPEN_ARTIFACT requires parameters.digest (artifact.get is digest-addressed); the kernel's
   view verbs do not attach artifacts to object ids, so a digest-less OPEN_ARTIFACT refuses
   INVALID with that explanation (honest; T08/T09 resolve artifacts by digest).
5. ESCALATE_OR_OVERRIDE refuses UNAVAILABLE (no kernel verb; T13 territory) — the golden's own
   refusal example.
6. Status files (CURRENT_STATUS.md/current_status.yml) intentionally left to the coordinator:
   go_gateway tasks may run in parallel and the handoff files are single-writer.

## Gates (logs under gates/, true exit codes captured)
- go vet ./... — EXIT=0
- go test -race -count=1 ./... — EXIT=0 (9 packages ok)
- go test -race -count=1 -tags live ./internal/... — EXIT=0 (includes the T05 sfwp live suite;
  one transient restart-timing flake in T05's TestLiveKillMidCommitRecovery observed once under
  machine load, passed on immediate retry and on the final full run)
- go test -count=1 -tags casework_fixture ./internal/... — EXIT=0 (dev fixture stack stays green)
- gofmt -l . — clean
- cargo test -p sea-forge-cli -p sea-forge-case-runner -p sea-forge-server — EXIT=0

## Teeth (transcripts under teeth/)
- Operator-role actor posts APPROVE_HUMAN_TASK -> UNAUTHORIZED_ROLE, zero kernel calls (proven
  against the kernel's durable correlation store: no requests/<intent_id>.json).
- Intent with an older cursor -> STALE_PROJECTION with the fresh cursor, no kernel write (same
  durable proof), after a real mutation advanced the case.
- PROPOSE_CASE with a digest that cannot match -> the KERNEL refuses (precondition_failed,
  rejected_as_stale, no case dir created); surfaced as STALE_PROJECTION; the control (real
  preflight digest) commits exactly one case.
- Idempotent replay: same intent twice -> recorded outcome verbatim, no duplicate kernel effect;
  across a "restarted" gateway (fresh handler) the kernel's own correlation store dedups and
  replays the recorded outcome (item_activated count stays 1).
- SSE relay: mutation over HTTP -> case.trace frames relayed as snapshot revisions with
  id=kernel cursor, snapshot reflecting the executed item; Last-Event-ID resume without gaps
  against the retained history.
- main.go serve-path smoke (main-serve-journey-smoke.log): real binary, real cell — preflight
  ("ready ... adapter=sfwp"), LIVE banner (provenance go:live:sfwp), preflight->PROPOSE_CASE->
  world showing task_prepare READY_TO_BEGIN with EXECUTE_ITEM offered and task_publish WAITING,
  SSE frame with the kernel cursor relayed live.

## Permanent tests added
- internal/projection: builder unit tests (sentry chain standing, role filtering, disjoint
  standings, approval inbox linkage, lifecycle offers, summary/attention, template/preflight
  mapping), store tests (monotonicity, bounded retention, replay), live golden
  (testdata/livesnapshot-sentry-chain.json, GOLDEN_UPDATE=1 affordance).
- internal/intents: full table tests (accepted path per kind, refusal classes incl. kernel-class
  mapping, idempotent replay, no-kernel-call teeth) + live teeth.
- internal/server: live routes, cursor-keyed history 404, SSE replay/resume/resync_required,
  relay cursor bookkeeping + dedup, intent envelope strictness; live SSE relay test.
