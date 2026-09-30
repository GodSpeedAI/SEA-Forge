# T07 independent runtime confirmation — 2026-09-30

## Decision

**Do not approve T07 yet.** Most session, authorization, replay, and mutation teeth passed against the isolated live binaries, but the gateway can report a `PROPOSE_CASE` `new_cursor` before the revision store has appended that cursor. An immediate authorized historical read of the returned cursor failed with `400 invalid: unknown or evicted kernel cursor ...`; the same cursor later appeared as the case trajectory base and became readable. This is a live correctness defect, not retention eviction. Root has assigned a fresh builder; no source files were changed here.

## Runtime posture and deviations

- Branch HEAD under review: `casework/live-wiring` / `ff7fe6f`. No production/source edits or compilation by this verifier.
- An earlier isolated cell had an extant task-owned kernel (PID 256835) holding its cell locks and bound to the staging `.server.sock.binding`; the published socket was absent. It was preserved. A separate cell `/tmp/t07-final.uUPtaO/cell-independent` was initialized.
- The first fresh-cell copy of `serve.json` failed closed before serving because `authority` was `required:true` without a credential indirection. The test config was corrected to match the checked-in `apps/godspeed-cognitive-ui/e2e/live-conformance.ts` SFWP authority capability (`required:false`); live authority readiness preflight remained mandatory. A third unmapped development-only user was added solely for the refusal tooth. Test config and policy remain under `/tmp`.
- User authorization was received for an actual loopback gateway launch after an auto-review rejection. The actual direct binary launch was reviewed and ran at `127.0.0.1:44179`; health reported `go:live:sfwp`, readiness returned 200. Vite at `127.0.0.1:4178` remained listening.
- Python's cookie jar does not send Secure cookies over loopback HTTP, so probes manually replayed the in-memory cookie header values without printing or saving them. This does not prove native browser Secure-cookie behavior; T08 browser evidence remains necessary.
- A transient readiness result was 503 at `2026-09-30T13:53:54Z` (3,566.5 ms) and again at `13:54:59Z` (3,016.6 ms), while health was 200. Readiness later returned 200 (`13:56:07Z`, 2,596.8 ms), and subsequent checks stayed ready. Kernel logs also contain recurring sanitized warnings `connection error: request line exceeded the 10s server timeout` (e.g. `14:14:04.476646Z`, `14:14:49.493028Z`); no request body or credential values were logged.

## Passing runtime assertions

- Health/readiness: 200; provenance `go:live:sfwp`.
- Anonymous `/api/world` and `/api/events`: 401, no snapshot leak. Separate operator and R-SO logins produced `operator_local/operator` and `rso_local/R-SO` session/current-world perspectives.
- Session cookie: Secure, HttpOnly, SameSite=Strict; CSRF cookie: Secure, not HttpOnly, SameSite=Strict.
- Exact trusted Origin received matching ACAO. Untrusted GET was served without ACAO; untrusted POST and missing-CSRF POST both returned 403 `csrf_refused`. Current case cursor did not move.
- The unmapped principal's world and events requests with an invalid cursor both returned 403 `authority_denied` before cursor lookup. Query `actor`/`role` overrides on world/events returned 400 `invalid` before historical lookup. A real cursor paired with a different case returned 400 `invalid`.
- Operator committed checked-in `e2e-sentry-chain@0.1.0` cases by preflight plus `PROPOSE_CASE`. A forged R-SO claim in the intent body was overwritten: result perspective remained operator. The successful intent ID appeared as its structured-log correlation ID.
- Historical/replayed/live SSE snapshots carried the corresponding session actor for operator and R-SO. Operator received an item-level `EXECUTE_ITEM` offer for ready work; R-SO did not. After task execution, `task_publish` became ready and was actionable only for the operator.
- On case `case_20260930T141504Z_dd74f9`, pre-mutation history responses at cursor `01M3SAPGD6W1CZYNPCPAZS58M3` were saved and then fetched after governed `task_prepare` execution. The operator and R-SO response bodies were byte-identical before/after (1,685 and 1,281 bytes); old `task_prepare=READY_TO_BEGIN` remained, while current became `COMPLETED`. Current `task_publish=READY_TO_BEGIN` had an item-level action for operator only.
- A newline-containing malformed intent was refused, with one escaped structured log record and no injected second line.
- Delegation was revoked only in the copied test cell by removing `operator_local` from `delegable_actors`, preserving `rso_local`. Before restart, the kernel PID/executable/cell lock descriptors were verified; only that task-owned kernel was gracefully stopped and restarted. Gateway readiness remained 200. Cached operator world and SSE both returned 403 `authority_denied`; still-delegated R-SO world and SSE returned 200.
- Production refusal: an isolated config with `serve.production=true`, a valid HTTPS origin, and existing `auth.mode=dev` exited 2 before binding. Exact error: `auth.mode dev is REFUSED in the production posture: the dev surface skips password verification and may carry a static bearer token; configure auth.mode local or oidc for production`. Port 44180 was not listening afterward.

## Failed expectations preserved; interpretation

- The first comprehensive script incorrectly required snapshot-level `available_actions` to match the state of one target item. It failed during an `IN_PROGRESS` frame and again when `task_prepare=COMPLETED` while `task_publish` was independently ready. These are harness expectation errors: the top-level action list aggregates actions across visible objects. Subsequent probes checked `visible_objects[id].actions` against that item's status and passed. Earlier failures were not overwritten.
- The mutation intent's `new_cursor` may refer to an intermediate revision; the relay can emit later terminal frames. In the case-B task_publish probe, the response cursor was `01M3SAMJ471DG4KV8JNGT5FE2A`, while the later current projection reached `01M3SAMJ4BCV4EM2V6TPNW7QGM`. Probes must await the target frame/status and must not assume response cursor equals the final current cursor.

## Accepted-cursor race evidence (blocking)

For case `case_20260930T141504Z_dd74f9`, `PROPOSE_CASE` returned cursor `01M3SAPGD6W1CZYNPCPAZS58M3`. An immediate `/api/world?case_id=...&cursor=<returned new_cursor>` returned HTTP 400 `invalid`, note `unknown or evicted kernel cursor 01M3SAPGD6W1CZYNPCPAZS58M3; refetch the live world`. A later request for the exact same cursor succeeded. The live `/api/trajectory?case_id=...` response showed HTTP 200, base equal to that cursor, head `01M3SAQG15NEJCPQ8VQCV9PVED`, and exactly six retained points; the returned cursor was the first retained `case.submitted` point. This excludes eviction pressure.

Source inspection explains the race: `internal/server/relay.go::accept` advances `caseCur` and notifies waiters before rebuilding facts and calling `store.Append`; `WaitForCaseAdvance` returns `caseCur`, and `internal/intents/intents.go::postMutationCursor` uses that result as `new_cursor`. The response can therefore outrun the retained-history append. Do not settle T07 until a fresh builder addresses the distinction between observed cursors (staleness) and retained/published cursors (mutation response), with deterministic blocked-source/append-failure coverage as root specified.

## Existing gates and evidence location

Previously recorded unchanged-source T07 gates remain in `.agents/evidence/casework-live-wiring/T07/resume-2026-09-29/final-independent/confirmation.md` (Go vet, global race, Argon count 50, four-crate 594/0); they were not rerun here because no source changed and this verifier held no compile token. This runtime record is under `/tmp` because repository `.agents/evidence` is read-only in this session. Supporting sanitized probe scripts are adjacent. The gateway, fresh cell, and logs are preserved; Vite was untouched.
