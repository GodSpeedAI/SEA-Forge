# Fixture root review supplement — repair still required

Root independently read the complete rejection and the fixture/mock/Client cleanup
anchors. The fresh builder is gitleaks_sibling_builder; only the two new fixture
files may change. Client d2f993d7 and stub f661bd8f remain frozen.

Apply every independent finding: real permit Release must wake/reacquire the
no-spin waiter; synchronize post-write cancellation; cancel and bounded-join every
queued goroutine even after fatal branches. No weaker/deleted assertions, simulated
release algorithm, compiler or production implementation is allowed.

Additional original-requirement gap: recordingRunGetPermit.Release currently only
records an event (client_run_get_admission_test.go:31). Checking socket-close before
that event does not prove pool retirement/release occurred first. Client.discard
closes then decrements live and signals; healthy release adds idle and signals
(client.go:352–377). A release before those pool changes can still pass the current
socket ordering assertion.

Add direct observation at the mock permit's release boundary: canceled transport
must already have live==0 after discard; healthy success must already retain its
returned idle connection with live==1. Use bounded observation (for example mutex
TryLock, not a lock that can deadlock against a defective implementation), buffered
error/results and joined callers. Invoke observations outside the mock's own mutex;
do not use fatal assertions in worker callbacks. This is evidence for the original
pool-cleanup ordering requirement, not a new production hook or architecture.

No-spin timer counting supports the absence of expired-timer churn in this fixture;
it alone cannot establish absence of every arbitrary CPU-spin implementation.
Independent production source review must also prove a blocking change/context wait
without default retry loops. Do not overclaim the fixture's formal completeness.

Give the next independent critic this supplement, the ORIGINAL assignment and
review trail, the previous rejection and the complete repaired result. Actual
compiled assertion RED is still mandatory after source approval.
