# Safe-trace fixture error-zero builder record

Date: 2026-10-05. This immutable record captures the bounded fixture edit
authorized by the root coordinator after review of the existing V3 fixture.

## Scope and change

File: `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace_test.go`

In `TestReadRunTraceV3TracePresenceAndReturnedRowBoundary` and
`TestReadRunTraceV3CommandMetadataProjection`, immediately after constructing
the expected successful snapshot, error rows (`tc.wantKind != ""`) now replace
that expectation with `ports.RunTraceSnapshot{}`. Successful rows retain their
existing exact snapshot expectations.

Equivalent added code in each table loop:

```go
if tc.wantKind != "" {
    want = ports.RunTraceSnapshot{}
}
```

SHA-256 of the resulting fixture file:
`e839346535b3b804326da3f03e2f99df909739528eea591046cbb2f3f5fd522e`.

## Bound and deviations

The root assignment allowed only these two test table expected-value repairs;
the root independently identified both affected loops. No production code,
other fixture, representation convention, schema, runtime behavior, test
command, compiler, scanner, Git operation, hook, status, or debt file was
changed. In particular, this edit makes no assertion about nil versus
non-nil zero-length frame slices on successful empty results. No verification
was run, as instructed. The two conditional zero expectations are the entire
change.
