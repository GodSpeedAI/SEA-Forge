# T08 native EventSource source reviews

Root preserves the independent critic's delivered verdicts here. The critic did not build
these files, edit source, compile, run tests, launch a browser, or claim runtime proof.

## Initial review: REJECT

The first builder matched the real-kernel stack, test-only retention injection, acquired
listeners, private Vite configuration, real adapter and native EventSource requirements.
It proved initial retention-one resync in source assertions, but disposed immediately after
the first interrupted stream. That omitted the required real mutation while disconnected
and the retry's new same-cursor resync/snapshot pair. Retention-two replay did not prove
that different recovery path. Critic cited the original UI phaseOne disposal sequence and
the Go expectation of one request. A fresh builder was assigned after this rejection.

## Fresh repair review: APPROVE source coverage only

The independent critic verified `phaseOne` in `e2e/native-events-live.ts` and
`assertPhase1KernelEvidence` in `internal/server/native_events_live_test.go`:

- First real stream drop/onError leaves the actual adapter subscribed.
- Authenticated ADD_DISCRETIONARY_WORK advances the real retained head K to L and records
  exactly one durable plan_mutated event under retention one.
- A test-only gate delays handling the actual second events request until that mutation
  and relay head are stable; it fabricates neither frames nor cursors.
- The native retry resumes from K and must deliver ordered resync_required(L), snapshot(L).
  Control requested/oldest cursors are K/L; streamed state matches authenticated retained
  and current reads, K is evicted, and the kernel trace proves the mutation.
- A second actual stream drop/onError is followed by disposal during pending retry.
  No third request, stream, subscriber or callback may occur beyond the configured cap.
- Retention-two C-to-L replay, genuine single-frame mutation, absence of resync and disposal
  remain covered. Default/production retention, adapters, dependencies and Vite config stay intact.

The critic found no further material source deviation. This approves the bounded source
coverage, not the execution results, F14 proof or full T08. An independent serialized live
run and shared real-kernel conformance remain required after T07's runtime repair verification.
