# Independent review: case membership and state writer inventory, revision 2

Date: 2026-10-07  
Reviewed artifact: `case-writer-inventory-source-recon-revision2-oct07.md`  
Reviewed SHA-256: `aec46446569d9948c38f11f70975760a8725ecdd818c4ac7ae7dbf6400bddbae`  
**Verdict: APPROVE as a bounded, factual source inventory.** It addresses the concrete omissions in the first inventory. It is not proof of a universal writer set, an atomic inventory/frontier, supported concurrency policy, or authorization for a public architecture change.

## Governing material and method

The revision 2 artifact preserves the original assignment verbatim: enumerate supported case membership/state writers across server and CLI/process entrypoints; state actual lock scope, local/global event publication and result handling, and cross-process participation; include deletion/rename/restore/migration/import only when found; provide bounded search evidence; do not turn source recon into an architectural guarantee. I compared the complete revision 2 with the first inventory and its rejection, plus the original atomicity/frontier recon and its independent review:

| Record | SHA-256 |
| --- | --- |
| Revision 2 under review | `aec46446569d9948c38f11f70975760a8725ecdd818c4ac7ae7dbf6400bddbae` |
| First inventory | `b73b4033f67a001faf4f792d951f9e5e655df9be9034776dbab61ec5bf203b4d` |
| Rejection of first inventory | `0844de8d9b8f13a140640477299464556c9b6a4862750682200090ffd079c418` |
| Atomicity/frontier recon | `c548cd87139b6d63e562357b07a5e845225cf9119b6b679ac595e8f6b4d58cf8` |
| Review of atomicity/frontier recon | `aaf7f51e19a4e6d726b9c07edb847903da36166f47a3540dfff16bfe3006e949` |

I used Graft first for the writer/lock landscape, then checked the claimed route and writer relationships against direct source and known-literal searches. In particular, I independently ran the revision's bounded searches over production Rust server/CLI/case-runner sources for case paths, `save_case`, `run_stage_case`, file mutation operations, CLI/server route names, and over the named Go subtree for case paths and commit delegation. I read the relevant CLI command/dispatch branches, shared `case_ops`, case runner, server request arms and mutation wrappers, ledger locking code, and approval journal code. No tests, compiler, services, runtime or Git commands were run; no source was changed.

## Material findings resolved

The revision adds every concrete route identified by the prior rejection and differentiates their effects:

* CLI `case reopen`, `case add-task`, `task complete`, `case manager-iterate`, and `project` are present as separate rows. The route enums and dispatch arms in `crates/sea-forge-cli/src/main.rs:25-35,607-685` point to the wrappers in `commands/case.rs:11-44`, `commands/task.rs:12-23`, `commands/manager.rs:61-290`, and `commands/project.rs:197-320`. The shared implementation confirms that reopen saves the case record, proposal writes the plan plus case ledger/event, human completion can save terminal state, manager escalation saves `AwaitingApproval`, and `run_stage_case` creates and rewrites directory-backed case state (`crates/sea-forge-case-runner/src/case_ops/mod.rs:154-176,241-303,332-445`; `crates/sea-forge-case-runner/src/lib.rs:374-420,548-560`).
* The inventory corrects the plain-intent/plan conflation. `commands/run.rs:13-37` selects `plan_pipeline::run_plan` only with `--plan`; plain intent goes to `pipeline::run_intent`, whose flat `cases/<case-id>.json` output (`pipeline.rs:166-182,265-281`) is distinct from the directory-backed `cases/<id>/case.json` enumerated by `case_views::list` (`case_views.rs:310-356`). Its narrower list-visibility statement is correctly qualified.
* Approval resolution is explicitly included only as broad case-keyed state: `resolve_approval` commits to the `case-<id>` ledger and appends `approvals.jsonl`; it does not save directory `case.json` or append `case-events.jsonl` (`case_ops/mod.rs:452-628`; `sea-forge-core/src/approvals.rs:24-49`). CLI and server callers are separately identified. This is a useful scope boundary, not a claim that approval changes case-list membership.
* The missed `manager.rs` → CLI `save_case` caller is verified directly: `commands/manager.rs:247-290` calls the wrapper at `commands/case.rs:53-55`, which calls shared `case_ops::save_case`. The revision correctly treats an absent Graft caller edge as a graph limitation, not proof of no caller.

## Lock, publication and error-handling verification

The inventory's substantive concurrency distinctions match the source:

* The lifetime `.server.lock` is taken at server startup to reject a second server for that cell (`crates/sea-forge-server/src/lib.rs:1090-1115,1187-1200`). The inspected CLI writers do not take it. The revision does not overstate this as a lock for every process.
* `ServerState.case_locks` protects the same-case advance/supervisor path (`lib.rs:343-375`; `sfwp/case_mutations.rs:392-461`), not `case.list`. The listed add-item/reopen/terminate/human-task wrappers use `run_case_mutation` without acquiring that keyed lock (`sfwp/case_mutations.rs:147-193,195-390`). The admission semaphore is capacity control, not an inventory mutex (`lib.rs:67-114`).
* The `_with` case-operation callbacks and `run_case_mutation` make local trace appends and attempted global publication distinct. The latter logs/consumes publication failures after mutation, while `case.commit` attempts `case.submitted` after submit and ignores the publication result (`sfwp/case_mutations.rs:147-193`; `lib.rs:2788-2828`). The inventory does not equate local `CaseCreated`/`PlanMutated` events with durable global SFWP `EventFrame`s.
* File writes themselves have narrower semantics than a shared writer lock: shared `case_ops::write_json` uses temporary-file rename (`case_ops/mod.rs:63-68`), while the case runner's writer is direct `fs::write` (`case-runner/src/lib.rs:570-575`). Neither source path establishes an inventory-wide lock.
* **Approval-lock precision:** the approval rows correctly say that no keyed server case mutex surrounds approval resolution and that the ledger's persistence lock is not an inventory lock. To interpret that precisely, `LedgerStream::commit_typed_once` takes its per-ledger exclusive file lock around lookup/append (`crates/sea-forge-ledger/src/types.rs:527-563`; lock path and scope at `:701-723`), so server and CLI do coordinate that specific ledger operation cross-process when they share the root. `approvals.jsonl` is appended separately with an `O_APPEND` handle and one record write, with no shared cell/inventory lock (`crates/sea-forge-core/src/approvals.rs:31-52`). Neither lock spans directory case listing, the whole approval-resolution read/ledger/journal/publication sequence, or global SFWP publication. The revision's wording distinguishes these scopes sufficiently; this review does not treat it as an unlocked ledger append.

The server request enum/arms inspected cover the listed case paths: commit, add item, reopen, terminate, advance/item execute, human completion, and approval; `CaseList` is a read path (`lib.rs:643-692,2390-2405,2450-2668`). The revision also accounts for CLI plan execution, artifact-transition plan execution, resume, manager, project, plain-intent legacy output, migration, and approvals. The bounded deletion/rename search reasonably reports migration relocation and empty legacy-run-directory cleanup, and does not claim an exhaustive search of arbitrary external/manual activity.

## Limits and disposition

No material factual contradiction or missing supported server/CLI writer path was found in the stated Rust server, CLI and case-runner scope, nor in the bounded Go delegation check. The inventory explicitly limits its conclusions to inspected trees and known source paths; it does not prove that manual actors, scripts, plugins, unindexed code or external processes never write the root. It leaves concurrent CLI policy unresolved. It also preserves the atomicity/frontier recon's conclusion that current list plus range cannot establish a complete atomic frontier; this inventory does not approve the earlier candidate architecture.

Approve revision 2 as factual input for further bounded source/design reasoning, subject to the stated path boundaries. No claim here authorizes a new public interface, schema, lock policy, operator support policy, implementation, or runtime behavior. This review is source-only.
