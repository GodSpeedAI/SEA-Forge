# Go casework boundary wire contract (cognitive projection API)

Version 1 (fixture-labeled provider). The Go application owns operational coordination; SEA-Forge
owns governed authority (not wired in this milestone: the fixture provider is honestly labeled and
must never be presented as governed integration).

All responses are strict JSON (UTF-8). Every projection response carries the UI contract shapes
with EXACTLY these field names (camelCase), matching `apps/godspeed-cognitive-ui/contracts/world.ts`.

## Objects

```jsonc
{
  "id": "eng-northstar",
  "kind": "engagement",
  "label": "Northstar Health",
  "position": { "x": 5.2, "y": 1.4, "depth": 0.5 },
  "salience": 1.0,
  "parentId": null,            // omitted or null when top-level
  "note": "Claims pilot - one path unresolved",
  "attention": "notable"       // "notable" | "requires-judgment" | omitted (= quiet)
}
```

Snapshot: `{ "cursor": 1150, "surfaces": [...], "objects": [...], "relationships": [...], "provenance": "go:fixture:northstar" }`
Surface: `{ "id", "label", "objectIds": [...] }`; relationship: `{ "from", "to", "kind" }`.

## Endpoints (default bind 127.0.0.1:4179, loopback only)

- `GET /api/healthz` -> `{ "status": "ok", "provenance": "go:fixture:northstar", "liveCursor": 1150 }`
- `GET /api/world` -> `{ "snapshot": <snapshot> }` (live revision)
- `GET /api/world?cursor=900` -> `{ "snapshot": <snapshot at that cursor> }`; unknown cursor -> 404
  `{ "error": { "kind": "invalid", "note": "unknown cursor" } }`
- `GET /api/events?last=<cursor>` -> SSE stream, `text/event-stream`:
  - event `hello`  data `{ "provenance": "go:fixture:northstar", "liveCursor": N }`
  - event `revision` data `{ "snapshot": <snapshot> }` (one per world revision; replay any with
    cursor > `last` before streaming live ones; `id:` field = cursor)
  - event `lease` data `{ "id": "...", "state": "claimed|active|released", "summary": "..." }`
- `POST /api/intents` body `{ "id", "kind", "target", "parameters", "cursor" }`
  -> 200 `{ "status": "accepted", "note": "...", "lease": { "id", "state", "summary" } }`
  -> 200 `{ "status": "refused", "reason": "authority_denied|unavailable|invalid|stale_projection", "note": "..." }`
  Refusals are NORMAL outcomes with HTTP 200. Same `id` + identical body replays the recorded
  outcome (idempotency); same `id` + different body -> `{ "status": "refused", "reason": "invalid" }`.
  `parameters.cursor` must equal the provider's liveCursor when supplied, else `stale_projection`.
- `POST /api/artifacts` body `{ "ref", "boundObject", "title" }` -> `{ "status": "accepted" }` |
  refused same shape (fixture-scoped durability: survives on the running process; never claimed as
  governed persistence).
- `GET /api/artifacts/{ref}?level=minimal|summary|source` -> raw bytes with the level's mediaType.

## Consequential flow (fixture demo)

`POST /api/intents` with kind `propose-consequence`, target `ns-migration`, parameters
`{ "action": "implement", "approach": "compat-layer" }`:
validate -> fixture authority allowlist -> lease `claimed` -> `active` -> append world revisions
(the compatibility-layer object appears, notes progress, the world quiets) -> `released`.
Every revision streams over SSE; the UI world reorganizes from these events alone.
