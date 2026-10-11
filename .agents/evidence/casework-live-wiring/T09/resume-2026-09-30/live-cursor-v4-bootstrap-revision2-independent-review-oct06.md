# Live cursor V4 bootstrap proposal revision 2 — independent review

Date: 2026-10-06  
Reviewed revision 2 SHA-256: `532e3d03acf1dd69d0fe4f920ffbfe4a48b0ca7a5f8c2b23e635044e2d94e037`.  
Disposition: **REJECT as a complete implementation blueprint; preserve
component, public contract, inventory, and V4 HOLDs.** This is DOCONLY review.

## Artifacts and review boundary

I read the original bootstrap assignment, predecessor `e02ab804d95bf9aeb02878c6057716be8c7d24f0863dd5efc100408f4368bcc1`, its independent rejection `f330603a176865150bce049c32ddfb11c7bf650562f12b69c936de52ad9f7894`, all of revision 2, the current canonical/app-port DTOs, authored schemas, HTTP adapter, UI proposal flow, Go intent handler, server route and middleware, and kernel request-correlation source. No code or contract work is approved by this review.

Revision 2 preserves the cursorless bootstrap arm and correctly keeps it disabled until a separately accepted fail-closed inventory and complete-empty proof exist. It preserves explicit-case nonredirect behavior, no synthetic history/cursor, ordinary snapshot readiness before mounting history or SSE, serialized authenticated refresh, proposal/preflight/governance boundaries, and the stated absence of atomic inventory/frontier guarantees.

## The three predecessor findings are materially repaired

### Canonical template normalization

Revision 2 now defines one `toTemplateEntryOptions` boundary shared by the
bootstrap parser and existing `HttpCaseworkAdapter.getTemplates`, rather than
feeding canonical wire values straight to the UI. It explicitly maps `type`
to `param_type`, absent `required` to `false`, canonical `default_value` to the
existing string-valued `default` conversion, and preserves enum options,
optional title/description, and template/parameter order. It requires primitive
validation before conversion and gives a mapping matrix for all four types,
required states, default types, options, and empty arrays. This matches the
actual distinct DTOs: canonical wire fields in
`.agents/reports/interface-contracts/typescript/types.ts:549-567`, app-port
fields in `apps/godspeed-cognitive-ui/src/ports/contract.ts:223-247`, and the
current conversion in `src/adapters/http/httpCaseworkAdapter.ts:451-474`.

### Runtime response validation boundary

The revision proposes the concrete new module
`apps/godspeed-cognitive-ui/src/adapters/http/worldResponse.ts` and exact
`unknown -> ParsedWorldResponse` parser boundary, with a safe typed validation
error and adapter conversion to typed unavailable. It specifies one-of arm
presence, null/undefined rejection, strict wrapper/bootstrap/template
properties, canonical template validation before normalization, and validation
of the whole nested ordinary snapshot rather than truthiness/cast. It states
that validation is hand-authored and does not load JSON Schema at runtime or
assume a generator. Shared positive/negative fixtures between parser and schema
conformance are required. This corrects the predecessor's missing parser
location/full-snapshot boundary and nonexistent implied generation path.

One scope detail remains for future implementation: the current authored
`cognitive-world.schema.json` does not set `additionalProperties: false` on its
nested snapshot objects. The new response wrapper can be closed while a
referenced ordinary snapshot schema remains permissive at nested object
boundaries. The parser must match the subsequently approved authored schema;
this proposal must not silently claim every nested unknown property is already
forbidden or tighten the snapshot contract without a separate schema decision.
Revision 2 at least names an unknown-property policy and requires schema/parser
fixtures; this is a source/schema alignment prerequisite, not evidence of an
existing runtime validator. The current adapter still blindly casts
`body.snapshot` (`httpCaseworkAdapter.ts:233-248`), and the current contract test
enumerates schemas rather than generating runtime code
(`.agents/reports/interface-contracts/tests/contract-conformance.test.ts:130-138`).

### Stable pending create owner

Revision 2 supplies a session-shell-owned `ProposalController`, explicit
`PendingCreate`, immutable full body/intent ID, same-body retry, no edits/new
submit/reset while unknown, persistence through view disposal/remount, and
no browser reload/process restart recovery claim. It also says logout/session
replacement must not turn uncertainty into refusal. This fills the predecessor
owner/state boundary. Current implementation has no pending attempt state
(`apps/godspeed-cognitive-ui/src/model/types.ts:409-424`) and creates a fresh
`uuid()` inside every `submit()` (`src/app/proposals.ts:60-78`), so the
proposal is not describing current behavior.

## Blocking retry ambiguity: a retry-only refusal cannot resolve the first attempt

Revision 2 section 6, immutable attempt rule 5, treats a non-success HTTP
response without `intent_id` as conclusive non-submission and says a conclusive
refusal clears `pendingCreate`. That conclusion is valid only when this is the
first attempt and the response proves it never crossed the intent handler.
After a prior attempt became ambiguous, an HTTP rejection on a retry proves
only that the retry did not enter the handler. The earlier request may already
have committed. A 401/403/429/CSRF refusal on retry must therefore leave the
original pending body and unknown status intact; it cannot authorize an
altered intent ID or a fresh create.

The existing adapter's `dispatchIntent` throws
`HttpRefusalError(..., intent.intent_id)` for **any** non-success response
without a body `intent_id` (`apps/godspeed-cognitive-ui/src/adapters/http/httpCaseworkAdapter.ts:252-261`).
That code has no attempt ordinal or proof that an earlier same-ID call did not
reach the server. The current server route applies session, CSRF, and rate
limit middleware before `handleIntent`, then overwrites the client actor and
calls the dispatcher (`internal/server/server.go:142-146,233-256`). Thus a
retry-time middleware refusal says nothing about the previous request's
outcome. Revision 2 must scope any “pre-dispatch” conclusiveness to an attempt
that has never previously been ambiguous.

Typed intent refusals also need outcome-sensitive treatment. The Go handler
records its returned response in its in-memory `outcomes` map after `execute`
returns, including mapped errors (`apps/godspeed-casework-go/internal/intents/intents.go:117-171,486-543,640-645`). `UNAVAILABLE` includes unreachable authority; `request_cancelled` and `request_interrupted` are mapped as unavailable. A transport/cancellation failure can occur after a mutation reached the kernel, so a cached `success:false` `UNAVAILABLE` is not proof of non-commit. The handler's next same-ID call in that process replays its cached response before another kernel call (`intents.go:138-140,626-638`). Consequently, “same ID/body retry” is the correct identity rule but does not by itself prove the UI can obtain a conclusive result from an ambiguous in-process cached refusal.

There is a durable kernel request-correlation layer: the Go dispatcher uses
`intent_id` for the governed request ID (as its package comment and execute
path state), and Rust correlation records pending/terminal outcomes by request
ID and payload hash (`crates/sea-forge-server/src/sfwp/correlation.rs:156-202,227-322,409-430`). The shared SFWP client also has a request-status recovery path (`apps/godspeed-casework-go/internal/adapters/sfwp/client.go:453-611`). Those are useful existing facts, not a UI-visible proof for every refusal: the gateway's in-memory replay can return first, and a lost response may leave a pending or interrupted request. Revision 2 correctly avoids cross-restart guarantees, but its current refusal-clearing rule still permits duplicate case creation after a prior ambiguous attempt.

**Required correction:** retain `outcome_unknown` and the exact original
identity/body whenever any earlier attempt is ambiguous. A subsequent
pre-dispatch error, `UNAVAILABLE`, cancellation/interruption, or other result
without evidence conclusively tied to the original request must not clear it.
Only a matching positive accepted response or an existing authoritative,
durable result that proves the original request's final outcome may settle that
attempt. Do not invent a status endpoint or stronger runtime/idempotency claim
in this component document. If the existing interfaces cannot distinguish
these cases, keep the attempt unknown and prohibit new/altered IDs; record the
need for a separately reviewed resolution path.

There is a related acceptance edge: a nominal successful intent response that
omits `resulting_object.id` establishes an accepted outcome but not an openable
case. Revision 2 says to show a conclusive protocol error and not create
another case, while its general accepted/refused rule clears `pendingCreate`.
Specify a terminal accepted-but-unresolved controller state that cannot reset
to a fresh proposal; do not erase the only attempt identity and accidentally
allow a new create. This is not a reason to retry the same accepted intent
blindly.

## Governance and remaining component boundaries

The proposal preserves the existing security chain and should retain it in any
later implementation: `/api/intents` is session + CSRF + rate limited, then
session perspective is verified before the handler
(`server.go:142-146,233-256`); the server overwrites actor from the session;
the handler checks consequential action, role offer, payload/template/digest,
and only then maps `PROPOSE_CASE` to governed `CommitCase`
(`intents.go:117-196,282-311`). The authored intent schema still requires
ordinary nonempty case/cursor patterns and omits the action enum entry, so the
narrow create-only empty-ID/cursor exception remains a separate authored
schema/contract decision. No security, CSRF, identity, preflight, idempotency,
or kernel governance rule may be weakened to implement bootstrap.

The complete-empty inventory correction, exact public error mapping, route
response/schema changes, startup shell wiring, accepted-case ready-snapshot
classification, parser implementation, and all source/test gates remain
unapproved. The revision is explicitly DOCONLY; neither it nor this review
authorizes a public bootstrap response, full V4, source change, runtime claim,
or inventory/frontier conclusion.

## Verdict

**Reject revision 2 as a complete implementation blueprint pending the
retry-outcome correction.** It resolves the three predecessor findings
substantially, and its template normalization and parser boundary are now
reviewable. The unresolved replay rule can turn a prior ambiguous commit into a
second create, violating the very immutable-attempt/idempotency boundary the
controller is meant to preserve. A fresh document revision should separate
first-attempt pre-dispatch proof from later retry failures, keep ambiguous
attempts locked until original-outcome evidence exists, and add the
accepted-without-case-ID terminal state. Root/operator approval is still
required after independent review; component review is not V4 authorization.

No source, tests, schema, status, debt, compiler, scanner, Graft build after
retrieval, Git, or network operation was performed. No runtime outcome is
claimed.
