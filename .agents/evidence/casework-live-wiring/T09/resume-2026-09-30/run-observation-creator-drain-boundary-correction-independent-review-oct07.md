# Independent review: creator/drain correction — 2026-10-07

**Verdict: REJECT this proposal correction pending one narrow admission check.** The new documents resolve the earlier bridge-lifetime, detach-result, and drain-timeout gaps. They still do not prevent a caller-canceled Prepare from attaching work if its held list callback returns success after cancellation.

## Evidence reviewed

The complete original correction assignment, supplement, and context-scope erratum were read. Their hashes are recorded in the accompanying immutable review assignment. I also reread the complete original proposal assignment, proposal, and prior rejection. The correction documents retain their exact expected hashes; no rewrite or overwrite was observed.

## Resolved prior findings

- **Bridge lifetime:** The supplement now says each admitted creator that has created a local derived request context cancels only that local context after its final list/readiness use, joins its bridge, then closes its exact `creatorDone` and balances the manager operation registration. This is before its own rollback-drain wait. The erratum correctly excludes pre-context refusals, which have no bridge. The shared worker context and eligible lease remain independent.
- **Typed detach result and B survival:** The held-list detach case requires the designated typed cancellation/stop/unavailable error, zero DTO, and nil lease, so an ordinary successful unavailable-list result cannot satisfy it. B remains live while A is detached; B's successful empty-list Prepare must return with its creator completion closed. Source review will still need to verify bridge JOIN precedes that closure.
- **Stop/detach admission race:** The proposal now rechecks manager stopping, lease draining, and exact cohort membership under the mutex after list return and before attachment/reservation. A Stop after reservation still requires the creator to launch the complete owned batch exactly once.
- **Timeout truth:** Stop and Detach timeout preserve exact cohort/poller capacity and reverse ownership. The documents prohibit fabricating `creatorDone`, `drainDone`, or `stopDone` completion and require actual list return, bridge JOIN, creator finish, and worker return/JOIN before a later retry can release ownership.
- **Fixture scope:** The four exact-lease finish closure updates and two held-list cases are bounded; the nine existing assertions, frozen manager fixture, and six primitives remain protected. No new enum, token, counter, callback field, API, schema, or public boundary is proposed.

## Remaining material gap: request-context cancellation after list admission

The prior rejection explicitly required that the production boundary not assume a list implementation always returns an error merely because its context was canceled. The correction's post-list rule instead rechecks only `m.stopping`, `lease.draining`, and exact cohort membership. It does not require the local Prepare/request context to remain live before reservation.

Therefore, if the caller cancels while the real list call is held and the callback returns a successful list despite cancellation, all three stated manager/lease predicates can still pass. The creator may reserve/attach a poller after caller cancellation. The bridge's cancellation of the local context alone does not establish that the caller-canceled Prepare is rejected before attachment.

**Smallest correction:** make the post-list admission decision also reject a canceled request/local Prepare context before any reservation or attachment, returning the contract's typed cancellation error with zero DTO and nil lease and performing owned rollback. State how this context check is ordered with the mutex-protected manager/lease checks so cancellation cannot be ignored at the admission boundary. Add or extend a deterministic held-list test where the caller cancels and the callback is explicitly released with a successful list; assert no read/attachment and the typed zero/nil result. Keep this separate from lease `leaseDone` cancellation and from successful unavailable-list A.

## Scope and limits

This is a document-only proposal rejection. The corrections to successful local bridge shutdown, exact-lease creator completion, timeout retention, typed detach failure, post-Stop/detach admission, and the bounded fixture matrix are accepted as resolved design clauses. No source or test implementation is approved, and no runtime or compiler gate was run. A fresh immutable supplement should address the remaining caller-cancellation race before TDD source release.
