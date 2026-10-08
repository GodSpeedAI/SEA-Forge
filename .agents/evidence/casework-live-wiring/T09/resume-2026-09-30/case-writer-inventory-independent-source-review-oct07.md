# Independent review: case membership and state writer inventory

**Date:** 2026-10-07  
**Reviewed artifact:** `case-writer-inventory-source-recon-oct06.md`  
**Reviewed SHA-256:** `b73b4033f67a001faf4f792d951f9e5e655df9be9034776dbab61ec5bf203b4d`  
**Verdict: REJECT as an exhaustive writer inventory.** The server-side summary is useful and its key lock/publication claims are largely supported, but several CLI/process entrypoints that mutate the directory-backed case state are missing. These paths directly matter to the requested cross-process synchronization question.

## Scope and source evidence

Reviewed the complete new recon and its quoted original instructions; the existing `case-inventory-atomicity-frontier-source-recon-oct06.md` and its independent review; and the direct server, CLI, and shared case-runner sources implicated by writer and entrypoint searches. Graft was used first for case-state writer discovery and CLI call paths, followed by direct reads and exhaustive known-literal searches within the bounded Rust server/CLI/case-runner trees. No source changes, tests, compiler, runtime, or Git commands were performed.

| Evidence | SHA-256 |
| --- | --- |
| reviewed new recon | `b73b4033f67a001faf4f792d951f9e5e655df9be9034776dbab61ec5bf203b4d` |
| existing atomicity/frontier recon | `c548cd87139b6d63e562357b07a5e845225cf9119b6b679ac595e8f6b4d58cf8` |
| existing recon review | `aaf7f51e19a4e6d726b9c07edb847903da36166f47a3540dfff16bfe3006e949` |
| `sea-forge-cli/src/main.rs` | `1bc2f076ad1fa9f10b12727ef4a855756d78032d3110f65e8a120c895d705014` |
| `sea-forge-cli/src/commands/manager.rs` | `9d1ce1612bfa070d28fe517cd2a89bea0da98de66760eb60cf44683b338fec7b` |
| `sea-forge-cli/src/commands/project.rs` | `517adf3aa64ac62a5a62c5d9337ccb0a8e3cf50c9e39aac1f6c57a328c47b6e7` |
| `sea-forge-case-runner/src/case_ops/mod.rs` | `4d641cba2635901bb9d535bcb936f37383c496048e8883603188ac9ccc217b14` |
| `sea-forge-case-runner/src/lib.rs` | `95e45548c197847130cc6a6e2e5e118ffdb4274e665ee1c9201dcccc0802720d` |

## Material omissions

The new recon's table at lines 18–27 covers server startup/list/commit, server mutation wrappers and advance/supervisor, CLI plan/resume/migration, and the Go adapter. It omits the following existing CLI routes and their shared filesystem effects:

| CLI entrypoint | Writer and observed effects | Coordination/publication consequence |
| --- | --- | --- |
| `sea-forge case reopen` | `main.rs:294-313,634-647` routes to `commands::case::reopen` (`commands/case.rs:11-13`), which calls shared `case_ops::reopen`. The helper appends `CaseReopened` and writes `case.json` (`case_ops/mod.rs:154-176`). | CLI uses the non-`_with` helper and its no-op notifier (`case_ops/mod.rs:70-87`); there is no `ServerState::publish_event`, keyed server mutex, or `.server.lock` participation. This changes the state read by case views while the server may be running. |
| `sea-forge case add-task` | `main.rs:294-313,634-647` routes to `commands::case::add_task` (`commands/case.rs:15-30`), then shared `propose_item`. It commits a `case_plan_mutation`, materializes `plan.json`, and appends `PlanMutated` (`case_ops/mod.rs:241-303`). | The CLI call uses the non-notifying helper; it writes case ledger/files without the server's per-case lock or global EventFrame publication. The plan and horizon are case view inputs even if this does not alter `case.json` membership. |
| `sea-forge task complete` | `main.rs:316-324,663-684` routes to `commands::task::complete` (`commands/task.rs:12-23`), then `complete_human_task`. It appends trace events and may save a terminal `case.json` (`case_ops/mod.rs:332-420,443-445`). | CLI uses the non-`_with` helper. No shared server lock or durable server-global event publication is established. This is a direct case lifecycle writer absent from the inventory. |
| `sea-forge case manager-iterate` | `main.rs:294-313,648-659` routes to `commands::manager::iterate`. The manager commits a case-scoped `manager_iteration` record (`manager.rs:203-241`); on a stalled/proposal branch it calls `propose_item` (`:221-230`); on escalation/iteration-cap it appends approval records and saves `case.json` as `AwaitingApproval` (`:247-290`). | No keyed case lock or server-wide publication is present in this CLI path. Its effects are conditional but include case plan/event and case record mutation. |
| `sea-forge project` | `main.rs:111-121,607-632` routes to `commands::project::execute`; it creates a case ID and plan then calls shared `run_stage_case` (`project.rs:197-228,312-320`). `run_stage_case` creates `cases/<id>`, writes `case.json` and `plan.json`, and appends `CaseCreated`; it rewrites the case record after its loop (`case-runner/src/lib.rs:374-420,548-560`). | This separate CLI process writes directory membership and case-local state; no server lock or global SFWP publisher participates. It can race `case.list`/bootstrap while the server owns the cell. |

These are not speculative helpers: the CLI command enum and dispatch arms directly reach them. The shared `case_ops` path is not covered by searching only literal `case.json` writes in CLI files: state writes are abstracted behind `save_case`/`write_json`, and `project` writes from another crate. A path inventory must follow these calls or query the shared helper callers.

## Additional qualification: legacy CLI `run`

The inventory labels `run` together with plan execution but only describes `plan_pipeline::run_plan_inner` (`recon:24`). The command has two distinct paths: `commands/run.rs:13-37` dispatches `--plan` to `run_plan`, but a plain intent calls `pipeline::run_intent`. That second path writes a flat `<root>/cases/<case-id>.json` (`pipeline.rs:166-182,265-281`), not `<root>/cases/<case-id>/case.json`. Current `case_views::list` enumerates directory entries and reads each directory's `case.json` (`server/src/sfwp/case_views.rs:310-356`), so this flat legacy record is not current `case.list` membership. It should be recorded as an excluded legacy state path with that evidence, rather than implied to be the same writer as plan execution. It still creates a case-keyed durable record and can matter to other CLI/run-index views.

## Other checks and scope caveats

- The existing recon's server facts around `case.commit`, keyed `advance`, unguarded mutation wrappers, supervisor enumeration, ignored global publication errors, and `.server.lock` were checked against source and are materially accurate. The newly found CLI paths reinforce its conclusion that neither the server singleton lock nor the process-local case lock covers all writers.
- Exact direct-source search used: `rg -n --glob '*.rs' 'case\.json|case-events\.jsonl|join\("cases"\)|create_dir(_all)?\(|remove_dir(_all)?\(|remove_file\(|rename\(|write_json\(' crates/sea-forge-server/src crates/sea-forge-cli/src crates/sea-forge-case-runner/src`, plus `rg -n 'ManagerIterate|manager.iterate|manager_iteration|case\.add_item|TaskCommand::Complete'` across those source trees. It exposed the runner-level `run_stage_case` / `case_ops` writes and CLI call sites. Graft `trace_calls(save_case, direction=in, in=crates/sea-forge-cli)` reported no indexed callers; direct `rg` and source reads then verified `manager.rs` imports and calls `save_case`, demonstrating why the graph result alone is not exhaustive.
- Exact removal/migration search used: `rg -n --glob '*.rs' 'remove_dir(_all)?\(|remove_file\(|rename\(|copy\(' crates/sea-forge-server/src crates/sea-forge-cli/src crates/sea-forge-case-runner/src`. In production source it found migration's `rename` and removal of empty legacy run directories, case operation temporary-file replacement, and unrelated socket/correlation/secret cleanup. I found no production case-tree deletion or restore path in this bounded server/CLI/case-runner source search. The test-only `pipeline.rs` cleanup is not a production writer. This does not prove absence outside these indexed/source trees or against manual filesystem actors.
- `case_ops::resolve_approval` is another case-keyed ledger writer reachable from CLI `approve`/`reject` and server approval verbs (`commands/approve.rs:22-36`, `case_ops/mod.rs:452-627`). It does not write the case directory or `case-events.jsonl`; if “case state” is meant to include approval records in the case ledger/journal, the inventory must state that inclusion boundary and add it. The current recon does not define that boundary. This is an open scope qualification, not proof that approval resolution mutates the directory-backed case object.
- The existing Go search is explicitly bounded to the indexed Go tree and the `case.json` literal. The review does not expand that into an assertion about arbitrary processes or indirect filesystems.

## Material difference from original instruction

The original assignment required **every** writer that can affect the requested observations, including all CLI/process paths, and a table with lock, publication-result, and cross-process participation. The new recon's server-side analysis meets much of that structure, but the omitted CLI command paths above are concrete state writers and are not just documentation detail: they invalidate any exhaustive writer set and leave the exact cross-process race surface incomplete. The search section's described literal search did not capture the shared `case_ops` calls or `project -> run_stage_case`; the actual indexed graph also missed the `save_case` incoming edge, so direct call-site source inspection was necessary.

The candidate still correctly avoids claiming that `.server.lock` is cell-wide and leaves operator policy unresolved. Those good boundaries do not cure the incomplete inventory. No deletion/restore path is claimed here beyond the bounded search noted above, and no design or source authorization follows.

## Required disposition

Do not use this table as an exhaustive architecture input yet. Add the missing CLI rows (including each shared helper and its local-vs-global event outcome), distinguish plain-intent `run` from plan `run`, and explicitly settle whether case-keyed approval ledger writes are inside the requested state scope. Preserve the existing recon and review as immutable records; this verdict is a separate source-only artifact and makes no runtime or implementation claim.
