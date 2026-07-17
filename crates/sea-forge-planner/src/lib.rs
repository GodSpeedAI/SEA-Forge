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
    instantiate, load, load_pinned, sea_model_demo_template, store_builtin, PlanTemplate,
};

pub const DEMO_MODEL: &str = r#"{"domain": "demo", "entities": [{"name": "Sample"}]}"#;

pub fn plan(
    intent: &Intent,
    case_id: &str,
    run_id: &str,
    executable: &str,
    state_root: &std::path::Path,
) -> Result<CasePlan, ForgeError> {
    let (operations, settlement_criteria) = match domain::interpret(&intent.summary)? {
        IntentPattern::Demo => {
            file_plan(executable, state_root, "model.sea", DEMO_MODEL, "model.sea")
        }
        IntentPattern::GeneratedZone => file_plan(
            executable,
            state_root,
            "src/gen/model.sea",
            DEMO_MODEL,
            "model.sea",
        ),
        IntentPattern::FalseSuccess => {
            file_plan(executable, state_root, "other.sea", DEMO_MODEL, "other.sea")
        }
        IntentPattern::Nonzero => file_plan(executable, state_root, "model.sea", "{}", "model.sea"),
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
        }],
        template_ref: None,
        job_contract_ref: None,
    })
}

fn file_plan(
    executable: &str,
    state_root: &std::path::Path,
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
                argv: vec![
                    executable.into(),
                    "validate".into(),
                    validate_path.into(),
                    "--root".into(),
                    state_root.to_string_lossy().into_owned(),
                    "--policy".into(),
                    state_root
                        .join("authority/active-policy.json")
                        .to_string_lossy()
                        .into_owned(),
                ],
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
