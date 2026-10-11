# Live cursor V4 bootstrap component proposal

Date: 2026-10-06  
Status: **DOCONLY component proposal; public implementation and full V4 remain
HOLD.**  
Scope: specify the cursorless empty-cell response and its UI/create transition
using existing case-entry authority. This proposal does not settle case/world
ID grammar, case inventory completeness, ledger frontier discovery, Store/SSE
architecture, or any public-contract release decision.

## 1. Boundary and invariant

Keep ready `GET /api/world` ordinary responses exactly in the current envelope
shape `{ "snapshot": CognitiveWorldSnapshot }`. Add a separate mutually
exclusive response arm:

```json
{
  "bootstrap": {
    "status": "no_cases",
    "timestamp": "<RFC3339 actual capture time>",
    "perspective": { "actor_id": "...", "role": "..." },
    "templates": []
  }
}
```

The bootstrap arm contains no `snapshot`, `world_id`, `case_id`, `cursor`,
history, revision, or synthetic identity. The timestamp is when this HTTP
handler captured the response, not a kernel event timestamp. `perspective` is
the existing `ActorPerspective` built from the session-verified actor. `templates`
is the existing `TemplateEntryOption[]` mapped from `case.entry_options` by the
existing Go template source. It may be empty when no template is published; an
empty template list is not itself evidence that the case inventory is empty.

This response says the complete inventory procedure established no cases at
that observation. It does not assert a filesystem snapshot or atomicity with a
concurrent commit. A later authenticated refresh discovers a case that appears
after the bootstrap capture. Do not add or infer a kernel cursor/frontier for
this purpose.

**The bootstrap success arm is not releasable yet.** Existing `case.list` can
collapse inventory failures to empty and Go ignores its `unreadable` field.
Until the separately held fail-closed inventory correction and its complete-
empty proof are approved and implemented, an uncertain, unreadable, or failed
inventory returns an error; it must never serialize as `bootstrap`. See
`case-inventory-fail-closed-correction-source-options-oct06.md`,
`case-inventory-frontier-race-source-recon-oct06.md`, and
`live-cursor-v4-boundary-recon-oct06.md`.

## 2. Response contract and source mapping

Author one route-level JSON Schema, proposed path
`.agents/reports/interface-contracts/schemas/world-response.schema.json`, with
a root `oneOf` containing exactly these wrapper arms:

1. Snapshot arm: required `snapshot`, the existing cognitive-world schema
   reference, `bootstrap: false`, and `additionalProperties: false`.
2. Bootstrap arm: required `bootstrap`, a strict `WorldBootstrap` object,
   `snapshot: false`, and `additionalProperties: false`.

The bootstrap object has `additionalProperties: false`; all four named fields
are required. `status` is `const: "no_cases"`; `timestamp` is a date-time
string; `perspective` reuses the existing actor-perspective contract shape;
`templates` is an array of the existing template-entry-option shape. Specify
the parameter fields from the existing DTO: required `name` and `type`;
optional `title`, `description`, `required`, `default_value`, and `options`;
`type` is the current `string | number | boolean | enum` set. Do not narrow or
change ordinary snapshot fields as part of this response-arm work. Keep
existing ID/schema reconciliation a separate HOLD.

Concrete source-level variants for the later implementation are:

```go
type WorldBootstrap struct {
    Status      string                `json:"status"`
    Timestamp   string                `json:"timestamp"`
    Perspective ActorPerspective      `json:"perspective"`
    Templates   []TemplateEntryOption `json:"templates"`
}

type worldResponse struct {
    Snapshot  *CognitiveWorldSnapshot `json:"snapshot,omitempty"`
    Bootstrap *WorldBootstrap         `json:"bootstrap,omitempty"`
}
```

The Go response constructor must set exactly one pointer; handler tests reject
both or neither. The bootstrap constructor sets `Status` to the literal above,
uses an actual UTC RFC3339 timestamp, and obtains actor perspective from the
already-verified session. `worldResponse` is currently private to the server;
`CognitiveWorldSnapshot`, `ActorPerspective`, and `TemplateEntryOption` already
exist in `apps/godspeed-casework-go/internal/contract/contract.go:50-83,141-147`.

The corresponding UI-facing type is:

```ts
export interface WorldBootstrap {
  readonly status: 'no_cases'
  readonly timestamp: string
  readonly perspective: ActorPerspective
  readonly templates: readonly TemplateEntryOption[]
}

export type WorldResponse =
  | { readonly snapshot: CognitiveWorldSnapshot; readonly bootstrap?: never }
  | { readonly bootstrap: WorldBootstrap; readonly snapshot?: never }
```

Use the canonical `ActorPerspective` from
`.agents/reports/interface-contracts/typescript/types.ts` and the existing
`TemplateEntryOption`; export the former through the app port without defining
a second perspective/template DTO. Runtime JSON validation is a required new
implementation step: current source has no runtime `CognitiveWorldSnapshot`
validator (the existing `wireContract.test.ts` is test-only), and `getSnapshot`
blindly casts a `{snapshot}` body
(`apps/godspeed-cognitive-ui/src/adapters/http/httpCaseworkAdapter.ts:233-240`).
Validate the response union and ordinary snapshot against the authored
contract before branching. A response with both/neither arm, unknown
wrapper/bootstrap properties, wrong status, malformed timestamp/perspective/
template data, or an ordinary snapshot that fails that new validator is
unavailable, never an empty world. Keep case-specific `getSnapshot` ordinary-
only.

Implementation source map (no edits in this proposal):

- Authored wire schema: new `schemas/world-response.schema.json`, reusing
  `schemas/cognitive-world.schema.json`; add strict bootstrap/snapshot-arm and
  exclusive-arm conformance cases to
  `tests/contract-conformance.test.ts` and a bootstrap golden beside
  `golden/world-snapshot.json`.
- Type definitions: add `WorldBootstrap` and `WorldResponse` to
  `.agents/reports/interface-contracts/typescript/types.ts`, then export/use
  them and `ActorPerspective` at `apps/godspeed-cognitive-ui/src/ports/contract.ts`.
  Existing template types are already defined in `types.ts:549-567` and
  re-expressed at `ports/contract.ts:223-247`.
- Go DTO: add the response payload type alongside the existing Go contract
  types in `apps/godspeed-casework-go/internal/contract/contract.go`; change
  only the private response wrapper and constructors in
  `internal/server/server.go`.
- HTTP/UI adapter: add a typed bootstrap-aware unqualified-world read and
  runtime union validator to `apps/godspeed-cognitive-ui/src/adapters/http/httpCaseworkAdapter.ts`;
  preserve `getSnapshot(caseId, ...)` as ordinary-only.
- Do not hand-edit generated files. No generator or generated output is being
  changed or authorized by this proposal.

## 3. HTTP selection and server decision order

The endpoint remains protected by the current session middleware. The
unqualified request uses the current session perspective. Preserve the existing
cursor-addressed ordinary snapshot branch. For requests without a historical
cursor:

1. Distinguish absence of `case_id` from a supplied value. No `case_id` means
   unqualified current world; exactly one nonempty supplied `case_id` means
   explicit selection. Reject a supplied empty or duplicated `case_id` as an
   invalid request. Never silently redirect an explicit request to another
   case, return bootstrap for it, or substitute a different case ID.
2. For explicit selection, return only a ready ordinary snapshot for that exact
   case. Unknown case, unavailable authority, missing observed cursor, or
   unready projection is an error; no cursorless bootstrap arm is eligible.
3. For unqualified selection, obtain a fail-closed complete inventory. If it
   contains cases, return a ready ordinary snapshot for the selected existing
   case. A known case without a real ready cursor is unavailable, not empty.
4. Only if a successfully completed, corrected inventory proves zero cases may
   the handler call the existing template source and build the bootstrap arm.
   If the template source fails, return its existing typed unavailable error;
   do not send a partial bootstrap. A valid empty template array is permitted.
5. If inventory is incomplete, unreadable, or fails, return a typed error using
   an existing class only after root selects the correct mapping. This proposal
   does not pick or broaden an error class. No error condition may be converted
   to `no_cases`.

Current route facts: `GET /api/world` is session-protected and calls
`verifySessionPerspective` (`internal/server/server.go:135-141,177-186`). The
current handler treats `Query().Get("case_id") == ""` as unqualified and can
return `projection.EmptyWorld` with HTTP 200 (`:209-220`); these behaviors need
the exact presence/empty distinction above. `GET /api/templates` is already
session-protected (`:139`) and returns `templatesResponse` (`:259-266,426-428`).
`LiveSource.EntryOptions` maps existing kernel template options to the existing
DTO (`internal/projection/live.go:207-239,295-302`). No new kernel verb or
template representation is needed.

The listing correction is a prerequisite, not part of this component release.
Source currently shows `case_views::list` returning a DTO rather than a
fallible result and collapsing `read_dir` failures (`crates/sea-forge-server/src/sfwp/case_views.rs:143-153,310-355`); Go
`Authority.ListCases` drops `Unreadable` (`apps/godspeed-casework-go/internal/adapters/sfwp/authority.go:69-88`). A false
empty must remain an error. Neither this handler nor the UI may claim an atomic
filesystem or range frontier; `events.get_range` has no returned frontier
token.

## 4. UI loading, bootstrap, and transition

### Startup

The current live startup in
`apps/godspeed-cognitive-ui/src/main.tsx:65-99` calls `loadCaseHistory` before
mounting `App`; live startup supplies an empty string when there is no `case`
URL parameter. `App` immediately requires `initialState(history)` and
`nowRevision(history)` (`app/App.tsx:55-62`, `model/store.ts:12-18,47-49`),
then connects a stream for `history.caseId` (`App.tsx:97-104`). There is no
honest empty `WorldHistory` representation.

Add a live startup response branch above `App`, owned by `SessionApp` or an
equivalent entry shell:

- Resolve and validate session first, as today.
- If there is no `case` query parameter on the live path, call the
  bootstrap-aware unqualified-world method. If the URL explicitly supplies
  `case`, require exactly one nonempty value and request that case only; errors
  remain errors and cannot redirect.
- On a validated ordinary arm, only then call the existing history loader for
  that exact case and mount `App`. Seed/fallback must come from a real validated
  snapshot; do not synthesize a revision, cursor, case ID, or history.
- On a validated bootstrap arm, render a standalone bootstrap screen/state
  outside `UiState.history`, `WorldHistory`, `rawByCase`, `history.snapshots`,
  cursor deduplication, trajectory, and SSE. Do not mount `App` with a fake
  one-point history. Render actor perspective, actual capture time, and the
  returned templates. The no-template state says no templates are published
  and provides no fake entry.
- Loading and unavailable are separate UI states from the validated
  `no_cases` state. Authentication/session failure returns to the login or
  session-ended path, never bootstrap.

The current center action returns no actions without a focused object
(`apps/godspeed-cognitive-ui/src/app/App.tsx:238-253`). The normal
`TemplateDesignPanel` is already usable for loading, unavailable, empty
templates, required parameters, passing/failed preflight, and commit
(`src/ui/TemplateDesignPanel.tsx:20-149`). Bootstrap must provide an explicit
"Design case" action outside `App` and render this same panel/component.
`createProposalFlow` is presently coupled to the main `Store`/`UiState` and
`focusCase` (`src/app/proposals.ts:12-27,60-103`); constructing a fake main
store to reuse it would violate this contract. Extract the proposal-specific
state/controller from `UiState` so both the ordinary App and standalone
bootstrap shell can use the same preflight/submit logic and panel without
inventing a history. This is a UI implementation seam and remains held.

The bootstrap supplies template options already. Opening its picker should
seed the shared proposal controller with those options rather than issue an
unnecessary duplicate catalogue read; ordinary worlds may continue to use the
existing `getTemplates()` call. Preflight must still run against the live
`POST /api/templates/preflight` and its passing digest must be included in
`PROPOSE_CASE`; bootstrap template data is display input, not authorization or
preflight evidence. A stale digest remains a refusal and requires fresh
preflight.

### Refresh/discovery

While the bootstrap screen is active, use one authenticated refresh request at
a time, with at least 1 second between request starts. The refresh signal is
abortable; abort on screen disposal, session change/logout, or transition to an
ordinary case. Never overlap a request, refresh faster than the interval, or
open an SSE stream while bootstrap is active. Another validated bootstrap
response keeps the screen in the empty state and may replace templates/time;
a validated ordinary response transitions to that real case's history;
malformed JSON or any non-success response moves to unavailable/retry UI, not
to the empty state. Provide a user retry from unavailable. Exact transient
versus terminal error classification remains subject to the existing typed
error contract; do not parse message text or retry authorization/invalid errors
as if they meant “still empty.”

The refresh is a discovery poll, not a ledger frontier, snapshot lock, or
proof that no case can commit immediately after the response. It only accepts
ordinary response data that validates and names a real case/cursor.

### `PROPOSE_CASE` and ordinary readiness

The current template path is preserved: choose template, gather params, run
preflight, submit `PROPOSE_CASE`, require the resulting case ID, then open that
case (`src/app/proposals.ts:47-97,105-122`). After submit succeeds:

1. Keep the returned `resulting_object.id` as the only target case identity.
   If the accepted response has no case ID, show unavailable/invalid and do
   not pick another case or create an ordinary history.
2. Enter an “Opening case” state. Fetch the explicit case route using that
   returned ID; do not use unqualified `GET /api/world`, which could select
   some other case. Poll with the same one-request/at-least-one-second/
   abort rules only for an error explicitly classified as not-ready. The
   current error contract does not distinguish transient cursor-not-yet-seen
   from every projection failure; if no safe existing classification is
   available, stop in unavailable/retry UI instead of blindly polling.
3. Validate an ordinary `snapshot`, and require its `case_id` to equal the
   requested returned ID and its cursor to be a real nonempty cursor accepted
   by the ordinary contract. A bootstrap arm, another case, missing cursor,
   malformed snapshot, or error never advances this transition.
4. Only after that snapshot validates, load/build ordinary history from real
   snapshots, focus the returned case, mount/transition to `App`, then let its
   existing case-following stream attach. No ordinary SSE is opened before
   readiness and history exist. Do not manufacture a resync boundary.

The Go intent handler exempts `PROPOSE_CASE` from existing-case staleness
checks because the case does not exist yet, and validates template ref, params,
and preflight digest (`apps/godspeed-casework-go/internal/intents/intents.go:154-160,180-196`).
It maps to the existing governed `CommitCase`, passes the intent ID as request
ID, and returns the minted case ID in `resulting_object` (`:269-311`). The
gateway waits up to five seconds for the first observed per-case cursor; that
wait can return an empty cursor and the success response can still carry the
case object (`:421-431`). Therefore intent success alone is not proof that the
ordinary snapshot is ready.

The UI currently creates a UUID for each submit and handles network exceptions
as an error (`proposals.ts:68-80,94-96`). Preserve the existing intent ID and
identical request body across an ambiguous transport retry; do not create a
second case by minting a fresh idempotency key when the first commit outcome is
unknown. The Go handler keeps same-ID/body replay and rejects same ID with a
different body (`intents.go:134-136`, `:618-637`); the governed kernel call also
uses that ID as its request ID (`:274-278`). The in-memory gateway outcome
cache is process-scoped, so do not claim it alone guarantees replay across
restart. Changed template/parameters require a fresh preflight and a new
intent only after the prior attempt has a conclusive outcome.

## 5. `InteractionIntent` create-only exception

The current authored `interaction-intents.schema.json` requires `case_id` and
`client_cursor`, restricts them to ordinary nonempty patterns, and omits
`PROPOSE_CASE` from its `action_name` enum (`schemas/interaction-intents.schema.json:8-56`).
The canonical TypeScript action union already includes `PROPOSE_CASE`, and the
UI sends empty case/cursor for it (`typescript/types.ts:113-170`; app proposal
source above). This is an existing schema mismatch that must be reconciled in
the separately approved normative contract update.

Proposed narrow rule: all fields remain required in the envelope. A
`PROPOSE_CASE` branch is valid only with `kind: "CONSEQUENTIAL_CASE"`,
`case_id: ""`, `client_cursor: ""`, a nonempty template target, and its typed
parameters (`template_ref`, `params`, `preflight_digest`). Every non-create
branch retains the existing case ID and cursor constraints. No other action
may use empty identifiers. Add `PROPOSE_CASE` to the schema action enum to
match the existing TS intent union. Enforce this in the authored schema and Go
intent validator; do not weaken the ordinary cursor or case-ID grammar. The
special branch does not bypass authentication, CSRF, rate limiting,
session-verified actor overwrite/delegation, role checks, payload validation,
template preflight digest comparison, kernel governance, or idempotency.

Existing boundary evidence: `/api/intents` is session+CSRF+rate-limited and
the handler overwrites/re-verifies session actor (`server.go:142-146,233-256`);
preflight is session+CSRF (`:146`); intent handler validates consequential
kind/action, actor, projection role, and mandatory typed payload before the
staleness exception (`intents.go:118-160`). `PROPOSE_CASE` still requires a
passing preflight digest and params (`:182-196`), and kernel preflight pins
template bytes for commit (`crates/sea-forge-server/src/sfwp/case.rs:184-256`).
No additional exception to those controls is proposed.

## 6. Compatibility, races, and error cases

The component affects these seams together if later authorized:

- Go `/api/world` successful response union, private DTO constructors, and
  handler paths for selected, unselected, not-ready, and complete-empty cases.
- New authored world-response schema and golden/conformance cases; an
  `InteractionIntent` schema correction for the already-used create exception.
- Go contract DTO, TS canonical types, UI port exports, strict runtime response
  validation, and explicit bootstrap shell before history initialization.
- UI proposal-state/controller reuse and commit-to-ready transition, preserving
  template/preflight/commit behavior and existing streams only after ordinary
  readiness.
- Authored docs/spec and ADR/decision-log synchronization before any public
  contract change, per repository scope rules.

Known races and their required handling:

- A case can be committed just after a complete-empty observation. Bootstrap
  is only a timestamped observation; the authenticated poll discovers a later
  ordinary snapshot. No atomic no-case window is promised.
- A case can exist before its first relay-observed cursor. It is not an empty
  cell. Explicit case selection remains unavailable; successful create stays
  in opening state until a validated real snapshot or becomes unavailable.
- Current `case.list` can omit unreadable/partial entries, so false-empty is
  possible until its independent held correction. Bootstrap success cannot be
  enabled before correction.
- Template catalogue lookup can fail after complete-empty inventory. Return an
  error, not a bootstrap with absent/partial templates. Empty catalog is a
  valid catalog result with a visible no-templates state.
- A successful commit may not yield its first cursor within the gateway's
  current five-second wait. Success with a case ID is not snapshot readiness.
- A commit response can be lost after the kernel accepts it. Preserve the same
  intent id/body for outcome recovery; fresh IDs risk a second commit.
- Poll abort, session expiry, duplicate query parameters, explicit empty case
  query, malformed response, mismatched case ID, missing cursor, invalid
  timestamp, and a non-success response all stay loading/unavailable or typed
  request error, never become true-empty success.

## 7. Test-first acceptance matrix for a future authorized implementation

These are required proposed tests, not runs or evidence.

### Schema, DTO, and Go route

1. Ordinary `{snapshot: ...}` still validates; ordinary snapshot fields and
   ready GET response remain unchanged.
2. A valid bootstrap with exact status, date-time timestamp, perspective, and
   existing template option arrays validates; `templates: []` validates.
3. Neither arm, both arms, unknown wrapper/bootstrap fields, wrong status,
   absent required fields, wrong field types, malformed timestamp, and invalid
   template option shapes fail validation.
4. Go response constructors serialize exactly one arm and preserve existing
   `TemplateEntryOption` wire names and omission behavior.
5. A fully corrected, complete empty inventory plus successful template read
   returns bootstrap with actual capture timestamp and session perspective.
   Nonempty inventory returns ordinary snapshot only. Known case without a
   real cursor returns unavailable, not bootstrap.
6. Inventory errors, unreadable IDs, missing/malformed inventory fields, or
   template source errors never return bootstrap. Tests explicitly prove the
   current false-empty modes cannot be treated as success until the held
   correction is in place.
7. Absent `case_id` permits unqualified selection; one explicit valid case
   returns only that case; explicit unknown/unready case, empty value, or
   duplicate values never redirect or return bootstrap. Cursor-addressed
   ordinary responses remain ordinary.
8. Successful empty observation followed by concurrent case creation may
   return the captured bootstrap, but the next refresh returns only the real
   ordinary case; no test asserts atomicity or a ledger frontier.

### UI response/startup and refresh

9. Startup remains loading until response validation; bootstrap shows no case
   history/timeline/cursor cache and offers an explicit template-entry action.
10. Ordinary startup follows existing history loader only after an ordinary
    snapshot validates. The ordinary case ID/cursor are real; no empty
    `WorldHistory` or fake revision is used.
11. Session expiry, network error, malformed arm, invalid template array, and
    failed ordinary snapshot validation render unavailable/login handling,
    never bootstrap.
12. With bootstrap active, requests are authenticated, serialized one at a
    time, starts are separated by at least one second, and cancellation on
    disposal/session change prevents stale responses from changing state.
    No SSE connection is opened while bootstrap is active.
13. A later bootstrap response keeps bootstrap state; a later ordinary
    response transitions to its real case; an error transitions to retryable
    unavailable, not empty.
14. Explicit `case` deep links never silently switch to the newest case or
    bootstrap. Duplicate/empty selection parameters are rejected.

### Entry, intent, idempotency, and ready transition

15. Bootstrap-provided template options render through the existing design
    panel; parameters, preflight result/digest, and errors remain usable. A
    changed parameter invalidates prior preflight; failed/stale preflight never
    submits.
16. `PROPOSE_CASE` with empty case/cursor is accepted by schema/handler only
    for the create action and required typed payload. Empty ordinary mutation
    IDs, wrong kind, missing params/digest, unauthorized role/session, missing
    CSRF, and rate limiting remain refused before a commit.
17. Existing passing digest is sent unchanged; the kernel's current-template
    digest mismatch remains a no-side-effect refusal. No direct template
    mutation bypasses preflight or governance.
18. Exact same intent id/body replays its outcome; same ID with altered body
    refuses. An ambiguous transport retry does not mint a second intent ID or
    duplicate a case.
19. Accepted response without resulting case ID stays unavailable. Accepted
    response with case ID but no ready snapshot stays opening/unavailable and
    targets exactly that ID. A bootstrap response, other case, malformed
    snapshot, empty cursor, or failed read cannot mount `App` or start SSE.
20. A matching validated ordinary snapshot permits real history/focus and
    then the ordinary case stream. Test that stream attachment happens after
    validation and not on a cursorless/empty bootstrap.

## 8. Remaining HOLDs and limits

This component is one input to full V4, not V4 approval. Separate operator/root
decisions and reviewed changes are still required for:

- the actual Rust case-ID and Go/UI world-ID producers versus frozen schema
  patterns, without silently changing IDs;
- fail-closed `case.list` semantics, accurate top-level error mapping, and
  evidence that a no-case result means complete empty inventory;
- `events.get_range` frontier/memory/work limitations and case creation races;
- per-case Store history, SSE selection/readiness, gap invalidation and
  cancellation ordering;
- any public schema/spec/ADR release and any source implementation.

No cursor is fabricated, no current kernel frontier is claimed, no tests or
runtime experiments were performed, and this document grants no source-change
authorization.
