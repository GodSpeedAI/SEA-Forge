# Bounded response-cap retry fixture repair

Date: 2026-10-05. Cancellation approval and trace source release remain held.

## Evidence and decision

Fresh serialized broader race gates fail two response-cap tests. The exact
baseline/current diagnostic reproduces deadline failures with original client
8cdfc52a as well as current e0d3c12c, using identical settings and fixtures.
This proves the cancellation callback is not required for those failures;
it does not quantify any callback contribution to their frequency.

The tests distinguish overflow retry and mutation recovery, while an unchanged
dedicated fragmented fixture already tests the full default 32 MiB boundary
and overflow poisoning (`response_limit_test.go:199-227`). The approved contract
permits a lower configured cap and explicitly calls for small-cap retry tests.
Use that existing configuration to make the retry/recovery fixtures bounded.
Keep every deadline, runtime rule, default cap and behavioral assertion intact.

## Builder scope

Read the entire target and nearby lower-cap tests before editing. Change ONLY
`apps/godspeed-casework-go/internal/adapters/sfwp/response_limit_test.go`, within
`TestInspectOverLimitResponseRetriesOnceOnFreshConnection` and
`TestOverLimitMutationAndRecoveryResponsesNeverResendMutation`.

In each test, use an explicit local 1024-byte response-line cap, construct the
oversized invalid-JSON response relative to that cap, set
`config.MaxResponseLineBytes` on the existing testConfig before New, and retain
all exact request/connection counts and recovered real outcome assertions.
The small valid recovery response must fit the cap. A short comment may state
that full default-boundary coverage stays in its dedicated fixture.

Do not change testConfig, timeouts, recovery budgets, production code, default
32 MiB ceiling, fragmented boundary fixtures, cancellations, subscriptions,
skip/lint settings, dependencies, status, debt or Git. No assertion removal or
weaker request/connection checks. Native apply_patch only. Format the target
without compiling. Produce a NEW builder evidence note with exact diff/hash
and all material differences from the original fixture. No compiler permission.

## Independent review and gates

An independent critic receives these original instructions and actual diff.
Verify scope, unchanged complete default-cap coverage, unchanged timeout and
recovery assertions, invalid over-cap response classification before decode,
and valid recovery-body size. No source approval without exact evidence.
Then request the sole compiler token for focused repeated race, full SFWP race,
canonical Go and fresh full-module race gates, with preflights and exact logs.
Do not approve cancellation until every required gate passes independently.
