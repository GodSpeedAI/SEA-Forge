# Physical run.get admission — production assignment

## Release and scope

Root verified all four frozen source hashes and byte-exact attempt4 raw, exit and
preflight copies. Session47180 joined1. Independent attempt4 approval establishes
compiled assertion RED only; no-spin and pool-at-release assertions are unproven.
Production implementation is now released within this bounded assignment.

Builder may edit ONLY `internal/adapters/sfwp/run_get_admission.go` and `client.go`
under `apps/godspeed-casework-go`. Read files and nearby tests before editing;
follow repository/scoped instructions, Graft and Neatcode. Preserve both frozen
fixture files exactly (limiter dae0406d; client becd4266). No test, main, manager,
public wire contract, kernel, dependency, status, ledger or Git edits. No compiling,
scanner or Graft build: independent critic will own compilation after source review.
Use native apply_patch. Return full diff, source hashes and deviations.

The original fixture assignment, root proposal, repair proposal, root adjudication,
wait clarification, fixture root-review supplement and independent attempt4 record
in this directory remain binding. This assignment implements their accepted design;
it does not release manager or SSE/UI work.

## Limiter algorithm and ownership

Implement the existing declared interfaces. Production constructor fixes exactly
two records and one second. Keep timing overrides private. Never add a map/queue.
Under the limiter mutex, check context before admission. A matching run must reuse
its matching eligible idle record, or wait; never use another record for that run.
An absent run uses first empty record, otherwise earliest eligible idle record,
with slot-index tie breaking. Never evict busy or unexpired idle records.

Capture change channel under mutex; unlock before waiting. Matching busy waits on
change/context only; matching cooling waits on its future deadline/change/context.
Absent and blocked waits on earliest FUTURE idle deadline/change/context; all-busy
waits only change/context. Reevaluate under mutex after wake. Stop unused timers.
No expired timer retry loop, default busy loop, polling sleeps or leaked waiters.
Context failures are typed unavailable and preserve errors.Is cancellation/deadline.

Permit mutations and slot eligibility are synchronized. Start immediately before
first payload Write; Finish at return of last attempted payload/LF Write, including
partial/error writes, before response read. Finish sets nextEligible to controlled
clock finish plus cooldown. No write means no cooldown. Release idempotently clears
busy and notifies waiters, after connection ownership cleanup. Do not fabricate an
earlier Finish for an ongoing write. No record may be reused while its permit owns it.

## Client integration and semantic preservation

Guard EVERY physical `run_get` attempt when Config.RunGetAdmission is nonnil,
including safe reconnect and server_busy retries. Other verbs are unguarded.
Acquire permit BEFORE pooled connection acquisition, inside the existing
roundTrip timeout. Every retry starts after the previous helper fully returns.
One physical helper registers permit release FIRST and connection cleanup LATER;
Go LIFO must clean the pool before permit release. Decode response while still
holding the permit. conn.call must join its existing cancellation callback before
the helper cleans the connection and releases the permit.

Keep canonical roundTrip flow; avoid a separate parallel run_get retry policy.
Factor the existing physical operation into a helper with explicit failure phase
so admission/connection-acquire/response-decode errors are never mistaken for
cn.call transport errors. Preserve first acquire errors, safe retry count, second
acquire failure's original-error behavior, decoded refusal handling, deadline error
normalization, correlated mutation recovery, Ask no-resend and response cap.
Mutation recovery and retries must occur only after helper cleanup. New admission
queue failure may return its typed unavailable error without resending. A successful
call with response decode failure still returns its healthy connection as before.

Preserve conn.call's existing signature for package callers/tests by delegating to
a private optional-permit implementation if needed. After SetWriteDeadline and
before first payload Write, check context; cancellation already observed at that
point performs no write and no cooldown. Bracket payload/LF writes so Finish runs
on every attempted-write return path BEFORE SetReadDeadline/read. Existing callback
retirement and lock ordering must remain intact. Nil owner is allowed only for
legacy isolated clients; a separate wiring builder injects one shared production owner.

## Required independent proof

Critic receives this ORIGINAL assignment and complete implementation, checks every
material deviation, exact frozen fixtures, bounded records, blocking waits, retry
phase semantics, write/callback/defer ordering and ALL production construction sites.
No source approval without code evidence. Only after source approval, one assigned
compiler with fresh RAM/swap preflight and existing Go caps runs verbose focused
race tests, full SFWP race, full module race and `just casework-go-check` sequentially.
Actual exits, immutable raw/preflight/exit captures and exact comparisons required.
No-spin and both pool-at-release assertions must now be reached and pass. Any defect
requires a DIFFERENT fresh builder and repeat independent review. This unit cannot
settle T09 or claim live/browser/CI/publication success.
