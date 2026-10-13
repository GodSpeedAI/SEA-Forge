//! Approval consumption ordering: a grant request that cannot be minted must
//! not spend the approval. Only a grant that is actually returned may consume
//! the approval's idempotency key.

use sea_forge_authority::{
    ActionGrant, AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine,
};
use sea_forge_core::{
    errors::ForgeError,
    types::{
        Actor, ActorRole, ApprovalRequest, ApprovalStatus, AuthorityAction, AuthorityDecision,
        CriteriaDerivation, DerivationMethod, OriginRef, OriginRefKind, OriginRole,
        SettlementCriteria, SettlementCriteriaRecord, Verdict,
    },
};
use sea_forge_ledger::{types::hash_canonical, CommittedRecordRef, LedgerStream};
use std::{path::Path, sync::Barrier, thread};

const REQUESTER: &str = "operator_local";
const RESOLVER: &str = "security_officer";
const CASE: &str = "case_test";
const RUN: &str = "run_20260710T120000Z_abcdef";
const PLAN_ITEM: &str = "item_01";

const GATED_WRITE: &str = "version: \"0.1\"\nrules:\n  \
  - name: gated_write\n    verdict: allow\n    actor_role: operator\n    operation_kind: write_file\n    requires_approval: true\n";
const SOD_WRITE: &str = "version: \"0.1\"\nidentity_bindings:\n  \
  - principal: operator_local\n    actor_type: human\n    role: operator\n  \
  - principal: security_officer\n    actor_type: human\n    role: R-SO\n  \
  - principal: wrong_resolver\n    actor_type: human\n    role: R-DEV\n\
sod_rules:\n  - name: write_requires_security_officer\n    requester_role: operator\n    approver_role: R-SO\n    operation_kind: write_file\n\
rules:\n  - name: gated_write\n    verdict: allow\n    actor_role: operator\n    operation_kind: write_file\n    requires_approval: true\n";
const ASSURED_WRITE: &str = "version: \"0.1\"\nintegrity_ledger:\n  required_for_side_effects: true\n\
rules:\n  - name: gated_write\n    verdict: allow\n    actor_role: operator\n    operation_kind: write_file\n    requires_approval: true\n";

struct Fixture {
    root: tempfile::TempDir,
    bundle: AuthorityPolicyBundle,
    engine: PolicyAuthorityEngine,
    action: AuthorityAction,
    decision: AuthorityDecision,
    decision_ref: CommittedRecordRef,
    criteria: SettlementCriteriaRecord,
    criteria_ref: CommittedRecordRef,
}

fn write_action(path: &str) -> AuthorityAction {
    AuthorityAction::WriteFile {
        path: path.into(),
        content_hint: "fixed".into(),
    }
}

fn open_stream(root: &Path) -> LedgerStream {
    LedgerStream::open(root, "case-case_test", "test-writer").unwrap()
}

fn consumptions(root: &Path) -> usize {
    open_stream(root)
        .read_entries()
        .unwrap()
        .iter()
        .filter(|entry| entry.record_kind == "approval_grant_consumption")
        .count()
}

fn criteria_record() -> SettlementCriteriaRecord {
    let criteria = SettlementCriteria {
        require_exit_zero: true,
        ..Default::default()
    };
    let mut record = SettlementCriteriaRecord {
        version: "0.2".into(),
        criteria_id: "crit_approval_consumption".into(),
        criteria_sha256: hash_canonical(&criteria).unwrap(),
        criteria,
        origin_refs: vec![OriginRef {
            kind: OriginRefKind::ImplementationDefined,
            reference: "approval-consumption-test".into(),
            sha256: "sha256:origin".into(),
            role: OriginRole::AcceptanceSource,
            evidence_refs: vec![],
            domain_model_ref: None,
        }],
        derivation: CriteriaDerivation {
            method: DerivationMethod::ImplementationDefined,
            actor_ref: REQUESTER.into(),
            producer_ref: None,
            rationale: "approval consumption fixture".into(),
        },
        declared_at: "2026-07-15T00:00:00Z".into(),
        criteria_record_hash: String::new(),
    };
    record.criteria_record_hash = hash_canonical(&record).unwrap();
    record
}

/// Evaluates a write_file action that escalates for approval and commits the
/// escalated decision and its settlement criteria to a fresh ledger.
fn fixture(policy: &str) -> Fixture {
    let bundle: AuthorityPolicyBundle = sea_forge_core::yaml::from_str(policy).unwrap();
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let action = write_action("out/report.txt");
    let actor = Actor {
        actor_id: REQUESTER.into(),
        role: ActorRole::Operator,
    };
    let root = tempfile::tempdir().unwrap();
    let decision = engine
        .evaluate(AuthorityEvaluation {
            actor: &actor,
            binding: bundle.resolve_identity(REQUESTER, ActorRole::Operator),
            run_id: RUN,
            case_id: CASE,
            plan_item_id: PLAN_ITEM,
            sequence: 1,
            action: &action,
            workspace_root: root.path(),
            evidence_refs: vec![],
            artifacts_root: None,
            timeout_secs: None,
            env_keys: Default::default(),
            domainforge_candidate: None,
            environment: None,
        })
        .unwrap();
    assert_eq!(decision.verdict, Verdict::Escalate);
    let stream = open_stream(root.path());
    let decision_ref = stream
        .commit_typed("authority_decision", vec![], &decision, vec![])
        .unwrap();
    let criteria = criteria_record();
    let criteria_ref = stream
        .commit_typed("settlement_criteria", vec![], &criteria, vec![])
        .unwrap();
    Fixture {
        root,
        bundle,
        engine,
        action,
        decision,
        decision_ref,
        criteria,
        criteria_ref,
    }
}

fn approved(f: &Fixture, approval_id: &str) -> ApprovalRequest {
    ApprovalRequest {
        version: "0.2".into(),
        approval_id: approval_id.into(),
        run_id: f.decision.run_id.clone(),
        case_id: f.decision.audit_record.case_id.clone().unwrap(),
        decision_id: f.decision.decision_id.clone(),
        plan_item_id: f.decision.plan_item_id.clone(),
        criteria_ref: Some(f.criteria.criteria_id.clone()),
        criteria_sha256: Some(f.criteria.criteria_sha256.clone()),
        criteria_record_hash: Some(f.criteria.criteria_record_hash.clone()),
        job_contract_ref: None,
        requested_at: "2026-07-15T00:00:00Z".into(),
        expires_at: "2099-07-16T00:00:00Z".into(),
        status: ApprovalStatus::Approved,
        resolved_by: Some(RESOLVER.into()),
        resolved_at: Some("2026-07-15T00:01:00Z".into()),
        note: None,
    }
}

fn commit_approval(f: &Fixture, approval: &ApprovalRequest) -> CommittedRecordRef {
    open_stream(f.root.path())
        .commit_typed("approval_resolution", vec![], approval, vec![])
        .unwrap()
}

fn grant_with(
    engine: &PolicyAuthorityEngine,
    f: &Fixture,
    stream: &LedgerStream,
    action: &AuthorityAction,
    approval: &ApprovalRequest,
    approval_ref: &CommittedRecordRef,
    idempotent: bool,
) -> Result<ActionGrant, ForgeError> {
    if idempotent {
        engine.grant_after_approval_idempotent(
            &f.decision,
            &f.decision_ref,
            action,
            approval,
            approval_ref,
            &f.criteria,
            &f.criteria_ref,
            None,
            stream,
        )
    } else {
        engine.grant_after_approval(
            &f.decision,
            &f.decision_ref,
            action,
            approval,
            approval_ref,
            &f.criteria,
            &f.criteria_ref,
            None,
            stream,
        )
    }
}

fn strict(
    f: &Fixture,
    action: &AuthorityAction,
    approval: &ApprovalRequest,
    approval_ref: &CommittedRecordRef,
) -> Result<ActionGrant, ForgeError> {
    let stream = open_stream(f.root.path());
    grant_with(&f.engine, f, &stream, action, approval, approval_ref, false)
}

#[test]
fn mismatched_action_does_not_consume_the_approval() {
    let f = fixture(GATED_WRITE);
    let approval = approved(&f, "apr_mismatch");
    let approval_ref = commit_approval(&f, &approval);

    let other = write_action("out/other.txt");
    let Err(error) = strict(&f, &other, &approval, &approval_ref) else {
        panic!("a grant for a different action must be refused");
    };
    assert!(
        error.to_string().contains("does not grant this action"),
        "{error}"
    );
    assert_eq!(
        consumptions(f.root.path()),
        0,
        "a refused grant must not spend the approval"
    );

    strict(&f, &f.action, &approval, &approval_ref)
        .expect("the correct action must still be granted after a refused attempt");
    assert_eq!(consumptions(f.root.path()), 1);
}

#[test]
fn missing_required_assurance_does_not_consume_the_approval() {
    let f = fixture(ASSURED_WRITE);
    let approval = approved(&f, "apr_no_assurance");
    let approval_ref = commit_approval(&f, &approval);

    let Err(error) = strict(&f, &f.action, &approval, &approval_ref) else {
        panic!("a side effect without required assurance must be refused");
    };
    assert!(
        error.to_string().contains("ledger_integrity_error"),
        "{error}"
    );
    assert_eq!(
        consumptions(f.root.path()),
        0,
        "a refused grant must not spend the approval"
    );
}

#[test]
fn decision_not_matching_its_committed_record_does_not_consume() {
    let f = fixture(GATED_WRITE);
    let approval = approved(&f, "apr_tampered_decision");
    let approval_ref = commit_approval(&f, &approval);

    let mut tampered = f.decision.clone();
    tampered.decided_at = "2099-01-01T00:00:00Z".into();
    let tampered_ref = open_stream(f.root.path())
        .commit_typed("authority_decision", vec![], &tampered, vec![])
        .unwrap();
    let stream = open_stream(f.root.path());
    let refused = f.engine.grant_after_approval(
        &f.decision,
        &tampered_ref,
        &f.action,
        &approval,
        &approval_ref,
        &f.criteria,
        &f.criteria_ref,
        None,
        &stream,
    );
    let Err(error) = refused else {
        panic!("a decision not bound to its committed record must be refused");
    };
    assert!(
        error
            .to_string()
            .contains("not bound to its committed record"),
        "{error}"
    );
    assert_eq!(
        consumptions(f.root.path()),
        0,
        "a refused grant must not spend the approval"
    );
    strict(&f, &f.action, &approval, &approval_ref)
        .expect("the bound decision must still be granted");
    assert_eq!(consumptions(f.root.path()), 1);
}

#[test]
fn valid_grant_consumes_exactly_once() {
    let f = fixture(GATED_WRITE);
    let approval = approved(&f, "apr_once");
    let approval_ref = commit_approval(&f, &approval);

    let grant = strict(&f, &f.action, &approval, &approval_ref).expect("first grant");
    assert!(grant
        .authorize(&f.action, RUN, PLAN_ITEM, f.root.path())
        .is_ok());
    assert_eq!(consumptions(f.root.path()), 1);

    let Err(error) = strict(&f, &f.action, &approval, &approval_ref) else {
        panic!("the second strict grant must be refused");
    };
    assert!(error.to_string().contains("already granted"), "{error}");
    assert_eq!(consumptions(f.root.path()), 1);

    let stream = open_stream(f.root.path());
    let replayed = grant_with(
        &f.engine,
        &f,
        &stream,
        &f.action,
        &approval,
        &approval_ref,
        true,
    )
    .expect("the idempotent variant re-derives the same grant");
    assert!(replayed
        .authorize(&f.action, RUN, PLAN_ITEM, f.root.path())
        .is_ok());
    assert_eq!(consumptions(f.root.path()), 1);
}

#[test]
fn failed_approval_checks_do_not_consume() {
    let f = fixture(GATED_WRITE);
    let mut cases: Vec<(&str, ApprovalRequest)> = Vec::new();

    let mut self_approved = approved(&f, "apr_self");
    self_approved.resolved_by = Some(REQUESTER.into());
    cases.push(("self-approval", self_approved));

    let mut rejected = approved(&f, "apr_rejected");
    rejected.status = ApprovalStatus::Rejected;
    cases.push(("rejected", rejected));

    let mut pending = approved(&f, "apr_pending");
    pending.status = ApprovalStatus::Pending;
    pending.resolved_by = None;
    pending.resolved_at = None;
    cases.push(("pending", pending));

    let mut expired = approved(&f, "apr_expired");
    expired.expires_at = "2026-07-15T00:00:30Z".into();
    cases.push(("expired resolution", expired));

    let mut criteria_mismatch = approved(&f, "apr_criteria");
    criteria_mismatch.criteria_sha256 = Some("sha256:bogus".into());
    cases.push(("criteria hash mismatch", criteria_mismatch));

    for (label, approval) in &cases {
        let approval_ref = commit_approval(&f, approval);
        assert!(
            strict(&f, &f.action, approval, &approval_ref).is_err(),
            "{label} must be refused"
        );
        assert_eq!(
            consumptions(f.root.path()),
            0,
            "{label} must not spend the approval"
        );
    }

    let sod = fixture(SOD_WRITE);
    let mut wrong_role = approved(&sod, "apr_wrong_role");
    wrong_role.resolved_by = Some("wrong_resolver".into());
    let wrong_role_ref = commit_approval(&sod, &wrong_role);
    assert!(
        strict(&sod, &sod.action, &wrong_role, &wrong_role_ref).is_err(),
        "wrong SOD role must be refused"
    );
    assert_eq!(consumptions(sod.root.path()), 0);
}

#[test]
fn concurrent_strict_grants_yield_exactly_one() {
    const N: usize = 8;
    let f = fixture(GATED_WRITE);
    let approval = approved(&f, "apr_race");
    let approval_ref = commit_approval(&f, &approval);

    let barrier = Barrier::new(N);
    let outcomes: Vec<Result<(), String>> = thread::scope(|scope| {
        let handles: Vec<_> = (0..N)
            .map(|_| {
                scope.spawn(|| {
                    // One engine and one ledger handle per request, as the server does.
                    let engine = PolicyAuthorityEngine::new(f.bundle.clone()).unwrap();
                    let stream = open_stream(f.root.path());
                    barrier.wait();
                    grant_with(
                        &engine,
                        &f,
                        &stream,
                        &f.action,
                        &approval,
                        &approval_ref,
                        false,
                    )
                    .map(|_| ())
                    .map_err(|e| e.to_string())
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect()
    });

    let granted = outcomes.iter().filter(|outcome| outcome.is_ok()).count();
    assert_eq!(granted, 1, "exactly one strict grant: {outcomes:?}");
    for outcome in outcomes.iter().filter_map(|o| o.as_ref().err()) {
        assert!(outcome.contains("already granted"), "{outcome}");
    }
    assert_eq!(consumptions(f.root.path()), 1);
}

#[test]
fn mismatched_then_correct_concurrently() {
    for round in 0..20 {
        let f = fixture(GATED_WRITE);
        let approval = approved(&f, &format!("apr_round_{round}"));
        let approval_ref = commit_approval(&f, &approval);
        let other = write_action("out/other.txt");

        let barrier = Barrier::new(2);
        let (mismatched, correct) = thread::scope(|scope| {
            let mismatched = scope.spawn(|| {
                let engine = PolicyAuthorityEngine::new(f.bundle.clone()).unwrap();
                let stream = open_stream(f.root.path());
                barrier.wait();
                grant_with(
                    &engine,
                    &f,
                    &stream,
                    &other,
                    &approval,
                    &approval_ref,
                    false,
                )
                .is_ok()
            });
            let correct = scope.spawn(|| {
                let engine = PolicyAuthorityEngine::new(f.bundle.clone()).unwrap();
                let stream = open_stream(f.root.path());
                barrier.wait();
                grant_with(
                    &engine,
                    &f,
                    &stream,
                    &f.action,
                    &approval,
                    &approval_ref,
                    false,
                )
                .is_ok()
            });
            (mismatched.join().unwrap(), correct.join().unwrap())
        });

        assert!(
            !mismatched,
            "round {round}: mismatched action must not be granted"
        );
        assert!(correct, "round {round}: correct action must be granted");
        assert_eq!(consumptions(f.root.path()), 1, "round {round}");
    }
}
