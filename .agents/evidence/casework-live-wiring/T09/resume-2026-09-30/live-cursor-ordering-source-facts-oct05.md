# Live cursor ordering and recovery: source facts (2026-10-05)

This is a read-only source/history fact record to inform the held cursor-contract
decision. It does not approve a public contract change or choose an implementation.
No cursor-related source, schema, runtime, generated, fixture, status, debt, or Git
file was changed for this report. The separate `.gitleaks.toml` one-key correction is
recorded in `gitleaks-target-rules-builder-oct05.md`.

## Producer grammar and history

The SFWP event producer has no epoch/sequence translation in the inspected source or
file history. `crates/sea-forge-server/src/sfwp/events.rs` constructs both newly appended
and replayed `EventFrame.cursor` directly from `entry.entry_ulid` (`:59-100`); its tracked
history begins with the SFWP event implementation (`5283575`, `26da6df`) and shows that
same raw-ID design. `crates/sea-forge-ledger/src/types.rs:81-147` generates a 26-character
Crockford ULID. Its monotonic state is a process-local `OnceLock` (`:89-105`), not a
persisted sequence. The repository's normative kernel rule says ULIDs are record identity,
not authoritative order; `append_ordinal` is the stream order (`.agents/specs/spec-full.md:544-557`).
So comments claiming raw ULIDs are “monotonic” do not establish ordering across process
restarts or a clock regression.

The decimal `<epoch>.<seq>` grammar appears in the earlier T01 canonical UI contract
(`59b8ff4`, confirmed at `T01/confirmation.md:54-59`) and in the cognitive UI mock
generator (`.agents/reports/interface-contracts/typescript/mock-adapter.ts:226,370`). The
T01 schemas/examples and mock are not kernel ledger producers. The later live Go path
passes the SFWP event string through unchanged (the c7fabaaa source recon gives exact
producer-to-relay spans). In this checkout/history I found no evidence that the ledger
event producer emitted dot-format cursors; this is a bounded repository-history finding,
not a claim about external cells or uninspected deployments.

## Kernel order and replay guarantees

The kernel already has an order-safe cursor-to-position mechanism. `ordinal_for_cursor`
resolves a cursor by exact `entry_ulid` equality to that entry's `append_ordinal`
(`events.rs:123-130`). `replay_after` and `get_range` compare append ordinals and return
entries in ledger iteration order (`:140-192`). Resume is exclusive of `from_cursor`;
`to_cursor` is inclusive. Both use a 500-entry maximum (`EVENTS_REPLAY_CAP`, `:37`). A
provided unknown cursor is an error, not an implicit replay from the beginning
(`:110-120,136-148`). The serialized `EventFrame` contains the cursor string but no ordinal
field (`:16-29`).

For live subscription overlap, the server subscribes to the broadcast channel before
reading replay (`crates/sea-forge-server/src/lib.rs:start_subscription`, around `:1990-2050`),
then skips live frames with `frame.cursor <= last_replayed_cursor`. This is a raw lexical
comparison layered over an ordinal-ordered replay; exact membership in the bounded replay
batch would be an equality fact available without a new wire field. The broadcast channel
capacity is 256 (`lib.rs:166`); lag/full writes are dropped with comments pointing to
`events.get_range` recovery.

## Raw lexical comparisons in live paths

The following current comparisons assume string order agrees with source order:

- SFWP Go client `apps/godspeed-casework-go/internal/adapters/sfwp/subscribe.go:144` drops
  `event.Cursor <= sub.LastCursor()`. It resumes by sending the last-delivered cursor as
  `from_cursor` (`:93-104`); the kernel resolves that cursor to ordinal. Consumer queue
  capacity is 256, and overflow reconnects from the last delivered cursor (`:36-45,144-160`).
  `EventsGetRange` exists (`:197-208`) but has no non-test caller in this Go module, so this
  subscription loop does not page/drain an arbitrary backlog itself.
- `server.start_subscription` suppresses overlapping replay/live frames with lexical `<=`
  as above. Kernel replay is capped at 500. The separate Workbench Tauri host does explicitly
  drain `events.get_range` until an empty page before subscribing
  (`workbench/apps/desktop/src-tauri/src/events.rs:1-15,103-191`); it persists a raw cursor
  string in an app-data JSON file (`:40-80`).
- Go relay `apps/godspeed-casework-go/internal/server/relay.go:103,107,156,186` lexically
  advances global head, per-case observed cursor, per-case published cursor, and the
  `WaitForCaseAdvance` baseline. Go projection Store `internal/projection/store.go:159`
  requires `rev.Cursor >` the newest retained cursor; `:200` selects replay rows with
  `Cursor > after`; `server/server.go:313` emits `resync_required` when `last < oldest`.
  Store lookup `At` uses exact map-key identity and the retained `Trajectory` preserves
  append order (`store.go:94-108,126-145`).
- UI native EventSource uses raw string `<=` for reconnect dedupe and fetch streaming uses
  raw `>` (`apps/godspeed-cognitive-ui/src/adapters/http/httpCaseworkAdapter.ts:344-356,419-426`).
  Its `lastCursor` is retained in the subscription closure across network reconnects, not
  persisted by this adapter across a page/app restart (`:297-377,380-447`). `compareCursor`
  instead splits on `.`, converts components with `Number`, and subtracts
  (`src/ports/project.ts:146-151`); ULID input produces `NaN` comparisons. It is used to
  sort history in `projectHistory` (`:127-143`), live revisions (`src/app/live.ts:31`), and
  the stale-intent refresh (`src/app/intents.ts:93`).

In contrast, Go intent freshness compares `client_cursor` for exact inequality with the
latest case cursor (`internal/intents/intents.go:438-446`), and exact historical lookup is
by cursor key. Neither operation requires lexical ordering.

## Retention, duplicates, restarts, and known mixed formats

The Go projection Store is process-memory-only and retains 512 global revisions by default
(`internal/projection/store.go:15-29,39-60`). It indexes only retained cursors in
`byCursor`; once an entry is evicted, Store alone cannot distinguish that cursor from one
never seen. The SFWP ledger is durable, but the Go relay's cursor maps and projection
history are rebuilt in memory after restart. The Go subscription object also keeps
`LastCursor` in memory only. The Tauri event host is the concrete persisted-cursor consumer
found in this tree; its persisted value is an unvalidated string and its recovery loop pages
by exact kernel cursor until an empty result. No repository evidence establishes that a
persisted production SFWP cursor used dot syntax.

The Go SSE path registers against the current Store, then filters retained replay by
lexical `Cursor > after`; it checks an old-position gap by lexical `last < oldest`
(`server.go:288-330`, Store `:196-211`). Since Store is empty after a Go process restart,
there is no previous retained ordering basis in that process. A client may still present a
prior cursor; the current empty-store branch emits no resync marker. Within a running Go
process the UI reconnect closure preserves `lastCursor`; the Store's bounded window also
means a cursor can fall outside retained history. Duplicate suppression outside the
retained window therefore depends on the upstream resume position/ordered feed, not on an
all-history duplicate set (none is present).

The Go UI's history is reconstructed from the Go trajectory endpoint (whose `Trajectory`
walks Store revisions in retained arrival order) and then currently re-sorted with
`compareCursor`. The endpoint/TS response has `base_cursor`, `head_cursor`, and point
cursors as strings; it has no dedicated canonical JSON schema in this checkout. The UI
runtime trajectory guard checks nonempty strings and that base/head equal the first/last
point, not their lexical ordering (`httpCaseworkAdapter.ts:101-108`).

No persisted mixed production history was found in the inspected source. The clear mismatch
is between the frozen numeric mock/contract grammar and the live kernel ULID grammar. The
SFWP `system.hello` protocol negotiation accepts protocol major `1` (`sfwp/mod.rs:34,285-305,414-438`);
it does not negotiate cursor syntax. The casework HTTP routes are not versioned in the
adapter; canonical schema `$id`s are stable and unversioned, and no cursor-format capability
field was found. The contract spec itself is version `0.2.5`.

## Cursor-shaped interface fields found

- Canonical JSON Schema explicitly constrains `CognitiveWorldSnapshot.cursor`, intent
  `client_cursor`, shared SSE envelope `cursor`, duplicate named
  `execution_observation.cursor`, and `InterruptedPayload.last_cursor` to numeric-dot
  syntax (`.agents/reports/interface-contracts/schemas/cognitive-world.schema.json:29-33`,
  `interaction-intents.schema.json:53-56`, and `event-stream.schema.json:24-27,176-183,212-215`).
- `ResyncRequiredPayload.requested_cursor` and `oldest_available_cursor` are plain strings,
  without patterns (`event-stream.schema.json:200-204`). The TypeScript interrupted
  `last_cursor` and resync fields are also plain strings (`typescript/types.ts:458-473`).
- Temporal trajectory `base_cursor`, `head_cursor`, and each point cursor are plain strings
  in TypeScript (`typescript/types.ts:406-423`); no dedicated trajectory schema was found.
- Current world/event/intent Go DTOs use strings and preserve raw cursor values; Go server
  stale-intent logic requires equality, not ordering.

This inventory records current source behavior only. It does not authorize changes to
cursor syntax, runtime ordering, schemas, or persisted formats.

🌱 graft saved ~44,893 tokens (~$0.04) this turn (1 call); ~18,524 tokens (~$0.01) this turn (1 call).
