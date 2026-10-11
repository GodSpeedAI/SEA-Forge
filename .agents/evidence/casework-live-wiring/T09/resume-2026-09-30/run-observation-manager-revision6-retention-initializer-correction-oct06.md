# Private run-observation manager revision 6 — initializer and retention correction

Date: 2026-10-06  
Status: DOCONLY normative correction. No source, tests, public interface, or runtime work is released.

## Scope and authority

Read with the original manager assignment `run-observation-manager-proposal-original-assignment-oct06.md`, frozen proposal revision 6 SHA-256 `60498c53f9cf953ed59015dfa338d592b89a5a9f8652483a511ce36ff7a9b99b`, addendum SHA-256 `9439cbfe8cb30fdf7b1beb41c617ff031ab383296b220317a8b7f529ce3b859a`, and its independent review. Preserve all those records unchanged. This correction supersedes only the initializer `Ready` disposition wording in addendum §2 and the unresolved retention-overflow lifecycle in addendum §4 / revision 6 “Exact ledger, read failures, version capture, and deltas” and its outcome table. All other original-assignment and revision 6/addendum clauses remain in force.

The cohort cap of 16, at most 128 lease-to-run attachments, combined serialized retained-value budget of exactly `1<<20` bytes per poller, and every overflow behavior below are **proposals requiring independent review and exact operator approval before implementation**. This note changes no public API, DTO, schema, manager export, or source behavior.

## 1. Exact private attach result contract

Use this disposition contract to resolve the contradictory `Ready` comment and prose in the prior addendum. Names remain private proposal details.

```go
type attachResult struct {
    Disposition    attachDisposition
    Ref            *runRef             // nonnil iff this transition acquired a lease ref
    CurrentVersion *pollerVersion      // nonnil only for current; exact immutable version
    Initializer    *initializerToken   // nonnil only for newInitializer
    Ready          <-chan struct{}     // nonnil for current/sharedInitializer/newInitializer
}
```

For `current`, `Ref`, `CurrentVersion`, and an already-closed `Ready` are nonnil; `Initializer` is nil. The version pointer names the exact immutable current generation acquired by this transition. For `sharedInitializer`, `Ref` and that initializer generation's `Ready` are nonnil; `CurrentVersion` and `Initializer` are nil. For `newInitializer`, `Ref`, its `Ready`, and the generation-bound `Initializer` token are nonnil; `CurrentVersion` is nil until publication. `pollerCapacity`, `cohortCapacity`, `draining`, `identityMismatch`, and `invalid` return no ref, version, ready channel, or token. A successful attach owns exactly the returned ref and releases it exactly once on rollback/detach. Only the manager-owned initializer token can publish a new generation; a launching caller's rollback cannot revoke shared work. No network call, wait, callback, cancellation, or join occurs under the manager mutex.

## 2. Proposed retention-overflow lifecycle

Candidate validation happens before any state publication. If the canonical current retained-value image would exceed the proposed per-poller budget, atomically reject the entire candidate: accept no candidate frame, ID, ordinal, window, standing, timestamp, or terminal control value. Do not disclose candidate-derived details in returned DTOs, deltas, error messages, logs visible to clients, or other caller-facing output. Every result is typed and safe; its text contains no path, payload, run metadata, or candidate fields. `Next` returns a zero delta and does not advance its watermark. Never serve an older immutable version as current after a rejected candidate.

For a validated **nonterminal** over-budget candidate, mark that poller `retentionUnavailable`, wake attached leases, and return the typed `ErrPollerRetentionUnavailable`. Keep the lease attached and its cohort slot counted. Polling may continue. Recovery requires a later complete validated candidate that fits **with the entire existing ID/ordinal ledger**; commit its version and any new ledger entries atomically, then clear the unavailable state and wake watchers. Do not reset, trim, or evict the ledger to make room. Until recovery, no old version is presented as the requested current observation.

For a validated **terminal** over-budget candidate, return only typed `ErrPollerRetentionUnavailable` with a zero delta; disclose no candidate frame, terminal standing, counts, timestamp, or other candidate payload. Mark terminal retention failure internally and stop scheduling future polls only after the actual source read and client retirement have returned and the poller worker has joined. Affected leases detach and drain through the normal ownership protocol. Keep lease and poller capacity counted until all refs, in-flight `Next` operations, per-target notifier refs, actual reads/client retirements, and worker joins owned by that teardown have completed. A timeout leaves entries draining and counted. A departing watcher must not cancel work still required by another authorized watcher; shared-work cancellation and join ownership remain exactly as in revision 6/addendum. No reset to an older buffer or ledger is allowed.

These are private proposed states, not a public distinction carrying terminal candidate data. They do not change the addendum rule that `ErrPollerReadUnavailable` is transient. They preserve the separate terminal auth/context/cancel/stop behavior and join-before-capacity-release requirement.

## 3. Required test matrix corrections

These are future requirements only; no tests ran here.

1. **Attach-result matrix:** assert `current` returns ref + exact current immutable version + closed ready channel and no token; `sharedInitializer` returns ref + ready channel and no version/token; `newInitializer` returns ref + ready channel + only its new token and no version. Every capacity/draining/mismatch/invalid result has all ownership fields nil. Verify each acquired ref is released once.
2. **Nonterminal retention overflow and recovery:** at the proposed byte boundary, reject the whole candidate with `ErrPollerRetentionUnavailable`, zero delta, unchanged watermark/ledger/ordinal/current-version commitment, and no candidate disclosure. Keep the lease and slot attached. Then supply a complete candidate that fits with the entire preserved ledger and verify one atomic recovery and later `Next` on the same lease. Include a nonfitting recovery candidate and prove it remains unavailable without partial identity acceptance or old-version substitution.
3. **Terminal retention overflow:** keep a source read blocked while its validated terminal candidate is over budget. Prove no terminal standing/frame/payload escapes; callers receive only the safe typed unavailable result and zero delta; no next poll begins and no slot/ref is released before actual read return, client retirement, and worker join. Prove teardown drains notifier and operation refs, preserves shared-needed work ownership, and remains counted on drain timeout.
4. **Safe errors:** ensure typed retention errors and terminal control paths do not include filesystem paths, run metadata, raw payload, or any field from the rejected candidate.

## Unchanged decisions and boundary

This correction changes only the two identified ambiguities. It retains exact opaque IDs and first-observation ordinals, captured immutable `Next` versions, nonterminal full-ledger recovery, no old buffer as current, the read-unavailable transient rule, manager-owned initializer publication, shared watcher ownership, no callbacks or waits under lock, join-before-removal, and zero/no-watermark error results. The exact proposed 16 cohort slots, 128 attachments, combined 1 MiB retained-value budget, and associated overflow policy remain unapproved. The accepted poller/admission ownership, counts, selector, auth/present-context checks, 1,024-frame windows, and original test-first requirements are unchanged. No source, test, public contract, runtime claim, or T09 settlement is approved.

Have a different independent critic review this correction against the frozen proposal, addendum, their reviews, and the original assignment before any source assignment. No tests, compiler, scanner, Graft build, Git, or network action is authorized by this document.
