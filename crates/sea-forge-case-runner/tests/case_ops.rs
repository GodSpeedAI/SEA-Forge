//! Library tests for the shared case-mutation operations (T04 step 1).
//!
//! `case_ops` was extracted verbatim from the CLI so the CLI and the SFWP
//! server call one governed implementation. The CLI's conformance suites
//! (e.g. `conformance_m15.rs` t15.4) already prove the behavior end-to-end
//! through the binary; these tests pin the same behaviors at the library
//! boundary the server now calls in-process, on fixtures small enough to
//! attribute a failure to the library rather than to process plumbing.

use sea_forge_case_runner::case_ops;
use sea_forge_core::errors::ForgeError;
use sea_forge_core::types::*;
use sea_forge_core::RECORD_VERSION;
use sea_forge_ledger::LedgerStream;
use sea_forge_planner::{compute_criteria_sha256, compute_record_hash};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

fn temp_root(_tag: &str) -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

/// The policy spelling the CLI conformance suites use: a minimal bundle whose
/// rules allow exactly the operations under test.
fn write_policy(root: &Path, rules: &str) -> PathBuf {
    let path = root.join("policy.yaml");
    fs::write(&path, format!("version: \"0.1\"\nrules:\n{rules}")).unwrap();
    path
}

fn intent() -> Intent {
    Intent {
        intent_id: "int_1".into(),
        summary: "fixture".into(),
        actor_id: "operator_runner".into(),
        process_id: "proc_1".into(),
        created_at: "2026-09-23T00:00:00Z".into(),
    }
}

fn case_record(case_id: &str, state: CaseState) -> Case {
    Case {
        version: RECORD_VERSION.into(),
        case_id: case_id.into(),
        intent: intent(),
        state,
        plan_ref: "plan.json".into(),
        run_ids: vec!["run_1".into()],
        stages: vec![],
        close_reason: None,
        created_at: "2026-09-23T00:00:00Z".into(),
        closed_at: None,
    }
}

fn plan(case_id: &str, items: Vec<PlanItem>) -> CasePlan {
    CasePlan {
        version: "0.2".into(),
        plan_id: "plan_1".into(),
        case_id: case_id.into(),
        run_id: "run_1".into(),
        intent_id: "int_1".into(),
        items,
        template_ref: None,
        job_contract_ref: None,
    }
}

fn item(id: &str, depends_on: Vec<&str>) -> PlanItem {
    PlanItem {
        plan_item_id: id.into(),
        name: id.into(),
        operations: vec![],
        entry_criteria: vec![],
        entry_criteria_mode: EntryCriteriaMode::Any,
        exit_criteria: vec![],
        settlement_criteria: SettlementCriteria::default(),
        settlement_criteria_ref: None,
        // A human task keeps the fixture minimal: sandboxed tasks must
        // declare operations, which these fixtures do not need.
        item_kind: ItemKind::HumanTask,
        sandbox_class: None,
        parent_stage: None,
        markers: ItemMarkers::default(),
        max_instances: 1,
        depends_on: depends_on.into_iter().map(str::to_owned).collect(),
        environment: None,
        proposed_by: None,
    }
}

/// Materialize the case-directory view the library reads
/// (`cases/<id>/{case.json,plan.json,case-events.jsonl}`).
fn write_case_state(root: &Path, case_id: &str, case: &Case, plan: &CasePlan) {
    let dir = root.join("cases").join(case_id);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("case.json"),
        serde_json::to_vec_pretty(case).unwrap(),
    )
    .unwrap();
    fs::write(
        dir.join("plan.json"),
        serde_json::to_vec_pretty(plan).unwrap(),
    )
    .unwrap();
    fs::write(dir.join("case-events.jsonl"), b"").unwrap();
}

fn case_events(root: &Path, case_id: &str) -> Vec<serde_json::Value> {
    fs::read_to_string(root.join("cases").join(case_id).join("case-events.jsonl"))
        .unwrap()
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn ledger_payloads(root: &Path, case_id: &str) -> Vec<(String, serde_json::Value)> {
    LedgerStream::open(root, format!("case-{case_id}"), "fixture")
        .unwrap()
        .read_entries()
        .unwrap()
        .into_iter()
        .map(|entry| (entry.record_kind, entry.payload))
        .collect()
}

/// A pending approval fixture committed to the case ledger, bound to an
/// escalated decision and a committed criteria record exactly the way the
/// dispatch/escalation paths do (see `case_dispatch.rs` / `manager.rs`).
fn approval_fixture(root: &Path, case_id: &str, item: PlanItem) -> (String, PathBuf) {
    let policy = write_policy(
        root,
        "  - name: allow-approval-resolution\n    verdict: allow\n    actor_role: operator\n    operation_kind: approval_resolution\n",
    );
    let stream = LedgerStream::open(root, format!("case-{case_id}"), "operator_runner").unwrap();

    let criteria = SettlementCriteria::default();
    let criteria_record = SettlementCriteriaRecord {
        version: RECORD_VERSION.into(),
        criteria_id: "crit_1".into(),
        criteria_sha256: compute_criteria_sha256(&criteria).unwrap(),
        criteria: criteria.clone(),
        origin_refs: vec![],
        derivation: CriteriaDerivation {
            method: DerivationMethod::ImplementationDefined,
            actor_ref: "operator_runner".into(),
            producer_ref: None,
            rationale: "fixture".into(),
        },
        declared_at: "2026-09-23T00:00:00Z".into(),
        criteria_record_hash: String::new(),
    };
    let criteria_record = SettlementCriteriaRecord {
        criteria_record_hash: compute_record_hash(&criteria_record).unwrap(),
        ..criteria_record
    };
    stream
        .commit_typed("settlement_criteria", vec![], &criteria_record, vec![])
        .unwrap();

    let action_request = AuthorityRequest {
        schema_version: "1".into(),
        action_id: "act_1".into(),
        correlation_id: "corr_1".into(),
        timestamp_utc: "2026-09-23T00:00:00Z".into(),
        actor: json!({"actor_id": "operator_runner"}),
        action: json!({"resource_type": "write_file", "resource_id": "out.txt"}),
        context: json!({"run_id": "run_1", "plan_item_id": item.plan_item_id}),
        evidence: json!([]),
    };
    let decision = AuthorityDecision {
        version: RECORD_VERSION.into(),
        decision_id: "auth_01".into(),
        run_id: "run_1".into(),
        plan_item_id: item.plan_item_id.clone(),
        action_id: "act_1".into(),
        correlation_id: "corr_1".into(),
        operation: AuthorityAction::WriteFile {
            path: "out.txt".into(),
            content_hint: "done".into(),
        },
        outcome: Verdict::Escalate,
        verdict: Verdict::Escalate,
        normalized_disposition: NormalizedDisposition::Escalate,
        matched_rule: Some("escalate-write".into()),
        reason_codes: vec![],
        reason: "fixture escalation".into(),
        policy_refs: vec![],
        required_next_steps: vec![],
        identity_binding: IdentityBinding {
            identity_id: None,
            principal: "operator_runner".into(),
            roles: vec![ActorRole::Operator],
            actor_type: ActorType::Human,
            binding_resolution: BindingResolution::Exact,
            identity_binding_source: "fixture".into(),
            source: None,
            sponsor: None,
            issued_at: None,
            expires_at: None,
            identity_binding_hash: None,
        },
        determinism: Determinism {
            policy_bundle_hash: "fixture".into(),
            action_request_hash: sea_forge_ledger::types::hash_canonical(&action_request).unwrap(),
            identity_binding_hash: "fixture".into(),
        },
        action_request,
        audit_record: AuditRecord {
            engine: "fixture".into(),
            disposition: "escalate".into(),
            subject: item.plan_item_id.clone(),
            reason: "fixture escalation".into(),
            evidence_refs: vec![],
            recorded_at: "2026-09-23T00:00:00Z".into(),
            decision_id: Some("auth_01".into()),
            case_id: Some(case_id.into()),
            run_id: Some("run_1".into()),
            policy_bundle_hash: None,
            action_request_hash: None,
            identity_binding_hash: None,
        },
        decided_at: "2026-09-23T00:00:00Z".into(),
        candidate_verdicts: vec![],
        winning_source: None,
        precedence_reason: None,
        sandbox_class_granted: None,
        approval_request_id: Some("apr_1".into()),
        opaque_constraint_id: None,
        boundary_constraints: Default::default(),
        compensating_controls: vec![],
    };
    stream
        .commit_typed("authority_decision", vec![], &decision, vec![])
        .unwrap();

    let approval = ApprovalRequest {
        version: RECORD_VERSION.into(),
        approval_id: "apr_1".into(),
        run_id: "run_1".into(),
        case_id: case_id.into(),
        decision_id: "auth_01".into(),
        plan_item_id: item.plan_item_id.clone(),
        criteria_ref: Some(criteria_record.criteria_id.clone()),
        criteria_sha256: Some(criteria_record.criteria_sha256.clone()),
        criteria_record_hash: Some(criteria_record.criteria_record_hash.clone()),
        job_contract_ref: None,
        requested_at: "2026-09-23T00:00:00Z".into(),
        expires_at: "2099-01-01T00:00:00Z".into(),
        status: ApprovalStatus::Pending,
        resolved_by: None,
        resolved_at: None,
        note: None,
    };
    stream
        .commit_typed("approval_request", vec![], &approval, vec![])
        .unwrap();

    write_case_state(
        root,
        case_id,
        &case_record(case_id, CaseState::Active),
        &plan(case_id, vec![item]),
    );
    ("apr_1".into(), policy)
}

/// Reopen round-trips a closed case back to Active and leaves the reopen in
/// both durable truth stores: the case event log and the case ledger.
#[test]
fn reopen_reactivates_a_closed_case_and_records_the_event() {
    let root = temp_root("reopen-happy");
    let policy = write_policy(
        root.path(),
        "  - name: allow-reopen\n    verdict: allow\n    actor_role: operator\n    operation_kind: case_reopen\n",
    );
    let case_id = "case_reopen_1";
    let mut case = case_record(case_id, CaseState::Completed);
    case.close_reason = Some("done".into());
    case.closed_at = Some("2026-09-23T01:00:00Z".into());
    write_case_state(
        root.path(),
        case_id,
        &case,
        &plan(case_id, vec![item("a", vec![])]),
    );

    case_ops::reopen(root.path(), &policy, "operator_local", case_id).unwrap();

    let reopened: Case = serde_json::from_slice(
        &fs::read(root.path().join("cases").join(case_id).join("case.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(reopened.state, CaseState::Active);
    assert_eq!(reopened.close_reason, None);
    assert_eq!(reopened.closed_at, None);

    let events = case_events(root.path(), case_id);
    assert_eq!(events.len(), 1, "reopen must append exactly one event");
    assert_eq!(events[0]["kind"], "case_reopened");
    assert_eq!(events[0]["actor_id"], "operator_local");

    let kinds: Vec<String> = ledger_payloads(root.path(), case_id)
        .into_iter()
        .map(|(kind, _)| kind)
        .collect();
    assert!(
        kinds.iter().any(|kind| kind == "case_event"),
        "the reopen event must also be committed to the case ledger: {kinds:?}"
    );
}

/// The reopen reason the requester gave is part of the durable record (it was being dropped on the
/// way to the kernel); a blank reason is recorded as absent, never as an empty string.
#[test]
fn reopen_records_the_requesters_reason_on_the_event() {
    let root = temp_root("reopen-reason");
    let policy = write_policy(
        root.path(),
        "  - name: allow-reopen\n    verdict: allow\n    actor_role: operator\n    operation_kind: case_reopen\n",
    );
    for (case_id, reason, expected) in [
        (
            "case_reopen_r1",
            Some("  late evidence arrived  "),
            Some("late evidence arrived"),
        ),
        ("case_reopen_r2", Some("   "), None),
        ("case_reopen_r3", None, None),
    ] {
        let mut case = case_record(case_id, CaseState::Completed);
        case.close_reason = Some("done".into());
        write_case_state(
            root.path(),
            case_id,
            &case,
            &plan(case_id, vec![item("a", vec![])]),
        );
        case_ops::reopen_with_reason(
            root.path(),
            &policy,
            "operator_local",
            case_id,
            reason,
            &mut |_| {},
        )
        .unwrap();
        let events = case_events(root.path(), case_id);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["kind"], "case_reopened");
        match expected {
            Some(expected) => {
                assert_eq!(events[0]["payload"]["reason"], expected);
                assert_eq!(events[0]["payload"]["requested_by"], "operator_local");
            }
            None => assert!(
                events[0]["payload"].get("reason").is_none(),
                "no reason must be recorded as absent: {}",
                events[0]["payload"]
            ),
        }
    }
}

/// An open case is not reopenable — the guard moved with the mutation.
#[test]
fn reopen_refuses_a_case_that_is_not_closed() {
    let root = temp_root("reopen-open");
    let policy = write_policy(
        root.path(),
        "  - name: allow-reopen\n    verdict: allow\n    actor_role: operator\n    operation_kind: case_reopen\n",
    );
    let case_id = "case_reopen_2";
    write_case_state(
        root.path(),
        case_id,
        &case_record(case_id, CaseState::Active),
        &plan(case_id, vec![item("a", vec![])]),
    );

    let error = case_ops::reopen(root.path(), &policy, "operator_local", case_id).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("only closed cases can be reopened"),
        "wrong refusal: {error}"
    );
    assert!(case_events(root.path(), case_id).is_empty());
}

/// Proposing an item whose dependency graph closes a cycle is refused before
/// any write: plan.json is byte-identical and no PlanMutated event exists.
#[test]
fn propose_item_refuses_a_cycle_and_leaves_the_plan_untouched() {
    let root = temp_root("propose-cycle");
    let policy = write_policy(
        root.path(),
        "  - name: allow-propose\n    verdict: allow\n    actor_role: operator\n    operation_kind: discretionary_task_add\n",
    );
    let case_id = "case_cycle_1";
    write_case_state(
        root.path(),
        case_id,
        &case_record(case_id, CaseState::Active),
        &plan(case_id, vec![item("a", vec![]), item("b", vec!["a"])]),
    );
    let plan_path = root.path().join("cases").join(case_id).join("plan.json");
    let before = fs::read(&plan_path).unwrap();

    // A self-dependency closes a cycle in the sentry graph the proposal
    // validation walks (`validate_proposal` → `plan_cycle_error`).
    let error = case_ops::propose_item(
        root.path(),
        &policy,
        "operator_local",
        case_id,
        item("c", vec!["c"]),
    )
    .unwrap_err();
    match error {
        ForgeError::Plan { class, .. } => assert_eq!(class, "plan_cycle_error"),
        other => panic!("expected a plan cycle error, got: {other}"),
    }

    assert_eq!(
        fs::read(&plan_path).unwrap(),
        before,
        "a refused proposal must not mutate plan.json"
    );
    assert!(
        case_events(root.path(), case_id)
            .iter()
            .all(|event| event["kind"] != "plan_mutated"),
        "a refused proposal must not emit PlanMutated"
    );
}

/// The structural SoD guard (§10.3): the actor who proposed a discretionary
/// item cannot resolve the approval gating it, and the refusal leaves no
/// resolution anywhere — no ledger record, no approvals journal line.
#[test]
fn the_proposer_cannot_resolve_its_own_items_approval() {
    let root = temp_root("approval-sod");
    let case_id = "case_sod_1";
    let mut proposed = item("item-1", vec![]);
    proposed.proposed_by = Some("thoth".into());
    let (approval_id, policy) = approval_fixture(root.path(), case_id, proposed);

    let error = case_ops::resolve_approval(
        root.path(),
        &policy,
        case_id,
        &approval_id,
        "thoth",
        None,
        true,
    )
    .unwrap_err();
    match error {
        ForgeError::Plan { class, .. } => assert_eq!(class, "sod_violation"),
        other => panic!("expected an SoD refusal, got: {other}"),
    }

    let payloads = ledger_payloads(root.path(), case_id);
    assert!(
        payloads
            .iter()
            .all(|(kind, _)| kind != "approval_resolution"),
        "a refused resolution must not be committed"
    );
    assert!(
        !root.path().join("approvals.jsonl").exists(),
        "a refused resolution must not write the approvals journal"
    );
}

/// The grant-minting half: an uninvolved actor resolves the approval, the
/// authority engine commits a fresh Allow decision for the resolution, and the
/// journal records who resolved it.
#[test]
fn an_uninvolved_actor_resolves_the_approval_and_mints_the_authority_decision() {
    let root = temp_root("approval-grant");
    let case_id = "case_grant_1";
    let (approval_id, policy) = approval_fixture(root.path(), case_id, item("item-1", vec![]));

    let resolved = case_ops::resolve_approval(
        root.path(),
        &policy,
        case_id,
        &approval_id,
        "operator_reviewer",
        Some("looks fine"),
        true,
    )
    .unwrap();

    assert_eq!(resolved.status, ApprovalStatus::Approved);
    assert_eq!(resolved.resolved_by.as_deref(), Some("operator_reviewer"));
    assert_eq!(resolved.note.as_deref(), Some("looks fine"));

    // The journal write the workbench inbox folds.
    let journal: Vec<serde_json::Value> = fs::read_to_string(root.path().join("approvals.jsonl"))
        .unwrap()
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(journal.len(), 1, "one journal line per resolution");
    assert_eq!(journal[0]["approval_id"], approval_id);
    assert_eq!(journal[0]["status"], "approved");
    assert_eq!(journal[0]["resolved_by"], "operator_reviewer");

    // The mint: a second authority decision, this one Allow, committed for
    // the resolution action — alongside the resolution record itself.
    let decisions: Vec<serde_json::Value> = ledger_payloads(root.path(), case_id)
        .into_iter()
        .filter(|(kind, _)| kind == "authority_decision")
        .map(|(_, payload)| payload)
        .collect();
    assert_eq!(
        decisions.len(),
        2,
        "escalation + minted resolution decision"
    );
    assert_eq!(decisions[1]["verdict"], "allow");
    assert_eq!(
        decisions[1]["action_request"]["action"]["resource_type"],
        "approval_resolution"
    );

    // A resolved approval is terminal: re-resolving is refused, not appended.
    let again = case_ops::resolve_approval(
        root.path(),
        &policy,
        case_id,
        &approval_id,
        "operator_reviewer",
        None,
        false,
    )
    .unwrap_err();
    assert!(
        again.to_string().contains("MUST NOT be re-resolved"),
        "wrong refusal: {again}"
    );
}
