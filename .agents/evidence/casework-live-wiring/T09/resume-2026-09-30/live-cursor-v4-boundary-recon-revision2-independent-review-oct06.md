# Independent review: Live cursor V4 boundary recon revision 2

Date: 2026-10-06  
Reviewed revision: `live-cursor-v4-boundary-recon-revision2-oct06.md`, SHA-256 `8f62484d18537be93a8ca296b9c0b3b92437809ff8d4240613b26babebc91b82`.  
Disposition: **REJECT as a fully verified factual record pending two bounded corrections.** This verdict is factual only; public V4 remains HOLD.

## Coverage repaired and source-verified

Revision 2 addresses both omissions from review `live-cursor-v4-boundary-recon-independent-review-oct06.md`:

- The Go path exempts only `PROPOSE_CASE` from the mutating intent staleness check (`apps/godspeed-casework-go/internal/intents/intents.go:154-163`); the code comment ties its freshness to the kernel preflight digest because the target case does not exist yet. `validate` requires `template_ref`, `preflight_digest`, and a nonnil `params` object (`:178-196`). `execute` maps it to `CommitCase` with the digest (`:282-298`) and then waits with an empty prior cursor (`:299-301`). The new note correctly distinguishes implementation from normative schema validity and separately records that the interaction-intent schema requires nonempty patterned `case_id` and `client_cursor` while omitting `PROPOSE_CASE` from its action enum (`interaction-intents.schema.json:7-15,22-44,49-56`).
- The `/api/world` branch inventory is accurate: authenticated cursor-qualified requests resolve retained history with `Store.At` (`internal/server/server.go:177-207`); no-cursor requests use explicit `case_id` or `NewestCaseID`, return an empty-world response if no case is selected, and otherwise call the live snapshot source at the relay cursor (`:209-230`). The document does not turn this current behavior into a V4 route decision.

The original supported facts are retained accurately: `case_id()` produces the underscore/timestamp form (`crates/sea-forge-core/src/ids.rs:32-47`); Go and local UI world ID builders do not append the cursor (`internal/projection/builder.go:99-104`, `localAdapter.ts:314-316`); the frozen schemas disagree (`cognitive-world.schema.json:19-32`); UI proposal submission preserves passing-preflight gating and uses empty create-case IDs/cursor (`proposals.ts:60-92`); kernel listing currently has no fallible operational `Result` path and collapses/skips multiple enumeration failures (`case_views.rs:249-270,309-355`; request arm `crates/sea-forge-server/src/lib.rs:2396-2397`); Go list decode does not validate both required arrays or reject unreadables (`authority.go:69-88`, `frame.go:664-667`); range responses contain only events, with an output cap after whole-ledger materialization (`sfwp/events.rs:165-207`, `lib.rs:2349-2361`, `sea-forge-ledger/src/types.rs:784-801`); the successful case-commit/in-memory insert precedes `case.submitted` (`lib.rs:2788-2822`); and Store/SSE remains global with no per-case readiness boundary (`store.go:55-65,149-220`; `server.go:293-325`).

The disposition correctly preserves the public HOLD and does not claim a schema validation result from the HTTP handler. Its inventory-repair discussion remains an option rather than an approved API/error-class change. The immutable capture, no fabricated backlog, cursor ordering, and gap/readiness assertions remain bounded by prior V3 review, not upgraded to approval.

## Corrections required

1. **Broken V3 review filename.** The first section links `live-cursor-v4-contract-v3-independent-review-oct05.md`, but the actual record is `live-cursor-contract-v3-independent-review-oct05.md`. Repository inventory confirms the latter exists and the former does not. Correct the filename in a fresh immutable note; do not edit the reviewed artifact in place.
2. **Observed versus successfully retained cursor wording.** The new Go-consumer paragraph says `postMutationCursor` waits for the new case's “first observed cursor.” The implementation calls `Relay.WaitForCaseAdvance` (`intents.go:299-301`), whose contract and implementation return the newest successfully retained/published cursor (`internal/server/relay.go:178-209`); observed cursors are separately tracked by `CursorForCase` (`:170-175`). Replace “observed” with “successfully retained/published,” and keep the limitation that timeout/feed end may return the last published cursor (or empty) rather than proving a new ready case snapshot. The recon already correctly says this does not prove inventory/frontier completeness.

These are factual-reference/precision defects, not V4 architecture decisions. Until corrected, the revision is not approved as the complete factual record requested by its assignment.

## Material differences, limits, and next step

Compared with original recon `live-cursor-v4-boundary-recon-oct06.md` (SHA-256 `f3aed6892086562f73e31ac868e97fe2361b96cb04966ee5636267ea8c4f0a2d`) and its independent review (SHA-256 `861926e0a103b89cbd9ddf97f8202dedde2b788a2c9d1ca83ef571861f6e7598`), revision 2 adds the requested Go staleness/payload/commit path and exact live-versus-historical world selection, while retaining previous inventory/frontier/identity gaps. No material source finding was found to be reversed. The record remains source recon only: no proof of complete inventory, atomic frontier, stable filesystem snapshot, or V4 readiness exists.

Produce a new corrected factual record with the two wording/reference fixes, then obtain a fresh critic if complete factual approval is needed. Do not infer V4 contract, public schema/API, operator, runtime, or implementation approval from that correction. No source, tests, compiler, scanner, Graft build, Git, status, debt, or network work was performed.
