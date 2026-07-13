use sea_forge_core::{errors::ForgeError, types::*, RECORD_VERSION};
use sea_forge_domain::{self as domain, IntentPattern};

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
        },
    )
}
