# UI order Phase1 fixture repair — fresh original assignment

Date: 2026-10-06. Fresh Luna builder local_cursor_fixture_fresh_repair,
different from original fixture author live_cursor_v4_boundary_recon.

Read complete original local-cursor-order-phase1 assignment, accepted design
revision4/review/erratum, full testc41a5f37 and independent review022e8401.
Edit ONLY existing NEW localAdapter.cursorOrder.test.ts. Keep conformance
edf8ed69 EXACT and localAdapter production unchanged. No extra source/API/hooks.

Repair every finding while preserving all focused unit obligations:

- Drive intercepted queryTemporalTrajectory timer BEFORE awaiting its promise.
  All awaited adapter reads/dispatch must receive their expected controlled
  timer advancement; no test may hang as a substitute for semantic RED.
- Include progress in ALL ordinary event uniqueness and strict numeric cursor
  ordering; require settlement side event strictly AFTER settlement snapshot,
  never equal, and subsequent snapshot strictly after all earlier events.
  Preserve public dispatch coverage and visible progress/settlement assertions.
- Both omitted/supplied floor cases must exhaust the bounded eligible delivery
  work needed for both subscribers, without assuming one global drain timer.
  No error for valid omitted input, returned unsubscribe, distinct H+1/H+2 proof.
- Manual timer scheduling/advancement/drain has explicit finite limits and clear
  failure on runaway work. Respect relative due time and insertion order for
  equal deadlines; allow intentional advancement through execution phases.
  Always restore global timers, including failed assertions. No sleeps/retries.
- Assert case allocator independence through exact before/after cursors for
  both cases, not only routed events or truthiness; retain queued disposal.
- Immutable old snapshots/trajectory and exact non-snapshot cursor lookup must
  be exercised after the required event. Identify any assertion stopped early
  by the honest pre-fix subject; no claim of runtime RED without execution.

Keep whitebox usage bounded to verified existing append/progress signatures
and preserve real public execution. Parser/FIFO64/ceiling fixture obligations
remain separate held units, not proved by this focused file. No production
algorithm, existing test weakening or public contract change.

Native apply_patch only; no tests/compiler/scanner/Graft build/Git/network,
docs/status/debt or other file edits. Root heavy idle/reserved; RAM/swap may
not meet the margin. Return new test hash, preserved conformance/source hashes,
every material deviation and expected semantic RED boundaries. Different
critic gets both originals/full result and review before any actual RED grant.
