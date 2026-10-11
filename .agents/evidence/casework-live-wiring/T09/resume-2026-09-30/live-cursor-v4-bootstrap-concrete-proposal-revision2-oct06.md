# Live cursor V4 bootstrap component proposal — revision 2

Date: 2026-10-06  
Status: **DOCONLY component proposal. Public implementation, no-case success, and full V4 remain HOLD.**  
Predecessor: `live-cursor-v4-bootstrap-concrete-proposal-oct06.md`, SHA-256 `e02ab804d95bf9aeb02878c6057716be8c7d24f0863dd5efc100408f4368bcc1`.  
This is a complete replacement proposal for review, not an implementation assignment or an erratum that authorizes source work.

## 1. Scope and preserved decisions

Keep a ready `GET /api/world` response as `{snapshot: CognitiveWorldSnapshot}`. The proposed mutually exclusive bootstrap arm is:

```json
{
  "bootstrap": {
    "status": "no_cases",
    "timestamp": "<actual RFC3339 capture time>",
    "perspective": { "actor_id": "...", "role": "..." },
    "templates": []
  }
}
```

The arm has no snapshot, world/case ID, cursor, revision, history, or synthetic identity. `timestamp` is when the handler captured this response; `perspective` is the existing session-verified `ActorPerspective`; `templates` is the canonical wire `TemplateEntryOption[]` copied from the existing `case.entry_options` source. An empty template list means the catalogue returned no templates; it does not prove the case inventory is empty.

The bootstrap is an as-of observation, not an atomic filesystem snapshot, kernel frontier, or guarantee that a case cannot commit immediately afterward. Bootstrap success remains disabled until the separately held fail-closed inventory correction and complete-empty proof are implemented and accepted. A failed, unreadable, incomplete, or uncertain inventory is an error, never `no_cases`. Full V4 inventory completeness, frontier/range semantics, IDs, Store/SSE readiness, and public schema release remain unresolved and held.

Preserve the original proposal's remaining decisions:

- A strict authored JSON Schema response `oneOf` has exactly a snapshot arm and bootstrap arm. Both wrappers reject unknown properties; each arm excludes the other. The bootstrap requires all four fields, literal status `no_cases`, RFC3339 capture timestamp, existing perspective shape, and canonical templates shape. Do not edit generated files or invent a generator.
- A supplied `case` query is explicit selection and never redirects. Reject duplicate or explicitly empty values. It yields only a ready ordinary snapshot for that exact case; failures remain failures. Only an absent `case` query can take the unqualified branch.
- Bootstrap UI state is outside `WorldHistory`, cursor caches, case trajectory, and SSE. Do not manufacture a world history or mount `App` on a fake revision.
- Preserve template design, preflight, `PROPOSE_CASE`, authentication, CSRF, rate limiting, session actor verification, role checks, template digest, kernel governance, and idempotency. The empty `case_id`/`client_cursor` exception stays create-only and is still a held authored-schema/validator reconciliation.
- A successful create is not ordinary-snapshot readiness. Use the returned case ID only; validate a ready ordinary snapshot for that exact ID before loading history, focusing, mounting/transitioning to `App`, or opening case SSE.
- Bootstrap refresh is authenticated, one outstanding request at a time, no faster than once per second, abortable on disposal/session change/transition, and never opens SSE. A malformed or failed response becomes unavailable/retry UI, not empty.
- No new kernel verb, no fabricated cursor, and no claim of complete/atomic frontier.

## 2. Wire schema, types, and canonical template mapping

### Wire response shape

The proposed authored schema path remains `.agents/reports/interface-contracts/schemas/world-response.schema.json`. It uses `oneOf` with these exact wrapper arms:

1. `{snapshot: CognitiveWorldSnapshot}` with `snapshot` required, `bootstrap` forbidden, and wrapper `additionalProperties: false`.
2. `{bootstrap: WorldBootstrap}` with `bootstrap` required, `snapshot` forbidden, and wrapper `additionalProperties: false`.

`WorldBootstrap` has exactly required `status`, `timestamp`, `perspective`, and `templates`; its object also has `additionalProperties: false`. `status` is constant `no_cases`; timestamp is RFC3339/date-time; perspective reuses canonical `ActorPerspective`; templates reuses canonical wire `TemplateEntryOption`. Canonical template parameter fields are `name`, `type` (`string | number | boolean | enum`), optional `title`, `description`, `required`, `default_value`, and `options`. Keep canonical wire names `type` and `default_value`; the UI's app port uses a different normalized shape and must not be populated by casting this wire DTO.

Concrete later source shapes:

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

The Go response constructor must set exactly one arm. Existing contract types are `CognitiveWorldSnapshot`, `ActorPerspective`, and `TemplateEntryOption` (`apps/godspeed-casework-go/internal/contract/contract.go:50-83,130-147`). The corresponding UI wire types belong in the authored canonical TypeScript source `.agents/reports/interface-contracts/typescript/types.ts`; export or adapt them through the app port without creating a second perspective or wire-template DTO. `WorldResponse` is a mutually exclusive TS union. Do not change generated files in this proposal.

### Exact wire-to-app-port conversion

`WorldBootstrap.templates` is canonical wire data. Before any value enters `ProposalState`, `TemplateDesignPanel`, or preflight UI, validate it as canonical data and run the same normalization as the existing `HttpCaseworkAdapter.getTemplates()` (`apps/godspeed-cognitive-ui/src/adapters/http/httpCaseworkAdapter.ts:451-474`). Define one pure authored mapper, e.g. `toTemplateEntryOptions(wire: readonly CanonicalTemplateEntryOption[]): readonly TemplateEntryOption[]`, adjacent to the authored response parser. It is shared by `getTemplates()` and `parseWorldResponse()`; do not copy slightly different transformations into bootstrap code.

For each template preserve `template_ref`, `title`, optional `description`, and parameter order. For each parameter:

| Canonical wire field | App-port field and rule |
|---|---|
| `name`, optional `title`, `description` | Preserve exact validated values |
| `type` | `param_type`; preserve exactly `string`, `number`, `boolean`, or `enum` |
| `required` | `required`; use `false` when absent, preserve validated boolean when present |
| `default_value` | `default`; when present use the existing adapter's `String(default_value)` conversion, including number/boolean values |
| `options` | Preserve ordered string array for enum values; absence remains absent |

Do not let the old permissive `String(...)`/`Boolean(...)` casts validate hostile input. The runtime parser validates primitive/container types first; the mapper then mirrors the existing conversion behavior. Keep optional property presence consistent with the existing app-port adapter. The same `TemplateDesignPanel` therefore receives `param_type`/`default`, never canonical `type`/`default_value` directly. This resolves the predecessor review's template-port mismatch.

Required focused fixture coverage: all four `type` kinds; required true, false, absent; absent and string/number/boolean `default_value` conversions; enum options and absent options; optional title/description; multiple parameters and templates preserving order; and empty templates. Assert normalized fields actually consumed by the panel/controller, not only schema acceptance.

## 3. Runtime response parser boundary

The existing `getSnapshot()` blindly casts `(res.body as {snapshot: ...}).snapshot` (`httpCaseworkAdapter.ts:233-240`); current contract conformance validates authored fixtures but is not a production runtime validator. Do not claim otherwise, add a dependency, or invent a schema-to-code generator.

For a later separately authorized source task, add one hand-authored runtime parser module (proposed `apps/godspeed-cognitive-ui/src/adapters/http/worldResponse.ts`) with this exact boundary:

```ts
export type ParsedWorldResponse =
  | { readonly kind: 'snapshot'; readonly snapshot: CognitiveWorldSnapshot }
  | { readonly kind: 'bootstrap'; readonly bootstrap: WorldBootstrap;
      readonly templates: readonly TemplateEntryOption[] }

export class WorldResponseValidationError extends Error { /* safe fixed message */ }
export function parseWorldResponse(input: unknown): ParsedWorldResponse
```

`parseWorldResponse` is a strict, hand-authored structural validator corresponding to the new authored schema; it does not load JSON Schema at runtime. It must:

1. Require a non-null plain object wrapper with exactly one own `snapshot` or `bootstrap` property, and reject unknown wrapper keys, both arms, neither arm, and a present arm whose value is `undefined`/null.
2. For bootstrap, require a plain object with exactly `status`, `timestamp`, `perspective`, `templates`; enforce literal status; validate the timestamp as a real RFC3339 value (syntax and calendar parse, not just `Date.parse` truthiness); validate canonical `ActorPerspective`; require an array of canonical templates; reject unknown keys and wrong field/value types throughout. Then invoke the shared canonical-template mapper and return the normalized templates alongside validated wire bootstrap.
3. For ordinary snapshot, validate the *whole* authored `CognitiveWorldSnapshot` structure: exact required keys and allowed optionals according to its authored contract, nested perspective/summary/object/action/focus members, enums, arrays, IDs/cursor/timestamps, and unknown-property policy. It must not validate only that `snapshot` is truthy. Return only a fully validated snapshot.
4. Throw only `WorldResponseValidationError` for malformed payloads, with no raw response body/value in the message. The HTTP adapter catches it and emits the existing typed `HttpRefusalError('UNAVAILABLE', safe fixed message)` (or an explicitly reviewed equivalent); callers receive a typed unavailable result, not empty state.

The route-level schema remains the machine-readable authored contract and the conformance tests exercise its examples. The hand-authored parser is a separate production implementation; test shared positive/negative fixtures against both schema conformance and parser so drift is visible. `getSnapshot(caseId,...)` remains ordinary-only and rejects a bootstrap arm. The new bootstrap-aware unqualified method uses the parser before branching. `getTemplates()` uses the same template parser/mapper for `/api/templates`. No runtime or source approval is given here.

## 4. Server route selection and no-case precondition

Keep the session-protected existing route. For a cursor-addressed request retain its ordinary historical behavior. For a request without a historical cursor:

1. Detect query-key presence and multiplicity; distinguish absent `case_id` from explicit empty or duplicate parameters. Explicit empty/duplicate is invalid, not unqualified.
2. Explicit case ID returns only a ready ordinary response for that exact ID. Unknown, denied, missing cursor, or unready projection returns error. It never returns bootstrap or substitutes another case.
3. With no explicit case, obtain a fail-closed complete inventory. Existing nonempty inventory follows the existing case selection and returns an ordinary snapshot only when its real readiness conditions hold.
4. Only an independently corrected complete-empty inventory permits reading template entry options and constructing bootstrap. Template read error returns typed error; a successful empty option array is valid.
5. Incomplete, unreadable, refused, or malformed inventory is error. This document does not choose a new public error class or claim the held source has been corrected.

Current anchors recorded in the predecessor review: session-protected `/api/world` and current branch at `apps/godspeed-casework-go/internal/server/server.go:135-141,177-230`; session-protected `/api/templates` at `:139,259-266,426-428`; `LiveSource.EntryOptions` at `internal/projection/live.go:207-239,295-302`. The source's current false-empty modes remain independently held: Rust case listing can collapse directory failures, and Go drops unreadable information (`crates/sea-forge-server/src/sfwp/case_views.rs:143-153,310-355`; `apps/godspeed-casework-go/internal/adapters/sfwp/authority.go:69-88`). They must be fixed and independently accepted before any true-empty response.

## 5. Startup, bootstrap UI, and refresh lifecycle

Live startup currently loads history before rendering `App` (`apps/godspeed-cognitive-ui/src/main.tsx:65-123`); `initialState` requires a real current revision (`src/model/store.ts:12-18,44-49`); `App` attaches `connectLive` using `history.caseId` (`src/app/App.tsx:89-104`). Preserve that invariant:

- Resolve session first.
- With no explicit `case` query, request the bootstrap-aware unqualified response. With one explicit nonempty `case`, request that case only; duplicate/empty values reject.
- A validated snapshot arm only then enters ordinary history loading for that exact case and mounts `App`.
- A validated bootstrap arm renders a standalone state outside `UiState.history`, `WorldHistory`, `rawByCase`, snapshots, cursor dedupe, trajectory, and SSE. Show session perspective, actual capture timestamp, template options or an honest no-templates state, and an explicit “Design case” action. Do not mount `App` with fake history.
- Loading, unavailable, authentication failure, and validated no-cases are distinct states. Malformed JSON and non-success responses cannot become bootstrap. Session expiry follows the login/session-ended route.
- Use the existing `TemplateDesignPanel` and proposal/preflight behavior, but feed it normalized app-port template options from the shared mapper above. Bootstrap catalogue values are display/input only, not preflight or authority.
- While bootstrap is active, issue one authenticated refresh at a time with at least one second between request starts. Abort refresh on view/session disposal or successful transition; ignore stale completions. Another valid bootstrap can refresh timestamp/templates; a valid ordinary response moves to that real case; error goes to unavailable/retry. Do not poll arbitrary errors as “not ready”; no safe generic retry class is established. Do not open SSE while no ordinary ready history exists.

Refresh is discovery, not proof of an atomic no-case interval or a range frontier. If a case appears immediately after a complete-empty observation, next refresh can find it; no test may claim otherwise.

## 6. Proposal controller and immutable pending intent ownership

### Why a separate owner is required

Today `createProposalFlow` is coupled to `Store`, selects templates from `TemplateSourcePort`, and creates a new `uuid()` inside every `submit()` invocation (`apps/godspeed-cognitive-ui/src/app/proposals.ts:12-103`, especially 60-97). `ProposalState` is nested in `UiState` and stores current form/preflight/submitting/result data but no pending intent (`apps/godspeed-cognitive-ui/src/model/types.ts:409-424`). A bootstrap page has no truthful `Store`/`WorldHistory` to supply. The predecessor correctly required extracting a shared proposal-specific owner; this revision defines that seam.

Use one session-shell-owned `ProposalController` and one immutable proposal state model, independent of `WorldHistory` and the ordinary case `Store`:

```ts
interface PendingCreate {
  readonly intentId: string
  readonly body: Readonly<InteractionIntent> // deep-cloned/frozen once; exact same object on retry
  readonly outcome: 'unknown'
}

interface ProposalControllerState {
  readonly templatesStatus: 'idle' | 'loading' | 'ready' | 'unavailable'
  readonly templates: readonly TemplateEntryOption[] // app-port normalized shape
  readonly selected: string | null
  readonly params: Readonly<Record<string, string>>
  readonly preflighting: boolean
  readonly preflight: { readonly passed: boolean; readonly digest?: string;
    readonly reasons: readonly string[] } | null
  readonly submitState: 'idle' | 'submitting' | 'outcome_unknown' | 'accepted' | 'refused'
  readonly pendingCreate: PendingCreate | null
  readonly resultCaseId?: string
  readonly error?: { readonly code?: string; readonly message: string }
}

interface ProposalController {
  getSnapshot(): ProposalControllerState
  subscribe(listener: () => void): () => void
  open(options?: { readonly templates?: readonly TemplateEntryOption[] }): Promise<void>
  select(templateRef: string | null): void
  setParam(name: string, value: string): void
  preflight(signal?: AbortSignal): Promise<void>
  submit(): Promise<ProposalOutcome>
  retryUnknown(): Promise<ProposalOutcome>
  reset(): void
  disposeView(): void
}
```

The concrete state may be stored in a private controller store rather than this literal class shape, but it must have these exact ownership semantics. The session shell creates the controller once after session resolution and passes it to either ordinary `App` or standalone bootstrap UI. View unmount, refresh completion, or bootstrap-to-case transition does not construct a replacement controller or discard an unknown create. Ordinary App continues to use its real history/store for case display; the shared controller owns only template form/preflight/intent state and invokes a supplied post-acceptance transition callback.

### Immutable attempt rules

1. Before an intent exists, select/parameter changes invalidate any old preflight. A successful preflight is bound to exact selected template and parameter snapshot; `submit` reads a snapshot and refuses unless the matching digest is present.
2. First submit creates one UUID exactly once, one deep-cloned immutable `InteractionIntent` body, and stores `{intentId, body, outcome:'unknown'}` before dispatch. This body retains the create-only empty `case_id` and `client_cursor`, the preflight digest, template ref, params, and actor fields as built at first submission. Retry calls `port.dispatchIntent(pending.body)` with that same ID and unchanged body; it does not invoke UUID generation or rebuild from current form state.
3. A conclusive matching accepted or refused intent outcome clears `pendingCreate`. Accepted response requires the actual returned case ID; absence of that ID is a conclusive protocol error and does not retry/create another case. A conclusive refusal is shown and permits a later changed form only after fresh preflight and a newly created intent.
4. A transport exception, lost response, or response that does not establish whether this intent ran sets/keeps `submitState='outcome_unknown'` and retains exact pending identity/body. It exposes only `retryUnknown` with the same immutable attempt. While outcome is unknown, disable changing selected template/params, preflight replacement, reset, close-and-forget, and any new submit. Do not permit a changed intent until the old attempt has a conclusive outcome.
5. An explicitly classified pre-dispatch rejection is conclusive non-submission only where the existing adapter contract proves it: current `dispatchIntent` treats a non-success HTTP response **without** an `intent_id` as a pre-dispatch refusal (`httpCaseworkAdapter.ts:252-261`). Preserve and test that typed distinction. Do not infer non-submission from message text or a generic `UNAVAILABLE` label.
6. `disposeView()` aborts view-owned template/preflight/refresh reads and prevents stale UI callbacks; it never erases an ambiguous pending attempt. The controller and pending immutable value live at the session shell across bootstrap/ordinary view transitions. Logout/session replacement cancels view I/O and prohibits further dispatch under the old session; it cannot falsely mark an in-flight commit refused. Retain the unknown state until this controller is conclusively resolved or the page/process itself is lost. The in-memory controller does not promise recovery across reload, and the gateway's process-scoped outcome cache does not prove replay across gateway restart. Do not claim stronger idempotency.
7. `reset()` is a no-op/refusal while submit is active or outcome unknown. It may clear form after a conclusive refusal or before an attempt exists. On accepted response it transitions to opening the returned case, not to a clean form.

These rules repair the predecessor review's missing pending-attempt owner. They do not claim the current code already supports same-ID retry: its current `submit()` generates `uuid()` on each invocation (`proposals.ts:68-78`). The later implementation must change that flow through this shared owner, not merely add an “unknown” label.

### Template and preflight behavior

`open()` accepts validated normalized bootstrap templates as seed data and skips a duplicate `/api/templates` read. In ordinary App, it may call the existing template source. Both modes use identical `select`, parameter, preflight and submit rules. The live `POST /api/templates/preflight` remains required; passing digest accompanies `PROPOSE_CASE`; changed parameters invalidate preflight; failed or stale preflight never submits. Empty template options render no fake entry. Template-source and preflight failures remain unavailable/refused states.

## 7. Create-only intent schema and accepted-case transition

The current authored interaction-intent schema requires ordinary `case_id` and `client_cursor` patterns and omits `PROPOSE_CASE` from its action enum, while the canonical TS action union includes it and UI already submits empty values (`schemas/interaction-intents.schema.json:8-56`; `.agents/reports/interface-contracts/typescript/types.ts:113-170`; `src/app/proposals.ts:68-78`). The separate reviewed contract/schema update must permit exactly this narrow branch: every envelope field remains present; `PROPOSE_CASE` only with `kind: CONSEQUENTIAL_CASE`, `case_id: ""`, `client_cursor: ""`, nonempty template target and required typed `template_ref`, `params`, and `preflight_digest`. Every non-create action retains existing nonempty ID/cursor checks; no other action may use empty IDs. Do not weaken authentication, CSRF, rate limiting, actor overwrite/delegation checks, role validation, preflight, governance, or idempotency.

Current Go `PROPOSE_CASE` behavior validates action, actor, role, typed payload and preflight digest before its narrow missing-case staleness exception (`apps/godspeed-casework-go/internal/intents/intents.go:118-196`); it uses intent ID as kernel request ID and returns the minted case ID (`:269-311`). Same-ID/body replay is handled by existing in-memory gateway outcome behavior; same ID/different body refuses (`:134-136,618-637`). Do not claim that process-local cache alone survives gateway restart. The gateway's bounded first-cursor wait can return an accepted case object even with no observed cursor (`intents.go:421-431`), so accepted response is not snapshot-ready.

On conclusive acceptance:

1. Require a nonempty returned `resulting_object.id`. If absent, show typed invalid/unavailable and do not choose another case or create history.
2. Keep that returned ID as the sole target. Enter `openingCase`; request the explicit case route. Do not use unqualified `/api/world` (could select another case).
3. Poll only on an existing explicitly typed not-ready response, with one outstanding authenticated request, at least one second between starts, and abort/dispose rules. The existing error classes do not reliably distinguish not-yet-ready from all projection failures; absent a safe class, stop in unavailable/retry UI instead of blind polling.
4. Strictly parse ordinary response. Require matching `snapshot.case_id` equal returned ID and a valid nonempty real cursor. A bootstrap arm, different case, malformed snapshot, missing cursor, auth error, or unclassified error does not advance.
5. Only after a validated matching ordinary snapshot exists, load/build real history, focus the returned case, transition/mount `App`, then attach its ordinary stream. The current `App` stream follows `history.caseId` (`src/app/App.tsx:89-104`); no SSE before history/readiness.

## 8. Bounded UI lifecycle and error cases

The bootstrap state machine is `loading -> bootstrap | ordinary-ready | unavailable`; refresh from bootstrap remains bootstrap, moves to ordinary-ready only on a validated ordinary response, or moves to unavailable on error. User retry can start a new serialized refresh. Every async completion is scoped to the active session/controller generation; disposed view completions do not mutate rendered state. Refresh interval is measured between request starts and never below one second. Abort pending refresh on unmount/session switch/transition. No polling multiple requests concurrently; no SSE in loading/bootstrap/unavailable.

Proposal attempt is orthogonal: `idle -> submitting -> accepted | refused | outcome_unknown`; only `outcome_unknown -> submitting` via retry of same stored intent; conclusive outcome ends the old attempt. View disposal does not reset it. Prevent proposal state from being coupled to fake world/history state. No guarantee across browser reload/process loss is claimed.

Route and UI errors: invalid/duplicate/empty explicit query; selected case absent/unready; malformed response or both/neither arm; invalid template/perspective/time; inventory/list failure; template source failure; session expiry; preflight failure/stale digest; pre-dispatch refusal; ambiguous dispatch; accepted result missing ID; accepted ID without ready cursor; mismatched case response; bootstrap refresh abort; and out-of-order disposed completion. Only fully validated complete-empty inventory plus successful templates yields bootstrap. No error class is broadened or inferred from text.

## 9. Future implementation test-first matrix (proposed only)

These are acceptance fixtures, not tests run here.

### Schema/parser/adapter

1. Ordinary snapshot arm remains byte/field compatible. Bootstrap valid with exact status, real RFC3339 timestamp, perspective, and empty/populated canonical templates validates.
2. Neither/both wrapper arms, unknown wrapper/bootstrap/template/parameter keys, present undefined/null arm, wrong status, wrong timestamp syntax/calendar, malformed perspective, invalid parameter type/required/default/options, and malformed template list fail closed with typed validation error and safe message.
3. Full ordinary nested snapshot malformed at each required nested member fails; valid snapshot passes. Case-specific `getSnapshot` rejects bootstrap.
4. Schema conformance and hand-authored parser share fixtures for exact response arms and reject equivalent invalid cases. No generated-source or runtime-schema-loader assumption.
5. Canonical-to-port mapping verifies `string`, `number`, `boolean`, `enum`; required true/false/absent; default string/number/boolean/absent; options; optional title/description; multiple values/order; empty arrays. Assert panel receives `param_type`, `default`, required and options, not wire field names.
6. `/api/templates` and bootstrap use the same canonical validator/mapper. A bad canonical response does not get string/boolean-coerced into a plausible option.

### Route and startup

7. Exactly one response arm is constructed. Session perspective and actual capture time are used. Valid empty inventory plus successful template result returns bootstrap; empty templates valid. Nonempty inventory selects only ordinary ready snapshot.
8. Inventory refusal/unreadable/partial/malformed and template read failure cannot return bootstrap. Tests must not pass against known false-empty behavior; this route remains disabled until separate correction is accepted.
9. Absent query permits unqualified path; explicit valid ID returns only that case; unknown/unready, empty or duplicate explicit query never redirects or bootstraps. Historical cursor stays ordinary.
10. Startup remains loading while validating; bootstrap has no `WorldHistory`, fake cursor/revision, `App`, or SSE; ordinary arm loads history only after validation. Auth/session failures do not look empty.
11. Refreshes are authenticated, serialized, starts at least 1 sec apart, abort on disposal/session change/transition, and ignore disposed stale completion. No SSE while bootstrap. Second empty stays bootstrap; ordinary moves to exact case; errors show retryable unavailable.
12. Concurrent case creation after a complete-empty result may be found on a later refresh; do not claim an atomic no-case snapshot or frontier.

### Proposal state/controller and create transition

13. Bootstrap's normalized template choices render the existing design panel; empty options produce no fake entry. Preflight receives exact parameter snapshot; edits invalidate prior pass/digest; failed/stale preflight sends no intent.
14. Create-only empty case/cursor passes only for valid `PROPOSE_CASE` plus consequential kind/typed payload. Every ordinary mutation still rejects empty identifiers. Session/actor/role, CSRF, rate limiting, template digest, and governance remain enforced.
15. Initial submit stores one intent ID and immutable full body before dispatch. Simulate lost/ambiguous transport response, view disposal/remount, and retry: same ID and byte-equivalent body reused, no new UUID; no param/template/preflight/reset/new submit while unknown. Definitive refusal permits a fresh attempt only after preflight; accepted outcome never submits again.
16. Explicit pre-dispatch refusal without intent outcome is classified from adapter's typed response, not message text. Ambiguous server response does not get treated as refusal. Same ID with changed body never dispatched. Test only in-process known server behavior; no cross-restart exactly-once claim.
17. Controller is owned above bootstrap/ordinary view transition, so its unknown pending attempt survives UI unmount. Session change/logout cancels UI I/O but does not mutate uncertainty into refusal. Document page/process loss limitation; no persistence store added without separate approval.
18. Accepted response without case ID remains unavailable and does not fabricate history. Accepted with ID but no ready ordinary snapshot remains opening/retry; requests only that ID. Bootstrap arm, mismatched case, malformed response, missing cursor, or failure cannot mount `App` or attach SSE.
19. Matching validated ordinary snapshot permits history load/focus and then stream attach; assert strict order and returned-ID equality.

## 10. Exact predecessor repairs, remaining deviations, and HOLDs

Revision 2 repairs all three findings in the independent review of predecessor SHA `e02ab804d95bf9aeb02878c6057716be8c7d24f0863dd5efc100408f4368bcc1`:

1. **Template mapping:** canonical bootstrap `TemplateEntryOption` is explicitly validated and normalized through one mapper shared with HTTP `getTemplates`; wire `type/default_value` become app `param_type/default`, preserving required, all four types, defaults, options, order, and empty arrays.
2. **Runtime validation:** defines a hand-authored parser module, exact `unknown -> discriminated parsed response` signature, strict one-of/unknown-key behavior, full nested ordinary snapshot checks, canonical template validation, typed safe error, adapter catch behavior, conformance relationship, and no dependency/schema-generator fiction.
3. **Pending create owner:** defines a session-shell-owned `ProposalController` and state including immutable intent ID/body/outcome; same-body retry only, no edited intent/reset/new submit while uncertain, clear only on conclusive accepted/refused result, view disposal does not erase uncertainty, and explicitly no reload/process-restart recovery guarantee.

No approval is inferred for implementation, contract release, public error mapping, or full V4. The original narrow no-world/no-case/cursor/history bootstrap, fail-closed inventory prerequisite, real RFC3339 capture/perspective/templates, exact explicit-case behavior, one-second serialized refresh, create-only exception and governance, returned-case readiness ordering, unresolved inventory/frontier, and no synthetic cursor are preserved. Component approval, if later given, cannot authorize full V4 or eliminate the held inventory/frontier and source prerequisites.

Material limits/uncertainties still requiring separate review: actual corrected complete-empty inventory proof and exact error mapping; authored schema/TS/Go compatibility and all schema field strictness; runtime parser correctness; ability of adapter/transport to distinguish every conclusive intent refusal from ambiguous outcome; no stable outcome recovery across page reload or gateway process restart; no automatic case readiness when no established not-ready error exists; exact session-shell wiring; any durable pending-intent storage would need new privacy/storage review and is not proposed. The proposal does not assert these capabilities exist today.

## Source anchors checked

- Existing HTTP ordinary snapshot blind cast and template mapping: `apps/godspeed-cognitive-ui/src/adapters/http/httpCaseworkAdapter.ts:233-248,451-474`; intent dispatch pre-dispatch typed error path `:252-261`.
- Actual app-port template names: `apps/godspeed-cognitive-ui/src/ports/contract.ts:223-247`; canonical wire template names: `.agents/reports/interface-contracts/typescript/types.ts:549-567`.
- Current proposal controller UUID-per-submit and outcome handling: `apps/godspeed-cognitive-ui/src/app/proposals.ts:12-103`; current `ProposalState` without pending intent: `src/model/types.ts:409-424`.
- Live startup/history and App stream boundary: `apps/godspeed-cognitive-ui/src/main.tsx:66-123`; `src/model/store.ts:12-18,44-49`; `src/app/App.tsx:89-104`; existing panel `src/ui/TemplateDesignPanel.tsx:20-149`.
- Existing server route/template source, Go contract, create intent, and inventory caveats are detailed in predecessor proposal/review; these remain subject to current source review before implementation. The prior independent review also verified session/CSRF/role/preflight/governance and same-ID/body behavior; this revision preserves those anchors and does not elevate them into a duplicate-runtime guarantee.

No source, test, status, debt, schema, generated file, compiler, scanner, gate, Graft build, Git, or network operation is authorized or claimed by this document.
