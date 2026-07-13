use sea_forge_core::types::*;
use sea_forge_planner::case_engine::*;
use std::collections::HashSet;

fn sentry(source: &str, event: &str) -> Sentry {
    Sentry {
        on: SentryTrigger {
            source: source.into(),
            event: event.into(),
        },
        if_predicate: None,
    }
}

fn milestone_item(id: &str, entry: Vec<Sentry>) -> PlanItem {
    PlanItem {
        plan_item_id: id.into(),
        name: id.into(),
        operations: vec![],
        entry_criteria: entry,
        exit_criteria: vec![],
        settlement_criteria: SettlementCriteria {
            require_exit_zero: false,
            required_artifacts: vec![],
            stdout_must_contain: None,
            require_approval: false,
        },
        item_kind: ItemKind::Milestone,
        sandbox_class: None,
        parent_stage: None,
        markers: ItemMarkers {
            required: false,
            repetition: false,
            manual_activation: false,
        },
        max_instances: 1,
    }
}

fn task_item(id: &str, required: bool, entry: Vec<Sentry>, max_instances: u32) -> PlanItem {
    PlanItem {
        plan_item_id: id.into(),
        name: id.into(),
        operations: vec![],
        entry_criteria: entry,
        exit_criteria: vec![],
        settlement_criteria: SettlementCriteria {
            require_exit_zero: true,
            required_artifacts: vec![],
            stdout_must_contain: None,
            require_approval: false,
        },
        item_kind: ItemKind::SandboxedTask,
        sandbox_class: Some("local".into()),
        parent_stage: None,
        markers: ItemMarkers {
            required,
            repetition: max_instances > 1,
            manual_activation: false,
        },
        max_instances,
    }
}

fn trace_event(
    kind: TraceKind,
    plan_item_id: Option<&str>,
    payload: serde_json::Value,
) -> TraceEvent {
    TraceEvent {
        version: "0.2".into(),
        event_id: format!("evt_{}", kind_to_str(&kind)),
        run_id: "run_test".into(),
        plan_item_id: plan_item_id.map(Into::into),
        kind,
        actor_id: "test".into(),
        timestamp: "2026-07-13T00:00:00Z".into(),
        payload,
    }
}

fn kind_to_str(kind: &TraceKind) -> String {
    format!("{:?}", kind).to_lowercase()
}

#[test]
fn conformance_m2_sentry_scenario_a_b_rep_c_m() {
    // Plan: A (required, no entry), B (required, on A milestone, rep max 2),
    //       C (on B milestone), M (milestone on C milestone)
    let items = vec![
        task_item("A", true, vec![], 1),
        task_item("B", true, vec![sentry("A", "milestone_achieved")], 2),
        task_item("C", false, vec![sentry("B", "milestone_achieved")], 1),
        milestone_item("M", vec![sentry("C", "milestone_achieved")]),
    ];

    // Static satisfiability check
    check_satisfiability(&items).unwrap();

    // Simulate trace events:
    // 1. A settles accepted → milestone for A
    // 2. B instance 1 settles rejected
    // 3. B instance 2 settles rejected → B failed (max reached)
    let events = vec![
        trace_event(
            TraceKind::SettlementRecorded,
            Some("A"),
            serde_json::json!({"status": "accepted"}),
        ),
        trace_event(
            TraceKind::MilestoneAchieved,
            Some("A"),
            serde_json::json!({}),
        ),
        trace_event(
            TraceKind::SettlementRecorded,
            Some("B"),
            serde_json::json!({"status": "rejected"}),
        ),
        trace_event(
            TraceKind::SettlementRecorded,
            Some("B"),
            serde_json::json!({"status": "rejected"}),
        ),
    ];

    let workspace_files = HashSet::new();

    // After A's milestone, B should be activated
    let activated_after_a = evaluate_sentries(&items, &events[..2], &workspace_files);
    assert!(activated_after_a.contains(&"B".to_string()));
    assert!(!activated_after_a.contains(&"C".to_string()));

    // After B's first rejection (no milestone), C should NOT be activated
    let activated_after_b1 = evaluate_sentries(&items, &events[..3], &workspace_files);
    assert!(!activated_after_b1.contains(&"C".to_string()));

    // After B's second rejection (no milestone), C should NOT be activated
    let activated_after_b2 = evaluate_sentries(&items, &events, &workspace_files);
    assert!(!activated_after_b2.contains(&"C".to_string()));
}

#[test]
fn conformance_m2_ledger_replay_reproduces_activations() {
    let items = vec![
        task_item("A", true, vec![], 1),
        task_item("B", true, vec![sentry("A", "milestone_achieved")], 2),
        task_item("C", false, vec![sentry("B", "milestone_achieved")], 1),
        milestone_item("M", vec![sentry("C", "milestone_achieved")]),
    ];

    let events = vec![
        trace_event(
            TraceKind::SettlementRecorded,
            Some("A"),
            serde_json::json!({"status": "accepted"}),
        ),
        trace_event(
            TraceKind::MilestoneAchieved,
            Some("A"),
            serde_json::json!({}),
        ),
        trace_event(
            TraceKind::SettlementRecorded,
            Some("B"),
            serde_json::json!({"status": "rejected"}),
        ),
    ];

    // Replay should produce the same activation sequence
    let replay = replay_activations(&items, &events);

    // After event 1 (A milestone), B should be activated
    let (_, activated_1) = &replay[1];
    assert!(activated_1.contains(&"B".to_string()));

    // After event 2 (B rejected), C should NOT be activated
    let (_, activated_2) = &replay[2];
    assert!(!activated_2.contains(&"C".to_string()));

    // Injecting a synthetic event changes the output (teeth)
    let mut modified_events = events.clone();
    modified_events.insert(
        1,
        trace_event(
            TraceKind::MilestoneAchieved,
            Some("B"),
            serde_json::json!({}),
        ),
    );
    let modified_replay = replay_activations(&items, &modified_events);
    let (_, modified_activated) = &modified_replay[2];
    assert!(
        modified_activated.contains(&"C".to_string()),
        "injected B milestone should activate C — proves replay isn't a stub"
    );
}

#[test]
fn conformance_m2_required_item_failure_terminates_case() {
    let items = vec![
        task_item("A", true, vec![], 1),
        task_item("B", true, vec![sentry("A", "milestone_achieved")], 2),
    ];

    let states = vec![
        ItemState {
            item_id: "A".into(),
            status: ItemStatus::Completed,
            instances: 1,
            failed_instances: 0,
        },
        ItemState {
            item_id: "B".into(),
            status: ItemStatus::Failed,
            instances: 2,
            failed_instances: 2,
        },
    ];

    let failed = required_item_failed(&states, &items);
    assert_eq!(failed, Some("B".to_string()));

    // Case cannot auto-complete
    assert!(!can_auto_complete(&states, &items));
}

#[test]
fn conformance_m2_parked_case_is_not_failure() {
    let items = vec![
        task_item("A", true, vec![], 1),
        task_item("B", false, vec![sentry("A", "milestone_achieved")], 1),
    ];

    // A completed, B available but not yet activated (waiting for sentry)
    let states = vec![
        ItemState {
            item_id: "A".into(),
            status: ItemStatus::Completed,
            instances: 1,
            failed_instances: 0,
        },
        ItemState {
            item_id: "B".into(),
            status: ItemStatus::Available,
            instances: 0,
            failed_instances: 0,
        },
    ];

    // No required item failed
    assert!(required_item_failed(&states, &items).is_none());

    // Case cannot auto-complete yet (B is non-required but still available)
    // But this is a parked state, not a failure
    // The case is still active
}

#[test]
fn conformance_m2_unsatisfiable_sentry_rejected() {
    // Sentry cycle: A depends on B, B depends on A
    let items = vec![
        task_item("A", true, vec![sentry("B", "milestone_achieved")], 1),
        task_item("B", true, vec![sentry("A", "milestone_achieved")], 1),
    ];

    let result = check_satisfiability(&items);
    assert!(result.is_err());
}

#[test]
fn conformance_m2_empty_entry_criteria_activates_immediately() {
    let items = vec![task_item("A", true, vec![], 1)];

    let events: Vec<TraceEvent> = vec![];
    let workspace_files = HashSet::new();

    let activated = evaluate_sentries(&items, &events, &workspace_files);
    assert!(activated.contains(&"A".to_string()));
}
