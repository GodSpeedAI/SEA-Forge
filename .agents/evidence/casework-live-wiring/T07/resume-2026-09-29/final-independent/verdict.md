# T07 independent review — no approval yet

## Decision

**Insufficient runtime evidence; T07 is not approved.** Static inspection and source gates support the session-bound retained-read repair, but the requested independent live gateway proof did not run. Auto-review rejected the real-binary gateway startup before execution; see `teeth/01-gateway-start-auto-review.txt`. Do not treat a successful build or unit tests as runtime proof.

## Evidence-backed results

- Fresh gateway build from the reviewed source snapshot: EXIT 0 (`gates/04-gateway-build.txt`). It was built before the stale perspective documentation comment was corrected; that edit changed comments only. No gateway process was started.
- After session-read test setup repair: focused server/auth race gate EXIT 0 (`gates/05-postrepair-focused-go-race.log`); Go vet EXIT 0 (`gates/07-postrepair-go-vet.log`); one full Go race characterization initially failed only at `TestVerifyRejectsTamperedHash` (`gates/06-postrepair-global-go-race.log`), then a single retry passed (`gates/08-global-go-race-retry.log`).
- The Argon2 test-only repair was independently reviewed. Its focused test passed 50 race-enabled repetitions (`gates/10-argon2-tamper-race-count50.log`), the final full Go race gate passed (`gates/11-final-global-go-race.log`), and final Go vet passed (`gates/12-final-go-vet.log`).
- The required four-crate Rust command `CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 cargo test --locked -p sea-forge-server -p sea-forge-planner -p sea-forge-case-runner -p sea-forge-cli` passed: 594 passed, 0 failed, 4 ignored (`gates/09-four-crate-cargo-test.log`).
- Fresh focused tests verify R-SO and operator historical/SSE actor and role offers, cursor state retention in a controlled `capturingFactsSource`, query overrides, cross-case cursor rejection, mocked revocation, legacy perspective refusal, and deep-copy behavior. The previous live T07 teeth cover CSRF no-write and mutation actor overwrite against a temporary real kernel. They do not prove the new retained-history/SSE behavior against a real kernel.

## Source review

The implementation adds internal, JSON-omitted captured `CaseFacts` to bounded revisions and deep-copies them on store writes and reads. The relay builds and retains a revision from the same `Facts` capture. Historical and SSE handlers reject actor/role overrides before cached access, verify the session actor against the kernel, enforce case/cursor binding, and rebuild role-filtered snapshots from captured facts without refetching current state. Legacy revisions only serve an exactly matching actor and role. I found no production code defect in these repair paths by static review.

The builder's session-test setup fix was limited to test sources: it corrects helper URL construction and aligns the legacy resync fixture's relay actor with the test session actor. `ServeSection` perspective comments now describe the captured-facts behavior. The bounded Argon2 test fix changes only a test: it verifies the source digest, mutates decoded digest bytes, canonically re-encodes, confirms the decoded value differs, and asserts rejection. No behavior, security policy, dependency, persisted schema, or external contract was changed by these review repairs.

## Remaining proof needed

Run the fresh binary on the isolated temporary cell only after the coordinator receives user guidance on the auto-review rejection. The probe plan is `teeth/02-runtime-probe-plan.md`; required checks include distinct authenticated operator/R-SO sessions, genuine old state after mutation, historical/replayed/live SSE per-session action offers, cross-case and override refusal, real invalid/revoked delegation refusal, and security no-write/actor/log/origin behavior. T08 global UI gates and the local ladder are outside this critic's token and remain pending the T08 owner.
