# T09 Unit5A cancellation runtime source review — READY FOR GATES

Date: 2026-10-05

## Scope and identities

Source-only independent review of the original
`observation-cancellation-test-first-assignment.md`,
`observation-cancellation-fixture-critique-supplement.md`,
`observation-cancellation-runtime-root-assignment.md`,
`run-children-root-next-unit-clarifications.md`, the rejected fixture review and
its round-two review, the accepted fixture review, and the fresh builder record
`cancellation-peer-read-repair-oct05.md`. I reviewed the exact current fixture
and production source. No compiler, tests, or runtime commands were run.

- Frozen fixture snapshot: `cancellation-fixture-third-frozen-oct05.go.snapshot`,
  SHA-256 `c4f1bd73f0f580b25d2367acaf5b331779d7bec078f08a17295cfdbe7314f43b`.
- Current repaired fixture SHA-256:
  `be8ad34bfe93306ede3fe1590b906c4ce2c4e9764c0db89af2f65e6cd7d0f91c`.
- Current `client.go` SHA-256:
  `e0d3c12c1af75b35c041889a45db7da3978db26bb0226e38149b07ef0f885efe`.

The fixture repair changed only the race test's second read at
`client_cancellation_test.go:467-470`: after the first request read sets a
four-second deadline in `readCancellationRequest` (`:117-122`) and publishes
request receipt, the test releases `raceStart` (`:462-463`). The race peer now
does `bufio.NewReader(conn).ReadString('\n')` directly, without attempting a
new deadline setter after a possible close. This establishes EOF only from an
actual read; on same-peer reuse it reads the next request line. Existing strict
EOF assertion, exact peer identity, dial count, successful response validation,
bounded manual-cancel assertion, and worker cleanup remain in place. The
builder's recorded deviation from my prior suggestion to add a separately
signaled post-response observer is justified: the initial request read already
prearms the deadline before the race begins, so this one-call repair is
sufficient and narrower.

## Production lifetime review

The production diff is confined to `conn.call` in
`apps/godspeed-casework-go/internal/adapters/sfwp/client.go:191-250`.

- `context.AfterFunc` (`:198-201`) closes only this checked-out `cn.nc` and
  signals `callbackDone`; the callback does not touch `cn.dead` or pool state.
- The call defer (`:203-215`) waits when callback stop reports it has started,
  marks the connection dead while the call still owns `cn.mu`, and preserves
  the valid response if the read completed. The existing unlock defer is older,
  so the callback join/dead update executes before unlocking and before
  `roundTrip` can release the connection.
- A failed I/O result after cancellation is wrapped as typed unavailable with
  `ctx.Err()` in its error chain (`:211-215`). Existing write/read deadline
  classification and `readBoundedLine` response limit behavior remain in
  place (`:216-250`, `:257-276`); the cancellation path does not reset a
  canceled operation to a later deadline.
- Pool release checks `cn.dead` and decrements `live` instead of returning a
  retired connection to idle (`:358-370`). Failed calls are discarded before
  retry/recovery handling (`:419-432`). Mutation recovery and read-only retry
  remain guarded by `ctx.Err() == nil` (`:433-450`), preserving correlated
  mutation ambiguity and inspect retry boundaries. Ask retains its existing
  no-resend/no-status semantics. The callback is per connection, so other
  checked-out and idle pool members are not closed.
- The separate completed-mutation fixture case synchronizes on successful
  `Do` return before canceling (`client_cancellation_test.go:372-435`) and then
  requires exact same-peer reuse. The response-race case permits a real decoded
  response while requiring retired-connection or exact-reuse disposition to
  match peer identity and dial evidence (`:439-550`).

I found no source-level blocker against the runtime assignment. This is
readiness for root-owned verification of the two frozen files only; it is not
production approval, race-safety proof, or evidence of actual blocked-read
cancellation. The focused cancellation matrix, full SFWP race, canonical
casework-go-check, and full-module race/count-one/parallel-one gates remain
required, along with their prescribed preflights and immutable outputs. Do not
approve the production repair unless those gates pass and their artifacts are
reviewed.

## Material changes and deviations

- Fixture change: only the race peer's post-response read changed from
  `readCancellationRequest` to a direct buffered `ReadString`, using the
  already armed deadline. No other helper or assertion changed.
- Runtime change: only `conn.call` was changed in the reviewed production
  patch; no dependency, public interface, pool architecture, request identity,
  correlation, or retry policy change was found.
- The builder reports `gofmt -d` produced no output. I did not independently
  run or treat that as a test/compiler result.
- No production or test gate was run by this reviewer. No other material
  deviation was found from the original fixture/runtime assignments.

Decision: **SOURCE READY FOR REQUIRED GATES; NOT APPROVED WITHOUT GATE EVIDENCE.**

🌱 graft saved ~40,611 tokens (~$0.03) this turn (1 call).
