# Independent review: private run-observation manager revision 6 addendum

Date: 2026-10-06  
Reviewed artifact: `run-observation-manager-concrete-proposal-revision6-addendum-oct06.md`, SHA-256 `9439cbfe8cb30fdf7b1beb41c617ff031ab383296b220317a8b7f529ce3b859a`.  
Disposition: **REJECT as complete; require a small new DOCONLY correction.** Preserve this addendum unchanged. The captured-ledger, initializer lifetime, and memory-scope repairs are substantially correct. Two local ambiguities remain: the initializer result comment contradicts its prose, and retention overflow does not define terminal versus recoverable lease behavior.

## Reviewed inputs

I compared the full addendum against the original manager proposal repair assignment, the frozen revision 6 proposal, and the independent revision 6 review. The addendum hash matches the builder's reported identity. No source, tests, gates, or runtime evidence were examined or run for this review.

## Findings

### 1. `Ready` result comment conflicts with the specified new-initializer result

The `attachResult` declaration says `Ready` is nonnil for `current/sharedInitializer`. The next paragraph says `newInitializer` carries a reference, the new token, **and a ready channel**. A builder could implement one or the other, which makes the exact private attach result ambiguous.

**Repair:** Make the comment state that `Ready` is nonnil for `sharedInitializer` and `newInitializer`; for `current`, either require a closed ready channel or state that it is nil and does not need waiting. Keep the subsequent disposition contract consistent with that choice. No public API change is implied.

### 2. Retention overflow leaves terminal disposition unresolved

Section 4 explicitly leaves retention overflow's terminal-versus-recoverable disposition to a later decision. Revision 6's prior policy rejects an over-budget candidate atomically, marks retention unavailable, never serves the old version as current, and allows recovery when a later complete nonterminal candidate fits. It separately says a terminal state in an unfit candidate is not disclosed and the worker stops only after the read returns and the worker joins. The addendum does not say whether a lease remains attached after an unfit nonterminal candidate, or what the lease receives and when it detaches for an unfit terminal candidate.

The distinction matters to liveness and ownership: treating all retention errors as terminal makes the documented later recovery unreachable for the same lease; treating all as transient can leave a terminal run attached to a worker that will never publish another version. The original assignment requires actual cancellation and join before capacity release.

**Repair direction:** Preserve the proposed policy and its approval hold, but state the lifecycle precisely. For a validated nonterminal over-budget candidate, atomically reject it, return typed retention-unavailable with zero delta/no watermark advance, keep eligible leases attached, and allow a later complete candidate that fits the entire retained ledger to recover. For a validated terminal over-budget candidate, disclose neither it nor the old version as current; stop further reads only after the actual read/client retirement returns and the worker joins, deliver the designated typed retention-unavailable result to affected leases, and detach/drain them. Do not release lease/poller capacity until references and actual owned work have joined. Keep both overflow transitions **proposed and subject to operator approval before source work**.

## Findings closed by this addendum

- `Next` now captures the immutable version, prior watermark, and copied exact-ID ledger under one lock boundary, and gap assembly filters to `P < FirstOrdinal <= captured HighestObservedOrdinal` using only those captures.
- Initializer publication is assigned to a manager-owned generation token rather than the initiating `Prepare` lifetime; rollback removes only that caller's ref, keeps shared initialization alive for surviving waiters, and cancels/joins it only after no eligible watcher remains.
- The 1 MiB proposal is explicitly limited to the canonical current retained-value image per poller. Old referenced versions, copied ledgers, and candidate/DTO/delta/transient storage are outside that image; the 16/128 proposal bounds counts only, not bytes or heap.
- `ErrPollerReadUnavailable` is explicitly transient and preserves the same lease for later recovery, while auth/context/cancel/stop paths are terminal.

These clauses address the principal revision 6 review findings. The proposal-only cohort and serialized-value policies remain unapproved; this review grants no source or runtime release.

## Verdict

Preserve revision 6 and this addendum immutably. Write a short correction document that fixes the `Ready` disposition contract and specifies the two retention-overflow transitions and join-before-release lifecycle above. Have a different independent critic review that correction before any source assignment. No implementation, public manager/V4 wiring, test execution, or T09 settlement is approved.

No tests, compiler, scanner, Git, Graft build, or network action was run.
