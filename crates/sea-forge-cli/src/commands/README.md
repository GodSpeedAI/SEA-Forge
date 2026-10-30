# sea-forge-cli — src/commands

## Overview
Command implementations for the `sea-forge` binary, declared as clap
subcommands in [../main.rs]. The module tree is a flat list of one file per
subcommand, declared in [mod.rs]. Several commands are deliberately thin
adapters over shared kernel libraries so the SFWP server calls the same
governed implementation in-process (approve, case, mediated, task).

## Interface
- Module tree ([mod.rs]): `agent`, `approve`, `artifact`, `ask`, `case`, `env`, `federation`, `inspect`, `ledger`, `manager`, `mediated`, `memory`, `migrate`, `project`, `recall`, `resume`, `run`, `runs`, `self_model`, `task`, `validate`.
- Agent orchestration ([agent.rs]): `list`, `probe`, `delegate`, `cancel`.
- Approvals ([approve.rs]): `ApproveOptions`, `approve`, `reject` — thin surface over `sea_forge_case_runner::case_ops::resolve_approval` [approve.rs:module docs].
- Artifact/IP ([artifact.rs]): `transition_command`, `AttestOptions`, `attest_command` (M8 transitions; the token is committed last) [artifact.rs:module docs].
- Thoth ask ([ask.rs]): `run` — exit codes: 0 answered, 4 denied (governance success), 2 usage, 1 error [ask.rs:module docs].
- Case mutations ([case.rs]): `reopen`, `add_task`, `propose_item` — thin surface over `sea_forge_case_runner::case_ops` [case.rs:module docs].
- Environments ([env.rs]): `list`, `show` (§7.6 EnvironmentSpecs).
- Federation ([federation.rs]): `ExportOptions`/`export`, `ImportOptions`/`import`, `AdoptOptions`/`adopt` — thin wrappers around `sea_forge_cell` with authority approval [federation.rs:module docs].
- Inspection and ledger ([inspect.rs]: `execute`; [ledger.rs]: `execute` with `LedgerAction`).
- Manager loop ([manager.rs]): `ManagerIterateOptions`, `iterate`.
- Mediated authority ([mediated.rs]): `policy_path`, `assurance` — thin surface over `sea_forge_case_runner::case_ops::mediation` [mediated.rs:module docs].
- Memory ([memory.rs]): `rebuild`.
- Migration ([migrate.rs]): `MigrateOptions`, `execute`.
- Projection ([project.rs]): `ProjectOptions`, `execute` — thin adapter wiring a CLI invocation through the case-runner stage path and a validated DomainForge model; SEA Forge (not DomainForge) owns every authorized output [project.rs:module docs].
- Recall ([recall.rs]): `RecallOptions`, `execute`.
- Resume ([resume.rs]): `ResumeOptions`, `ResumeOutcome`, `resume`.
- Run/runs ([run.rs]: `execute`; [runs.rs]: `list` with `--unsettled`).
- Self-model ([self_model.rs]): `validate`, `rebuild`, `show` (§11.1; probe execution here is a no-op stub until the probe runner exists) [self_model.rs:module docs].
- Human task ([task.rs]): `complete` — thin surface over `case_ops::complete_human_task` [task.rs:module docs].
- Validation ([validate.rs]): `execute`, `execute_stage_check`.

## Invariants
- Governed mutations (approve, case, task, mediated) have exactly one implementation in `sea_forge_case_runner::case_ops`; these wrappers keep the CLI command surface unchanged [approve.rs:module docs], [case.rs:module docs], [task.rs:module docs], [mediated.rs:module docs].
- Federation export/import/adopt require authority approval at this layer [federation.rs:module docs].
- M8 transitions commit the token last [artifact.rs:module docs].
- `ask` denial is governance success (exit 4), not an error [ask.rs:module docs].

## Side Effects
- Commands print to stdout/stderr (rendered results of typed kernel calls) and return process exit codes [agent.rs], [../main.rs:dispatch].
- Write JSON/JSONL state under the case/run directories through the kernel libraries they call (case mutations, approvals journal, ledger appends) [case.rs], [approve.rs].
- Run stage paths and sandboxed episodes through `sea_forge_case_runner`/`sea_forge_planner` [project.rs], [resume.rs].

## Verification
- Run: `cargo test -p sea-forge-cli` (repo-wide: `just test`)
