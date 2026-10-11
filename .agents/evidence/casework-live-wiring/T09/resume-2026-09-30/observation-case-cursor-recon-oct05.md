# T09 case-cursor availability for SSE observation

Read-only source recon, 2026-10-05. No source, test, status, debt, or Git files were changed; no compiler or test command was run.

## Finding

Yes. `Relay.CursorForCase(caseID)` can be absent (`"", false`) at the start of an explicit-case observation for a case with real runs. Relay state is process-local and starts with empty `caseCur` (`apps/godspeed-casework-go/internal/server/relay.go:66-75`). It is populated only after the feed yields a frame with both nonempty case ID and cursor (`relay.go:101-111`). `Run` begins consuming the feed asynchronously (`relay.go:78-97`), and `CursorForCase` is a direct map lookup (`relay.go:170-176`). Production startup creates an empty projection store, subscribes, then starts the relay goroutine (`apps/godspeed-casework-go/cmd/godspeed-casework/main.go:226-244`); no case-list/bootstrap path seeds `caseCur`.

Thus a case and its runs can already exist in authority while the gateway has not yet observed that case's replay/live frame. This is especially possible during startup. A frame whose source rebuild or store append fails still advances `caseCur` before the failure path (`relay.go:99-111,131-150`), so cursor availability is not equivalent to retained projection availability.

The projection store exposes `Trajectory(caseID)`, which returns retained case revisions in cursor order, or empty when the process has no retained revision for that case (`apps/godspeed-casework-go/internal/projection/store.go:122-140`). In current production it is also created empty at startup. Its latest retained cursor is a real prior event cursor if present, but may lag a cursor already observed by the relay after a rebuild/append failure; it is therefore not an unconditional substitute for `CursorForCase`.

The world endpoint itself tolerates the missing relay cursor: after choosing/accepting a case ID it ignores the bool from `CursorForCase` and calls `Snapshot` with the possibly empty string (`apps/godspeed-casework-go/internal/server/server.go:209-230`). `LiveSource` can independently query case/run facts; this makes “case has runs” insufficient proof that a relay cursor has arrived.

## Contract and UI implications

The canonical event envelope requires `cursor`; its schema pattern is `^[0-9]+\\.[0-9]{10}$`, with no separate `minLength` declaration (`.agents/reports/interface-contracts/schemas/event-stream.schema.json:5-27`). The specific `ExecutionObservationEvent` repeats the requirement and pattern and describes the cursor as informational/latest real case cursor (`event-stream.schema.json:174-184`). Empty string cannot satisfy that pattern. Canonical TS types also require `cursor: string` (`.agents/reports/interface-contracts/typescript/types.ts:438-447`). The interface contract says observation copies the latest real case cursor known to the gateway, creates no logical cursor, carries no SSE ID, does not advance the client's case cursor, and must be routed before ordinary cursor comparison (`.agents/reports/interface-contracts/04-REACT-COGNITIVE-ENVIRONMENT-CONTRACT-SPECIFICATION.md:227-233`).

The UI wire-contract test uses a weaker `^\\d+\\.\\d+$` regex, which still rejects empty strings but does not enforce the canonical ten-digit sequence width (`apps/godspeed-cognitive-ui/src/ports/wireContract.test.ts:416,489-495`). Current HTTP adapter routing performs ordinary monotonic-cursor comparison before event-type handling, advances `lastCursor` for non-resync events, and its native EventSource listener list omits `execution_observation` (`apps/godspeed-cognitive-ui/src/adapters/http/httpCaseworkAdapter.ts:340-359`). The fetch-stream path likewise compares and advances the revision cursor for ordinary events (`:416-427`). This is a current implementation gap relative to the approved observation contract, not a reason to change the contract.

## Compatible behavior candidates for the architecture owner

* Emit an observation only once a real case cursor is available from the relay; until then, keep the connection without an observation frame or close/retry through existing transport behavior. This avoids inventing a cursor or an invalid event envelope.
* If the architecture elects to consult the projection store, a nonempty case trajectory can supply an exact retained real cursor; use it only with an explicit freshness policy because it can lag the relay's observed cursor. An empty trajectory offers no cursor fallback.
* If a bounded wait expires before any real cursor is known, report/defer via a transport-level outcome rather than an `execution_observation` with empty/synthetic cursor. Any alternate SSE event envelope remains subject to the same required-cursor schema.

These are source-compatible options, not an implementation recommendation or authorization. No cursor should be derived from trace/event IDs, global head, timestamps, or sequence fabrication.

## Retrieval record

Graft was used before source inspection (`graft ask` for cursor/bootstrap and store behavior; `graft skeleton` for `projection.Store`). The exact source anchors above were then checked with line-numbered reads. Graft reported approximately 198,472 tokens saved this turn, worth about $0.15 at its displayed rates.
