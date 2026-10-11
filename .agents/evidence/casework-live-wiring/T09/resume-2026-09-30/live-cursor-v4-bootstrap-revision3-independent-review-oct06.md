# Live cursor V4 bootstrap proposal revision 3 — independent review

Date: 2026-10-06  
Reviewed proposal SHA-256: `7626878fce73c63f276847e5bf3670e9ef1395687966a937fec38386d20b0dcb`.  
Reviewed cursor erratum SHA-256: `882c2f05ebfa903ce5eb44faa0348021c66719a02e417808ba325277f5a9f2c4`.  
Verdict: **ACCEPT as a DOCONLY component proposal, with public V4, implementation, schema changes, and no-case success still HOLD.**

## Scope and artifacts

Reviewed the original assignment, revision 2, its independent review, the complete revision 3 proposal, the revision 3 cursor erratum, the revision 3 boundary-recon review, and the source/schema anchors cited below. This review is not source authorization or approval of the public V4 contract. No source, test, compiler, scanner, Git, or network operation was performed.

## Revision 2 findings and resolution

The revision 2 independent review rejected its retry conclusion: a refusal received on a retry cannot prove an earlier ambiguous attempt did not commit. Revision 3 repairs this. Its attempt rules retain the original ID and immutable body once any attempt is ambiguous; later 401/403/429, unavailable, cancellation, interruption, malformed response, and cached refusal do not clear that state. Its test matrix explicitly covers a lost first response, middleware-denied retry, and cached unavailable replay. This is consistent with the current adapter synthesizing the intent ID on body-less non-2xx responses (`apps/godspeed-cognitive-ui/src/adapters/http/httpCaseworkAdapter.ts:252-262`) and the gateway caching handler outcomes (`apps/godspeed-casework-go/internal/intents/intents.go:134-175,486-509,626-645`).

Revision 2 also required a terminal state when acceptance lacks a resulting case ID. Revision 3 specifies `accepted_unresolved`, retains the ID/body, and blocks retry, reset, edits, and new creates. It does not imply case readiness: the documented handler wait is bounded and the relay returns its last published value on timeout/feed end (`apps/godspeed-casework-go/internal/intents/intents.go:294-311,418-431`; `apps/godspeed-casework-go/internal/server/relay.go:178-217`).

## Assignment coverage and source alignment

- **Canonical template mapping:** The proposal adds one shared validated wire-to-app mapper, preserving order and optionals, mapping `type` to `param_type`, absent `required` to false, and `default_value` through the existing string conversion. It specifies coverage across all wire types/defaults/options. This matches the canonical wire shape and distinct app-port shape (`.agents/reports/interface-contracts/typescript/types.ts:549-567`; `apps/godspeed-cognitive-ui/src/ports/contract.ts:223-247`) and current adapter conversion (`httpCaseworkAdapter.ts:451-474`).
- **Runtime validation:** It specifies a hand-authored parser over `unknown`, strict one-arm wrappers/bootstrap/templates, safe typed failure, and full ordinary-snapshot validation. It explicitly preserves the existing nested snapshot schema's permissive unknown-property behavior rather than closing nested objects. That matches the authored `cognitive-world.schema.json`; the existing adapter currently casts the response rather than validating it (`httpCaseworkAdapter.ts:233-248`). It does not claim a runtime schema generator.
- **Bootstrap and route boundary:** The four-field bootstrap arm, actual capture time, session-verified perspective, canonical entry options, absence of fabricated identity/cursor/history, explicit-case nonredirect behavior, and disabled no-case success pending independently accepted complete-empty inventory all meet the assignment. Current listing has known uncertainty: Rust listing is infallible and can collapse/skip filesystem errors (`crates/sea-forge-server/src/sfwp/case_views.rs:249-270,309-355`), and Go drops `Unreadable` (`apps/godspeed-casework-go/internal/adapters/sfwp/authority.go:69-88`). The proposal accurately leaves this held.
- **UI and lifecycle:** Bootstrap stays outside history/cursor/SSE; only validated ordinary snapshots mount normal history and streaming. Refresh serialization, minimum interval, cancellation, stale-result handling, and error-vs-empty distinction are specified. Accepted create requires the returned case ID and a matching ready ordinary snapshot before transition.
- **Intent ownership and privacy:** The session-shell controller owns a frozen ID/body across remounts. Same-body retry is the only retry while unknown; no status endpoint, durable persistence, or cross-principal transfer is invented. Principal replacement is explicitly a held privacy boundary, including the page/process-loss limitation.
- **Governance and scope:** Session, CSRF, rate limit, verified actor/role, preflight, idempotency, and kernel governance remain required. The create-only empty-ID/cursor exception remains a separately reviewed contract change. The proposal and erratum authorize no source/public contract changes and retain operator adjudication before implementation.

## Cursor erratum verification

The erratum is accurate. `apps/godspeed-casework-go/internal/server/server.go:188` enters retained-history lookup only when `r.URL.Query().Get("cursor")` is nonempty; `:189-207` performs `Store.At` and returns the retained snapshot. An absent or explicitly empty cursor falls through to the live branch at `:209-230`, which uses explicit `case_id`, otherwise `NewestCaseID`, and can return the existing synthesized empty world. The erratum limits itself to this classification correction and does not claim inventory completeness or authorize a bootstrap response. It properly preserves the prior boundary-recon review unchanged.

## Material deviations and limits

No material deviation from the assigned DOCONLY proposal scope remains. The unresolved inventory proof, create-only schema reconciliation, cross-principal quarantine policy, runtime parser implementation, route behavior, and full V4 frontier/readiness are acknowledged HOLDs rather than omissions disguised as resolved work. Acceptance here does not settle those choices or permit implementation.

No source or generated file was changed. This independent review is the only new artifact.
