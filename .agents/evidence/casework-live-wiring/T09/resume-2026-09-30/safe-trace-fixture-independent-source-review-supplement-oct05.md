# Safe-trace fixture independent review supplement

Date: 2026-10-05. This supplement is a new immutable record for two additional source checks requested after the original review. It does not alter the original verdict or artifact.

## Payload-absence case is not actually absent

`apps/godspeed-casework-go/internal/adapters/sfwp/run_trace_test.go:171-179` defines `runTraceRow`; its `if payload == ""` branch rewrites the value to `{}` before formatting it into the row. The command-metadata table at lines 417-449 labels one entry `missing payload` and gives it the empty string. The loop at lines 450-457 passes that empty string to `runTraceRow`, so the actual JSON is `"payload":{}`. This case tests an empty object, not a missing payload field. Consequently the required raw-absent versus null handling distinction is not covered by this named case. `runTraceResponse` separately has a trace/records omission mechanism, but no analogous payload omission path is used here.

## Unknown-kind malformed-ID coverage

At `run_trace_test.go:301-305`, the one unknown-kind row has a string `event_id` (`evt_selected`), a future string kind, boolean timestamp, and object payload containing a marker. The adjacent selected row uses the same string ID. This proves the accepted selected/unknown string-ID collision and ignored malformed timestamp/payload. It does **not** cover an unknown row whose event ID is missing, null, or the wrong JSON type. The root assignment asks to omit unknown string kinds before validating their event IDs, timestamps, or payload fields (`safe-trace-test-first-root-assignment-oct05.md:46-48`), so malformed/missing unknown-ID shapes remain uncovered.

## Disposition

These are additional matrix gaps and reinforce the original `REJECT` verdict in `safe-trace-fixture-independent-source-review-oct05.md`. No source, test, status, debt, or Git files were changed. No compiler or tests were run.
