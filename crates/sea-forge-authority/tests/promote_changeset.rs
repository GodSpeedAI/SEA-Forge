//! PromoteChangeset: the authority to move an authoritative Git reference to
//! one exact proposed changeset. It must always escalate for human approval
//! and be grantable only through `grant_after_approval`.

use chrono::Utc;
use sea_forge_authority::{
    ActionGrant, AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine,
};
use sea_forge_core::errors::ForgeError;
use sea_forge_core::types::{
    Actor, ActorRole, ApprovalRequest, ApprovalStatus, AuthorityAction, AuthorityDecision,
    CriteriaDerivation, DerivationMethod, OriginRef, OriginRefKind, OriginRole, SettlementCriteria,
    SettlementCriteriaRecord, Verdict,
};
use sea_forge_ledger::types::hash_canonical;
use sea_forge_ledger::{CommittedRecordRef, LedgerStream};
use std::path::Path;

const REPO: &str = "/srv/forge/repo";
const MAIN: &str = "refs/heads/main";
const CASE: &str = "case_promote";
const RUN: &str = "run_promote";
const ITEM: &str = "item_promote";

const POLICY: &str = r#"version: "0.1"
identity_bindings:
  - principal: operator_local
    actor_type: human
    role: operator
  - principal: security_officer
    actor_type: human
    role: R-SO
rules:
  - name: promote-changeset
    verdict: allow
    actor_role: operator
    operation_kind: promote_changeset
    requires_approval: true
    boundary_constraints:
      repository: ["/srv/forge/repo"]
      reference: ["refs/heads/main"]
sod_rules:
  - name: promote_requires_security_officer
    requester_role: operator
    approver_role: R-SO
    operation_kind: promote_changeset
    allow_same_principal: false
"#;

fn parse(yaml: &str) -> AuthorityPolicyBundle {
    sea_forge_core::yaml::from_str(yaml).unwrap()
}

fn load_error(yaml: &str) -> ForgeError {
    PolicyAuthorityEngine::new(parse(yaml))
        .err()
        .expect("policy must fail validation")
}

fn hex(c: char, len: usize) -> String {
    c.to_string().repeat(len)
}

/// A well-formed promotion of `refs/heads/main` from base `1…` to proposal `2…`.
fn promote(reference: &str, base: &str, tree: &str, digest: &str) -> AuthorityAction {
    AuthorityAction::PromoteChangeset {
        repository: REPO.into(),
        reference: reference.into(),
        expected_base: base.into(),
        proposed_commit: hex('2', 40),
        proposed_tree: tree.into(),
        changeset_digest: digest.into(),
        paths: vec!["docs/a.md".into(), "src/lib.rs".into()],
    }
}

fn digest(c: char) -> String {
    format!("sha256:{}", hex(c, 64))
}

fn matching() -> AuthorityAction {
    promote(MAIN, &hex('1', 40), &hex('3', 40), &digest('4'))
}

fn operator() -> Actor {
    Actor {
        actor_id: "operator_local".into(),
        role: ActorRole::Operator,
    }
}

fn decide(
    engine: &PolicyAuthorityEngine,
    bundle: &AuthorityPolicyBundle,
    action: &AuthorityAction,
) -> AuthorityDecision {
    let actor = operator();
    let binding = bundle.resolve_identity(&actor.actor_id, actor.role.clone());
    engine
        .evaluate(AuthorityEvaluation {
            actor: &actor,
            binding,
            run_id: RUN,
            case_id: CASE,
            plan_item_id: ITEM,
            sequence: 1,
            action,
            workspace_root: Path::new("/tmp/workspace"),
            evidence_refs: vec!["intent:promote_changeset".into()],
            artifacts_root: None,
            timeout_secs: None,
            env_keys: Default::default(),
            domainforge_candidate: None,
            opa_candidate: None,
            environment: None,
        })
        .unwrap()
}

fn criteria() -> SettlementCriteriaRecord {
    let mut record = SettlementCriteriaRecord {
        version: sea_forge_core::RECORD_VERSION.into(),
        criteria_id: "crit_promote".into(),
        criteria: SettlementCriteria {
            require_exit_zero: true,
            required_artifacts: vec![],
            stdout_must_contain: None,
            agent_output_must_contain: None,
            require_approval: true,
            evaluator: None,
            records: None,
            per_record_evaluator: None,
            min_pass_ratio: None,
        },
        origin_refs: vec![OriginRef {
            kind: OriginRefKind::Intent,
            reference: "int_promote".into(),
            sha256: "sha256:intent".into(),
            role: OriginRole::AcceptanceSource,
            evidence_refs: vec![],
            domain_model_ref: None,
        }],
        derivation: CriteriaDerivation {
            method: DerivationMethod::DeterministicPlanner,
            actor_ref: "operator_local".into(),
            producer_ref: None,
            rationale: "promote_changeset fixture".into(),
        },
        declared_at: Utc::now().to_rfc3339(),
        criteria_sha256: String::new(),
        criteria_record_hash: String::new(),
    };
    record.criteria_sha256 = hash_canonical(&record.criteria).unwrap();
    let mut unhashed = record.clone();
    unhashed.criteria_record_hash.clear();
    record.criteria_record_hash = hash_canonical(&unhashed).unwrap();
    record
}

/// The committed ledger records a grant needs, for one escalated decision.
struct Ledger {
    stream: LedgerStream,
    decision_ref: CommittedRecordRef,
    criteria: SettlementCriteriaRecord,
    criteria_ref: CommittedRecordRef,
    decision: AuthorityDecision,
}

fn commit_decision(root: &Path, decision: AuthorityDecision) -> Ledger {
    let stream = LedgerStream::open(root, "case-case_promote", "test-writer").unwrap();
    let decision_ref = stream
        .commit_typed("authority_decision", vec![], &decision, vec![])
        .unwrap();
    let criteria = criteria();
    let criteria_ref = stream
        .commit_typed("settlement_criteria", vec![], &criteria, vec![])
        .unwrap();
    Ledger {
        stream,
        decision_ref,
        criteria,
        criteria_ref,
        decision,
    }
}

/// Create and resolve an approval for the ledger's decision, committing it.
fn resolve(
    ledger: &Ledger,
    approval_id: &str,
    status: ApprovalStatus,
    resolved_by: Option<&str>,
) -> (ApprovalRequest, CommittedRecordRef) {
    let resolved = status == ApprovalStatus::Approved || status == ApprovalStatus::Rejected;
    let approval = ApprovalRequest {
        version: sea_forge_core::RECORD_VERSION.into(),
        approval_id: approval_id.into(),
        run_id: ledger.decision.run_id.clone(),
        case_id: CASE.into(),
        decision_id: ledger.decision.decision_id.clone(),
        plan_item_id: ledger.decision.plan_item_id.clone(),
        criteria_ref: Some(ledger.criteria.criteria_id.clone()),
        criteria_sha256: Some(ledger.criteria.criteria_sha256.clone()),
        criteria_record_hash: Some(ledger.criteria.criteria_record_hash.clone()),
        job_contract_ref: None,
        requested_at: Utc::now().to_rfc3339(),
        expires_at: (Utc::now() + chrono::Duration::hours(1)).to_rfc3339(),
        status,
        resolved_by: resolved_by.map(str::to_owned),
        resolved_at: resolved.then(|| Utc::now().to_rfc3339()),
        note: None,
    };
    let committed = ledger
        .stream
        .commit_typed("approval_resolution", vec![], &approval, vec![])
        .unwrap();
    (approval, committed)
}

fn grant_after_approval(
    engine: &PolicyAuthorityEngine,
    ledger: &Ledger,
    action: &AuthorityAction,
    approval: &ApprovalRequest,
    committed_approval: &CommittedRecordRef,
) -> Result<ActionGrant, ForgeError> {
    engine.grant_after_approval(
        &ledger.decision,
        &ledger.decision_ref,
        action,
        approval,
        committed_approval,
        &ledger.criteria,
        &ledger.criteria_ref,
        None,
        &ledger.stream,
    )
}

fn approved_by_security_officer(
    ledger: &Ledger,
    id: &str,
) -> (ApprovalRequest, CommittedRecordRef) {
    resolve(
        ledger,
        id,
        ApprovalStatus::Approved,
        Some("security_officer"),
    )
}

#[test]
fn a_well_formed_promotion_escalates_through_the_rule_and_never_allows() {
    let bundle = parse(POLICY);
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let decision = decide(&engine, &bundle, &matching());
    assert_eq!(decision.verdict, Verdict::Escalate);
    assert_ne!(decision.verdict, Verdict::Allow);
    assert_eq!(decision.matched_rule.as_deref(), Some("promote-changeset"));
}

#[test]
fn engine_grant_cannot_mint_a_promotion_without_approval() {
    let bundle = parse(POLICY);
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let action = matching();
    let root = tempfile::tempdir().unwrap();
    let ledger = commit_decision(root.path(), decide(&engine, &bundle, &action));
    let error = match engine.grant(&ledger.decision, &ledger.decision_ref, &action, None) {
        Ok(_) => panic!("an unapproved promotion must never be granted"),
        Err(error) => error,
    };
    assert!(
        error
            .to_string()
            .contains("promote_changeset cannot be granted without approval"),
        "unexpected error: {error}"
    );
}

#[test]
fn an_approval_by_another_security_officer_grants_exactly_once() {
    let bundle = parse(POLICY);
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let action = matching();
    let root = tempfile::tempdir().unwrap();
    let ledger = commit_decision(root.path(), decide(&engine, &bundle, &action));
    let (approval, committed) = approved_by_security_officer(&ledger, "apr_once");

    let grant = grant_after_approval(&engine, &ledger, &action, &approval, &committed)
        .expect("a correctly-roled approval must grant");
    let bound = ledger.decision.action_request.evidence["payload_hash"]
        .as_str()
        .unwrap();
    assert_eq!(grant.action_hash(), bound);

    let error = match grant_after_approval(&engine, &ledger, &action, &approval, &committed) {
        Ok(_) => panic!("the same approval must not grant twice"),
        Err(error) => error,
    };
    assert!(
        error.to_string().contains("already granted"),
        "unexpected error: {error}"
    );
}

#[test]
fn the_requester_cannot_approve_their_own_promotion() {
    let bundle = parse(POLICY);
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let action = matching();
    let root = tempfile::tempdir().unwrap();
    let ledger = commit_decision(root.path(), decide(&engine, &bundle, &action));
    let (approval, committed) = resolve(
        &ledger,
        "apr_self",
        ApprovalStatus::Approved,
        Some("operator_local"),
    );
    let error = match grant_after_approval(&engine, &ledger, &action, &approval, &committed) {
        Ok(_) => panic!("self-approval must be refused"),
        Err(error) => error,
    };
    assert!(
        error
            .to_string()
            .contains("approval resolution does not match the escalated decision"),
        "unexpected error: {error}"
    );
}

#[test]
fn an_approval_is_bound_to_the_exact_changeset_base_tree_and_reference() {
    let bundle = parse(POLICY);
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let action = matching();
    let root = tempfile::tempdir().unwrap();
    let ledger = commit_decision(root.path(), decide(&engine, &bundle, &action));

    let substitutes = [
        (
            "changeset_digest",
            promote(MAIN, &hex('1', 40), &hex('3', 40), &digest('5')),
        ),
        (
            "proposed_tree",
            promote(MAIN, &hex('1', 40), &hex('6', 40), &digest('4')),
        ),
        (
            "expected_base",
            promote(MAIN, &hex('7', 40), &hex('3', 40), &digest('4')),
        ),
        (
            "reference",
            promote(
                "refs/heads/release",
                &hex('1', 40),
                &hex('3', 40),
                &digest('4'),
            ),
        ),
    ];
    for (index, (field, substituted)) in substitutes.iter().enumerate() {
        // A fresh approval per attempt: a refused grant still spends the
        // approval's consumption key, so each mismatch needs its own approval.
        let (approval, committed) =
            approved_by_security_officer(&ledger, &format!("apr_mismatch_{index}"));
        let error = match grant_after_approval(&engine, &ledger, substituted, &approval, &committed)
        {
            Ok(_) => panic!("an approval for one changeset must not grant a different {field}"),
            Err(error) => error,
        };
        assert!(
            error.to_string().contains("does not grant this action"),
            "{field}: unexpected error: {error}"
        );
    }
}

#[test]
fn rejected_and_pending_approvals_never_grant() {
    let bundle = parse(POLICY);
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let action = matching();
    let root = tempfile::tempdir().unwrap();
    let ledger = commit_decision(root.path(), decide(&engine, &bundle, &action));

    let (rejected, rejected_ref) = resolve(
        &ledger,
        "apr_rejected",
        ApprovalStatus::Rejected,
        Some("security_officer"),
    );
    let (pending, pending_ref) = resolve(&ledger, "apr_pending", ApprovalStatus::Pending, None);
    for (label, approval, committed) in [
        ("rejected", &rejected, &rejected_ref),
        ("pending", &pending, &pending_ref),
    ] {
        let error = match grant_after_approval(&engine, &ledger, &action, approval, committed) {
            Ok(_) => panic!("a {label} approval must not grant"),
            Err(error) => error,
        };
        assert!(
            error
                .to_string()
                .contains("approval resolution does not match the escalated decision"),
            "{label}: unexpected error: {error}"
        );
    }
}

#[test]
fn an_allow_rule_without_requires_approval_does_not_load() {
    let yaml = POLICY.replace("    requires_approval: true\n", "");
    let error = load_error(&yaml);
    assert!(
        error
            .to_string()
            .contains("promote_changeset requires approval"),
        "unexpected error: {error}"
    );
}

#[test]
fn malformed_promotions_are_denied_and_never_grantable() {
    let bundle = parse(POLICY);
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let base = hex('1', 40);
    let tree = hex('3', 40);
    let d = digest('4');
    let cases: Vec<(&str, AuthorityAction)> = vec![
        (
            "relative repository",
            AuthorityAction::PromoteChangeset {
                repository: "forge/repo".into(),
                reference: MAIN.into(),
                expected_base: base.clone(),
                proposed_commit: hex('2', 40),
                proposed_tree: tree.clone(),
                changeset_digest: d.clone(),
                paths: vec!["docs/a.md".into()],
            },
        ),
        (
            "padded repository",
            AuthorityAction::PromoteChangeset {
                repository: " /srv/forge/repo".into(),
                reference: MAIN.into(),
                expected_base: base.clone(),
                proposed_commit: hex('2', 40),
                proposed_tree: tree.clone(),
                changeset_digest: d.clone(),
                paths: vec!["docs/a.md".into()],
            },
        ),
        ("ref without refs/", promote("heads/main", &base, &tree, &d)),
        (
            "ref with dot-dot",
            promote("refs/heads/a..b", &base, &tree, &d),
        ),
        (
            "ref with trailing slash",
            promote("refs/heads/main/", &base, &tree, &d),
        ),
        (
            "ref with whitespace",
            promote("refs/heads/ma in", &base, &tree, &d),
        ),
        ("short base hash", promote(MAIN, "abc123", &tree, &d)),
        (
            "uppercase base hash",
            promote(MAIN, &hex('A', 40), &tree, &d),
        ),
        ("short digest", promote(MAIN, &base, &tree, "sha256:abc")),
        (
            "non-hex digest",
            promote(MAIN, &base, &tree, &format!("sha256:{}Z", hex('4', 63))),
        ),
        (
            "empty paths",
            AuthorityAction::PromoteChangeset {
                repository: REPO.into(),
                reference: MAIN.into(),
                expected_base: base.clone(),
                proposed_commit: hex('2', 40),
                proposed_tree: tree.clone(),
                changeset_digest: d.clone(),
                paths: vec![],
            },
        ),
        (
            "unsorted paths",
            AuthorityAction::PromoteChangeset {
                repository: REPO.into(),
                reference: MAIN.into(),
                expected_base: base.clone(),
                proposed_commit: hex('2', 40),
                proposed_tree: tree.clone(),
                changeset_digest: d.clone(),
                paths: vec!["src/lib.rs".into(), "docs/a.md".into()],
            },
        ),
        (
            "duplicate paths",
            AuthorityAction::PromoteChangeset {
                repository: REPO.into(),
                reference: MAIN.into(),
                expected_base: base.clone(),
                proposed_commit: hex('2', 40),
                proposed_tree: tree.clone(),
                changeset_digest: d.clone(),
                paths: vec!["docs/a.md".into(), "docs/a.md".into()],
            },
        ),
        (
            "traversal path",
            AuthorityAction::PromoteChangeset {
                repository: REPO.into(),
                reference: MAIN.into(),
                expected_base: base.clone(),
                proposed_commit: hex('2', 40),
                proposed_tree: tree.clone(),
                changeset_digest: d.clone(),
                paths: vec!["../escape.md".into()],
            },
        ),
        (
            "base equals proposal",
            AuthorityAction::PromoteChangeset {
                repository: REPO.into(),
                reference: MAIN.into(),
                expected_base: hex('2', 40),
                proposed_commit: hex('2', 40),
                proposed_tree: tree.clone(),
                changeset_digest: d.clone(),
                paths: vec!["docs/a.md".into()],
            },
        ),
    ];
    let root = tempfile::tempdir().unwrap();
    for (index, (label, action)) in cases.iter().enumerate() {
        let decision = decide(&engine, &bundle, action);
        assert_eq!(decision.verdict, Verdict::Deny, "{label} must be denied");
        assert!(
            decision
                .reason_codes
                .iter()
                .any(|code| code == "unclassified"),
            "{label}: expected reason unclassified, got {:?}",
            decision.reason_codes
        );
        // Even with an approval for this exact action, a Deny is never grantable.
        let ledger = commit_decision(root.path(), decision);
        let (approval, committed) =
            approved_by_security_officer(&ledger, &format!("apr_malformed_{index}"));
        assert!(
            grant_after_approval(&engine, &ledger, action, &approval, &committed).is_err(),
            "{label} must not be grantable"
        );
    }
}

#[test]
fn with_no_matching_rule_the_promotion_is_not_allowed() {
    let yaml = "version: \"0.1\"\nrules: []\n";
    let bundle = parse(yaml);
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let decision = decide(&engine, &bundle, &matching());
    assert_ne!(decision.verdict, Verdict::Allow);
    assert_eq!(decision.matched_rule, None);
}

#[test]
fn git_commit_still_escalates_as_unsupported_and_is_not_grantable() {
    let bundle = parse(POLICY);
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let action = AuthorityAction::GitCommit {
        paths: vec!["docs/a.md".into()],
    };
    let decision = decide(&engine, &bundle, &action);
    assert_eq!(decision.verdict, Verdict::Escalate);
    assert!(decision
        .reason_codes
        .iter()
        .any(|code| code == "unsupported_action_surface"));
    // The existing GitCommit special case names a surface, not a policy rule,
    // so no rule of the bundle is ever matched for it.
    assert_eq!(
        decision.matched_rule.as_deref(),
        Some("authority_surface:git_commit")
    );

    let root = tempfile::tempdir().unwrap();
    let ledger = commit_decision(root.path(), decision);
    let (approval, committed) = approved_by_security_officer(&ledger, "apr_git_commit");
    let error = match grant_after_approval(&engine, &ledger, &action, &approval, &committed) {
        Ok(_) => panic!("git_commit must not be grantable through approval"),
        Err(error) => error,
    };
    assert!(
        error
            .to_string()
            .contains("authority decision was not escalated for approval"),
        "unexpected error: {error}"
    );
}

#[test]
fn an_untrusted_execute_command_is_still_denied_as_untrusted() {
    let yaml = r#"version: "0.1"
rules:
  - name: shell
    verdict: allow
    actor_role: operator
    operation_kind: execute_command
    argv0: sea-forge
"#;
    let bundle = parse(yaml);
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let action = AuthorityAction::ExecuteCommand {
        argv: vec!["/bin/sh".into(), "-c".into(), "true".into()],
        cwd: ".".into(),
    };
    let decision = decide(&engine, &bundle, &action);
    assert_eq!(decision.verdict, Verdict::Deny);
    assert!(decision
        .reason_codes
        .iter()
        .any(|code| code == "untrusted_executable"));
}

#[test]
fn promote_rules_match_only_promote_actions_and_git_rules_never_match_promotions() {
    // A promote rule must not catch a GitCommit or a Reserved action that
    // names `promote_changeset` as its resource type.
    let bundle = parse(POLICY);
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let git = decide(
        &engine,
        &bundle,
        &AuthorityAction::GitCommit {
            paths: vec!["docs/a.md".into()],
        },
    );
    assert_ne!(git.matched_rule.as_deref(), Some("promote-changeset"));
    let reserved = decide(
        &engine,
        &bundle,
        &AuthorityAction::Reserved {
            resource_type: "promote_changeset".into(),
            resource_id: MAIN.into(),
            parameters: serde_json::json!({}),
        },
    );
    assert_ne!(reserved.matched_rule.as_deref(), Some("promote-changeset"));
    assert_ne!(reserved.verdict, Verdict::Allow);

    // A git_commit rule must never match a PromoteChangeset.
    let git_rule = r#"version: "0.1"
rules:
  - name: git-commit-rule
    verdict: allow
    actor_role: operator
    operation_kind: git_commit
    requires_approval: true
"#;
    let bundle = parse(git_rule);
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let decision = decide(&engine, &bundle, &matching());
    assert_ne!(decision.matched_rule.as_deref(), Some("git-commit-rule"));
    assert_ne!(decision.verdict, Verdict::Allow);
}
