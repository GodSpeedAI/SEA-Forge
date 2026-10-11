# Bounded ledger reader: source recon

**Scope:** read-only source recon for the Rust ledger and existing SFWP
`events.get_range` method. This is not an implementation, test, or runtime
approval. No source, schema, generated contract, or Git state was changed.

## Governing contract and protocol compatibility

The current C2 supplement defines the target in
`.agents/specs/casework-live-cursor-v4-spec.yaml`: `limits.event_page` (lines
90–104) caps a page at 500 scanned ledger rows, 500 emitted frames, 1,048,576
serialized response bytes, and 50 ms waiting for the ledger lock.
`REQ-C2-RANGE-001..003` (lines 203–230) requires optional
`range_version=2`, exact cursor plus canonical decimal-string ordinal per
frame, a real pinned head, scanned-through progress, continuation/completion,
and validation of every complete global row before event filtering. A
continuation binds the stream, pinned head, last validated row, byte position,
and checksum; resume verifies the preceding row. Non-event rows advance global
progress. An empty filtered page may progress; a partial or failed page never
proves completion. Omitted version keeps the legacy vector response, but may
succeed only after bounded proof through the actual pinned head; failure is
typed unavailable with no partial vector. The u64 rule in `REQ-C2-U64-001`
(lines 156–167) requires canonical unsigned decimal strings at JSON boundaries
and checked u64 handling in Rust/Go.

ADR-008, “Global event frontier and case projection” (pp. 4–5, especially
lines 76 and 133–137), approves adding optional `range_version=2` to the
existing `events.get_range` method and expressly says to add no kernel verb.
That is consistent with the additive compatibility convention in ADR-003
(“Established compatibility convention”): optional additions use
`#[serde(default)] Option<T>`, omit absent values where serialized shape must
stay old, and do not require a version bump. The SFWP module’s opening comment
(`crates/sea-forge-server/src/sfwp/mod.rs:1–14`) says new capabilities are new
verb variants and existing variants are not reshaped; the approved range
version is a mode on an existing method, not a new capability/verb. Preserve
the existing request fields and their types/defaults (`from_cursor: Option<String>`,
`to_cursor: Option<String>`, `limit: Option<u32>`) and their omitted-version
behavior. In particular, no-version requests must continue to receive the
existing `{"events": [...]}` response containing the legacy `EventFrame`
shape. Do not add a required field, rename an old field, change its type, or
add a verb. `Request` is Deserialize-only (`lib.rs:642+`); old clients’ JSON
shape is the compatibility concern, not server serialization of `Request`.

If v2 adds ordinal/page metadata, a separate v2 frame/response DTO is the
least disruptive option. `EventFrame` is also used by live broadcast and
`events.subscribe` (`lib.rs:34, 113, 166, 214`), so adding fields to it can
change those existing payloads as well as `get_range`. That choice should be
settled before implementation. Existing `IMPLEMENTED_METHODS` already lists
`events.get_range` (`sfwp/mod.rs:108–136`); no method-catalog addition is
needed.

## Current implementation and narrow ownership boundary

`crates/sea-forge-server/src/sfwp/events.rs:169–208` implements `get_range`
by calling `ledger.read_entries()`, resolving cursors by scanning the
materialized vector, filtering `record_kind == "sfwp_event"`, applying
exclusive-from/inclusive-to ordinal bounds, then taking up to the 500-frame
cap. `read_entries()` in
`crates/sea-forge-ledger/src/types.rs:785–802` parses every nonblank JSONL row
into a `Vec<LedgerEntry>`. It neither bounds total history nor validates
ordinal/hash continuity. `read_last_entry()` (`types.rs:765–782`) also scans
the complete file. `verify()` (`types.rs:1093–1150`) checks contiguous
ordinals, predecessor links, payload and entry hashes, ULID uniqueness, and
MMR consistency, but is an unbounded full-ledger operation; it is not a page
reader.

The smallest implementation partition is:

1. **Synchronous ledger primitive** in `sea-forge-ledger`: bounded sequential
   JSONL row reads with a byte position, checked expected ordinal, exact row
   cursor and hashes, prior-row link/checksum, and a pinned real head pair.
   This layer owns offsets and row boundaries. Keep it synchronous: the crate
   boundary instructions keep ledger/kernel crates out of Tokio/async.
2. **SFWP projection** in `sfwp/events.rs`: scan global rows through that
   primitive, validate them before filtering, then construct event frames and
   continuation/completion metadata. Count every ledger row against the scan
   limit, including non-event rows; only event rows count toward emitted
   frames. Enforce serialized response size before success. The existing
   `get_range` path branches on optional version while retaining its current
   vector output for omitted version.
3. **Existing Request dispatch** in `lib.rs:748–790, 2349–2360`: add only the
   approved optional version to `EventsGetRange`, route v2 to the new page
   projection, and preserve old fields, defaults, response, and error envelope
   for legacy requests.
4. **Schema ownership**, if the v2 DTOs are public: `EventFrame` currently
   appears in both `sfwp::SCHEMA_TYPES` (`sfwp/mod.rs:345–410`) and the
   generated schema list (`bin/gen_sfwp_schema.rs:35, 55–66`). Add any new
   public DTO consistently in both locations and to the existing conformance
   oracle; do not hand-edit generated Workbench schema output. `Request` itself
   is not part of that generated type list.

## Gaps the source does not resolve

* The ledger’s `with_exclusive_lock` (`types.rs:702–724`) uses blocking
  `File::lock()` and has no 50 ms timeout or bounded read counterpart. A
  reader must define how it obtains a consistent pinned head within the
  approved wait, and how it reads that append-only prefix without turning a
  long page into an unbounded writer lock. No existing ledger lock helper
  demonstrates this behavior.
* A continuation checksum plus preceding-row verification does not, by itself,
  establish that a caller-supplied offset follows a previously validated
  prefix. The spec requires binding and verification but does not state whether
  continuations are authenticated opaque tokens, server-held state, or
  independently revalidated from origin. Revalidating from origin defeats
  bounded paging. Resolve the proof model explicitly; do not silently trust a
  client-supplied offset/checksum.
* The page limits specify row count and serialized response bytes but no
  explicit maximum raw bytes for one scanned JSONL row or aggregate bytes
  parsed across 500 rows. Current `.lines()` allocates each complete line.
  A bounded reader needs a defined byte-allocation/parse bound, including
  malformed or non-event rows, before claiming resource boundedness. Do not
  infer that response-size cap bounds input allocation.
* `append_under_lock` derives the next ordinal with unchecked `u64 + 1`
  (`types.rs:1029`); a checked reader must refuse overflow, while append-side
  overflow behavior is a separate writer concern.
* `frame_from_entry` (`events.rs:92–105`) defaults malformed event `kind` to
  an empty string and missing `detail` to `null`. C2 requires malformed rows
  to fail closed. The new authoritative path must validate event payload shape
  rather than inherit these defaults.
* Existing unknown cursors return `ForgeError::Input` with
  `events_unknown_cursor` (`events.rs:110–130`), and conformance tests expect
  `input_error` and no frames (`tests/conformance_sfwp.rs:1160–1185`).
  `ForgeError` has no dedicated frontier/unavailable variant
  (`crates/sea-forge-core/src/errors.rs:4–27, 33–66`); mapping the new typed
  unavailable failure into the existing response envelope/class needs an
  explicit boundary decision. Do not relabel unavailable as unknown input or
  successful empty history.

## Focused compatibility and correctness oracles

Preserve current behavior checked in
`crates/sea-forge-server/tests/conformance_sfwp.rs:491–537` (gap recovery from
an exclusive `from_cursor`, durable events only) and `:1160–1185` (unknown
cursor refuses with no replay). Add focused tests for: omitted version keeping
the old request/response shapes; v2 page schema and decimal u64 extremes;
global ordinal progression across non-event and sparse rows; empty filtered
pages with progress; stable pinned-head continuation over concurrent appends;
forged, stale, malformed, and byte-offset continuations; broken ordinal/hash
chains and malformed event payloads; exact scan/frame/response/lock limits;
and legacy refusal with no partial vector when bounded coverage cannot reach
the actual pinned head. These are test boundaries, not claims that any test
was added or run.

## Provenance and scope

This recon used Graft first and then checked the cited source/spec ranges. It
is limited to Rust ledger + SFWP event range. It does not audit or approve the
Go frontier, Store, journal, inventory, SSE, UI, historical reconstruction,
writer migration, or runtime behavior. No commands compiled or tested code.
Graft retrieval saved approximately 101,856 tokens ($0.08) in the two calls in
this turn.
