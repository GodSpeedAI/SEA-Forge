# Unit5A cancellation fixture independent source review — REJECT

Reviewed the original `observation-cancellation-test-first-assignment.md`, its
`observation-cancellation-fixture-critique-supplement.md`, and
`run-children-root-next-unit-clarifications.md` against the frozen
`apps/godspeed-casework-go/internal/adapters/sfwp/client_cancellation_test.go` and
the current SFWP client source. This is a source-only review. The fixture SHA-256
is `17130f1f47aa22cb259f6f733f99aa1c1b440abefa539d8bf1a1ea7f43f21f89`, matching
the frozen assignment identity. `client.go` remains
`8cdfc52a68c07df84e1f88506163da1a8dab212004d45689a4c8ab8007b3b61a` and was not
changed. No compiler or test command was run; therefore there is no expected-RED
evidence and no fixture approval.

## Blocking fixture defects

1. **Completed-mutation test waits for a peer before creating the request.** In
   `client_cancellation_test.go:367-376`, the test builds the mutation and calls
   `h.nextPeer(t)` before `h.startCall(ctx, mutation)` at line 405. The injected
   `Config.Dial` only publishes a peer when `Client.Do` acquires/dials a
   connection (`client.go:268-276, 400-409`). No call has started, so this test
   necessarily times out in `nextPeer`; it cannot exercise completed-response
   preservation or post-completion cancellation. The comment at line 404
   recognizes lazy pipe construction, but the actual first `nextPeer` remains
   before `startCall`. This is a harness failure, not an assertion RED.

2. **The pool-isolation test leaks its gated peer worker on earlier failure.**
   The second peer worker blocks on `<-writeSecond` (`client_cancellation_test.go:269-282`),
   while cleanup only closes peer connections and waits (`65-82`). If any
   preceding assertion fails, cleanup never closes `writeSecond`; closing a
   `net.Conn` cannot unblock a goroutine receiving from that channel. The
   bounded join then reports a leaked worker, contrary to the explicit
   owned-cleanup requirement. Cleanup needs an owned release path for the gate.

3. **The cancellation-winning response-race branch confuses write failure with
   peer EOF.** The peer worker forwards a failed response `Write` directly to
   `peerEOF` and returns (`client_cancellation_test.go:456-461`); the canceled
   branch then requires that value to satisfy `errors.Is(err, io.EOF)` at
   lines 503-509. A `net.Pipe` write interrupted because the client closed its
   endpoint reports the write-side close error, not an observed EOF from a peer
   read. Thus the test's intended race outcome can fail its EOF assertion even
   when the owned client connection was closed correctly. The peer must perform
   a read after a failed write (or otherwise independently observe closure)
   before calling the result EOF evidence.

## Requirement coverage and deviations

- `TestManualCancellationInterruptsRunGetAskAndMutationWithoutRetry` does
  synchronize on receipt of each request, uses the configured five-second
  request timeout and one-second completion bound, asserts unavailable wrapping
  `context.Canceled`, checks peer closure, and checks one dial. The request
  shapes include Ask and a correlated `case_commit` ID. These are good source
  assertions, but cannot supply expected-RED evidence until the fixture is
  runnable.
- The concurrent pool test distinguishes peer identities and checks that the
  second `case_list` succeeds after the first call is canceled. Its gated-worker
  cleanup defect above prevents approval.
- The partial-response test checks that a later inspect gets a fresh dial and
  clean response, giving useful positional-connection coverage.
- The completed-call test intends to synchronize cancellation after actual
  `Client.Do` return, and checks same-peer reuse by its second request and one
  dial. The pre-call `nextPeer` makes this coverage unreachable.
- The response/cancel race attempts both pool-return and connection-retirement
  paths; its cancellation-wins close-proof path is not a valid EOF observation,
  as described above.
- `waitCanceledCall` closes its owned peer after a timeout, which gives the
  blocked call a cleanup route. The suite as a whole still lacks reliable
  cleanup for the separately gated second-pool peer.
- The fixture is tests-only and makes no production change. No source-level
  correctness approval, test result, race result, or runtime claim is made.

## Decision and evidence boundary

REJECT before compilation. The required focused race/count-one expected RED
must only follow a fresh builder repair that removes the pre-call peer wait,
gives all gates a cleanup release, and observes closure independently of a
failed write. All original and supplemented assertions remain binding. The
sole compiler token is released unused. No external messages, source edits,
test edits, or changes to production files were made by this review.
