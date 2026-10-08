# Live cursor V4 boundary recon revision 3 — independent factual review

Date: 2026-10-06  
Reviewed revision 3 SHA-256: `fbd7a5f482f32cd632e23479d536fe7a03306ab717cdd0400b30c30bdf3c9e59`.  
Verdict: **Factual claims substantially source-supported; correct one query
presence precision before calling the recon complete. Public V4 remains HOLD.**

## Reviewed artifacts

- Revision 3 original assignment: `live-cursor-v4-boundary-recon-revision3-original-assignment-oct06.md`.
- Prior rejection/review: `live-cursor-v4-boundary-recon-independent-review-oct06.md`.
- Candidate: `live-cursor-v4-boundary-recon-revision3-oct06.md`.
- Related trace source continuity recon: `run-get-trace-source-continuity-recon-oct06.md`.

I read the full revision 3 and prior review. I checked the corrected filename,
the Go intent consumer and relay wait, kernel/UI identity producers, current
world route, inventory, range path, Store/SSE and the traced Rust reader/writer
paths. This is factual verification only; no architecture, schema, public
contract, or source authorization follows.

## Required revision 2 repairs are supported

1. The V3 review reference is now the real path
   `live-cursor-contract-v3-independent-review-oct05.md`; it exists. The
   previous misspelled `live-cursor-v4-contract-v3-independent-review-oct05.md`
   does not.
2. `Handler.execute` calls `postMutationCursor(ctx, receipt.CaseID, "")`
   (`apps/godspeed-casework-go/internal/intents/intents.go:294-301`). The
   helper bounds the wait at five seconds (`:418-431`). Actual
   `Relay.WaitForCaseAdvance` reads the `published` map, and on context/feed
   termination returns the last published value (or empty), not the
   independently observed `caseCur` value
   (`apps/godspeed-casework-go/internal/server/relay.go:178-217`, compared
   with `:170-176`). Revision 3 accurately says this is not proof of a new
   commit frame, ordinary snapshot readiness, complete inventory, or frontier.

The consumer coverage requested by the prior review is present and accurate:
`PROPOSE_CASE` has the narrow no-existing-case staleness exception and
preflight-digest basis (`intents.go:154-163`), validates template ref, digest,
and params (`:178-196`), and maps to `CommitCase` (`:282-301`). The canonical
intent schema still requires nonempty ordinary case/cursor shapes and does not
list `PROPOSE_CASE` (`.agents/reports/interface-contracts/schemas/interaction-intents.schema.json:7-15,26-56`).
The recon does not pretend this implemented Go/UI exception is schema-valid.

## Other checked source findings

- Kernel case IDs are `case_<UTC timestamp>_<random>` from
  `crates/sea-forge-core/src/ids.rs:32-47`; Go projection and local UI use
  `world-` prefixes while the authored Cognitive World schema expects the
  different world/case patterns (`apps/godspeed-casework-go/internal/projection/builder.go:99-104`,
  `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.ts:314-316`,
  `.agents/reports/interface-contracts/schemas/cognitive-world.schema.json:19-32`).
  The incompatibility is correctly stated without recommending silent rekeying.
- UI creates cursorless `PROPOSE_CASE` only after preflight and then loads the
  returned case's history (`apps/godspeed-cognitive-ui/src/app/proposals.ts:60-122`).
  Revision 3 properly separates that flow from proof of a new bootstrap or
  ready-snapshot protocol.
- The current Rust list path is infallible and can turn `read_dir`/iterator
  issues into apparently empty output; Go's `ListCases` ignores `Unreadable`
  (`crates/sea-forge-server/src/sfwp/case_views.rs:310-355`,
  `apps/godspeed-casework-go/internal/adapters/sfwp/authority.go:69-88`). The
  direct handler serialization and empty-world branch do not establish complete
  inventory (`crates/sea-forge-server/src/lib.rs:2396-2397`,
  `apps/godspeed-casework-go/internal/server/server.go:209-220`).
- `events.get_range` loads ledger entries before filtering/capping, returns
  only event rows, and does not expose a frontier token
  (`crates/sea-forge-server/src/sfwp/events.rs:165-207`,
  `crates/sea-forge-server/src/lib.rs:2349-2361`,
  `crates/sea-forge-ledger/src/types.rs:784-801`).
- The global Store/SSE paths do not supply the proposed case-specific
  readiness boundary (`apps/godspeed-casework-go/internal/projection/store.go:149-220`,
  `apps/godspeed-casework-go/internal/server/server.go:293-325`). The
  successful publication ordering cited is narrow and does not prove an atomic
  inventory (`crates/sea-forge-server/src/lib.rs:2788-2822`,
  `crates/sea-forge-case-runner/src/lib.rs:570-575`).

These findings preserve the limits already stated in the prior review. No
claim of an atomic filesystem view, case frontier, public V4 readiness,
continuous inventory completeness, or production implementation is supported.

## Remaining factual precision issue

Revision 3 says that if `?cursor=` is “present,” the current handler uses the
historical path. The actual handler checks `if raw := Query().Get("cursor");
raw != ""` (`apps/godspeed-casework-go/internal/server/server.go:188`). A
nonempty cursor uses `Store.At`; an explicitly empty `?cursor=` falls through
to the live case/newest-case path. Revise the sentence to say “a nonempty
cursor value” (or explicitly note empty-value behavior). The later summary
calling this simply “cursor → retained historical snapshot” should carry the
same qualification. The explicit `case_id` empty/absent behavior is otherwise
accurately described: both produce an empty string and enter `NewestCaseID`
(`server.go:209-220`).

This is a small but real request-path difference. No other material source
contradiction was found in the claims checked. The earlier review already
audited the wider inventory/frontier and Store/SSE anchors; revision 3
preserves them with explicit limitations.

## Verdict and boundary

The revision resolves the two assigned review findings and adds the missing
consumer path, but it is not fully exact on an empty cursor query. Record that
qualification before treating this factual recon as complete. Once corrected,
it can be accepted as source recon only. It does not authorize the bootstrap
component, alter the rejected V3 verdict, or approve any public contract,
frontier, inventory behavior, route, or implementation.

No source, test, compiler, scanner, Graft build, Git, or network action was
performed for this review.
