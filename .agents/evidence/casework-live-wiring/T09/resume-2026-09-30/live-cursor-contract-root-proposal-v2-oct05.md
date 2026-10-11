# Cursor correction proposal V2 — HOLD for independent and operator review

This replaces the rejected design proposal, without rewriting it or releasing
public contract/runtime edits. Root AGENTS section 4, governing spec change_rule
and T09's return-to-T01 trigger require prior operator approval and ADR/spec
updates. Source reconnaissance is evidence of defects, not approval.

## Decision and compatibility

Treat cursors as opaque durable addresses. Production keeps the exact kernel
entry ULID. Legacy numeric logical cursors remain accepted for the local/mock
adapter and its existing fixtures. Neither form is converted or newly minted.
Syntactic validity does not imply ordinal order; no consumer compares unlike
forms, ULID text, timestamps or numeric cursor fragments to order live history.

The canonical validity union remains
`^(?:[0-9]+\.[0-9]{10}|[0-7][0-9A-HJKMNP-TV-Z]{25})$`.
This is a public semantic revision, not an unqualified backwards-compatible
correction. Existing strict numeric-only readers reject live ULIDs. Deploy the
matching gateway/UI contract revisions together; no mixed-version compatibility
claim or transparent negotiation is made. Keep stable unversioned schema `$id`s
intentionally, and increment the governing spec from 0.2.5 to 0.3.0 with a new
ADR amendment and decision-log approval record before implementation. The union
is durable support for the two existing adapters, not a migration of persisted
kernel IDs. There is no demonstrated numeric-form production ledger history.

## Order and recovery

Root source inspection confirms kernel `events.get_range` resolves exact
cursor IDs to authoritative append ordinals and returns durable ledger order.
Broadcast arrival order is insufficient: concurrent blocking append tasks can
finish and publish in a different order. Kernel overlap suppression and Go
consumers also contain lexical comparisons. A schema-only edit is insufficient.

Make the casework Go feed consume existing `events.get_range` pages as the
authoritative delivery path. Subscription events may wake reconciliation but
must not directly become ordered revisions. Periodic bounded reconciliation
also catches missed hints, replay caps and disconnects. One sequential range
request is outstanding at a time, each capped at the existing 500 records.
Advance the exact resume token only after delivering a validated event to the
bounded existing consumer channel. Drain full pages with context cancellation
and backpressure; after an incomplete/empty page, idle reconciliation is no
more frequent than once per second. Inspect retries remain existing transport
behavior. Unknown durable resume tokens fail closed with explicit recovery;
never silently resume from the beginning while claiming continuity.

On a new gateway process, replay begins from no cursor; retained Go history is
process-local. This does not manufacture historical authority facts or promise
unlimited retained history. Verify existing captured-fact replay semantics and
report any conflict before implementation. No kernel ID generation, ledger
schema, public SFWP field, new verb or Workbench modification is authorized.
Kernel broadcast lexical assumptions remain separately recorded CW-09; this
proposal does not claim to repair every SFWP consumer.

Downstream relay heads follow this ordered feed, deduplicating by exact identity
within bounded retained state. Store appends keep that delivery order, using
the exact cursor index to reject retained duplicates. SSE replay slices after
the requested retained index, under the same lock as subscriber registration.
An unknown/evicted nonempty resume token emits explicit resync, followed by
retained replay, rather than comparing it with the oldest string. Mutation
waiters compare publication identity against their captured prior cursor.
Stale-intent authorization remains exact case-cursor equality.

UI native and fetch ingestion consume ordered SSE revisions and suppress exact
duplicates using bounded identity state, preserving it through reconnect.
Resync establishes a new replay boundary and invalidates stale deduplication
state explicitly. History/trajectory order comes from ordered source arrays;
numeric sorting is confined to mock preparation where needed. New IDs lower
lexically than prior IDs must still advance ordinary live revisions. No-ID
observations remain informational, use the latest real case cursor, and do not
advance ordinary replay/deduplication/history. Before a real cursor exists,
withhold the observation instead of synthesizing one.

## Explicit authored and runtime inventory

Canonical sources: `schemas/cognitive-world.schema.json` snapshot cursor;
`schemas/interaction-intents.schema.json` client_cursor;
`schemas/event-stream.schema.json` shared envelope cursor, named observation
cursor, and `definitions.ResyncRequired.properties.last_cursor`. Review
requested/oldest recovery payload cursor descriptions together: reported known
cursors follow the union, an unrecognized requested token remains opaque input
reported for diagnosis, not a claimed validated durable position.

Update `typescript/types.ts`, `typescript/client.ts`, mock adapter semantic
comments, spec-04 cursor/replay/history descriptions and relevant schema/example
conformance tests. Preserve legacy examples and add actual ULID producer cases.
Historical T01 confirmations remain immutable; a new correction record explicitly
supersedes their numeric-only semantics. Generated mirrors follow their existing
source/generator workflow; no hand edits to generated zones.

Go runtime scope: SFWP range/subscription adapter, kernel feed assembly, relay,
projection Store and HTTP replay handler, plus focused tests. UI scope: HTTP
adapter native/fetch ordinary cursor handling and history projection/live/intents
callers, plus focused tests. Exact files and decomposition must be preregistered
after approval and before builders are released. Adjacent intent response
`new_cursor`/`current_cursor` and trajectory base/head/checkpoint cursors remain
exact opaque kernel addresses with source order; no ordinal field is added.
Hello/bootstrap no-cursor state is not a valid revision or observation cursor.

## Required proof and review questions

Prove actual producer ULID acceptance and all union negatives, authored/mirror
parity and absence of ID conversion. Prove durable replay/live overlap with
lexically descending IDs, concurrent publish hints, more than 500 events,
consumer overflow and restarted gateway; no skipped/duplicate retained revisions.
Prove exact-index retention/resync, native/fetch reconnect duplicate suppression,
bounded caches/backpressure, mutation waiters, stale intent refusals and frozen
historical snapshots. Keep local/mock regression and live conformance gates.

Independent critic must reject if the bounded duplicate state, resync ordering,
captured historical-fact semantics or range-to-feed identity/authorization cannot
be proved from the current interfaces. Identify missing scope and every material
change from the first proposal. This proposal is not ready for implementation
or operator approval merely because its regex accepts ULIDs.
