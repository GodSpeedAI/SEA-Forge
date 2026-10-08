# C2 global frontier clarification

Date: 2026-10-08. DOCONLY architectural decision for a fresh complete candidate.
Revision5 remains rejected. No public source or operator approval is granted.

## Global continuity

The append ordinal belongs to the whole ledger, including non-event records
and events for other cases. Case A may have ordinals10 and12 while case B has11.
Never test continuity with `last_case_ordinal + 1` or infer missing case events
from spacing between that case's real frames.

The proposed v2 reader must validate every complete ledger row from its verified
global continuation/frontier through its pinned head, before filtering event
frames. Validate checked successive global ordinals, exact row cursor identity,
the prior continuation row/checksum and pinned-head identity. Non-event rows
advance the scan frontier; they do not produce invented event frames. A page's
`scanned_through_ordinal` identifies its last fully validated global row, even
when its event array is empty. A continuation must bind that row's exact cursor
and ordinal. Initial scans establish continuity from the ledger's actual origin;
resumed scans validate the preceding row before seeking. The existing scan,
row/frame/byte/lock/time ceilings remain unchanged. Never claim completion from
a filtered event count, a partial row or a failed page.

The Go reconciliation owner maintains one validated global scan frontier. It
accepts a page only for the expected continuation and pinned head, acknowledges
only fully validated progress and distributes its event frames by exact case ID.
Case heads and per-case delivery watermarks are separate from this global scan
frontier. A case watermark may skip many global ordinals. Broadcast is only a
wake hint; duplicate or reordered hints cannot advance any acknowledged frontier.
Repeated exact ledger pairs are duplicate deliveries; inconsistent cursor/ordinal
identity in the authoritative global scan is a global integrity failure.

## Failure scope and recovery

An unreadable, missing or malformed global row, invalid global continuation or
unexplained global ordinal discontinuity has unknown case impact. Put ordinary
live projection, bootstrap readiness and dependent streams into a conservative
cell-wide unavailable/drain state; advance no frontier or case acknowledgement
from the failed page. Keep immutable retained bytes within existing caps, but
do not serve them as ready current state or successful replay while global
coverage is unverified. A per-case index containing latest heads does not prove
which case a missing global row belonged to.

Global readiness returns only after bounded repair validates complete coverage
to a real pinned head, the derived index and strict inventory, with journal
cleanliness and capture stamps as already specified. A damaged ledger that
cannot establish this proof remains unavailable for operational review. Do not
invent a new origin, reset cumulative rebuild ceilings, manufacture a cursor,
or use current Facts to reconstruct lost global history.

Once global coverage is proved, a failure whose affected case is established
by validated metadata may remain case-local: an unretainable authorized capture,
case-local dirty journal marker, or unavailable/evicted retained capture pair.
Invalidate only that case's readiness/history and drain only its streams; other
independently validated cases continue. A later actual, stamped capture at that
case's real indexed head may establish its new current boundary, without
reconstructing old history. Resource ceilings still apply globally; a genuine
global budget exhaustion is not mislabeled a case-local gap.

## Candidate and proof requirements

Revision6 must replace every blanket claim that any ordinal gap is case-local
or that one gap cannot block another case. Distinguish unknown global scan
failures from proven case-local capture/history failures in errors, Store,
SSE admission/drain, acknowledgement, recovery and the exact approval list.
Retain all other revision5/root/bootstrap decisions and their HOLDs.

Future tests must cover interleaved cases, intervening non-event rows, empty
filtered pages with real scan progress, malformed/missing global rows, stale
continuations, duplicate/reordered wake hints, no acknowledgement on failure,
global unavailability versus isolated case invalidation, and bounded recovery.
Independent review precedes an exact operator approval request. No gate was
run and no implementation is released by this clarification.
