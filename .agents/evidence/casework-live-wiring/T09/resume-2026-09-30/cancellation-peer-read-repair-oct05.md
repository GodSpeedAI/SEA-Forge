# T09 Unit5A cancellation peer-read fixture repair

Date: 2026-10-05

## Scope and identities

Fresh tests-only builder repair for the rejected retirement-read assertion.
The assigned file was the frozen
`apps/godspeed-casework-go/internal/adapters/sfwp/client_cancellation_test.go`;
no helper, production source, or other fixture was in scope.

- Third-frozen fixture snapshot, retained byte-for-byte:
  `cancellation-fixture-third-frozen-oct05.go.snapshot`, SHA-256
  `c4f1bd73f0f580b25d2367acaf5b331779d7bec078f08a17295cfdbe7314f43b`.
- Repaired fixture SHA-256:
  `be8ad34bfe93306ede3fe1590b906c4ce2c4e9764c0db89af2f65e6cd7d0f91c`.
- Production `client.go` remains SHA-256
  `e0d3c12c1af75b35c041889a45db7da3978db26bb0226e38149b07ef0f885efe`;
  this repair made no production edit.

## Repair rationale

The retirement review correctly found that `readCancellationRequest` tries to
set a deadline before reading, and `net.Pipe.SetReadDeadline` can return
`io.ErrClosedPipe` after the client endpoint has closed. In this race, however,
the first peer request already goes through `readCancellationRequest`. That
sets the four-second read deadline before it publishes request receipt; only
after receipt does the test release `raceStart`, which starts the response and
cancellation race. The original deadline therefore already bounds the later
retirement observation.

The minimal repair changes only the race peer's second/retirement read to
`bufio.NewReader(conn).ReadString('\n')`. It performs an actual read without
retrying deadline setup. A retired peer can therefore satisfy the strict EOF
assertion only when `ReadString` returns that actual read error. On reuse, the
same read obtains the exact next request line and the existing peer identity,
dial-count, response, and successful-Do-before-reuse assertions still apply.
This follows the retirement critique's requirement to prove EOF by reading,
while using the already armed deadline instead of adding a new readiness gate
or moving race ordering.

## Exact source diff

```diff
@@ -464,7 +464,10 @@ func TestRunGetCancellationRacesResponseAndPoolReturn(t *testing.T) {
 		writeResult <- err
 		// Write completion and the caller's Do result are separate race outcomes.
 		// Read independently to learn whether this peer is reused or retired.
-		line, err = readCancellationRequest(conn)
+		// The initial request read already armed the peer deadline before raceStart.
+		// Read directly here so a retired pipe reports EOF from Read itself instead
+		// of SetReadDeadline returning ErrClosedPipe before any read is attempted.
+		line, err = bufio.NewReader(conn).ReadString('\n')
 		if err != nil {
 			peerEOF <- err
 			return
```

## Verification boundary and deviations

`gofmt -d apps/godspeed-casework-go/internal/adapters/sfwp/client_cancellation_test.go`
produced no output. No compiler, test, or runtime command was run, per assignment;
this record makes no compiling assertion-RED or runtime claim. All strict EOF,
completed-Do-before-cancel reuse, exact peer identity, dial-count, no-resend,
manual cancellation bounds, and owned cleanup assertions remain unchanged.

Material deviation: the retirement review suggested starting a post-response
observer and signaling readiness before releasing the race. The root
clarification established that the existing initial request read had already
set the peer deadline before `raceStart`; the one-call direct-read change
therefore fixes the actual error path with the least fixture change. No other
deviation or test assertion change was made.
