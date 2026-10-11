# T09 fixture independent review

## Verdict

APPROVE the repaired canonical contract fixtures and the repaired SFWP response-line-cap fixtures for their test-first RED stage. This approves the fixtures only; it does not approve implementation or claim the client cap is present.

The review used the original assignments in `test-first-builder-assignments.md`, the approved `t09-contract-extension-proposal.md`, and the source-preparation facts in `contract-source-preparation.md`. The canonical fixture stays within the assigned test/golden-only scope. The cap fixture is confined to `response_limit_test.go`; the separately authorized `MaxResponseLineBytes` field is the only implementation-adjacent seam.

## Canonical contract fixtures

The repaired test adds the Ask schema filename to the explicit schema inventory; asserts request and answer keys, exact question/claim/status/disposition/freshness literals, and string assurance/authority notice; and reads one request plus answered, partial, and denied answer goldens. The observation checks pin the cohort, run, and frame property/required sets, count optionality in the TypeScript declaration, count absence on unavailable `run.list`, bounded runs/frames, separate execution/settlement standing, and rejection of extra trace fields and invented progress. It also checks that stream parity retains `interrupted` and `error` and lists the new observation event.

The command-frame status repair matches the source semantics: `ExecutionStatus` in `crates/sea-forge-core/src/types.rs:478-489` serializes five values (`completed`, `spawn_failed`, `timed_out`, `sandbox_violation`, `suspected_sandbox_violation`). `ExecutionStanding` in `crates/sea-forge-server/src/sfwp/case_views.rs:55-70` has six distinct run values; `SettlementStanding` at lines 74-84 is separate. `run_views.rs:556-570` projects command-finished status from the actual trace payload and notes that command completion does not itself set run completion. The fixture pins the five command values separately from the six run standings.

The test does not claim to validate UTF-8 purpose bytes or raw HTTP-body limits; those remain route tests as specified. The SSE JSONL golden is an envelope example and does not prove absence of a native SSE `id:` line; the proposal assigns that proof to server framing tests.

One verification limitation is material and must remain visible during implementation review: `bun test` transpiles this file, while the existing UI `tsconfig.json` includes only `src`, `e2e`, and `vite.config.ts`. Thus the `Expect<Equal<...>>` pins in this `.agents` test are not checked by `bun run typecheck` or `just casework-ui-check`. I ran an explicit TypeScript CLI invocation against the canonical test; before the new contract types exist it exited 2 with missing-export and dependent type diagnostics (output SHA-256 `b916b741fce892c5d3710416f31cf4c6ad0a02a9ab6c30a1300275e0dc3be278`). The attempted durable copy of that compiler output was rejected because the tool-rendered content was truncated, so no raw compiler log is claimed here. After the canonical types are implemented, run the same explicit file type-check and retain its complete result; do not count the existing UI typecheck as exercising these pins.

The focused Bun run used actual RAM/process preflight and ran serially:

```text
bun test ./.agents/reports/interface-contracts/tests/contract-conformance.test.ts
exit 1; 8 pass, 3 fail
```

The expected failures are preserved in `canonical-independent-red.log` (SHA-256 `5c88117f0a943edc5c426501133689ea29232dc37b02584501360672176df34c`). They identify the absent new Ask schema in Criterion 12 and its focused check, plus the not-yet-updated event schema enum. The log bytes were compared to the original captured output. Fixture source SHA-256 is `d38043e5087202c781b4ae07bdaaab4787fba24a02e25f3f715cfc8c165bf751`; SSE golden SHA-256 is `04e06c7b8a905c0bffc4055032d795b697b05ce9be888e472bdd75a950652528`.

Ask golden SHA-256 values: request `30efdb892da29fa21760db57511bac75f85d4116df0f06d4a1461e5ecf295864`; answered `9d90a163f10d3ad066cc0ed0e5911eaa19f633bf6a1f7a9ae3858258d4fbd945`; partial `2d8f2f659b540c5c2842d0080a517c18b39b327ca9973a8b42eab94a19d6d7bc`; denied `2df780254d4332afd6d4885bcf6467009bb7860ad96e7be223a334af32ef52aa`.

## SFWP response-line-cap fixtures

The repaired oversized-subscription fixture first verifies its `cap + 1` line decodes as a real event, then offers 64 KiB of tail bytes in the same pipe write. This fixes the earlier overconstraint: a correct reader may consume the cap plus detection byte before closing, so the writer must still have unconsumed bytes to observe closure. The fixture checks no event delivery, typed `unavailable`, no EOF/deadline substitution, non-backpressure classification, and writer observation of connection close. It bounds idle/pass waits to 30/45 seconds and owns cancellation, client close, and both pipe ends. This removes the earlier 100 ms idle-timeout false-positive path while giving failing runs a finite cleanup route.

The first focused run in the restricted sandbox could not exercise several fake Unix-listener tests because `setsockopt` was denied; its complete output is retained as `cap-repaired-sandbox-limited-attempt.log` (SHA-256 `5dc937c9687a3f33e66e78eb16e0da01dcaa9d15f1387dcd3632fad717ba453a`). I did not treat that run as fixture RED evidence. Before the host run, `free -h` showed 2.3 GiB available and the process scan found no Bun, Node, Go, compiler, Cargo, or TypeScript process. From `apps/godspeed-casework-go`, I reran all eight focused tests with the explicit escalation needed for local Unix sockets:

```text
GOCACHE=/tmp/sea-rs-t09-critic-gocache GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 go test -p=1 -race -count=1 ./internal/adapters/sfwp -run '^(TestResponseLineLimitConfigValidationBeforeDial|TestLowerResponseLineLimitAppliesToRPC|TestLowerResponseLineLimitAppliesToSubscription|TestResponseLineLimitExactBoundaryAndOverflowPoison|TestResponseLineLimitTruncatedEOFIsUnavailable|TestInspectOverLimitResponseRetriesOnceOnFreshConnection|TestOverLimitMutationAndRecoveryResponsesNeverResendMutation|TestSubscribeOverLimitEventIsRejectedBeforeDecode)$'
exit 1
```

All eight test functions reached their assertions. The host run showed the intended baseline failures: invalid config accepted; over-limit RPC response accepted; over-limit subscription event delivered; over-limit response not rejected/poisoned; inspect retry decoded the oversized invalid line instead of retrying; mutation recovery could not resolve after the oversized status response. Exact-limit, truncated-EOF, and lower-limit boundary positive cases passed. The full original output is `cap-repaired-independent-red.log`, SHA-256 `6561396751e2bcb8d9ed138099ee208481c2864b1646587159c435a149c50332`; it compares byte-for-byte with the captured run. The test source SHA-256 is `7ec880bc2a7cdf15eb8c4dfa26daca7d4ea021b7c50940a20e16839cec7b6299`; the current client source SHA-256 is `6d954707db888ae0a08267042f3afd63de9d7bffa76b258d85e6f3a66a071992`.

No dependency, implementation, or unrelated-file change was made by this review. The separate sandbox-restricted attempt and the actual-host RED are preserved distinctly; only the latter supports the cap fixture verdict.
