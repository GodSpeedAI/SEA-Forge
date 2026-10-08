# Independent review: read-start testability source proposal

Date: 2026-10-07  
Verdict: **Approve the mutex-linearized read-eligibility model; return the
concrete lifecycle proposal for two bounded clarifications before fixture or
source release.** No source, fixture, compiler, test, or runtime work is
authorized by this review.

## Reviewed evidence

- Root decision `manager-read-start-testability-root-decision-oct07.md`, SHA-256 `e1b1db0e7155ff162869c54845d0251783c72e4ed2837f3961ac91e58df602ac`.
- Recon proposal `manager-read-start-testability-proposal-oct07.md`, SHA-256 `2700427c0e63f9780c47062fba2f5c7d04c07992551498b6323a015ed08790bb`.
- Prior independent architecture review `manager-read-start-testability-root-decision-independent-review-oct07.md`, SHA-256 `0e6f3385f703aaf3eeb5832b7b66414dec2d85984afafea3f2f1e4b9a830612c`.
- Original Unit 1 assignment SHA-256 `de8c9019ebdf98a43525264a32897868f05ab3346cacf7c4f013ccdae23d9916`; preregistration SHA-256 `f48df1c85ca67ce8dafa3d21b82f08a8c39aae1ec710d4ee78ce617e215869f5`.
- Revision 6 SHA-256 `60498c53f9cf953ed59015dfa338d592b89a5a9f8652483a511ce36ff7a9b99b`; addendum SHA-256 `9439cbfe8cb30fdf7b1beb41c617ff031ab383296b220317a8b7f529ce3b859a`; retention corrections 1/2/3 SHA-256 `449c73301de85794e9aec3093e82a88fdba8c60e46f207cd30454a15f1ce0524`, `3494e3de99b2f5d3fe725a85971dccd0d70cef226da257ce237cce7fba2d72e1`, `b8077e9b086f4fe3f15b9733ad3f1cb6fc6f87ca22c5a989494216c50e26d15e`.
- Failure-fixture assignment SHA-256 `d6f5f4c922c78a6c7f40215609b6199b4ecbad32c28f0b2b38002779bed6ad3a`; its independent rejection SHA-256 `dc37ae4c4170c7be0ea50a2235c55e8b09d21c230dbacda2ebf5d760e3cc4a23`.
- Initial-failure clarification SHA-256 `34750ceb5e7dc99b0a7c3755c514f6a7fbdb6fd31ac60d99833fad181ed117c2`; ref ownership decision SHA-256 `5b9888e5a7212b8d0696f790822f08e718dd03a71f51b28c2bec2ea0098f8df4`.
- Manager scaffold `run_observation_manager.go` SHA-256 `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d`; frozen original manager fixture SHA-256 `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`.
- Current pure encoder `run_observation_poller_image.go` SHA-256 `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9`; current encoder fixture hash at review time is `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256`.

No separate recon result receipt was present in the reviewed evidence directory;
the review is against the proposal file identity above and the source hashes
rechecked here.

## What the proposal gets right

The private `claimPollerReadStart` transition is a valid production
linearization point. Under the same manager mutex, it checks manager stop,
exact registered-entry identity, phase, and eligible refs. It performs no
port call, callback, cancellation, send, wait, or JOIN. Stop/final-ref release
winning that lock order prevents the claim. A prior claim remains worker-owned
through actual port return/client retirement when invoked and worker JOIN.

This is consistent with the original count contract: `ReadsAttempted` counts
actual `ReadRunTrace` invocations, not eligibility checks. It also respects
revision 6's no-I/O/no-callback/no-cancel/no-wait-under-lock rule. The
`RunTracePort` is an ordinary method call (`internal/ports/run_trace.go:6-8`),
so it cannot be invoked while holding the manager mutex. No normative source
requires physical callback entry to precede Stop in wall-clock time if the
worker won the mutex transition first. The proposal explicitly rejects that
stronger guarantee instead of silently claiming it.

The proposed files and private state are justified by the current scaffold:
`run_observation_manager.go:73-85` has refs/readiness/context/workerDone but
no phase, stop bit, current state or worker; `:125-153` leaves Prepare/Stop/
detach unwired. The frozen fixture's `:1049-1134`, `:1136-1216`, and
`:1218-1270` only cover reads after fake-port entry, cancellation/shared
ownership after entry, and JOIN/capacity after entry. Thus a worker and a
manager-protected eligibility transition must be new production seams. The
proposal correctly revises the path boundary to manager source + a worker file
+ a new failure fixture and keeps the old fixture exact.

## Clarifications required before the proposal is complete

### 1. A claim canceled before invocation still needs a complete first-read outcome

The recon proposal says the worker checks its manager-owned context after a
successful eligibility claim and that cancellation can prevent the call. It
defines code 4/phase 2 or 3 only when `claimPollerReadStart` itself refuses.
It does not define the case where the claim succeeds, Stop cancels the context,
and the context check prevents the first `ReadRunTrace` invocation.

That path must not leave `ready` unresolved or publish a fictitious read. For
an initializer with no actual first port invocation, specify the existing
stop-before-first-port-call outcome: nil current, initializer code 4, phase 2
or 3, exactly one ready resolution, zero actual `ReadsAttempted`, worker
return/`workerDone`, and capacity retained through JOIN. For a recurring poll
with an existing current image, cancellation ends scheduling without
rewriting the accepted current as a failed initializer. This follows the
root's code-4 stop clarification and actual-invocation count rule; it adds no
code, counter or serialized field.

### 2. Stop-before-claim setup must be a production-realizable state

The proposed fixture setup creates a registered initializing entry whose
worker has not yet launched, starts `stopAndDrain`, observes the actual
`manager.stopping` transition under `mu`, then calls the real `runPoller`.
This is deterministic without a fake port barrier if production permits that
reservation-to-launch interval and guarantees the reserved entry's worker is
still launched exactly once after Stop has begun. Otherwise Stop can wait
forever for `workerDone`, or the test's manually seeded entry can prove a
state that production never reaches.

Before fixture release, require the implementation assignment to name the
production reservation/launch order and guarantee: every registered worker
entry has one completion owner; Stop may mark/cancel it before launch; and the
actual worker path still executes its stopped branch, resolves ready once and
closes `workerDone`. The test must observe the real manager-stop transition,
then call the real worker method; it may not write `stopping` directly,
install a test callback, or simulate the worker's completion. If the intended
production order cannot support this, revise the deterministic case before
release rather than add a hook.

The positive eligibility test should directly exercise the same method with a
registered, referenced, running entry. The claim-first/in-flight test should
then observe actual fake-port entry before Stop, hold that call to actual
return, and prove JOIN-before-capacity-reuse. This proves both branches
without asserting an impossible wall-clock ordering between a prior claim and
callback entry.

## Identity discrepancy and scope

The recon proposal calls the encoder fixture frozen at SHA-256
`cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256`. That is
the current file hash, but the immediately preceding root release/task and
immutable exact-oracle result named the fixture hash
`8e741f7604f49bf28674e669f820ec9e696abd84ad4c60bb2a145b36123496f8` as frozen.
This review cannot determine the reason for the hash transition from the
available assignment/result records. Preserve both records; root must
identify the authorized source/result transition and state which exact file
is frozen before approving a cross-file lifecycle assignment.

The proposal's optional later extraction of `runObservationPollerKey` is
outside the read-start fix and should remain a separate, explicitly reviewed
unit. The current lifecycle source may rely on the existing private key type;
do not bundle extraction into the new worker/testability seam.

## Verdict and limits

Approve only the lock-linearized eligibility model and its refusal semantics.
Return the implementation/testability proposal for the canceled-before-call
outcome, a production-realizable reservation-to-worker-start guarantee, and
reconciliation of the exact fixture identity. These are bounded documentation
and release-record corrections; they do not require a callback hook or a
public/API change. The physical callback may enter after Stop only when its
eligibility claim won first; if the desired contract instead forbids all
wall-clock method entry after Stop begins, reject that contract because it
conflicts with no-I/O-under-lock and offer only a separately source-supported
adapter gate or a deferred proof.

No source, fixture, test, compiler, formatter, scanner, Git, or runtime action
was performed. This independent review grants no source or fixture release.
