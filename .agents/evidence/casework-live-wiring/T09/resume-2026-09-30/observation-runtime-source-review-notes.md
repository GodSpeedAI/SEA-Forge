# Observation runtime source constraints

Root records independent read-only unit5 preparation by t09_run_children_fixture_builder,
plus narrow source verification. No poller ownership/API/design decision or implementation
release is made here. Original approved contract and prepared unit5 requirements govern.

handleEvents currently verifies perspective once at stream open (server.go293–376) and
consumes only Store revision subscriptions. Logout removes the session (session.go376–385)
without canceling existing requests. Store cancellation is not poller cancel-and-drain.
Root must decide authorized sharing and subscriber revalidation/cancellation semantics;
do not claim immediate revocation from the current code. A later authorization failure
must not fan out stale buffers under an earlier successful identity check.

writeSSE emits id only when passed nonempty ID (http.go186–198). Observation events must
use empty ID and bypass Store.Append/case cursor publication; latest actual relay cursor
is informational. Store clones factual revisions and requires real advancing cursors.
Relay is a case-cursor map, not a run trace manager. Pollers need an explicit lifetime owner.

Existing RunArtifactProvenanceView intentionally omits timestamp/status/raw payload fields;
do not widen or repurpose artifact projection. Separate safe run trace projection must
validate requested run/case/item, discard actor/raw payload/output, retain only allowlisted
real frames and fail unavailable on an unsafe present exit integer. Execution and settlement
remain separate facts. Later transport failure retains last validated observation with an
explicit unavailable marker. Security revocation must not reuse that delivery path.

The incremental32MiB response cap bounds one JSONL line, not directory enumeration,
aggregate upstream bytes or poll lifetime. Safe inspect transport retry can double socket
attempts; define budgets as logical operations and enclose retries in global concurrency
slots if claiming at most2 active reads. Keep initial reads≤8,16qualified shared pollers,
global2read slots, at least1second per actual run,1024safe frames/run. Distinguish list
unreadables, read failures and unread candidates/capacity; unavailable dominates capacity.
Initial projected envelope≤1MiB includes all metadata/counts/IDs and actual encoded framing
as the canonical spec defines; frame eviction/count updates must not overshoot the cap.

Root additionally verified httpCaseworkAdapter native/fetch URL construction: current
subscriptions receive caseId locally but do not send it in the events URL. Case selection
is used by client parsing instead. A case-scoped observation subscription therefore needs
an explicit requested-case seam; decide optional case_id compatibility/default behavior
with the server and add it to BOTH native/fetch unit6 tests. Do not infer requested case from
an unrelated most-recent global Store revision. Existing snapshot stream behavior and real
cursor replay need direct regression proof during integration.

Remaining root design questions: manager identity key (verified session/authority or proven
read invariance), per-subscriber authorization on fanout, logout/expiry/revocation handling,
actual run rate limiting across different qualified pollers, terminal/no-subscriber cancellation
AND drain, shared existing-poller initial-read accounting, deterministic candidate selection,
initial envelope budget calculation, and malformed/oversized metadata behavior. Independent
source critique is required before implementation instructions stand.
