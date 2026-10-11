# Primitive checkpoint manifest proposal — 2026-10-07

Status: proposal only. No paths were staged or committed by this work. The
canonical Go retry and full-module race remain pending; the existing canonical
attempt failed in the sandbox, and no full-module race result is available.
This manifest is not a PASS record or release authorization.

## Proposed checkpoint allowlist

The checkpoint should contain exactly these six private primitive files, two
status handoff files, and the scoped evidence paths below. Root must add its
own final acceptance record after direct verification before committing.

### Primitive source and focused fixtures

1. `apps/godspeed-casework-go/internal/server/run_observation_key.go` — the
   extracted private key primitive.
2. `apps/godspeed-casework-go/internal/server/run_observation_retained_version.go`
   — retained-version helper implementation.
3. `apps/godspeed-casework-go/internal/server/run_observation_retained_version_test.go`
   — helper boundary and terminal-state fixtures.
4. `apps/godspeed-casework-go/internal/server/run_observation_retained_policy_test.go`
   — policy fixture with formatting-only repair.
5. `apps/godspeed-casework-go/internal/server/run_observation_poller_image.go`
   — pure private canonical image encoder.
6. `apps/godspeed-casework-go/internal/server/run_observation_manager_retained_image_test.go`
   — independent canonical byte and cap fixtures.

### Handoff state

7. `.agents/CURRENT_STATUS.md`
8. `.agents/current_status.yml`

These must be updated by the checkpoint owner to match the final verified
state. This proposal does not update either status file.

### Supporting evidence allowlist

All paths are under
`.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/`.

**Source identity and independent review**

- `run-observation-primitives-snapshot-copy-result-oct07.md`
- `run-observation-key-extraction-result-oct07.md`
- `run-observation-key-extraction-independent-source-review-oct07.md`
- `run-observation-retained-version-helper-independent-source-review-oct07.md`
- `run-observation-retained-version-terminal-fixture-independent-source-review-oct07.md`
- `run-observation-poller-image-testfirst-source-independent-review-oct07.md`
- `run-observation-poller-image-exact-oracle-source-independent-review-oct07.md`
- `run-observation-poller-image-green-independent-acceptance-oct07.md`

These identify the isolated source set and final focused reviews/acceptance.
Focused helper and encoder evidence remains limited to those tests; it does not
establish manager lifecycle behavior.

**Canonical retry and full-module race**

- `run-observation-primitives-canonical-retry-fullrace-assignment-oct07.md`
- `run-observation-primitives-canonical-retry-fullrace-result-oct07.md`
- `run-observation-primitives-canonical-retry-preflight-oct07.raw`
- `run-observation-primitives-canonical-retry-preflight-exit-oct07.raw`
- `run-observation-primitives-canonical-retry-output-oct07.raw`
- `run-observation-primitives-canonical-retry-exit-oct07.raw`
- `run-observation-primitives-fullrace-preflight-oct07.raw`
- `run-observation-primitives-fullrace-preflight-exit-oct07.raw`
- `run-observation-primitives-fullrace-output-oct07.raw`
- `run-observation-primitives-fullrace-exit-oct07.raw`
- `run-observation-primitives-canonical-fullrace-final-acceptance-oct07.md` —
  root to author only after direct gate review; conditional future path.

The future final acceptance must report each gate's exact outcome and any
limitations. Preserve the older formatting and sandbox failures on disk, but
do not add their intermediate assignments/capture sets to this checkpoint.
The full-module race capture set must not be presented as passing until its
actual exit and independent review confirm success.

## Explicit exclusions

Do not include manager, poller worker, lifecycle, public API, or lifecycle
fixture drafts in this primitive checkpoint. In particular exclude
`apps/godspeed-casework-go/internal/server/run_observation_manager.go`, its
manager fixture, any proposed worker/read-start test or lifecycle scaffold,
and all manager/lifecycle/public design proposals and assignments. The six
source files above are the complete source boundary for this checkpoint.

Also exclude these 11 unrelated staged paths from this checkpoint:

- `.agents/AGENTS.md`
- `.agents/DEBT.md` — preserve both its existing staged blob and separate
  worktree changes.
- `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/observation-safe-trace-port-proposal-independent-review-anchor-clarification.md`
- `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/observation-safe-trace-port-proposal-independent-review.md`
- `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/observation-safe-trace-port-root-proposal.md`
- `.agents/plans/sea-forge-trusted-daemon-target-plan_v0.1.0.yml`
- `.agents/specs/sea-forge-trusted-daemon-target-spec_v0.1.0.yml`
- `.jolli/jollimemory/debug.log`
- `.jolli/jollimemory/discovery-cursors.json`
- `.jolli/jollimemory/plans.json`
- `.jolli/jollimemory/sessions.json`

These paths remain outside the proposed checkpoint; their existing stage-0
index blobs must be preserved. Do not include other public drafts or
`.jolli/jollimemory/*` files by directory expansion.

## Future staging and commit handling

Only after independent canonical/full-module race acceptance and root's final
acceptance record should the checkpoint owner stage the exact allowlist above.
Use normal repository hooks and a path-limited commit such as
`git commit --only -- <exact allowlist>` so unrelated staged entries remain
outside the commit. Before and after, compare the 11 foreign stage-0 index
blob IDs; do not use broad `git add`, `git add -A`, or a whole-index commit.
This is a future handling recommendation, not an action performed here.

No tests, compilers, formatters, scanners, builds, or Git mutations were run
for this manifest proposal.
