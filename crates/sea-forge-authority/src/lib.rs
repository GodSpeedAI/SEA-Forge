use chrono::Utc;
use sea_forge_core::{errors::ForgeError, ids::random_id, types::*, RECORD_VERSION};
use sea_forge_domainforge::{
    evaluate_authority, CandidateDisposition, DomainForgeTrace, DomainModel, DomainModelRef,
};
use sea_forge_ledger::{types::payload_hash, CommittedRecordRef, LedgerStream, PreActionAssurance};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fs,
    path::{Component, Path, PathBuf},
    sync::Mutex,
};
use unicode_normalization::UnicodeNormalization;

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn hash_canonical<T: Serialize>(value: &T) -> Result<String, ForgeError> {
    fn sorted(value: Value) -> Value {
        match value {
            Value::Object(map) => Value::Object(
                map.into_iter()
                    .map(|(key, value)| (key, sorted(value)))
                    .collect(),
            ),
            Value::Array(items) => Value::Array(items.into_iter().map(sorted).collect()),
            Value::String(value) => Value::String(value.nfc().collect()),
            other => other,
        }
    }
    let value = sorted(serde_json::to_value(value)?);
    Ok(format!(
        "sha256:{}",
        sha256_bytes(&serde_json::to_vec(&value)?)
    ))
}

fn issued_decision_key(decision: &AuthorityDecision) -> Result<String, ForgeError> {
    hash_canonical(decision)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceDisposition {
    Allow,
    Deny,
    Escalate,
    Boundary,
    Degraded,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GovernanceVerdict {
    pub engine: String,
    pub disposition: GovernanceDisposition,
    #[serde(default)]
    pub boundaries: BTreeMap<String, BTreeSet<String>>,
    #[serde(default)]
    pub compensating_controls: BTreeSet<String>,
    pub reason: String,
    pub evidence_refs: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResolutionPolicy {
    pub allow_degraded: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedGovernance {
    pub disposition: GovernanceDisposition,
    pub boundaries: BTreeMap<String, BTreeSet<String>>,
    pub compensating_controls: BTreeSet<String>,
}

/// Move-only proof that one exact action may execute in one exact context.
/// It is intentionally not serializable or cloneable.
pub struct ActionGrant {
    action: AuthorityAction,
    run_id: String,
    plan_item_id: String,
    workspace_root: PathBuf,
    artifacts_root: Option<PathBuf>,
    timeout_secs: Option<u64>,
    env_keys: BTreeSet<String>,
    _assurance_ref: Option<String>,
    sandbox_class: String,
    boundaries: BTreeMap<String, Vec<String>>,
    compensating_controls: Vec<String>,
    expires_at: chrono::DateTime<Utc>,
}

impl ActionGrant {
    pub fn sandbox_class(&self) -> &str {
        &self.sandbox_class
    }

    pub fn authorize(
        self,
        action: &AuthorityAction,
        run_id: &str,
        plan_item_id: &str,
        workspace_root: &Path,
    ) -> Result<(), ForgeError> {
        self.authorize_with_controls(action, run_id, plan_item_id, workspace_root, &[])
    }

    pub fn authorize_with_controls(
        self,
        action: &AuthorityAction,
        run_id: &str,
        plan_item_id: &str,
        workspace_root: &Path,
        compensating_controls: &[String],
    ) -> Result<(), ForgeError> {
        if Utc::now() > self.expires_at {
            return Err(ForgeError::Input("authority grant expired".into()));
        }
        if &self.action != action
            || self.run_id != run_id
            || self.plan_item_id != plan_item_id
            || self.workspace_root != workspace_root
        {
            return Err(ForgeError::Input(
                "authority grant does not match action context".into(),
            ));
        }
        if self.compensating_controls != compensating_controls {
            return Err(ForgeError::Input(
                "authority grant compensating controls mismatch".into(),
            ));
        }
        if let Some(allowed) = self.boundaries.get("workspace") {
            let workspace = workspace_root.to_string_lossy();
            if !allowed.iter().any(|value| value == workspace.as_ref()) {
                return Err(ForgeError::Input(
                    "authority grant workspace is outside its boundary".into(),
                ));
            }
        }
        if let Some(allowed) = self.boundaries.get("sandbox_class") {
            if !allowed.contains(&self.sandbox_class) {
                return Err(ForgeError::Input(
                    "authority grant sandbox is outside its boundary".into(),
                ));
            }
        }
        Ok(())
    }

    pub fn authorize_execution(
        self,
        action: &AuthorityAction,
        context: ExecutionGrantContext<'_>,
    ) -> Result<(), ForgeError> {
        if self.artifacts_root.as_deref() != Some(context.artifacts_root)
            || self.timeout_secs != Some(context.timeout_secs)
            || self.env_keys != context.env_keys
        {
            return Err(ForgeError::Input(
                "authority grant does not match execution context".into(),
            ));
        }
        if let Some(allowed) = self.boundaries.get("artifacts_root") {
            let artifacts = context.artifacts_root.to_string_lossy();
            if !allowed.iter().any(|value| value == artifacts.as_ref()) {
                return Err(ForgeError::Input(
                    "authority grant artifacts root is outside its boundary".into(),
                ));
            }
        }
        if let Some(allowed) = self.boundaries.get("timeout_secs") {
            if !allowed
                .iter()
                .any(|value| value == &context.timeout_secs.to_string())
            {
                return Err(ForgeError::Input(
                    "authority grant timeout is outside its boundary".into(),
                ));
            }
        }
        if let Some(allowed) = self.boundaries.get("env_keys") {
            if context.env_keys.iter().any(|key| !allowed.contains(key)) {
                return Err(ForgeError::Input(
                    "authority grant environment is outside its boundary".into(),
                ));
            }
        }
        if self.compensating_controls.contains(&"short_timeout".into()) && context.timeout_secs > 30
        {
            return Err(ForgeError::Input(
                "short_timeout compensating control not satisfied".into(),
            ));
        }
        if self
            .compensating_controls
            .contains(&"minimal_environment".into())
            && context
                .env_keys
                .iter()
                .any(|key| !matches!(key.as_str(), "PATH" | "HOME"))
        {
            return Err(ForgeError::Input(
                "minimal_environment compensating control not satisfied".into(),
            ));
        }
        self.authorize_with_controls(
            action,
            context.run_id,
            context.plan_item_id,
            context.workspace_root,
            context.compensating_controls,
        )
    }
}

pub struct ExecutionGrantContext<'a> {
    pub run_id: &'a str,
    pub plan_item_id: &'a str,
    pub workspace_root: &'a Path,
    pub artifacts_root: &'a Path,
    pub timeout_secs: u64,
    pub env_keys: BTreeSet<String>,
    pub compensating_controls: &'a [String],
}

pub struct DomainForgeCandidate {
    disposition: CandidateDisposition,
    model_ref: DomainModelRef,
    trace: DomainForgeTrace,
}

impl DomainForgeCandidate {
    pub fn evaluate(
        model: &DomainModel,
        action: &AuthorityAction,
        evidence_refs: Vec<String>,
    ) -> Result<Self, ForgeError> {
        let (operation_kind, resource_id) = match action {
            AuthorityAction::WriteFile { path, .. } => ("write_file", path.as_str()),
            AuthorityAction::ExecuteCommand { argv, .. } => (
                "execute_command",
                argv.first().map(String::as_str).unwrap_or(""),
            ),
            AuthorityAction::Reserved {
                resource_type,
                resource_id,
                ..
            } => (resource_type.as_str(), resource_id.as_str()),
            AuthorityAction::ExternalApi { host } => ("external_api", host.as_str()),
            AuthorityAction::GitCommit { .. } => ("git_commit", "repository"),
            AuthorityAction::GithubPr { .. } => ("github_pr", "pull_request"),
            AuthorityAction::Unclassified { .. } => ("unclassified", "unknown"),
        };
        let trace = evaluate_authority(model, operation_kind, resource_id, evidence_refs)?;
        Ok(Self {
            disposition: trace.normalized_disposition.clone(),
            trace,
            model_ref: model.model_ref.clone(),
        })
    }
}

pub fn resolve_candidates(
    candidates: &[GovernanceVerdict],
    policy: &ResolutionPolicy,
) -> ResolvedGovernance {
    let mut boundaries = BTreeMap::<String, BTreeSet<String>>::new();
    let mut empty_boundary = false;
    for candidate in candidates {
        for (dimension, allowed) in &candidate.boundaries {
            boundaries
                .entry(dimension.clone())
                .and_modify(|current| {
                    current.retain(|value| allowed.contains(value));
                    empty_boundary |= current.is_empty();
                })
                .or_insert_with(|| allowed.clone());
        }
    }
    let compensating_controls: BTreeSet<String> = candidates
        .iter()
        .flat_map(|candidate| candidate.compensating_controls.iter().cloned())
        .collect();
    let missing_evidence = candidates
        .iter()
        .any(|candidate| candidate.evidence_refs.is_empty());
    let has = |disposition| {
        candidates
            .iter()
            .any(|candidate| candidate.disposition == disposition)
    };
    let disposition = if candidates.is_empty()
        || missing_evidence
        || empty_boundary
        || has(GovernanceDisposition::Deny)
    {
        GovernanceDisposition::Deny
    } else if has(GovernanceDisposition::Escalate) {
        GovernanceDisposition::Escalate
    } else if has(GovernanceDisposition::Boundary) {
        GovernanceDisposition::Boundary
    } else if has(GovernanceDisposition::Degraded) {
        if policy.allow_degraded && !compensating_controls.is_empty() {
            GovernanceDisposition::Degraded
        } else {
            GovernanceDisposition::Deny
        }
    } else {
        GovernanceDisposition::Allow
    };
    ResolvedGovernance {
        disposition,
        boundaries,
        compensating_controls,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct AuthorityPolicyBundle {
    pub version: String,
    #[serde(default)]
    pub bundle_id: Option<String>,
    #[serde(default)]
    pub policy_bundle_hash: Option<String>,
    #[serde(default)]
    pub policy_bundle_version: Option<String>,
    #[serde(default)]
    pub loaded_at: Option<String>,
    #[serde(default)]
    pub source_base: Option<PathBuf>,
    #[serde(default)]
    pub sources: Vec<PolicySource>,
    #[serde(default)]
    pub roles: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub permissions: Vec<PermissionGrant>,
    #[serde(default)]
    pub sod_rules: Vec<SodRule>,
    #[serde(default = "default_deny")]
    pub default: String,
    #[serde(default)]
    pub identity: IdentityPolicy,
    #[serde(default)]
    pub policy_surfaces: PolicySurfaces,
    pub rules: Vec<PolicyRule>,
    #[serde(default)]
    pub policy_engines: Vec<PolicyEngineConfig>,
    #[serde(default)]
    pub allow_degraded: bool,
    #[serde(default)]
    pub identity_bindings: Vec<ConfiguredIdentity>,
    #[serde(default)]
    pub integrity_ledger: IntegrityLedgerPolicy,
    #[serde(default)]
    pub settlement_authorities: Vec<SettlementAuthorityDescriptor>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SettlementAuthorityDescriptor {
    pub authority_ref: String,
    pub adapter: String,
    #[serde(default)]
    pub command: Vec<String>,
    #[serde(default = "default_settlement_timeout_secs")]
    pub timeout_secs: u64,
    pub trust_anchor_ref: String,
    pub declarer_actor_id: String,
    pub permitted_declarer_roles: Vec<String>,
    pub standing_basis: String,
    #[serde(default)]
    pub may_issue_strong: bool,
}

fn default_settlement_timeout_secs() -> u64 {
    60
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct IntegrityLedgerPolicy {
    #[serde(default)]
    pub required_for_side_effects: bool,
    #[serde(default)]
    pub min_witnesses: usize,
    #[serde(default)]
    pub signing_key_dir: Option<PathBuf>,
    #[serde(default = "default_signing_key_id")]
    pub signing_key_id: String,
    #[serde(default)]
    pub witnesses: Vec<WitnessPolicy>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct WitnessPolicy {
    pub witness_id: String,
    pub key_dir: PathBuf,
    pub key_id: String,
}

fn default_signing_key_id() -> String {
    "sea-forge-local".into()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct PolicySource {
    pub surface: String,
    pub path: String,
    pub sha256: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct PermissionGrant {
    pub role: String,
    pub operation_kind: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct SodRule {
    pub name: String,
    pub requester_role: String,
    pub approver_role: String,
    #[serde(default)]
    pub operation_kind: Option<String>,
    /// When set, scopes a transition SOD rule to one edge
    /// (`synthesize` | `productize` | `capitalize`). Required for
    /// `operation_kind: transition_artifact_stage`; forbidden otherwise.
    /// Absent on non-transition rules so canonical serialization and policy
    /// hashes stay unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transition_kind: Option<String>,
    #[serde(default)]
    pub allow_same_principal: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ConfiguredIdentity {
    pub principal: String,
    pub actor_type: ActorType,
    pub role: ActorRole,
    #[serde(default)]
    pub sponsor: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct PolicyEngineConfig {
    pub engine: String,
    pub fail_mode: String,
    #[serde(default)]
    pub required: bool,
}
fn default_deny() -> String {
    "deny".into()
}
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct IdentityPolicy {
    #[serde(default = "default_source")]
    pub source: String,
    #[serde(default)]
    pub allow_unresolved: bool,
}
fn default_source() -> String {
    "local-slice-default".into()
}
impl Default for IdentityPolicy {
    fn default() -> Self {
        Self {
            source: default_source(),
            allow_unresolved: false,
        }
    }
}
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct PolicySurfaces {
    #[serde(default)]
    pub file: FileSurface,
    #[serde(default)]
    pub external_api: ExternalApiSurface,
    #[serde(default)]
    pub git_commit: GitCommitSurface,
    #[serde(default)]
    pub github_pr: GithubPrSurface,
    #[serde(default)]
    pub prompt_risk: PromptRiskSurface,
    /// M11 (E13 §8.2): self-disclosure policy surface. Absent ⇒ deny-all.
    #[serde(default)]
    pub self_disclosure: SelfDisclosureSurface,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct FileSurface {
    #[serde(default = "deny_mode")]
    pub mode: String,
    #[serde(default = "default_denies")]
    pub deny_write: Vec<String>,
}
fn deny_mode() -> String {
    "deny-by-default".into()
}
fn default_denies() -> Vec<String> {
    vec![
        "**/src/gen/**",
        "docs/specs/**/*.ast*.json",
        "docs/specs/**/*.ir.json",
        "docs/specs/**/*.manifest.json",
        "docs/specs/**/fixtures/semantic/*.semantic.fixture.yaml",
        ".git/**",
        ".env*",
        "**/*secret*",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ExternalApiSurface {
    #[serde(default = "deny_mode")]
    pub mode: String,
    #[serde(default)]
    pub allow_hosts: Vec<String>,
}
impl Default for ExternalApiSurface {
    fn default() -> Self {
        Self {
            mode: deny_mode(),
            allow_hosts: vec![],
        }
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct GitCommitSurface {
    #[serde(default = "default_protected_paths")]
    pub protected_paths: Vec<String>,
}
fn default_protected_paths() -> Vec<String> {
    vec!["config/governance/**".into(), ".github/workflows/**".into()]
}
impl Default for GitCommitSurface {
    fn default() -> Self {
        Self {
            protected_paths: default_protected_paths(),
        }
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct GithubPrSurface {
    #[serde(default = "default_escalate")]
    pub default: String,
}
fn default_escalate() -> String {
    "escalate".into()
}
impl Default for GithubPrSurface {
    fn default() -> Self {
        Self {
            default: default_escalate(),
        }
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct PromptRiskSurface {
    #[serde(default = "default_risk")]
    pub default: String,
}
fn default_risk() -> String {
    "allow-with-risk-escalation".into()
}
impl Default for PromptRiskSurface {
    fn default() -> Self {
        Self {
            default: default_risk(),
        }
    }
}
impl Default for FileSurface {
    fn default() -> Self {
        Self {
            mode: deny_mode(),
            deny_write: default_denies(),
        }
    }
}

// ── M11 (E13 §8.2): self_disclosure policy surface ──

/// Valid claim class names for the self_disclosure surface.
pub const VALID_CLAIM_CLASSES: &[&str] = &[
    "identity",
    "architecture",
    "declared_capability",
    "installed_capability",
    "demonstrated_capability",
    "authority_requirements",
    "environment_status",
    "failure_condition",
    "security_implementation",
    "customer_private",
    "credential_bearing",
    "policy_thresholds",
];

/// The four high-risk classes that require explicit_high_risk + compensating
/// control (§8.2).
pub const HIGH_RISK_CLASSES: &[&str] = &[
    "security_implementation",
    "customer_private",
    "credential_bearing",
    "policy_thresholds",
];

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct SelfDisclosureGrant {
    pub name: String,
    pub actor_role: String,
    pub claim_classes: Vec<String>,
    #[serde(default)]
    pub explicit_high_risk: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compensating_control: Option<String>,
    /// Per-grant freshness override (implementation-defined default false).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub require_fresh_snapshot: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct SelfDisclosureSurface {
    #[serde(default = "deny_mode")]
    pub mode: String,
    #[serde(default)]
    pub grants: Vec<SelfDisclosureGrant>,
}

impl Default for SelfDisclosureSurface {
    fn default() -> Self {
        Self {
            mode: deny_mode(),
            grants: vec![],
        }
    }
}

impl SelfDisclosureSurface {
    /// Validate the surface (§8.2): mode must be deny-by-default, claim
    /// classes must be known, high-risk grants need friction.
    pub fn validate(&self) -> Result<(), ForgeError> {
        if self.mode != "deny-by-default" {
            return Err(schema(
                "self_disclosure.mode must be deny-by-default".into(),
            ));
        }
        for grant in &self.grants {
            for class in &grant.claim_classes {
                if !VALID_CLAIM_CLASSES.contains(&class.as_str()) {
                    return Err(schema(format!(
                        "unknown claim class '{class}' in self_disclosure grant '{}'",
                        grant.name
                    )));
                }
                if HIGH_RISK_CLASSES.contains(&class.as_str())
                    && (!grant.explicit_high_risk
                        || grant
                            .compensating_control
                            .as_deref()
                            .unwrap_or("")
                            .is_empty())
                {
                    return Err(schema(format!(
                        "high-risk class '{class}' in grant '{}' requires explicit_high_risk: true and a compensating control",
                        grant.name
                    )));
                }
            }
        }
        Ok(())
    }

    /// Returns true if `actor_role` is permitted to disclose `claim_class`.
    /// Absent surface ⇒ deny (§8.2).
    pub fn permits(&self, actor_role: &str, claim_class: &str) -> bool {
        self.grants.iter().any(|grant| {
            grant.actor_role == actor_role && grant.claim_classes.iter().any(|c| c == claim_class)
        })
    }
}

fn schema(message: String) -> ForgeError {
    ForgeError::Config {
        class: "schema_error",
        path: PathBuf::new(),
        message,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct PolicyRule {
    pub name: String,
    pub verdict: Verdict,
    pub actor_role: ActorRole,
    pub operation_kind: String,
    #[serde(default)]
    pub path_prefix: Option<String>,
    #[serde(default)]
    pub argv0: Option<String>,
    #[serde(default)]
    pub disposition: Option<NormalizedDisposition>,
    #[serde(default)]
    pub boundary_constraints: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub compensating_controls: Vec<String>,
    #[serde(default)]
    pub sandbox_class: Option<String>,
    #[serde(default)]
    pub requires_approval: Option<bool>,
    #[serde(default)]
    pub memory_scope: Option<String>,
    /// Environment reference `name@version` — when set, ExecuteCommand rules
    /// match the intersection of provides.commands and the rule's argv0 (§7.6).
    #[serde(default)]
    pub environment: Option<String>,
    #[serde(default)]
    pub transition_kind: Option<String>,
    #[serde(default)]
    pub from_stage: Option<String>,
    #[serde(default)]
    pub to_stage: Option<String>,
    #[serde(default)]
    pub modes: Option<Vec<String>>,
    #[serde(default)]
    pub gate_profile_ref: Option<String>,
    #[serde(default)]
    pub required_settlement_strength: Option<String>,
    #[serde(default)]
    pub qualifying_value_evidence_kinds: Option<Vec<String>>,
    #[serde(default)]
    pub license_allowlist: Vec<String>,
    #[serde(default, rename = "ledger")]
    pub attestation_ledger: Option<String>,
    #[serde(default)]
    pub requester_roles: Option<Vec<String>>,
    #[serde(default)]
    pub approver_roles: Option<Vec<String>>,
    #[serde(default)]
    pub degraded_mode: Option<String>,
}

impl AuthorityPolicyBundle {
    pub fn strong_settlement_authority(
        &self,
    ) -> Result<&SettlementAuthorityDescriptor, ForgeError> {
        let mut matching = self
            .settlement_authorities
            .iter()
            .filter(|descriptor| descriptor.may_issue_strong);
        let descriptor = matching.next().ok_or_else(|| ForgeError::Config {
            class: "settlement_authority_config_error",
            path: PathBuf::from("<policy>"),
            message: "no strong settlement authority is configured".into(),
        })?;
        if matching.next().is_some() {
            return Err(ForgeError::Config {
                class: "settlement_authority_config_error",
                path: PathBuf::from("<policy>"),
                message: "multiple strong settlement authorities are configured".into(),
            });
        }
        Ok(descriptor)
    }

    pub fn refresh_policy_bundle_hash(&mut self) -> Result<(), ForgeError> {
        self.policy_bundle_hash = None;
        let source_base = self.source_base.take();
        self.policy_bundle_hash = Some(hash_canonical(self)?);
        self.source_base = source_base;
        Ok(())
    }
    pub fn load_domainforge_model(&self) -> Result<Option<DomainModel>, ForgeError> {
        if !self
            .policy_engines
            .iter()
            .any(|engine| engine.engine == "domainforge")
        {
            return Ok(None);
        }
        let source = self
            .sources
            .iter()
            .find(|source| source.surface == "domain_model")
            .ok_or_else(|| {
                ForgeError::Input("DomainForge engine requires domain_model source".into())
            })?;
        let base = self
            .source_base
            .as_deref()
            .unwrap_or_else(|| Path::new("."));
        let path = base.join(&source.path);
        let content = fs::read_to_string(&path)
            .map_err(|error| ForgeError::io("read DomainForge model source", error))?;
        let uri = source.path.clone();
        sea_forge_domainforge::load_validate(&sea_forge_domainforge::SeaSourceSet {
            entry_uri: uri.clone(),
            files: vec![sea_forge_domainforge::SourceFile {
                uri,
                sha256: source.sha256.clone(),
                content,
            }],
        })
        .map(Some)
    }
    pub fn load(path: &Path) -> Result<Self, ForgeError> {
        let text = fs::read_to_string(path).map_err(|e| ForgeError::Config {
            class: "missing_config_error",
            path: path.into(),
            message: e.to_string(),
        })?;
        let yaml: serde_yaml::Value =
            serde_yaml::from_str(&text).map_err(|error| ForgeError::Config {
                class: "parse_error",
                path: path.into(),
                message: error.location().map_or_else(
                    || "invalid YAML".into(),
                    |location| {
                        format!(
                            "invalid YAML at line {}, column {}",
                            location.line(),
                            location.column()
                        )
                    },
                ),
            })?;
        let mut bundle: Self = serde_yaml::from_value(yaml).map_err(|_| ForgeError::Config {
            class: "schema_error",
            path: path.into(),
            message: "policy does not match the required schema".into(),
        })?;
        bundle.validate(path)?;
        if bundle.version == "0.2" {
            let base = bundle
                .source_base
                .clone()
                .unwrap_or_else(|| path.parent().unwrap_or_else(|| Path::new(".")).into());
            for source in &bundle.sources {
                let source_path = base.join(&source.path);
                let bytes = fs::read(&source_path)
                    .map_err(|error| ForgeError::io("read authority policy source", error))?;
                if source.sha256 != sha256_bytes(&bytes) {
                    return Err(ForgeError::Config {
                        class: "source_hash_mismatch",
                        path: source_path,
                        message: format!("authority source {} hash mismatch", source.surface),
                    });
                }
            }
            bundle.source_base = Some(base);
        }
        Ok(bundle)
    }
    pub fn resolve_identity(&self, principal: &str, role: ActorRole) -> IdentityBinding {
        if let Some(configured) = self
            .identity_bindings
            .iter()
            .find(|identity| identity.principal == principal && identity.role == role)
        {
            return IdentityBinding {
                identity_id: Some(format!("identity:{}", configured.principal)),
                principal: configured.principal.clone(),
                roles: vec![configured.role.clone()],
                actor_type: configured.actor_type.clone(),
                binding_resolution: BindingResolution::Exact,
                identity_binding_source: self.identity.source.clone(),
                source: Some(self.identity.source.clone()),
                sponsor: configured.sponsor.clone(),
                issued_at: None,
                expires_at: None,
                identity_binding_hash: Some(sha256_bytes(
                    format!("{}:{:?}", configured.principal, configured.role).as_bytes(),
                )),
            };
        }
        if self.version == RECORD_VERSION {
            return IdentityBinding {
                identity_id: Some(format!("identity:{principal}")),
                principal: principal.into(),
                roles: vec![role.clone()],
                actor_type: actor_type_for_role(&role),
                binding_resolution: BindingResolution::LocalDefault,
                identity_binding_source: self.identity.source.clone(),
                source: Some(self.identity.source.clone()),
                sponsor: None,
                issued_at: None,
                expires_at: None,
                identity_binding_hash: Some(sha256_bytes(
                    format!("{principal}:{role:?}").as_bytes(),
                )),
            };
        }
        IdentityBinding {
            identity_id: None,
            principal: principal.into(),
            roles: vec![role.clone()],
            actor_type: actor_type_for_role(&role),
            binding_resolution: BindingResolution::Unresolved,
            identity_binding_source: self.identity.source.clone(),
            source: Some(self.identity.source.clone()),
            sponsor: None,
            issued_at: None,
            expires_at: None,
            identity_binding_hash: None,
        }
    }
    fn validate(&self, path: &Path) -> Result<(), ForgeError> {
        let schema = |message: String| ForgeError::Config {
            class: "schema_error",
            path: path.into(),
            message,
        };
        if self.version != RECORD_VERSION && self.version != "0.2" {
            return Err(schema("version must equal 0.1 or 0.2".into()));
        }
        if self.version == "0.2" {
            const SURFACES: &[&str] = &[
                "authority_hooks",
                "identity_map",
                "domain_model",
                "file_access",
                "api_allowlist",
                "git_commit",
                "pr_merge",
                "prompt_risk",
                "memory_recall",
                "spec_pipeline",
                "artifact_transition",
                "attestation",
                "approval",
                "settlement_authority",
                "capability_promotion",
                "deployment",
                "secret_access",
                "policy_mutation",
                "evidence_mutation",
            ];
            const ROLES: &[&str] = &["R-DS", "R-AG", "R-LC", "R-SO", "R-RM", "R-DEV", "R-AA"];
            // (name, operation_kind, optional transition_kind selector)
            const SOD: &[(&str, &str, Option<&str>)] = &[
                ("production_proposer_approver", "pr_merge", None),
                (
                    "semantic_debt_requester_acceptor",
                    "settlement_authority_mutation",
                    None,
                ),
                ("break_glass_requester_approver", "policy_mutation", None),
                ("key_generator_approver", "identity_minting", None),
                (
                    "capitalization_requester_approver",
                    "transition_artifact_stage",
                    Some("capitalize"),
                ),
            ];
            if self.bundle_id.as_deref().is_none_or(str::is_empty)
                || self.policy_bundle_hash.as_deref().is_none_or(str::is_empty)
                || self
                    .policy_bundle_version
                    .as_deref()
                    .is_none_or(str::is_empty)
                || self.loaded_at.as_deref().is_none_or(str::is_empty)
                || SURFACES
                    .iter()
                    .any(|surface| !self.sources.iter().any(|source| source.surface == *surface))
                || ROLES.iter().any(|role| !self.roles.contains_key(*role))
                || ROLES.iter().any(|role| {
                    !self
                        .permissions
                        .iter()
                        .any(|permission| permission.role == *role)
                })
                || SOD.iter().any(|(name, operation, transition)| {
                    !self.sod_rules.iter().any(|rule| {
                        rule.name == *name
                            && rule.operation_kind.as_deref() == Some(*operation)
                            && rule.transition_kind.as_deref() == *transition
                    })
                })
                || !self.sod_rules.iter().any(|rule| {
                    rule.name == "break_glass_requester_approver" && rule.approver_role == "R-SO"
                })
            {
                return Err(schema("incomplete v0.2 authority policy bundle".into()));
            }
            let mut canonical = self.clone();
            canonical.policy_bundle_hash = None;
            canonical.source_base = None;
            let computed = hash_canonical(&canonical)
                .map_err(|error| schema(format!("cannot hash policy bundle: {error}")))?;
            if self.policy_bundle_hash.as_deref() != Some(computed.as_str()) {
                return Err(schema("policy_bundle_hash mismatch".into()));
            }
        }
        if self.identity.source.is_empty() || self.identity.allow_unresolved {
            return Err(schema("invalid identity policy".into()));
        }
        if self.default != "deny"
            || self.policy_surfaces.file.mode != "deny-by-default"
            || self.policy_surfaces.external_api.mode != "deny-by-default"
            || self.policy_surfaces.github_pr.default != "escalate"
            || self.policy_surfaces.prompt_risk.default != "allow-with-risk-escalation"
        {
            return Err(schema("invalid policy surface mode/default".into()));
        }
        // M11 (E13 §8.2): validate self_disclosure surface.
        self.policy_surfaces
            .self_disclosure
            .validate()
            .map_err(|e| ForgeError::Config {
                class: "schema_error",
                path: path.into(),
                message: e.to_string(),
            })?;
        let mut names = HashSet::new();
        let mut authority_refs = HashSet::new();
        for descriptor in &self.settlement_authorities {
            let invalid_common = descriptor.authority_ref.is_empty()
                || !authority_refs.insert(&descriptor.authority_ref)
                || descriptor.trust_anchor_ref.is_empty()
                || descriptor.declarer_actor_id.is_empty()
                || descriptor.permitted_declarer_roles.is_empty()
                || descriptor
                    .permitted_declarer_roles
                    .iter()
                    .any(String::is_empty)
                || descriptor.standing_basis.is_empty()
                || descriptor.timeout_secs == 0;
            let invalid_adapter = match descriptor.adapter.as_str() {
                "local" => descriptor.may_issue_strong || !descriptor.command.is_empty(),
                "swe_seed" => descriptor.command.first().is_none_or(String::is_empty),
                _ => true,
            };
            if invalid_common || invalid_adapter {
                return Err(ForgeError::Config {
                    class: "settlement_authority_config_error",
                    path: path.into(),
                    message: "invalid settlement authority descriptor".into(),
                });
            }
        }
        if self.policy_engines.iter().any(|engine| {
            engine.engine.is_empty()
                || engine.fail_mode != "closed"
                || !matches!(
                    engine.engine.as_str(),
                    "local" | "domainforge" | "opa" | "governedspeed" | "policy-gateway"
                )
        }) {
            return Err(ForgeError::Config {
                class: "authority_engine_config_error",
                path: path.into(),
                message: "action-gating engines must be known and fail_mode closed".into(),
            });
        }
        for rule in &self.sod_rules {
            if rule.name.is_empty()
                || rule.requester_role.is_empty()
                || rule.approver_role.is_empty()
                || rule.operation_kind.as_deref().is_none_or(str::is_empty)
                || rule.allow_same_principal
            {
                return Err(schema("invalid separation-of-duty rule".into()));
            }
            match (
                rule.operation_kind.as_deref(),
                rule.transition_kind.as_deref(),
            ) {
                (
                    Some("transition_artifact_stage"),
                    Some("synthesize" | "productize" | "capitalize"),
                ) => {}
                (Some("transition_artifact_stage"), Some(_)) => {
                    return Err(schema(
                        "unknown transition_kind on separation-of-duty rule".into(),
                    ));
                }
                (Some("transition_artifact_stage"), None) => {
                    return Err(schema(
                        "transition_artifact_stage SOD rules require transition_kind".into(),
                    ));
                }
                (_, Some(_)) => {
                    return Err(schema(
                        "transition_kind is only allowed on transition_artifact_stage SOD rules"
                            .into(),
                    ));
                }
                _ => {}
            }
        }
        for rule in &self.rules {
            if rule.name.is_empty() || !names.insert(&rule.name) {
                return Err(schema("rule names must be non-empty and unique".into()));
            }
            if !matches!(
                rule.operation_kind.as_str(),
                "write_file"
                    | "execute_command"
                    | "external_api"
                    | "git_commit"
                    | "github_pr"
                    | "recall_memory"
                    | "inspect_run"
                    | "validate_model"
                    | "install_extension"
                    | "adopt_extension"
                    | "disable_extension"
                    | "projection_execution"
                    | "approval_resolution"
                    | "policy_mutation"
                    | "evidence_mutation"
                    | "secret_access"
                    | "deployment"
                    | "rollback"
                    | "delete_file"
                    | "sandbox_execution"
                    | "pr_merge"
                    | "spec_mutation"
                    | "generated_zone_mutation"
                    | "settlement_authority_mutation"
                    | "settlement_declaration"
                    | "human_task_completion"
                    | "case_reopen"
                    | "case_terminate"
                    | "discretionary_task_add"
                    | "identity_minting"
                    | "artifact_transition"
                    | "attestation"
                    | "transition_artifact_stage"
                    | "attest_artifact_identity"
                    | "artifact_lifecycle"
                    | "review_artifact_rights"
                    | "run_spec_pipeline"
                    | "run_projection"
                    | "import_bundle"
                    | "export_bundle"
                    | "adopt_template"
            ) {
                return Err(ForgeError::Config {
                    class: "unsupported_kind_error",
                    path: path.into(),
                    message: "policy contains an unsupported operation_kind".into(),
                });
            }
            if rule.operation_kind == "execute_command"
                && rule.verdict == Verdict::Allow
                && rule.sandbox_class.as_deref().unwrap_or("local") == "local"
                && !rule.argv0.as_deref().is_some_and(trusted_policy_argv0)
            {
                return Err(schema(
                    "local allow execute_command requires trusted argv0 sea-forge".into(),
                ));
            }
            if rule
                .sandbox_class
                .as_deref()
                .is_some_and(|class| !matches!(class, "local" | "jail" | "microvm"))
            {
                return Err(schema("unknown sandbox class".into()));
            }
            if rule.disposition == Some(NormalizedDisposition::Degraded)
                && (!self.allow_degraded || rule.compensating_controls.is_empty())
            {
                return Err(schema(
                    "degraded rule requires explicit policy permission and controls".into(),
                ));
            }
            if rule.compensating_controls.iter().any(|control| {
                !matches!(
                    control.as_str(),
                    "audit" | "short_timeout" | "minimal_environment"
                )
            }) {
                return Err(schema("unsupported compensating control".into()));
            }
            if rule.disposition == Some(NormalizedDisposition::Boundary)
                && rule.boundary_constraints.is_empty()
            {
                return Err(schema("boundary rule requires constraints".into()));
            }
            if rule.boundary_constraints.keys().any(|dimension| {
                !matches!(
                    dimension.as_str(),
                    "workspace" | "artifacts_root" | "timeout_secs" | "env_keys" | "sandbox_class"
                )
            }) {
                return Err(schema("unsupported boundary dimension".into()));
            }
            if rule.operation_kind == "transition_artifact_stage" {
                let transition = rule.transition_kind.as_deref();
                let from = rule.from_stage.as_deref();
                let to = rule.to_stage.as_deref();
                let legal = matches!(
                    (transition, from, to),
                    (Some("synthesize"), Some("cognitive"), Some("intellectual"))
                        | (Some("productize"), Some("intellectual"), Some("product"))
                        | (Some("capitalize"), Some("product"), Some("capital"))
                );
                let modes = rule.modes.as_deref().unwrap_or_default();
                let evidence = rule
                    .qualifying_value_evidence_kinds
                    .as_deref()
                    .unwrap_or_default();
                if !legal
                    || modes.is_empty()
                    || modes
                        .iter()
                        .any(|mode| mode != "derive" && mode != "promote")
                    || rule.gate_profile_ref.as_deref().is_none_or(str::is_empty)
                    || rule
                        .required_settlement_strength
                        .as_deref()
                        .is_none_or(|strength| !matches!(strength, "local" | "strong"))
                    || rule.qualifying_value_evidence_kinds.is_none()
                    || rule.requires_approval.is_none()
                    || (transition == Some("capitalize")
                        && (rule.requires_approval != Some(true)
                            || modes != ["promote"]
                            || evidence.is_empty()))
                {
                    return Err(schema("invalid artifact transition policy rule".into()));
                }
            }
            if rule.operation_kind == "attest_artifact_identity"
                && (rule.attestation_ledger.as_deref().is_none_or(str::is_empty)
                    || rule.requester_roles.as_ref().is_none_or(Vec::is_empty)
                    || rule.approver_roles.as_ref().is_none_or(Vec::is_empty)
                    || rule
                        .degraded_mode
                        .as_deref()
                        .is_none_or(|mode| !matches!(mode, "forbidden" | "pre_mint_only")))
            {
                return Err(schema("invalid artifact attestation policy rule".into()));
            }
        }
        Ok(())
    }
}

fn trusted_policy_argv0(argv0: &str) -> bool {
    if argv0 == "sea-forge" {
        return true;
    }
    #[cfg(test)]
    {
        std::env::current_exe()
            .ok()
            .and_then(|path| path.file_name().map(|name| name.to_owned()))
            .is_some_and(|name| name == argv0)
    }
    #[cfg(not(test))]
    false
}

pub struct PolicyAuthorityEngine {
    bundle: AuthorityPolicyBundle,
    bundle_hash: String,
    issued_decisions: Mutex<HashSet<String>>,
    opaque_constraints: Mutex<BTreeMap<String, OpaqueConstraint>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OpaqueConstraint {
    pub constraint_id: String,
    pub resource_type: String,
    pub resource_id: String,
    pub created_by_decision_id: String,
    pub reason: String,
    pub created_at: String,
    pub expires_at: Option<String>,
}

pub struct AuthorityEvaluation<'a> {
    pub actor: &'a Actor,
    pub binding: IdentityBinding,
    pub run_id: &'a str,
    pub case_id: &'a str,
    pub plan_item_id: &'a str,
    pub sequence: usize,
    pub action: &'a AuthorityAction,
    pub workspace_root: &'a Path,
    pub evidence_refs: Vec<String>,
    pub artifacts_root: Option<&'a Path>,
    pub timeout_secs: Option<u64>,
    pub env_keys: BTreeSet<String>,
    pub domainforge_candidate: Option<&'a DomainForgeCandidate>,
    /// `(item_env_ref, provides_commands)` — when the PlanItem declares an
    /// environment, pass its reference and the spec's `provides.commands` list
    /// so policy rules with `environment:` can enforce intersection (§7.6).
    pub environment: Option<(&'a str, &'a [String])>,
}
impl PolicyAuthorityEngine {
    pub fn new(bundle: AuthorityPolicyBundle) -> Result<Self, ForgeError> {
        bundle.validate(Path::new("<in-memory-policy>"))?;
        let bundle_hash = hash_canonical(&bundle)?;
        Ok(Self {
            bundle,
            bundle_hash,
            issued_decisions: Mutex::new(HashSet::new()),
            opaque_constraints: Mutex::new(BTreeMap::new()),
        })
    }
    pub fn load_opaque_constraints(
        &self,
        constraints: impl IntoIterator<Item = OpaqueConstraint>,
    ) -> Result<(), ForgeError> {
        let mut stored = self
            .opaque_constraints
            .lock()
            .map_err(|_| ForgeError::Internal("opaque constraint state poisoned".into()))?;
        for constraint in constraints {
            if constraint
                .expires_at
                .as_deref()
                .map(chrono::DateTime::parse_from_rfc3339)
                .transpose()
                .map_err(|_| ForgeError::Input("opaque constraint expiry is invalid".into()))?
                .is_some_and(|expiry| expiry.with_timezone(&Utc) < Utc::now())
            {
                continue;
            }
            stored.insert(
                format!("{}:{}", constraint.resource_type, constraint.resource_id),
                constraint,
            );
        }
        Ok(())
    }
    pub fn evaluate(
        &self,
        input: AuthorityEvaluation<'_>,
    ) -> Result<AuthorityDecision, ForgeError> {
        self.evaluate_at(input, random_id("act")?, Utc::now().to_rfc3339())
    }

    pub fn grant(
        &self,
        decision: &AuthorityDecision,
        committed: &CommittedRecordRef,
        action: &AuthorityAction,
        assurance: Option<&PreActionAssurance>,
    ) -> Result<ActionGrant, ForgeError> {
        self.grant_exact(
            decision,
            committed,
            action,
            assurance,
            Verdict::Allow,
            decision.sandbox_class_granted.clone(),
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn grant_after_approval(
        &self,
        decision: &AuthorityDecision,
        committed_decision: &CommittedRecordRef,
        action: &AuthorityAction,
        approval: &ApprovalRequest,
        committed_approval: &CommittedRecordRef,
        criteria: &SettlementCriteriaRecord,
        committed_criteria: &CommittedRecordRef,
        assurance: Option<&PreActionAssurance>,
        stream: &LedgerStream,
    ) -> Result<ActionGrant, ForgeError> {
        self.grant_after_approval_inner(
            decision,
            committed_decision,
            action,
            approval,
            committed_approval,
            criteria,
            committed_criteria,
            assurance,
            stream,
            false,
        )
    }

    /// Idempotent variant for resume retries: when the approval was already
    /// consumed by a prior grant (e.g. a parked resume that minted the grant
    /// but could not complete the side effect), re-derive the same grant
    /// instead of rejecting. Strict double-spend callers use
    /// [`grant_after_approval`]; the protected side effect stays idempotent.
    #[allow(clippy::too_many_arguments)]
    pub fn grant_after_approval_idempotent(
        &self,
        decision: &AuthorityDecision,
        committed_decision: &CommittedRecordRef,
        action: &AuthorityAction,
        approval: &ApprovalRequest,
        committed_approval: &CommittedRecordRef,
        criteria: &SettlementCriteriaRecord,
        committed_criteria: &CommittedRecordRef,
        assurance: Option<&PreActionAssurance>,
        stream: &LedgerStream,
    ) -> Result<ActionGrant, ForgeError> {
        self.grant_after_approval_inner(
            decision,
            committed_decision,
            action,
            approval,
            committed_approval,
            criteria,
            committed_criteria,
            assurance,
            stream,
            true,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn grant_after_approval_inner(
        &self,
        decision: &AuthorityDecision,
        committed_decision: &CommittedRecordRef,
        action: &AuthorityAction,
        approval: &ApprovalRequest,
        committed_approval: &CommittedRecordRef,
        criteria: &SettlementCriteriaRecord,
        committed_criteria: &CommittedRecordRef,
        assurance: Option<&PreActionAssurance>,
        stream: &LedgerStream,
        idempotent: bool,
    ) -> Result<ActionGrant, ForgeError> {
        if decision.verdict != Verdict::Escalate
            || decision.normalized_disposition != NormalizedDisposition::Escalate
            || committed_decision.record_kind() != "authority_decision"
            || committed_approval.record_kind() != "approval_resolution"
            || committed_criteria.record_kind() != "settlement_criteria"
            || committed_decision.ledger_id() != committed_approval.ledger_id()
            || committed_decision.ledger_id() != committed_criteria.ledger_id()
        {
            return Err(ForgeError::Input(
                "approval grant requires exact records from one ledger".into(),
            ));
        }
        if committed_approval.payload_hash() != payload_hash(&serde_json::to_value(approval)?)?
            || committed_criteria.payload_hash() != payload_hash(&serde_json::to_value(criteria)?)?
        {
            return Err(ForgeError::Input(
                "approval or criteria record is not bound to its committed payload".into(),
            ));
        }
        let case_id =
            decision.audit_record.case_id.as_deref().ok_or_else(|| {
                ForgeError::Input("authority decision has no case context".into())
            })?;
        let requester = decision
            .action_request
            .actor
            .get("actor_id")
            .and_then(Value::as_str)
            .ok_or_else(|| ForgeError::Input("authority requester is malformed".into()))?;
        if approval.status != ApprovalStatus::Approved
            || approval.decision_id != decision.decision_id
            || approval.case_id != case_id
            || approval.run_id != decision.run_id
            || approval.plan_item_id != decision.plan_item_id
            || approval.criteria_ref.as_deref() != Some(criteria.criteria_id.as_str())
            || approval.criteria_sha256.as_deref() != Some(criteria.criteria_sha256.as_str())
            || approval.criteria_record_hash.as_deref()
                != Some(criteria.criteria_record_hash.as_str())
            || approval
                .resolved_by
                .as_deref()
                .is_none_or(|resolver| resolver.is_empty() || resolver == requester)
        {
            return Err(ForgeError::Input(
                "approval resolution does not match the escalated decision".into(),
            ));
        }
        let expected_criteria_sha = hash_canonical(&criteria.criteria)?;
        let mut unhashed_criteria = criteria.clone();
        unhashed_criteria.criteria_record_hash.clear();
        if criteria.criteria_sha256 != expected_criteria_sha
            || criteria.criteria_record_hash != hash_canonical(&unhashed_criteria)?
        {
            return Err(ForgeError::Input(
                "approval criteria hashes are invalid".into(),
            ));
        }
        let expires_at = chrono::DateTime::parse_from_rfc3339(&approval.expires_at)
            .map_err(|_| ForgeError::Input("approval expiry is invalid".into()))?
            .with_timezone(&Utc);
        let resolved_at = approval
            .resolved_at
            .as_deref()
            .ok_or_else(|| ForgeError::Input("approved resolution has no timestamp".into()))
            .and_then(|value| {
                chrono::DateTime::parse_from_rfc3339(value).map_err(|_| {
                    ForgeError::Input("approval resolution timestamp is invalid".into())
                })
            })?
            .with_timezone(&Utc);
        // An approval resolved Approved before expires_at remains valid after
        // wall clock TTL; expiry only applies to unresolved pending or to a
        // resolution recorded after the deadline.
        if resolved_at > expires_at {
            return Err(ForgeError::Input("approval resolution expired".into()));
        }
        let rule = decision
            .matched_rule
            .as_deref()
            .and_then(|name| self.bundle.rules.iter().find(|rule| rule.name == name))
            .ok_or_else(|| {
                ForgeError::Input("authority decision was not escalated for approval".into())
            })?;
        // Re-derive matching SOD from the exact action and the matched rule's
        // actor role so grant validation agrees with initial evaluation.
        let rule_actor_role = serde_json::to_value(&rule.actor_role)?
            .as_str()
            .unwrap_or_default()
            .to_owned();
        let matching_sod = matching_sod_rules(&self.bundle.sod_rules, &rule_actor_role, action);
        let escalated_for_approval =
            rule.requires_approval == Some(true) || !matching_sod.is_empty();
        if !escalated_for_approval {
            return Err(ForgeError::Input(
                "authority decision was not escalated for approval".into(),
            ));
        }
        // SOD-only and SOD-backed escalations require non-empty approver roles
        // and a resolver whose bound role matches those roles.
        if !matching_sod.is_empty() {
            let required_roles = matching_sod
                .iter()
                .map(|sod| sod.approver_role.as_str())
                .filter(|role| !role.is_empty())
                .collect::<Vec<_>>();
            if required_roles.is_empty() {
                return Err(ForgeError::Input(
                    "approval requires non-empty required_approver_roles".into(),
                ));
            }
            let resolver_id = approval.resolved_by.as_deref().ok_or_else(|| {
                ForgeError::Input(
                    "approval resolution does not match the escalated decision".into(),
                )
            })?;
            let resolver_role = self
                .bundle
                .identity_bindings
                .iter()
                .find(|binding| binding.principal == resolver_id)
                .map(|binding| binding.role.clone())
                .ok_or_else(|| {
                    ForgeError::Input("approval resolver is not bound in policy identity".into())
                })?;
            let resolver_role_name = serde_json::to_value(&resolver_role)?
                .as_str()
                .unwrap_or_default()
                .to_owned();
            if !required_roles
                .iter()
                .any(|role| *role == resolver_role_name)
            {
                return Err(ForgeError::Input(
                    "approval resolver role does not match SOD approver roles".into(),
                ));
            }
        }
        // Ledger-backed consumption keyed by semantic decision+approval identity.
        // Fail-closed if grant minting fails after the consumption append.
        let consumption_key = format!("{}:{}", decision.decision_id, approval.approval_id);
        let consumption = json!({
            "decision_id": decision.decision_id,
            "approval_id": approval.approval_id,
            "consumed_at": Utc::now().to_rfc3339(),
        });
        match stream.commit_typed_new(
            "approval_grant_consumption",
            &consumption_key,
            vec![decision.decision_id.clone(), approval.approval_id.clone()],
            &consumption,
            vec![
                committed_decision.entry_ulid().into(),
                committed_approval.entry_ulid().into(),
            ],
        ) {
            Ok(_) => {}
            Err(ForgeError::Internal(message))
                if message.contains("idempotency key already committed") && idempotent =>
            {
                // Resume retry: a prior grant already consumed this approval;
                // fall through and re-derive the same grant.
            }
            Err(ForgeError::Internal(message))
                if message.contains("idempotency key already committed") =>
            {
                return Err(ForgeError::Input(
                    "approval resolution was already granted".into(),
                ));
            }
            Err(other) => return Err(other),
        }
        // The grant window is a fresh five minutes; the approval's expiry only
        // shortens it when it still lies in the future.
        let grant_window = Utc::now() + chrono::Duration::minutes(5);
        let grant_expiry = if expires_at > Utc::now() {
            std::cmp::min(grant_window, expires_at)
        } else {
            grant_window
        };
        self.grant_exact(
            decision,
            committed_decision,
            action,
            assurance,
            Verdict::Escalate,
            Some(rule.sandbox_class.clone().unwrap_or_else(|| "local".into())),
            Some(grant_expiry),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn grant_exact(
        &self,
        decision: &AuthorityDecision,
        committed: &CommittedRecordRef,
        action: &AuthorityAction,
        assurance: Option<&PreActionAssurance>,
        expected_verdict: Verdict,
        sandbox_class: Option<String>,
        approval_expiry: Option<chrono::DateTime<Utc>>,
    ) -> Result<ActionGrant, ForgeError> {
        if committed.record_kind() != "authority_decision" {
            return Err(ForgeError::Input(
                "authority decision has the wrong committed record kind".into(),
            ));
        }
        let is_read = matches!(
            action,
            AuthorityAction::Reserved { resource_type, .. }
                if matches!(resource_type.as_str(), "recall_memory" | "inspect_run" | "validate_model")
        );
        if self.bundle.integrity_ledger.required_for_side_effects && !is_read && assurance.is_none()
        {
            return Err(ForgeError::Internal(
                "ledger_integrity_error: required pre-action checkpoint assurance unavailable"
                    .into(),
            ));
        }
        if self.bundle.integrity_ledger.required_for_side_effects
            && !is_read
            && !assurance.is_some_and(|proof| proof.covers(committed))
        {
            return Err(ForgeError::Internal(
                "ledger_integrity_error: assurance does not cover authority decision".into(),
            ));
        }
        let action_hash = hash_canonical(action)?;
        let bound_hash = decision
            .action_request
            .evidence
            .get("payload_hash")
            .and_then(Value::as_str);
        if decision.verdict != expected_verdict || bound_hash != Some(action_hash.as_str()) {
            return Err(ForgeError::Input(
                "authority decision does not grant this action".into(),
            ));
        }
        if committed.payload_hash() != payload_hash(&serde_json::to_value(decision)?)? {
            return Err(ForgeError::Input(
                "authority decision is not bound to its committed record".into(),
            ));
        }
        if decision.determinism.policy_bundle_hash != self.bundle_hash {
            return Err(ForgeError::Input(
                "authority decision belongs to another policy context".into(),
            ));
        }
        let context = decision
            .action_request
            .context
            .as_object()
            .ok_or_else(|| ForgeError::Input("authority context is malformed".into()))?;
        let context_string = |key: &str| {
            context
                .get(key)
                .and_then(Value::as_str)
                .ok_or_else(|| ForgeError::Input(format!("authority context missing {key}")))
        };
        let context_run_id = context_string("run_id")?;
        let context_plan_item_id = context_string("plan_item_id")?;
        if decision.run_id != context_run_id || decision.plan_item_id != context_plan_item_id {
            return Err(ForgeError::Input(
                "authority decision is detached from its context".into(),
            ));
        }
        let workspace_root = PathBuf::from(context_string("workspace_root")?);
        let artifacts_root = context
            .get("artifacts_root")
            .and_then(Value::as_str)
            .map(PathBuf::from);
        let timeout_secs = context.get("timeout_secs").and_then(Value::as_u64);
        let env_keys = context
            .get("env_keys")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect();
        let expires_at = if let Some(expiry) = approval_expiry {
            expiry
        } else {
            let decision_hash = issued_decision_key(decision)?;
            if !self
                .issued_decisions
                .lock()
                .map_err(|_| ForgeError::Internal("authority issuer state poisoned".into()))?
                .remove(&decision_hash)
            {
                return Err(ForgeError::Input(
                    "authority decision was not issued by this mediator or was replayed".into(),
                ));
            }
            chrono::DateTime::parse_from_rfc3339(&decision.decided_at)
                .map_err(|_| ForgeError::Input("authority decision timestamp is invalid".into()))?
                .with_timezone(&Utc)
                + chrono::Duration::minutes(5)
        };
        if Utc::now() > expires_at {
            return Err(ForgeError::Input("authority decision expired".into()));
        }
        Ok(ActionGrant {
            action: action.clone(),
            run_id: decision.run_id.clone(),
            plan_item_id: decision.plan_item_id.clone(),
            workspace_root,
            artifacts_root,
            timeout_secs,
            env_keys,
            _assurance_ref: assurance.map(|value| value.global_checkpoint_hash().into()),
            sandbox_class: sandbox_class
                .ok_or_else(|| ForgeError::Input("authority decision grants no sandbox".into()))?,
            boundaries: decision.boundary_constraints.clone(),
            compensating_controls: decision.compensating_controls.clone(),
            expires_at,
        })
    }

    fn evaluate_at(
        &self,
        input: AuthorityEvaluation<'_>,
        action_id: String,
        timestamp: String,
    ) -> Result<AuthorityDecision, ForgeError> {
        let AuthorityEvaluation {
            actor,
            binding,
            run_id,
            case_id,
            plan_item_id,
            sequence,
            action,
            workspace_root,
            evidence_refs,
            artifacts_root,
            timeout_secs,
            env_keys,
            domainforge_candidate,
            environment,
        } = input;
        let (kind, resource_type, resource_id, parameters) = match action {
            AuthorityAction::WriteFile { path, content_hint } => (
                "write_file",
                "file",
                path.clone(),
                json!({"path":path,"content_sha256":sha256_bytes(content_hint.as_bytes())}),
            ),
            AuthorityAction::ExecuteCommand { argv, cwd } => (
                "execute_command",
                "shell_cmd",
                argv.first().cloned().unwrap_or_default(),
                json!({"argv":argv,"cwd":cwd}),
            ),
            AuthorityAction::ExternalApi { host } => (
                "external_api",
                "external_api",
                host.clone(),
                json!({"host":host}),
            ),
            AuthorityAction::GitCommit { paths } => (
                "git_commit",
                "git_commit",
                "working-tree".into(),
                json!({"paths":paths}),
            ),
            AuthorityAction::GithubPr {
                has_required_evidence,
            } => (
                "github_pr",
                "github_pr",
                "pull-request".into(),
                json!({"has_required_evidence":has_required_evidence}),
            ),
            AuthorityAction::Reserved {
                resource_type,
                resource_id,
                parameters,
            } => (
                "reserved",
                resource_type.as_str(),
                resource_id.clone(),
                parameters.clone(),
            ),
            AuthorityAction::Unclassified {
                raw_kind,
                parameters,
            } => (
                raw_kind.as_str(),
                "unclassified",
                raw_kind.clone(),
                parameters.clone(),
            ),
        };
        let request = AuthorityRequest {
            schema_version: "cam.v1".into(),
            action_id: action_id.clone(),
            correlation_id: run_id.into(),
            timestamp_utc: timestamp.clone(),
            actor: json!({"actor_id":actor.actor_id,"actor_type":binding.actor_type,"principal":binding.principal}),
            action: json!({"tool_name":"sea-forge-cli","operation":kind,"resource_type":resource_type,"resource_id":resource_id,"parameters":parameters}),
            context: json!({"repo":null,"branch":null,"environment":"local-slice","run_id":run_id,"plan_item_id":plan_item_id,"workspace_root":workspace_root,"artifacts_root":artifacts_root,"timeout_secs":timeout_secs,"env_keys":env_keys,"domain_model_ref":domainforge_candidate.map(|candidate| &candidate.model_ref),"source_platform":"cli","channel":"cli"}),
            evidence: json!({"identity_binding_source":binding.identity_binding_source,"tool_trace_ref":null,"payload_hash":hash_canonical(action)?}),
        };
        let expected_binding = self
            .bundle
            .resolve_identity(&actor.actor_id, actor.role.clone());
        let identity_not_expired = binding
            .expires_at
            .as_deref()
            .map(chrono::DateTime::parse_from_rfc3339)
            .transpose()
            .map_err(|_| ForgeError::Input("identity expiry is invalid".into()))?
            .is_none_or(|expiry| expiry.with_timezone(&Utc) >= Utc::now());
        let identity_valid = binding == expected_binding
            && binding.principal == actor.actor_id
            && binding.actor_type == actor_type_for_role(&actor.role)
            && binding.identity_binding_source == self.bundle.identity.source
            && (self.bundle.version == RECORD_VERSION || binding.identity_binding_hash.is_some())
            && identity_not_expired
            && (!matches!(actor.role, ActorRole::Agent | ActorRole::AutomatedAgent)
                || binding.sponsor.is_some());
        let existing_constraint = self
            .opaque_constraints
            .lock()
            .map_err(|_| ForgeError::Internal("opaque constraint state poisoned".into()))?
            .get(&format!("{resource_type}:{resource_id}"))
            .cloned();
        let (mut verdict, matched, mut reasons, refs, mut next) = if let Some(constraint) =
            &existing_constraint
        {
            (
                Verdict::Escalate,
                None,
                vec!["opaque_constraint_active".into()],
                vec![format!("opaque-constraint:{}", constraint.constraint_id)],
                vec!["resolve-governance-ambiguity".into()],
            )
        } else if binding.binding_resolution == BindingResolution::Unresolved || !identity_valid {
            (
                Verdict::Escalate,
                None,
                vec!["identity_unresolved".into()],
                vec!["identity-binding:unresolved".into()],
                vec!["complete-onboarding".into()],
            )
        } else if matches!(action, AuthorityAction::Unclassified { .. }) || malformed_action(action)
        {
            (
                Verdict::Deny,
                None,
                vec!["unclassified".into()],
                vec!["authority:default-deny".into()],
                vec![],
            )
        } else if untrusted_executable(action) {
            (
                Verdict::Deny,
                Some("trusted-self-executable".into()),
                vec!["untrusted_executable".into()],
                vec!["shell-cmd-policy:deny".into()],
                vec![],
            )
        } else if hard_denied(action, &self.bundle.policy_surfaces.file.deny_write) {
            (
                Verdict::Deny,
                Some("policy_surfaces.file.deny_write".into()),
                vec!["file_policy_deny".into(), "generated_zone_denied".into()],
                vec!["file-access-policy:deny".into()],
                vec![],
            )
        } else if let AuthorityAction::ExternalApi { host } = action {
            if self
                .bundle
                .policy_surfaces
                .external_api
                .allow_hosts
                .contains(host)
            {
                (
                    Verdict::Escalate,
                    Some("policy_surfaces.external_api.allow_hosts".into()),
                    vec!["unsupported_action_surface".into()],
                    vec!["api-allowlist:allow".into()],
                    vec!["extend-authority-policy".into()],
                )
            } else {
                (
                    Verdict::Deny,
                    Some("policy_surfaces.external_api.default".into()),
                    vec!["external_api_denied".into(), "unknown_api_host".into()],
                    vec!["api-allowlist:default".into()],
                    vec!["extend-authority-policy".into()],
                )
            }
        } else if protected_git_commit(
            action,
            &self.bundle.policy_surfaces.git_commit.protected_paths,
        ) {
            (
                Verdict::Deny,
                Some("policy_surfaces.git_commit.protected_paths".into()),
                vec!["protected_governance_path".into()],
                vec!["git-commit-policy:deny".into()],
                vec![],
            )
        } else if self.bundle.version == "0.1"
            && matches!(
                action,
                AuthorityAction::Reserved { resource_type, .. }
                    if matches!(resource_type.as_str(), "recall_memory" | "inspect_run" | "validate_model")
            )
        {
            (
                Verdict::Allow,
                Some("legacy-read-compatibility".into()),
                vec!["legacy_read_allow".into()],
                vec!["minimum-spec:read-compatibility".into()],
                vec![],
            )
        } else if matches!(
            action,
            AuthorityAction::GitCommit { .. } | AuthorityAction::GithubPr { .. }
        ) {
            (
                Verdict::Escalate,
                Some(format!("authority_surface:{resource_type}")),
                vec!["unsupported_action_surface".into()],
                vec![format!("{resource_type}:unsupported")],
                vec!["extend-authority-policy".into()],
            )
        } else if let Some(rule) = self.bundle.rules.iter().find(|r| {
            if matches_rule(r, actor, action) {
                // For ExecuteCommand, the argv0 + environment intersection
                // is enforced by command_allowed (§7.6 three axes).
                if let AuthorityAction::ExecuteCommand { .. } = action {
                    command_allowed(r, action, environment)
                } else {
                    true
                }
            } else {
                false
            }
        }) {
            let role_name = serde_json::to_value(&actor.role)?
                .as_str()
                .unwrap_or_default()
                .to_owned();
            let has_permission = self.bundle.version == RECORD_VERSION
                || self.bundle.permissions.iter().any(|permission| {
                    permission.role == role_name
                        && (permission.operation_kind == "*"
                            || permission.operation_kind == rule.operation_kind)
                });
            let matching_sod = matching_sod_rules(&self.bundle.sod_rules, &role_name, action);
            let sod_requires_approval = !matching_sod.is_empty();
            let effective_verdict = if !has_permission {
                Verdict::Deny
            } else if rule.requires_approval.unwrap_or(false) || sod_requires_approval {
                Verdict::Escalate
            } else {
                rule.verdict.clone()
            };
            (
                effective_verdict.clone(),
                Some(rule.name.clone()),
                vec![format!(
                    "policy_{}",
                    match effective_verdict {
                        Verdict::Allow => "allow",
                        Verdict::Deny => "deny",
                        Verdict::Escalate => "escalate",
                    }
                )],
                std::iter::once(format!("local-policy:{}", rule.name))
                    .chain(matching_sod.iter().map(|sod| format!("sod: {}", sod.name)))
                    .collect(),
                if effective_verdict == Verdict::Escalate {
                    std::iter::once("require-human-approval".into())
                        .chain(
                            matching_sod
                                .iter()
                                .map(|sod| format!("approval-role:{}", sod.approver_role)),
                        )
                        .collect()
                } else {
                    vec![]
                },
            )
        } else {
            (
                Verdict::Deny,
                None,
                vec!["default_deny".into()],
                vec!["file-access-policy:deny".into()],
                vec![],
            )
        };
        let matched_policy_rule = matched
            .as_deref()
            .and_then(|name| self.bundle.rules.iter().find(|rule| rule.name == name));
        let local_disposition = if verdict == Verdict::Allow {
            matched_policy_rule
                .and_then(|rule| rule.disposition.as_ref())
                .map(|disposition| match disposition {
                    NormalizedDisposition::Allow => GovernanceDisposition::Allow,
                    NormalizedDisposition::Deny => GovernanceDisposition::Deny,
                    NormalizedDisposition::Escalate => GovernanceDisposition::Escalate,
                    NormalizedDisposition::Boundary => GovernanceDisposition::Boundary,
                    NormalizedDisposition::Degraded => GovernanceDisposition::Degraded,
                })
                .unwrap_or(GovernanceDisposition::Allow)
        } else {
            match verdict {
                Verdict::Allow => GovernanceDisposition::Allow,
                Verdict::Deny => GovernanceDisposition::Deny,
                Verdict::Escalate => GovernanceDisposition::Escalate,
            }
        };
        let mut candidates = vec![GovernanceVerdict {
            engine: "local".into(),
            disposition: local_disposition,
            boundaries: matched_policy_rule
                .map(|rule| {
                    rule.boundary_constraints
                        .iter()
                        .map(|(key, values)| (key.clone(), values.iter().cloned().collect()))
                        .collect()
                })
                .unwrap_or_default(),
            compensating_controls: matched_policy_rule
                .map(|rule| rule.compensating_controls.iter().cloned().collect())
                .unwrap_or_default(),
            reason: reasons.join(","),
            evidence_refs: if refs.is_empty() {
                vec!["local-policy:default".into()]
            } else {
                refs.clone()
            },
        }];
        for configured in self
            .bundle
            .policy_engines
            .iter()
            .filter(|engine| engine.engine != "local")
        {
            let disposition = if configured.engine == "domainforge" {
                match domainforge_candidate {
                    Some(candidate)
                        if candidate.trace.raw_decision == "NotApplicable"
                            && !configured.required =>
                    {
                        continue
                    }
                    Some(candidate) if candidate.trace.raw_decision == "NotApplicable" => {
                        GovernanceDisposition::Deny
                    }
                    Some(candidate) => match candidate.disposition {
                        CandidateDisposition::Allow => GovernanceDisposition::Allow,
                        CandidateDisposition::Deny => GovernanceDisposition::Deny,
                        CandidateDisposition::Escalate => GovernanceDisposition::Escalate,
                    },
                    _ if configured.required => GovernanceDisposition::Deny,
                    _ => GovernanceDisposition::Escalate,
                }
            } else if configured.required {
                GovernanceDisposition::Deny
            } else {
                GovernanceDisposition::Escalate
            };
            candidates.push(GovernanceVerdict {
                engine: configured.engine.clone(),
                disposition,
                boundaries: BTreeMap::new(),
                compensating_controls: BTreeSet::new(),
                reason: if configured.engine == "domainforge" && domainforge_candidate.is_some() {
                    "domainforge candidate normalized".into()
                } else {
                    "configured authority engine unavailable".into()
                },
                evidence_refs: domainforge_candidate
                    .filter(|_| configured.engine == "domainforge")
                    .map(|candidate| candidate.trace.evidence_refs.clone())
                    .unwrap_or_else(|| vec![format!("engine:{}:candidate", configured.engine)]),
            });
        }
        candidates.sort();
        let resolved = resolve_candidates(
            &candidates,
            &ResolutionPolicy {
                allow_degraded: self.bundle.allow_degraded,
            },
        );
        verdict = match resolved.disposition {
            GovernanceDisposition::Allow => Verdict::Allow,
            GovernanceDisposition::Escalate => {
                next.push("resolve-authority-engine".into());
                Verdict::Escalate
            }
            GovernanceDisposition::Deny => Verdict::Deny,
            GovernanceDisposition::Boundary | GovernanceDisposition::Degraded => Verdict::Allow,
        };
        if candidates.len() > 1 {
            reasons.push("candidate_resolution".into());
        }
        let disposition = match resolved.disposition {
            GovernanceDisposition::Allow => NormalizedDisposition::Allow,
            GovernanceDisposition::Deny => NormalizedDisposition::Deny,
            GovernanceDisposition::Escalate => NormalizedDisposition::Escalate,
            GovernanceDisposition::Boundary => NormalizedDisposition::Boundary,
            GovernanceDisposition::Degraded => NormalizedDisposition::Degraded,
        };
        let reason = reasons.join(",");
        let action_request_hash = hash_canonical(&request)?;
        let identity_binding_hash = hash_canonical(&binding)?;
        let decision_id = format!("auth_{sequence:02}");
        let decision = AuthorityDecision {
            version: "0.2".into(),
            decision_id: decision_id.clone(),
            run_id: run_id.into(),
            plan_item_id: plan_item_id.into(),
            action_id: action_id.clone(),
            correlation_id: run_id.into(),
            operation: redacted_action(action),
            outcome: verdict.clone(),
            verdict: verdict.clone(),
            normalized_disposition: disposition.clone(),
            matched_rule: matched,
            reason_codes: reasons,
            reason: reason.clone(),
            policy_refs: refs.clone(),
            required_next_steps: next,
            identity_binding: binding.clone(),
            determinism: Determinism {
                policy_bundle_hash: self.bundle_hash.clone(),
                action_request_hash: action_request_hash.clone(),
                identity_binding_hash: identity_binding_hash.clone(),
            },
            action_request: request,
            audit_record: AuditRecord {
                engine: "sea-forge-authority".into(),
                disposition: match disposition {
                    NormalizedDisposition::Allow => "allow",
                    NormalizedDisposition::Deny => "deny",
                    NormalizedDisposition::Escalate => "escalate",
                    NormalizedDisposition::Boundary => "boundary",
                    NormalizedDisposition::Degraded => "degraded",
                }
                .into(),
                subject: format!("{resource_type}:{resource_id}"),
                reason: reason.clone(),
                evidence_refs,
                recorded_at: timestamp.clone(),
                decision_id: Some(decision_id),
                case_id: Some(case_id.into()),
                run_id: Some(run_id.into()),
                policy_bundle_hash: Some(self.bundle_hash.clone()),
                action_request_hash: Some(action_request_hash),
                identity_binding_hash: Some(identity_binding_hash),
            },
            decided_at: timestamp.clone(),
            candidate_verdicts: candidates
                .iter()
                .map(|candidate| GovernanceVerdictRecord {
                    engine: candidate.engine.clone(),
                    disposition: match candidate.disposition {
                        GovernanceDisposition::Allow => NormalizedDisposition::Allow,
                        GovernanceDisposition::Deny => NormalizedDisposition::Deny,
                        GovernanceDisposition::Escalate => NormalizedDisposition::Escalate,
                        GovernanceDisposition::Boundary => NormalizedDisposition::Boundary,
                        GovernanceDisposition::Degraded => NormalizedDisposition::Degraded,
                    },
                    subject: resource_id.clone(),
                    reason: candidate.reason.clone(),
                    evidence_refs: candidate.evidence_refs.clone(),
                    recorded_at: timestamp.clone(),
                    boundary_constraints: candidate
                        .boundaries
                        .iter()
                        .map(|(key, values)| (key.clone(), values.iter().cloned().collect()))
                        .collect(),
                    compensating_controls: candidate
                        .compensating_controls
                        .iter()
                        .cloned()
                        .collect(),
                })
                .collect(),
            winning_source: candidates
                .iter()
                .filter(|candidate| candidate.disposition == resolved.disposition)
                .map(|candidate| candidate.engine.clone())
                .min(),
            precedence_reason: Some("typed deterministic candidate resolution".into()),
            sandbox_class_granted: (verdict == Verdict::Allow).then(|| {
                matched_policy_rule
                    .and_then(|rule| rule.sandbox_class.clone())
                    .unwrap_or_else(|| "local".into())
            }),
            approval_request_id: (verdict == Verdict::Escalate)
                .then(|| format!("approval_{}", sequence)),
            opaque_constraint_id: existing_constraint
                .as_ref()
                .map(|constraint| constraint.constraint_id.clone())
                .or_else(|| {
                    (candidates
                        .iter()
                        .all(|candidate| candidate.disposition == GovernanceDisposition::Escalate))
                    .then(|| {
                        format!(
                            "opaque_{}",
                            &sha256_bytes(format!("{resource_type}:{resource_id}").as_bytes())
                                [..16]
                        )
                    })
                }),
            boundary_constraints: resolved
                .boundaries
                .iter()
                .map(|(key, values)| (key.clone(), values.iter().cloned().collect()))
                .collect(),
            compensating_controls: resolved.compensating_controls.into_iter().collect(),
        };
        if existing_constraint.is_none() {
            if let Some(constraint_id) = &decision.opaque_constraint_id {
                self.load_opaque_constraints([OpaqueConstraint {
                    constraint_id: constraint_id.clone(),
                    resource_type: resource_type.into(),
                    resource_id: resource_id.clone(),
                    created_by_decision_id: decision.decision_id.clone(),
                    reason: "all configured evaluators escalated".into(),
                    created_at: decision.decided_at.clone(),
                    expires_at: None,
                }])?;
            }
        }
        self.issued_decisions
            .lock()
            .map_err(|_| ForgeError::Internal("authority issuer state poisoned".into()))?
            .insert(issued_decision_key(&decision)?);
        Ok(decision)
    }
}
fn actor_type_for_role(role: &ActorRole) -> ActorType {
    match role {
        ActorRole::Agent | ActorRole::AutomatedAgent => ActorType::Agent,
        ActorRole::Service => ActorType::Service,
        ActorRole::System => ActorType::System,
        ActorRole::Operator
        | ActorRole::DataSteward
        | ActorRole::AgentGovernor
        | ActorRole::LifecycleCustodian
        | ActorRole::SecurityOfficer
        | ActorRole::RiskManager
        | ActorRole::Developer => ActorType::Human,
    }
}
fn redacted_action(action: &AuthorityAction) -> AuthorityAction {
    match action {
        AuthorityAction::WriteFile { path, content_hint } => AuthorityAction::WriteFile {
            path: path.clone(),
            content_hint: format!("sha256:{}", sha256_bytes(content_hint.as_bytes())),
        },
        other => other.clone(),
    }
}
fn untrusted_executable(action: &AuthorityAction) -> bool {
    let AuthorityAction::ExecuteCommand { argv, .. } = action else {
        return false;
    };
    let Some(executable) = argv.first() else {
        return true;
    };
    let Ok(expected) = std::env::current_exe().and_then(std::fs::canonicalize) else {
        return true;
    };
    std::fs::canonicalize(executable).map_or(true, |actual| actual != expected)
}
/// Canonical SOD match: requester role, operation kind, and optional
/// transition_kind selector. Never treats a configured selector as a wildcard
/// when action parameters are missing or malformed.
fn sod_rule_matches_action(rule: &SodRule, actor_role: &str, action: &AuthorityAction) -> bool {
    if rule.requester_role != actor_role {
        return false;
    }
    let Some(operation_kind) = rule.operation_kind.as_deref() else {
        return false;
    };
    let (action_kind, parameters) = match action {
        AuthorityAction::WriteFile { .. } => ("write_file", None),
        AuthorityAction::ExecuteCommand { .. } => ("execute_command", None),
        AuthorityAction::ExternalApi { .. } => ("external_api", None),
        AuthorityAction::GitCommit { .. } => ("git_commit", None),
        AuthorityAction::GithubPr { .. } => ("github_pr", None),
        AuthorityAction::Reserved {
            resource_type,
            parameters,
            ..
        } => (resource_type.as_str(), Some(parameters)),
        AuthorityAction::Unclassified { .. } => return false,
    };
    if operation_kind != action_kind {
        return false;
    }
    match rule.transition_kind.as_deref() {
        None => true,
        Some(expected) => match parameters {
            Some(params) if action_kind == "transition_artifact_stage" => {
                parameter_str(params, "transition_kind") == Some(expected)
            }
            _ => false,
        },
    }
}

fn matching_sod_rules<'a>(
    sod_rules: &'a [SodRule],
    actor_role: &str,
    action: &AuthorityAction,
) -> Vec<&'a SodRule> {
    sod_rules
        .iter()
        .filter(|sod| sod_rule_matches_action(sod, actor_role, action))
        .collect()
}

fn matches_rule(rule: &PolicyRule, actor: &Actor, action: &AuthorityAction) -> bool {
    if rule.actor_role != actor.role {
        return false;
    }
    match action {
        AuthorityAction::WriteFile { path, .. } => {
            let prefix = rule.path_prefix.as_deref().unwrap_or("");
            rule.operation_kind == "write_file" && (prefix.is_empty() || path.starts_with(prefix))
        }
        AuthorityAction::ExecuteCommand { .. } => rule.operation_kind == "execute_command",
        AuthorityAction::Reserved {
            resource_type,
            parameters,
            ..
        } => {
            rule.operation_kind == *resource_type
                && match resource_type.as_str() {
                    "transition_artifact_stage" => matches_transition_parameters(rule, parameters),
                    "attest_artifact_identity" => {
                        matches_attestation_parameters(rule, parameters, actor)
                    }
                    "approval_resolution" => {
                        matches_approval_resolution_parameters(parameters, actor)
                    }
                    _ => true,
                }
        }
        _ => false,
    }
}

fn matches_approval_resolution_parameters(parameters: &Value, actor: &Actor) -> bool {
    let role = serde_json::to_value(&actor.role)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned));
    let required_roles = parameters
        .get("required_approver_roles")
        .and_then(Value::as_array)
        .map(|roles| roles.iter().filter_map(Value::as_str).collect::<Vec<_>>());
    parameter_str(parameters, "approval_id").is_some()
        && parameter_str(parameters, "original_decision_id").is_some()
        && parameter_str(parameters, "requester_id")
            .is_some_and(|requester| requester != actor.actor_id)
        && matches!(
            parameter_str(parameters, "resolution"),
            Some("approved" | "rejected")
        )
        && required_roles.is_some_and(|roles| {
            roles.is_empty() || role.as_deref().is_some_and(|role| roles.contains(&role))
        })
}

fn parameter_str<'a>(parameters: &'a Value, name: &str) -> Option<&'a str> {
    parameters.get(name).and_then(Value::as_str)
}

fn matches_transition_parameters(rule: &PolicyRule, parameters: &Value) -> bool {
    let configured_evidence = rule
        .qualifying_value_evidence_kinds
        .as_deref()
        .unwrap_or_default();
    let requested_evidence = parameters
        .get("qualifying_value_evidence_kinds")
        .and_then(Value::as_array)
        .map(|values| values.iter().filter_map(Value::as_str).collect::<Vec<_>>());
    let licenses = parameters
        .get("licenses")
        .and_then(Value::as_array)
        .map(|values| values.iter().filter_map(Value::as_str).collect::<Vec<_>>());
    parameter_str(parameters, "transition_kind") == rule.transition_kind.as_deref()
        && parameter_str(parameters, "from_stage") == rule.from_stage.as_deref()
        && parameter_str(parameters, "to_stage") == rule.to_stage.as_deref()
        && parameter_str(parameters, "mode").is_some_and(|mode| {
            rule.modes
                .as_deref()
                .is_some_and(|modes| modes.iter().any(|allowed| allowed == mode))
        })
        && parameter_str(parameters, "gate_profile_ref") == rule.gate_profile_ref.as_deref()
        && parameter_str(parameters, "required_settlement_strength")
            == rule.required_settlement_strength.as_deref()
        && requested_evidence.as_ref().is_some_and(|requested| {
            requested.len() == configured_evidence.len()
                && requested
                    .iter()
                    .zip(configured_evidence)
                    .all(|(actual, expected)| actual == expected)
        })
        && parameters.get("requires_approval").and_then(Value::as_bool) == rule.requires_approval
        && licenses.as_ref().is_some_and(|licenses| {
            !licenses.is_empty()
                && (rule.license_allowlist.is_empty()
                    || licenses.iter().all(|license| {
                        rule.license_allowlist
                            .iter()
                            .any(|allowed| allowed == license)
                    }))
        })
}

fn matches_attestation_parameters(rule: &PolicyRule, parameters: &Value, actor: &Actor) -> bool {
    let actor_role = serde_json::to_value(&actor.role)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned));
    parameter_str(parameters, "ledger") == rule.attestation_ledger.as_deref()
        && parameter_str(parameters, "degraded_mode") == rule.degraded_mode.as_deref()
        && parameter_str(parameters, "requester_role") == actor_role.as_deref()
        && parameter_str(parameters, "requester_role").is_some_and(|role| {
            rule.requester_roles
                .as_deref()
                .is_some_and(|roles| roles.iter().any(|allowed| allowed == role))
        })
        && parameter_str(parameters, "approver_role").is_some_and(|role| {
            rule.approver_roles
                .as_deref()
                .is_some_and(|roles| roles.iter().any(|allowed| allowed == role))
        })
}

/// Pure three-axis permission check for ExecuteCommand (§7.6).
///
/// - **content axis**: `environment.provides.commands` declares what the env supplies.
/// - **permission axis**: `rule.environment` / `rule.argv0` gates which commands policy allows.
/// - **isolation axis**: `rule.sandbox_class` decides jail/local/microvm (orthogonal, not checked here).
///
/// The effective allow-list is the intersection of the environment's
/// `provides.commands` and the rule's `argv0` constraint, when both are present.
/// Call directly from tests to prove three-axis independence.
pub fn command_allowed(
    rule: &PolicyRule,
    action: &AuthorityAction,
    environment: Option<(&str, &[String])>,
) -> bool {
    let AuthorityAction::ExecuteCommand { argv, .. } = action else {
        return false;
    };
    if rule.operation_kind != "execute_command" {
        return false;
    }
    let argv0 = argv
        .first()
        .and_then(|v| Path::new(v).file_name())
        .and_then(|v| v.to_str());
    // Permission axis: environment-matched rules.
    if let Some(rule_env) = &rule.environment {
        // Rule requires the item to use this environment.
        let Some((item_env, provides)) = environment else {
            return false;
        };
        if rule_env.as_str() != item_env {
            return false;
        }
        // Content axis: argv0 must be in provides.commands.
        let in_provides = argv0.is_some_and(|a| provides.iter().any(|p| p == a));
        if !in_provides {
            return false;
        }
    }
    // Permission axis: argv0 constraint (intersection with environment if both).
    if let Some(expected) = &rule.argv0 {
        if argv0 != Some(expected.as_str()) {
            return false;
        }
    }
    true
}
fn hard_denied(action: &AuthorityAction, patterns: &[String]) -> bool {
    match action {
        AuthorityAction::WriteFile { path, .. } => {
            const BUILT_INS: &[&str] = &[
                "**/src/gen/**",
                "docs/specs/**/*.ast*.json",
                "docs/specs/**/*.ir.json",
                "docs/specs/**/*.manifest.json",
                "docs/specs/**/fixtures/semantic/*.semantic.fixture.yaml",
                ".git/**",
                ".env*",
                "**/*secret*",
            ];
            BUILT_INS.iter().any(|pattern| path_denied(path, pattern))
                || patterns.iter().any(|pattern| path_denied(path, pattern))
        }
        _ => false,
    }
}
fn malformed_action(action: &AuthorityAction) -> bool {
    match action {
        AuthorityAction::WriteFile { path, .. } => invalid_relative_path(path),
        AuthorityAction::ExecuteCommand { argv, cwd } => {
            argv.is_empty() || invalid_relative_path(cwd)
        }
        AuthorityAction::ExternalApi { host } => host.is_empty(),
        AuthorityAction::GitCommit { paths } => paths.is_empty(),
        AuthorityAction::Reserved {
            resource_type,
            resource_id,
            parameters,
        } => {
            const RESERVED: &[&str] = &[
                "file",
                "shell_cmd",
                "external_api",
                "git_commit",
                "github_pr",
                "prompt_risk",
                "policy_file",
                "evidence_record",
                "spec_projection",
                "artifact_transition",
                "transition_artifact_stage",
                "attest_artifact_identity",
                "artifact_lifecycle",
                "identity_binding",
                "install_extension",
                "adopt_extension",
                "disable_extension",
                "projection_execution",
                "approval_resolution",
                "recall_memory",
                "inspect_run",
                "validate_model",
                "delete_file",
                "sandbox_execution",
                "pr_merge",
                "spec_mutation",
                "generated_zone_mutation",
                "settlement_authority_mutation",
                "settlement_declaration",
                "human_task_completion",
                "case_reopen",
                "case_terminate",
                "discretionary_task_add",
                "policy_mutation",
                "evidence_mutation",
                "identity_minting",
                "secret_access",
                "deployment",
                "rollback",
                "artifact_transition",
                "attestation",
                "review_artifact_rights",
            ];
            !RESERVED.contains(&resource_type.as_str())
                || resource_id.is_empty()
                || !parameters.is_object()
        }
        _ => false,
    }
}

fn invalid_relative_path(path: &str) -> bool {
    let value = Path::new(path);
    value.is_absolute()
        || path.is_empty()
        || !path
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._/-".contains(c))
        || value.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
}
fn protected_git_commit(action: &AuthorityAction, configured: &[String]) -> bool {
    const BUILT_INS: &[&str] = &["config/governance/**", ".github/workflows/**"];
    matches!(action, AuthorityAction::GitCommit { paths } if paths.iter().any(|path| BUILT_INS.iter().any(|pattern| path_denied(path, pattern)) || configured.iter().any(|pattern| path_denied(path, pattern))))
}
fn path_denied(path: &str, pattern: &str) -> bool {
    let path = path.to_ascii_lowercase();
    let pattern = pattern.to_ascii_lowercase();
    path == ".git"
        || wildcard_match(pattern.as_bytes(), path.as_bytes())
        || pattern
            .strip_prefix("**/")
            .is_some_and(|suffix| wildcard_match(suffix.as_bytes(), path.as_bytes()))
}
fn wildcard_match(pattern: &[u8], value: &[u8]) -> bool {
    let mut previous = vec![false; value.len() + 1];
    previous[0] = true;
    for token in pattern {
        let mut current = vec![false; value.len() + 1];
        if *token == b'*' {
            current[0] = previous[0];
            for index in 1..=value.len() {
                current[index] = previous[index] || current[index - 1];
            }
        } else {
            for index in 1..=value.len() {
                current[index] = previous[index - 1] && *token == value[index - 1];
            }
        }
        previous = current;
    }
    previous[value.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_forge_ledger::LedgerStream;

    fn engine() -> PolicyAuthorityEngine {
        let bundle: AuthorityPolicyBundle = serde_yaml::from_str(
            "version: \"0.1\"\nrules:\n  - name: allow-write\n    verdict: allow\n    actor_role: operator\n    operation_kind: write_file\n    path_prefix: \"\"\n",
        ).unwrap();
        PolicyAuthorityEngine::new(bundle).unwrap()
    }
    pub(super) fn complete_v02(mut bundle: AuthorityPolicyBundle) -> AuthorityPolicyBundle {
        const SURFACES: &[&str] = &[
            "authority_hooks",
            "identity_map",
            "domain_model",
            "file_access",
            "api_allowlist",
            "git_commit",
            "pr_merge",
            "prompt_risk",
            "memory_recall",
            "spec_pipeline",
            "artifact_transition",
            "attestation",
            "approval",
            "settlement_authority",
            "capability_promotion",
            "deployment",
            "secret_access",
            "policy_mutation",
            "evidence_mutation",
        ];
        const ROLES: &[&str] = &[
            "R-DS", "R-AG", "R-LC", "R-SO", "R-RM", "R-DEV", "R-AA", "operator",
        ];
        // (name, operation_kind, optional transition_kind)
        const SOD: &[(&str, &str, Option<&str>)] = &[
            ("production_proposer_approver", "pr_merge", None),
            (
                "semantic_debt_requester_acceptor",
                "settlement_authority_mutation",
                None,
            ),
            ("break_glass_requester_approver", "policy_mutation", None),
            ("key_generator_approver", "identity_minting", None),
            (
                "capitalization_requester_approver",
                "transition_artifact_stage",
                Some("capitalize"),
            ),
        ];
        bundle.bundle_id = Some("bundle_test".into());
        bundle.policy_bundle_hash = None;
        bundle.policy_bundle_version = Some("1".into());
        bundle.loaded_at = Some("2026-07-13T00:00:00Z".into());
        bundle.sources = SURFACES
            .iter()
            .map(|surface| PolicySource {
                surface: (*surface).into(),
                path: format!("policies/{surface}.yaml"),
                sha256: "sha256:test".into(),
            })
            .collect();
        bundle.roles = ROLES
            .iter()
            .map(|role| ((*role).into(), vec!["fixture".into()]))
            .collect();
        bundle.permissions = ROLES
            .iter()
            .map(|role| PermissionGrant {
                role: (*role).into(),
                operation_kind: "*".into(),
            })
            .collect();
        // requester_role is operator so unit tests that evaluate as operator
        // exercise SOD matching; schema only requires non-empty requester.
        bundle.sod_rules = SOD
            .iter()
            .map(|(name, operation, transition)| SodRule {
                name: (*name).into(),
                requester_role: "operator".into(),
                approver_role: "R-SO".into(),
                allow_same_principal: false,
                operation_kind: Some((*operation).into()),
                transition_kind: transition.map(str::to_owned),
            })
            .collect();
        bundle.policy_bundle_hash = Some(hash_canonical(&bundle).unwrap());
        bundle
    }
    fn actor() -> Actor {
        Actor {
            actor_id: "operator_local".into(),
            role: ActorRole::Operator,
        }
    }
    fn binding() -> IdentityBinding {
        IdentityBinding {
            identity_id: Some("identity:operator_local".into()),
            principal: "operator_local".into(),
            roles: vec![ActorRole::Operator],
            actor_type: ActorType::Human,
            binding_resolution: BindingResolution::LocalDefault,
            identity_binding_source: "local-slice-default".into(),
            source: Some("local-slice-default".into()),
            sponsor: None,
            issued_at: None,
            expires_at: None,
            identity_binding_hash: Some(sha256_bytes(b"operator_local:Operator")),
        }
    }
    fn evaluate(action: &AuthorityAction) -> AuthorityDecision {
        let actor = actor();
        engine()
            .evaluate_at(
                AuthorityEvaluation {
                    actor: &actor,
                    binding: binding(),
                    run_id: "run_20260710T120000Z_abcdef",
                    case_id: "case_test",
                    plan_item_id: "item_01",
                    sequence: 1,
                    action,
                    workspace_root: Path::new("/tmp/workspace"),
                    evidence_refs: vec![],
                    artifacts_root: None,
                    timeout_secs: None,
                    env_keys: Default::default(),
                    domainforge_candidate: None,
                    environment: None,
                },
                "act_abcdef".into(),
                "2026-07-10T12:00:00Z".into(),
            )
            .unwrap()
    }
    fn evaluate_with(
        engine: &PolicyAuthorityEngine,
        action: &AuthorityAction,
    ) -> AuthorityDecision {
        let actor = actor();
        engine
            .evaluate_at(
                AuthorityEvaluation {
                    actor: &actor,
                    binding: binding(),
                    run_id: "run_20260710T120000Z_abcdef",
                    case_id: "case_test",
                    plan_item_id: "item_01",
                    sequence: 1,
                    action,
                    workspace_root: Path::new("/tmp/workspace"),
                    evidence_refs: vec![],
                    artifacts_root: None,
                    timeout_secs: None,
                    env_keys: Default::default(),
                    domainforge_candidate: None,
                    environment: None,
                },
                "act_abcdef".into(),
                Utc::now().to_rfc3339(),
            )
            .unwrap()
    }

    fn commit(decision: &AuthorityDecision) -> (tempfile::TempDir, CommittedRecordRef) {
        let root = tempfile::tempdir().unwrap();
        let stream = LedgerStream::open(root.path(), "authority-test", "test-writer").unwrap();
        let committed = stream
            .commit_typed("authority_decision", vec![], decision, vec![])
            .unwrap();
        (root, committed)
    }

    #[test]
    fn same_canonical_request_has_stable_decision_and_hashes() {
        let action = AuthorityAction::WriteFile {
            path: "model.sea".into(),
            content_hint: "fixed".into(),
        };
        let first = evaluate(&action);
        let second = evaluate(&action);
        assert_eq!(first.outcome, second.outcome);
        assert_eq!(first.reason_codes, second.reason_codes);
        assert_eq!(first.policy_refs, second.policy_refs);
        assert_eq!(first.required_next_steps, second.required_next_steps);
        assert_eq!(first.determinism, second.determinism);
    }

    #[test]
    fn unclassified_and_reserved_surfaces_fail_closed() {
        let unknown = evaluate(&AuthorityAction::Unclassified {
            raw_kind: "mystery".into(),
            parameters: json!({}),
        });
        assert_eq!(unknown.verdict, Verdict::Deny);
        assert_eq!(unknown.reason_codes, ["unclassified"]);
        let api = evaluate(&AuthorityAction::ExternalApi {
            host: "unknown.example".into(),
        });
        assert_eq!(api.verdict, Verdict::Deny);
        let git = evaluate(&AuthorityAction::GitCommit {
            paths: vec!["config/governance/policy.yaml".into()],
        });
        assert_eq!(git.verdict, Verdict::Deny);
        let pr = evaluate(&AuthorityAction::GithubPr {
            has_required_evidence: false,
        });
        assert_eq!(pr.verdict, Verdict::Escalate);
        assert!(pr
            .reason_codes
            .contains(&"unsupported_action_surface".into()));
        let unknown_reserved = evaluate(&AuthorityAction::Reserved {
            resource_type: "mystery".into(),
            resource_id: "x".into(),
            parameters: json!({}),
        });
        assert_eq!(unknown_reserved.verdict, Verdict::Deny);
        assert_eq!(unknown_reserved.reason_codes, ["unclassified"]);
    }

    #[test]
    fn reserved_transition_grant_rejects_a_substituted_action() {
        let bundle: AuthorityPolicyBundle = serde_yaml::from_str(
            "version: \"0.1\"\nrules:\n  - name: transition\n    verdict: allow\n    actor_role: operator\n    operation_kind: transition_artifact_stage\n    transition_kind: synthesize\n    from_stage: cognitive\n    to_stage: intellectual\n    modes: [promote]\n    gate_profile_ref: synthesize@1\n    required_settlement_strength: local\n    qualifying_value_evidence_kinds: []\n    requires_approval: false\n    license_allowlist: [internal]\n",
        )
        .unwrap();
        let engine = PolicyAuthorityEngine::new(bundle).unwrap();
        let action = AuthorityAction::Reserved {
            resource_type: "transition_artifact_stage".into(),
            resource_id: "art_model".into(),
            parameters: json!({"transition_kind":"synthesize","from_stage":"cognitive","to_stage":"intellectual","mode":"promote","gate_profile_ref":"synthesize@1","required_settlement_strength":"local","qualifying_value_evidence_kinds":[],"requires_approval":false,"licenses":["internal"]}),
        };
        let decision = evaluate_with(&engine, &action);
        assert_eq!(decision.verdict, Verdict::Allow);
        let (_root, committed) = commit(&decision);
        let substituted = AuthorityAction::Reserved {
            resource_type: "transition_artifact_stage".into(),
            resource_id: "art_other".into(),
            parameters: json!({"transition_kind":"synthesize","from_stage":"cognitive","to_stage":"intellectual","mode":"promote","gate_profile_ref":"synthesize@1","required_settlement_strength":"local","qualifying_value_evidence_kinds":[],"requires_approval":false,"licenses":["internal"]}),
        };
        assert!(engine
            .grant(&decision, &committed, &substituted, None)
            .is_err());
    }

    fn approval_transition_fixture() -> (
        PolicyAuthorityEngine,
        AuthorityAction,
        AuthorityDecision,
        ApprovalRequest,
        SettlementCriteriaRecord,
        tempfile::TempDir,
        CommittedRecordRef,
        CommittedRecordRef,
        CommittedRecordRef,
    ) {
        let bundle: AuthorityPolicyBundle = serde_yaml::from_str(
            "version: \"0.1\"\nrules:\n  - name: capitalize\n    verdict: allow\n    actor_role: operator\n    operation_kind: transition_artifact_stage\n    transition_kind: capitalize\n    from_stage: product\n    to_stage: capital\n    modes: [promote]\n    gate_profile_ref: capital@1\n    required_settlement_strength: strong\n    qualifying_value_evidence_kinds: [adoption]\n    requires_approval: true\n    license_allowlist: [internal]\n",
        )
        .unwrap();
        let engine = PolicyAuthorityEngine::new(bundle).unwrap();
        let action = AuthorityAction::Reserved {
            resource_type: "transition_artifact_stage".into(),
            resource_id: "art_model".into(),
            parameters: json!({"transition_kind":"capitalize","from_stage":"product","to_stage":"capital","mode":"promote","gate_profile_ref":"capital@1","required_settlement_strength":"strong","qualifying_value_evidence_kinds":["adoption"],"requires_approval":true,"licenses":["internal"]}),
        };
        let requester = actor();
        let decision_time = Utc::now() - chrono::Duration::minutes(10);
        let decision = engine
            .evaluate_at(
                AuthorityEvaluation {
                    actor: &requester,
                    binding: binding(),
                    run_id: "run_20260710T120000Z_abcdef",
                    case_id: "case_test",
                    plan_item_id: "item_01",
                    sequence: 1,
                    action: &action,
                    workspace_root: Path::new("/tmp/workspace"),
                    evidence_refs: vec![],
                    artifacts_root: None,
                    timeout_secs: None,
                    env_keys: Default::default(),
                    domainforge_candidate: None,
                    environment: None,
                },
                "act_abcdef".into(),
                decision_time.to_rfc3339(),
            )
            .unwrap();
        assert_eq!(decision.verdict, Verdict::Escalate);
        let criteria = SettlementCriteriaRecord {
            version: RECORD_VERSION.into(),
            criteria_id: "crit_capital".into(),
            criteria: SettlementCriteria {
                require_exit_zero: true,
                required_artifacts: vec![],
                stdout_must_contain: None,
                require_approval: true,
                evaluator: None,
                records: None,
                per_record_evaluator: None,
                min_pass_ratio: None,
            },
            origin_refs: vec![OriginRef {
                kind: OriginRefKind::Intent,
                reference: "int_capital".into(),
                sha256: "sha256:intent".into(),
                role: OriginRole::AcceptanceSource,
                evidence_refs: vec![],
                domain_model_ref: None,
            }],
            derivation: CriteriaDerivation {
                method: DerivationMethod::DeterministicPlanner,
                actor_ref: "operator_local".into(),
                producer_ref: None,
                rationale: "capitalization fixture".into(),
            },
            declared_at: Utc::now().to_rfc3339(),
            criteria_sha256: String::new(),
            criteria_record_hash: String::new(),
        };
        let mut criteria = criteria;
        criteria.criteria_sha256 =
            sea_forge_ledger::types::hash_canonical(&criteria.criteria).unwrap();
        let mut unhashed = criteria.clone();
        unhashed.criteria_record_hash.clear();
        criteria.criteria_record_hash = sea_forge_ledger::types::hash_canonical(&unhashed).unwrap();
        let approval = ApprovalRequest {
            version: RECORD_VERSION.into(),
            approval_id: "apr_capital".into(),
            run_id: decision.run_id.clone(),
            case_id: decision.audit_record.case_id.clone().unwrap(),
            decision_id: decision.decision_id.clone(),
            plan_item_id: decision.plan_item_id.clone(),
            criteria_ref: Some(criteria.criteria_id.clone()),
            criteria_sha256: Some(criteria.criteria_sha256.clone()),
            criteria_record_hash: Some(criteria.criteria_record_hash.clone()),
            job_contract_ref: None,
            requested_at: decision_time.to_rfc3339(),
            expires_at: (Utc::now() + chrono::Duration::hours(1)).to_rfc3339(),
            status: ApprovalStatus::Approved,
            resolved_by: Some("security_officer".into()),
            resolved_at: Some(Utc::now().to_rfc3339()),
            note: None,
        };
        let root = tempfile::tempdir().unwrap();
        let stream = LedgerStream::open(root.path(), "case-case_test", "test-writer").unwrap();
        let decision_ref = stream
            .commit_typed("authority_decision", vec![], &decision, vec![])
            .unwrap();
        let criteria_ref = stream
            .commit_typed("settlement_criteria", vec![], &criteria, vec![])
            .unwrap();
        let approval_ref = stream
            .commit_typed("approval_resolution", vec![], &approval, vec![])
            .unwrap();
        (
            engine,
            action,
            decision,
            approval,
            criteria,
            root,
            decision_ref,
            approval_ref,
            criteria_ref,
        )
    }

    #[test]
    fn caller_supplied_approval_refs_do_not_change_escalation() {
        let (engine, mut action, ..) = approval_transition_fixture();
        if let AuthorityAction::Reserved { parameters, .. } = &mut action {
            parameters["approval_ref"] = json!("apr_capital");
            parameters["approver_id"] = json!("security_officer");
        }
        assert_eq!(evaluate_with(&engine, &action).verdict, Verdict::Escalate);
    }

    #[test]
    fn exact_ledgered_approval_yields_one_exact_grant() {
        let (
            engine,
            action,
            decision,
            approval,
            criteria,
            _root,
            decision_ref,
            approval_ref,
            criteria_ref,
        ) = approval_transition_fixture();
        let resumed_engine = PolicyAuthorityEngine::new(engine.bundle.clone()).unwrap();
        let stream = LedgerStream::open(_root.path(), "case-case_test", "test-writer").unwrap();
        let grant = resumed_engine
            .grant_after_approval(
                &decision,
                &decision_ref,
                &action,
                &approval,
                &approval_ref,
                &criteria,
                &criteria_ref,
                None,
                &stream,
            )
            .unwrap();
        grant
            .authorize(
                &action,
                &decision.run_id,
                &decision.plan_item_id,
                Path::new("/tmp/workspace"),
            )
            .unwrap();
        assert!(resumed_engine
            .grant_after_approval(
                &decision,
                &decision_ref,
                &action,
                &approval,
                &approval_ref,
                &criteria,
                &criteria_ref,
                None,
                &stream,
            )
            .is_err());
    }

    #[test]
    fn wrong_self_expired_rejected_or_tampered_approval_is_denied() {
        for variant in ["wrong", "self", "expired", "rejected", "tampered"] {
            let (
                engine,
                action,
                decision,
                mut approval,
                criteria,
                _root,
                decision_ref,
                approval_ref,
                criteria_ref,
            ) = approval_transition_fixture();
            match variant {
                "wrong" => approval.plan_item_id = "other_item".into(),
                "self" => approval.resolved_by = Some("operator_local".into()),
                "expired" => approval.expires_at = "2020-01-01T00:00:00Z".into(),
                "rejected" => approval.status = ApprovalStatus::Rejected,
                "tampered" => approval.criteria_record_hash = Some("sha256:tampered".into()),
                _ => unreachable!(),
            }
            let stream = LedgerStream::open(_root.path(), "case-case_test", "test-writer").unwrap();
            assert!(
                engine
                    .grant_after_approval(
                        &decision,
                        &decision_ref,
                        &action,
                        &approval,
                        &approval_ref,
                        &criteria,
                        &criteria_ref,
                        None,
                        &stream,
                    )
                    .is_err(),
                "variant {variant}"
            );
        }
    }

    #[test]
    fn approved_before_expiry_remains_valid_after_wall_clock_ttl() {
        let (
            engine,
            action,
            decision,
            mut approval,
            criteria,
            root,
            decision_ref,
            _approval_ref,
            criteria_ref,
        ) = approval_transition_fixture();
        // Resolved Approved before expires_at, but wall clock is now past expiry.
        let expires = Utc::now() - chrono::Duration::minutes(10);
        let resolved = Utc::now() - chrono::Duration::minutes(20);
        approval.status = ApprovalStatus::Approved;
        approval.resolved_at = Some(resolved.to_rfc3339());
        approval.expires_at = expires.to_rfc3339();
        let stream = LedgerStream::open(root.path(), "case-case_test", "test-writer").unwrap();
        let approval_ref = stream
            .commit_typed_once(
                "approval_resolution",
                "apr_past_ttl",
                vec!["apr_past_ttl".into()],
                &approval,
                vec![],
            )
            .unwrap();
        let resumed_engine = PolicyAuthorityEngine::new(engine.bundle.clone()).unwrap();
        let grant = resumed_engine
            .grant_after_approval(
                &decision,
                &decision_ref,
                &action,
                &approval,
                &approval_ref,
                &criteria,
                &criteria_ref,
                None,
                &stream,
            )
            .unwrap();
        grant
            .authorize(
                &action,
                &decision.run_id,
                &decision.plan_item_id,
                Path::new("/tmp/workspace"),
            )
            .unwrap();
    }

    #[test]
    fn approved_resolved_after_expiry_is_denied() {
        let (
            engine,
            action,
            decision,
            mut approval,
            criteria,
            root,
            decision_ref,
            _approval_ref,
            criteria_ref,
        ) = approval_transition_fixture();
        let past = Utc::now() - chrono::Duration::hours(2);
        let even_more_past = Utc::now() - chrono::Duration::days(1);
        approval.status = ApprovalStatus::Approved;
        approval.resolved_at = Some(past.to_rfc3339());
        approval.expires_at = even_more_past.to_rfc3339();
        let stream = LedgerStream::open(root.path(), "case-case_test", "test-writer").unwrap();
        let approval_ref = stream
            .commit_typed_once(
                "approval_resolution",
                "apr_late",
                vec!["apr_late".into()],
                &approval,
                vec![],
            )
            .unwrap();
        let resumed_engine = PolicyAuthorityEngine::new(engine.bundle.clone()).unwrap();
        assert!(resumed_engine
            .grant_after_approval(
                &decision,
                &decision_ref,
                &action,
                &approval,
                &approval_ref,
                &criteria,
                &criteria_ref,
                None,
                &stream,
            )
            .is_err());
    }

    #[test]
    fn m8_transition_parameter_mismatches_deny_before_grant() {
        let policy = "version: \"0.1\"\nrules:\n  - name: transition\n    verdict: allow\n    actor_role: operator\n    operation_kind: transition_artifact_stage\n    transition_kind: synthesize\n    from_stage: cognitive\n    to_stage: intellectual\n    modes: [promote]\n    gate_profile_ref: synthesize@1\n    required_settlement_strength: local\n    qualifying_value_evidence_kinds: [adoption]\n    requires_approval: false\n    license_allowlist: [internal]\n";
        let valid = json!({
            "transition_kind":"synthesize",
            "from_stage":"cognitive",
            "to_stage":"intellectual",
            "mode":"promote",
            "gate_profile_ref":"synthesize@1",
            "required_settlement_strength":"local",
            "qualifying_value_evidence_kinds":["adoption"],
            "requires_approval":false,
            "licenses":["internal"]
        });
        for (field, replacement) in [
            ("transition_kind", json!("productize")),
            ("from_stage", json!("intellectual")),
            ("to_stage", json!("product")),
            ("mode", json!("derive")),
            ("gate_profile_ref", json!("other@1")),
            ("required_settlement_strength", json!("strong")),
            ("qualifying_value_evidence_kinds", json!(["quality"])),
            ("requires_approval", json!(true)),
            ("licenses", json!(["proprietary"])),
        ] {
            let mut parameters = valid.clone();
            parameters[field] = replacement;
            let engine = PolicyAuthorityEngine::new(serde_yaml::from_str(policy).unwrap()).unwrap();
            let action = AuthorityAction::Reserved {
                resource_type: "transition_artifact_stage".into(),
                resource_id: "art_model".into(),
                parameters,
            };
            let decision = evaluate_with(&engine, &action);
            assert_eq!(decision.verdict, Verdict::Deny, "field {field}");
            let (_root, committed) = commit(&decision);
            assert!(
                engine.grant(&decision, &committed, &action, None).is_err(),
                "field {field}"
            );
        }
    }

    #[test]
    fn m8_attestation_parameter_mismatches_deny_before_grant() {
        let policy = "version: \"0.1\"\nrules:\n  - name: attest\n    verdict: allow\n    actor_role: operator\n    operation_kind: attest_artifact_identity\n    ledger: ifl\n    requester_roles: [operator]\n    approver_roles: [R-SO]\n    degraded_mode: forbidden\n";
        let valid = json!({
            "ledger":"ifl",
            "requester_role":"operator",
            "approver_role":"R-SO",
            "degraded_mode":"forbidden"
        });
        for (field, replacement) in [
            ("ledger", json!("other")),
            ("requester_role", json!("R-AA")),
            ("approver_role", json!("R-DEV")),
            ("degraded_mode", json!("pre_mint_only")),
        ] {
            let mut parameters = valid.clone();
            parameters[field] = replacement;
            let engine = PolicyAuthorityEngine::new(serde_yaml::from_str(policy).unwrap()).unwrap();
            let action = AuthorityAction::Reserved {
                resource_type: "attest_artifact_identity".into(),
                resource_id: "art_model".into(),
                parameters,
            };
            let decision = evaluate_with(&engine, &action);
            assert_eq!(decision.verdict, Verdict::Deny, "field {field}");
            let (_root, committed) = commit(&decision);
            assert!(engine.grant(&decision, &committed, &action, None).is_err());
        }
    }

    #[test]
    fn allowlisted_api_host_reaches_unsupported_surface_escalation() {
        let bundle: AuthorityPolicyBundle = serde_yaml::from_str("version: \"0.1\"\npolicy_surfaces:\n  external_api:\n    mode: deny-by-default\n    allow_hosts: [known.example]\nrules: []\n").unwrap();
        let engine = PolicyAuthorityEngine::new(bundle).unwrap();
        let actor = actor();
        let action = AuthorityAction::ExternalApi {
            host: "known.example".into(),
        };
        let decision = engine
            .evaluate_at(
                AuthorityEvaluation {
                    actor: &actor,
                    binding: binding(),
                    run_id: "run_20260710T120000Z_abcdef",
                    case_id: "case_test",
                    plan_item_id: "item_01",
                    sequence: 1,
                    action: &action,
                    workspace_root: Path::new("/tmp/workspace"),
                    evidence_refs: vec![],
                    artifacts_root: None,
                    timeout_secs: None,
                    env_keys: Default::default(),
                    domainforge_candidate: None,
                    environment: None,
                },
                "act_abcdef".into(),
                "2026-07-10T12:00:00Z".into(),
            )
            .unwrap();
        assert_eq!(decision.verdict, Verdict::Escalate);
        assert_eq!(decision.reason_codes, ["unsupported_action_surface"]);
    }

    #[test]
    fn engine_constructor_rejects_unvalidated_allow_any_command_bundle() {
        let bundle: AuthorityPolicyBundle = serde_yaml::from_str("version: \"0.1\"\nrules:\n  - name: unsafe\n    verdict: allow\n    actor_role: operator\n    operation_kind: execute_command\n").unwrap();
        assert!(PolicyAuthorityEngine::new(bundle).is_err());
    }

    #[test]
    fn configured_engine_must_fail_closed_and_unavailability_blocks() {
        let fail_open: AuthorityPolicyBundle = serde_yaml::from_str(
            "version: \"0.2\"\npolicy_engines:\n  - engine: opa\n    fail_mode: pass\n    required: true\nrules: []\n",
        )
        .unwrap();
        assert!(PolicyAuthorityEngine::new(complete_v02(fail_open)).is_err());

        let bundle: AuthorityPolicyBundle = serde_yaml::from_str(
            "version: \"0.2\"\npolicy_engines:\n  - engine: opa\n    fail_mode: closed\n    required: true\nrules:\n  - name: allow-write\n    verdict: allow\n    actor_role: operator\n    operation_kind: write_file\n    path_prefix: \"\"\n",
        )
        .unwrap();
        let engine = PolicyAuthorityEngine::new(complete_v02(bundle)).unwrap();
        let action = AuthorityAction::WriteFile {
            path: "model.sea".into(),
            content_hint: "fixed".into(),
        };
        let decision = evaluate_with(&engine, &action);
        assert_eq!(decision.verdict, Verdict::Deny);
        assert_eq!(decision.candidate_verdicts.len(), 2);
    }

    #[test]
    fn v02_identity_map_resolves_and_unknown_identity_escalates() {
        let bundle: AuthorityPolicyBundle = complete_v02(serde_yaml::from_str(
            "version: \"0.2\"\nidentity_bindings:\n  - principal: operator_local\n    actor_type: human\n    role: operator\nrules:\n  - name: allow-write\n    verdict: allow\n    actor_role: operator\n    operation_kind: write_file\n    path_prefix: \"\"\n",
        )
        .unwrap());
        let resolved = bundle.resolve_identity("operator_local", ActorRole::Operator);
        assert_eq!(resolved.binding_resolution, BindingResolution::Exact);
        let unknown = bundle.resolve_identity("unknown", ActorRole::Operator);
        assert_eq!(unknown.binding_resolution, BindingResolution::Unresolved);
    }

    #[test]
    fn domainforge_candidate_is_composed_with_local_authority() {
        let bundle: AuthorityPolicyBundle = complete_v02(serde_yaml::from_str(
            "version: \"0.2\"\nidentity_bindings:\n  - principal: operator_local\n    actor_type: human\n    role: operator\npolicy_engines:\n  - engine: domainforge\n    fail_mode: closed\n    required: true\nrules:\n  - name: allow-write\n    verdict: allow\n    actor_role: operator\n    operation_kind: write_file\n    path_prefix: \"\"\n",
        )
        .unwrap());
        let source = include_str!("../../sea-forge-domainforge/tests/fixtures/demo.sea");
        let model = sea_forge_domainforge::load_validate(&sea_forge_domainforge::SeaSourceSet {
            entry_uri: "demo.sea".into(),
            files: vec![sea_forge_domainforge::SourceFile {
                uri: "demo.sea".into(),
                sha256: sha256_bytes(source.as_bytes()),
                content: source.into(),
            }],
        })
        .unwrap();
        for (path, expected) in [
            ("Sample.sea", Verdict::Allow),
            ("Unknown.sea", Verdict::Deny),
        ] {
            let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
            let actor = actor();
            let action = AuthorityAction::WriteFile {
                path: path.into(),
                content_hint: "fixed".into(),
            };
            let candidate =
                DomainForgeCandidate::evaluate(&model, &action, vec!["evi_domainforge".into()])
                    .unwrap();
            let decision = engine
                .evaluate(AuthorityEvaluation {
                    actor: &actor,
                    binding: bundle.resolve_identity("operator_local", ActorRole::Operator),
                    run_id: "run_domainforge",
                    case_id: "case_test",
                    plan_item_id: "item_domainforge",
                    sequence: 1,
                    action: &action,
                    workspace_root: Path::new("/tmp/workspace"),
                    evidence_refs: vec![],
                    artifacts_root: None,
                    timeout_secs: None,
                    env_keys: Default::default(),
                    domainforge_candidate: Some(&candidate),
                    environment: None,
                })
                .unwrap();
            assert_eq!(decision.verdict, expected);
            assert_eq!(decision.candidate_verdicts.len(), 2);
        }
    }

    #[test]
    fn automated_agent_requires_sponsor() {
        for (sponsor_yaml, expected) in [
            ("", Verdict::Escalate),
            ("    sponsor: human_owner\n", Verdict::Allow),
        ] {
            let bundle: AuthorityPolicyBundle = serde_yaml::from_str(&format!(
                "version: \"0.1\"\nidentity_bindings:\n  - principal: agent_one\n    actor_type: agent\n    role: agent\n{sponsor_yaml}rules:\n  - name: allow-write\n    verdict: allow\n    actor_role: agent\n    operation_kind: write_file\n    path_prefix: \"\"\n"
            ))
            .unwrap();
            let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
            let actor = Actor {
                actor_id: "agent_one".into(),
                role: ActorRole::Agent,
            };
            let action = AuthorityAction::WriteFile {
                path: "model.sea".into(),
                content_hint: "fixed".into(),
            };
            let decision = engine
                .evaluate(AuthorityEvaluation {
                    actor: &actor,
                    binding: bundle.resolve_identity("agent_one", ActorRole::Agent),
                    run_id: "run_agent",
                    case_id: "case_test",
                    plan_item_id: "item_agent",
                    sequence: 1,
                    action: &action,
                    workspace_root: Path::new("/tmp/workspace"),
                    evidence_refs: vec![],
                    artifacts_root: None,
                    timeout_secs: None,
                    env_keys: Default::default(),
                    domainforge_candidate: None,
                    environment: None,
                })
                .unwrap();
            assert_eq!(decision.verdict, expected);
        }
    }

    #[test]
    fn configured_service_identity_can_authorize() {
        let bundle: AuthorityPolicyBundle = serde_yaml::from_str(
            "version: \"0.1\"\nidentity_bindings:\n  - principal: service_one\n    actor_type: service\n    role: service\nrules:\n  - name: allow-service-write\n    verdict: allow\n    actor_role: service\n    operation_kind: write_file\n    path_prefix: \"\"\n",
        )
        .unwrap();
        let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
        let actor = Actor {
            actor_id: "service_one".into(),
            role: ActorRole::Service,
        };
        let action = AuthorityAction::WriteFile {
            path: "model.sea".into(),
            content_hint: "fixed".into(),
        };
        let decision = engine
            .evaluate(AuthorityEvaluation {
                actor: &actor,
                binding: bundle.resolve_identity("service_one", ActorRole::Service),
                run_id: "run_service",
                case_id: "case_test",
                plan_item_id: "item_service",
                sequence: 1,
                action: &action,
                workspace_root: Path::new("/tmp/workspace"),
                evidence_refs: vec![],
                artifacts_root: None,
                timeout_secs: None,
                env_keys: Default::default(),
                domainforge_candidate: None,
                environment: None,
            })
            .unwrap();
        assert_eq!(decision.verdict, Verdict::Allow);
    }

    #[test]
    fn opaque_constraint_preempts_repeated_ambiguous_action() {
        let bundle: AuthorityPolicyBundle = serde_yaml::from_str(
            "version: \"0.1\"\nrules:\n  - name: review-write\n    verdict: escalate\n    actor_role: operator\n    operation_kind: write_file\n    path_prefix: \"\"\n",
        )
        .unwrap();
        let engine = PolicyAuthorityEngine::new(bundle).unwrap();
        let action = AuthorityAction::WriteFile {
            path: "ambiguous.sea".into(),
            content_hint: "fixed".into(),
        };
        let first = evaluate_with(&engine, &action);
        let second = evaluate_with(&engine, &action);
        assert_eq!(first.opaque_constraint_id, second.opaque_constraint_id);
        assert_eq!(second.reason_codes, vec!["opaque_constraint_active"]);
    }

    #[test]
    fn approval_required_rule_escalates_without_resolution() {
        let bundle: AuthorityPolicyBundle = serde_yaml::from_str(
            "version: \"0.1\"\nrules:\n  - name: approved-write\n    verdict: allow\n    requires_approval: true\n    actor_role: operator\n    operation_kind: write_file\n    path_prefix: \"\"\n",
        )
        .unwrap();
        let decision = evaluate_with(
            &PolicyAuthorityEngine::new(bundle).unwrap(),
            &AuthorityAction::WriteFile {
                path: "reviewed.sea".into(),
                content_hint: "fixed".into(),
            },
        );
        assert_eq!(decision.verdict, Verdict::Escalate);
        assert!(decision.approval_request_id.is_some());
    }

    #[test]
    fn configured_sod_approver_role_propagates_to_decision() {
        let base: AuthorityPolicyBundle = serde_yaml::from_str(
            "version: \"0.2\"\nidentity_bindings:\n  - principal: operator_local\n    actor_type: human\n    role: operator\nrules:\n  - name: mutate-policy\n    verdict: allow\n    actor_role: operator\n    operation_kind: policy_mutation\n",
        )
        .unwrap();
        let action = AuthorityAction::Reserved {
            resource_type: "policy_mutation".into(),
            resource_id: "active-policy".into(),
            parameters: json!({}),
        };
        let bundle = complete_v02(base.clone());
        let binding = bundle.resolve_identity("operator_local", ActorRole::Operator);
        let engine = PolicyAuthorityEngine::new(bundle).unwrap();
        let actor = actor();
        let decision = engine
            .evaluate(AuthorityEvaluation {
                actor: &actor,
                binding,
                run_id: "run_sod",
                case_id: "case_test",
                plan_item_id: "item_sod",
                sequence: 1,
                action: &action,
                workspace_root: Path::new("/tmp/workspace"),
                evidence_refs: vec![],
                artifacts_root: None,
                timeout_secs: None,
                env_keys: Default::default(),
                domainforge_candidate: None,
                environment: None,
            })
            .unwrap();
        assert_eq!(decision.verdict, Verdict::Escalate);
        assert!(
            decision
                .required_next_steps
                .contains(&"approval-role:R-SO".into()),
            "{:?}",
            decision
        );

        let mut invalid = complete_v02(base);
        invalid
            .sod_rules
            .iter_mut()
            .find(|rule| rule.name == "break_glass_requester_approver")
            .unwrap()
            .approver_role = "R-AG".into();
        invalid.refresh_policy_bundle_hash().unwrap();
        assert!(PolicyAuthorityEngine::new(invalid).is_err());
    }

    #[test]
    fn rule_disposition_cannot_override_rbac_or_sod() {
        let base: AuthorityPolicyBundle = serde_yaml::from_str(
            "version: \"0.2\"\nallow_degraded: true\nidentity_bindings:\n  - principal: operator_local\n    actor_type: human\n    role: operator\nrules:\n  - name: mutate-policy\n    verdict: allow\n    disposition: degraded\n    compensating_controls: [audit]\n    actor_role: operator\n    operation_kind: policy_mutation\n",
        )
        .unwrap();
        let action = AuthorityAction::Reserved {
            resource_type: "policy_mutation".into(),
            resource_id: "active-policy".into(),
            parameters: json!({}),
        };
        for (remove_permission, expected) in [(false, Verdict::Escalate), (true, Verdict::Deny)] {
            let mut bundle = complete_v02(base.clone());
            if remove_permission {
                bundle
                    .permissions
                    .retain(|permission| permission.role != "operator");
                bundle.refresh_policy_bundle_hash().unwrap();
            }
            let binding = bundle.resolve_identity("operator_local", ActorRole::Operator);
            let engine = PolicyAuthorityEngine::new(bundle).unwrap();
            let actor = actor();
            let decision = engine
                .evaluate(AuthorityEvaluation {
                    actor: &actor,
                    binding,
                    run_id: "run_precedence",
                    case_id: "case_test",
                    plan_item_id: "item_precedence",
                    sequence: 1,
                    action: &action,
                    workspace_root: Path::new("/tmp/workspace"),
                    evidence_refs: vec![],
                    artifacts_root: None,
                    timeout_secs: None,
                    env_keys: Default::default(),
                    domainforge_candidate: None,
                    environment: None,
                })
                .unwrap();
            assert_eq!(decision.verdict, expected);
            assert_ne!(
                decision.normalized_disposition,
                NormalizedDisposition::Degraded
            );
        }
    }

    #[test]
    fn forged_identity_and_substitute_executable_cannot_use_allow_rules() {
        let engine = engine();
        let actor = actor();
        let action = AuthorityAction::WriteFile {
            path: "model.sea".into(),
            content_hint: "fixed".into(),
        };
        let forged = IdentityBinding {
            identity_id: Some("identity:forged".into()),
            principal: "someone_else".into(),
            roles: vec![ActorRole::Operator],
            actor_type: ActorType::Service,
            binding_resolution: BindingResolution::Exact,
            identity_binding_source: "forged".into(),
            source: Some("forged".into()),
            sponsor: None,
            issued_at: None,
            expires_at: None,
            identity_binding_hash: None,
        };
        let decision = engine
            .evaluate_at(
                AuthorityEvaluation {
                    actor: &actor,
                    binding: forged,
                    run_id: "run_20260710T120000Z_abcdef",
                    case_id: "case_test",
                    plan_item_id: "item_01",
                    sequence: 1,
                    action: &action,
                    workspace_root: Path::new("/tmp/workspace"),
                    evidence_refs: vec![],
                    artifacts_root: None,
                    timeout_secs: None,
                    env_keys: Default::default(),
                    domainforge_candidate: None,
                    environment: None,
                },
                "act_abcdef".into(),
                "2026-07-10T12:00:00Z".into(),
            )
            .unwrap();
        assert_eq!(decision.verdict, Verdict::Escalate);
        assert_eq!(decision.reason_codes, ["identity_unresolved"]);
        let bundle: AuthorityPolicyBundle = serde_yaml::from_str("version: \"0.1\"\nrules:\n  - name: allow-self\n    verdict: allow\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n").unwrap();
        let engine = PolicyAuthorityEngine::new(bundle).unwrap();
        let substitute = AuthorityAction::ExecuteCommand {
            argv: vec!["/tmp/sea-forge".into(), "validate".into()],
            cwd: ".".into(),
        };
        let denied = engine
            .evaluate_at(
                AuthorityEvaluation {
                    actor: &actor,
                    binding: binding(),
                    run_id: "run_20260710T120000Z_abcdef",
                    case_id: "case_test",
                    plan_item_id: "item_01",
                    sequence: 1,
                    action: &substitute,
                    workspace_root: Path::new("/tmp/workspace"),
                    evidence_refs: vec![],
                    artifacts_root: None,
                    timeout_secs: None,
                    env_keys: Default::default(),
                    domainforge_candidate: None,
                    environment: None,
                },
                "act_abcdef".into(),
                "2026-07-10T12:00:00Z".into(),
            )
            .unwrap();
        assert_eq!(denied.verdict, Verdict::Deny);
        assert_eq!(denied.reason_codes, ["untrusted_executable"]);
    }

    #[test]
    fn generated_zone_boundary_precedes_allow_rule() {
        let decision = evaluate(&AuthorityAction::WriteFile {
            path: "crate/src/gen/model.rs".into(),
            content_hint: "fixed".into(),
        });
        assert_eq!(decision.verdict, Verdict::Deny);
        assert!(decision
            .reason_codes
            .contains(&"generated_zone_denied".into()));
        let declared_reality = evaluate(&AuthorityAction::WriteFile {
            path: "docs/specs/demo/example.ast.v1.json".into(),
            content_hint: "fixed".into(),
        });
        assert_eq!(declared_reality.verdict, Verdict::Deny);
        for path in [".git", ".ENV", "docs/Secret.txt"] {
            assert_eq!(
                evaluate(&AuthorityAction::WriteFile {
                    path: path.into(),
                    content_hint: "super-secret-token".into()
                })
                .verdict,
                Verdict::Deny
            );
        }
        let secret = evaluate(&AuthorityAction::WriteFile {
            path: ".env".into(),
            content_hint: "super-secret-token".into(),
        });
        assert!(!serde_json::to_string(&secret)
            .unwrap()
            .contains("super-secret-token"));
    }

    #[test]
    fn malformed_executable_actions_are_unclassified() {
        for action in [
            AuthorityAction::WriteFile {
                path: "../escape".into(),
                content_hint: "fixed".into(),
            },
            AuthorityAction::ExecuteCommand {
                argv: vec![],
                cwd: ".".into(),
            },
        ] {
            let decision = evaluate(&action);
            assert_eq!(decision.verdict, Verdict::Deny);
            assert_eq!(decision.reason_codes, ["unclassified"]);
        }
    }

    #[test]
    fn grant_is_bound_to_exact_action_and_context() {
        let engine = engine();
        let action = AuthorityAction::WriteFile {
            path: "model.sea".into(),
            content_hint: "fixed".into(),
        };
        let decision = evaluate_with(&engine, &action);
        let (_root, committed) = commit(&decision);
        let grant = engine.grant(&decision, &committed, &action, None).unwrap();
        assert!(grant
            .authorize(
                &action,
                "run_20260710T120000Z_abcdef",
                "item_01",
                Path::new("/tmp/other")
            )
            .is_err());
    }

    #[test]
    fn denied_or_substituted_action_cannot_receive_grant() {
        let engine = engine();
        let allowed = AuthorityAction::WriteFile {
            path: "model.sea".into(),
            content_hint: "fixed".into(),
        };
        let substitute = AuthorityAction::WriteFile {
            path: "other.sea".into(),
            content_hint: "fixed".into(),
        };
        let decision = evaluate_with(&engine, &allowed);
        let (_root, committed) = commit(&decision);
        assert!(engine
            .grant(&decision, &committed, &substitute, None)
            .is_err());
        let mut denied = evaluate_with(
            &engine,
            &AuthorityAction::WriteFile {
                path: ".env".into(),
                content_hint: "fixed".into(),
            },
        );
        let (_root, committed) = commit(&denied);
        assert!(engine
            .grant(&denied, &committed, &denied.operation, None)
            .is_err());
        denied.verdict = Verdict::Allow;
        denied.outcome = Verdict::Allow;
        let (_root, forged_commit) = commit(&denied);
        assert!(engine
            .grant(&denied, &forged_commit, &denied.operation, None)
            .is_err());
    }

    #[test]
    fn expired_decision_cannot_receive_grant() {
        let engine = engine();
        let actor = actor();
        let action = AuthorityAction::WriteFile {
            path: "model.sea".into(),
            content_hint: "fixed".into(),
        };
        let decision = engine
            .evaluate_at(
                AuthorityEvaluation {
                    actor: &actor,
                    binding: binding(),
                    run_id: "run_20260710T120000Z_abcdef",
                    case_id: "case_test",
                    plan_item_id: "item_01",
                    sequence: 1,
                    action: &action,
                    workspace_root: Path::new("/tmp/workspace"),
                    evidence_refs: vec![],
                    artifacts_root: None,
                    timeout_secs: None,
                    env_keys: Default::default(),
                    domainforge_candidate: None,
                    environment: None,
                },
                "act_expired".into(),
                "2026-07-10T12:00:00Z".into(),
            )
            .unwrap();
        let (_root, committed) = commit(&decision);
        assert!(engine.grant(&decision, &committed, &action, None).is_err());
    }

    #[test]
    fn execution_grant_rejects_timeout_or_artifact_substitution() {
        let executable = std::env::current_exe().unwrap();
        let argv0 = executable.file_name().unwrap().to_string_lossy();
        let bundle: AuthorityPolicyBundle = serde_yaml::from_str(&format!(
            "version: \"0.1\"\nrules:\n  - name: execute\n    verdict: allow\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: {argv0}\n"
        ))
        .unwrap();
        let engine = PolicyAuthorityEngine::new(bundle).unwrap();
        let actor = actor();
        let action = AuthorityAction::ExecuteCommand {
            argv: vec![executable.to_string_lossy().into_owned()],
            cwd: ".".into(),
        };
        let decision = engine
            .evaluate(AuthorityEvaluation {
                actor: &actor,
                binding: binding(),
                run_id: "run_context",
                case_id: "case_test",
                plan_item_id: "item_context",
                sequence: 1,
                action: &action,
                workspace_root: Path::new("/tmp/workspace"),
                evidence_refs: vec![],
                artifacts_root: Some(Path::new("/tmp/artifacts")),
                timeout_secs: Some(10),
                env_keys: BTreeSet::from(["PATH".into()]),
                domainforge_candidate: None,
                environment: None,
            })
            .unwrap();
        let (_root, committed) = commit(&decision);
        let grant = engine.grant(&decision, &committed, &action, None).unwrap();
        assert!(grant
            .authorize_execution(
                &action,
                ExecutionGrantContext {
                    run_id: "run_context",
                    plan_item_id: "item_context",
                    workspace_root: Path::new("/tmp/workspace"),
                    artifacts_root: Path::new("/tmp/other-artifacts"),
                    timeout_secs: 11,
                    env_keys: BTreeSet::from(["PATH".into()]),
                    compensating_controls: &[],
                },
            )
            .is_err());
    }

    #[test]
    fn boundary_and_degraded_requirements_survive_into_grants() {
        for (extra_policy, expected, controls) in [
            (
                "    disposition: boundary\n    boundary_constraints:\n      workspace: [/tmp/workspace]\n",
                NormalizedDisposition::Boundary,
                vec![],
            ),
            (
                "    disposition: degraded\n    compensating_controls: [audit]\n",
                NormalizedDisposition::Degraded,
                vec!["audit".into()],
            ),
        ] {
            let bundle: AuthorityPolicyBundle = serde_yaml::from_str(&format!(
                "version: \"0.1\"\nallow_degraded: true\nidentity_bindings:\n  - principal: operator_local\n    actor_type: human\n    role: operator\nrules:\n  - name: constrained-write\n    verdict: allow\n    actor_role: operator\n    operation_kind: write_file\n    path_prefix: \"\"\n{extra_policy}"
            ))
            .unwrap();
            let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
            let actor = actor();
            let action = AuthorityAction::WriteFile {
                path: "model.sea".into(),
                content_hint: "fixed".into(),
            };
            let decision = engine
                .evaluate(AuthorityEvaluation {
                    actor: &actor,
                    binding: bundle.resolve_identity("operator_local", ActorRole::Operator),
                    run_id: "run_constraints",
                    case_id: "case_test",
                    plan_item_id: "item_constraints",
                    sequence: 1,
                    action: &action,
                    workspace_root: Path::new("/tmp/workspace"),
                    evidence_refs: vec![],
                    artifacts_root: None,
                    timeout_secs: None,
                    env_keys: Default::default(),
                    domainforge_candidate: None,
                environment: None,
                })
                .unwrap();
            assert_eq!(decision.verdict, Verdict::Allow);
            assert_eq!(decision.normalized_disposition, expected);
            assert_eq!(
                decision.audit_record.disposition,
                match expected {
                    NormalizedDisposition::Boundary => "boundary",
                    NormalizedDisposition::Degraded => "degraded",
                    _ => unreachable!(),
                }
            );
            assert_eq!(decision.audit_record.subject, "file:model.sea");
            assert_eq!(decision.audit_record.case_id.as_deref(), Some("case_test"));
            let (_root, committed) = commit(&decision);
            engine
                .grant(&decision, &committed, &action, None)
                .unwrap()
                .authorize_with_controls(
                    &action,
                    "run_constraints",
                    "item_constraints",
                    Path::new("/tmp/workspace"),
                    &controls,
                )
                .unwrap();
        }
    }
}

#[cfg(test)]
mod resolver_tests {
    use super::*;

    fn verdict(disposition: GovernanceDisposition) -> GovernanceVerdict {
        GovernanceVerdict {
            engine: "test".into(),
            disposition,
            boundaries: BTreeMap::new(),
            compensating_controls: BTreeSet::new(),
            reason: "fixture".into(),
            evidence_refs: vec!["ev_1".into()],
        }
    }

    #[test]
    fn allow_is_identity_and_candidate_order_is_irrelevant() {
        let boundary = GovernanceVerdict {
            boundaries: BTreeMap::from([("workspace".into(), BTreeSet::from(["/safe".into()]))]),
            ..verdict(GovernanceDisposition::Boundary)
        };
        let left = resolve_candidates(
            &[verdict(GovernanceDisposition::Allow), boundary.clone()],
            &ResolutionPolicy::default(),
        );
        let right = resolve_candidates(
            &[boundary, verdict(GovernanceDisposition::Allow)],
            &ResolutionPolicy::default(),
        );
        assert_eq!(left, right);
        assert_eq!(left.disposition, GovernanceDisposition::Boundary);
    }

    #[test]
    fn escalation_blocks_while_retaining_boundaries() {
        let boundary = GovernanceVerdict {
            boundaries: BTreeMap::from([("workspace".into(), BTreeSet::from(["/safe".into()]))]),
            ..verdict(GovernanceDisposition::Boundary)
        };
        let resolved = resolve_candidates(
            &[boundary, verdict(GovernanceDisposition::Escalate)],
            &ResolutionPolicy::default(),
        );
        assert_eq!(resolved.disposition, GovernanceDisposition::Escalate);
        assert_eq!(
            resolved.boundaries,
            BTreeMap::from([("workspace".into(), BTreeSet::from(["/safe".into()]))])
        );
    }

    #[test]
    fn incompatible_boundaries_deny() {
        let first = GovernanceVerdict {
            boundaries: BTreeMap::from([("workspace".into(), BTreeSet::from(["/one".into()]))]),
            ..verdict(GovernanceDisposition::Boundary)
        };
        let second = GovernanceVerdict {
            boundaries: BTreeMap::from([("workspace".into(), BTreeSet::from(["/two".into()]))]),
            ..verdict(GovernanceDisposition::Boundary)
        };
        assert_eq!(
            resolve_candidates(&[first, second], &ResolutionPolicy::default()).disposition,
            GovernanceDisposition::Deny
        );
    }

    #[test]
    fn deny_blocks_every_other_candidate() {
        let resolved = resolve_candidates(
            &[
                verdict(GovernanceDisposition::Allow),
                verdict(GovernanceDisposition::Deny),
                verdict(GovernanceDisposition::Escalate),
            ],
            &ResolutionPolicy::default(),
        );
        assert_eq!(resolved.disposition, GovernanceDisposition::Deny);
    }

    #[test]
    fn degraded_requires_permission_and_accumulates_controls() {
        let degraded = GovernanceVerdict {
            compensating_controls: BTreeSet::from(["audit".into(), "read_only".into()]),
            ..verdict(GovernanceDisposition::Degraded)
        };
        let denied = resolve_candidates(
            std::slice::from_ref(&degraded),
            &ResolutionPolicy::default(),
        );
        assert_eq!(denied.disposition, GovernanceDisposition::Deny);
        let permitted = resolve_candidates(
            &[degraded],
            &ResolutionPolicy {
                allow_degraded: true,
            },
        );
        assert_eq!(permitted.disposition, GovernanceDisposition::Degraded);
        assert_eq!(permitted.compensating_controls.len(), 2);
    }

    #[test]
    fn missing_required_evidence_denies() {
        let mut candidate = verdict(GovernanceDisposition::Allow);
        candidate.evidence_refs.clear();
        let resolved = resolve_candidates(&[candidate], &ResolutionPolicy::default());
        assert_eq!(resolved.disposition, GovernanceDisposition::Deny);
    }

    fn capitalize_action(kind: &str) -> AuthorityAction {
        let (from, to, strength, value, requires) = match kind {
            "synthesize" => ("cognitive", "intellectual", "local", json!([]), false),
            "productize" => ("intellectual", "product", "local", json!([]), false),
            "capitalize" => ("product", "capital", "strong", json!(["adoption"]), true),
            other => panic!("unknown transition kind {other}"),
        };
        AuthorityAction::Reserved {
            resource_type: "transition_artifact_stage".into(),
            resource_id: "art_model".into(),
            parameters: json!({
                "transition_kind": kind,
                "from_stage": from,
                "to_stage": to,
                "mode": "promote",
                "gate_profile_ref": format!("{kind}@1"),
                "required_settlement_strength": strength,
                "qualifying_value_evidence_kinds": value,
                "requires_approval": requires,
                "licenses": ["internal"],
            }),
        }
    }

    fn capitalize_sod_bundle(requester_role: &str) -> AuthorityPolicyBundle {
        let mut base: AuthorityPolicyBundle = serde_yaml::from_str(
            r#"version: "0.2"
identity_bindings:
  - principal: operator_local
    actor_type: human
    role: operator
  - principal: security_officer
    actor_type: human
    role: R-SO
  - principal: wrong_resolver
    actor_type: human
    role: R-DEV
rules:
  - name: synthesize
    verdict: allow
    actor_role: operator
    operation_kind: transition_artifact_stage
    transition_kind: synthesize
    from_stage: cognitive
    to_stage: intellectual
    modes: [promote]
    gate_profile_ref: synthesize@1
    required_settlement_strength: local
    qualifying_value_evidence_kinds: []
    requires_approval: false
    license_allowlist: [internal]
  - name: productize
    verdict: allow
    actor_role: operator
    operation_kind: transition_artifact_stage
    transition_kind: productize
    from_stage: intellectual
    to_stage: product
    modes: [promote]
    gate_profile_ref: productize@1
    required_settlement_strength: local
    qualifying_value_evidence_kinds: []
    requires_approval: false
    license_allowlist: [internal]
  - name: capitalize
    verdict: allow
    actor_role: operator
    operation_kind: transition_artifact_stage
    transition_kind: capitalize
    from_stage: product
    to_stage: capital
    modes: [promote]
    gate_profile_ref: capitalize@1
    required_settlement_strength: strong
    qualifying_value_evidence_kinds: [adoption]
    requires_approval: true
    license_allowlist: [internal]
  - name: resolve-approval
    verdict: allow
    actor_role: R-SO
    operation_kind: approval_resolution
"#,
        )
        .unwrap();
        base = super::tests::complete_v02(base);
        if let Some(rule) = base
            .sod_rules
            .iter_mut()
            .find(|rule| rule.name == "capitalization_requester_approver")
        {
            rule.requester_role = requester_role.into();
            rule.approver_role = "R-SO".into();
            rule.operation_kind = Some("transition_artifact_stage".into());
            rule.transition_kind = Some("capitalize".into());
            rule.allow_same_principal = false;
        }
        base.refresh_policy_bundle_hash().unwrap();
        base
    }

    fn evaluate_capital(engine: &PolicyAuthorityEngine, kind: &str) -> AuthorityDecision {
        let actor = Actor {
            actor_id: "operator_local".into(),
            role: ActorRole::Operator,
        };
        let action = capitalize_action(kind);
        engine
            .evaluate(AuthorityEvaluation {
                actor: &actor,
                binding: engine
                    .bundle
                    .resolve_identity("operator_local", ActorRole::Operator),
                run_id: "run_capital_sod",
                case_id: "case_test",
                plan_item_id: "item_capital",
                sequence: 1,
                action: &action,
                workspace_root: Path::new("/tmp/workspace"),
                evidence_refs: vec![],
                artifacts_root: None,
                timeout_secs: None,
                env_keys: Default::default(),
                domainforge_candidate: None,
                environment: None,
            })
            .unwrap()
    }

    #[test]
    fn capitalize_sod_escalates_only_capitalize_with_role_and_decision_markers() {
        let bundle = capitalize_sod_bundle("operator");
        let sod = bundle
            .sod_rules
            .iter()
            .find(|rule| rule.name == "capitalization_requester_approver")
            .unwrap();
        assert!(sod_rule_matches_action(
            sod,
            "operator",
            &capitalize_action("capitalize")
        ));
        let engine = PolicyAuthorityEngine::new(bundle).unwrap();
        let synthesize = evaluate_capital(&engine, "synthesize");
        assert_eq!(
            synthesize.verdict,
            Verdict::Allow,
            "{:?}",
            synthesize.policy_refs
        );
        let productize = evaluate_capital(&engine, "productize");
        assert_eq!(
            productize.verdict,
            Verdict::Allow,
            "{:?}",
            productize.policy_refs
        );
        let capital = evaluate_capital(&engine, "capitalize");
        assert_eq!(capital.verdict, Verdict::Escalate);
        assert!(
            capital
                .policy_refs
                .iter()
                .any(|r| r == "sod: capitalization_requester_approver"),
            "{:?}",
            capital.policy_refs
        );
        assert!(
            capital
                .required_next_steps
                .contains(&"approval-role:R-SO".into()),
            "{:?}",
            capital.required_next_steps
        );
    }

    #[test]
    fn sod_requester_role_must_match_actor() {
        let mismatched = PolicyAuthorityEngine::new(capitalize_sod_bundle("R-DEV")).unwrap();
        let decision = evaluate_capital(&mismatched, "capitalize");
        assert_eq!(decision.verdict, Verdict::Escalate);
        assert!(
            !decision
                .policy_refs
                .iter()
                .any(|reason| reason == "sod: capitalization_requester_approver"),
            "requester_role R-DEV must not apply to operator"
        );
        let matched = PolicyAuthorityEngine::new(capitalize_sod_bundle("operator")).unwrap();
        assert_eq!(
            evaluate_capital(&matched, "capitalize").verdict,
            Verdict::Escalate,
            "matching requester_role must apply"
        );
    }

    #[test]
    fn sod_transition_kind_validation_is_fail_closed() {
        let path = Path::new("policy.yaml");
        let mut unscoped = capitalize_sod_bundle("operator");
        unscoped
            .sod_rules
            .iter_mut()
            .find(|rule| rule.name == "capitalization_requester_approver")
            .unwrap()
            .transition_kind = None;
        unscoped.refresh_policy_bundle_hash().unwrap();
        let err = unscoped.validate(path).unwrap_err();
        assert_eq!(err.class(), "schema_error");

        let mut unknown = capitalize_sod_bundle("operator");
        unknown
            .sod_rules
            .iter_mut()
            .find(|rule| rule.name == "capitalization_requester_approver")
            .unwrap()
            .transition_kind = Some("teleport".into());
        unknown.refresh_policy_bundle_hash().unwrap();
        let err = unknown.validate(path).unwrap_err();
        assert_eq!(err.class(), "schema_error");

        let mut non_transition = capitalize_sod_bundle("operator");
        non_transition
            .sod_rules
            .iter_mut()
            .find(|rule| rule.name == "break_glass_requester_approver")
            .unwrap()
            .transition_kind = Some("capitalize".into());
        non_transition.refresh_policy_bundle_hash().unwrap();
        let err = non_transition.validate(path).unwrap_err();
        assert_eq!(err.class(), "schema_error");
    }

    #[test]
    fn legacy_non_transition_sod_omits_transition_kind_in_serialization() {
        let rule = SodRule {
            name: "write_requires_security_officer".into(),
            requester_role: "operator".into(),
            approver_role: "R-SO".into(),
            operation_kind: Some("write_file".into()),
            transition_kind: None,
            allow_same_principal: false,
        };
        let value = serde_json::to_value(&rule).unwrap();
        assert!(
            value.get("transition_kind").is_none(),
            "absent transition_kind must not serialize as null: {value}"
        );
    }

    #[test]
    fn capitalize_sod_post_approval_grant_and_exact_action_binding() {
        let bundle = capitalize_sod_bundle("operator");
        let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
        let action = capitalize_action("capitalize");
        let decision = evaluate_capital(&engine, "capitalize");
        assert_eq!(decision.verdict, Verdict::Escalate);
        assert!(decision
            .policy_refs
            .iter()
            .any(|r| r == "sod: capitalization_requester_approver"));

        let criteria = SettlementCriteriaRecord {
            version: RECORD_VERSION.into(),
            criteria_id: "crit_capital_sod".into(),
            criteria: SettlementCriteria {
                require_exit_zero: true,
                required_artifacts: vec![],
                stdout_must_contain: None,
                require_approval: true,
                evaluator: None,
                records: None,
                per_record_evaluator: None,
                min_pass_ratio: None,
            },
            origin_refs: vec![OriginRef {
                kind: OriginRefKind::Intent,
                reference: "int_capital".into(),
                sha256: "sha256:intent".into(),
                role: OriginRole::AcceptanceSource,
                evidence_refs: vec![],
                domain_model_ref: None,
            }],
            derivation: CriteriaDerivation {
                method: DerivationMethod::DeterministicPlanner,
                actor_ref: "operator_local".into(),
                producer_ref: None,
                rationale: "capitalization SOD fixture".into(),
            },
            declared_at: Utc::now().to_rfc3339(),
            criteria_sha256: String::new(),
            criteria_record_hash: String::new(),
        };
        let mut criteria = criteria;
        criteria.criteria_sha256 =
            sea_forge_ledger::types::hash_canonical(&criteria.criteria).unwrap();
        let mut unhashed = criteria.clone();
        unhashed.criteria_record_hash.clear();
        criteria.criteria_record_hash = sea_forge_ledger::types::hash_canonical(&unhashed).unwrap();

        let approval = ApprovalRequest {
            version: RECORD_VERSION.into(),
            approval_id: "apr_capital_sod".into(),
            run_id: decision.run_id.clone(),
            case_id: decision.audit_record.case_id.clone().unwrap(),
            decision_id: decision.decision_id.clone(),
            plan_item_id: decision.plan_item_id.clone(),
            criteria_ref: Some(criteria.criteria_id.clone()),
            criteria_sha256: Some(criteria.criteria_sha256.clone()),
            criteria_record_hash: Some(criteria.criteria_record_hash.clone()),
            job_contract_ref: None,
            requested_at: Utc::now().to_rfc3339(),
            expires_at: (Utc::now() + chrono::Duration::hours(1)).to_rfc3339(),
            status: ApprovalStatus::Approved,
            resolved_by: Some("security_officer".into()),
            resolved_at: Some(Utc::now().to_rfc3339()),
            note: None,
        };

        let root = tempfile::tempdir().unwrap();
        let stream = LedgerStream::open(root.path(), "case-case_test", "test-writer").unwrap();
        let decision_ref = stream
            .commit_typed("authority_decision", vec![], &decision, vec![])
            .unwrap();
        let criteria_ref = stream
            .commit_typed("settlement_criteria", vec![], &criteria, vec![])
            .unwrap();
        let approval_ref = stream
            .commit_typed("approval_resolution", vec![], &approval, vec![])
            .unwrap();

        // Non-R-SO resolver is rejected.
        let mut wrong_role = approval.clone();
        wrong_role.resolved_by = Some("wrong_resolver".into());
        wrong_role.approval_id = "apr_capital_sod_wrong".into();
        let wrong_ref = stream
            .commit_typed("approval_resolution", vec![], &wrong_role, vec![])
            .unwrap();
        assert!(engine
            .grant_after_approval(
                &decision,
                &decision_ref,
                &action,
                &wrong_role,
                &wrong_ref,
                &criteria,
                &criteria_ref,
                None,
                &stream,
            )
            .is_err());

        // Substituting synthesize/productize parameters fails exact-action binding
        // (and must not be granted under the capitalization decision).
        for kind in ["synthesize", "productize"] {
            let substituted = capitalize_action(kind);
            let mut substituted_approval = approval.clone();
            substituted_approval.approval_id = format!("apr_capital_sod_{kind}");
            let substituted_ref = stream
                .commit_typed("approval_resolution", vec![], &substituted_approval, vec![])
                .unwrap();
            let error = match engine.grant_after_approval(
                &decision,
                &decision_ref,
                &substituted,
                &substituted_approval,
                &substituted_ref,
                &criteria,
                &criteria_ref,
                None,
                &stream,
            ) {
                Ok(_) => panic!("substituted {kind} must fail exact-action validation"),
                Err(error) => error,
            };
            assert!(
                error.to_string().contains("does not grant this action"),
                "substituted {kind} must fail exact-action validation: {error}"
            );
        }

        // R-SO resolver distinct from requester succeeds; SOD match agrees.
        let grant = engine
            .grant_after_approval(
                &decision,
                &decision_ref,
                &action,
                &approval,
                &approval_ref,
                &criteria,
                &criteria_ref,
                None,
                &stream,
            )
            .unwrap();
        grant
            .authorize(
                &action,
                &decision.run_id,
                &decision.plan_item_id,
                Path::new("/tmp/workspace"),
            )
            .unwrap();
    }
}
