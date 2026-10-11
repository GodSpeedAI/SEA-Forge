# T06 critic runtime attack — phases 2-4 (driven over the REAL served surface)

Stack: real sea-forge-server (target/debug) on temp cell /tmp/t06-critic-cell
(server.yaml: invoking uid bound to gateway principal "gateway"/service; delegable
actors operator_local/operator and rso_local/R-SO — exactly the checked-in E2E cell
posture) + the PRODUCTION gateway binary (go build -o /tmp/t06-prod ./cmd/godspeed-casework,
NO build tags) serving http://127.0.0.1:4179 with provenance go:live:sfwp.
All POSTs below hit the real HTTP surface; kernel truth read off the cell's durable files.

## Idle-pool defect (controlled reproduction, T05-client scope)
- sleep 12s (server closes connections whose next request line exceeds its 10s line timeout;
  server.log shows repeating "request line exceeded the 10s server timeout" warnings).
- First mutation after idle: UNAVAILABLE "the authority has no record of request critic-idle-1
  (it never crossed admission); it is safe to retry only with a NEW request id" (0.4s, honest,
  no kernel record in requests/, no case created).
- Immediate second mutation: success (fresh connection).
- Reads fail too when the whole pool is stale: GET /api/world?actor=...&role=... once returned
  {"kind":"authority_denied","note":"... write: broken pipe"}.
- Root cause: internal/adapters/sfwp/client.go pool has no idle TTL/health check; server's
  10s line timeout closes idle conns; mutation discipline (never re-send, resolve via
  request.get_status) then reports "never crossed admission". Failure containment is correct;
  availability is not (T05 scope, inherited by T06 serveLive).

## Delegated approval flow (signoff-gate + escalating policy)
- The checked-in E2E policy cannot open any approval through the served surface: the
  signoff-gate human task's completion closed the case directly (human_task_completed ->
  case_closed; no approvals.jsonl). Approvals open only on an episode settled under a policy
  verdict "escalate" (T04 semantics). On MY cell only, the write_file rule verdict was changed
  allow -> escalate; then:
  - PROPOSE_CASE e2e-sentry-chain as operator_local -> case_20260925T230647Z_a49301.
  - EXECUTE_ITEM task_prepare as operator_local -> success; approvals.jsonl gained
    apr_0001 status=pending (run_20260925T230656Z_f551c2, plan_item task_prepare);
    world summary "The case is parked until the pending approval is resolved."
  - APPROVE_HUMAN_TASK as rso_local (R-SO) through the gateway with justification ->
    success. approvals.jsonl decision record: resolved_by "rso_local", note = justification.
  - Delegation-audit ledger (ledgers/delegation-audit/entries.jsonl) carries BOTH principals
    for each delegated mutation, e.g.:
    record_kind=delegated_request subject_refs=["actor:rso_local","gateway:gateway"]
    payload {"effective_actor_id":"rso_local","effective_role":"R-SO","gateway_actor_id":
    "gateway","gateway_uid":1000,"request_id":"critic-sod-approve-byB","verb":"approval_decide"}
    and the matching entry for operator_local/item_execute.

## A-approves-A refusal paths
- Gateway guard: operator_local posts APPROVE_HUMAN_TASK -> UNAUTHORIZED_ROLE ("the action
  list for role operator does not offer APPROVE_HUMAN_TASK on this projection; the gateway
  will not route it to the kernel"); approvals.jsonl line count unchanged (3 -> 3); NO kernel
  correlation record for the refused intent (zero kernel calls).
- Kernel SoD backstop (raw NDJSON probe on the kernel socket, on_behalf_of operator_local on
  an approval whose submitter is operator_local):
  {"error":"actor `operator_local` requested the work approval `apr_0001` gates and cannot
  resolve it; approval requires a different actor","error_class":"separation_of_duty",
  "no_side_effect":true} — approvals.jsonl unchanged. The gateway's mapKernelRefusal maps
  separation_of_duty -> SOD_VIOLATION (pinned by internal/intents unit test line ~448).

## Other intents driven live
- PROPOSE_CASE mismatched digest -> STALE_PROJECTION ("The kernel rejected this commit as
  stale..."), cases dir count unchanged (5 -> 5).
- TERMINATE_CASE -> success; durable case_terminated. REOPEN_CASE -> success; durable
  case_reopened.
- OPEN_ARTIFACT without digest -> INVALID with the honest explanation; with a real content
  digest (from the case's evidence.jsonl artifact entry) -> success:true (artifact fetched
  through the governed path).
- ESCALATE_OR_OVERRIDE -> UNAVAILABLE ("No escalation target is configured in this
  deployment; the request cannot be routed.").
- COMPLETE_HUMAN_TASK payload strictness: a string "result" is refused INVALID (contract
  wants map[string]any); correct shape accepted (durable human_task_completed, case_closed).
- ADD_DISCRETIONARY_WORK: CANNOT succeed as wired. Without stage_id -> INVALID (gateway
  requires it). With any stage on the E2E templates -> kernel plan_schema_error
  ("sandboxed task item-disc-... must declare at least one operation" for kind=work_item;
  "unknown parent stage" otherwise); the kernel class plan_schema_error surfaces as
  AUTHORITY_DENIED (gateway default), arguably INVALID. Direct kernel probe (case_add_item
  with an operation-less, stage-less milestone) SUCCEEDS: {"ok":true,"proposed_by":
  "operator_local"} and the case ledger gains plan_mutated + item_enabled — so the kernel
  verb is fine; the gateway's translation cannot reproduce the accepted path on the checked-in
  E2E cell (T01 contract requires stage_id: types.ts:217; gateway proposals carry no
  operations).

## SSE resume + history
- GET /api/events with header "Last-Event-ID: <cursor>" -> hello then replay of the stored
  revision(s) strictly newer, id=kernel cursor.
- GET /api/world?cursor=<old stored cursor> -> the true historical snapshot (task_prepare
  READY_TO_BEGIN at the pre-execution cursor).
- GET /api/world?cursor=<unknown> -> HTTP 404 {"kind":"invalid","note":"unknown or evicted
  kernel cursor ...; refetch the live world"}.

## Response new_cursor race wart
- EXECUTE_ITEM's response returned the PRE-mutation cursor as new_cursor
  ("critic-exec-1" -> new_cursor equal to the client_cursor it carried) while the kernel
  frames arrived right after (the SSE revisions show the post-mutation standings). Cause:
  intents.execute calls CursorForCase/WaitForCaseAdvance(ctx, caseID, "") and relay.go's
  WaitForCaseAdvance returns the CURRENT observed cursor when before=="" and the case was
  already observed — the wait is vacuous for existing cases. Advisory field only; the
  permanent live test asserts non-empty and passes by timing.
