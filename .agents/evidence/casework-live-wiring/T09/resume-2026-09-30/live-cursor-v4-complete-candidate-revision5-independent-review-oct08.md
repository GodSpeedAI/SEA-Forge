# Live cursor V4 revision 5 — independent review

Date: 2026-10-08. Reviewed candidate SHA-256
`a48fd3e0d592fa43d57ff3ba230aebf446e2359bbba58a235bf705182093e721`.
The full revision-5 assignment SHA-256 is
`535ea0bd1ed5a7717edbddbcb7422b779ed9949fc01653ed18e434d072672187`.
**REJECT for exact operator approval; retain C-2 HOLD.** This is a DOCONLY
review. It grants no source authorization, public-contract approval, or runtime
proof.

## Scope

I read the full revision-5 assignment, original candidate assignment,
revision-3 assignment and candidate, revision-2 candidate and independent
review, root decisions and capture-boundary clarification, revision-4 root
review and independent rejection, bootstrap revision 3 and its review, and
revision 5. I checked the shared event-range and ledger append source plus
crate manifests and the delegated settlement writer. No source, test,
compiler, formatter, scanner, Git, build, or network operation was run.

## Blocking finding: global versus case-local ordinal gaps

Revision 5 proposes a global range page with `scanned_through_ordinal`
(`§2`, line 15), then says an ordinal gap invalidates only the affected case
and must not block another case (`§§4–5`, lines 56, 60). It does not define
how a global ledger gap is attributed to a case or distinguish that gap from
normal spacing between events for one case.

The current event frame has an optional `case_id` (`crates/sea-forge-server/
src/sfwp/events.rs:19–31`). `append_event` stores that ID in the shared events
ledger and returns the resulting frame (`:61–88`). `get_range` has no case
filter; it scans the shared ledger and returns event records ordered by their
ledger ordinal (`:169–208`). `LedgerEntry.append_ordinal` is a ledger-wide
`u64` (`crates/sea-forge-ledger/src/types.rs:191–208`), incremented from the
last entry while append owns the stream lock (`:701–723,1029–1042`). Thus
case A can legitimately have ordinals 10 and 12 when case B has ordinal 11.
Treating A's local difference as a missing frame would falsely invalidate A.
Conversely, if a complete global scan discovers a missing ordinal, its absent
row has no `case_id`; the candidate gives no basis to mark only one case
affected.

Before approval, specify that case-local ordinal spacing is allowed and define
continuity from the complete global scan frontier. Also define how an
unexplained global discontinuity affects case readiness when its case is
unknown. The current index stores latest per-case pairs, but the candidate does
not say how this index establishes the missing-row case or proves per-case
isolation. Do not infer either property from `last_case_ordinal + 1`.

## Prior findings and architecture

Revision 5 materially closes the prior revision-4 review findings: historical
GET pairs cursor with `capture_digest`; SSE reconnect pairs `last` with
`last_capture_digest` and specifies pre-header resync; it states the 1 MiB
per-item and 64-replay caps; fixes Go digest fields/order/exclusions and golden
coverage; persists bounded operation-scan continuation and allows absence only
at a pinned head; blocks empty/bootstrap success on any pending journal item;
names permission-broker, rejection, probe, sandbox, and cancellation-recovery
settlement writers; keeps complete writer coverage as a pre-implementation
gate; and candidly records the lost revision-4 preimage. I verified the
delegated success writer in `crates/sea-forge-server/src/delegation.rs`:
`execute_with_permission_broker` persists `runs/<run>/settlement.json` through
`commit_view` at lines 793–814. Its rejection path persists the same view at
lines 1090–1105.

The journal placement does not create a crate cycle: case-runner depends on
ledger (`crates/sea-forge-case-runner/Cargo.toml:10–22`), and server already
depends on case-runner (`crates/sea-forge-server/Cargo.toml:16`). Revision 5
places the shared journal in case-runner and lists delegation persistence as a
server writer (`§§2–3`); it does not propose moving the server routine into
case-runner or adding a reverse dependency. Its scope preserves D-2 and existing
authority checks (`§1`), and accurately says cooperative locking does not
cover manual, plugin, or out-of-tree writers (`§2`, lines 27, 44). It retains
full writer migration proof as an implementation-readiness HOLD.

The candidate enumerates the required approval set and states that C-2,
no-case success, schemas, Store/SSE, and implementation remain held pending
exact operator approval and later verification (`§§5`, lines 68–70). Its
2,584-word length exceeds the assignment's approximate 2,200-word target by
384 words; the target was conditional (“if practical”), so this is editorial,
not a correctness blocker.

## Disposition

Keep C-2 and implementation HOLD. The prior revision-4 findings are addressed,
but the global/case-local sequence policy remains ambiguous and affects the
candidate's gap isolation and acknowledgement guarantees. Add a new immutable
revision defining that policy, then obtain another independent review before
requesting exact operator approval. No gates or runtime behavior are claimed.
