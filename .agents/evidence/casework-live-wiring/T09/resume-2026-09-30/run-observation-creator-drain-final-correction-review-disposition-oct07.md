# Independent review disposition: final creator/drain correction — 2026-10-07

This immutable disposition supplements the prior review
`run-observation-creator-drain-final-correction-independent-review-oct07.md`
(SHA-256 `03243da7879ce821eed31fe23e7640138e78f57613b608cc687322811f06fe99`).
Preserve that historical review unchanged. Its proposal-only APPROVE is
**withheld pending an additive wording clarification** to
`run-observation-creator-drain-final-correction-supplement-oct07.md` (SHA-256
`6526c249f800f23b966063b69ef97f00fa3531b69c0057ccedd52241b2160758`).

## Required clarification

1. **Stop only watcherless entries.** The phrase “marks its now-ineligible
   batch for stop” must expressly mean that the canceled lease loses its own
   eligibility and only entries with no surviving eligible lease become
   watcherless and are stopped. Shared entries with another eligible lease
   keep running and must not be forced into code-4/no-read solely because
   this Prepare canceled. The proposal later says rollback cancels only
   watcherless workers and surviving eligible leases keep shared work, but
   the earlier batch/worker wording could be read to stop every entry.

2. **Logical owner bound, not instantaneous goroutine bound.** The “maximum
   16” background drain wording must describe at most one logical outstanding
   drain owner for each currently admitted lease, plus the single Stop owner.
   It must not claim a strict instantaneous maximum of 16 goroutines or an RSS
   cap: after exact registry removal permits a new admission, an old owner's
   final completion-channel close/return may briefly overlap. The required
   safety guarantee remains that capacity and reverse ownership are retained
   until actual creator/worker completion and JOIN.

## Scope pending independent review

The nonblocking `prepareDone` admission check, caller-canceled held-list test,
post-reservation rollback and final context check, stable completion channels,
single-owner retry protocol, and exact-lease creator ordering remain
otherwise approved at the document-design level. This disposition withholds
the proposal approval until the two wording points are corrected and
independently reviewed. It approves no source, fixture, implementation,
compiler, or runtime behavior.

The failed attempt to create a qualification file was rejected by automatic
approval review before execution due a usage limit; no file was written by
that attempt. This disposition is a new file and does not modify or replace
any earlier record.
