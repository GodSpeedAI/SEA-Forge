use sea_forge_core::{errors::ForgeError, types::*, RECORD_VERSION};
use sea_forge_domain::{self as domain, IntentPattern};

pub mod case_engine;
pub mod criteria;
pub mod templates;

pub use criteria::{
    compute_criteria_sha256, compute_record_hash, derive_from_intent, derive_from_template,
    verify_desired_outcome_refs, verify_item_criteria, verify_plan_criteria,
    verify_plan_criteria_with_resolver, CriteriaLookup, DesiredOutcomeResolver, NoModelResolver,
};
pub use templates::{
    adlc_case_template, instantiate, load, load_pinned, odi_adlc_case_template,
    sea_model_demo_template, store_builtin, PlanTemplate,
};

pub const DEMO_MODEL: &str = r#"{"domain": "demo", "entities": [{"name": "Sample"}]}"#;

pub fn plan(
    intent: &Intent,
    case_id: &str,
    run_id: &str,
    executable: &str,
) -> Result<CasePlan, ForgeError> {
    let (operations, settlement_criteria) = match domain::interpret(&intent.summary)? {
        IntentPattern::Demo => file_plan(executable, "model.sea", DEMO_MODEL, "model.sea"),
        IntentPattern::GeneratedZone => {
            file_plan(executable, "src/gen/model.sea", DEMO_MODEL, "model.sea")
        }
        IntentPattern::FalseSuccess => file_plan(executable, "other.sea", DEMO_MODEL, "other.sea"),
        IntentPattern::Nonzero => file_plan(executable, "model.sea", "{}", "model.sea"),
        IntentPattern::Timeout => (
            vec![Operation::ExecuteCommand {
                argv: vec![executable.into(), "internal-test-sleep".into(), "5".into()],
                cwd: ".".into(),
            }],
            SettlementCriteria {
                require_exit_zero: true,
                ..Default::default()
            },
        ),
    };
    Ok(CasePlan {
        version: RECORD_VERSION.into(),
        plan_id: "plan_01".into(),
        case_id: case_id.into(),
        run_id: run_id.into(),
        intent_id: intent.intent_id.clone(),
        items: vec![PlanItem {
            plan_item_id: "item_01".into(),
            name: "generate_and_validate_sea_model".into(),
            operations,
            entry_criteria: vec![],
            entry_criteria_mode: Default::default(),
            exit_criteria: vec![],
            settlement_criteria,
            settlement_criteria_ref: None,
            item_kind: Default::default(),
            sandbox_class: None,
            parent_stage: None,
            markers: Default::default(),
            max_instances: 1,
            depends_on: vec![],
            environment: None,
            proposed_by: None,
        }],
        template_ref: None,
        job_contract_ref: None,
    })
}

/// Build the canonical stage `CasePlan` for M5 spec-pipeline execution
/// (spec-audit-remediation Task 10A): each governed stage becomes an
/// ordinary `SandboxedTask` `PlanItem`, chained to the prior stage by the
/// existing settlement-accepted reactivation sentry (mirroring
/// `sequential_agents_template`'s chaining, §7.5/§10.7) so stages run in
/// canonical order without a second scheduler.
///
/// Every stage passed here must declare `command` (it is queued to run);
/// stages that are already-settled predecessor evidence should be omitted
/// from `stages` and referenced only via a later stage's `inputs`.
pub fn stage_case_plan(
    stages: &[SpecPipelineStage],
    case_id: &str,
    run_id: &str,
    intent_id: &str,
) -> Result<CasePlan, ForgeError> {
    let mut items = Vec::with_capacity(stages.len());
    for (index, stage) in stages.iter().enumerate() {
        let argv = stage.command.clone().ok_or_else(|| {
            ForgeError::Input(format!(
                "spec_pipeline_error: stage {} has no command to execute",
                stage.stage_id
            ))
        })?;
        let entry_criteria = if index == 0 {
            vec![]
        } else {
            vec![templates::reactivation_sentry(
                &stages[index - 1].stage_id,
                "accepted",
            )]
        };
        items.push(PlanItem {
            plan_item_id: stage.stage_id.clone(),
            name: format!("{:?}", stage.kind),
            operations: vec![Operation::ExecuteCommand {
                argv,
                cwd: ".".into(),
            }],
            entry_criteria,
            entry_criteria_mode: EntryCriteriaMode::Any,
            exit_criteria: vec![],
            settlement_criteria: SettlementCriteria {
                require_exit_zero: true,
                ..Default::default()
            },
            settlement_criteria_ref: None,
            item_kind: ItemKind::SandboxedTask,
            // `jail` (Landlock), never `local`: a stage's command is
            // arbitrary generator/validator tooling, not necessarily the
            // trusted `sea-forge` binary itself, so it cannot rely on the
            // `local` class's trusted-argv0 exemption (§11.2).
            sandbox_class: Some("jail".into()),
            parent_stage: None,
            // A spec-pipeline stage is load-bearing, not optional side work:
            // the case engine only guarantees a required item runs, and only
            // terminates the case on a required item's failure (§10.7 "the
            // pipeline routes back to the spec/generator stage ... until
            // fixed"). An unrequired item can be silently skipped by
            // `can_auto_complete` before it ever activates.
            markers: ItemMarkers {
                required: true,
                ..Default::default()
            },
            max_instances: 1,
            depends_on: vec![],
            environment: None,
            proposed_by: None,
        });
    }
    Ok(CasePlan {
        version: RECORD_VERSION.into(),
        plan_id: format!("plan_{case_id}"),
        case_id: case_id.into(),
        run_id: run_id.into(),
        intent_id: intent_id.into(),
        items,
        template_ref: None,
        job_contract_ref: None,
    })
}

fn file_plan(
    executable: &str,
    path: &str,
    content: &str,
    validate_path: &str,
) -> (Vec<Operation>, SettlementCriteria) {
    (
        vec![
            Operation::WriteFile {
                path: path.into(),
                content_hint: content.into(),
            },
            Operation::ExecuteCommand {
                // Minimum §11.2: exact argv is [current_exe, "validate", <file>].
                argv: vec![executable.into(), "validate".into(), validate_path.into()],
                cwd: ".".into(),
            },
        ],
        SettlementCriteria {
            require_exit_zero: true,
            required_artifacts: vec!["model.sea".into()],
            stdout_must_contain: Some("sea-forge: model valid".into()),
            ..Default::default()
        },
    )
}
