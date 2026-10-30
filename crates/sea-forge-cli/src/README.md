# sea-forge-cli — src

## Overview
Source tree for the `sea-forge` one-shot CLI binary ("Governed
capability-execution kernel", per the clap command in [main.rs]).
`#![forbid(unsafe_code)]` [main.rs]. `main.rs` defines the clap `Cli`/
`Command` grammar, `dispatch`, and process `ExitCode` mapping;
`pipeline.rs` implements the intent-run pipeline; `plan_pipeline.rs` the
plan-run pipeline with approval parking/extension; `approvals.rs` is the
CLI's view onto the shared approvals journal; `commands/` holds the per-
subcommand implementations.

## Interface
- Module tree, declared in [main.rs]: `approvals` ([approvals.rs]), `commands` ([commands/README.md]), `pipeline` ([pipeline.rs]), `plan_pipeline` ([plan_pipeline.rs]).
- Entry points ([main.rs]): `main` and `dispatch` (clap `Parser`/`Subcommand` grammar, `ExitCode` mapping), `init_diagnostics` (tracing EnvFilter).
- Intent pipeline ([pipeline.rs]): `RunOptions`, `RunOutcome`, `run_intent`, `seed_desired_outcome_resolver`, `load_item_environment`, `ADLC_ODI_SEED_DOMAIN_MODEL_REF`.
- Plan pipeline ([plan_pipeline.rs]): `PlanRunOptions`, `PlanRunOutcome`, `ApprovalParkingContext`, `PlanApprovalExtension`, `run_plan`, `run_plan_with_approval_extension`, `append_event`, re-exported `write_json`.
- Approvals ([approvals.rs]): re-exports `append`, `check_expiry`, `latest_status`, `load_all`, `pending_for_case` from `sea_forge_core::approvals` — the journal format and "latest record per `approval_id` wins" fold have a single owner in core [approvals.rs:module docs].
- Subcommands: see [commands/README.md].

## Invariants
- `#![forbid(unsafe_code)]` [main.rs:forbid(unsafe_code)].
- The approvals journal has exactly one fold owner (`sea_forge_core::approvals`); the CLI must not implement a second one, because two independent folds could disagree about whether an approval is still open and offer an operator an action on an already-decided request [approvals.rs:module docs].
- Attribution names passed to the intent pipeline are validated before use [pipeline.rs:validate_attribution].
- Process exit is `ExitCode::from(dispatch(...))`: the dispatch result code is the process exit code [main.rs:main].

## Side Effects
- Runs governed cases end-to-end: appends trace events to case ledgers via `LedgerStream`, writes plan/trace/evidence JSONL artifacts, creates run directories and stdout/stderr artifact files [pipeline.rs:run_intent], [plan_pipeline.rs:run_plan_inner].
- Executes SandboxedTask episodes through `CaseRunner::run_sandboxed_episode` and the sandbox/runtime crates [plan_pipeline.rs].
- Initializes tracing diagnostics from the environment [main.rs:init_diagnostics].
- The binary exits with the dispatch code; see command-level side effects in [commands/README.md].

## Verification
- Run: `cargo test -p sea-forge-cli` (repo-wide: `just test`)
