# T09 Unit5A cancellation retirement independent review — fixture REJECT

Date: 2026-10-05

## Scope and identities

Source-only review of the original cancellation test-first assignment, its
critique supplement, root clarifications, runtime root assignment, the accepted
`cancellation-fixture-independent-review-oct05.md`, the frozen fixture, current
client source, and the installed Go `net.Pipe` implementation. No compiler or
tests were run. No production, fixture, status, Git, or debt files were changed.
The sole new file is this immutable review.

Reviewed fixture SHA-256:
`c4f1bd73f0f580b25d2367acaf5b331779d7bec078f08a17295cfdbe7314f43b`.
Reviewed `client.go` SHA-256:
`e0d3c12c1af75b35c041889a45db7da3978db26bb0226e38149b07ef0f885efe`.
These match the requested fixture and new-client source identities. The final
builder evidence records only one focused race failure:
`TestRunGetCancellationRacesResponseAndPoolReturn`, where the retired race peer
ended with `io: read/write on closed pipe` rather than the required independently
observed `io.EOF`.

## Blocking finding: retirement proof is not an actual read

`readCancellationRequest` (`client_cancellation_test.go:117-122`) first calls
`conn.SetReadDeadline(...)`, then invokes `bufio.Reader.ReadString`. In the race
peer (`:456-470`), after writing the response it immediately calls that helper
to distinguish same-peer reuse from retirement. When the client has already
closed its endpoint, the setter itself returns `io.ErrClosedPipe`; the helper
returns before making any `Read`. The caller records this error in `peerEOF`,
then the retired-peer branch (`:547-548`) rejects it because it is not `io.EOF`.

This behavior is confirmed directly by the installed Go 1.27.1 standard-library
source, `/home/sprime01/.local/share/mise/installs/go/1.27.1/src/net/pipe.go`:
`SetReadDeadline` returns `io.ErrClosedPipe` when either local or remote done is
closed (`:219-224`). By contrast, `Read` returns `io.EOF` for a closed remote
endpoint (`:149-164`). Thus the current assertion is structurally unable to
establish its promised independent EOF proof in the relevant interleaving. The
builder's only remaining failure is a fixture proof defect, not evidence that
the client failed to retire the connection.

Minimal fresh-fixture repair: arrange and signal the peer's post-response
observer while the pipe is still open. Set its read deadline then, start the
actual blocking `Read`/buffered request-line read, and only after an explicit
ready signal release the response/cancellation race. The observer must publish
EOF only when that actual `Read` returns `io.EOF`; preserve a distinct path for
reading the next request line on exact same-peer reuse. Keep exact peer identity
and dial-count checks. Do not translate `SetReadDeadline`, write errors, or
`io.ErrClosedPipe` into EOF. Keep request-receipt synchronization, the bounded
manual-cancel assertion, no-retry/no-status checks, response decoding,
post-completion same-peer reuse, cleanup releases, and bounded joins intact.
Freeze the repaired fixture and source-review it before any new compile attempt.

## Production lifetime review and evidence boundary

The new `client.go` installs a context `AfterFunc` scoped to the checked-out
`cn.nc`; the callback only closes that socket and signals completion. The call
owner waits for an already-started callback in its defer while still holding
`cn.mu`, marks `cn.dead`, and returns typed unavailable wrapping
`ctx.Err()` when canceled I/O failed. The round-trip owner discards failed
connections before its cancellation guards allow mutation recovery or
read-only retry. This ownership shape is consistent with the bounded runtime
assignment: callback has no access to `dead` or pool bookkeeping, callback
retirement precedes release, and unrelated pooled connections are not targeted.
The response/cancel branch can preserve a decoded response while the callback
still retires the connection; the separate completed-mutation test correctly
requires `Do` to have returned before cancellation and then checks same-peer
reuse.

This is a source-level lifetime assessment only. It does not prove race safety,
actual blocked-read cancellation, pool accounting, or production correctness.
The focused run exited 1; therefore I do not approve the production repair or
the fixture's expected-RED/green proof. The source appears directionally within
the assigned boundary, but production approval requires a fresh fixture with a
valid actual-Read EOF proof and the required focused, package-race, canonical,
and full-module evidence. No claim is made about those unrun gates.

## Material deviations

- No compiler or test command was run, as instructed.
- No source, test, status, Git, or debt files were modified; only this new
  review artifact was written.
- The reviewed source identity is the supplied `e0d3c12c` client revision,
  whose cancellation callback is present. This is not the earlier frozen
  baseline client identity documented in the prior accepted fixture review.
- The builder's failure is not accepted as EOF evidence and does not count as
  a production failure; it is rejected as a false-negative fixture assertion.

Decision: **REJECT fixture proof; no production approval.**

🌱 graft saved ~1,845,870 tokens (~$1.47) this turn (2 calls).
