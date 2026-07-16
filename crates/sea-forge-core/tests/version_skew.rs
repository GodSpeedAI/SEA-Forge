use sea_forge_core::types::{
    ActorType, ArtifactRef, Attribution, AuditRecord, AuthorityAction, AuthorityDecision,
    AuthorityRequest, BindingResolution, CapabilityDelta, Case, CaseState, Determinism,
    EvidenceKind, EvidenceRecord, IdentityBinding, Intent, NormalizedDisposition, SemanticEnvelope,
    SettlementEvent, SettlementStatus, TraceEvent, TraceKind, Verdict,
};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[allow(dead_code)]
struct V01Case {
    version: String,
    case_id: String,
    intent: V01Intent,
    state: V01CaseState,
    plan_ref: String,
    run_ids: Vec<String>,
    close_reason: Option<String>,
    created_at: String,
    closed_at: Option<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct V01Intent {
    intent_id: String,
    summary: String,
    actor_id: String,
    process_id: String,
    created_at: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum V01CaseState {
    Active,
    Completed,
    Terminated,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct V01TraceEvent {
    version: String,
    event_id: String,
    run_id: String,
    plan_item_id: Option<String>,
    kind: V01TraceKind,
    actor_id: String,
    timestamp: String,
    payload: Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum V01TraceKind {
    CaseCreated,
    RunStarted,
    PlanCreated,
    AuthorityEvaluated,
    WorkspaceCreated,
    CommandStarted,
    CommandFinished,
    ArtifactCaptured,
    SettlementRecorded,
    RunHalted,
    RunFinished,
    CaseClosed,
    InternalError,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct V01EvidenceRecord {
    version: String,
    evidence_id: String,
    run_id: String,
    kind: V01EvidenceKind,
    uri: String,
    sha256: Option<String>,
    source_event_id: String,
    #[serde(default)]
    metadata: BTreeMap<String, Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum V01EvidenceKind {
    Artifact,
    AuthorityDecision,
    ExecutionResult,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct V01SettlementEvent {
    version: String,
    settlement_id: String,
    run_id: String,
    status: V01SettlementStatus,
    basis: Vec<String>,
    review_required: bool,
    settled_at: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum V01SettlementStatus {
    Accepted,
    Rejected,
    Escalated,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct V01SemanticEnvelope {
    version: String,
    run_id: String,
    case_ref: String,
    intent: V01Intent,
    plan_ref: String,
    authority_decisions: Vec<String>,
    evidence_refs: Vec<String>,
    settlement_ref: String,
    capability_delta: V01CapabilityDelta,
    attribution: V01Attribution,
    artifact_refs: Vec<V01ArtifactRef>,
    extension_refs: Vec<String>,
    projection_refs: Vec<Value>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct V01CapabilityDelta {
    attempted_capability: String,
    result: V01SettlementStatus,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct V01Attribution {
    entity_id: String,
    process_id: String,
    session_id: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct V01ArtifactRef {
    evidence_id: String,
    artifact_id: String,
    pre_mint_identity: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct V01AuthorityDecision {
    version: String,
    decision_id: String,
    run_id: String,
    plan_item_id: String,
    action_id: String,
    correlation_id: String,
    operation: V01AuthorityAction,
    outcome: V01Verdict,
    verdict: V01Verdict,
    normalized_disposition: V01NormalizedDisposition,
    matched_rule: Option<String>,
    reason_codes: Vec<String>,
    reason: String,
    policy_refs: Vec<String>,
    required_next_steps: Vec<String>,
    identity_binding: V01IdentityBinding,
    determinism: V01Determinism,
    action_request: V01AuthorityRequest,
    audit_record: V01AuditRecord,
    decided_at: String,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[allow(dead_code)]
enum V01AuthorityAction {
    WriteFile {
        path: String,
        content_hint: String,
    },
    ExecuteCommand {
        argv: Vec<String>,
        cwd: String,
    },
    ExternalApi {
        host: String,
    },
    GitCommit {
        paths: Vec<String>,
    },
    GithubPr {
        has_required_evidence: bool,
    },
    Reserved {
        resource_type: String,
        resource_id: String,
        parameters: Value,
    },
    Unclassified {
        raw_kind: String,
        parameters: Value,
    },
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum V01Verdict {
    Allow,
    Deny,
    Escalate,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum V01NormalizedDisposition {
    Allow,
    Deny,
    Escalate,
    Boundary,
    Degraded,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct V01IdentityBinding {
    principal: String,
    actor_type: V01ActorType,
    binding_resolution: V01BindingResolution,
    identity_binding_source: String,
    sponsor: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum V01ActorType {
    Human,
    Agent,
    Service,
    System,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum V01BindingResolution {
    Exact,
    LocalDefault,
    Unresolved,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct V01Determinism {
    policy_bundle_hash: String,
    action_request_hash: String,
    identity_binding_hash: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct V01AuthorityRequest {
    schema_version: String,
    action_id: String,
    correlation_id: String,
    timestamp_utc: String,
    actor: Value,
    action: Value,
    context: Value,
    evidence: Value,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct V01AuditRecord {
    engine: String,
    disposition: String,
    subject: String,
    reason: String,
    evidence_refs: Vec<String>,
    recorded_at: String,
}

#[test]
fn exact_v01_minimum_record_readers_ignore_v02_additive_fields() {
    let intent = Intent {
        intent_id: "int_01".into(),
        summary: "demo".into(),
        actor_id: "operator_local".into(),
        process_id: "test".into(),
        created_at: "2026-07-16T00:00:00Z".into(),
    };
    let case = Case {
        version: "0.2".into(),
        case_id: "case_01".into(),
        intent: intent.clone(),
        state: CaseState::Active,
        plan_ref: "plan_01".into(),
        run_ids: vec!["run_01".into()],
        stages: vec!["stage_01".into()],
        close_reason: None,
        created_at: "2026-07-16T00:00:00Z".into(),
        closed_at: None,
    };
    let trace = TraceEvent {
        version: "0.2".into(),
        event_id: "evt_01".into(),
        run_id: "run_01".into(),
        plan_item_id: Some("task".into()),
        kind: TraceKind::RunStarted,
        actor_id: "operator_local".into(),
        timestamp: "2026-07-16T00:00:00Z".into(),
        payload: serde_json::json!({}),
        cell_id: Some("cell_12345678".into()),
    };
    let evidence = EvidenceRecord {
        version: "0.2".into(),
        evidence_id: "evi_0001".into(),
        run_id: "run_01".into(),
        kind: EvidenceKind::Artifact,
        uri: "artifacts/model.sea".into(),
        sha256: Some("sha256:abc".into()),
        source_event_id: "evt_01".into(),
        metadata: BTreeMap::new(),
        cell_id: Some("cell_12345678".into()),
    };
    let settlement = SettlementEvent {
        version: "0.2".into(),
        settlement_id: "set_01".into(),
        run_id: "run_01".into(),
        status: SettlementStatus::Accepted,
        basis: vec!["exit_zero".into()],
        review_required: false,
        settled_at: "2026-07-16T00:00:00Z".into(),
        criteria_ref: Some("criteria_01".into()),
    };
    let authority = AuthorityDecision {
        version: "0.2".into(),
        decision_id: "auth_01".into(),
        run_id: "run_01".into(),
        plan_item_id: "task".into(),
        action_id: "act_01".into(),
        correlation_id: "run_01".into(),
        operation: AuthorityAction::WriteFile {
            path: "model.sea".into(),
            content_hint: "sha256:abc".into(),
        },
        outcome: Verdict::Allow,
        verdict: Verdict::Allow,
        normalized_disposition: NormalizedDisposition::Allow,
        matched_rule: Some("allow-write".into()),
        reason_codes: vec!["policy_allow".into()],
        reason: "policy_allow".into(),
        policy_refs: vec!["local-policy:allow-write".into()],
        required_next_steps: vec![],
        identity_binding: IdentityBinding {
            identity_id: Some("identity_01".into()),
            principal: "operator_local".into(),
            roles: vec![],
            actor_type: ActorType::Human,
            binding_resolution: BindingResolution::Exact,
            identity_binding_source: "test".into(),
            source: Some("test".into()),
            sponsor: None,
            issued_at: Some("2026-07-16T00:00:00Z".into()),
            expires_at: None,
            identity_binding_hash: Some("sha256:identity".into()),
        },
        determinism: Determinism {
            policy_bundle_hash: "sha256:policy".into(),
            action_request_hash: "sha256:action".into(),
            identity_binding_hash: "sha256:identity".into(),
        },
        action_request: AuthorityRequest {
            schema_version: "0.2".into(),
            action_id: "act_01".into(),
            correlation_id: "run_01".into(),
            timestamp_utc: "2026-07-16T00:00:00Z".into(),
            actor: serde_json::json!({"actor_id":"operator_local"}),
            action: serde_json::json!({"kind":"write_file"}),
            context: serde_json::json!({"run_id":"run_01"}),
            evidence: serde_json::json!([]),
        },
        audit_record: AuditRecord {
            engine: "local".into(),
            disposition: "allow".into(),
            subject: "model.sea".into(),
            reason: "policy_allow".into(),
            evidence_refs: vec!["evi_0001".into()],
            recorded_at: "2026-07-16T00:00:00Z".into(),
            decision_id: Some("auth_01".into()),
            case_id: Some("case_01".into()),
            run_id: Some("run_01".into()),
            policy_bundle_hash: Some("sha256:policy".into()),
            action_request_hash: Some("sha256:action".into()),
            identity_binding_hash: Some("sha256:identity".into()),
        },
        decided_at: "2026-07-16T00:00:00Z".into(),
        candidate_verdicts: vec![],
        winning_source: Some("local".into()),
        precedence_reason: Some("allow".into()),
        sandbox_class_granted: Some("local".into()),
        approval_request_id: None,
        opaque_constraint_id: None,
        boundary_constraints: BTreeMap::new(),
        compensating_controls: vec![],
    };
    let envelope = SemanticEnvelope {
        version: "0.2".into(),
        run_id: "run_01".into(),
        case_ref: "case_01".into(),
        intent,
        plan_ref: "plan_01".into(),
        template_ref: Some("demo@0.1.0".into()),
        authority_decisions: vec!["auth_01".into()],
        evidence_refs: vec!["evi_0001".into()],
        settlement_ref: "set_01".into(),
        capability_delta: CapabilityDelta {
            attempted_capability: "demo".into(),
            result: SettlementStatus::Accepted,
        },
        attribution: Attribution {
            entity_id: "operator_local".into(),
            process_id: "test".into(),
            session_id: "run_01".into(),
        },
        artifact_refs: vec![ArtifactRef {
            evidence_id: "evi_0001".into(),
            artifact_id: "art_000001".into(),
            pre_mint_identity: "ifl:hash:abc".into(),
        }],
        extension_refs: vec![],
        projection_refs: vec![],
        cell_id: Some("cell_12345678".into()),
    };

    serde_json::from_value::<V01Case>(serde_json::to_value(case).unwrap()).unwrap();
    serde_json::from_value::<V01TraceEvent>(serde_json::to_value(trace).unwrap()).unwrap();
    serde_json::from_value::<V01EvidenceRecord>(serde_json::to_value(evidence).unwrap()).unwrap();
    serde_json::from_value::<V01SettlementEvent>(serde_json::to_value(settlement).unwrap())
        .unwrap();
    serde_json::from_value::<V01SemanticEnvelope>(serde_json::to_value(envelope).unwrap()).unwrap();
    serde_json::from_value::<V01AuthorityDecision>(serde_json::to_value(authority).unwrap())
        .unwrap();
}
