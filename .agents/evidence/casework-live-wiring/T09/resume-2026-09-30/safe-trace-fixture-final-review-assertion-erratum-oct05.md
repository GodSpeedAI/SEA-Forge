# Safe-trace fixture final review assertion erratum

Date: 2026-10-05. This immutable erratum supplements
`safe-trace-fixture-final-independent-review-oct05.md`; it does not edit that
review or the frozen source.

The final review's blocking assertion finding also applies to
`TestReadRunTraceV3TracePresenceAndReturnedRowBoundary` at
`apps/godspeed-casework-go/internal/adapters/sfwp/run_trace_test.go:473-479`.
That table unconditionally constructs and passes `validRunTraceSnapshot` for
all rows, including cases with `tc.wantKind == apperr.KindUnavailable`. For
those malformed trace or records cases, the correct postcondition is an exact
zero snapshot. The nonempty identity/standing snapshot supplied as `want`
causes the stub's correct unavailable-plus-zero result to fail the common
helper's exact snapshot comparison. This is the second independent table
family with the same test-fixture defect; the required repair is to use the
exact zero snapshot for error cases while preserving successful-result
expectations and every behavior assertion.

The final review's count of 35 snapshot mismatch messages covers both the
trace-presence/records and command-metadata error tables, not metadata alone.
The underlying rejection is unchanged and strengthened. This erratum makes no
claim that a nil versus non-nil zero-length frame slice distinguishes a valid
successful empty trace; V3 permits zero returned rows and does not prescribe
that Go slice representation. Failed reads must still return the exact zero
snapshot.
