# Case inventory and range-frontier race source recon

Date: 2026-10-06  
Scope: read-only source analysis. Filesystem concurrency and atomicity are not proven by this review.

## Case creation ordering

`case.commit` reaches `case_dispatch::submit` (`crates/sea-forge-server/src/lib.rs:2527-2539`). In its blocking mint closure, it records the correlation locator, creates the case directory/runs subdirectory (`case_dispatch.rs:77-84`; `sea-forge-case-runner/src/lib.rs:20-25`), writes `case.json` and `plan.json` (`case_dispatch.rs:97-115`), then appends `TraceKind::CaseCreated` (`:116-124`). `CaseRunner::append_event` commits that trace to the per-case `case-{id}` ledger before appending/flushing `case-events.jsonl` (`sea-forge-case-runner/src/lib.rs:31-72,84-101`). This is not the global `events.get_range` ledger.

For a successful dispatch, `commit_plan` receives the outcome only after `case_dispatch::submit` returns, inserts its in-memory `CaseEntry`, then publishes global `case.submitted` (`crates/sea-forge-server/src/lib.rs:2788-2822`). Thus, for a successful commit, the durable directory and case record precede its global `case.submitted` cursor. There is no global `case.created` frame in this path. A later range frontier F that includes that commit's `case.submitted` cannot represent a creation that this commit has not yet materialized; the specific create-after-F race is closed by source ordering, assuming successful filesystem operations and ordinary visibility. Dispatch can return an error after minting/partial work, in which case no successful `case.submitted` frame is guaranteed.

## What `case.list` can and cannot establish

`case.list` does one `read_dir(cases)` pass with no request filter or page token (`sfwp/case_views.rs:310-355`). An absent cases directory returns an empty default. The iterator uses `flatten`, ignores non-directories and non-UTF-8 names, and reads each directory's `case.json` (`:314-326`). A missing/unstatable record is `NotFound` and omitted; malformed or oversized JSON is `Unreadable` and placed in a separate list, not `cases` (`:253-271,336-341`). A read-dir open failure also returns the same empty default; individual iterator errors are skipped.

The listing is not locked with case creation or updates and is not documented as a filesystem snapshot. `write_json` uses direct `fs::write` (`sea-forge-case-runner/src/lib.rs:570-575`), not temp-file/rename publication. Concurrent case-state writes can therefore race a list read; the source does not prove atomic file visibility, stable directory iteration, or that all records readable before/after a scan remain so throughout it. A second list after F is useful reconciliation evidence, but cannot by itself prove complete inventory under filesystem failures/concurrent mutation. For a previously successful case, an update race can move its record into `unreadable` or a concurrent external disappearance can make it omitted; source does not establish external mutation behavior.

## Range cursor semantics

`EventsGetRange` has serde-defaulted optional `from_cursor`, `to_cursor`, and `limit` (`lib.rs:773-781`). Missing endpoints mean unbounded start/end. A supplied endpoint, including an empty string, is looked up by exact entry ULID; unknown values return typed input error (`sfwp/events.rs:110-129,175-189`). The lookup accepts any entry in this events ledger; returned frames are then filtered to `record_kind == sfwp_event`. Range semantics are exclusive after `from` and inclusive through `to` by append ordinal; omitted limit defaults to 500 and supplied limit clamps at 500 (`events.rs:165-168,190-207`). A short/empty page only states that fewer matching frames were returned by that call; the response contains only `{events:[...]}` and no explicit `done` or frontier (`lib.rs:2349-2361`).

## Conclusion

Source ordering refutes a successful-commit case materialization that occurs after its `case.submitted` frontier. It does not make `case.list` an atomic completeness proof; unreadable/omitted records and concurrent filesystem changes remain distinct failure modes. Also, the local `CaseCreated` trace is not itself addressable through the global SFWP event cursor frontier.
