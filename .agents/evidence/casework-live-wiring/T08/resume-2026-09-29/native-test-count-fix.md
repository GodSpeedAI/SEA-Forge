# T08 native retry test count repair — source-only builder

## Original assignment and regression contract

Fresh bounded T08 builder after the round-6 static rejection. Modify only
`apps/godspeed-cognitive-ui/src/adapters/http/httpCaseworkAdapter.nativeEvents.test.ts`;
no tests, Bun, vet, build, or compile commands because `t07_final_independent`
owns the compile token. Preserve coverage for native capped-backoff retries and
resume cursor, open/drop retry growth, malformed and foreign frames not poisoning
scope/cursor, `resync_required` followed by a same-ID snapshot, and disposal of
pending timers. Do not change production stream code or other contracts/dependencies.
Record this result in this evidence note and use only `git diff --check` verification.

## Changes

- Count the initial EventSource plus three completed timer-driven retries as four.
  Capture that count before the final failed source schedules its retry; assert the
  count is unchanged before that timer fires and grows by exactly one after it.
  Assert the final retry URL is exactly
  `http://gw.test/api/events?last=01M2`.
- Replace the direct plain-async-function fetch assignment with a function typed
  from `Parameters<typeof fetch>` / `ReturnType<typeof fetch>`, wrapped in
  `Object.assign` with the original Bun `fetch.preconnect` property. The mock still
  increments `fetchCount` and rejects if native EventSource attempts fetch fallback.
- All named stream regressions remain present and unchanged. No production source,
  contracts, dependencies, or other source files changed.

## Verification and pending gates

Ran `git diff --check -- <source-file> <this-note>`: PASS. No Bun, typecheck,
vet, build, or compile command was run per assignment; no runtime result is claimed.
The previously recorded focused Bun suite, UI typecheck, and T08 gates remain
pending fresh independent verification. F14 live native EventSource resync/reconnect
proof and shared live port conformance also remain pending.

## Material deviations

None from the source-file/evidence-only scope or the requested verification limit.
