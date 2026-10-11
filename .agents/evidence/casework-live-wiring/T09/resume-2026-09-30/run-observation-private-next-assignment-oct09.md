# Private Next transaction and lifecycle assignment

Root orchestration decision, 2026-10-09. Source release is HELD until root
confirms checkpoint 9efd039 was published and its actual captures compared.
This document is an implementation specification, not runtime evidence.

## Authority and bounded scope

Read root and nearest instructions, the current canonical status snapshot,
the governing casework spec, manager concrete proposal revision6/addendum,
retention initializer correction revision3 and its independent review, private
policy operator approval, and next-integration-root-decisions-oct08.md in this
directory. Earlier proposal-only labels are superseded only by the recorded
specific policy approval. Preserve all existing authority and admission paths.

The source scope is exactly manager.go, poller_worker.go, and new private
run_observation_next.go / run_observation_next_test.go under the Go server
package. Preserve every existing assertion and the accepted pure per-run
delta helper. No public SSE/UI/wire/schema, dependency, persisted storage,
authority, retention budget, polling/admission, or unrelated formatting change.
Read each file before editing and use native apply_patch for persistent writes.

## Required transaction

Provide private lease next(ctx) returning a private aggregate with exact
case/as-of cursor, observation time, nonnil runs and window gaps. Use existing
private deltas and scalar watermarks; sort runs/gaps by exact run-ID bytes and
retain captured frame source order. Do not invent source continuity.

Each call uses its own context. Under manager.mu admit at most one Next for
an exact active, non-draining lease and register its operation before releasing
the mutex. No operation or per-target notifier Add after drain begins. Wait
outside locks on capacity-one wake (never closed), caller cancellation and
leaseDone. A successful Prepare seeds one coalesced wake, including complete
empty and successful terminal cohorts; unavailable list fails typed without
waiting. This permits the initial call without replaying hydrated-away frames;
subsequent calls require publication/wake or cancellation.

Capture all exact attached entry pointers, immutable current pointers, copied
SeenByID ledgers and prior watermarks together under the mutex. Assemble only
those captured values outside it. Reuse current authorization/present-context
checks outside the mutex before and immediately before disclosure. Commit all
candidate watermarks together only under final exact active-cohort/token,
manager-entry, forward/reverse attachment, non-stop/non-drain and caller-context
checks. Do not require current-pointer equality: publication during assembly
belongs to a subsequent call and must retain a coalesced wake. Never recapture
or retry the aggregate. Any run failure returns zero aggregate/no advancement.

Read-unavailable and nonterminal retention-unavailable retain refs/capacity and
may recover only after a later complete fitting candidate. Successful terminal
retainedCurrent remains deliverable after workerDone. Terminal retention stop,
auth/session/cursor/context failure, cancellation, detach and Stop schedule the
single drain owner. Release the call's own operation before any drain wait to
avoid self-deadlock. Drain joins creator, operations, per-target notifiers and
actual owned workers outside mu before removing counted capacity; timeouts do
not free it. Preserve work still required by a surviving authorized lease.

Publishers acquire exact eligible target notifier references under mu, then
send nonblocking and balance Done outside mu even when sends coalesce. No
callbacks, auth/guard calls, sends, cancel, waits, reads or joins under mu.

## Deterministic proof and review loop

Production next delegates to one private transaction helper with a typed pure
projector function argument fixed to the existing assembler. Tests may wrap
that same assembler with a barrier after capture and before projection. No
manager-global mutable hook, alternate lifecycle implementation or bypass.

First submit tightly scoped compiling TDD fixtures and a minimal typed-error
stub if required for the missing private method. Root owns actual behavioral
RED and compiler token. Freeze after source review; do not implement before
root accepts expected RED. A fresh builder then implements the full unit.

Prove complete-empty versus unavailable and fresh context after Prepare;
multi-run all-or-nothing/no candidate commit and fitting recovery; exact
captured generation despite publication during projection and next-call wake;
detach between capture and commit with counted actual JOIN; shared survivor;
single-call rejection; successful terminal and terminal-retention disposition;
auth/session/cursor/cancellation/Stop races; notifier coalescing/ownership;
initial pre-hydration no replay and distinct lease watermarks. Prefer barriers
and exact counters over sleeps; never manipulate watermarks to simulate Next.

Independent critic receives this FULL assignment plus resulting diff, verifies
all material deviations and source/runtime evidence, and cannot approve on
insufficient evidence. Rejection requires a fresh builder and re-review.
Required actual gates: focused manager/Next race, just casework-go-check, and
full-module Go race. Only root's granted compiler owner runs gates; actual
RAM/swap/no competing heavy process preflight applies. Immediately after each
JOIN archive all six actual captures and root losslessly compares before any
next gate. No T09 settlement or public readiness follows from private approval.
