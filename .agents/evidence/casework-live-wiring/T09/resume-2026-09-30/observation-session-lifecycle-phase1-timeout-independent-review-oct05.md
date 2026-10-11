# Independent review: session lifecycle Phase 1 total-timeout repair

Date: 2026-10-05. Verdict: **APPROVE source readiness for the Phase 1 stub and
fixture only**. This is not a runtime implementation or SSE/read-drain
approval. No source was changed and no compiler/test/scanner/Git/status/debt
command was run.

## Evidence

The timeout repair record reports the original fixture hash
`5bbc19a4f1719defd79c564623603a64e2daa391a7db9ebe04821b94c34dd934`, repaired
fixture hash
`8de408be7efc31d6e19d73870bee541d82f127272b5c2ab048dbee6ba3212e3f`, and
unchanged `session.go` hash
`f8f0df88d283ef0c28057f08fcdd6b23c7111c89abd68e23b52f754eb9d17b48`; all
match the current checkout. The repair moves one `time.NewTimer(5*time.Second)`
and its `defer Stop` before the 16-iteration race loop. This now bounds the
whole loop with one shared timeout, resolving the prior per-iteration timeout
finding. `gofmt -l` reports neither owned Go file.

The race join retains the required synchronization: each worker performs its
store operation and, for `Current`, writes `got`/`found` before sending on the
capacity-two `completed` channel. The test receives two sends before reading
those results. The `start` close releases both workers together. The 16
iterations, outcomes, and assertions are unchanged.

The full required matrix remains present: detached/no-slide Current state;
earliest deadline; exact and just-after idle and absolute expiry; legitimate
Resolve idle touch with fixed absolute expiry; stable signal closure via
Destroy, Resolve expiry, Current expiry, Sweep expiry, and capacity eviction;
idempotent removal; unrelated-session safety; bounded store capacity; and
synchronized Current/Destroy lookup races. The test clock serializes reads
and updates. The zero/false stub causes the first `requireCurrentSession`
behavioral assertion to fail before race setup, so the expected RED is a live
Current behavior failure, not setup or compilation. The declarations remain
the approved detached state shape; existing production store methods remain
untouched.

## Timeout cleanup limit and approval boundary

The shared timeout bounds how long the test goroutine waits, but it cannot
forcibly stop a worker blocked inside the context-free `Current` or `Destroy`
method. On timeout, `t.Fatal` exits the test while such a worker may remain
blocked on that store's mutex. A worker that returns later will not block on
its completion send because the channel is buffered. This limitation is
accurately disclosed in the repair assignment and does not invalidate the
bounded assertion wait; it must not be described as goroutine cancellation or
full cleanup.

No material deviation from the root lifecycle proposal or the bounded Phase 1
repair assignment was found. Approval covers source readiness for the stub
and fixture only; the sole compiler owner must still establish the intended
RED. It authorizes no production revocation logic, watcher, server integration,
pending run-trace read cancellation/drain, or full Unit5C completion.
