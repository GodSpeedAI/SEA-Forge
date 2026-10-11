# Live cursor contract V2 — independent source review

Date: 2026-10-05  
Verdict: **REJECT for operator review in its current form.** V2 substantially
repairs the first proposal's syntax-only approach, but captured historical facts,
feed acknowledgement, and cold-store resync semantics remain unresolved. This
review is source-only; it authorizes no contract, schema, runtime, or version
change.

## Evidence reviewed

Reviewed the original `live-cursor-contract-root-proposal-oct05.md`, its
independent rejection, V2, `live-cursor-contract-independent-recon-oct05.md`,
`observation-case-cursor-recon-oct05.md`, and
`live-cursor-ordering-source-facts-oct05.md`. Checked the frozen T01 cursor
contract, the approved T09 amendment at
`.agents/reports/casework-live-wiring/t09-contract-extension-proposal.md`, the
governing `.agents/specs/godspeed.casework-cognitive-environment-spec.yaml`,
canonical cursor schemas, and current Go/Rust/UI source anchors below.

The producer mismatch is verified: Rust constructs `EventFrame.cursor` from
`entry_ulid` (`crates/sea-forge-server/src/sfwp/events.rs:59-100`); ULID is an
opaque record identity while `append_ordinal` is authoritative order
(`.agents/specs/spec-full.md:544-557`). Kernel `get_range` resolves exact cursor
identity to ordinal and returns events in ledger order, exclusive of the
`from_cursor` and capped at 500 (`events.rs:125-192`). The canonical snapshot,
intent, shared event, observation, and resync `last_cursor` fields still contain
numeric-dot constraints (`schemas/cognitive-world.schema.json:29-33`,
`interaction-intents.schema.json:53-56`, `event-stream.schema.json:24-27,176-183,215`).
T01 and T09 text still describe the earlier logical cursor model.

## V2 improvements and material differences from the original

V2 changes the model from “accept numeric-dot or ULID while leaving ordering
unchanged” to opaque durable addresses with no cross-format ordering claim. It
adds the missing resync `last_cursor` arm and explicit scope for adjacent opaque
fields; describes a public semantic revision, matching gateway/UI rollout,
spec version `0.3.0`, stable schema IDs, ADR, and decision-log approval; and
replaces raw cursor ordering with an ordinal-ordered `events.get_range` feed.
Subscription frames become wake hints, with periodic reconciliation, paging,
backpressure, and cursor advancement after delivery. It expands runtime scope
to feed, relay, Store, SSE replay, mutation waiters, and UI ingestion/history;
defines bounded observation identity-cache limits and resync invalidation; and
adds stronger ordering, restart, retention, reconnect, and no-conversion proofs.
These are substantial and necessary corrections to the original proposal.

## Blocking gaps

1. **Replaying events cannot recreate captured historical facts.** V2 says a
new gateway process rebuilds from no cursor and asks to “verify existing
captured-fact replay semantics,” but supplies no safe rebuilding rule. Current
`LiveSource.Facts` reads current case/run views and stamps the supplied cursor
(`internal/projection/live.go:43-137`). `Relay.accept` calls it for each event,
then appends those facts under that event cursor
(`internal/server/relay.go:101-140`). `Store.Revision.Facts` is explicitly the
immutable authority view captured for that cursor (`internal/projection/store.go:32-52`),
and SSE history renders retained revisions from it (`server.go:402-424`).
There is no source API to fetch historical case facts as of an event cursor.
Replaying old ledger events through `Facts` after restart would therefore label
current authority state with old cursors, manufacturing historical facts and
violating the frozen-history semantics. V2 must specify how restart recovery
avoids that, or explicitly change the guarantee through the required contract
decision; the current “verify and report” deferral does not resolve it.

2. **Range advancement is acknowledged too early.** V2 advances its exact
resume token after delivering a validated event to a bounded consumer channel.
That is not proof the revision was accepted into Store. In current
`Relay.accept`, observed cursors advance before source rebuilding; a `Facts`
failure returns without `Store.Append` (`relay.go:101-140`). If the new feed has
already advanced past that event on channel delivery, a transient projection
failure leaves a permanent gap that later range pages cannot repair. Define an
acknowledgement boundary after successful projection/append (or an explicit
retry/dead-letter policy) before claiming ordered, gap-free revisions.

3. **Cold-store resync has no valid boundary yet.** V2 requires an unknown or
evicted nonempty token to produce `resync_required` followed by retained replay,
and separately says a new process starts replay from no cursor. At startup the
Go Store is empty; current SSE only emits resync when `Oldest()` is nonempty
(`server.go:327-337`). Before the range feed has rebuilt any retained revision,
there is no real oldest cursor to place in the resync envelope. Define a
readiness barrier or a valid no-retained-history response that preserves the
required real cursor and does not claim continuity. Exact map lookup alone does
not settle this case.

The range-driven Go design does address the primary Go ordering defect if fully
implemented: current relay and Store compare cursor strings (`relay.go:103-107,156,186`,
`store.go:153-160,196-211`), and UI compares/sorts them lexically or numerically
(`httpCaseworkAdapter.ts:347,419`, `ports/project.ts:133-151`). V2 correctly
routes ordinary ordering through the ordered range feed and source arrays. The
remaining Rust `start_subscription` lexical overlap check
(`crates/sea-forge-server/src/lib.rs:1995-2058`) is acknowledged as separate
CW-09; it can remain outside this Go-path correction only if range polling is
truly authoritative and periodic recovery covers dropped hints.

## Authorization and disposition

V2 now states a concrete version consequence and correctly calls this a public
semantic revision. The governing spec is `0.2.5` and requires prior review,
approval, and ADR/spec updates (`change_rule`). V2 remains explicitly held for
operator review; this independent source review cannot supply that
authorization. Keep schemas, runtime, generators, and IDs unchanged until the
operator decision and the blocking restart/acknowledgement/resync semantics are
resolved. No compiler, tests, scanners, schema/runtime edits, Git, status, or
debt operations were performed.
