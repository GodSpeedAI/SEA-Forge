# Present-context guard prerequisite — original Phase1 assignment

Date: 2026-10-06. Builder: Luna cursor_manager_revision3_doc_builder.
Root releases one private guard prerequisite, whose input/validation behavior
was supported independently in manager review7b27271f. The complete manager is
still rejected. No manager, authorization, lease, byte-policy or public wiring
is released. Guard knowledge is as-of a check, not authority or atomic truth.

## Exact write scope

Two NEW files only under apps/godspeed-casework-go/internal/server:
`run_observation_present_context.go` and its `_test.go` file.
Read applicable instructions, Graft, exact Store/Relay/Facts/Horizon types and
nearby helper tests first. Persistent edits use native apply_patch.

Source Phase1 is an honest always-unavailable stub, no validation algorithm:

- Private dependency seams with only `Trajectory(caseID string)
  []projection.Revision` and `CursorForCase(caseID string) (string,bool)`.
  Use unit-prefixed private names, do not redefine exported server interfaces.
- Private result contains exact caseID, cursor and owned parent-ID set.
- Private function `checkRunObservationPresentContext(history, relay, caseID)`
  returns that result plus error. All non-nil inputs still return zero result
  and existing `apperr.KindUnavailable` using `apperr.New`. No fabricated ready
  result, callers, constructor integration or kernel read.

## Behavior distinguishing the eventual correct algorithm

The tests specify this complete contract, against the stub:

1. Reject blank requested case, nil dependencies, missing/empty retained history.
2. Select ONLY final returned revision for exact requested case. Reject newest
   invalid row without falling back to older valid history. Never compare cursor
   strings lexically, invent a cursor or call authority/kernel hydration.
3. Reject mismatch of revision.CaseID, snapshot.CaseID, snapshot.Cursor,
   facts.Cursor or facts record/overview/horizon Ref against requested case and
   revision.Cursor. Reject blank revision cursor or nil Facts.
4. Reject nil Horizon.Items; accept a NONNIL empty horizon as complete empty.
   Reject blank or duplicate ItemID. Copy exact opaque parent IDs, preserving
   full distinct suffixes; no sanitizing/truncating or dependence on request IDs.
5. Query relay for the exact requested case; reject absent/blank cursor or any
   cursor unequal to the final retained revision. This covers observed-cursor
   advancement without successful capture. Preserve exact opaque cursor text.
6. Success returns exact case/cursor and copied parent set. Mutating caller's
   returned set or source items/facts afterwards must not affect the other;
   repeated results are independent. Full input remains DeepEqual unchanged.
7. Show history/relay advancing across successive checks makes earlier knowledge
   insufficient: later mismatch fails; a new matching retained row succeeds.
   Explicitly test no old-parent fallback when latest horizon removes a parent.
8. Every failure returns a ZERO private result and typed unavailable error, no
   partial parent set, raw source payload disclosure or fake freshness flag.

Fixtures must be well-formed near production shapes (exact source field types,
nonempty cursors and case refs, complete nonnil horizon). Enumerate each identity
mismatch independently so one earlier defect cannot mask its assertion. Include
positive populated and empty success tests that will actually FAIL the stub;
negative stub passes are not evidence that eventual validation works. Do not add
sleep/race-luck tests or public fixture controls. No existing tests weakened.

This guard does not authorize a watcher, guarantee continuous kernel freshness,
establish public V4 readiness or prevent between-check races. Future consumers
must recheck at their own boundaries; no callsite is included in this assignment.

## Verification and handoff

NO compiler, tests, scanner, Graft build, Git or network for builder. Root owns
sole heavy push37401. Return both source/test SHA-256 hashes and every material
deviation. Independent source critic receives this ORIGINAL plus both complete
files and source anchors. Only after evidence-based fixture approval may root
assign actual focused RED with fresh RAM/swap and exclusive ownership. Production
algorithm remains HELD until root accepts actual RED. No status/debt/doc edits
by this builder. Whole T09/manager completion is not claimed.
