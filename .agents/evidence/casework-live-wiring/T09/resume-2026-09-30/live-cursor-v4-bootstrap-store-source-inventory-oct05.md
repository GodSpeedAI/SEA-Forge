# V4 bootstrap and Store source inventory

Date: 2026-10-05  
Scope: source facts only; no design choice or implementation.

## World and template HTTP surface

`GET /api/world` is session-protected (`internal/server/server.go:137`). Its handler verifies the session perspective, accepts optional `case_id` or `cursor`, and currently wraps every success as `{ "snapshot": ... }` (`:177-230`; `worldResponse` at `:422-424`). Without an explicit case it calls `NewestCaseID`; if empty it returns HTTP 200 with `projection.EmptyWorld(actor, s.relay.Head(), now)` (`:209-220`). That empty snapshot therefore carries ordinary snapshot structure and a relay head, not a separate identity-free DTO. `writeTypedError` wraps `{error:{kind,note}}` (`internal/server/http.go:149-151`); case readiness/errors currently map to 502 in this handler (`server.go:211-228`).

Templates already have independent protected routes: `GET /api/templates` returns `{templates: TemplateEntryOption[]}` (`server.go:139,259-266,426-428`); each entry is `template_ref`, title, optional description, and parameters (`internal/contract/contract.go:141-147`). UI maps the same shape into `TemplateEntryOption` (`ports/contract.ts:234-247`). `POST /api/templates/preflight` is session+CSRF guarded and returns the authoritative preflight result (`server.go:146,268-285`). Existing `ProposalFlow` depends on store, a port with both methods, actor callback, and `focusCase` (`app/proposals.ts:24-27`); it has no direct case-history guard while opening/preflighting, but after commit calls `focusCase`, whose `loadAndFocusCase` loads history and dispatches `loadHistory` (`:105-122`). `initialState` and `nowRevision` require at least one history revision (`model/store.ts:12-18,47-49`), so a cursorless bootstrap cannot be represented by the current initialized `UiState` as-is.

## World ID and UI usage

UI runtime TypeScript references to `world_id` are only fixture/local producers; tests check presence/type. The live adapter widens snapshots by cast and its event parser validates cursor plus optional payload `case_id`, not `world_id` (`httpCaseworkAdapter.ts:530-555`). In these paths `world_id` is presentation/contract data, not an authority key; case identity and cursor drive history and stream selection (`project.ts:126-143`; `App.tsx:97-104`).

## Store, listener, and SSE behavior

The projection Store has global retention 512 and per-listener live buffer 256 (`projection/store.go:31-39`), a global `revisions` slice, cursor index, and unscoped `subs` (`:55-65`). `Subscribe(after)` replays every global revision with lexically greater cursor, registers under one mutex, and returns a cancel closure (`:193-226`). `Append` evicts globally when full; on a full subscriber channel it closes and removes that subscriber (`:180-188`). Handler cancellation removes/closes a still-registered channel; after overflow, deferred cancellation finds none. SSE selects request-context cancellation and the listener channel; channel closure returns (`server/server.go:345-359`). Writes/flushes ignore errors (`:320-325,345-374`; `http.go:184-198`). Production `http.Server` sets only `ReadHeaderTimeout: 10s`, no `WriteTimeout` (`cmd/godspeed-casework/main.go:271-275`); server options expose heartbeat but no per-write deadline (`internal/server/http.go:23-56`). Current candidate boundaries are global Store `Subscribe`/`Append` and `handleEvents`; there is no case-scoped subscription API or selected-case query (`server.go:138,293-317`).

## Kernel inventory and range interface

`case.list` takes no filter/paging fields (`internal/adapters/sfwp/frame.go:291-293`); result has newest-first cases and an `unreadable` list (`crates/sea-forge-server/src/sfwp/case_views.rs:143-152,310-355`). `events.get_range` takes optional exact `from_cursor`, `to_cursor`, and `limit` (`frame.go:185-198`), filters by append ordinal and caps returned events at 500 (`crates/sea-forge-server/src/sfwp/events.rs:34-37,165-208`). Kernel response is only `{events:[...]}` (`crates/sea-forge-server/src/lib.rs:2349-2361`); there is no frontier token in that result. `get_range` reads the durable entries for each call (`events.rs:174-176`); existing full-ledger allocation concern is separately recorded by root.
