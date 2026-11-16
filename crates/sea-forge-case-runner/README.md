# sea-forge-case-runner

## Overview
Synchronous lifecycle primitives for SEA Forge cases (Cargo.toml). Provides
the governed case mutation operations and advancement engine shared verbatim
by the CLI and the SFWP server, stage-case execution, and governed cell
episodes: one command in an OpenShell sandbox, its changeset review, and a
separately approved promotion. `#![forbid(unsafe_code)]` [src/lib.rs].

## Interface
- Modules: `case_ops` (governed case mutations, advancement, authority mirrors, mediation) and `cell_episode` (sandbox episodes) [src/lib.rs:pub mod].
- `CaseRunner` facade (`initialize_case`, `next_ready_actions`, `append_event`), `run_stage_case`, `StageCaseOutcome`, `write_json` [src/lib.rs].
- Cell episodes: `run_cell_episode`, `promote_cell_changeset`, `staging_dir` [src/cell_episode.rs].
- Case mutations and mediation: `advance_case`, `authorize_action`, `with_authorized_action`, `append_case_event_with`, `reopen_with_reason`, `terminate_with`, `propose_item_with`, `complete_human_task_with`, `resolve_approval`, `CaseEventNotifier` [src/case_ops/mod.rs].
- Source tree contract: [src/README.md]; case_ops module: [src/case_ops/README.md]

## Invariants
- Case mutations have exactly one implementation shared by CLI and server; no private copies of governed writes [src/case_ops/mod.rs:module docs].
- No append lands on `case-events.jsonl` without a host event frame being published [src/case_ops/mod.rs:module docs].
- Cell-episode ordering is the security contract: authorize the sandbox command before touching the repository; destroy the sandbox before trusting its result; promotion escalates and moves the authoritative ref only after human approval via compare-and-swap [src/cell_episode.rs:module docs].
- A mutation is not authorized until its decision is ledgered [src/case_ops/mediation.rs:module docs].

## Side Effects
- Appends case trace events and ledger records; writes case state atomically via temp-file rename [src/case_ops/mod.rs:append_case_event_with], [src/case_ops/mod.rs:save_case].
- Creates workspaces/artifacts directories and writes JSON outputs [src/lib.rs:run_stage_case].
- Runs sandboxed episodes through host-supplied executor callbacks; writes authority mirror views [src/case_ops/advance.rs:EpisodeExecutor], [src/case_ops/authority_views.rs:rebuild_authority_mirrors].

## Verification
- Run: `cargo test -p sea-forge-case-runner` (repo-wide: `just test`)
