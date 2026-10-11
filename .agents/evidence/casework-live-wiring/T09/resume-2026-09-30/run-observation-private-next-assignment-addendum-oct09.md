# Private Next assignment sequencing clarification

Root decision, 2026-10-09. This supplements the immutable private-next
assignment in this directory; source release remains held until publication.

Admit the exact single-call token and register its operation under manager.mu
before invoking authorization or context callbacks. This makes any blocked
callback owned by the drain. Outside the mutex, authorize and check present
context before blocking on wake. After wake, revalidate authorization/context
before capturing and projecting; immediately before final disclosure revalidate
again. All callbacks remain outside mu. A failed terminal check schedules the
single drain owner, returns zero aggregate/no commit, and releases the call's
own operation before waiting for teardown. These checks are as-of observations,
not continuous revocation or atomic Store/Relay promises.

A concurrently attempted second Next is rejected with the private typed
already-in-progress distinction. It owns no operation/token and must not drain
or invalidate the already admitted call or consume its wake. This is distinct
from an admitted call's terminal auth/context/cancellation failure.

The deterministic projector seam wraps the existing pure per-run
buildRunObservationRunDelta function. Production supplies that exact function;
tests may pause its first invocation after the entire captured set has been
frozen. Aggregation and lifecycle use the same transaction helper in both cases.
No manager-global callback, fake commit path, public interface or weaker test
claim is introduced.
