# Root decision: deterministic worker read-start proof

This is an architectural proposal for independent review, not a source release.
It resolves the missing synchronization point identified in the lifecycle
failure-fixture assignment and its independent review.

## Use a genuine production boundary

The manager's sole poller worker must check eligibility under the actual manager
mutex before starting each trace read: manager admission is still open, the exact
entry is still registered, its phase permits a read, and it still has eligible
lease references. Use one private production method for that transition. Tests
may exercise that same method directly; do not introduce a test callback,
scheduler hook, sleep, or fake port barrier before a supposedly unstarted call.

The method's decision is the read-start eligibility linearization point. If Stop
or the final ref release wins that mutex first, it rejects the read and the worker
resolves the stopped initializer using existing code4/phase2-or3/nil current.
If the worker wins first, teardown must treat its work as owned until actual
return and JOIN. No claim is made that a callback already authorized at that
transition instantly disappears when Stop is requested. Logical DTO read-attempt
counts still increment only for actual RunTracePort invocations, not merely an
eligibility check. Cancellation can prevent a claimed call from occurring.

The sole-worker invariant supplies ownership; do not add a second worker, generic
initializer token, extra encoded phase, or speculative read-in-flight counter.
All port calls, cancellation, signal closes, waits and JOIN remain outside the
manager mutex. The implementation must invoke this real transition immediately
before every initial/recurring read and recheck its manager-owned cancellation
before invoking the port. Independent review must check actual call sites.

## Separate deterministic tests

1. Directly order manager Stop before the production transition and assert
   rejection, no port invocation, stopped initializer code/phase, readiness and
   actual worker completion. A negative transition alone does not prove those
   worker outcomes; exercise the real worker path as well.
2. Verify the permitted transition for a registered, referenced, running entry,
   preventing a universal-rejection seam from satisfying the negative test.
3. Separately test pending Prepare leases under global Stop and cancellation of
   one Prepare while another authorized lease survives.
4. Separately hold an actually started port call and prove actual return,
   retirement and worker JOIN before reservation/capacity reuse.

## Source scope must be explicitly revised

The current scaffold has neither the worker method nor the required lifecycle
fields. A fixture restricted to one new test file cannot compile against an
invented method. Before release, a different builder must propose the minimal
compile-safe production seam and required private lifecycle fields alongside
the new fixture, with explicit paths and frozen-file exceptions. These are
fields/methods required by Unit1, not optional testing infrastructure.

The independent critic must check this proposal against the full original Unit1
assignment, lifecycle preregistration, revision6/corrections and prior root
decisions. Any conflict with their stop/read-start semantics must be reported
before source release. The fixture/worker claim cannot substitute for physical
adapter admission, authorization, retirement, or the public C2 correction.

No source, fixture, public interface, dependency, code range, cap or byte-budget
change is authorized here. CW-28 remains open until the actual proof exists.
