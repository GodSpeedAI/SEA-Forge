# T08 independent recovery review — round 3 static findings

**Status: REJECT remains in force.** These source findings supplement the round-2 rejection after reviewing the fresh trajectory classifier, native EventSource reconnect, shared conformance suite, artifact provenance join, and T07 retained-facts session reprojection. No Go/Bun/Cargo commands were run in this round; verification remains reserved. The builder's focused Bun result is reported evidence, not independently rerun here.

## F-10 — Stale refresh accepts a missing or malformed case identity (blocking)

The stale path only rejects a different string case ID:
`apps/godspeed-cognitive-ui/src/app/intents.ts:82-85` uses
`typeof fresh.case_id === 'string' && fresh.case_id !== caseId`. Thus a missing,
null, or non-string `case_id` bypasses validation and can be projected and merged
into the requested case history at lines 89-107. This is reachable at the adapter
boundary: `httpCaseworkAdapter.ts:508-512` casts the decoded snapshot without
runtime validation. The current wrong-case regression uses another valid string
(`src/app/intents.test.ts:157-190`) and does not cover absent/malformed identity.
T08 requires the typed stale refusal to trigger a current projection refetch and
show it honestly (`.agents/plans/2026-09-23-casework-live-wiring-production.plan.yaml:499-500`);
the separately assigned race repair explicitly requires validating returned case
identity. A missing identity cannot establish that the returned projection belongs
to the requested case.

**Required:** require `fresh.case_id === caseId` before projection or merge, keep
the stale refusal and report refresh failure otherwise, and add missing/null or
wrong-type response coverage. Preserve the passing delayed case-switch behavior.

## F-11 — Browser retry backoff resets on connection-open without source recovery (blocking)

The native branch has capped exponential retries, but `current.onopen` resets the
counter to the base delay before any source event is received
(`src/adapters/http/httpCaseworkAdapter.ts:309-310`). An open-then-drop sequence
therefore repeatedly retries at the base delay instead of increasing backoff.
The adapter's actual source-backed reset at lines 319-322 is enough; the earlier
open reset defeats it. T08 requires an EventSource path with Last-Event-ID resume
and backoff (`.agents/plans/2026-09-23-casework-live-wiring-production.plan.yaml:479-486`).

**Required:** reset backoff after a valid source-backed event, not merely an open
connection; test open-then-immediate-error cycles and verify the cap is reached.

## F-12 — `resync_required` advances the cursor and suppresses the first replayed snapshot

In the EventSource route, `lastCursor` is advanced from every event's
`lastEventId` before the event is parsed or classified
(`httpCaseworkAdapter.ts:312-322`). When the requested cursor is older than
retention, the server sends `resync_required` with SSE id `oldest` and then
replays retained revisions after the caller's old cursor
(`apps/godspeed-casework-go/internal/server/server.go:295-309`). That replay
includes the oldest snapshot with the same id. Since `lastCursor` was advanced by
the control event, the duplicate guard drops that source snapshot as
`cursor <= lastCursor` (`httpCaseworkAdapter.ts:314-318`). `connectLive` handles
`resync_required` by marking the connection recovering; it does not request a
fresh snapshot (`src/app/live.ts:95-104`). If the oldest revision is the only
retained frame for the case, there is no later state frame to mark recovery, so the
UI remains reconnecting/interrupted despite receiving no authoritative snapshot.

**Required:** do not advance the source resume cursor for a control event, or make
resync explicitly fetch and install a session-authenticated current snapshot before
declaring recovery. Add a real server/adapter regression for the single-retained-
revision case and confirm no gap or duplicate behavior.

## Evidence status and remaining gates

- T08 F-7/F-8 were corrected in source: only `COMPLETED` contributes to completed
  trajectory count, and the current kernel `TraceKind` set is explicitly
  classified. No focused Go test was run by this critic.
- T08 F-9 now has a native EventSource test reported in
  `round2-fixes/bun-focused.log`; I have not rerun it. The open-reset and
  resync-control interactions above are not covered by that test.
- The shared suite is now one assertion implementation wired to local tests and
  `e2e/live-conformance.ts`. Local focused output is reported, but the suite has
  not run against a fresh live kernel/gateway cell. A real same-suite run remains
  required; mocked adapter tests are not live evidence.
- Artifact ownership is now derived by joining `artifact.get` to `run.get`, then
  checking the run evidence row and its `artifact_captured` trace reference
  (`internal/adapters/sfwp/artifact_provenance.go`). The real gateway artifact
  digest/provenance proof remains pending.
- T07's captured-facts re-projection change is present, with a source test for a
  second user's historical and SSE replay perspective in
  `internal/server/session_read_test.go`. It remains uncompiled/unexecuted by this
  critic and must be covered in fresh Go gates and live cross-session checks.
- Production default/local/invalid bundle checks, the broad UI gate, fresh Go
  gates, live shared suite, and live SSE reconnect/resync checks are outstanding.

Do not approve T08 from this static review or reported builder logs. Require fresh
fixes for F-10 through F-12, the complete T08 verification window, and independent
inspection of its logs before settlement.

## Final handoff state

The TS stream/identity builder is still editing the F-10–F-12 repairs and trajectory
response case/shape validation. Any edits made after the source anchors above were
reviewed are **not reviewed** by this critic. No new gates were run; all listed live,
bundle, and cross-session evidence remains pending. Keep T08 rejected until the
builder reports a stable diff and an authorized verification window produces fresh
results for the required UI and live-kernel gates.
