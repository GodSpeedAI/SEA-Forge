# Independent review: case inventory atomicity/frontier source recon

Date: 2026-10-06  
Reviewed artifact: `case-inventory-atomicity-frontier-source-recon-oct06.md`,
reported identity prefix `c548cd`.  
Disposition: **APPROVE as bounded DOCONLY source recon and candidate for
adjudication.** It does not prove an atomic inventory/frontier and does not
authorize V4, a DTO change, source work, or runtime claims.

## Scope and source verification

I reviewed the original source-recon scope, the complete recon, the fail-closed
source-options note, the prior case-inventory/range-frontier recon, and CW-13
and CW-14 in `.agents/DEBT.md:669-710`. I verified the material source anchors
against current files:

* `case_views::list` returns a default result when `read_dir` fails, flattens
  entry errors, skips non-directories/non-UTF-8 names, and omits
  `NotFound` records; it has no lock (`crates/sea-forge-server/src/sfwp/case_views.rs:310-355`).
  `read_case` maps metadata and file-read failures to `NotFound`
  (`:249-270`). The production `Request::CaseList` branch directly serializes
  that result (`crates/sea-forge-server/src/lib.rs:2396-2397`). The recon
  accurately distinguishes these behaviors and does not claim the fail-closed
  source-options proposal solves concurrent atomicity.
* `ServerState.case_locks` is documented and used for same-case
  `case.advance`/supervisor serialization, not inventory
  (`lib.rs:77-86`; `sfwp/case_mutations.rs:396-462`;
  `supervisor.rs:153-194`). The semaphore is
  capacity-based admission, not a single-writer fence (`lib.rs:91-100`). The
  listed `case.add_item`, `case.reopen`, `case.terminate`, and
  `human_task.complete` wrappers call `run_case_mutation` without acquiring
  `case_locks` (`sfwp/case_mutations.rs:147-193,195-355`). Those are valid
  concrete counterexamples to “all writers share the existing lock.”
* `case.commit` dispatches through `case_dispatch::submit` and writes case
  state before its later `case.submitted` publication (`lib.rs:2759-2828`;
  `case_dispatch.rs:39-124`). The publication result is ignored before a
  successful response (`lib.rs:2813-2828`), matching CW-14. Case JSON writes
  are direct `fs::write` (`crates/sea-forge-case-runner/src/lib.rs:570-575`).
  The recon correctly says successful create ordering closes only the specific
  create-after-frontier schedule under successful writes; it does not make a
  separate list/range pair atomic.
* The mutation helper drains its event channel and awaits the publisher, but
  logs and consumes a failed global append (`sfwp/case_mutations.rs:147-193`).
  Thus the recon's distinction between durable local mutation and a real global
  cursor is accurate. `publish_event` returns the actual appended frame/cursor
  only after the durable append (`lib.rs:208-234`); no cursor should be
  fabricated from a successful mutation response.
* `events.get_range` returns rows, not a head/completion marker, and its
  implementation clamps pages at 500 after reading ledger entries
  (`lib.rs:2349-2361`; `sfwp/events.rs:165-207`). The recon correctly notes
  that page size does not bound full-ledger materialization (CW-12,
  `.agents/DEBT.md:655-667`).
* The recon's cross-process CLI caveat is warranted: CLI plan execution writes
  case files (`crates/sea-forge-cli/src/plan_pipeline.rs:374-413,480-500`),
  while the cited server lock is acquired by server startup
  (`crates/sea-forge-server/src/lib.rs:1094-1115`) and no participation by
  this CLI writer is established by the inspected path. The recon correctly
  leaves supported concurrent CLI use unresolved rather than asserting it
  occurs or is forbidden.
* The proposed Go Store/Relay use remains derived capture, not a second durable
  truth source (`apps/godspeed-casework-go/internal/projection/store.go:55-65,149-220`;
  `internal/server/relay.go:178-217`). Existing case-list and range responses
  do not provide the exact captured global frontier and per-case cursor
  association the candidate requires. The recon explicitly holds any DTO/verb
  expansion for review.

## Findings

No material factual error or unsupported atomicity claim found. In particular,
the recon does not overstate the useful `case.commit` ordering: it records the
narrow successful-commit result and then provides interleavings and failure
paths that defeat a complete inventory/frontier proof. It preserves the
nonempty-cursor handler qualification and does not treat absent/empty cursor as
historical mode.

The candidate is appropriately labeled incomplete and held for adjudication.
Its process-local barrier is only sufficient if every authoritative writer
participates. The recon explicitly surfaces the CLI/direct-filesystem boundary,
the need to inventory all writer/publication intervals, and the need for an
operator decision on supported sole-writer ownership or a shared cross-process
mechanism. Before any future source assignment, the exact writer set must be
enumerated, including commit/dispatch, advance/supervisor, case mutation
wrappers, and any approved CLI/direct writers; the barrier interval must end
only after the corresponding durable global publication succeeds or the case
is made unavailable. This is a completeness requirement for future design,
not a defect in this bounded recon, which does not claim exhaustive proof.

The candidate's uses of a fallible inventory, an actual ledger cursor, and
existing Go Store/Relay are proposals only. Existing `case.list` failure
semantics, Go's ignored `Unreadable`/missing-array behavior
(`apps/godspeed-casework-go/internal/adapters/sfwp/authority.go:69-88`),
publication-result handling, public error mapping, and bounded ledger
materialization remain unresolved. The listed DTO/error and operator holds are
accurate and must remain gates. No source-options proposal, public error
contract, added DTO field/verb, cursor namespace, read cap, or operator policy
is approved by this review.

## Verdict boundary

Approve the recon as a source-grounded explanation of why the current
inventory-plus-range flow cannot prove a complete frontier, and as a candidate
for further operator/design adjudication. Do not interpret this verdict as
approval of the candidate architecture, a proof that all writer paths have
been exhaustively enumerated, or authorization to change source, schema,
runtime, or T09 settlement. No tests, compiler, scanner, Git, or runtime action
was performed.
