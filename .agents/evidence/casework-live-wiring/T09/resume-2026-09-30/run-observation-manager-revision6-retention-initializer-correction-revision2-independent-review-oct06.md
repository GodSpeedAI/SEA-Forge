# Independent review: manager revision 6 retention/initializer correction 2

Date: 2026-10-06  
Reviewed artifact: `run-observation-manager-revision6-retention-initializer-correction-revision2-oct06.md`, SHA-256 `3494e3de99b2f5d3fe725a85971dccd0d70cef226da257ce237cce7fba2d72e1`.  
Disposition: **REJECT pending one final DOCONLY wording correction.** This revision resolves the prior `Ready` mismatch and provides the requested overflow test detail. Its terminal-retention lifecycle still conflates rejected candidate data with an internal stop marker and puts poll suppression after worker join, which reverses the required lifecycle ordering.

## Reviewed bundle

I read the original manager repair assignment, frozen revision 6 proposal, revision 6 independent review, revision 6 addendum and its review, the prior correction draft, and this revision 2 correction. The reviewed hash matches the reported identity. This is a document review only; no code, tests, compiler, or runtime evidence was used.

## Finding: terminal retention rejection needs separate data and lifecycle state, with non-circular ordering

Section 2 first says to reject the entire over-budget candidate and accept no candidate-derived `terminal control value`. It then says to “Mark terminal retention failure internally.” That second value is necessary and is safe because it is manager-generated lifecycle state, not disclosure or acceptance of the rejected source terminal standing. State the distinction directly: reject the candidate's terminal status/payload, but publish a separate internal `terminalRetentionFailure`/stop marker.

The same paragraph says to “stop scheduling future polls only after the actual source read and client retirement have returned and the poller worker has joined.” This sequence is circular: the worker cannot be joined while it is still eligible to schedule another poll. The intended order should be explicit:

1. After a validated terminal over-budget candidate is known, atomically reject its data and set the manager-owned terminal-retention stop marker so no later read can start.
2. Let the already-running source read and client retirement finish; the worker exits without scheduling another read.
3. The teardown owner joins the actual worker and drains notifier/operation refs outside locks.
4. Keep lease and poller capacity counted until refs, operations, actual reads/client retirement, and worker join have completed; timeout remains counted.

The existing text's “no next poll begins” test intent is correct, but the prose should describe the above order. No caller receives the candidate terminal standing, frames, counts, timestamp, or payload. Affected leases receive only the safe typed unavailable result and detach/drain. A watcher leaving does not cancel shared work still needed by another authorized watcher.

The nonterminal branch is otherwise clear: reject atomically, return zero/no-advance `ErrPollerRetentionUnavailable`, keep the same lease attached, and permit recovery only on a later complete candidate fitting with the preserved full ledger. Read-poll failure remains transient and separate. The proposed cohort and retained-value caps and all overflow semantics remain unapproved pending exact operator approval.

## Revision 2 changes and carried constraints

Compared with the prior correction draft, revision 2 makes `attachResult` explicit: `current` returns the acquired ref, exact immutable current version, and closed ready channel; shared/new initializer results carry their ready channel, with the initializer token only for the new owner; capacity/draining/mismatch/invalid outcomes carry no ownership fields. It adds exact boundary and reference-balance tests. This resolves the previous comment/prose conflict.

The addendum's captured ledger/version filter, manager-owned initializer generation, same-lease transient read recovery, transient-copy budget exclusions, and original shared-work ownership remain in force. Revision 2 adds the required nonterminal/terminal overflow matrix and safe-error assertions. I found no authorization, DTO, schema, public API, or cap approval change in this correction; the 16 cohorts, 128 attachments, 1 MiB retained-image limit, and overflow policies are still explicitly proposal-only.

## Required next step

Preserve the frozen proposal, addendum, and both correction documents unchanged. Write one new, short normative correction that separates rejected candidate terminal data from the internal terminal-retention stop marker and orders “prevent future read starts” before actual completion and worker join. State that join and all reference/operation drains happen outside the manager lock, and capacity is released only after actual read/client retirement and worker join. Have a different critic review it before any source assignment. No implementation or T09 settlement is authorized by this review.

No tests, compiler, scanner, Git, Graft build, or network action was run.
