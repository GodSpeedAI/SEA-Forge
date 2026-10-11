# Private run-observation manager — revision 6 addendum

Date: 2026-10-06  
Status: DOCONLY normative repair to revision 6, SHA-256
`60498c53f9cf953ed59015dfa338d592b89a5a9f8652483a511ce36ff7a9b99b`.
Revision 6 and its independent review remain immutable. This addendum is a
narrow replacement for the clauses named below; all other revision 6 clauses
remain in force. No source, caller, public interface, test, or runtime work is
released.

## Superseded revision 6 clauses

This addendum supersedes only:

1. The `Next` captured-version/gap-assembly wording and the related gap rule in
   **State, lock, and exact ownership**.
2. The undefined `attachResult` contract and initializer lifetime wording in
   **Private API and types** and **State, lock, and exact ownership**.
3. The scope and accounting interpretation of **Bounds and admission**'s
   proposed combined serialized JSON budget, including its derived memory
   figures.
4. The classification of `ErrPollerReadUnavailable` and the generic
   “terminal auth/context/unavailable error” cleanup wording in **Private API
   and types** and **State, lock, and exact ownership**.
5. The matching test-matrix entries listed below. All other constraints,
   including the no-release boundary, remain unchanged.

## 1. One captured version and ledger boundary per `Next`

Under the manager mutex, `Next` captures one immutable `pollerVersion`, the
lease's prior watermark, and a copied entry ledger for every attached key.
The ledger copy contains exact `(EventID, FirstOrdinal)` pairs. First ordinals
are immutable after assignment. The capture is one atomic logical boundary;
after unlocking, all run deltas, watermarks, and gaps are assembled solely
from those captured values and the request's own lease state. Gap assembly
must not inspect the mutable entry ledger or current entry version off-lock.

For a captured version with `HighestObservedOrdinal = H` and prior watermark
`P`, only copied ledger identities satisfying `P < FirstOrdinal <= H` may
contribute to that delta's observed-ID gap. Compare those IDs against the
captured version's frame window, never a subsequently published window. An
identity assigned after capture (`FirstOrdinal > H`) belongs to a later
`Next`, even if its poller publishes before this call finishes assembly.
Source order, captured timestamps, and the existing no-global-retry behavior
remain unchanged. Captured copies and references are read-only after unlock.

## 2. Explicit initializer result and manager-owned publication token

`attachResult` is defined as the private result of one mutex-protected attach
transition (field names are illustrative private names, not public API):

```go
type attachResult struct {
    Disposition attachDisposition
    Ref         *runRef // nonnil only when this transition acquired a lease ref
    Initializer *initializerToken // nonnil only for newInitializer
    Ready       <-chan struct{} // nonnil for current/sharedInitializer
}
type initializerToken struct {
    key        pollerKey
    generation uint64
    owner      uint64 // manager-owned initializer generation identity
}
```

The token is a capability for one entry generation, owned by the manager's
single-flight record, not by the lifetime of the `Prepare` call that first
observed the entry. A manager-owned initializer worker publishes exactly once
for the matching key/generation/token. The launching caller may wait on its
completion, but it does not own publication and its rollback cannot revoke
the token or discard a result needed by another authorized attachment.
Stale-generation completion is ignored and cannot overwrite a newer entry.

Every successful attach result names its disposition and exact acquired
reference. `current` carries the current exact-key run reference;
`sharedInitializer` carries a reference plus the existing generation's ready
channel; `newInitializer` carries a reference plus the new token and ready
channel. Capacity, draining, identity mismatch, and invalid results acquire
no reference and carry no initializer token. The caller must release exactly
the reference returned by this transition on its own rollback/detach path.

On initiating-caller rollback, that caller's reference is removed. If another
authorized watcher remains attached or is waiting on this generation, the
manager-owned initializer continues and publishes once for those watchers; no
second initialization starts and no extra reference is fabricated. If no
eligible watcher remains, cancel the initialization and drain/join its actual
read and retirement before removing the entry or releasing its poller slot.
An initializer waiter whose own `Prepare` fails rolls back only its own
reference; it neither cancels work needed by other watchers nor assumes the
initializer's caller remains alive. All waits, reads, cancellation, callbacks,
and joins remain outside the manager mutex. The original caller's
`Prepare` failure still returns its original typed error, zero wrapper, and
nil lease after its internal rollback completes; it must not depend on a
caller-visible lease for cleanup.

## 3. Serialized current-state budget scope

The revision 6 proposed `1<<20` byte budget applies only to the canonical
serialized image of the **current retained value state of one poller**. It
does not account for immutable older versions still referenced by in-flight
`Next` calls; copied ledger snapshots; captured version sets; initial DTO,
delta, candidate, or watermark working copies; decoded candidate responses;
transport buffers; or other transient heap. Those exclusions apply even when
the 16-cohort/128-attachment proposal and one-`Next`-per-lease rule are
considered. The proposed cohort/attachment limits bound counts only; they do
not convert those copies into a byte or heap bound.

Consequently, the revision 6 serialized-image multiplication and initial
assembly arithmetic describe only their named serialized images. They do not
bound simultaneously referenced old versions, copied ledgers, candidate/DTO
buffers, Go heap, RSS, or whole-process memory. No multiplicity estimate or
whole-heap/OOM guarantee is implied. The per-poller 1 MiB combined retained
image, 16-cohort/128-attachment policy, and all associated overflow/recovery
rules remain **proposed and require operator approval before source work**.
This clarification creates no additional budget, cap, or public field/API.

## 4. Transient read failure and terminal detach

`ErrPollerReadUnavailable` is transient for the affected poller and current
`Next`: return a zero delta, do not advance that lease's watermark, retain its
attachment and cohort slot, and permit the poller to continue for other
authorized watchers. A later accepted successful read publishes a new
captured version and wakes eligible watchers; a later `Next` on the same lease
may then recover. A failed read does not publish or serve the previous version
as a substitute for the failed requested observation.

Authentication/current-session failure, present-context/cursor guard failure,
explicit lease cancellation, explicit manager stop, and shutdown are terminal
for the affected lease/manager path: return zero delta with no watermark
advance, detach, and drain/join work when no other authorized watcher needs
it. A watcher leaving does not cancel a shared read still needed by another
authorized watcher. Retention overflow remains governed by the separately
proposed revision 6 policy: return typed unavailable with zero delta and no
advance, without partial commit or serving an old version as current. Its
terminal-versus-recoverable disposition must be decided explicitly with the
retention policy before implementation; it must not be inferred from the
transient read rule. The proposed retention policy and any terminal-overflow
rule require operator approval before source work. Drain timeout remains
counted and cannot release capacity.

## Required test matrix additions

Add these cases to revision 6's test-first matrix. They are requirements for a
future separately authorized source assignment, not evidence that tests ran.

1. **Captured ledger/version:** capture `H`, publish a later version and add
   IDs with ordinals above `H` before gap assembly; prove the older delta uses
   only the copied ledger and captured window. Cover `P < ordinal <= H`,
   `ordinal == P`, `ordinal == H`, and `ordinal > H`; captured IDs absent from
   the captured window are reported exactly once with exact byte identity.
2. **Initializer handoff/rollback:** owner `Prepare` rolls back while a second
   authorized watcher waits; prove one initialization, manager-owned token
   publishes once, waiter receives the result, and references balance. Also
   test waiter rollback while another watcher remains, all-watchers-leave
   cancellation and join, stale-generation completion, and initial caller
   internal rollback returning its original error/zero wrapper/nil lease.
3. **Budget accounting interpretation:** assert the canonical budget image
   includes current retained fields exactly once and excludes captured older
   versions, copied ledgers, DTO/delta/candidate/transient assembly buffers,
   and transport buffers. Confirm proposed 16/128 limits assert counts only;
   no test or documentation claims a heap bound. Boundary/overflow behavior
   remains gated on operator approval.
4. **Read error lifecycle:** fail a poll read while this lease and another
   authorized watcher are attached; assert zero delta/no watermark advance,
   both attachments remain valid, no duplicate initialization occurs, and a
   later accepted read wakes and recovers the same lease. Separately prove
   terminal auth, cursor/context, cancellation, and stop paths detach/drain;
   retention overflow follows only its explicitly approved disposition.

## Review and release boundary

This addendum repairs only the four findings in
`run-observation-manager-concrete-proposal-revision6-independent-review-oct06.md`.
Have a different independent critic review this new file against frozen
revision 6, its review, and the original assignment. A successful document
review does not approve the proposed cohort or retained-value limits, any
source/test work, public manager/V4 wiring, or T09 settlement. No gates,
compiler, scanner, Graft build, Git action, or external write is authorized by
this addendum.
