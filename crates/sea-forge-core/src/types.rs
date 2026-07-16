use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Intent {
    pub intent_id: String,
    pub summary: String,
    pub actor_id: String,
    pub process_id: String,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CaseState {
    Active,
    AwaitingApproval,
    Completed,
    Terminated,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Case {
    pub version: String,
    pub case_id: String,
    pub intent: Intent,
    pub state: CaseState,
    pub plan_ref: String,
    pub run_ids: Vec<String>,
    #[serde(default)]
    pub stages: Vec<String>,
    pub close_reason: Option<String>,
    pub created_at: String,
    pub closed_at: Option<String>,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CasePlan {
    pub version: String,
    pub plan_id: String,
    pub case_id: String,
    pub run_id: String,
    pub intent_id: String,
    pub items: Vec<PlanItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub job_contract_ref: Option<String>,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    #[default]
    SandboxedTask,
    HumanTask,
    Milestone,
    Stage,
    TimerListener,
    UserEventListener,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct ItemMarkers {
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub repetition: bool,
    #[serde(default)]
    pub manual_activation: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SentryTrigger {
    pub source: String,
    pub event: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SentryPredicate {
    ArtifactExists { path: String },
    SettlementStatus { status: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Sentry {
    pub on: SentryTrigger,
    #[serde(rename = "if", default, skip_serializing_if = "Option::is_none")]
    pub if_predicate: Option<SentryPredicate>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlanItem {
    pub plan_item_id: String,
    pub name: String,
    #[serde(default)]
    pub operations: Vec<Operation>,
    #[serde(default)]
    pub entry_criteria: Vec<Sentry>,
    #[serde(default)]
    pub exit_criteria: Vec<Sentry>,
    pub settlement_criteria: SettlementCriteria,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settlement_criteria_ref: Option<String>,
    #[serde(default)]
    pub item_kind: ItemKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sandbox_class: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_stage: Option<String>,
    #[serde(default)]
    pub markers: ItemMarkers,
    #[serde(default = "default_max_instances")]
    pub max_instances: u32,
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Optional environment reference `name@version` (§7.6).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
}

fn default_max_instances() -> u32 {
    1
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Operation {
    WriteFile { path: String, content_hint: String },
    ExecuteCommand { argv: Vec<String>, cwd: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AuthorityAction {
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

impl From<&Operation> for AuthorityAction {
    fn from(operation: &Operation) -> Self {
        match operation {
            Operation::WriteFile { path, content_hint } => Self::WriteFile {
                path: path.clone(),
                content_hint: content_hint.clone(),
            },
            Operation::ExecuteCommand { argv, cwd } => Self::ExecuteCommand {
                argv: argv.clone(),
                cwd: cwd.clone(),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActorRole {
    Operator,
    Agent,
    Service,
    System,
    #[serde(rename = "R-DS")]
    DataSteward,
    #[serde(rename = "R-AG")]
    AgentGovernor,
    #[serde(rename = "R-LC")]
    LifecycleCustodian,
    #[serde(rename = "R-SO")]
    SecurityOfficer,
    #[serde(rename = "R-RM")]
    RiskManager,
    #[serde(rename = "R-DEV")]
    Developer,
    #[serde(rename = "R-AA")]
    AutomatedAgent,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Actor {
    pub actor_id: String,
    pub role: ActorRole,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ActorType {
    Human,
    Agent,
    Service,
    System,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum BindingResolution {
    Exact,
    LocalDefault,
    Unresolved,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct IdentityBinding {
    #[serde(default)]
    pub identity_id: Option<String>,
    pub principal: String,
    #[serde(default)]
    pub roles: Vec<ActorRole>,
    pub actor_type: ActorType,
    pub binding_resolution: BindingResolution,
    pub identity_binding_source: String,
    #[serde(default)]
    pub source: Option<String>,
    pub sponsor: Option<String>,
    #[serde(default)]
    pub issued_at: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub identity_binding_hash: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AuthorityRequest {
    pub schema_version: String,
    pub action_id: String,
    pub correlation_id: String,
    pub timestamp_utc: String,
    pub actor: Value,
    pub action: Value,
    pub context: Value,
    pub evidence: Value,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Allow,
    Deny,
    Escalate,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NormalizedDisposition {
    Allow,
    Deny,
    Escalate,
    Boundary,
    Degraded,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Determinism {
    pub policy_bundle_hash: String,
    pub action_request_hash: String,
    pub identity_binding_hash: String,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AuditRecord {
    pub engine: String,
    pub disposition: String,
    pub subject: String,
    pub reason: String,
    pub evidence_refs: Vec<String>,
    pub recorded_at: String,
    #[serde(default)]
    pub decision_id: Option<String>,
    #[serde(default)]
    pub case_id: Option<String>,
    #[serde(default)]
    pub run_id: Option<String>,
    #[serde(default)]
    pub policy_bundle_hash: Option<String>,
    #[serde(default)]
    pub action_request_hash: Option<String>,
    #[serde(default)]
    pub identity_binding_hash: Option<String>,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GovernanceVerdictRecord {
    pub engine: String,
    pub disposition: NormalizedDisposition,
    pub subject: String,
    pub reason: String,
    pub evidence_refs: Vec<String>,
    pub recorded_at: String,
    #[serde(default)]
    pub boundary_constraints: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub compensating_controls: Vec<String>,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AuthorityDecision {
    pub version: String,
    pub decision_id: String,
    pub run_id: String,
    pub plan_item_id: String,
    pub action_id: String,
    pub correlation_id: String,
    pub operation: AuthorityAction,
    pub outcome: Verdict,
    pub verdict: Verdict,
    pub normalized_disposition: NormalizedDisposition,
    pub matched_rule: Option<String>,
    pub reason_codes: Vec<String>,
    pub reason: String,
    pub policy_refs: Vec<String>,
    pub required_next_steps: Vec<String>,
    pub identity_binding: IdentityBinding,
    pub determinism: Determinism,
    pub action_request: AuthorityRequest,
    pub audit_record: AuditRecord,
    pub decided_at: String,
    #[serde(default)]
    pub candidate_verdicts: Vec<GovernanceVerdictRecord>,
    #[serde(default)]
    pub winning_source: Option<String>,
    #[serde(default)]
    pub precedence_reason: Option<String>,
    #[serde(default)]
    pub sandbox_class_granted: Option<String>,
    #[serde(default)]
    pub approval_request_id: Option<String>,
    #[serde(default)]
    pub opaque_constraint_id: Option<String>,
    #[serde(default)]
    pub boundary_constraints: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub compensating_controls: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ExecutionRequest {
    pub plan_item_id: String,
    pub operation: Operation,
    pub timeout_secs: u64,
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub compensating_controls: Vec<String>,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Completed,
    SpawnFailed,
    TimedOut,
    SandboxViolation,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ExecutionResult {
    pub status: ExecutionStatus,
    pub exit_code: Option<i64>,
    pub stdout_path: String,
    pub stderr_path: String,
    pub started_at: String,
    pub finished_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TraceKind {
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
    MilestoneAchieved,
    PlanMutated,
    CaseFileItemAdded,
    ItemEnabled,
    ItemActivated,
    ItemCompleted,
    ItemFailed,
    ItemTerminated,
    HumanTaskCompleted,
    CaseReopened,
    CaseTerminated,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TraceEvent {
    pub version: String,
    pub event_id: String,
    pub run_id: String,
    pub plan_item_id: Option<String>,
    pub kind: TraceKind,
    pub actor_id: String,
    pub timestamp: String,
    pub payload: Value,
    /// Federation origin (spec-full §7.4). Absent = local legacy, valid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cell_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactType {
    SeaModel,
    GeneratedContract,
    EvidenceFile,
    Other,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactStage {
    Cognitive,
    Intellectual,
    Product,
    Capital,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    Draft,
    Reviewed,
    Approved,
    Capitalized,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ArtifactProducer {
    pub entity_id: String,
    pub process_id: String,
    pub run_id: String,
    pub plan_item_id: String,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ArtifactDescriptor {
    pub artifact_id: String,
    pub artifact_type: ArtifactType,
    pub stage: Option<ArtifactStage>,
    pub producer: ArtifactProducer,
    pub owner: String,
    pub license: String,
    pub review_status: ReviewStatus,
    pub source_refs: Vec<String>,
    pub content_sha256: String,
    pub pre_mint_identity: String,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Artifact,
    AuthorityDecision,
    ExecutionResult,
    Recall,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EvidenceRecord {
    pub version: String,
    pub evidence_id: String,
    pub run_id: String,
    pub kind: EvidenceKind,
    pub uri: String,
    pub sha256: Option<String>,
    pub source_event_id: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, Value>,
    /// Federation origin (spec-full §7.4). Absent = local legacy, valid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cell_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct SettlementCriteria {
    #[serde(default)]
    pub require_exit_zero: bool,
    #[serde(default)]
    pub required_artifacts: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdout_must_contain: Option<String>,
    #[serde(default)]
    pub require_approval: bool,
    /// `<env>.<name>` — combines AND with existing checks (§7.6).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evaluator: Option<String>,
    /// Workspace-relative JSONL path for batch settlement (§7.6).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub records: Option<String>,
    /// `<env>.<name>` applied per-record in batch mode (§7.6).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub per_record_evaluator: Option<String>,
    /// Pass ratio in `0.0..=1.0` for batch acceptance (§7.6).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_pass_ratio: Option<f64>,
}

/// Origin reference for a job or settlement criterion.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct OriginRef {
    pub kind: OriginRefKind,
    pub reference: String,
    pub sha256: String,
    pub role: OriginRole,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OriginRefKind {
    Intent,
    PlanTemplate,
    SpecPipelineStage,
    PolicyRequirement,
    Issue,
    ExternalRequirement,
    JobContract,
    ImplementationDefined,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OriginRole {
    DesiredResult,
    Constraint,
    AcceptanceSource,
    DerivationInput,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CriteriaDerivation {
    pub method: DerivationMethod,
    pub actor_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub producer_ref: Option<String>,
    pub rationale: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DerivationMethod {
    Manual,
    DeterministicPlanner,
    PlanTemplate,
    SpecPipeline,
    Imported,
    ImplementationDefined,
}

/// Dedicated job contract, used only when no existing canonical record expresses
/// the requirement with sufficient provenance.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct JobContract {
    pub version: String,
    pub job_contract_id: String,
    pub direction_kind: DirectionKind,
    pub statement: String,
    pub origin_refs: Vec<OriginRef>,
    pub declared_by: String,
    pub declared_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes_ref: Option<String>,
    pub job_contract_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DirectionKind {
    Jtbd,
    Requirement,
    Goal,
    Problem,
    Constraint,
    ImplementationDefined,
}

/// Ledgered settlement-criteria record with attributable origin and derivation.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SettlementCriteriaRecord {
    pub version: String,
    pub criteria_id: String,
    pub criteria: SettlementCriteria,
    pub origin_refs: Vec<OriginRef>,
    pub derivation: CriteriaDerivation,
    pub declared_at: String,
    pub criteria_sha256: String,
    pub criteria_record_hash: String,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SettlementClaim {
    pub run_id: String,
    pub plan_item_id: String,
    pub criteria_ref: Option<String>,
    pub criteria: SettlementCriteria,
    pub execution: Option<ExecutionResult>,
    pub authority_verdicts: Vec<Verdict>,
    /// Evaluator scores keyed by `<env>.<name>`; recorded in the settlement
    /// basis as evidence (§10.6) — never standing.
    pub evaluator_scores: BTreeMap<String, f64>,
    /// Pre-computed batch evaluation result (if batch criteria set).
    pub batch: Option<BatchEvaluationResult>,
}

/// Result of batch evaluation over a JSONL records file (§7.6).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BatchEvaluationResult {
    pub total: usize,
    pub passed: usize,
    pub pass_ratio: f64,
    pub min_pass_ratio: f64,
    pub failures: Vec<BatchFailure>,
}

/// A failing record in batch evaluation, written to quarantine (§7.6).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BatchFailure {
    pub record: Value,
    pub score: f64,
    pub evidence_ref: String,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SettlementStatus {
    Accepted,
    Rejected,
    Escalated,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SettlementEvent {
    pub version: String,
    pub settlement_id: String,
    pub run_id: String,
    pub status: SettlementStatus,
    pub basis: Vec<String>,
    pub review_required: bool,
    pub settled_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub criteria_ref: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ApprovalRequest {
    pub version: String,
    pub approval_id: String,
    pub run_id: String,
    pub case_id: String,
    pub decision_id: String,
    pub plan_item_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub criteria_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub criteria_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub criteria_record_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub job_contract_ref: Option<String>,
    pub requested_at: String,
    pub expires_at: String,
    pub status: ApprovalStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

// === M4a: Settlement Declarations ===

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SettlementStrength {
    Local,
    Strong,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeclarationStatus {
    Accepted,
    Rejected,
    Escalated,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Declarer {
    pub actor_id: String,
    pub authority_ref: String,
    pub role: String,
    pub standing_basis: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DeclarationIndependence {
    pub acting_entity_id: String,
    pub independent: bool,
    pub basis: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DeclarationReliability {
    pub feedback_delay_ms: u64,
    pub attribution_confidence: String,
    pub gaming_exposure: String,
    pub hidden_debt_blindness: String,
    pub weight: String,
    pub basis: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SettlementDeclarationRequest {
    pub settlement_ref: String,
    pub run_id: String,
    pub case_id: String,
    pub plan_item_id: String,
    pub claim_manifest_sha256: String,
    pub criteria_ref: String,
    pub criteria_sha256: String,
    pub criteria_record_hash: String,
    pub criteria_declared_at: String,
    pub execution_started_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub job_contract_ref: Option<String>,
    pub origin_refs: Vec<OriginRef>,
    pub verifier_ref: String,
    pub verifier_sha256: String,
    pub acting_entity_id: String,
    pub requested_strength: SettlementStrength,
    pub declarer: Declarer,
    pub variation_tags: BTreeMap<String, String>,
    pub disruption_tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orchestration_burden: Option<String>,
    pub source_evidence_refs: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SettlementDeclaration {
    pub version: String,
    pub declaration_id: String,
    pub settlement_ref: String,
    pub run_id: String,
    pub case_id: String,
    pub plan_item_id: String,
    pub claim_manifest_sha256: String,
    pub status: DeclarationStatus,
    pub strength: SettlementStrength,
    pub qualifies_for_capability: bool,
    pub criteria_ref: String,
    pub criteria_sha256: String,
    pub criteria_record_hash: String,
    pub criteria_declared_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub job_contract_ref: Option<String>,
    pub origin_refs: Vec<OriginRef>,
    pub verifier_ref: String,
    pub verifier_sha256: String,
    pub verification_evidence_refs: Vec<String>,
    pub declarer: Declarer,
    pub independence: DeclarationIndependence,
    pub reliability: DeclarationReliability,
    pub variation_tags: BTreeMap<String, String>,
    pub disruption_tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orchestration_burden: Option<String>,
    pub issued_at: String,
    pub source_evidence_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter_attestation_ref: Option<String>,
    pub declaration_hash: String,
}

// === M4a: Capability Records ===

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityStatus {
    Attempted,
    Demonstrated,
    Proven,
    Metabolized,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct CapabilityCounts {
    #[serde(default)]
    pub accepted: u64,
    #[serde(default)]
    pub rejected: u64,
    #[serde(default)]
    pub escalated: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CapabilityQualifying {
    pub declaration_count: u64,
    pub total_weight: String,
    pub accepted_weight: String,
    pub regression_weight: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CapabilityVariation {
    pub required_dimensions: Vec<String>,
    pub covered_values: BTreeMap<String, Vec<String>>,
    pub coverage_ratio: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CapabilityRecovery {
    pub required_disruptions: Vec<String>,
    pub recovered_disruptions: Vec<String>,
    pub recovery_ratio: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CapabilityOrchestration {
    pub baseline_burden: Option<String>,
    pub current_burden: Option<String>,
    pub reduction: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CapabilityEvidenceSampleItem {
    pub run_id: String,
    pub settlement_status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declaration_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CapabilityPromotionPolicy {
    pub version: String,
    pub name: String,
    pub capability_pattern: String,
    pub policy_sha256: String,
    pub min_declarations: u64,
    pub min_total_weight: String,
    pub min_reliability_weight: String,
    pub max_regression_weight: String,
    pub required_variation_dimensions: Vec<String>,
    pub min_distinct_values_per_dimension: u64,
    pub required_disruptions: Vec<String>,
    pub require_burden_reduction: bool,
    pub min_confidence: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CapabilityRecord {
    pub version: String,
    pub capability_name: String,
    pub first_seen: String,
    pub last_seen: String,
    pub counts: CapabilityCounts,
    pub status: CapabilityStatus,
    pub promotion_policy_ref: String,
    pub promotion_policy_sha256: String,
    pub qualifying: CapabilityQualifying,
    pub variation: CapabilityVariation,
    pub recovery: CapabilityRecovery,
    pub orchestration: CapabilityOrchestration,
    pub confidence: String,
    pub contraction_reasons: Vec<String>,
    pub evidence_sample: Vec<CapabilityEvidenceSampleItem>,
    pub rebuilt_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CapabilityDelta {
    pub attempted_capability: String,
    pub result: SettlementStatus,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Attribution {
    pub entity_id: String,
    pub process_id: String,
    pub session_id: String,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ArtifactRef {
    pub evidence_id: String,
    pub artifact_id: String,
    pub pre_mint_identity: String,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SemanticEnvelope {
    pub version: String,
    pub run_id: String,
    pub case_ref: String,
    pub intent: Intent,
    pub plan_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template_ref: Option<String>,
    pub authority_decisions: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub settlement_ref: String,
    pub capability_delta: CapabilityDelta,
    pub attribution: Attribution,
    pub artifact_refs: Vec<ArtifactRef>,
    pub extension_refs: Vec<String>,
    pub projection_refs: Vec<ProjectionRef>,
    /// Federation origin (spec-full §7.4). Absent = local legacy, valid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cell_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryKind {
    Fact,
    Decision,
    Outcome,
    Preference,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MemoryItemProvenance {
    pub run_ids: Vec<String>,
    pub evidence_refs: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MemoryItem {
    pub version: String,
    pub memory_id: String,
    pub kind: MemoryKind,
    pub statement: String,
    pub attribution: Attribution,
    pub provenance: MemoryItemProvenance,
    pub dedup_key: String,
    pub created_at: String,
    pub last_confirmed_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionKind {
    ProjectionAdapter,
    RuntimeAdapter,
    SandboxBackend,
    EventSink,
    MemoryIndex,
    Evaluator,
    ArtifactAttestor,
    UiClient,
    NotificationAdapter,
    ImportExportAdapter,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ContractRef {
    pub schema: String,
    pub sha256: String,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ExtensionDescriptor {
    pub extension_id: String,
    pub kind: ExtensionKind,
    pub name: String,
    pub version: String,
    pub provider: String,
    pub capabilities: Vec<String>,
    pub authority_surface: String,
    pub input_contract: ContractRef,
    pub output_contract: ContractRef,
    pub deterministic: bool,
    pub installed_at: Option<String>,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionKind {
    Sea,
    Calm,
    Rdf,
    Sbvr,
    Shacl,
    Manifest,
    KgEvent,
    CapabilityRecord,
    MemoryIndex,
    CapitalRecord,
    ImplementationDefined,
    // M9 (E11) self-model projections. Appended so existing variants' Ord ranks
    // are unchanged. Persisted ONLY under the `self_model_projection` ledger
    // record_kind and `.sea-forge/self-model/`; old readers filter on
    // record_kind and never deserialize these variants. See
    // tests/projection_kind_boundary.rs for the isolation proof.
    Kg,
    SelfModelSnapshot,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionStatus {
    Accepted,
    Rejected,
    Quarantined,
    Degraded,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ProjectionRef {
    pub projection_id: String,
    pub projection_kind: ProjectionKind,
    pub source_refs: Vec<String>,
    pub adapter_ref: String,
    pub output_uri: String,
    pub output_sha256: String,
    pub status: ProjectionStatus,
    pub settlement_ref: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn operation_uses_tagged_snake_case_json() {
        let value = serde_json::to_value(Operation::WriteFile {
            path: "model.sea".into(),
            content_hint: "fixed".into(),
        })
        .unwrap();
        assert_eq!(value["kind"], "write_file");
    }
    #[test]
    fn settlement_status_round_trips() {
        let value = serde_json::to_string(&SettlementStatus::Escalated).unwrap();
        assert_eq!(value, "\"escalated\"");
        assert_eq!(
            serde_json::from_str::<SettlementStatus>(&value).unwrap(),
            SettlementStatus::Escalated
        );
    }
}

// ── M5: Spec-to-code pipeline + DomainForge projections (§7.8, §7.0b) ──

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PipelineRoute {
    SpecAuthoring,
    GeneratorAuthoring,
    Regeneration,
    LastMile,
    FullSpecToRuntime,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProofClassification {
    AuthorityOnly,
    GeneratedContract,
    FocusedSlice,
    LiveDevProof,
    ReleaseGateProof,
    EnterpriseShippable,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StageKind {
    Adr,
    Prd,
    Sds,
    Sea,
    Ast,
    Ir,
    Manifest,
    GeneratedContract,
    SemanticFixture,
    LastMileAdapter,
    RuntimeWiring,
    AcceptanceProof,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StageStatus {
    Pending,
    Accepted,
    Rejected,
    Quarantined,
    Skipped,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct StageFile {
    pub path: String,
    pub sha256: String,
    #[serde(default)]
    pub generated: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SpecPipelineStage {
    pub stage_id: String,
    pub kind: StageKind,
    #[serde(default)]
    pub inputs: Vec<StageFile>,
    #[serde(default)]
    pub outputs: Vec<StageFile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,
    pub status: StageStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quarantine_ref: Option<String>,
    #[serde(default)]
    pub settlement_basis: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SpecPipelineRun {
    pub version: String,
    pub pipeline_id: String,
    pub case_id: String,
    pub run_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain_model_ref: Option<crate::types::Value>,
    pub route: PipelineRoute,
    #[serde(default)]
    pub authority_refs: Vec<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settlement_ref: Option<String>,
    pub stages: Vec<SpecPipelineStage>,
    pub proof_classification: ProofClassification,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProjectionRecord {
    pub projection_id: String,
    pub projection_kind: ProjectionKind,
    pub adapter_ref: String,
    pub case_id: String,
    pub run_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain_model_ref: Option<Value>,
    pub source_refs: Vec<String>,
    pub input_hash: String,
    pub output_refs: Vec<StageFile>,
    #[serde(default)]
    pub quarantine_refs: Vec<String>,
    pub validation: ProjectionValidation,
    #[serde(default)]
    pub authority_refs: Vec<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settlement_ref: Option<String>,
    pub created_at: String,
    pub rebuild_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ProjectionValidation {
    pub status: ProjectionStatus,
    pub validator_ref: String,
    #[serde(default)]
    pub basis: Vec<String>,
}

// ── M6: SeaCell federation bundles (§7.4) ──

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BundleFile {
    /// Path relative to bundle root (e.g. `runs/<run_id>/plan.json`).
    pub path: String,
    pub sha256: String,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BundleManifest {
    pub schema_version: String,
    pub bundle_id: String,
    /// Exporter cell id (`cell_<8 hex>`).
    pub cell_id: String,
    pub created_at: String,
    pub run_ids: Vec<String>,
    /// Template references carried by the bundle (e.g. `name@version`).
    #[serde(default)]
    pub templates: Vec<String>,
    pub files: Vec<BundleFile>,
}
