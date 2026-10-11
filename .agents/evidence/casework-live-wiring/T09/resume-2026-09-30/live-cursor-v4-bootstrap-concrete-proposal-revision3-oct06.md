# Live cursor V4 bootstrap component proposal — revision 3

Date: 2026-10-06  
Status: **DOCONLY proposal. Public implementation, no-case success, and full V4 remain HOLD.**  
Predecessor: revision 2, SHA-256 `532e3d03acf1dd69d0fe4f920ffbfe4a48b0ca7a5f8c2b23e635044e2d94e037`.  
This is a full replacement proposal for independent review. It does not authorize source, schema, UI, or runtime changes.

## 1. Preserved bootstrap boundary

Keep the ready response `{snapshot: CognitiveWorldSnapshot}`. The candidate second arm is mutually exclusive and has exactly these bootstrap fields:

```json
{"bootstrap":{"status":"no_cases","timestamp":"<RFC3339 capture time>","perspective":{"actor_id":"...","role":"..."},"templates":[]}}
```

It has no snapshot, world/case identity, cursor, revision, history, or synthetic identity. The timestamp is the actual response capture time, perspective comes from the verified session, and templates are canonical wire `TemplateEntryOption[]` from the existing entry-options source. Empty templates mean only that no template options were returned.

Bootstrap is an as-of observation. It is not an atomic filesystem snapshot, a kernel frontier, or assurance that a case cannot commit immediately afterward. A successful `no_cases` response remains disabled until the separately held fail-closed inventory correction and complete-empty proof are implemented and accepted. Any incomplete, unreadable, malformed, refused, or uncertain inventory is an error. Full V4 inventory/frontier, IDs/cursors, Store/SSE readiness, public schema and source changes remain HOLD.

Preserve these route and lifecycle rules:

- The authored response schema has `oneOf` snapshot/bootstrap wrappers. The wrappers reject unknown wrapper fields and require exactly one arm. Bootstrap's own object rejects unknown fields and requires all four fields. It has literal `no_cases`, RFC3339 time, existing `ActorPerspective`, and canonical template shape.
- A supplied `case` query is explicit. It never redirects or falls through to bootstrap. Duplicate/empty explicit case values reject. Only an absent case parameter enters the unqualified path. A nonempty historical cursor retains its current historical path; empty cursor query semantics are separately clarified in the erratum below.
- Bootstrap UI state stays outside `WorldHistory`, history/cursor caches, trajectory, and SSE. No `App` mounted over a fake revision or cursor.
- Preserve session auth, CSRF, rate limits, verified actor/role, preflight digest, idempotency, and kernel governance. Empty `case_id`/`client_cursor` remains a narrow create-only contract change under separate review.
- Accepted create uses only the returned actual case ID. It is not readiness: validate a ready ordinary snapshot for exactly that ID, then load history/focus/mount `App`, then attach case-scoped streaming. Never fabricate history or cursor.
- Bootstrap refresh is authenticated, serialized, starts at least one second apart, aborts on disposal/session change/transition, ignores stale completion, and never opens SSE. Errors are unavailable/retry, never empty. Refresh observes a later state; it does not create an atomic no-case interval.

## 2. Canonical template mapping and response validation

Authored response schema location remains `.agents/reports/interface-contracts/schemas/world-response.schema.json`. The ordinary wrapper references the existing snapshot schema; the bootstrap wrapper references a new `WorldBootstrap`. Go/TypeScript shapes reuse existing canonical `CognitiveWorldSnapshot`, `ActorPerspective`, and `TemplateEntryOption` (`apps/godspeed-casework-go/internal/contract/contract.go:50-83,130-147`; canonical TS template definitions at `.agents/reports/interface-contracts/typescript/types.ts:549-567`). No generated files or generators are edited by this proposal.

Concrete later type shapes:

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

The response constructor must set exactly one pointer. The authored JSON Schema uses `oneOf`: snapshot arm requires `snapshot`, forbids `bootstrap`, and closes the wrapper; bootstrap arm requires `bootstrap`, forbids `snapshot`, and closes the wrapper. `WorldBootstrap` requires exactly `status`, `timestamp`, `perspective`, `templates` and closes its object. Canonical TS source should define `WorldBootstrap` and `WorldResponse = { snapshot: CognitiveWorldSnapshot } | { bootstrap: WorldBootstrap }`, reusing existing canonical types rather than creating duplicate wire types. Add the new schema to authored conformance inventory/golden. Current conformance enumerates schemas; no runtime schema-to-code generator or runtime validator is established (`.agents/reports/interface-contracts/tests/contract-conformance.test.ts:130-138`).

### Unknown-property policy

The new response wrappers and `WorldBootstrap` object are closed (`additionalProperties: false`). Within the referenced ordinary `CognitiveWorldSnapshot`, preserve the exact unknown-property behavior of the current authored `cognitive-world.schema.json`; its nested objects do not currently all set `additionalProperties: false` (revision-2 independent review, lines 45-53). A future hand-written parser must match that existing nested policy. Do not silently tighten nested ordinary snapshots as part of bootstrap. Any broader strictness change needs its own authored schema decision and review.

### Wire-to-app mapping

Wire templates use canonical `type` and optional `default_value`; the proposal panel consumes app-port `param_type`, boolean `required`, and string `default` (`apps/godspeed-cognitive-ui/src/ports/contract.ts:223-247`; adapter mapping at `src/adapters/http/httpCaseworkAdapter.ts:451-474`). Add one pure authored normalization boundary shared by `/api/templates` and bootstrap. Validate wire primitives before conversion; preserve template/parameter order and optional title/description/options. Map `type` to `param_type`; absent `required` to false; present default values using the existing `String(default_value)` behavior; absent defaults/options remain absent. Cover all four types, required true/false/absent, string/number/boolean/absent defaults, options, optional labels, ordering, and empty arrays.

### Hand-authored parser boundary

For a later separately authorized source task, use one parser such as `apps/godspeed-cognitive-ui/src/adapters/http/worldResponse.ts`:

```ts
type ParsedWorldResponse =
  | { readonly kind: 'snapshot'; readonly snapshot: CognitiveWorldSnapshot }
  | { readonly kind: 'bootstrap'; readonly bootstrap: WorldBootstrap;
      readonly templates: readonly TemplateEntryOption[] }

function parseWorldResponse(input: unknown): ParsedWorldResponse
```

It is hand-authored; no runtime JSON Schema loader, invented generator, or new dependency. The parser rejects non-plain/null wrapper, neither/both arms, unknown wrapper keys, null/undefined arms, malformed bootstrap fields, invalid real RFC3339 date/time, malformed canonical templates, and unknown nested fields wherever the currently authored schema forbids them. It validates the full ordinary snapshot against current authored required/optional fields and **preserves permissive nested unknown-property behavior where that schema is permissive**. It returns a safe typed validation error without echoing payloads; HTTP adapter maps malformed successful responses to typed unavailable. Case-specific `getSnapshot` accepts only validated ordinary snapshots. Conformance and parser share positive/negative fixtures. Existing `getSnapshot` currently casts the body (`httpCaseworkAdapter.ts:233-248`), so no current runtime validator is claimed.

## 3. Route and UI integration constraints

Current protected `/api/world` and `/api/templates` registrations are at `apps/godspeed-casework-go/internal/server/server.go:137-139`; current handler is `:177-231`; `LiveSource.EntryOptions` is `apps/godspeed-casework-go/internal/projection/live.go:207-239,295-302`. For a later route implementation:

1. Resolve and verify session perspective first.
2. Preserve a nonempty historical cursor request as historical ordinary response. Explicit case requests return only that case's ordinary snapshot or an error. They never select another case or bootstrap.
3. Only an absent case query may use a fail-closed complete inventory. Nonempty valid inventory follows the ordinary ready-case path. Only a separately proven complete-empty inventory permits canonical template read and bootstrap response.
4. Template read failures and any inventory uncertainty are errors; empty template list itself is valid. Do not claim present Rust/Go listing is fail-closed: `case_views::list` is infallible and collapses/skips some filesystem failures (`crates/sea-forge-server/src/sfwp/case_views.rs:249-270,309-355`), and Go `ListCases` ignores `Unreadable` (`apps/godspeed-casework-go/internal/adapters/sfwp/authority.go:69-88`).

At startup, resolve session before selecting either branch. An explicit case selects only that case; no explicit case can receive parsed bootstrap or validated ordinary response. Bootstrap rendering has no `WorldHistory`, cursor cache, fake `App`, or SSE. It offers existing template design and preflight flow and refreshes only under the serialization/rate/abort rules above. Session failure, loading, unavailable, and validated no-cases are distinct states.

Use the existing design panel with normalized app-port template options. Bootstrap catalogue values are display/input, not authorization or preflight. A passing live preflight is required before `PROPOSE_CASE`. After accepted intent, require the response's returned ID, query that exact case, validate matching `snapshot.case_id` and a real nonempty cursor, and only then load history and open the ordinary app/stream. Current `postMutationCursor` wait is not a readiness proof (`intents.go:294-301,418-431`; `server/relay.go:178-217`). If no typed not-ready result exists, stop at unavailable/retry rather than polling generic errors.

## 4. Proposal controller and immutable attempt state

Current `createProposalFlow` is Store-coupled and makes a fresh UUID inside each `submit()` (`apps/godspeed-cognitive-ui/src/app/proposals.ts:12-103`); `ProposalState` has no pending immutable attempt (`src/model/types.ts:409-424`). A later component needs a session-shell-owned proposal controller independent of `WorldHistory`, shared across ordinary/bootstrap view transitions. It owns form, normalized templates, preflight binding, and one attempt; it does not own ordinary world/history state.

Proposed private state shape:

```ts
type AttemptProof =
  | { readonly kind: 'none' }
  | { readonly kind: 'first_attempt_pre_dispatch'; readonly source: 'session' | 'csrf' | 'rate_limit' }
  | { readonly kind: 'matching_accepted'; readonly caseId?: string }
  | { readonly kind: 'authoritative_original_terminal' }

interface PendingCreate {
  readonly intentId: string
  readonly body: Readonly<InteractionIntent> // deep cloned/frozen once
  readonly attemptOrdinal: number
  readonly everAmbiguous: boolean
  readonly status: 'in_flight' | 'outcome_unknown' | 'accepted_unresolved'
  readonly proof: AttemptProof
}

type CreateState = 'idle' | 'submitting' | 'outcome_unknown' |
  'accepted' | 'accepted_unresolved' | 'refused'

interface ProposalController {
  getSnapshot(): ProposalControllerState
  subscribe(listener: () => void): () => void
  open(options?: { readonly templates?: readonly TemplateEntryOption[] }): Promise<void>
  select(templateRef: string | null): void
  setParam(name: string, value: string): void
  preflight(signal?: AbortSignal): Promise<void>
  submit(): Promise<void>
  retryUnknown(): Promise<void>
  reset(): void
  disposeView(): void
}
```

The controller state also contains template loading/readiness, normalized templates, selected template/params, preflight state/digest/reasons, active create state, and display-safe error/result fields. This is a proposed private controller model, not a public wire type. Session shell creates it after session resolution and passes it to bootstrap or ordinary view; transition and view disposal do not recreate it or discard pending work. `attemptOrdinal` increases on each send of the same attempt. `everAmbiguous` is monotonic: once true, later negative responses cannot clear it unless an existing, validated authoritative result tied to the **original** request proves its terminal outcome. Keep the original exact ID and immutable full body with the attempt while unresolved. Do not hash/rebuild the body from current form on retry.

### Attempt transitions and proof rules

1. Before first dispatch, form changes invalidate preflight. First submit requires passing digest bound to exact template/parameters, creates exactly one ID and deep-cloned/frozen body, records attempt 1 before send, and preserves the create-only empty IDs. No UUID regeneration on retry.
2. A transport error, timeout, lost connection after send, cancellation/interruption, malformed response, missing/wrong intent correlation, success/refusal shape mismatch, or generic unavailable is ambiguous unless direct existing source evidence proves that **this first attempt** never reached intent dispatch. Mark `everAmbiguous=true`; retain exact ID/body; disable form edits, new preflight replacement, reset/forget, new submit, and any changed-ID create.
3. A first-attempt session 401, CSRF 403, or rate-limit 429 can prove pre-dispatch only when the adapter returns a specific typed phase/evidence that is bound to the actual response and known route stage. The route order proves `requireSession`, `requireCSRF`, and `rateLimitIntent` run before `handleIntent` (`server.go:142-146`; `session.go:85-95,144-165`; `ratelimit.go:119-142,168-170`). Message text or generic error class is not proof. Current `dispatchIntent` throws `HttpRefusalError` for any non-2xx body without `intent_id` and fills the original intent ID into that error (`httpCaseworkAdapter.ts:252-262`); that does **not** preserve explicit dispatch-stage evidence. A later source implementation must establish a typed proof boundary from status/route provenance or leave the result unknown. No status endpoint is proposed.
4. A proven pre-dispatch rejection can clear the attempt only if `attemptOrdinal == 1` and `everAmbiguous == false`. Once any previous send became ambiguous, subsequent 401/403/429, no-intent-id, `UNAVAILABLE`, cancellation, interruption, network failure, or any other negative that does not prove the original request's outcome leaves the exact original attempt unknown. It cannot enable a different ID/body.
5. A matching positive accepted response with `intent_id` equal to the stored ID and valid success shape resolves acceptance. A mismatched/missing ID or malformed body remains unknown. A negative response after ambiguity is not proof merely because it matches the ID: the current gateway cache may replay a refusal produced after a possibly committed kernel call.
6. An existing validated authoritative terminal outcome could resolve uncertainty only when it is demonstrably tied to the original request ID and exact payload. The Go SFWP client has internal request-correlation recovery (`client.go:453-611`) but no UI-visible original-intent recovery surface is established here. Do not expose or invent a status route, claim durable UI proof, or infer that current error strings establish terminality. Until an existing source/interface supplies verifiable proof, retain the attempt unknown.
7. Same-ID/same-body retry is the only permitted retry action while unknown. Each retry increments ordinal and preserves ID/body. Matching positive acceptance resolves it; any retry middleware rejection or cached/typed unavailable leaves it locked. There is no automatic switch to a fresh intent.
8. A matching accepted response without a nonempty `resulting_object.id` enters `accepted_unresolved`, retaining original ID/body and proof of accepted response. Disable retry, reset, editing, and all new create actions. Show a non-creating unresolved state. Do not invent a case ID, blindly re-dispatch an already accepted attempt, or erase the only attempt record. Resolution requires a separately available validated authoritative result or operator-approved recovery path; none is claimed here.
9. A matching refusal may clear only if its source-backed outcome proves non-submission for the original attempt. Generic `success:false`, `UNAVAILABLE`, `request_cancelled`, or `request_interrupted` is insufficient. Error text heuristics are forbidden.

The gateway handler checks its process-local cache before role/payload/kernel work, and records returned handler outcomes, including mapped errors (`apps/godspeed-casework-go/internal/intents/intents.go:134-175,486-509,626-645`). Thus a same-ID retry can replay a cached `UNAVAILABLE` forever even if its underlying cause was an in-flight/possibly committed mutation; that replay does not prove non-commit. The SFWP client's correlation recovery is internal to kernel round trips, not automatically available to the UI. No exactly-once, gateway-restart, or browser-reload guarantee is asserted.

### Session ownership and principal changes

The controller is bound to the authenticated session generation that created the attempt. View disposal, refresh completion, and bootstrap-to-case transition do not discard it. Logout/session replacement stops old-session dispatch and must not expose an old principal's stored intent ID, body, template parameters, or response to a newly authenticated different principal.

The current component has no durable attempt store or secure cross-session recovery interface. It cannot honestly promise that an unresolved attempt survives page/process loss, and it cannot transfer the body to a different principal. Therefore a principal change while an attempt is pending is a **held policy boundary**: do not implement automatic transfer, reveal, retry, clear-to-refused, or fresh create based on the old attempt. Root/operator must decide whether the old controller is quarantined and the new session is blocked, or another privacy-safe recovery boundary is authorized. This proposal adds no persistence dependency, session archive, status endpoint, or cross-principal mechanism.

## 5. Governance, schema, and readiness preservation

`POST /api/intents` remains session + CSRF + rate limited; the server replaces client actor data with verified session actor and verifies session perspective before handler dispatch (`server.go:142-146,233-256`). Handler checks envelope, role offer, typed payload/digest, then maps `PROPOSE_CASE` to governed `CommitCase` (`intents.go:118-196,282-311`). Current canonical interaction-intent schema still requires ordinary nonempty case/cursor patterns and omits `PROPOSE_CASE`; the create-only empty-ID/cursor exception needs its separate authored schema/contract review (`interaction-intents.schema.json:7-56`, canonical TS action union `typescript/types.ts:113-170`). No identity, CSRF, role, preflight, governance, or idempotency weakening is allowed.

The controller cannot treat accepted response as ready snapshot: dispatcher performs a bounded first-cursor wait and can return the accepted object without a reliable ready cursor (`intents.go:294-311,418-431`; `server/relay.go:178-217`). After positive acceptance with ID, use only that ID; require a parsed ordinary snapshot matching it and a valid real cursor before history, focus, `App`, or SSE. Bootstrap stays outside ordinary history and no empty SSE is opened.

## 6. Proposed test-first matrix

These are future acceptance cases, not tests run here.

### Bootstrap, parser, and route

1. Ordinary response remains compatible; valid bootstrap uses exact status, actual RFC3339 capture time, verified perspective, and canonical templates. Empty template list is accepted without implying empty inventory.
2. Wrapper neither/both/unknown keys, null/undefined arms, invalid status/time/perspective/template fields reject. Nested ordinary unknown-field behavior matches current authored schema: wrapper closed; referenced snapshot permissiveness preserved.
3. Parser and authored schema share fixtures; malformed HTTP success becomes typed unavailable. `getSnapshot(caseId)` never accepts bootstrap. Wire-to-panel mapping covers all types, defaults, required, options, labels, ordering, and empty lists.
4. Explicit case is exact and nonredirecting; absent query is only bootstrap candidate; duplicate/empty case never becomes unqualified. Historical nonempty cursor remains historical. Inventory failure/unreadable/partial cannot produce `no_cases`; route remains disabled until separately accepted.
5. Startup mounts ordinary app/history only after validated real snapshot; bootstrap has no fake IDs/cursor/history/SSE. Refreshes are session-authenticated, one-at-a-time, at least 1 second between starts, canceled on disposal/session switch/transition, and stale completions ignored.

### Intent attempt state matrix

| Starting state / result | Required next state | Identity/body rule |
|---|---|---|
| First attempt; source-proven pre-dispatch 401/403/429 | `refused` only if typed phase proof is present | Original attempt may clear; later new create still requires fresh preflight |
| First attempt; lost response / timeout / generic network failure | `outcome_unknown` | Keep ID/body; mark `everAmbiguous`; only same-body retry |
| Lost first response; retry receives middleware 401, CSRF 403, or rate 429 | `outcome_unknown` | Retry rejection proves nothing about first attempt; retain original ID/body |
| Lost first response; retry gets cached `UNAVAILABLE` or cancellation/interruption | `outcome_unknown` | Same-ID cache replay is not original commit proof; retain ID/body |
| Any attempt; matching positive accepted body and matching intent ID | `accepted` or `accepted_unresolved` | Never mint/retry; nonempty returned case ID required to open |
| Matching positive acceptance lacks case ID | `accepted_unresolved` | Retain ID/body; disable reset/new create; no fabricated ID or blind resend |
| Any ambiguous state; mismatched/missing correlation, malformed body, generic negative | `outcome_unknown` | Retain exact ID/body; no negative-message heuristic |
| Any ambiguous state; existing validated authoritative terminal result for original ID+body | Terminal accepted/refused state per that result | Only if current interface actually exposes verifiable original-result proof; otherwise this row is unavailable and state remains unknown |
| Logout/session replacement with pending attempt | held/quarantined by session owner; no transfer to new principal | No disclosure or dispatch under replacement identity; page/process loss limitation remains |

Test the critical sequence explicitly: attempt 1 loses response after send; attempt 2 is denied by auth/CSRF/rate middleware; another same-ID retry receives cached `UNAVAILABLE`; all outcomes remain unknown with identical body/ID, and a new ID is impossible. Then separately show matching positive acceptance resolves; missing case ID resolves only to locked `accepted_unresolved`. Also test first-attempt typed pre-dispatch proof as the sole negative that may clear, and show text-only/ordinary `UNAVAILABLE` cannot clear.

### Accepted transition and lifecycle

6. Preflight binds exact selected template/params; edits invalidate it. Intent body is deep-cloned/frozen before first send. Retry uses exact stored object/bytes, increments ordinal only, never calls UUID generation. Unknown blocks changed params/template, replacement preflight, reset, fresh submit, and view-close forget.
7. Same-ID/different-body is never sent. A response with wrong/missing intent ID is unknown. Non-2xx middleware refusal during first attempt only clears with source-backed typed pre-dispatch evidence; same refusal after any ambiguous attempt cannot clear.
8. Generic `UNAVAILABLE`, cancellation/interruption, malformed HTTP/body, lost response, and cache replay preserve pending identity/body. No tests claim gateway restart or page reload exactly-once behavior.
9. Accepted with ID requests only that case; mismatched case, bootstrap response, missing cursor, or malformed response cannot mount history/App/SSE. Missing case ID becomes locked accepted-unresolved and cannot submit another create.
10. Pending controller survives view unmount/remount within its bound session. Principal replacement never displays old request details or dispatches them under the new identity; unresolved cross-session recovery is explicitly not implemented without a separate decision.

## 7. Remaining HOLDs and material revision-3 corrections

This revision preserves the bootstrap shape, template normalization, hand-written parser boundary, no-fake-history lifecycle, fail-closed inventory precondition, create-only schema exception, protected governance, and real-case readiness ordering from revision 2. It does not release the component implementation or public V4.

It corrects revision 2's retry conclusion: pre-dispatch rejection can clear only a first, never-ambiguous attempt with actual typed/source proof; any prior ambiguity is sticky against later negative responses. A same-ID retry receiving middleware denial or cached `UNAVAILABLE` retains the pending ID/body. Matching positive acceptance with a missing case ID becomes terminal `accepted_unresolved` and remains locked. It corrects ordinary nested unknown-property policy to match current authored schema rather than silently closing it. It adds an explicit unresolved privacy/session boundary for a different authenticated principal.

The current adapter does not expose enough provenance to claim all these transitions are executable: its non-2xx/no-body-ID path synthesizes `intent_id` into `HttpRefusalError` (`httpCaseworkAdapter.ts:252-262`). The current gateway can cache a refusal after attempting governed work (`intents.go:134-175,486-509,626-645`). Any future private typed proof addition, authoritative recovery exposure, session-change treatment, or state persistence requires separate source/security review. No recovery endpoint, durable store, cross-principal transfer, nested schema tightening, inventory fix, frontier, cursor grammar, readiness rule, or public contract is invented here.

Root/operator and independent reviewer must adjudicate all held boundaries before source implementation. Inventory/list errors, full V4 frontier/range semantics, actual route error mapping, schema conformance, source gates, runtime claims, and public release remain HOLD.

## Source anchors reviewed

- Current world route and response branches: `apps/godspeed-casework-go/internal/server/server.go:177-231`; session-protected `/api/world` and `/api/templates`: `:137-139`.
- Intent middleware order: `server.go:142-146`; session `401` short-circuit: `internal/server/session.go:85-95,114-123`; CSRF `403` short-circuit: `:144-165`; rate-limit `429`: `internal/server/ratelimit.go:119-142,168-170`.
- Handler perspective verification and dispatch: `server.go:233-256`; response/resulting case construction: `internal/intents/intents.go:282-311`.
- Gateway cache, cached outcomes, refusal mapping: `intents.go:134-175,486-509,626-645`; kernel client correlation recovery: `apps/godspeed-casework-go/internal/adapters/sfwp/client.go:441-477,533-620`.
- Current adapter's lossy non-2xx distinction: `apps/godspeed-cognitive-ui/src/adapters/http/httpCaseworkAdapter.ts:252-262`.
- Current UI submit and controller absence: `apps/godspeed-cognitive-ui/src/app/proposals.ts:12-103`; `src/model/types.ts:409-424`.
- Authored current nested snapshot policy: `.agents/reports/interface-contracts/schemas/cognitive-world.schema.json`; full bootstrap/template/schema anchors retained from revision 2 above.

No source, schema, test, status, debt, compiler, scanner, Graft build, Git, or network operation is authorized or claimed by this proposal.
