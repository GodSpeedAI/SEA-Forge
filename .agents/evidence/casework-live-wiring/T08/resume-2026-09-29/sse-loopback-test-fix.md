# T08 SSE failure-path listener loopback correction

Date: 2026-09-30

## Original condition and scope

The TOOTH test `SSE drop flips the store to reconnecting via the live adapter error path, and an event resumes it` creates a temporary HTTP 503 gateway with `Bun.serve({ port: 0, fetch: ... })`, then connects the real `HttpCaseworkAdapter` to `http://127.0.0.1:${broken.port}`. The test was reported to fail in the full Bun suite with `EPERM`: the server's unspecified hostname bound beyond loopback, which exceeds the allowed network scope in this environment.

Assignment: change only `apps/godspeed-cognitive-ui/src/ui/journeys.test.tsx` and this evidence note. Keep real network use, ephemeral port allocation, the actual HTTP 503 response, reconnect/backoff behavior, later resume assertions, and cleanup. No mocks, skipped tests, hidden failures, or sandbox changes. The independent critic owns Bun verification.

## Material diff

- Original listener at the TOOTH setup: `Bun.serve({ port: 0, fetch: () => new Response('gateway broken', { status: 503 }) })`.
- Changed it to specify `hostname: '127.0.0.1'` explicitly while retaining `port: 0` and the same fetch handler returning status 503.
- The adapter still targets the listener's actual ephemeral port through `http://127.0.0.1:${broken.port}`. The reconnecting-state assertions, later resumed-event assertions, and `broken.stop(true)` cleanup are unchanged.

## Result and verification status

The test listener now requests an explicit loopback bind, matching the adapter's loopback URL while retaining the real HTTP failure and recovery path. This addresses the reported bind-scope mismatch; it does not establish that the test or suite passes.

No Bun test, typecheck, build, vet, or compile command was run, per assignment. `git diff --check` is the only permitted verification in this builder turn. Independent critic Bun verification remains pending; T08 has no new pass claim from this change.

## Deviations

None. Only the named source file and this evidence note were changed.
