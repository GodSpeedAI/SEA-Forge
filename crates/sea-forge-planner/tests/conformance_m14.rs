use sea_forge_core::types::{
    EntryCriteriaMode, ItemKind, ItemMarkers, Sentry, SentryPredicate, SentryTrigger,
    SettlementCriteria, TraceEvent, TraceKind,
};
use sea_forge_planner::case_engine::evaluate_sentries;
use sea_forge_planner::templates::{
    concurrent_agents_template, instantiate, sequential_agents_template, RepeatEntry, RepeatedItem,
    TemplateItem, TemplateOperation, TemplatePlan,
};
use std::collections::BTreeMap;

fn settlement_event(item: &str, status: &str) -> TraceEvent {
    TraceEvent {
        version: "0.2".into(),
        event_id: format!("evt_{item}_{status}"),
        run_id: "run_test".into(),
        plan_item_id: Some(item.into()),
        kind: TraceKind::SettlementRecorded,
        actor_id: "test".into(),
        timestamp: "2026-07-21T00:00:00Z".into(),
        payload: serde_json::json!({"status": status}),
        cell_id: None,
    }
}

fn agent_task_body() -> TemplateItem {
    TemplateItem {
        plan_item_id: "".into(),
        name: "branch".into(),
        operations: vec![TemplateOperation::AgentTask {
            endpoint_ref: "agent:local".into(),
            instruction: "do ${task}".into(),
            max_turns: 1,
            token_budget: None,
        }],
        settlement_criteria: SettlementCriteria::default(),
        item_kind: ItemKind::AgentTask,
        sandbox_class: None,
        markers: ItemMarkers::default(),
        max_instances: 1,
        environment: None,
        entry_criteria: vec![],
        entry_criteria_mode: EntryCriteriaMode::Any,
        exit_criteria: vec![],
        parent_stage: None,
        depends_on: vec![],
    }
}

fn plan_with_repeated(entries: Vec<RepeatEntry>) -> sea_forge_planner::templates::PlanTemplate {
    sea_forge_planner::templates::PlanTemplate {
        name: "t14_fixture".into(),
        version: "0.1.0".into(),
        description: "fixture".into(),
        parameters: BTreeMap::new(),
        origin_refs: vec![],
        job_contract: None,
        plan: TemplatePlan {
            items: vec![],
            repeated: vec![RepeatedItem {
                id_prefix: "branch".into(),
                item: agent_task_body(),
                entries,
            }],
        },
    }
}

#[test]
fn t14_0_repeated_item_expands_to_deterministic_ids() {
    let tmpl = plan_with_repeated(vec![
        RepeatEntry {
            key: "a".into(),
            params: BTreeMap::new(),
            entry_criteria: vec![],
        },
        RepeatEntry {
            key: "b".into(),
            params: BTreeMap::new(),
            entry_criteria: vec![],
        },
        RepeatEntry {
            key: "c".into(),
            params: BTreeMap::new(),
            entry_criteria: vec![],
        },
    ]);
    let plan = instantiate(&tmpl, &BTreeMap::new(), "case_1", "run_1", "intent_1").unwrap();
    assert_eq!(
        plan.items
            .iter()
            .map(|i| i.plan_item_id.clone())
            .collect::<Vec<_>>(),
        vec!["branch_a", "branch_b", "branch_c"]
    );
}

#[test]
fn t14_0_duplicate_entry_keys_reject() {
    let tmpl = plan_with_repeated(vec![
        RepeatEntry {
            key: "a".into(),
            params: BTreeMap::new(),
            entry_criteria: vec![],
        },
        RepeatEntry {
            key: "a".into(),
            params: BTreeMap::new(),
            entry_criteria: vec![],
        },
    ]);
    let err = instantiate(&tmpl, &BTreeMap::new(), "case_1", "run_1", "intent_1").unwrap_err();
    assert_eq!(err.class(), "schema_error");
}

#[test]
fn t14_0_expansion_over_hard_max_rejects() {
    let entries = (0..sea_forge_planner::templates::MAX_REPEATED_ENTRIES + 1)
        .map(|i| RepeatEntry {
            key: format!("k{i}"),
            params: BTreeMap::new(),
            entry_criteria: vec![],
        })
        .collect();
    let tmpl = plan_with_repeated(entries);
    let err = instantiate(&tmpl, &BTreeMap::new(), "case_1", "run_1", "intent_1").unwrap_err();
    assert_eq!(err.class(), "schema_error");
}

#[test]
fn t14_0_agent_task_endpoint_ref_forbids_substitution() {
    let mut body = agent_task_body();
    body.operations = vec![TemplateOperation::AgentTask {
        endpoint_ref: "${ep}".into(),
        instruction: "do it".into(),
        max_turns: 1,
        token_budget: None,
    }];
    let tmpl = plan_with_repeated(vec![RepeatEntry {
        key: "a".into(),
        params: BTreeMap::new(),
        entry_criteria: vec![],
    }]);
    let mut tmpl = tmpl;
    tmpl.plan.repeated[0].item = body;
    let err = instantiate(&tmpl, &BTreeMap::new(), "case_1", "run_1", "intent_1").unwrap_err();
    assert_eq!(err.class(), "schema_error");
}

#[test]
fn t14_0_any_of_legacy_behavior_unchanged() {
    let items = vec![sea_forge_core::types::PlanItem {
        plan_item_id: "gate".into(),
        name: "gate".into(),
        operations: vec![],
        entry_criteria: vec![
            Sentry {
                on: SentryTrigger {
                    source: "a".into(),
                    event: "settlement_status".into(),
                },
                if_predicate: Some(SentryPredicate::SettlementStatus {
                    status: "accepted".into(),
                }),
            },
            Sentry {
                on: SentryTrigger {
                    source: "b".into(),
                    event: "settlement_status".into(),
                },
                if_predicate: Some(SentryPredicate::SettlementStatus {
                    status: "accepted".into(),
                }),
            },
        ],
        exit_criteria: vec![],
        settlement_criteria: SettlementCriteria::default(),
        settlement_criteria_ref: None,
        item_kind: ItemKind::Milestone,
        sandbox_class: None,
        parent_stage: None,
        markers: ItemMarkers::default(),
        max_instances: 1,
        depends_on: vec![],
        environment: None,
        entry_criteria_mode: EntryCriteriaMode::Any,
        proposed_by: None,
    }];
    let ws = std::collections::HashSet::new();
    let activated = evaluate_sentries(&items, &[settlement_event("a", "accepted")], &ws);
    assert!(activated.contains(&"gate".to_string()));
}

#[test]
fn t14_0_all_of_waits_for_every_named_source() {
    let items = vec![sea_forge_core::types::PlanItem {
        plan_item_id: "gate".into(),
        name: "gate".into(),
        operations: vec![],
        entry_criteria: vec![
            Sentry {
                on: SentryTrigger {
                    source: "a".into(),
                    event: "settlement_status".into(),
                },
                if_predicate: Some(SentryPredicate::SettlementStatus {
                    status: "accepted".into(),
                }),
            },
            Sentry {
                on: SentryTrigger {
                    source: "b".into(),
                    event: "settlement_status".into(),
                },
                if_predicate: Some(SentryPredicate::SettlementStatus {
                    status: "accepted".into(),
                }),
            },
        ],
        exit_criteria: vec![],
        settlement_criteria: SettlementCriteria::default(),
        settlement_criteria_ref: None,
        item_kind: ItemKind::Milestone,
        sandbox_class: None,
        parent_stage: None,
        markers: ItemMarkers::default(),
        max_instances: 1,
        depends_on: vec![],
        environment: None,
        entry_criteria_mode: EntryCriteriaMode::All,
        proposed_by: None,
    }];
    let ws = std::collections::HashSet::new();
    let only_a = evaluate_sentries(&items, &[settlement_event("a", "accepted")], &ws);
    assert!(!only_a.contains(&"gate".to_string()));
    let both = evaluate_sentries(
        &items,
        &[
            settlement_event("a", "accepted"),
            settlement_event("b", "accepted"),
        ],
        &ws,
    );
    assert!(both.contains(&"gate".to_string()));
}

#[test]
fn t14_1_sequential_agents_instantiation_is_deterministic_and_settles_in_order() {
    let params = BTreeMap::new();
    let a = instantiate(
        &sequential_agents_template(),
        &params,
        "case_1",
        "run_1",
        "intent_1",
    )
    .unwrap();
    let b = instantiate(
        &sequential_agents_template(),
        &params,
        "case_1",
        "run_1",
        "intent_1",
    )
    .unwrap();
    assert_eq!(
        serde_json::to_vec(&a).unwrap(),
        serde_json::to_vec(&b).unwrap(),
        "same template + same params must yield identical plan"
    );
    let ids: Vec<&str> = a.items.iter().map(|i| i.plan_item_id.as_str()).collect();
    assert!(
        ids.len() >= 2,
        "sequential template must expand to >=2 steps"
    );
    let second = a.items.iter().find(|i| i.plan_item_id == ids[1]).unwrap();
    assert!(
        second.entry_criteria.iter().any(|s| s.on.source == ids[0]),
        "step 2 must gate on step 1's settlement"
    );
}

#[test]
fn t14_2_concurrent_agents_rollup_fires_only_when_all_n_branches_settle() {
    let plan = instantiate(
        &concurrent_agents_template(),
        &BTreeMap::new(),
        "case_1",
        "run_1",
        "intent_1",
    )
    .unwrap();
    let rollup = plan
        .items
        .iter()
        .find(|i| i.item_kind == ItemKind::Milestone)
        .expect("concurrent_agents must have a rollup milestone");
    assert_eq!(rollup.entry_criteria_mode, EntryCriteriaMode::All);
    let branch_ids: Vec<&str> = plan
        .items
        .iter()
        .filter(|i| i.item_kind == ItemKind::AgentTask)
        .map(|i| i.plan_item_id.as_str())
        .collect();
    assert!(
        branch_ids.len() >= 2,
        "concurrent template needs >=2 branches"
    );

    let ws = std::collections::HashSet::new();
    let mut events: Vec<TraceEvent> = branch_ids[..branch_ids.len() - 1]
        .iter()
        .map(|id| settlement_event(id, "accepted"))
        .collect();
    let partial = evaluate_sentries(&plan.items, &events, &ws);
    assert!(!partial.contains(&rollup.plan_item_id));

    events.push(settlement_event(
        branch_ids[branch_ids.len() - 1],
        "accepted",
    ));
    let full = evaluate_sentries(&plan.items, &events, &ws);
    assert!(full.contains(&rollup.plan_item_id));
}

#[test]
fn t14_3_one_branch_rejected_plus_unrelated_rejection_rollup_never_fires() {
    let plan = instantiate(
        &concurrent_agents_template(),
        &BTreeMap::new(),
        "case_1",
        "run_1",
        "intent_1",
    )
    .unwrap();
    let rollup = plan
        .items
        .iter()
        .find(|i| i.item_kind == ItemKind::Milestone)
        .unwrap();
    let branch_ids: Vec<&str> = plan
        .items
        .iter()
        .filter(|i| i.item_kind == ItemKind::AgentTask)
        .map(|i| i.plan_item_id.as_str())
        .collect();
    let ws = std::collections::HashSet::new();
    let mut events: Vec<TraceEvent> = branch_ids[1..]
        .iter()
        .map(|id| settlement_event(id, "accepted"))
        .collect();
    events.push(settlement_event(branch_ids[0], "rejected"));
    events.push(settlement_event("unrelated_item", "rejected"));
    let activated = evaluate_sentries(&plan.items, &events, &ws);
    assert!(!activated.contains(&rollup.plan_item_id));
}
