# Independent architecture review: private run-observation manager revision 6

Date: 2026-10-06  
Reviewed artifact: `run-observation-manager-concrete-proposal-revision6-oct06.md`, SHA-256 `60498c53f9cf953ed59015dfa338d592b89a5a9f8652483a511ce36ff7a9b99b`.  
Disposition: **REJECT as a complete implementation blueprint pending a narrow DOCONLY addendum.** The revision closes the identified revision 5 findings and preserves the no-release boundary. The remaining issues concern consistency of captured delta state, exact initializer ownership, transient memory accounting, and transient read-error lifecycle. No source assignment follows.

## Reviewed inputs

I read the complete original manager proposal assignment, revision 2 and revision 3 assignments, complete revision 5 proposal, complete revision 5 independent review, and complete frozen revision 6. Revision 6's SHA-256 matches the builder's reported hash. This review is limited to document evidence; it makes no implementation or runtime claim.

## Findings requiring addendum

### 1. A `Next` delta can mix a captured version with a newer mutable ledger

Revision 6 says `Next` captures one immutable `pollerVersion` per attached key and its watermarks under the manager lock (state/ownership and delta sections). `pollerVersion` contains `HighestObservedOrdinal` and the current `Frames`, but not the exact `SeenByID` ledger. The later gap rule says to report exact IDs above the prior watermark that are absent from the captured window. The only ledger in the proposed structs is mutable `pollerEntry.seenByID`.

If a poller publishes another version after `Next` captures its older version but before gap assembly, reading the current ledger can introduce IDs and ordinals newer than the captured version into the older delta. The document does not state an upper bound tying these IDs to the captured version's `HighestObservedOrdinal`. That violates the one-version-set/as-of guarantee and can report an identity the captured window could not have observed.

**Minimal repair:** In the captured-version-set lock section, copy `seenByID` (or an immutable ledger snapshot/reference) together with each version and watermark. For gap construction, include only `previousWatermark < firstOrdinal <= capturedVersion.HighestObservedOrdinal`, and select absent IDs against that same captured window. State that ledger ordinals are immutable after first assignment. A captured ledger snapshot is not necessary if the bounded ordinal filter and exact IDs are copied under the same lock; either approach must make the capture boundary explicit.

### 2. The single-flight initializer's owner and handoff are not represented by a complete result type

The API declares `attach(...) (attachResult, error)` and the disposition enum distinguishes `newInitializer` from `sharedInitializer`, but `attachResult` is not defined. `pollerEntry` has `ready` but no initializer token/owner field. Prose says “the initializer owner alone may publish,” yet does not define how that right is represented or what happens if the owning lease begins rollback while other authorized leases are waiting on the shared initialization.

This leaves the exact attach contract and the root-required initializer owner/ref handoff unresolved. A waiter must neither publish the owner's result nor be stranded if the owner detaches while another lease still needs the read.

**Minimal repair:** Define `attachResult` fields for disposition, acquired ref, and (only for the owner) an opaque initializer token containing key/generation/owner identity/completion. State that publication authority belongs to a manager-owned initializer generation rather than the continued lifetime of the initiating watcher, or specify an atomic owner transfer to a surviving eligible lease. Define completion/cancel behavior for owner rollback with waiters and ensure the shared read continues only while an eligible attachment needs it. Preserve reads, waits, cancellation, and joins outside the lock.

### 3. The proposed JSON budget is not scoped against retained old versions and candidate copies

The combined 1 MiB image explicitly includes the current retained ledger/window/auxiliary safe values and excludes decoded uncommitted responses and transport buffers. `Next` can retain pointers to immutable old versions while pollers publish new ones; each concurrent lease call can capture up to eight versions. Initial/Next assembly also creates copied DTO/delta frames and candidate watermarks off-lock. These bytes are not part of the proposed per-entry image, but the document does not identify them in its memory-accounting exclusions or quantify the multiplicity permitted by the proposed 16 leases and 128 attachments.

This does not invalidate the deliberately limited serialized-current-state policy, but a reader could mistake its “16 MiB retained” arithmetic for covering all simultaneously referenced versions or working copies.

**Minimal repair:** State explicitly that the 1 MiB policy bounds only the canonical current retained value image per poller. Immutable generations retained by in-flight `Next` calls, DTO/delta working copies, and initial assembly buffers are outside it. If useful, bound their count from the proposed lease/attachment and one-`Next`-per-lease constraints, while making clear that JSON length is not a heap/RSS bound and no whole-process memory/OOM guarantee follows. Do not add a new byte policy without review and operator approval.

### 4. Clarify transient read failure versus lease-terminal unavailable errors

Revision 6 correctly specifies `ErrPollerReadUnavailable` with zero delta/no watermark advance and says a later complete read may recover. It also says after a “terminal auth/context/unavailable error” the owner detaches and drains, while `Next` lists poller read/retention unavailable among typed errors. It is not explicit enough whether a failed poll read is terminal to the lease. If callers must detach on every unavailable result, the stated same-lease recovery path cannot occur.

**Minimal repair:** State that `ErrPollerReadUnavailable` is a transient per-poller result: keep the lease attached and counted, permit a later successful read to wake it, and allow a later `Next` to recover. State which auth, guard, retention, or manager errors are terminal and trigger detach/drain. Keep every error result zero/no-advance.

## Revision 5 findings and preserved constraints

Revision 6 does repair the reviewed revision 5 omissions: the separate proposed 16-cohort/128-attachment bound is explicit; `ErrPollerReadUnavailable` has zero/no-advance/recovery behavior; initial and delta timestamps are tied to the captured version and `firstAcceptedAt` is removed; there is one retained frame representation with deep-copied optional pointers; and per-target notifier references, Add/Done, drain-gated Wait, nonclosing wake, separate once-closed `leaseDone`, and waits outside the mutex are specified. These conclusions match the reviewed revision 6 text and do not approve the proposed limits.

The exact opaque entry-lifetime ID ledger, first-observation ordinals, initial pre-pruning watermark, immutable captured `Next` versions, source-order delivery, no global retry, shared physical admission ownership, session/bearer behavior, as-of guard, exact counts/exhaustion interpretation, and join-before-capacity release remain stated. The 16-cohort limit, 1 MiB combined retained-value budget and its overflow/terminal/recovery behavior remain explicitly proposed pending independent review and operator approval. The existing 32 MiB source-line cap remains an unimplemented prerequisite; no heap, RSS, or process-memory guarantee is claimed.

## Verdict

Preserve revision 6 unchanged. A short, new normative addendum can close these four contract gaps without rewriting the full proposal. Have an independent critic review that addendum against the frozen revision 6 and prior assignments before any separate source assignment. No manager source, caller, public contract, runtime gate, implementation policy, or T09 completion is approved by this review.

No tests, compiler, scanner, Git, Graft build, or network action was run.
