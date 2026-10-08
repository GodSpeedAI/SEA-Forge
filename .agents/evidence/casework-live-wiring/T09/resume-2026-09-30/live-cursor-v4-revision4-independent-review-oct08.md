# Live cursor V4 revision 4 — independent review

Date: 2026-10-08. **REJECT for exact operator approval; retain C-2 HOLD.**
Reviewed candidate SHA-256 `3cbf9e42561bbaaf965815709266362460ba76bfdf930bb5549afd4f3ffe7bf2`
was independently recomputed and matches the current file. This is a DOCONLY
review, not source, runtime, or public-contract approval.

## Original review assignment

> New INDEPENDENT DOC critic task, noheavy/source/Git: Root verifiedGraft6actualpairs/11hashes/pass, tokenROOTidle. You did NOT build C2candidate. Read FULL original live-cursor-v4-complete-candidate-assignment-oct08.md; FULL new revision3-assignment-oct08.md; root-decisions-oct08 andcapture-boundary-clarification-oct08; base/revision2+REJECTe85135de/bootstraprev3+review; actual currentrevision4 candidateSHA3cbf9e42561bbaaf965815709266362460ba76bfdf930bb5549afd4f3ffe7bf2; root newrevision4-root-review-oct08.md (fiveconcretegaps). Independently verify consistency/completeness/exactoperatorapproval readiness, sourceanchors/ULIDhelpervisibility/writerinventory/journalepochdurablecrashes/recoveryboundedscan/strictinventory/intentdigest+historicalGET+SSEdigestmatching/caps/deadlines/authority/noI/Ounderlocks. Cite direct code/docs, explain EVERYmaterialdifferencefromFULLinstructions; no sufficientevidence=noAPPROVE. Onlyproposalreadyclaim, gatesunrun/sourceHOLD. Native newimmutable review, preservecandidate; noimplementation/noextraGraftbuild. ≤1000words focusedfindings, no giantidentitytables/manualhashtranscription. Root expects HOLD ifgapsremain; freshdifferentbuilderwillrepair.

## Scope and preserved decisions

I read the complete original and revision-3 assignments, root decisions and
capture-boundary clarification, base/revision-2 candidate and its rejection,
bootstrap revision 3 and its independent review, the revision-4 candidate and
root review, C-2 writer recon, and relevant source/spec/schema anchors. The
candidate preserves revision-3 bootstrap boundaries, exact real IDs, append
ordinal ordering, the no-new-verb direction, D-2 authority, cooperative-lock
limits, and the explicitly unproven writer migration. It correctly states
that captures are as-of, not atomic Facts/history reconstruction, and says no
network I/O or callbacks run under the Store mutex. The writer journal, checked
epoch, bounded range/index, strict inventory, Store/SSE model, digest, and
approval list remain proposals; the listed gates are unrun.

## Blocking findings

1. **Restart capture identity is not carried through history or reconnect.**
   Revision 2 explicitly requires `capture_digest` paired with historical
   `GET /api/world?cursor=...` and `last_capture_digest` paired with SSE `last`
   (revision-2 §4). Revision 4 §6 only binds the digest to intents; its SSE
   request is `/api/events?case_id=...&last=<opaque>` and it specifies no
   historical GET digest match. The current adapter likewise persists and
   sends only `lastCursor` (`httpCaseworkAdapter.ts:297-356,379-430`), while
   current SSE uses cursor-only resume (`server.go:308-317`). A restart may
   replace the capture at the same real cursor, so the candidate cannot tell a
   client to resync that old capture at these boundaries.

2. **Required item and replay caps are incomplete.** Revision 4 §6 gives
   64 revisions/4 MiB per case and 4 MiB replay, but omits the root-selected
   1 MiB maximum individual snapshot/event and an explicit maximum of 64 replay
   revisions. The 1 MiB range-page limit in §2 is not a snapshot/event limit;
   byte caps do not imply the replay-count cap. These limits are also part of
   the approved candidate requirements and must be stated in the exact policy.

3. **Digest, mismatch, and recovery behavior is underspecified.** Revision 4
   §6 says Go `encoding/json` bytes excluding the digest, but does not give the
   exact included field set/order, byte-level golden/exclusions, or digest
   validation profile required by the root decision and revision 2. It gives
   generic 404/409 resync codes, not the exact historical/SSE digest-mismatch
   body and client reset sequence; no reconnect digest is present. Section 3
   searches an operation ID from a prior ordinal to a pinned head and permits
   reconciliation after an “absent” complete bounded scan, but does not define
   persisted continuation for an operation-ID scan that spans bounded pages.
   A partial scan cannot prove absence or authorize a reconciliation event.

4. **Strict inventory does not bind empty success to journal recovery, and
   writer scope is still not complete.** Revision 4 §4 excludes a flat legacy
   case file from membership, but does not say that any pending journal
   operation blocks complete-empty/bootstrap. A crashed pending create can
   leave no directory case while the event head is unchanged; inventory must
   consult pending journal state or fail unavailable. The `case.list` clause
   specifies filesystem errors/caps but not this pending-operation rule.
   ULID visibility itself is verified: `sea-forge-ledger` exposes `pub mod
   types` (`src/lib.rs:5-6`) and `pub fn ulid` (`types.rs:93`). The inventory
   remains a HOLD, not exhaustive proof: `LiveSource.Facts` includes
   `RunsListForCase` (`internal/projection/live.go:64-78`), and a planned
   agent writer reaches `delegation::execute_with_permission_broker`
   (`case_dispatch.rs:609-631`), which writes case-keyed records and
   `runs/<run>/settlement.json` (`delegation.rs:261-278,803-814`);
   `run_views` reads that settlement into the projected summary
   (`run_views.rs:793-819`). The table names `case_dispatch` broadly but does
   not enumerate this persistence boundary. The separate all-writer recon
   confirms that CLI/CaseRunner writers bypass global publication and that
   server mutation/publication is non-atomic. Revision 4 acknowledges
   migration coverage is unproven, appropriately, but an exact implementation
   approval must keep that proof as a gate.

5. **Prior revision-4 provenance is unavailable.** Root review §5 records that
   an earlier reported revision-4 hash was overwritten without preserving its
   preimage. I verified only the current candidate identity above. No claim
   about or comparison to the lost earlier content is possible; revision 3 and
   prior reviews remain preserved.

## Disposition

The candidate carries the root’s broad architecture direction and preserves
its authority, no-network-under-lock, bounded-work, and honest-capture limits.
The five gaps above remain material departures from the full assignments and
root choices. Keep C-2 and no-case success held; do not request exact operator
approval or release implementation from this revision. A fresh builder should
add a new immutable candidate revision addressing these findings. No source,
test, build, Graft build, Git, or external operation was run for this review.
