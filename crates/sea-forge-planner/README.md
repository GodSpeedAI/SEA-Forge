# sea-forge-planner

## Overview
SEA Forge crate for case plan construction (`Cargo.toml` description). Builds `CasePlan`s from domain intents (`plan`) and the canonical spec-pipeline stage plan (`stage_case_plan`), and owns the deterministic case engine (sentry evaluation, plan validation, replay, action derivation), settlement-criteria derivation/verification, and plan-template loading, pinning, and instantiation. Depends on `sea-forge-core`, `sea-forge-domain`, and `sea-forge-ledger`.

## Interface
- Source contract: [./src/README.md] — `plan`, `stage_case_plan`, case engine (`evaluate_sentries`, `validate_proposal`, `replay_case`, ...), criteria (`derive_from_intent`, `verify_plan_criteria`, ...), templates (`PlanTemplate`, `instantiate`, `load_pinned`, built-in templates) [src/lib.rs:plan], [src/lib.rs:stage_case_plan], [src/case_engine.rs], [src/criteria.rs], [src/templates.rs]
- Workspace manifest: `Cargo.toml` (package `sea-forge-planner`; depends on `sea-forge-core`, `sea-forge-domain`, `sea-forge-ledger` 0.3.0)

## Invariants
- Spec-pipeline stages run with `sandbox_class: "jail"` (never `local`) and are always `required`, since an unrequired item can be silently skipped by `can_auto_complete` [src/lib.rs:stage_case_plan], [src/case_engine.rs:can_auto_complete]
- `SettlementStatus` sentry predicates are source-bound (evaluate only the named source item's settlement events; `"case"` is the wildcard) [src/case_engine.rs:predicate_holds]
- Template references follow a strict `name@version` grammar that blocks path traversal; `load_pinned` fails on any content change under a pinned version [src/templates.rs:parse_template_ref], [src/templates.rs:load_pinned]
- Plans are bounded at `MAX_PLAN_ITEMS = 256`; typed repeated expansion is bounded at `MAX_REPEATED_ENTRIES = 32` [src/case_engine.rs:MAX_PLAN_ITEMS], [src/templates.rs:MAX_REPEATED_ENTRIES]

## Side Effects
- Filesystem: template load/store reads and writes `<root>/templates/<name>@<version>.yaml` [src/templates.rs:load], [src/templates.rs:store_builtin]

## Verification
- Run: `cargo test -p sea-forge-planner` (repo-wide: `just test`)
