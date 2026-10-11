# Shared replay conformance fixture tests: independent red review

**Verdict: expected RED against the current common conformance assertion.** The fixture-only
tests exercise the accepted-cursor replay semantics approved in
`shared-replay-expectation-semantic-review.md`. No production or common-conformance source
was changed in this test-first phase.

## Fixture review

The diff adds three tests in `src/adapters/local/localAdapter.test.ts` and a test helper
that proxies a fresh, real `LocalContractAdapter`. The first subscription is passed through
to that adapter; only the second (replay) subscription is controlled by the fixture. The
helper captures the actual accepted cursor from the adapter's real dispatch response and
checks that its synthetic prefix is strictly between the prior cursor and that response.
It emits either an ordered prefix plus the captured receipt, a prefix with no receipt, or
an out-of-order prefix. The fake subscription returns its own no-op unsubscribe, while the
shared suite's first real local subscription is unsubscribed by the suite's existing
`finally` block. Each case runs with its own `beforeEach` adapter. This fixture does not
alter `HttpCaseworkAdapter`, create a fake EventSource, or claim live transport evidence.

The three cases distinguish acceptance of an earlier chronological frame, omission of the
receipt point, and unordered delivery. Their failing run below confirms they reach the
current assertion and fail for the intended missing behavior, not a fixture setup error.

## Command and actual-host checks

Actual-host preflight at 2026-09-30 21:38:09 UTC reported 1,878,516 kB available RAM and
no Go/Cargo/Vite compiler process. Bun PID 13814 was the only matching runtime and was
left untouched.

```sh
bun test src/adapters/local/localAdapter.test.ts -t 'shared conformance'
```

Exit status: **1**, as expected for the red phase. Bun ran exactly these three tests; all
three failed against the current first-frame assertion:

- Ordered prefix plus receipt: failed because the current suite treated the first valid
  prefix cursor (`1.0000000006~retained-prefix`) as a failure instead of continuing to the
  captured receipt cursor (`1.0000000007`).
- Prefix only: the current suite failed immediately on the prefix cursor, so the intended
  timeout assertion failed. This demonstrates that the current assertion does not wait for
  the accepted cursor.
- Unordered prefix: the current suite failed immediately on the first later prefix cursor,
  before it could validate chronological order on the next prefix. This demonstrates the
  missing order check.

Post-run actual-host check at 21:39:07 UTC reported 1,848,868 kB available RAM, no Go/Cargo/
Vite compiler process, and the same unrelated Bun PID. No source was modified during this
review.

## Raw evidence and hashes

- Raw command output with explicit `exit=1`:
  `/tmp/t08-shared-replay-red-20260930.log`
- Raw log SHA-256: `5f38f98c94d3f7ff3a04cbadeea1e9e25f3bc20b86e8b352520d35be845ce025`
- Test fixture source SHA-256:
  `4b4e76d8a9bd5424327d265042d961baba8561d2afac1b2d2bcd48cb8d3a8aef`
- Common conformance source remains at the reviewed pre-fix SHA-256:
  `85e118edc991d175e422069c93a5f1baecdd3b2cb5427bf8695fdb2155abf295`
