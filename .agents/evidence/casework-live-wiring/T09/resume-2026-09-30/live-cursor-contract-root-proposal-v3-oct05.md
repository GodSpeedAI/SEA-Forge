# Cursor correction V3 — HOLD for independent review and operator approval

This proposed public semantic revision supersedes neither frozen contracts nor
historical evidence until approved. Read V2 and its rejection as binding context.
Keep V2's exact opaque cursor union, unchanged kernel identities/verbs, ordinal
range authority, matched gateway/UI rollout, spec0.3.0, stable schema IDs and
ADR/decision-log requirement. This document replaces V2's unresolved cold replay,
acknowledgement and readiness rules and adds the newly discovered bootstrap and
world_id inventory. No builder is released by this proposal.

## Captured history and cold start

Historical UI revisions are immutable authority snapshots captured by this gateway
process, with their actual capture timestamps. They are not historical kernel
reducers, and a committed event timestamp is not the timestamp of a later authority
read. Never replay an old ledger backlog by repeatedly labeling current Facts as
past revisions. Outer snapshot-event timestamps use the actual capture time;
kernel event time stays internal provenance, not a substituted capture time.

At cold start, use existing case.list and ordinal events.get_range paging to find
the latest real cursor of each currently existing case at the reconciled ledger
frontier. Scanning old rows establishes addresses/order only; it creates no past
world revision. Track only case keys from that current authority inventory, not
arbitrary case IDs found in historical events. Keep one range page of at most500,
one outstanding request and cancellation/backpressure; no full backlog allocation.
Capture each existing case once at its latest real case cursor, timestamped NOW.
Those are new process-local baseline captures, explicitly marked as such in the
revised contract documentation. They do not reconstruct prior captured history.
An existing case without a verified real cursor remains unavailable, not synthetic.

The established feed watermark is the last scanned exact durable token. After
baseline capture, events appended after that frontier are reconciled in ordinal
order; subscription frames remain wake hints. A current Facts read may already
include effects later than its trigger: the revised guarantee is immutable capture
at a known observed case frontier, not an atomic as-of-event state reconstruction.
Document that limitation explicitly; no atomicity or contiguous-event snapshot
claim is permitted. Snapshots never mutate after being retained.

## Acknowledgement, failures and stale guards

Separate the newest durably observed case token (stale-intent guard) from its
newest successfully published capture. Advance observed tokens by ordinal feed
position, never by string maximum. A single sequential feed/relay coordinator owns
the page position and completion acknowledgement. Delivery to a channel is not
an acknowledgement. For each case event, acknowledge after successful Store.Append
and published-head update, or after an explicit fail-closed gap disposition.
Events without a case have an explicit no-projection disposition.

A projection failure immediately makes that case unavailable, invalidates its
retained replay window and cancels its active ordinary streams before that event
is acknowledged. Preserve the observed token for stale checks. Do not retain a
pretend revision or silently resume past the failure as if history were complete.
Other cases may continue. A subsequent successful capture establishes a new real
case boundary; old resume tokens require resync to that boundary. No replay or
observation is served for the affected case during the gap. Mutation publication
waiters succeed only on an actual accepted capture, never on gap acknowledgement.
Internal gap state is process-local; no persisted ledger or ID change is proposed.

Case-specific stream cancellation/gap visibility and Store invalidation must be
atomic with respect to subscriber registration and replay selection. Source reads
and stream cancellation joins occur outside Store/relay locks. Before implementation,
preregister a concrete coordinator/Store API decomposition that proves this ordering.
If current interfaces cannot support it without an additional public or architectural
change, return this proposal for review instead of improvising an implementation.

## Readiness and resync

For an existing selected case with no retained real boundary, GET world/history
and ordinary SSE return typed unavailable (503) before committing SSE headers.
Never emit a resync envelope with an empty or invented boundary. A cold process
has no prior retained history; once a baseline exists, any unknown/evicted token
produces resync at the real oldest available case boundary and ordered replay.
All replay/registration decisions use exact membership under one atomic boundary.
Resync clears the client's retained identity/history state for that case before
accepting the supplied boundary; no lexical comparison or dedup suppression may
drop the boundary revision. Observations wait for a valid published case cursor.

## Explicit empty-cell bootstrap

Add an authored bootstrap response variant for the existing unqualified GET world,
discriminated from an ordinary case snapshot. It contains actor perspective,
capture timestamp, no-cases status and available case-entry template information;
it contains NO world_id, case_id, cursor or ordinary history revision. Existing
case-scoped requests keep the ordinary snapshot contract and fail unavailable
until ready. Do not broaden the ordinary snapshot schema to admit empty strings.

The UI represents bootstrap as a separate state, outside WorldHistory and its
cursor/dedup caches. It renders the available entry flow using existing approved
case-entry interfaces. While no case exists, bounded authenticated GET world
refresh (no faster than1second; one outstanding request, cancellation on disposal)
can discover the first ready actual case. Only a validated ordinary snapshot
creates history and a case-scoped stream/observation subscription. Explicitly
requested case selection is never silently redirected to another case. No empty
SSE subscription, synthetic revision or invented session/ledger identity is used.
The exact bootstrap wire DTO and entry-flow compatibility must be source-reviewed
before this proposal can be approved; if existing entry interfaces cannot support
it, enumerate that gap and reject rather than inventing a new kernel capability.

## Additional contract inventory and proofs

V2's inventory omitted snapshot world_id, whose current regex embeds numeric-dot
cursor syntax. Preserve the existing deterministic construction from exact case
ID and cursor, but admit the canonical cursor union in that embedded suffix.
Update authored schema descriptions, bootstrap response types, mirrors/generators,
Go response handling and UI bootstrap/history state together. Never re-key ledger
IDs or rewrite retained immutable evidence. Full JSON Schema semantic conformance
must prove ordinary and bootstrap arms, not only required-field presence checks.

Keep every V2 proof, and add: cold backlog never creates fake past captures;
baseline capture timestamps differ honestly from trigger timestamps; a projection
failure cancels stale delivery and cannot acknowledge continuity; unaffected cases
continue; failed-case recovery has an actual resync boundary; cold unavailable
returns before SSE headers; empty-cell response has no cursor/history identity;
first real case transitions to validated history and scoped SSE; template entry
remains usable; caches, range pages, refresh concurrency and disposal remain bounded.

Independent critic must inspect source and normative specs, identify every material
change from V2 and frozen guarantees, and reject unresolved compatibility/API/entry
flow assumptions. Source review is a prerequisite to an exact operator approval
request, never authorization for public edits. Approved auth/trace prerequisites
may proceed independently while this proposal is held.
