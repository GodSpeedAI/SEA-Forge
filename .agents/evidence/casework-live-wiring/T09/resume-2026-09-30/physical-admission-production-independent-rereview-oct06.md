# Physical run.get admission — independent production source rereview

Date: 2026-10-06  
Verdict: **APPROVE frozen production source for assigned runtime verification only.**  
Scope: complete source-only rereview against both original production/wiring assignments, the accepted proposal and fixture trail, and `physical-admission-root-source-adjudication-oct06.md`. This is not runtime approval, settlement, or T09 completion.

## Frozen source identities

| Path | SHA-256 |
|---|---|
| `apps/godspeed-casework-go/internal/adapters/sfwp/client.go` | `ad3d599224527beda603a320b1b86faa82b80c75480d914cd805fa1e4de3a857` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_get_admission.go` | `af040128d0d0288a2495d63bdb959a14edbf4ca8d5a2d8e29954193bb4e5fc6f` |
| `apps/godspeed-casework-go/cmd/godspeed-casework/main.go` | `dcc92dd73a951d3053ad907d66bda6caaf03f39b22fa33029ac81598349ddeed` |
| frozen limiter fixture | `dae0406d503e16546b24c507b02db63489dd8b06cd381b34fd045326d1983d58` |
| frozen client fixture | `becd4266dca4a28e7641f5eaab1dafe5dc932e04723895cf5e795e7671524103` |

Both fixtures remain byte-identical to the accepted attempt-4 fixture identities. Main and both fixtures were verified unchanged from the previous final freeze. Read-only `gofmt -d` on `client.go` and `run_get_admission.go` exited 0 and emitted no output.

## Prior rejection and compatibility repair

The previous source rejection remains immutable. Its formatting finding is resolved by the aligned `runGetPermit` fields (`run_get_admission.go:193-199`), and its compatibility analysis is superseded by the root adjudication and this rereview.

The prior implementation checked `ctx.Err()` before writes even when `permit == nil`. That changed the pre-write cancellation/error path for Ask, mutations, and other unguarded operations. The adjudication correctly identified this as a material deviation from the original requirement to preserve other verbs and their existing behavior. In the final frozen source the explicit check is nested inside `if permit != nil` (`client.go:231-236`). Thus guarded physical `run_get` attempts honor the required observed-prewrite-cancellation/no-write/no-cooldown rule; legacy nil-permit clients, Ask, mutations, and other verbs retain the original callback/write path. No extra admission or wire validation was added. `conn.call` retains its signature and delegates to `callWithPermit(..., nil)` (`client.go:194-199`).

## Assignment comparison

No unresolved material deviation from the original assignments or binding decisions remains:

- **Bounded limiter.** Production uses exactly two slot records and a fixed one-second cooldown (`run_get_admission.go:11,27-35,50-56`). Timing seams are package-private (`:43-48,58-73`); there is no map, queue, or per-run timer registry.
- **Selection and no-spin waits.** Context is checked before and after locking (`:80-90`). Matching run identity is handled before absent-run selection, so a busy/cooling match cannot fall through (`:92-115`). An absent run selects the first empty slot, otherwise the eligible idle slot with earliest deadline and stable slot-index tie behavior (`:118-154`). Matched busy waits use change/context; matched cooling waits its own future deadline/change/context; absent requests wait on the earliest future idle deadline/change/context; all-busy waits use change/context. The wait path creates no expired-deadline timer and stops each created timer (`:142-154,167-186`). No default spin loop is present.
- **Empty ID and slot retention.** The empty-ID sentinel is considered empty only when the slot has no busy owner and no deadline (`:158-165`). A busy or cooling empty-ID record is consequently matched and retained. No unexpired idle or busy record can be evicted (`:118-139`). Request construction stores the supplied run ID as a string (`frame.go:308-313`); no production empty-ID bypass was found.
- **Write boundary.** For a guarded call, the context check occurs after `SetWriteDeadline` and before `WriteAttemptStarted` or payload write (`client.go:223-249`). Finish is deferred around payload and LF attempts and therefore executes on either attempted-write return path, before `SetReadDeadline` or response reading (`:237-269`). The limiter records finish-time plus cooldown and signals waiters under its mutex (`run_get_admission.go:201-219`). A prewrite cancellation has no start/finish event.
- **Callback, pool, and permit ownership.** The existing `conn.call` signature is preserved. Its cancellation callback is stopped or joined before it returns, retaining connection locking and retirement behavior (`client.go:198-222`). A physical attempt registers permit release first and connection cleanup second, so Go LIFO runs discard/release before permit release; response decode occurs before helper return (`client.go:489-530`). The connection helper's callback join therefore precedes pool cleanup, which precedes permit release. No retry starts until the prior helper has returned.
- **Finish fallback boundary.** `Release` timestamps a started-but-unfinished permit (`run_get_admission.go:221-235`). In the assigned client path, the single `callWithPermit` write closure defers `WriteAttemptFinished` before returning from each attempted write, and helper release is deferred outside and after that call. Thus this fallback does not timestamp an ongoing write on the correct client path. This inspection establishes that path ordering; it is not a proof against arbitrary external misuse of the package interface or a panic that violates the synchronous call lifecycle.
- **Retry and error phases.** Admission precedes connection acquisition inside the timeout context; only `run_get` requests acquire permits (`client.go:441-446,489-521`). Admission, acquire, call, and decode have distinct phases (`:479-487,521-530`). `roundTrip` recovers correlated mutations and retries transport-safe reads only for call-phase failures; a retry connection-acquisition failure preserves the first call error; a queue/admission failure is returned without resend (`:446-477`). The existing bounded server-busy retry remains in `Do` and each physical attempt re-enters the helper (`:417-439`). Ask no-resend, mutation recovery, deadline normalization, refusal classification, request encoding, and response cap are unchanged by the reviewed paths. Successful transport with response decode failure returns the healthy connection through `release` (`:526-529,503-514`).
- **Complete production wiring.** Exactly one production `sfwp.New` site exists in the indexed package inventory; it receives one limiter constructed immediately before the live-client loop, and that same owner is supplied to every live capability client, including those not selected for serving (`main.go:86-103`). The only other `sfwp.New` site is test assembly (`internal/livetest/harness.go:245-261`). Both production run_get paths route through `Client.Do` and the shared hook: trace retrieval (`run_trace.go:72`) and artifact ownership provenance (`authority.go:758`). Other verbs skip admission at the `req.verb == "run_get"` guard (`client.go:489-498`).

The change set matches the scoped assignments: limiter and client runtime integration plus one shared CLI owner; no manager, SSE/UI, public wire contract, kernel, dependency, or test-file change is present. The builder repair changed only the permit-gated prewrite context check and gofmt field alignment, as required by the fresh repair assignment.

## Evidence boundary and next gate

Graft retrieval covered the client and limiter APIs, all `sfwp.New` sites, and all `NewRunGet` uses; anchors were checked against the complete client and limiter sources and affected wiring locations. Graft refreshed two changed files during retrieval; no explicit `graft build` was run. Only the read-only gofmt check ran in this rereview. No compiler, test, scanner, or runtime command ran. Root may now grant one sequential compiler owner for the bounded focused race tests, full SFWP race, full module race, and `just casework-go-check`, with the assigned preflight/capture discipline. Source approval alone does not establish no-spin runtime behavior, pool-at-release assertions, or any T09/live/settlement claim.
