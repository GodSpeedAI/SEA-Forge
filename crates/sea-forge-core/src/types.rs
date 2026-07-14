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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
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
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SettlementCriteria {
    pub require_exit_zero: bool,
    pub required_artifacts: Vec<String>,
    pub stdout_must_contain: Option<String>,
    #[serde(default)]
    pub require_approval: bool,
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
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
