# ADR-008: Casework Live Cursor V4 shared journal and global frontier

## Status

Proposed normative ADR based on the approved recommendation-level architecture;
independent review of this authored spec/ADR package remains pending.
Implementation remains held pending complete writer participation proof,
contract/schema updates, and required verification gates.
The operator approved the six recommendation areas recorded in
.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/c2-six-recommendations-operator-approval-oct08.md.
That approval does not claim the operator read or approved one exact candidate
hash. It does not authorize source, generated schema, or runtime release by
itself.
The operator separately approved four bounded-reader policy choices in
.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/c2-bounded-reader-additional-policy-operator-approval-oct08.md
for incorporation into the normative spec and this ADR. This remains
proposal-level approval; runtime proof and readiness are not established.

## Date

2026-10-08

## Context

The existing Go application obtains a case projection from several independently
locked SFWP reads. The sequence is not one coherent capture. The SEA Forge event
range reads all ledger entries before filtering and applying its frame cap
(crates/sea-forge-server/src/sfwp/events.rs, get_range; LedgerStream::read_entries
in crates/sea-forge-ledger/src/types.rs). Case-visible writers span
CaseRunner, CLI, and server. Some server mutations publish asynchronously and
can fail after the case mutation. The global event stream therefore cannot by
itself prove complete case-facts notification coverage.

Ledger append ordinals are assigned over the shared ledger, which also contains
other cases and non-event rows. A case's observed ordinals may legitimately skip
values. A missing global row has no known case identity unless validated source
metadata proves it.

This decision preserves SEA Forge authority and settlement, the existing
CaseRunner-to-ledger and server/CLI dependency direction, the reviewed
revision-3 bootstrap boundary, exact real IDs, and existing authorization.
Current code is evidence of the starting point, not proof that the proposed
reader, journal, capture, or stream behavior exists.

## Decision

### Shared journal and mutation ordering

Place a derived case-projection mutation journal under
.sea-forge/case-projection-journal/v1/ and implement its shared boundary in
sea-forge-case-runner, which is already shared by CLI/server and already
depends on sea-forge-ledger. Add no crate dependency, kernel verb, case
authority, or settlement authority.

All supported case-visible writers and coherent Facts captures use one
cell-scoped interprocess writer lock. Acquire it before the ledger append lock;
never acquire them in reverse. Resolve and authorize the caller and run existing
guards before journal effects. Do not run authentication callbacks, network
I/O, capability execution, or unbounded work while holding the lock. Synchronous
kernel APIs stay synchronous; async edges use their existing blocking boundary.

Before visible mutation, persist a bounded pending record with exact IDs, fixed
mutation class, already-authorized caller class, operation ULID, prior real
global ordinal, checked u64 epoch, and pending status. Store no credentials, raw
errors, or arbitrary payload. Use synced temporary write, atomic rename, and
parent-directory sync. After case/local writes, append a real safe global
notification event with the operation identity, update the derived index, then
durably mark the record clean. A failed publish is not clean success.

On restart, pending state blocks the affected projection. Persist bounded
recovery continuation and prove absence only after scanning all validated
global rows through a real pinned head. Incomplete or over-budget scans remain
pending and unavailable. Recovery reads actual present state and appends a new
case.projection_reconciled event only after validated absence; it reports
present-state reconciliation, not past success. The journal and head index are
derived coordination data, never a second source of truth.

### Global event frontier and case projection

Extend events.get_range with optional range_version=2; add no kernel verb.
Version 2 supplies bounded ordered pages, exact opaque cursor and canonical
decimal-string ordinal pairs, real pinned head, global scanned-through row,
continuation, completion, and optional case-state stamp. Validate every complete
global ledger row through the pinned head before filtering frames. Non-event
rows advance only global progress. An empty filtered page may advance the
validated global frontier. A failed page never advances a frontier or
acknowledgement.

Keep a global scan frontier distinct from each per-case head and delivery
watermark. Case watermarks may skip rows belonging to another case or
non-event records. Wake broadcasts are hints only. Case-local invalidation is
permitted only after global coverage is proved and validated metadata attributes
the case. An unexplained global discontinuity has unknown impact: make dependent
projection/bootstrap unavailable and drain dependent streams cell-wide. Do not
infer missing-case identity from a per-case ordinal gap or latest-head index.

#### Bounded reader, continuation, and cursor filters

The existing `events.get_range` verb gains only the approved optional
`range_version=2`; no kernel verb is added. Omitted version retains its exact
legacy vector wrapper and frame shape. V2 uses distinct page/frame DTOs. The
reader validates the global prefix from actual origin or a verified continuation
through a real pinned head before filtering. Non-event rows advance only global
progress. A page that cannot prove completion returns typed unavailable rather
than a successful partial legacy vector.

Each server start creates a fresh in-memory Ed25519 signing key using the
existing `getrandom::fill` entropy source. Startup fails before accepting
requests if entropy is unavailable. The local seed is zeroized with existing
`zeroize` support immediately after signing-key construction. The key is never
persisted or logged; restart invalidates outstanding tokens. A continuation is
issued only for an acknowledged validated prefix and binds the exact stream,
pinned head, last acknowledged row and byte boundary/checksum, and exact
request-filter bindings. The stream binding is a field-specific
domain-separated SHA-256 digest, not a raw string in the token. Compute it from
a distinct fixed ASCII domain tag, a presence byte, the checked UTF-8 byte
length as unsigned 64-bit big-endian, and exact UTF-8 bytes when present; absent
uses the absent tag and zero length. Recompute it for the requested registered
stream and require equality before any seek. The reader enforces the
4,096-byte encoded
token and 2,048-byte payload caps and fixed signature representation/length
before decoding or trusting seek fields. Invalid or oversized tokens fail
unavailable. A trusted Go owner that loses its expected continuation
acknowledges nothing from that failed page and follows the existing cell-wide
unavailable/drain and bounded-rebuild rule. An arbitrary SFWP request cannot
mutate that owner's state.

Cursor filters remain exact opaque UTF-8 values: `from_cursor` is exclusive
and `to_cursor` inclusive. Each signed filter binding is a field-specific
domain-separated SHA-256 digest computed from a distinct fixed ASCII domain
tag, a presence byte, the checked UTF-8 byte length as unsigned 64-bit
big-endian, and exact UTF-8 bytes when present; absent uses the absent tag and
zero length. The token carries the digests, not raw filter strings. On resume,
recompute and compare both bindings against the exact requested values before
using resolved state or seek data. The continuation carries nullable resolved ordinals,
with an absent filter distinct from a present-but-undiscovered filter; a cursor
resolves only when found inside the acknowledged validated prefix. Ordinal zero
is the real string `"0"`. Undelivered lookahead cannot advance either resolved
filter state or the acknowledged frontier. An unresolved requested bound fails
with the existing unknown-cursor input error at the pinned head; earlier
incomplete v2 pages are only validated prefixes. Only an actually registered
stream whose registered path is successfully checked as absent or zero-byte
under the cooperative lock returns the v2 complete-empty arm with
`complete=true`, null head/frontier, no frames, and no continuation. It has no
synthetic cursor. A requested bound on that stream is unknown; unregistered,
corrupt, or unreadable history is unavailable.

Acceptance vectors require entropy-source failure to prevent startup from
accepting requests and require the local seed to be zeroized immediately after
signing-key construction. Complete-empty vectors distinguish a registered
absent/zero-byte stream from an unregistered identity, unreadable stream, and
corrupt history; only the registered, successfully checked case succeeds.
Digest vectors pin the distinct stream/from/to domain tags and exact
presence/checked-length/UTF-8-byte encoding; changed identity bytes fail before
seek and token fields contain digests rather than raw stream/filter strings.

Row integrity retains the established Rust hash protocol: `payload_hash` covers
the complete payload `Value`, and `entry_hash` uses the typed `LedgerEntry`
helper. Unknown top-level row fields remain tolerated and are ignored by typed
deserialization/reserialization; unknown payload fields remain in the value
and are hashed. Known fields and types are validated. Event payloads require a
string `kind` and present `detail` (including explicit null); optional case/run
IDs may be absent/null or strings. No new kind grammar or rejection of extra
payload fields is introduced.

The legacy unversioned vector shape remains available only after a bounded scan
validates coverage to the real pinned head. If row, input-byte, response-byte,
or lock-wait ceilings prevent proof, return typed unavailable and no successful
vector or partial frames. Callers surface the unavailable result; large
histories use version 2. There is no per-page elapsed-time guarantee. The
two-page/500 ms reconciliation scheduling bound does not preempt or certify a
blocking OS read and cannot acknowledge an unfinished scan or convert it to
partial success. Do not provide an unbounded compatibility fallback.

### Identity, capture, and history

Preserve actual generated and supported legacy case IDs, exact world IDs, and
opaque ledger entry_ulid cursors. Use checked u64 internally and canonical
unsigned decimal strings on the wire, including TypeScript's checked BigInt
range. Never use JavaScript Number or lexical ordering for ordinals; never parse
an opaque cursor.

An ordinary snapshot's capture_digest is a versioned digest of its fixed public
snapshot representation. It distinguishes captures, including changed
same-cursor process baselines, but is not a cursor, event, authority token, or
historical reconstruction. Bind exact cursor and digest for historical reads,
SSE reconnect, and non-create stale preconditions. A digest mismatch refuses
before effects/disclosure and resynchronizes from an actual current boundary.
Existing authorization and domain checks remain authoritative.

Complete-empty bootstrap requires strict complete inventory, clean journal, and
validated global coverage. Preserve revision-3 bootstrap response arms and the
narrow create-only PROPOSE_CASE exception. Accepted create retains only the
returned real ID; it is not ready for history or SSE until that exact case has
a validated ordinary snapshot. Explicit selection never redirects.

### Failure scope and resource limits

Unknown or unvalidated global coverage causes cell-wide unavailable/drain and
no acknowledgement. A validated case-local dirty marker, unretainable
authorized capture, or unavailable/evicted pair may isolate only that case.
Keep typed inventory_unavailable, frontier_rebuilding, projection_gap,
case_not_ready, and resync_required failures. Preserve exact HTTP distinctions:
global/inventory/frontier/cap unavailable is 503; explicit unknown case is 404;
historical resync is pre-body 404; SSE resync is pre-header 409. Errors never
become empty success.

The V4 limits are serialized-payload and availability bounds, not process RSS
guarantees: 500 ledger rows, 500 frames, and 1 MiB for the complete serialized
page response; a 2 MiB raw row including LF and 4 MiB raw input per page
including non-events and lookahead; 4,096 encoded token bytes and 2,048 payload
bytes; 50 ms ledger-lock wait; at most two pages or 500 ms per reconciliation
turn; 4,096 cases/8 MiB
for the derived head index; rebuild ceiling of 1,000,000 rows or 10 active-work
minutes per detected lineage with no automatic reset; 4,096 cases/4 MiB strict
inventory; journal records of 32 KiB, 256 records/8 MiB and 50 ms lock wait;
1 MiB per snapshot/event; 64 revisions/4 MiB per case; 64 MiB Store; 128 case
heads; 32 streams; two live frames/2 MiB per queue; 64 revisions/4 MiB replay;
64 MiB aggregate subscriptions including replay, queues, and in-flight writes;
five seconds per response header/frame; at most one already-permitted
post-invalidation write. Never truncate/drop while claiming continuity. If a
limit cannot be enforced, remain held. Raw and serialized byte caps do not
guarantee RSS, heap use, or I/O latency; no per-page elapsed-time bound is
introduced.

### Writer migration and unsupported writers

Before readiness, independently enumerate every in-tree writer of projected
case facts, records, local events, and run membership, then prove each supported
writer participates. The initial migration checklist includes:

- CaseRunner case_ops and lib operations, including run_stage_case, append/save,
  and run membership.
- CLI case, task, manager, project, approve, migrate, plan_pipeline, pipeline,
  and resume commands.
- Server lib, case_mutations, case_ops/advance, and case_dispatch.
- Delegation execute_with_permission_broker and finish_rejected;
  case_dispatch execute_sandbox; agent_probe probe and finish_rejected; and
  recover_cancelled_delegations. Their success, rejection, probe, and
  cancellation-recovery writes to runs/<run>/settlement.json affect RunsListForCase.

This list is deliberately not an exhaustive audit. Global event publication
coverage alone does not prove case-facts coverage. Manual, plugin, out-of-tree,
and external filesystem writers are not fenced by a cooperative OS lock. If
such writers may be active, dependent projection remains unavailable; no
coordination claim is permitted.

## Consequences

- The parent GodSpeed casework specification remains authoritative outside
  V4 live cursor scope. The supplemental spec is normative for V4 after its
  required independent review.
- The four approved bounded-reader choices are for normative incorporation
  only; they do not authorize implementation or readiness.
- Authored Rust/Go/TypeScript contracts and schemas must be updated together
  after review; generated contracts remain generator-owned.
- Complete writer participation is a readiness prerequisite, not an assumption
  supplied by this ADR or the initial checklist.
- Independent review, focused TDD, real participation proof, canonical gates,
  and runtime verification remain required. No readiness or settlement claim
  follows from authoring these documents.

## Alternatives considered

- Reusing only the best-effort global event stream cannot prove every
  CLI/CaseRunner/server mutation reached it; rejected.
- Treating per-case ordinal spacing as a gap mistakes interleaved global rows
  for case loss; rejected.
- Returning a successful legacy vector after an incomplete bounded scan falsely
  labels an unverified prefix complete; rejected. Legacy mode fails typed and
  large scans use v2.
- Treating digest or cursor as authority or as reconstructed history is
  rejected.
- Resetting cumulative rebuild ceilings automatically or relaxing external
  writer limits would hide unproved history; rejected.

## References

- .agents/specs/casework-live-cursor-v4-spec.yaml
- .agents/specs/godspeed.casework-cognitive-environment-spec.yaml
- .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/live-cursor-v4-complete-candidate-revision7-oct08.md
- .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/live-cursor-v4-complete-candidate-revision7-independent-review-oct08.md
- .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/c2-six-recommendations-operator-approval-oct08.md
- .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/c2-bounded-reader-additional-policy-operator-approval-oct08.md
- .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-bounded-ledger-reader-proposal-revision4-correction-oct08.md
