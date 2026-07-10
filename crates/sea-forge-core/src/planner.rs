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
    let path = match domain::interpret(&intent.summary)? {
        IntentPattern::Demo => "model.sea",
        IntentPattern::GeneratedZone => "src/gen/model.sea",
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
            operations: vec![
                Operation::WriteFile {
                    path: path.into(),
                    content_hint: DEMO_MODEL.into(),
                },
                Operation::ExecuteCommand {
                    argv: vec![executable.into(), "validate".into(), "model.sea".into()],
                    cwd: ".".into(),
                },
            ],
            entry_criteria: vec![],
            settlement_criteria: SettlementCriteria {
                require_exit_zero: true,
                required_artifacts: vec!["model.sea".into()],
                stdout_must_contain: Some("sea-forge: model valid".into()),
            },
        }],
    })
}
