use sea_forge_core::types::{
    CasePlan, ItemKind, ItemMarkers, Operation, PlanItem, Sentry, SentryPredicate, SentryTrigger,
    SettlementCriteria, TraceEvent, TraceKind,
};
use sea_forge_planner::case_engine::{evaluate_sentries, validate_proposal};

fn agent_item() -> PlanItem {
    PlanItem {
        plan_item_id: "agent".into(),
        name: "agent_task".into(),
        operations: vec![Operation::AgentTask {
            endpoint_ref: "local-test".into(),
            instruction: "publish proof".into(),
            max_turns: 1,
            token_budget: None,
            response_schema: None,
            transcript_retention: None,
        }],
        entry_criteria: vec![],
        exit_criteria: vec![],
        settlement_criteria: SettlementCriteria::default(),
        settlement_criteria_ref: None,
        item_kind: ItemKind::AgentTask,
        sandbox_class: None,
        parent_stage: None,
        markers: ItemMarkers::default(),
        max_instances: 1,
        depends_on: vec![],
        environment: None,
    }
}

fn downstream_item() -> PlanItem {
    PlanItem {
        plan_item_id: "downstream".into(),
        name: "sandboxed_task".into(),
        operations: vec![Operation::WriteFile {
            path: "out.txt".into(),
            content_hint: "done".into(),
        }],
        entry_criteria: vec![Sentry {
            on: SentryTrigger {
                source: "agent".into(),
                event: "settlement_status".into(),
            },
            if_predicate: Some(SentryPredicate::SettlementStatus {
                status: "accepted".into(),
            }),
        }],
        exit_criteria: vec![],
        settlement_criteria: SettlementCriteria::default(),
        settlement_criteria_ref: None,
        item_kind: ItemKind::SandboxedTask,
        sandbox_class: None,
        parent_stage: None,
        markers: ItemMarkers::default(),
        max_instances: 1,
        depends_on: vec![],
        environment: None,
    }
}

fn settlement_event(item: &str, status: &str) -> TraceEvent {
    TraceEvent {
        version: "0.2".into(),
        event_id: format!("evt_{item}_{status}"),
        run_id: "run_test".into(),
        plan_item_id: Some(item.into()),
        kind: TraceKind::SettlementRecorded,
        actor_id: "test".into(),
        timestamp: "2026-07-20T00:00:00Z".into(),
        payload: serde_json::json!({"status": status}),
        cell_id: None,
    }
}

/// T13.1 (plan-acceptance half): the case engine accepts an `agent_task`
/// item carrying one `Operation::AgentTask`, and a downstream
/// `sandboxed_task` gated by a settlement sentry activates only when the
/// agent item settles `accepted` — never on `rejected`.
#[test]
fn t13_1_agent_task_plan_accepted_and_downstream_sentry_gates_on_settlement() {
    let mut plan = CasePlan {
        version: "0.2".into(),
        plan_id: "plan_test".into(),
        case_id: "case_test".into(),
        run_id: "run_test".into(),
        intent_id: "int_test".into(),
        items: vec![agent_item(), downstream_item()],
        template_ref: None,
        job_contract_ref: None,
    };
    validate_proposal(&mut plan).expect("agent_task plan must be accepted");

    let items = plan.items;
    let ws = std::collections::HashSet::new();

    // Agent settled accepted → downstream activates.
    let accepted = evaluate_sentries(&items, &[settlement_event("agent", "accepted")], &ws);
    assert!(
        accepted.contains(&"downstream".to_string()),
        "downstream must activate on agent accepted settlement"
    );

    // Agent settled rejected → downstream must NOT activate.
    let rejected = evaluate_sentries(&items, &[settlement_event("agent", "rejected")], &ws);
    assert!(
        !rejected.contains(&"downstream".to_string()),
        "downstream must NOT activate on agent rejected settlement"
    );

    // No settlement yet → downstream must NOT activate.
    let none = evaluate_sentries(&items, &[], &ws);
    assert!(
        !none.contains(&"downstream".to_string()),
        "downstream must NOT activate before agent settles"
    );
}

/// An `agent_task` item carrying a non-AgentTask operation is rejected.
#[test]
fn t13_1_agent_task_item_rejects_alien_operation() {
    let mut bad = agent_item();
    bad.operations = vec![Operation::WriteFile {
        path: "x".into(),
        content_hint: "y".into(),
    }];
    let mut plan = CasePlan {
        version: "0.2".into(),
        plan_id: "plan_test".into(),
        case_id: "case_test".into(),
        run_id: "run_test".into(),
        intent_id: "int_test".into(),
        items: vec![bad],
        template_ref: None,
        job_contract_ref: None,
    };
    assert!(validate_proposal(&mut plan).is_err());
}

/// An `agent_task` item with an empty instruction is rejected at validation.
#[test]
fn t13_1_agent_task_item_rejects_empty_instruction() {
    let mut bad = agent_item();
    if let Operation::AgentTask { instruction, .. } = &mut bad.operations[0] {
        *instruction = String::new();
    }
    let mut plan = CasePlan {
        version: "0.2".into(),
        plan_id: "plan_test".into(),
        case_id: "case_test".into(),
        run_id: "run_test".into(),
        intent_id: "int_test".into(),
        items: vec![bad],
        template_ref: None,
        job_contract_ref: None,
    };
    assert!(validate_proposal(&mut plan).is_err());
}
