# Unit5A cancellation fixture independent review, round 2 — REJECT

Reviewed the original assignment, cancellation critique supplement, and root's
next-unit clarifications against the fresh tests-only fixture
`apps/godspeed-casework-go/internal/adapters/sfwp/client_cancellation_test.go`.
The fixture SHA-256 is
`d75ca5c8bbb46bb1f774cfe384f4c1d80ba76f024020d5388304d611d8e618d3`, matching
the newly frozen identity. The original
`client_cancellation_test.go` snapshot remains separately retained at
`cancellation-fixture-first-frozen.go.snapshot`. `client.go` still hashes to
`8cdfc52a68c07df84e1f88506163da1a8dab212004d45689a4c8ab8007b3b61a`.

The three prior source findings are repaired: the completed-mutation test now
starts `Do` before waiting for the peer (lines 383-386); cleanup releases the
concurrent peer's gate (268-272); and the race peer independently reads after a
failed write to establish EOF (460-468). No production files were changed.

## Remaining blocking finding: race predicate rejects a valid interleaving

At lines 498-517, the race test couples the API result too tightly to the peer
write result and subsequent connection disposition:

- If `Do` returns a valid response, lines 500-510 require the original peer to
  be reused and require exactly one dial.
- If `Do` returns cancellation, lines 511-518 require the peer write to have
  failed and require EOF.

Those are not exhaustive valid outcomes for a genuinely simultaneous response
and cancellation. The response can be fully received and decoded while the
cancellation callback wins before pool release, yielding a valid `Do` response
with the old connection retired. Conversely, the peer's write may complete
while cancellation wins before the call reports success. The root's stated
interleaving concern therefore remains: success does not prove the peer stayed
healthy, and a successful peer write does not by itself prove `Do` returned a
valid response. The test rejects these cases instead of checking the valid
response/error contract and then validating whichever connection disposition
actually occurred.

The distinct `TestCancelAfterCompletedMutationPreservesAndReusesItsConnection`
does correctly establish the strict required case: `Do` has returned a valid
mutation response before cancellation, followed by an exact same-peer reuse
check (lines 411-435). Keep that assertion strict. For the race test, accept the
valid cancellation/response outcomes while still proving any retired connection
is not reused, exact peer identity and dial accounting match the observed
disposition, and owned workers join. This is required before the race test can
serve as a runnable expected-RED fixture for callback retirement and pool
return.

## Remaining coverage and evidence boundary

The synchronized run_get, Ask, and correlated mutation cancellation cases
retain the five-second request timeout, one-second manual-cancellation bound,
typed error-chain assertion, and request/peer synchronization. The concurrent
pool case checks the other request succeeds and the released gate now participates
in cleanup. The partial-read case still requires a fresh positional connection.
The completed mutation case now reaches its intended post-Do cancellation check.
These source assertions do not substitute for execution evidence.

REJECT before compilation. No test or compiler command was run, so this record
makes no expected-RED, Go build, race, or runtime claim. The sole compiler token
remains unused. No fixture or production edits were made by this reviewer.
