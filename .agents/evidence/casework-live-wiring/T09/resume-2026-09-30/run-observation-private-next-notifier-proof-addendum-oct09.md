# Private Next notifier ownership proof clarification

Root decision, 2026-10-09. Supplement to the original private Next assignment
and sequencing addendum. No notifier callback, mutable hook or public seam is
authorized. The existing four-file source boundary remains unchanged.

Factor the production notifier path into a private target-registration helper
called under manager.mu and a private send/release helper called outside it.
Registration validates exact eligible attached leases and acquires each real
target's notifyWG reference under the mutex. Sending is nonblocking and
balances each registered target's Done even when coalesced. The actual worker
publication path must consume these same helpers, with no parallel test-only
implementation or authority bypass.

A deterministic notifier-ownership unit test may invoke the real registration
helper on an actual Prepare lease/poller while holding the mutex, retain the
registered target references across drain initiation, then invoke the real
send/release helper. Assert counted capacity remains until that registration
is released and drain actually joins. Do not manually Add fake WaitGroup
counts or invent lease state. This tests production ownership bookkeeping;
separate real worker/read/publication tests prove actual wake/coalescing and
shared-survivor behavior. Do not conflate the two evidence claims.

The compiling TDD phase may declare minimal private helper signatures/field
scaffolding with inert stubs; expected behavioral RED must precede a fresh
builder's real implementation. No stub can be approved as the completed unit.
