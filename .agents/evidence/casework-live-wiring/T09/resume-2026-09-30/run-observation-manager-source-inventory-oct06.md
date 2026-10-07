# Run-observation manager source inventory

Date: 2026-10-06. Read-only source recon for the next Unit5C manager decision.
No implementation, gate, or completion claim is recorded here.

## Existing DTO, ports, and integration seams

- The existing Go mirror already defines the named informational event and
  complete cohort/run DTOs in `apps/godspeed-casework-go/internal/contract/contract.go:167-238`.
  Count fields are optional pointers; the initial logical read budget and safe
  per-run frame counts are represented. Normative bounds are in
  `.agents/specs/godspeed.casework-cognitive-environment-spec.yaml:82-106`:
  eight selected runs and initial reads per cohort, 16 pollers per process, two
  concurrent run.get reads, one-second minimum polling, and 1,024 retained
  frames per run. The event is an informational side channel with no SSE id or
  historical snapshot mutation.
- Existing source ports are `ports.CaseAuthorityPort.RunsListForCase` at
  `internal/ports/ports.go:442-470` and `ports.RunTracePort.ReadRunTrace` at
  `internal/ports/run_trace.go:5-8`. `sfwp.Authority` implements both.
  `RunsListForCase` makes a case-scoped `run_list`, requires both arrays, and
  rejects malformed, duplicate, overlapping, or foreign rows
  (`internal/adapters/sfwp/authority.go:254-307`). `ReadRunTrace` makes one
  logical `Client.Do(NewRunGet(runID))` and checks returned run/case/item
  identities (`internal/adapters/sfwp/run_trace.go:66-98`).
- The production server gets its `*sfwp.Authority` at
  `cmd/godspeed-casework/main.go:262-274`; auth sessions are already available
  through `authOpts.Sessions`. The live stream seam is the protected existing
  `/api/events` route (`internal/server/server.go:137-141,288-375`). `writeSSE`
  supports an event without an SSE id when called with an empty `id`
  (`internal/server/http.go:184-198`). Existing server `Options` and `Server`
  have no observation manager/source fields (`http.go:23-48`, `server.go:72-104`).
  No producer exists yet; `.agents/CURRENT_STATUS.md:107-113` leaves shared
  pollers and server cancellation/drain pending.

## Authorization and actor identity path

- `requireSession` resolves a cookie session or configured dev bearer and puts
  that request identity in context (`internal/server/session.go:64-95,111-141`).
  `handleEvents` obtains its actor from that identity and calls
  `verifySessionPerspective` before streaming (`server.go:293-301,385-399`).
- Production `LiveSource` is assembled with the configured gateway actor in
  `cmd/godspeed-casework/main.go:226-229`. Its `VerifyPerspective` calls
  `ResolveIdentity` with gateway `Actor` and the request actor as
  `OnBehalfOf` (`internal/projection/live.go:317-339`). The adapter serializes
  both claims (`internal/adapters/sfwp/authority.go:54-60,782-805`);
  `NewIdentityGet` applies governance to the request body
  (`internal/adapters/sfwp/frame.go:55-60,164-170`).
- The subsequent run reads do not carry that actor. `NewRunListForCase`
  supplies only `case_id` and `NewRunGet` only `run_id`
  (`internal/adapters/sfwp/frame.go:298-313`); `EncodeRequest` copies only the
  request body and adds `verb` (`:120-131`). Neither source-port method accepts
  an actor claim. Rust classifies `RunList` and `RunGet` as unprotected,
  read-only projections (`crates/sea-forge-server/src/identity.rs:803-845`),
  and dispatches directly to the filesystem-backed views
  (`crates/sea-forge-server/src/lib.rs:2404-2414`).
- Therefore the current ports can perform the manager reads after a watcher’s
  separate successful `VerifyPerspective`; they cannot perform or attribute
  those reads as `on_behalf_of` that watcher. The read result is not kernel
  actor-filtered. A process-shared cache/poller keyed only by `(case, run)` is
  not an approved identity decision: actor, role, and session isolation and
  per-subscriber authorization/fanout must be settled before choosing any
  sharing key or making a cross-watcher sharing claim. No identity-model change
  is needed merely to call the existing read ports, but the current ports do
  not provide per-watcher kernel authorization for those reads.

## Session lifecycle and unresolved boundaries

- `auth.SessionStore.Current` returns a detached actor/role claim, an idle /
  absolute deadline, and the stable revocation channel without sliding
  `LastSeen` (`internal/auth/session.go:53-59,146-169`). Removal closes that
  signal on logout, expiry, sweep, and capacity eviction (`:171-225`).
- No session identity/role mutation path was found: `Start` stores the
  authenticated `Identity`; `Resolve` only slides `LastSeen`; `Current` reads
  the stored claim. Consequently `Current` and `Revoked` track session lifetime,
  not a kernel role/delegation change. Perspective verification currently
  occurs when the SSE connection opens, not on each poll. A kernel-side role or
  delegation change is not observed by the session signal or `Current`.
- `requireSession` also permits a dev bearer, which has no `Session` pointer or
  revocation channel (`internal/server/session.go:64-70,126-141`). Whether
  observation polling supports bearer watchers remains unresolved.
- `/api/events` currently reads only `last` / `Last-Event-ID` and subscribes to
  global retained revisions (`server.go:308-318`); it has no case selector.
  Cohort case selection and how it relates to the current case-scoped stream
  must be decided without changing cursor/event identity semantics.
- Root’s physical-admission adjudication binds logical accounting and failure
  behavior: each invocation of `ReadRunTrace`, including one that fails while
  queued, consumes a unit; client retries do not. Continue through already
  selected candidates after an individual failure while the session remains
  valid, select no replacements, stop new starts on cancellation, and cancel
  and join outstanding reads (`run-trace-physical-admission-root-adjudication-oct06.md:55-62`).
  Shared-poller reuse across reconnect cohorts still needs precise counter
  semantics against the DTO’s `reads_attempted` field.
- The list response separates readable summaries from `UnreadableIDs`
  (`ports/ports.go:291-296`). Existing source recon records unresolved
  response-complete versus source-complete language, listed-count treatment of
  unreadable IDs, current-horizon parent validation, and deterministic recency
  fallback (`observation-cohort-runlist-recon-oct05.md:79-120`).

This inventory makes no decision about authorization partition, bearer behavior,
role-change revalidation, case selection, poller sharing keys, or cohort count
semantics.
