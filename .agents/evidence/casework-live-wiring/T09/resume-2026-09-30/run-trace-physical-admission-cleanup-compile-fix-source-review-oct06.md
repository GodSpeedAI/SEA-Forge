# Physical run.get admission — cleanup compile-fix source review

Date: 2026-10-06  
Verdict: **APPROVE this exact test-only compile fix for one separately owned focused assertion-RED attempt.**  
Scope: source-only. This does not assert that compilation succeeds, that any test passes/fails, or that production behavior is approved.

## Frozen source identities

| Path | SHA-256 |
|---|---|
| `apps/godspeed-casework-go/internal/adapters/sfwp/client.go` | `d2f993d7dda3a5d63a414ef5a64d6f995c04de15f94537374940dc6c031027c6` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_get_admission.go` | `f661bd8f8d9046d6eac29738d8521a48ca964361db2e107151cdedce9a0d62a7` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_get_admission_test.go` | `dae0406d503e16546b24c507b02db63489dd8b06cd381b34fd045326d1983d58` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/client_run_get_admission_test.go` | `3466befd626c75785c03ce90c659351ce470f5499ecf69ee5fb8bd18aadcf6cb` |

The only changed test file is `run_get_admission_test.go`; the client fixture, client Config-only field, and typed-unavailable stub retain their reviewed identities. The cleanup registration now passes the test handle with `t.Cleanup(func() { w.cleanup(t) })` at line 238, and `cleanup` receives `t *testing.T` at line 274. The existing cancel, already-joined short circuit, one-second bounded result join, and error report body remain intact. This directly fixes the prior compile error (`undefined: t` at the cleanup callback) without changing fixture assertions or behavior.

## Review disposition

The previous assertion-red attempt remains a compile/setup failure and remains immutable; it is not replaced or reinterpreted. This repair is within the explicitly prescribed two-line scope and does not touch the limiter algorithm, client hooks, other fixture file, production code, or verification gates. It is sufficient to proceed to the assigned single focused compile/test attempt.

No compiler or test was run in this source review. The next attempt must retain the previously assigned focused command, memory limits, fresh preflight, exact raw/exit captures, and actual process join. A compiler error, sandbox socket failure, timeout, or race/setup failure is not expected assertion RED. No production implementation, full package/module gate, or T09 settlement is authorized by this review.

Graft retrieval saved approximately 26,859 tokens (~$0.02) this review turn.
