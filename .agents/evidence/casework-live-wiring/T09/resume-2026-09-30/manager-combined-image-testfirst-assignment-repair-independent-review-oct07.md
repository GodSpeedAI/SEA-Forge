# Independent review: combined-image test-first assignment repair

Date: 2026-10-07  
Verdict: **REJECT pending two narrow protocol clarifications.**  
Scope: source/design review only. No source or fixture edits, compiler, tests, build, scanners, or Git operations were performed.

## Records reviewed

- Full `manager-combined-image-testfirst-assignment-repair-oct07.md` (SHA-256 recorded in the task as `477ee9aa...`; 230-line supplement).
- Full original bounded Unit 1 assignment, durable lifecycle preregistration, combined-image proposal and root decisions, revision 6/addendum/corrections 2–3, and response-cap erratum.
- Prior independent rejection `run_observation_manager_combined_retained_image_independent_review_oct07.md` and accepted initial-failure clarification/review.
- Current manager source and frozen fixture sources, read-only, to verify the cited seam and existing coverage.
- Exact preserved identities cited in the repair: manager `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d`, manager fixture `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`, helper `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd`, base helper fixture `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7`, policy fixture `4c70bc853ae73f4025b177b5fb505d8a456c89c41b1b13ac86faed5cba9620e7`.

## What the repair gets right

The repair fixes the prior material omissions. Its explicit branch-specific wrapper encoding proves key/current omission and raw nested JSON; outer controls remain fixed-width and range-checked. It separates the pure encoder seam and tests from later manager admission/publication work, and correctly says the oversized-key encoder test cannot prove no map entry or read. The future lifecycle fixture now covers oversized-key admission, shared initial failure, stop/detach versus ready, actual return/JOIN/capacity, and all-eight-starts-before-waiting.

Root’s clarifications are carried forward accurately: initial unavailable/invalid/unretainable selected read is successful `A` (normal unavailable DTO, successful-list counts, nonnil cohort lease, no Runs row), unlike failed Prepare; the shared poller result is shared while per-Prepare `ReadsAttempted`, derived `Exhausted`, and capture time may differ. Pointer identity plus one publisher remains subject to actual JOIN-before-reuse. Frozen files remain explicitly untouched.

Scope narrowing from the original Unit 1 assignment is appropriate for this assignment: only a new pure image encoder and its new fixture are proposed now. No manager implementation, fixture write, public/API/Next/SSE work, or runtime claim is made. The future lifecycle plan correctly remains a separate approval/release.

## Blocking gaps

### 1. Lease detach has two owners for the same ref release

In the signal ownership table, the lease-detach owner “detaches each exact ref once” under the mutex (around lines 112–115), but the waiter rule then says that after waking the waiter must “release only their exact ref.” The lifecycle fixture expects the same stop/detach-versus-ready race. As written, a waiter waking because of detach can remove a ref already removed by the detach owner, or the owner can be interpreted as only marking the refs while waiters later remove them. Those are different protocols with different conditions for canceling the shared worker and declaring drain complete.

Choose one exact ownership rule in the supplement: either the detach owner atomically removes every lease ref and awakened waiters only observe/recheck (idempotently doing no second release), or it marks the lease draining and each Prepare waiter removes its own registered ref exactly once, with a drain owner waiting for the ref count to reach zero. Preserve the original contract that rollback/detach releases exact acquired refs once, shared work is canceled only when no eligible ref remains, and capacity remains counted until all owned work is joined. Add a fixture assertion that would detect double decrement/double release and premature cancellation while another authorized ref remains.

### 2. Stopped-before-read result is not mapped to an allowed initializer code

The wrapper enum has only codes 0–5: pending, accepted, read-unavailable, retention-unavailable, invalid/unavailable, terminal-retention-failure (around lines 74–85). The worker section says a worker stopped before starting a read “installs a generic non-observable stopped result” (around line 112), but does not say which of those codes represents that state or whether it leaves `InitResult` pending while phase/stop signals resolve waiters. A separate stopped code would violate the closed range and fixed-width design; using read-unavailable would falsely imply a read failed.

State the exact permitted transition. For example, use the existing generic invalid/unavailable code 4 as a private, non-observable stop outcome if that is root’s intended interpretation, or leave the initializer result pending and define that Stop/phase wins and no initializer result is published. The future fixture must assert the chosen result stays in 0–5, ready closes once, a waiter that selects ready still rechecks phase and returns the typed stop outcome, and no read starts after stop.

## Source-release boundary and recommendation

The pure image encoder/file/test scope is otherwise bounded and consistent with the full original instructions plus the later preregistration. The stated fresh test sequence does not authorize writing or implementing the separate manager failure fixture now. Because the assignment also records a concrete future channel/lease protocol, that protocol must be unambiguous before it can serve as test-first lifecycle direction. Repair the two points above in a new immutable supplement and obtain a different independent critic’s review. Until then, I do not approve even the bounded encoder test-first release from this combined assignment; no source/fixture or compiler work is authorized by this review.

## Material differences from the original direction

- The current release is narrowed from the original private manager/Prepare lifecycle test-first assignment to a pure encoder seam and tests. The preregistration’s manager work, DTO assembly, admission, shared initialization, stop/drain, and actual JOIN remain future work; the repair says so and does not claim they are implemented.
- JSON retained-image accounting is brought in from the later lifecycle preregistration/revision 6 direction, not the original Oct 6 bounded assignment. It is confined to the new private encoder and does not change the helper or frozen fixtures.
- Scalar initializer generation is replaced by root-approved exact pointer identity plus one publisher, subject to JOIN-before-reuse. This is an explicit later root decision, not an unapproved deviation.
- The initial failed selected read outcome is corrected by later root clarification to successful `A` with a nonnil cohort lease and per-Prepare counts; it does not turn auth/guard/cancel/stop/irreducible assembly failures into successful outcomes.
- The shared first-failure and stop/ready tests are deferred to a distinct future manager fixture; pure image tests make no admission, no-read, stale-publication, or JOIN claim. This is materially clearer than the previous proposal, but the detach ref ownership and stopped initializer code remain unresolved as described above.
