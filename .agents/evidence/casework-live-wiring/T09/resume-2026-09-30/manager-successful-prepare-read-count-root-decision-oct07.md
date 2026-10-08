# Successful Prepare read-count accounting

Root implementation guidance; existing contract and actual-call semantics
remain binding. This is not algorithm or runtime approval.

A successful Prepare may derive ReadsAttempted from its creator-owned new
initializer batch only if production guarantees that every counted initializer
actually invoked RunTracePort once. Shared/cache entries and oversized nil-image
refusals are absent from that batch and count zero. A real initial unavailable
read is successful A and counts one for its owner, zero for shared waiters.

Stop/cancellation/final-ref release before an initial actual call must produce
failed Prepare: typed error, zero DTO and nil lease with owned rollback. It
cannot be represented as successful A merely to continue DTO assembly. If any
owned initializer stops before its call, no successful partial wrapper may be
published. Read eligibility or reservation alone never counts as an attempt.

These implications follow from the lifecycle preregistration actual-start and
failed-Prepare clauses, the root read-start decision and initial-failure
clarification. Source-only recon found no counterexample within those rules.
The forthcoming implementation and independent critic must verify actual
worker call sites, all stopped/invalid result branches, final Prepare checks,
all-eight launch-before-wait and membership/JOIN ownership. They must reject
batch-length accounting if any successful path contains an unperformed call.

No speculative persisted counter, initializer token, encoded field or callback
field is authorized by this guidance. Private local accounting can be chosen
if it observes actual starts safely and preserves the original interface and
ownership constraints. Report a concrete counterexample to root before
introducing extra state. Actual logical port invocations are counted; physical
wire retries remain governed by the separately verified adapter.
