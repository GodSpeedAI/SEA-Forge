# Physical run.get admission fixture — independent source review

Date: 2026-10-06  
Verdict: **REJECT — mandatory no-spin release/reacquire fixture does not exercise the release contract.**  
Scope: source-only review against the original fixture assignment and its complete proposal/review/adjudication trail. No compiler, test, Git, status, or runtime operation was performed.

## Evidence reviewed

- Original assignment: `run-trace-physical-admission-fixture-root-assignment-oct06.md`.
- Original proposal, first independent rejection, repair proposal, root adjudication, round-two rejection, wait clarification, and round-two approving supplement in this directory.
- Current source: `apps/godspeed-casework-go/internal/adapters/sfwp/client.go`, `run_get_admission.go`, `run_get_admission_test.go`, and `client_run_get_admission_test.go`.
- Graft retrieval: `graft ask "physical run.get admission fixture limiter retry semantics" --source --in apps/godspeed-casework-go/internal/adapters/sfwp/` refreshed/used the current index and located the new fixture declarations and client retry paths. Full changed Go files and exact diff were then inspected directly.

## Scope and declaration findings

The actual source change is within the permitted four paths. `client.go` adds only the documented `Config.RunGetAdmission` field (lines 66–70); it adds no hook or behavior. `run_get_admission.go` contains only the proposed interfaces, fixed two-slot/one-second constructor, package-private clock/timer seams, nil normalization, and typed-unavailable stub. The timer factory correctly adapts `time.NewTimer(d)` to the declared interface with `realRunGetAdmissionTimer{timer: ...}`; no function-return covariance/type mismatch is apparent. No production admission algorithm, send integration, constructor wiring, or manager logic was added.

Current SHA-256 identities:

| File | SHA-256 |
|---|---|
| `client.go` | `d2f993d7dda3a5d63a414ef5a64d6f995c04de15f94537374940dc6c031027c6` |
| `run_get_admission.go` | `f661bd8f8d9046d6eac29738d8521a48ca964361db2e107151cdedce9a0d62a7` |
| `run_get_admission_test.go` | `933934649613de9015af1e078b62a577d81bca00c9c06e740a969d06a208d469` |
| `client_run_get_admission_test.go` | `6b5c7ecbbb6753a77efbe948fb031d82e3b02e7894ea1fe98262fbce91f9412a` |

## Material defect

The round-two supplement makes the busy same-run / expired unrelated-slot case mandatory, and requires both cancellation and normal acquisition after release. The fixture sets up the state directly at `run_get_admission_test.go:250–256`, correctly joins the canceled waiter before checking timer count at `:277–288`, then starts a second waiter. However, its claimed release path at `:310–315` directly clears `limiter.slots[0].busy`, replaces `limiter.changed`, and closes the old channel. It never obtains or releases a permit in this path. The assertion at `:317–325` therefore proves only that a waiter reacts to a manually fabricated channel/state transition; it does not prove that `RunGetPermit.Release` performs the transition and wakeup. This is the exact path whose behavior the fixture is meant to distinguish. It violates the assignment's instruction not to self-test a parallel/fabricated algorithm and the supplement's required release/reacquire path.

The test also does not verify a lack of repeated wakeups while the waiter remains pending. It counts timer factory calls after cancellation completion (`:286`) and after the manually changed state completes (`:324`); a faulty implementation that repeatedly wakes without creating timers could evade this assertion. The observed `Done()` wrapper establishes only that `Done()` was requested (`:268–275`, `:370–375`), not that the goroutine is blocked in the select over the correct wait set. The no-spin scenario's required core property is absence of immediate deadline/timer churn; the timer count helps, but the direct state mutation means the release half remains unproved.

## Other fixture completeness and honesty

- The limiter contract covers fixed constructor dimensions, same-run exclusion, exact matching-slot reuse/no fallthrough, two-slot contention, no eviction of unexpired records, cancellation/deadline errors, prewrite release without cooldown, and a controlled-clock cooldown calculation (`run_get_admission_test.go:18–244`). Some tests seed slot state directly, which can be useful to arrange edge states; it is not itself a behavioral implementation. The mandatory no-spin test crosses the line by manually implementing the transition under test.
- The direct cooldown calculation at `:229–243` verifies permit method bookkeeping against a manually advanced clock; it does not independently prove a client integration timestamps a blocked/partial payload or LF write at the final write return. The separate client tests do observe start/finish event order for payload/LF and blocked/error paths (`client_run_get_admission_test.go:196–220,250–290,319–384`), but the client has no hook yet, so they are properly assertion-RED fixtures and do not claim runtime proof.
- The four-attempt `run_get` sequence asserts four physical requests and four admission calls (`client_run_get_admission_test.go:80–112`). Ask and mutation exclusion, refusal preservation, and mutation no-resend are represented (`:114–194`).
- Cancellation retirement coverage gates `Close` and checks permit release remains absent while close is blocked (`client_run_get_admission_test.go:415–477`). It does not add a synchronization barrier proving cancellation occurs only after request/write entry (`:454–460` cancels immediately after launching the goroutine). The close callback may still be exercised, but the test's setup does not deterministically establish the intended post-write cancellation state. A controlled request-received/write-started barrier should precede cancellation.
- Several queued tests defer cancellation but do not explicitly join their waiter goroutine after a fatal assertion (for example `run_get_admission_test.go:35–61,83–107,114–138,152–179`). Their bounded result channels avoid blocked sends, and cancellation should unblock a conforming implementation, but the assignment asks bounded completion/join evidence; cleanup currently cancels without joining those goroutines.
- The stub itself is explicitly typed unavailable (`run_get_admission.go`), and no test permanently asserts that stub behavior as the desired contract. The tests are honestly expected to fail before implementation. No focused test result, actual assertion RED, or compile claim is present or inferred.

## Required repair

Replace the fabricated release transition in the no-spin fixture with an actual acquired permit held by the first owner, then release that permit through `Release`; retain the stale unrelated idle deadline and busy matched record setup as needed. Wait for actual waiter completion and assert the timer count remains zero across both the cancellation case and release/reacquire case. Add a deterministic post-request/write barrier before cancellation in the client cleanup fixture. Join queued goroutines in cleanup after canceling their contexts. Do not weaken or delete any required behavior to make the typed-unavailable stub appear green.

No compile ownership, production implementation, or runtime approval is conveyed by this review. The actual assertion RED remains required after source approval, under the separately assigned compiler/capture protocol.
