# Manager revision3 — fresh original assignment

Date: 2026-10-06. Builder: Luna cursor_manager_revision3_doc_builder,
different from revision2 author physical_admission_runtime_independent_critic.

DOCONLY. Read original manager and revision2 assignments, complete revision2
00a2fb58 and independent review7b27271f. Preserve every prior record. Write
ONE NEW complete revision3 proposal, not an erratum alone.

Repair all three blocking findings:

1. Every failed Prepare owns rollback: mark provisional lease draining, prevent
   further attachments, detach every acquired reference, cancel ONLY pollers
   with no other authorized attachment, join owned work/notification references
   outside locks, free capacity only after actual completion. No lease is
   returned on failure, so caller cleanup cannot be assumed. Cover failure after
   reservation, list, partial attach, shared initialization and concurrent Stop.
2. Root chooses exact private return contract `Prepare(ctx, req)
   (InitialCohort, CohortLease, error)`. Initial includes the approved bounded
   hydration DTO/counts; later lease.Next returns deltas only. Define exact types,
   watermark initialization after successful initial return, error variants and
   truthful initial/delta omission and generation accounting. Eventual public
   DTO/SSE mapping remains held.
3. Define private attach signature and typed outcomes for cache, shared init,
   new reservation, capacity, draining and invalid identity. Specify initializer
   owner/ref handoff, rollback, selected-unavailable versus whole-Prepare failure.
   Concrete seams follow existing `Current(sessionID)
   (auth.CurrentSessionState, bool)` and `VerifyPerspective(ctx,
   ports.ActorClaim) error`. No callback, external call or join under mutex.

Preserve accepted 16 pollers, eight selected/logical reads, one-second poll
floor, exact count equations, physical-client ownership, session/bearer rules,
present-context guard and as-of limitations. Extra 16-cohort and 1 MiB/poller
retention limits remain PROPOSALS ONLY requiring review/approval. Do not assume
their approval, implement a byte policy or claim whole-process memory safety.
Provide complete test-first matrix including cancellation and initial-return
failure races. Cite actual source anchors and all material deviations.

Read applicable instructions/Graft first. Native apply_patch for the single new
document only. No source, tests, status or debt edits; no gates, compiler,
scanner, Graft build, Git or network. Root owns sole heavy push37401. Return
path/hash. A different independent critic receives all originals/full result.
No implementation, public contract, frontier or runtime acceptance follows.
