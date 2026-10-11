# Independent review: safe trace bounded production repair

Date: 2026-10-05. Verdict: **REJECT pending complete RFC3339 syntax validation.**
This is a source-only review. It grants no runtime approval.

## Review envelope and identities

Reviewed the original production assignment, approved V3 proposal, empty-slice
erratum, prior source rejection, bounded-repair builder record, and the complete
repair diff against the previously rejected implementation. The original
assignment, erratum, and V3 hashes match the builder record:

| Evidence/source | SHA-256 |
|---|---|
| `safe-trace-production-root-assignment-oct05.md` | `31faeb43d160ffc2bf32f463e771fc17afc4693aff735c3ea6360853c00f0681` |
| `safe-trace-production-assignment-empty-slice-erratum-oct05.md` | `3e672aaf4c06c166823b394622cb1a8946506439020863080943b26835a87b9d` |
| `observation-safe-trace-port-root-proposal-v3.md` | `e280649130163c99cdc52f8d208e5eb64fd0c253a686130823d82f3a807085d6` |
| Prior independent rejection | `18136b53983f4f91f1df1f6533ea199b5d27c2f2bdb75cd606cc0e9c587deb14` |
| Bounded-repair builder record | `d8a073d3ec51385ae8e592fa84708b220e6416aa17f0b0aaa97ef6d5554d83df` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace.go` | `efb70d16a98e087e13149fbcf2fa4a7429603ed07614fd4436ba3d7436341d20` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace_test.go` | `5ccc705112ea02baf9c26f349376e5e97b95cb86c6b83c7a44988a18fc72dae1` |

The production assignment confines implementation edits to `run_trace.go`,
requires exactly one logical `Client.Do(NewRunGet(runID))`, and requires all
selected rows to be validated while retaining at most the newest 1024 in source
order. Selected duplicate IDs must be checked across the full returned array.
V3 requires selected timestamps to be RFC3339/RFC3339Nano. The approved
empty-slice erratum permits nil or empty frames for successful empty results;
errors still return the exact zero snapshot.

## Bounded-retention repair: accepted by source inspection

The repaired implementation allocates frame storage with capacity at most
`runTraceRetention` (`run_trace.go:50-56`). It continues traversing all rows
and checking selected IDs (`:60-93`), replacing the oldest retained slot after
the ring fills (`:104-110`). When more than 1024 rows were selected, it copies
the newest frames in original order into a fresh, exact-size allocation
(`:113-117`). The returned count remains the number of validated selected rows
before retention (`:110, :122`). This closes the prior backing-array retention
finding by source inspection.

The builder’s complete diff adds a comma-fraction rejection case and checks
length/capacity for the 1027-row retained result. It does not remove or weaken
the existing fixture assertions. These observations approve only the bounded
retention repair and fixture diff; they do not approve the adapter as a whole.

## Blocking finding: comma-only rejection does not establish RFC3339 syntax

The implementation rejects commas (`run_trace.go:83-86`) and then calls
`time.Parse(time.RFC3339Nano, timestamp)` (`:87`). That handles the previously
identified comma fraction but still delegates syntax validation to Go's
permissive layout fallback.

The pinned Go 1.27.1 source at
`/home/sprime01/.local/share/mise/installs/go/1.27.1/src/time/format.go:1026-1033`
shows `Parse` tries the optimized RFC3339 parser and falls back to generic
layout parsing when that fast parser rejects the input. The generic parser
exists because its layout machinery does not enforce all RFC3339 width/range
constraints. In `format_rfc3339.go:163-181`, the standard library documents
that strict parsing is disabled (`case true`) and retains checks for a
one-digit hour, timezone hour outside `[0,23]`, and timezone minute outside
`[0,59]`. The comma check appears among those distinct checks at `:174-175`.

The new `strings.Contains(timestamp, ",")` guard rejects only the comma class.
It does not reject other inputs rejected by the RFC3339 fast parser but
accepted by the generic fallback, such as a one-digit hour or an out-of-range
numeric offset. Consequently the implementation still does not establish the
RFC3339/RFC3339Nano requirement from V3. A complete lexical/semantic grammar
check (or equivalent strict parser) is required before runtime approval. The
fixture should cover fallback classes beyond comma, including hour width and
timezone offset bounds, while retaining valid RFC3339/RFC3339Nano examples.

## Verdict and limits

The bounded-ring behavior and the non-weakened test diff are source-approved.
Strict timestamp validation remains a blocking defect, so the frozen adapter
is **REJECTED** and receives no compiler/runtime approval. No source or fixture
files were changed by this review. No compile, test, scanner, Git, status, or
debt command was run. Runtime behavior remains unverified.
