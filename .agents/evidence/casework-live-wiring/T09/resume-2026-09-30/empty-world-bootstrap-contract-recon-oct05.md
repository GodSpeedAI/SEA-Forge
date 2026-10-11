# Empty-world bootstrap contract/source recon

Date: 2026-10-05. This is a source-only compatibility inventory for the
cursor V3 design discussion. It chooses no contract or architecture and makes
no source, schema, or runtime changes.

## Canonical snapshot constraints

The frozen Cognitive World schema requires `world_id`, `case_id`, and `cursor`
(`.agents/reports/interface-contracts/schemas/cognitive-world.schema.json:7-17`).
All three are strings with nonempty format constraints: `world_id` must match
`^ws-[a-z0-9_.-]+-[0-9]+\\.[0-9]{10}$` (`:19-23`), `case_id` must match
`^case-[a-z0-9_.-]+$` (`:24-28`), and `cursor` must match
`^[0-9]+\\.[0-9]{10}$` (`:29-33`). Therefore an empty string, omitted cursor,
or invented sentinel cursor does not satisfy the canonical snapshot contract.
The schema describes a purposeful case world and currently has no explicit
empty-cell/bootstrap variant. Spec-04 §3 (`04-REACT-COGNITIVE-ENVIRONMENT-
CONTRACT-SPECIFICATION.md:41-43,106-110`) repeats the snapshot fields and
describes `/api/world` as serving schema-conforming projections.

The project spec calls the frozen interface package authoritative and notes
that its package test checks required fields and selected structural shapes,
but does not perform full draft-2020-12 JSON Schema semantic validation
(`.agents/specs/godspeed.casework-cognitive-environment-spec.yaml:32-59`).
Go's `CognitiveWorldSnapshot` stores these as ordinary strings without runtime
validation (`apps/godspeed-casework-go/internal/contract/contract.go:73-83`),
so serializing a struct is not proof of schema conformity.

The existing Go `EmptyWorld` sets `world_id: "world-empty"`, blank `case_id`,
and its caller-supplied cursor (`internal/projection/live.go:191-204`). The
first two do not match their schema patterns. With no relay head the cursor is
blank; with a global relay head it may be a cursor from an event, but it is
not evidence of a real cursor for any case. The function's comment says only
that the caller supplies a kernel cursor it can vouch for; it does not make a
case-less cursor a real case revision.

## Current Go bootstrap behavior

`LiveSource.NewestCaseID` reads `case.list` and returns empty only when there
are no cases (`internal/projection/live.go:173-188`). For an unqualified
`GET /api/world`, `handleWorld` calls this, and on empty case ID returns HTTP
200 with `EmptyWorld(actor, relay.Head(), now)` (`internal/server/server.go:209-
220`). If a caller explicitly supplies a nonexistent case ID, this branch is
not taken: the gateway tries a case-scoped live snapshot and returns an error
if the case is absent (`:209-230`). Historical requests instead require a
retained Store revision at the supplied cursor (`:188-207`).

The relay maintains per-case cursors only from event frames naming a case
(`internal/server/relay.go:101-111,170-176`). `relay.Head()` is process-wide,
so it is not interchangeable with the per-case cursor. The empty-cell test
currently checks only blank `CaseID` and the "No cases yet" headline; it does
not assert the canonical world ID/cursor or schema validity
(`internal/server/server_test.go:398-414`).

## Current UI bootstrap and no-cursor path

For live mode without a `case` query parameter, `main.tsx` starts with an empty
case ID (`apps/godspeed-cognitive-ui/src/main.tsx:77-89`). `loadCaseHistory`
tries trajectory first, then falls back to `getSnapshot` for that same ID
(`src/ports/project.ts:152-173`). The HTTP adapter serializes that ID into
`/api/world?case_id=` and widens the returned object by type cast without
validating schema fields (`src/adapters/http/httpCaseworkAdapter.ts:233-240,
551-556`). `projectHistory` uses the snapshot cursor as the revision key and
the snapshot case ID as the history case ID, including empty strings
(`src/ports/project.ts:126-143`); `initialState` assumes a last revision exists
(`src/model/store.ts:12-18,44-49`). Thus current UI code can carry the
noncanonical empty bootstrap values into history state; no explicit
empty-bootstrap union or validator separates them from ordinary revisions.

`App` subscribes using `history.caseId` and the last snapshot cursor
(`src/app/App.tsx:97-104`; `src/app/live.ts:92-95`). The HTTP stream parser
rejects events with an empty cursor and drops any payload whose `case_id`
differs from the supplied case ID (`src/adapters/http/httpCaseworkAdapter.ts:
537-548`). An empty-case bootstrap therefore has no resumable real case cursor;
ordinary later case snapshots name a nonempty case and are filtered while the
UI remains subscribed under empty `caseId`. The SSE endpoint itself is global
and does not take a case ID; it replays retained revisions after `last` and
streams new ones (`internal/server/server.go:288-376`). This current behavior
does not itself specify how the first real case becomes the UI's active case.

## Smallest inventory to resolve before implementation

An honest empty bootstrap must not masquerade as a canonical case revision.
The minimum review surface depends on the eventual contract decision, but
source review must include:

1. The frozen schema and types for `world_id`, `case_id`, and `cursor`,
   Spec-04's snapshot/replay prose and examples, the T09 requirement to use a
   latest **real case** cursor for informational events, and the casework
   spec's prior-approval/ADR rule for public contract or architecture changes
   (`godspeed.casework-cognitive-environment-spec.yaml:22-25,82-88`).
2. The Go HTTP response and `EmptyWorld`, including the distinction between
   process-wide relay head and per-case cursor, plus the empty-world test.
3. The UI bootstrap path from `main.tsx` through `loadCaseHistory`, adapter
   parsing, `projectHistory`/initial state, and event subscription filtering;
   decide how empty bootstrap state transitions to the first actual case
   without inventing an event cursor or feeding an empty sentinel into normal
   revision ordering.
4. All canonical schema mirrors/generators, examples, package checks, API
   version/release compatibility, and contract confirmation/ADR artifacts
   named by repository governance. The package test's stated lack of full
   schema validation is a relevant proof boundary.

This list is an inventory, not a proposed design. No contract version,
identifier, cursor, sentinel, HTTP status, or migration path is selected here.
No implementation or verification command was run.
