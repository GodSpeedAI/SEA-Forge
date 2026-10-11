# Physical run.get admission — independent production source review

Date: 2026-10-06  
Verdict: **REJECT source review pending formatting repair.**  
Scope: source-only review against both original physical-admission assignments and all binding proposal/adjudication supplements. No compilation, test, scanner, Git, or index-build command was run.

## Frozen source identities

The reviewed final source identities match the root's final freeze notice:

| Path | SHA-256 |
|---|---|
| `apps/godspeed-casework-go/internal/adapters/sfwp/client.go` | `fc38385e5077c48ba8e678f668ae87084c121b86f6057fffb13ab73a874a57d9` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_get_admission.go` | `e96cbaf12e43b5f8dcc1d5f20c073d6f8468b588c35a429d72e90f41b887ddc8` |
| `apps/godspeed-casework-go/cmd/godspeed-casework/main.go` | `dcc92dd73a951d3053ad907d66bda6caaf03f39b22fa33029ac81598349ddeed` |
| frozen limiter fixture | `dae0406d503e16546b24c507b02db63489dd8b06cd381b34fd045326d1983d58` |
| frozen client fixture | `becd4266dca4a28e7641f5eaab1dafe5dc932e04723895cf5e795e7671524103` |

The two fixture identities are unchanged from the accepted attempt-4 RED record. The limiter hash is the final frozen identity, superseding the intermediate `262919...` hash observed before the final freeze.

## Blocking finding

### Limiter source is not gofmt-clean

S4 · confirmed · introduced  
**Location** `apps/godspeed-casework-go/internal/adapters/sfwp/run_get_admission.go:193-199`  
**Evidence** Root ran read-only `gofmt -d` on the frozen limiter and observed exit 1 with a field-alignment-only diff in `runGetPermit`. The source currently has uneven spacing across `limiter`, `slot`, `started`, `finished`, and `released`.  
**Consequence** The source fails the repository's required Go formatting gate, so this source review cannot approve the frozen production result.  
**Correction** Apply only the gofmt-equivalent alignment to this struct with the authorized native patch workflow, freeze and hash the corrected source, then request a fresh independent source review. No other source edit is indicated by this review.

## Assignment comparison

Subject to the formatting blocker above, source behavior matches the two original assignments and binding root decisions:

- `RunGetLimiter` stores exactly `[2]runGetSlot` records and the production constructor fixes a one-second cooldown (`run_get_admission.go:11,27-35,50-56`). The shorter timing seam is package-private (`:43-48,58-73`). No map, queue, or per-run timer registry was added.
- Acquisition checks context before and after locking, matches an existing run before selecting any other slot, and never falls through to another slot for a busy/cooling matching run (`run_get_admission.go:80-115`). Absent runs prefer the first empty slot; otherwise only eligible idle records are selected, with earliest eligibility and slot-index tie behavior (`:118-154`).
- Matching-busy waits use change/context only; matching-cooling waits on its own deadline/change/context; absent-run waits use the earliest future idle deadline/change/context; all-busy waits use change/context. Timers are only created for positive durations and stopped on every select exit (`run_get_admission.go:108-115,142-154,167-186`). There is no default retry loop. The empty-string run ID uses the empty-slot sentinel only when its record is idle, has no cooldown, and has no busy owner; a busy or cooling empty-ID record remains nonempty and matched. `NewRunGet` always encodes a string run ID (`frame.go:308-313`); no bypass was found.
- `WriteAttemptStarted` and `WriteAttemptFinished` are idempotent under the limiter mutex; finish timestamps the final attempted payload/LF completion plus cooldown (`run_get_admission.go:201-219`). `callWithPermit` checks context after setting the write deadline and before marking/attempting writes (`client.go:227-239`); deferred finish covers payload and LF error returns before the response read (`:235-247,252-267`). Release has a last-resort finish timestamp only if a started attempt somehow reaches release unfinished (`run_get_admission.go:221-235`); normal client write returns execute the finish defer first.
- `physicalAttempt` acquires admission before connection acquisition under the existing timeout context, only for `run_get`, and releases only after decode and connection cleanup (`client.go:439-445,487-529`). Permit release is registered before connection cleanup, so Go's LIFO order performs pool release/discard before permit release (`:497-512`). `conn.callWithPermit` joins the cancellation callback before returning (`:204-222`), after which the helper retires/releases the connection and then the permit.
- The explicit failure phase distinguishes admission, connection acquisition, call, and response decode errors (`client.go:477-485,487-528`). `roundTrip` retries only call-phase safe transport failures; mutation recovery remains call-phase only; a failed second connection acquisition preserves the first call error; an admission failure returns its own typed error without resend (`:444-472`). Busy refusal remains in `Do`'s existing bounded retry (`:415-436`). Decode failure follows the healthy connection release path because `discard` remains false (`:520-528`). No change was found to correlation recovery, Ask no-resend, refusal handling, deadline normalization, or response-size enforcement.
- The one shared production limiter is constructed immediately before the live-client loop and injected in the sole production `sfwp.New` literal, including clients not selected for serve (`main.go:86-103`). Graft's exhaustive `sfwp.New(` inventory found only this production site and the test-only `internal/livetest/harness.go:247`; production construction is therefore covered. Both production `run_get` paths go through the same `Client.Do`/`physicalAttempt` hook: trace reads at `run_trace.go:72` and artifact provenance at `authority.go:758`. Other verbs bypass admission by the `req.verb == "run_get"` guard (`client.go:489-495`).

No material behavioral deviation or additional source defect was found in this source-only pass. The earlier observation that `ctx.Err()` is checked for calls without a permit is explicitly required by the original production assignment to prevent an observed pre-write cancellation from writing; it does not route those verbs through admission. This verdict does not approve runtime behavior or compiler gates.

## Evidence boundary

Graft source retrieval covered the client API, limiter API, every `sfwp.New` construction site, and every `NewRunGet` call; source anchors above were then checked against the complete files and affected fixture ranges. Graft refreshed one changed file during retrieval; no `graft build` was run. No compiler ownership or runtime approval is implied. The next step is formatting-only repair, final freeze, and a fresh independent source review before any assigned runtime gates.
