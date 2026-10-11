# Private Next integration: root decisions

Date: 2026-10-08. Architecture/scoping clarification; no source release.
Read the full revision6 proposal, addendum, retention initializer correction2
and superseding correction3, plus integration recon856fd9bd. Preserve public
contracts and the existing private retention/admission limits.

## List outcome and request lifetime

Add one private lease list-state field using the existing contract list-state
enum. Set it under manager.mu from the validated Prepare list outcome before
returning the lease. Complete-empty and unavailable are distinct even when
both have zero pollers. Never infer list failure from an empty map. Next on an
unavailable-list lease returns the existing safe typed run-list unavailable
error and zero aggregate, without candidate watermark commit. A complete-empty
lease is valid and returns a nonnil empty aggregate after normal authorization
and context checks.

Each Next uses its own caller context. Successful Prepare cancels its local
request context before return, so its prepareDone cannot govern Next. Reuse
authorization/guard logic outside manager.mu and exact active lease/token/
manager-entry/forward-ref/reverse-ref validation under it, rather than reusing
the Prepare-specific disclosure predicate verbatim.

## Error disposition and drain

Nonterminal retainedRetentionUnavailable preserves the existing worker's
recovery policy explicitly described by correction2: zero aggregate, typed
retention unavailable, no watermark advance, same lease/ref/slot retained.
Recover only after a later complete fitting candidate with the preserved full
identity ledger; never disclose an older version as current. This is an
explicit retention decision, not an inference from transient read failure.
No new cap or broader availability guarantee is introduced.

Transient read-unavailable also keeps attachments and permits a later worker
publication/wake. Successful terminal retainedCurrent remains deliverable even
after workerDone. Terminal retention-failure/stop-scheduling, auth/session/
cursor/context failure, explicit cancellation and manager Stop produce zero
aggregate/no commit and enter the existing terminal drain ownership path.

A terminal Next must not wait for an operation reference that it still owns.
Mark draining and schedule the existing asynchronous single drain owner, then
return through deferred release of Next's token/operation reference. The drain
owner joins creator, operations, notifiers and actual workers outside the mutex
before removing capacity. Error return alone does not prove completed teardown.
If the drain times out, keep counted ownership until actual JOIN. Never cancel
shared work still required by another eligible authorized lease.

## Captures, wake ownership and commit

Initialize each lease watermark from the exact accepted initial retained value
under the capture lock, before DTO hydration can omit frames. Admit at most one
Next per lease and register operations.Add under the same mutex that marks it
draining; no Add after draining. Use a capacity-one wake channel that is never
closed. Register each notifier target under the same mutex before unlocking;
send nonblocking and balance Done outside it. Drain waits outside the mutex.

Capture retained pointers, copied exact ledgers and prior scalar watermarks
atomically under manager.mu. Assemble only those values outside it with the
pure assembler. After final auth/guard outside it, commit only if the exact
lease/token and both sides of each captured reference still match under the
mutex. Do not require the current version pointer still equal the captured
pointer: publication during assembly belongs to the next call and retains a
coalesced wake. No global recapture/retry. Any run error discards every proposed
watermark and the entire aggregate. Sort runs/gaps by exact run-ID bytes only;
preserve each captured frame's source order.

## Bounded implementation sequence

Independent review of these decisions and acceptance of the pure assembler
precede a separate TDD source grant. That grant is limited to manager lease/
Prepare/drain fields, worker publication notifications, a new private Next file
and a new Next test file; existing lifecycle and pure-delta assertions remain.
Prove list distinction, initial pre-hydration watermark, independent leases,
transient/retention recovery, single-call rejection, publication during assembly,
terminal success, auth/cursor/context/Stop/detach races and counted teardown.
No public SSE/UI/schema, dependency, persisted-storage or authority change is
part of this private integration. Runtime claims require actual independently
verified gates with serialized compiler ownership and lossless captures.
