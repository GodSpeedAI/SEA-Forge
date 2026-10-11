# Root clarification: terminal publication and surviving authority

Date: 2026-10-08. Architecture clarification for independent proposal review;
no source or runtime release. Read with watcher/terminal proposal de8c1647.

For an accepted fitting terminal snapshot, publish the immutable valid current
state and resolve initializer readiness. Then the worker returns without a
later read; actual worker completion may precede Prepare's final DTO return.
The valid retained state remains available for that DTO and authorized cache
reuse. Do not make the worker wait for DTO/creator completion, and do not drain
valid leases just because a fitting terminal state was accepted. Keep the
existing accepted phase/current representation; bar new claims for terminal
current state. Detach/Stop releases capacity only after actual completion.
Thus the proposal's handoff wording concerns preservation of the final value,
not an additional worker wait or a new completion field.

An invalid or canceled creator does not authorize a shared read. A different
eligible watcher that passes the existing current authorization and exact
cursor/parent checks can still authorize that manager-owned shared worker.
Drain the invalid watcher independently; preserve the valid survivor. Do not
equate loss of the initiating caller with loss of every watcher's authority.

This clarifies the existing independent-watcher and immutable-publication
contract. All other proposal limits and independent review requirements stand.
