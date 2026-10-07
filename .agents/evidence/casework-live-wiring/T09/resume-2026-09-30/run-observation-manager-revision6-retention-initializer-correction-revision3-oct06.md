# Private run-observation manager revision 6 — terminal retention lifecycle correction

Date: 2026-10-06  
Status: DOCONLY normative correction to revision 2, whose independent review
is `run-observation-manager-revision6-retention-initializer-correction-revision2-independent-review-oct06.md`
(reviewed revision 2 SHA-256 `3494e3de99b2f5d3fe725a85971dccd0d70cef226da257ce237cce7fba2d72e1`).
Preserve revision 2 and all earlier records unchanged. This note supersedes
only revision 2 §2 terminal-retention lifecycle wording, its terminal test
sequence, and the budget-image representation needed for that transition.
All other clauses remain in force.

## Terminal candidate rejection and stop ordering

On a validated terminal source candidate that exceeds the proposed retained
value budget, atomically reject all candidate data: no frames, IDs, ordinals,
window, standing, source timestamp, counts, or other candidate fields become
current, retained, or disclosed. In particular, do not publish the source
candidate's terminal status. Return only the generic typed
`ErrPollerRetentionUnavailable` and a zero delta; do not include candidate
payload or metadata in caller-facing errors, DTOs, deltas, or logs.

Separately from the rejected candidate, atomically record manager-generated
safe internal `terminalRetentionFailure` and stop-control state under the
manager mutex. This state marks the poller unavailable and bars **every**
future `ReadRunTrace` start for that poller, including a start racing with
terminal-candidate validation. It does not accept or disclose any source
terminal value. The stop decision must precede worker join; never wait for
`workerDone` before setting the marker that prevents the worker from starting
another read.

The in-progress read has already returned its validated candidate through the
`RunTracePort`; that call returns only after its adapter/client retirement has
completed under the existing port/admission contract. The poller worker then
observes the stop state, starts no further read, completes, and alone closes
`workerDone`. The teardown owner joins that worker outside the manager mutex,
then drains acquired refs, in-flight lease operations, and per-target notifier
references outside the mutex. Preserve revision 6/addendum ownership: no
callbacks, waits, cancellation, or joins under the lock. A departing watcher
does not cancel shared work needed by another authorized watcher; this
terminal poller marker, however, prevents new reads for all attachments to
that poller. Keep poller and lease capacity counted until the actual port call
has returned after retirement, the worker has been joined, and every owned
reference/operation/notifier drain has completed. A timeout leaves entries
stopping/draining and counted; it never releases capacity.

No caller receives the rejected candidate's terminal standing or other
candidate data. Affected watchers receive only the safe typed unavailable
result and detach/drain under the stated ownership rules. The existing
nonterminal retention-overflow policy and transient `ErrPollerReadUnavailable`
recovery rule are unchanged: the former still awaits exact operator approval,
and the latter remains per-poller recoverable without detaching the lease.

## Fixed-width private retained-image control representation

The existing per-poller `1<<20` canonical retained-value budget and the
16-cohort/128-attachment limits remain proposed policies requiring exact
operator approval before source work. For that proposed accounting image,
encode bounded internal lifecycle enums and booleans as fixed-width
single-digit codes, and encode `uint64` lifecycle counters/generations as
exactly 20 decimal digits. Include every such control field exactly once in a
deterministic image from the outset, including availability, terminal
retention-failure, stop/scheduling state, and relevant generation/counter
fields. These fields must be present at their fixed width in both unset and
set states; do not use omission or variable-width values for them. Thus a
marker-only lifecycle transition changes code values but not serialized
length, and remains representable even when the current candidate image is
exactly `1<<20` bytes. It does not evict IDs, frames, or any retained ledger
entry.

This is a private proposed budget-accounting representation only. It changes
no public/persisted JSON, DTO, schema, API, or source encoding. Candidate
variable-length source fields continue to use their full canonical JSON
encoding and are checked against the proposed limit as already specified.
No heap/RSS/process-memory bound is implied. This fixed-width rule does not
approve the budget or any overflow policy.

## Required focused tests

Future separately authorized implementation must include these focused cases;
no test or runtime evidence is claimed here:

1. **Atomic candidate rejection:** for a terminal over-budget candidate,
   assert no frame, event ID, ordinal, window, standing, source timestamp,
   count, or candidate-derived data commits; return only the generic typed
   unavailable result with zero delta and no watermark advance. Verify the
   internal safe `terminalRetentionFailure`/stop marker is committed
   separately.
2. **Stop-before-join lifecycle:** hold the worker at a boundary after its
   current port read has returned. Assert the stop marker bars every future
   read start before any join wait; then let the worker finish and close
   `workerDone`, join outside the mutex, drain refs/operations/notifiers, and
   only then release capacity. Verify no terminal candidate payload is
   disclosed and a timeout leaves capacity counted.
3. **Exact-budget control transition:** with a canonical retained image of
   exactly `1<<20` bytes, apply only the marker/control-state transition and
   assert encoded length does not grow, no ledger entry is evicted, and the
   resulting image remains within the same proposed cap. Verify fixed-width
   enum/boolean and `uint64` counter/generation encodings in both states.

## Boundary

This correction repairs only the terminal-data/stop ordering and proposed
control-image sizing raised by the revision 2 review. It approves no source,
tests, public interface, runtime, cap, retention policy, overflow policy,
manager/V4 wiring, or T09 settlement. A different independent reviewer must
review this new record against frozen revision 6, its addendum, correction
revision 2, both applicable reviews, and the original assignment before any
source assignment. No gates, compiler, scanner, Git action, Graft build, or
external write is authorized here.
