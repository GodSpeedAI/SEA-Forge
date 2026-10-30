# sea-forge-planner/src

## Overview
Source for the `sea-forge-planner` crate: SEA Forge's case plan construction. `lib.rs` builds case plans from domain intents and the canonical spec-pipeline stage plan; `case_engine.rs` is the deterministic case engine (sentry evaluation, plan validation, replay, action derivation); `criteria.rs` derives and verifies settlement-criteria records; `templates.rs` loads, pins, instantiates, and stores plan templates.

## Interface
- Modules: `case_engine`, `criteria`, `templates` [lib.rs:pub mod]
- `plan(intent, case_id, run_id, executable) -> Result<CasePlan, ForgeError>` — maps an `IntentPattern` (via `sea_forge_domain::interpret`) to a single-item `CasePlan` (demo/TEST_ONLY variants) [lib.rs:plan]
- `stage_case_plan(stages, case_id, run_id, intent_id) -> Result<CasePlan, ForgeError>` — canonical M5 spec-pipeline plan; each governed stage becomes a required `SandboxedTask` `PlanItem` chained by a settlement-accepted reactivation sentry [lib.rs:stage_case_plan]
- `DEMO_MODEL` — the fixed demo `.sea` model content [lib.rs:DEMO_MODEL]
- Criteria: `compute_criteria_sha256`, `compute_record_hash`, `derive_from_intent`, `derive_from_template`, `verify_desired_outcome_refs`, `verify_item_criteria`, `verify_plan_criteria`, `verify_plan_criteria_with_resolver`; traits `CriteriaLookup`, `DesiredOutcomeResolver`; resolvers `NoModelResolver`, `SeedModelResolver` [criteria.rs]
- Case engine: `evaluate_sentries`, `check_satisfiability`, `validate_proposal`, `can_auto_complete`, `required_item_failed`, `replay_case`, `next_case_actions`, `replay_activations`; types `ItemState`, `ItemStatus`, `CaseAction`, `CaseProjection`; `MAX_PLAN_ITEMS = 256` [case_engine.rs]
- Templates: `PlanTemplate`, `ParameterDef`, `TemplatePlan`, `TemplateItem`, `TemplateOperation`, `RepeatedItem`, `MAX_REPEATED_ENTRIES = 32`; functions `instantiate`, `load`, `parse_template_ref`, `load_pinned`, `store_builtin`, `is_built_in_template_ref`; built-in templates `sea_model_demo_template`, `adlc_case_template`, `odi_adlc_case_template`, `sequential_agents_template`, `concurrent_agents_template`; `DEFAULT_TOPOLOGY_ENDPOINT_REF = "agent_default"` [templates.rs]

## Invariants
- Every `stage_case_plan` item uses `sandbox_class: Some("jail")` (Landlock), never `local`: a stage's command is arbitrary generator/validator tooling, not necessarily the trusted `sea-forge` binary, so it cannot rely on the `local` class's trusted-argv0 exemption (§11.2) [lib.rs:stage_case_plan]
- Every `stage_case_plan` item is `markers.required = true` (load-bearing, not optional side work): an unrequired item can be silently skipped by `can_auto_complete` before it ever activates [lib.rs:stage_case_plan], [case_engine.rs:can_auto_complete]
- Spec-pipeline stages must declare `command` or planning fails with `spec_pipeline_error`; already-settled predecessors are omitted from `stages` and referenced only via a later stage's `inputs` [lib.rs:stage_case_plan]
- `SettlementStatus` sentry predicates evaluate only settlement events from the sentry's named source item, not any matching event in the case (source-bound, spec-adlc-thoth §7.5/§10.4); `"case"` is the wildcard [case_engine.rs:predicate_holds]
- `compute_record_hash` hashes a `SettlementCriteriaRecord` excluding only the `criteria_record_hash` field; criteria digests are the canonical SHA-256 via `sea-forge-ledger`'s `hash_canonical` [criteria.rs:compute_record_hash], [criteria.rs:compute_criteria_sha256]
- `parse_template_ref` accepts only `name@version` with a strict grammar (lowercase/digit/`_`/`-` names; alnum/`.`/`-` versions), so a traversal-shaped ref (`../x@1`) can never escape the templates directory [templates.rs:parse_template_ref]
- `load_pinned` pins the template's first-used byte hash; a later content change under the same version fails [templates.rs:load_pinned]
- A single `RepeatedItem` group may expand to at most `MAX_REPEATED_ENTRIES = 32` items [templates.rs:MAX_REPEATED_ENTRIES]
- Plans are bounded at `MAX_PLAN_ITEMS = 256` items [case_engine.rs:MAX_PLAN_ITEMS]
- `validate_proposal` and satisfiability checks run before a proposal is accepted [case_engine.rs:validate_proposal], [case_engine.rs:check_satisfiability]

## Side Effects
- Filesystem: template load/store reads and writes `<root>/templates/<name>@<version>.yaml` (`load`, `load_pinned`, `store_builtin`) [templates.rs:load], [templates.rs:load_pinned], [templates.rs:store_builtin]
- The case engine itself is deterministic and pure over `PlanItem`/`TraceEvent` inputs [case_engine.rs]

## Verification
- Run: `cargo test -p sea-forge-planner` (repo-wide: `just test`)
