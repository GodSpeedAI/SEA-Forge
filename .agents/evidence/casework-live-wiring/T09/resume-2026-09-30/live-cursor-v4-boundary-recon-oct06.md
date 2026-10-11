# Live cursor V4 boundary recon

Date: 2026-10-06  
Scope: read-only source and prior-evidence recon to inform a V4 proposal. This
note records facts and unresolved boundaries; it is not an architecture
proposal, approval, implementation authorization, or runtime evidence.

## Disposition

**Public implementation remains HOLD.** V3 was explicitly rejected for operator
approval by the independent review. This recon does not approve V3, authorize a
V4 contract, or release any public or kernel change. Preserve V3's useful
semantics while resolving the factual and interface gaps below. Any public
contract, schema, Rust/Go API, ID grammar, or architecture decision remains for
root and operator review.

## V3 record and unresolved deltas

The proposal is
`live-cursor-contract-root-proposal-v3-oct05.md`; the independent disposition is
`live-cursor-contract-v3-independent-review-oct05.md`. The latter rejects V3
for operator approval and retains HOLD. It accepts the direction of immutable
capture-time history, no fabricated cold backlog, exact opaque cursors and
ordinal ordering, explicit publication/gap disposition, real-boundary resync,
pre-header unavailability, and cursorless bootstrap outside ordinary history.

V3's material unresolved points are:

1. Its `world_id` producer claim is incorrect, and the frozen schema also
   disagrees with actual case ID production (details below).
2. Bootstrap is not an exact wire contract or concrete UI transition. It lacks
   an identified discriminant, property/type definitions, template shape, and
   failure behavior. The create-case exception for empty `case_id` and
   `client_cursor` must be explicit while retaining preflight authority.
3. `case.list` plus `events.get_range` does not currently establish a
   race-safe, bounded-cost frontier. A page cap is not a bound on ledger
   loading or repeated scan cost, and the response has no frontier token.
4. Store and SSE interfaces do not currently provide the proposed per-case
   readiness, invalidation, cancellation, and replay-registration boundary.

V3 also correctly forbids synthetic IDs/cursors, fake ordinary revisions,
relabeling present Facts as past history, lexical cursor ordering, and claims of
atomic as-of-event snapshot reconstruction. Keep these limits explicit.

## Identity producers and consumers

- Kernel case IDs come from `case_id()` -> `timestamped("case")` and therefore
  have the form `case_<UTC timestamp>_<hex>`:
  `crates/sea-forge-core/src/ids.rs:32-47`. The frozen schema instead requires
  `^case-[a-z0-9_.-]+$`, as recorded with its exact schema span in the V3 review
  (`.agents/reports/interface-contracts/schemas/cognitive-world.schema.json:24-27`).
  Do not silently translate or re-key existing case IDs.
- The Go ordinary snapshot producer sets `WorldID` to `"world-" + case ID` and
  copies the case ID and cursor unchanged:
  `apps/godspeed-casework-go/internal/projection/builder.go:99-104`. It does
  not append a cursor suffix. The frozen world schema requires
  `^ws-[a-z0-9_.-]+-[0-9]+\\.[0-9]{10}$` (V3 review:
  `.agents/reports/interface-contracts/schemas/cognitive-world.schema.json:19-32`).
  The UI local adapter also creates `world-${caseId}`:
  `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.ts:314-315`.
- V3's claim that deterministic `world_id` already embeds cursor syntax is
  therefore factually wrong. A cursor-union suffix change alone would not
  reconcile either actual producer with the frozen patterns. Resolve both
  producer/schema mismatches explicitly before any public contract edit.

## Existing proposal/bootstrap path

The UI already has a create-case path in
`apps/godspeed-cognitive-ui/src/app/proposals.ts:60-98`. It refuses submission
unless the selected template has a passing preflight and digest (`:64-65`), then
sends `PROPOSE_CASE` with empty `case_id` and `client_cursor` because no case
projection exists yet (`:68-78`). On acceptance it requires a returned case ID
and calls `focusCase` (`:85-92`); `loadAndFocusCase` loads case history before
dispatching `loadHistory` and focus (`:105-122`). Go's existing payload shape is
`ProposeCasePayload{TemplateRef, Params, PreflightDigest}` at
`apps/godspeed-casework-go/internal/contract/contract.go:336-341`.

Any V4 bootstrap contract must name the exact UI state and response variant,
preserve this preflight/`PROPOSE_CASE` route, and specify that ordinary history
and case-scoped streaming begin only after a validated ordinary snapshot with a
real cursor is available. Do not invent a kernel verb. The independent V3
review notes that current history initialization requires a revision and the
empty-world center action path offers no entry action; see that review's
bootstrap blocker and `apps/godspeed-cognitive-ui/src/model/store.ts:12-18,44-49`
and `src/app/App.tsx:238-253`.

## Kernel case inventory contract and failure behavior

`CaseListResult` is the existing success DTO `{cases, unreadable}` at
`crates/sea-forge-server/src/sfwp/case_views.rs:143-153`. Current `list` is
infallible (`:310-355`): any `read_dir(cases)` failure returns the default
empty result (`:314-317`); `flatten` drops iterator errors (`:319`); failed
directory metadata and non-UTF-8 names are skipped (`:320-325`); and
`read_case` maps metadata/read failures to `NotFound` (`:249-270`), which list
omits (`:336-341`). Only parse/size failures populate `unreadable`.

The `Request::CaseList` handler directly serializes the list DTO, while
case-view GETs have separate typed error mapping; the documented source options
are in `case-inventory-fail-closed-correction-source-options-oct06.md`. Thus
the Rust listing has no existing fallible operational `Result` channel to
propagate enumeration failure. By contrast, Go's
`Authority.ListCases(ctx) ([]CaseRecord, error)` already propagates transport
and decode errors but ignores `Unreadable` and accepts absent/null arrays as an
empty zero-value view:
`apps/godspeed-casework-go/internal/adapters/sfwp/authority.go:69-88`.
`NewestCaseID` treats empty as no cases and `handleWorld` returns `EmptyWorld`
with HTTP 200:
`apps/godspeed-casework-go/internal/projection/live.go:173-189` and
`apps/godspeed-casework-go/internal/server/server.go:209-220`.

The smallest no-wire-field correction recorded in
`case-inventory-fail-closed-correction-source-options-oct06.md` is a
Rust-internal fallible list path used by the production handler, plus strict Go
validation of required arrays and rejection of nonempty `Unreadable` through
Go's existing error return. Preserve empty success only for a genuinely valid
cell whose cases directory is absent or readable and empty. Do not flatten
iterator errors; retain known unreadable case IDs in the existing field.
Go-only strictness cannot detect inventory omissions already collapsed by
Rust. Root must choose a correct existing top-level error-class mapping or
request a reviewed public error change; this recon does not select one. Changing
the public Rust signature or Go port interface also requires API review.

This does not prove an atomic filesystem inventory: listing is not locked with
writers, and `CaseRunner::write_json` uses direct `fs::write` (source anchor
`crates/sea-forge-case-runner/src/lib.rs:570-575`). A complete-empty or
snapshot-atomicity claim must not be inferred from a successful scan.

## Range/frontier and create ordering

`events.get_range` accepts optional exact `from_cursor`, `to_cursor`, and
`limit`; it reads ledger entries, filters by append ordinal (exclusive after
`from`, inclusive through `to`), and caps returned frames at 500:
`crates/sea-forge-server/src/sfwp/events.rs:165-207`. The kernel response is
only `{events:[...]}`, with no `done` or frontier member:
`crates/sea-forge-server/src/lib.rs:2349-2361`. A short/empty page is not an
explicit frontier. The 500 cap bounds response frames, not the current
`read_entries()` materialization or repeated scan cost (V3 review cites
`crates/sea-forge-ledger/src/types.rs:784-801`).

The bounded source recon
`case-inventory-frontier-race-source-recon-oct06.md` establishes one narrower
ordering fact: on successful commit, durable case files and in-memory case
entry precede global `case.submitted` publication
(`crates/sea-forge-server/src/lib.rs:2788-2822`). Thus a frontier that includes
that event closes the specific successful-commit-after-frontier race, assuming
ordinary filesystem visibility. It does not prove full inventory, stable
filesystem snapshot semantics, or recovery of failed/partial dispatch. The
`CaseCreated` trace is in the per-case ledger, not the global range ledger.
The range contract currently cannot supply an atomic head token, and this
source recon does not prove one can be inferred under concurrent writes.

## Store/SSE boundary

The gateway `Store` is global and stores revisions in cursor order:
`apps/godspeed-casework-go/internal/projection/store.go:55-65`. `Append`
uses raw cursor comparison and broadcasts to all subscribers (`:149-190`);
`Subscribe` selects replay and registers under the Store lock (`:193-220`).
The SSE route has no selected-case input; it subscribes globally and writes
HTTP 200/hello before streaming (`apps/godspeed-casework-go/internal/server/server.go:293-325`).
These interfaces do not implement V3's proposed per-case 503 readiness,
case-scoped replay, or atomic gap invalidation/cancellation with subscriber
registration. V4 must provide a concrete reviewed decomposition before any
implementation; do not claim the current boundary already supports it.

## Evidence limits

This is a source/evidence recon only. It ran no tests, compiler, scanner,
Graft build, runtime experiment, or external action. It does not establish a
working implementation, an approved proposal, a complete inventory under
concurrency, atomic snapshot semantics, or operator authorization. Prior
reviews and recon records are inputs only and retain their own dispositions.
