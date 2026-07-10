use crate::{
    domain::{self, IntentPattern},
    errors::ForgeError,
    types::*,
    RECORD_VERSION,
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
                required_artifacts: vec![],
                stdout_must_contain: None,
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
            settlement_criteria,
        }],
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
                argv: vec![executable.into(), "validate".into(), validate_path.into()],
                cwd: ".".into(),
            },
        ],
        SettlementCriteria {
            require_exit_zero: true,
            required_artifacts: vec!["model.sea".into()],
            stdout_must_contain: Some("sea-forge: model valid".into()),
        },
    )
}
