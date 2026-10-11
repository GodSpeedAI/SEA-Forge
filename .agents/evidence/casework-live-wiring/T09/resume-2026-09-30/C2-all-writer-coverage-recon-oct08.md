# C2 global-event and case-facts writer coverage recon

Date: 2026-10-08  
Scope: read-only source recon supporting the C2 candidate. This identifies the in-tree global ledger append gateway and concrete case-state writers outside it. It does not establish complete participation by arbitrary external writers or approve an architecture.

## Finding

In the indexed in-tree Rust server/CLI/case-runner source, global SFWP event ledger append is centralized through `ServerState::publish_event` → `sfwp::events::append_event` → `LedgerStream::append`. The current search found no second direct caller of `sfwp::events::append_event`. This does **not** prove complete case-facts writer coverage: multiple in-tree CLI paths mutate directory-backed or case-keyed state without using the server publisher, and the server's mutation/publication sequence is not one atomic commit. A derived head index over the global event ledger therefore cannot, by itself, prove that case facts are current or that every facts mutation has an indexed event. The selected candidate remains on HOLD for writer coverage and facts/frontier completeness.

## Global append and read paths

* `crates/sea-forge-server/src/sfwp/events.rs:61-89` constructs a stored event, calls `ledger.append`, and returns an `EventFrame` whose cursor is the appended entry's `entry_ulid`. This is the only in-tree direct call target found for the global event append operation.
* `crates/sea-forge-server/src/lib.rs:208-234` is the gateway. It appends on a blocking task, then calls `event_bus.send(frame.clone())`; the append and broadcast are sequential operations, not an atomic transaction.
* Gateway callers found by Graft are `sfwp::case_mutations::run_case_mutation` (`case_mutations.rs:147-193`), `commit_plan` (`lib.rs:2764-2832`, two events), `decide` (`lib.rs:3034-3134`), `delegate_inner` (`lib.rs:3139-3252`), and `cancel_delegation` (`lib.rs:3267-3339`). These cover mutation trace publication, submit/config notification, approval outcomes, and delegation lifecycle events. The full source call search did not reveal another direct ledger append path.
* Durable reads go through `sfwp::events::get_range` (`events.rs:169-208`) and `replay_after` (`events.rs:140-163`), called by `ServerState::events_get_range` (`lib.rs:236-251`) and `events_replay_after` (`lib.rs:253-265`). Protocol handling reaches these through the `events.get_range` request and subscription catch-up. They read the ledger; they do not provide a fence over case-directory mutations.

## In-tree facts writers not covered by global event append

The accepted source inventory `case-writer-inventory-source-recon-revision2-oct07.md` and its independent review establish these path facts; this recon narrows their relevance to C2 writer coverage:

* CLI `case reopen`, `case add-task`, `task complete`, and `case manager-iterate` reach shared `case_ops` helpers that update case JSON, plans, local case ledgers, or approvals. CLI uses non-notifying helpers and does not call `ServerState::publish_event` (`commands/case.rs:11-44`; `commands/task.rs:12-23`; `commands/manager.rs:203-290`; `case_ops/mod.rs:154-176,241-303,332-445`).
* CLI `project` reaches `sea_forge_case_runner::run_stage_case`, which creates `cases/<id>/case.json`, `plan.json`, and local case events (`commands/project.rs:197-228,312-320`; `case-runner/src/lib.rs:374-420,548-560`). CLI plan execution, resume, migration, and case approval resolution are additional case-state or case-keyed-ledger writers listed in the accepted inventory. They do not use this server process's publisher. Plain-intent CLI `run` writes a flat legacy record, distinct from directory membership.
* Within the server, `run_case_mutation` performs the case operation first and then attempts per-event global publication; publication errors are logged/consumed and do not roll back the mutation (`sfwp/case_mutations.rs:147-193`). `commit_plan` similarly attempts `case.submitted` after submit/cache mutation and ignores publication failure (`lib.rs:2788-2828`). Approval resolution writes case-keyed records before attempting a global event (`lib.rs:3087-3117`). These paths cannot establish transactional index coverage solely from the current ledger append point.
* `case.advance`/supervisor uses a process-local keyed lock, but `case.list` does not take an inventory snapshot lock; the server `.server.lock` excludes a second server process, not the CLI paths. The accepted revision-2 inventory/review explicitly limits conclusions to inspected in-tree paths and does not claim exclusion of manual actors, scripts, plugins, unindexed code, or external processes.

## Retrieval and verification evidence

Graft was queried first:

* `graft_find_all(pattern="append_event")`: 43 matches in 14 symbols across 8 files; results distinguish `CaseRunner::append_event` local journals from server `sfwp::events::append_event`.
* `graft_find_all(pattern="sfwp::events::append_event")`: one match, the call from `ServerState::publish_event` at `lib.rs:222`.
* `graft_find_all(pattern="publish_event")`: gateway and all listed in-tree call symbols.
* `graft_find_all(pattern="get_range")` and `graft_find_all(pattern="replay_after")`: global ledger read helpers and their server/protocol callers.

Then the narrowed known-literal source search was run:

```text
rg -n --glob '*.rs' 'sfwp::events::append_event|events::append_event|publish_event\(|EventFrame|append_event\(' crates/sea-forge-server/src crates/sea-forge-cli/src crates/sea-forge-case-runner/src
```

It showed one direct `sfwp::events::append_event` call in `lib.rs`, server gateway call sites, and numerous `CaseRunner::append_event` calls in CLI/server/case-runner code. Those latter calls are case-local journals and are not the global SFWP event ledger. Prior accepted inventory searches covered `case.json`, `case-events.jsonl`, `save_case`, `run_stage_case`, CLI/server routes, and production file operations in the Rust server/CLI/case-runner trees; its independent review approved the stated bounded path inventory and caveated unobserved external writers.

## Coverage disposition and limit

In-tree **global event append gateway coverage** is supported for the indexed Rust sources and the bounded literal search above. **Case-facts writer-to-global-event coverage is disproved** by the CLI/CaseRunner paths and non-transactional server publication handling. Coverage outside those source trees, and external processes that modify the cell, remains unknown. No process-local lock or append/broadcast order may be treated as an external-writer fence. The C2 candidate must remain unavailable/held wherever it cannot reconcile facts against a provable ledger boundary; an event-head index alone cannot promote those observations to complete.

No source, test, public contract, schema, runtime, compiler, formatter, build, or Git state was changed.
