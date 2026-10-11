# Physical run.get admission — retry-counter source review

Date: 2026-10-06  
Verdict: **APPROVE one focused attempt-4 assertion-RED run.**  
Scope: source-only review of the bounded test-fixture race repair. This is not a compile/test result or production approval.

## Frozen source identities

| Path | SHA-256 |
|---|---|
| `apps/godspeed-casework-go/internal/adapters/sfwp/client.go` | `d2f993d7dda3a5d63a414ef5a64d6f995c04de15f94537374940dc6c031027c6` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_get_admission.go` | `f661bd8f8d9046d6eac29738d8521a48ca964361db2e107151cdedce9a0d62a7` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_get_admission_test.go` | `dae0406d503e16546b24c507b02db63489dd8b06cd381b34fd045326d1983d58` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/client_run_get_admission_test.go` | `becd4266dca4a28e7641f5eaab1dafe5dc932e04723895cf5e795e7671524103` |

Only the client fixture changed from the prior reviewed source. It imports `sync/atomic`, changes `calls` to `atomic.Int32`, and uses `calls.Add(1)` as the switch value in `TestClientAdmissionCountsAllFourRunGetRetryAttempts` (`client_run_get_admission_test.go:11,87–93`). This removes the unsynchronized shared increment/read reported by attempt 3 while retaining the response sequence: first and third requests cause safe transport failures, second returns the explicit busy refusal, fourth succeeds. The physical request-count and four-admission assertions remain unchanged (`:111–117`). The other three source hashes are unchanged. No source hooks, limiter algorithm, fixture assertion, harness, or gate were modified.

The earlier failures were reviewed and remain preserved: attempt 1 was a compile error in waiter cleanup; attempt 2 had sandboxed Unix-socket setup failures mixed with expected assertion failures; attempt 3 reached fixtures under the narrowly approved local-fixture escalation and found the retry counter race described above. None is being represented as passing or as actual assertion RED.

This exact atomic-only correction is sufficient to authorize one new attempt with the previously assigned nine-function regex and limits, adding only `-v` for explicit top-level reachability. Capture a fresh preflight and formatter result, use unique attempt-4 files, run only after this review, and join the actual process before disposition. Any race, compile/type/setup failure, timeout, or unexpected assertion is rejection. Expected typed-stub and missing-hook failures are acceptable RED, with the known no-spin/pool observations potentially unreached because the stub/absent hook fails earlier. No production behavior or T09 settlement is approved.

Graft saved approximately 4,182 tokens (<$0.01) this source-review turn.
