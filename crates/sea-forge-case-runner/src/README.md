# sea-forge-case-runner — src

## Overview
Source tree for the synchronous lifecycle primitives for SEA Forge cases
(Cargo.toml). Crate root is `#![forbid(unsafe_code)]` [lib.rs]. `lib.rs` owns
the `CaseRunner` facade and stage-case execution; `case_ops/` holds the
governed case mutation operations shared by CLI and server; `cell_episode.rs`
implements one governed cell episode in an OpenShell sandbox end-to-end.

## Interface
- Module tree, declared in [lib.rs]: `case_ops` ([case_ops/README.md]) and `cell_episode` ([cell_episode.rs]).
- Root API ([lib.rs]): `CaseRunner` (`initialize_case`, `next_ready_actions`, `append_event`), `StageCaseOutcome`, `run_stage_case`, `write_json`.
- Cell episodes ([cell_episode.rs]): `run_cell_episode`, `promote_cell_changeset`, `staging_dir`, `CellEpisodeRequest`, `CellEpisode`, `CellOpaInput`, `PromotionRequest`, `PromotionApproval`.
- Case mutations: see [case_ops/README.md].

## Invariants
- `#![forbid(unsafe_code)]` [lib.rs:forbid(unsafe_code)].
- The cell-episode ordering is the security contract and does not change: projection identity verified before any authority call; sandbox command authorized as `execute_sandbox_command` before the repository is read or any openshell process runs; the sandbox is destroyed before its result is trusted; the returned tree is evidence, not a decision; promotion must escalate and be human-approved before the authoritative ref moves by compare-and-swap [cell_episode.rs:module docs].
- External effects of a sandbox command are outside the transaction: the ledger records what was decided and observed; it does not undo what a command did inside the sandbox [cell_episode.rs:module docs].
- Persisted trace ordinals are additive (spec §17.2 T13.2) [lib.rs:append_event].
- Case mutation invariants: see [case_ops/README.md].

## Side Effects
- Creates case `runs/` directories, workspaces, and artifact directories; appends trace events to JSONL; writes JSON via `fs::write` [lib.rs:CaseRunner::initialize_case], [lib.rs:run_stage_case], [lib.rs:write_json].
- Cell episodes: read projection/sandbox files, run the OpenShell sandbox through the host executor, read `execution.json`/`denials.log`, canonicalize repository paths, and create artifacts directories [cell_episode.rs:run_cell_episode], [cell_episode.rs:promote_cell_changeset].
- Authority decisions are evaluated and committed to ledger streams via `sea-forge-authority` [cell_episode.rs], [lib.rs].

## Verification
- Run: `cargo test -p sea-forge-case-runner` (repo-wide: `just test`)
