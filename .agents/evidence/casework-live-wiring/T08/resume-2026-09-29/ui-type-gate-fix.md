# T08 UI type-gate repair (2026-09-29)

## Original bounded assignment

Repair the independent round-5 typecheck findings in exactly these files:
`apps/godspeed-cognitive-ui/src/app/intents.ts`,
`src/app/live.test.ts`, `src/adapters/http/httpCaseworkAdapter.test.ts`, and
`src/adapters/local/localAdapter.test.ts`. The findings were: stale-refresh
actor role widened to `string`; three partial live test ports passed where the
full `CaseworkPort` is required; an artifact fixture widened `content_type` to
`string`; and two calls supplied an error callback to the local adapter's
three-argument `subscribeEvents`. Reuse the prepared canonical intent actor,
make honest complete live test ports whose unused methods fail if called,
preserve fixture typing, and match the local adapter signature. Do not edit
production contract types or `localAdapter.ts`, weaken/delete tests, or edit the
other builder's `httpCaseworkAdapter.ts`, `httpTrajectory.test.ts`,
`httpCaseworkAdapter.nativeEvents.test.ts`. Source-only: no tests, vet, build,
typecheck, Bun, Go, or Cargo commands. Record this note and run `git diff
--check` only.

## Changes

- `intents.ts`: stale-projection refresh uses the exact prepared
  `intent.actor.actor_id` and canonical `intent.actor.role`. The case-ID guard,
  one refresh, refusal result, and no-replay behavior remain in place.
- `live.test.ts`: replaced each partial `Pick` port with a fully typed
  `CaseworkPort`; every method outside `subscribeEvents` throws a descriptive
  unexpected-call error. The three tests retain their event/error handlers,
  recovery, history, and disposal assertions.
- `httpCaseworkAdapter.test.ts`: preserved the `text/markdown` content-type
  literal in the artifact fixture.
- `localAdapter.test.ts`: both direct adapter subscriptions now use its
  declared three-argument signature; event collection and assertions remain.

## Material deviations

None. No production contract or local adapter implementation changed; no test
was removed or disabled. The only additional file is this requested evidence
note. No dependencies were added.

## Verification and pending commands

- `git diff --check` was run and returned exit 0.
- No test, vet, build, typecheck, Bun, Go, or Cargo command was run, per the
  compile-token restriction.
- Pending after the independent compile token is released: from
  `apps/godspeed-cognitive-ui`, run `bun run typecheck` and the focused review
  set:
  `bun test src/adapters/http/httpTrajectory.test.ts src/adapters/http/httpCaseworkAdapter.nativeEvents.test.ts src/app/intents.test.ts src/app/live.test.ts src/adapters/local/localAdapter.test.ts`.
  The milestone UI gate `just casework-ui-check` remains pending at the task
  level.
