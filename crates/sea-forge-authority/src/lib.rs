use chrono::Utc;
use sea_forge_core::{errors::ForgeError, ids::random_id, types::*, RECORD_VERSION};
use sea_forge_domainforge::{
    evaluate_authority, CandidateDisposition, DomainForgeTrace, DomainModel, DomainModelRef,
};
use sea_forge_ledger::{types::payload_hash, CommittedRecordRef, PreActionAssurance};
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
    pub requires_approval: bool,
    #[serde(default)]
    pub memory_scope: Option<String>,
}

impl AuthorityPolicyBundle {
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
            const SOD: &[(&str, &str)] = &[
                ("production_proposer_approver", "pr_merge"),
                (
                    "semantic_debt_requester_acceptor",
                    "settlement_authority_mutation",
                ),
                ("break_glass_requester_approver", "policy_mutation"),
                ("key_generator_approver", "identity_minting"),
                ("capitalization_requester_approver", "artifact_transition"),
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
                || SOD.iter().any(|(name, operation)| {
                    !self.sod_rules.iter().any(|rule| {
                        rule.name == *name && rule.operation_kind.as_deref() == Some(*operation)
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
        let mut names = HashSet::new();
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
        if self.sod_rules.iter().any(|rule| {
            rule.name.is_empty()
                || rule.requester_role.is_empty()
                || rule.approver_role.is_empty()
                || rule.operation_kind.as_deref().is_none_or(str::is_empty)
                || rule.allow_same_principal
        }) {
            return Err(schema("invalid separation-of-duty rule".into()));
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
                    | "run_spec_pipeline"
                    | "run_projection"
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
        if decision.verdict != Verdict::Allow || bound_hash != Some(action_hash.as_str()) {
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
        let decided_at = chrono::DateTime::parse_from_rfc3339(&decision.decided_at)
            .map_err(|_| ForgeError::Input("authority decision timestamp is invalid".into()))?
            .with_timezone(&Utc);
        let expires_at = decided_at + chrono::Duration::minutes(5);
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
            sandbox_class: decision
                .sandbox_class_granted
                .clone()
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
        } else if let Some(rule) = self
            .bundle
            .rules
            .iter()
            .find(|r| matches_rule(r, actor, action))
        {
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
            let matching_sod = self
                .bundle
                .sod_rules
                .iter()
                .filter(|sod| sod.operation_kind.as_deref() == Some(&rule.operation_kind))
                .collect::<Vec<_>>();
            let sod_requires_approval = !matching_sod.is_empty();
            let effective_verdict = if !has_permission {
                Verdict::Deny
            } else if rule.requires_approval || sod_requires_approval {
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
fn matches_rule(rule: &PolicyRule, actor: &Actor, action: &AuthorityAction) -> bool {
    if rule.actor_role != actor.role {
        return false;
    }
    match action {
        AuthorityAction::WriteFile { path, .. } => {
            let prefix = rule.path_prefix.as_deref().unwrap_or("");
            rule.operation_kind == "write_file" && (prefix.is_empty() || path.starts_with(prefix))
        }
        AuthorityAction::ExecuteCommand { argv, .. } => {
            rule.operation_kind == "execute_command"
                && rule.argv0.as_deref().is_none_or(|expected| {
                    argv.first()
                        .and_then(|v| Path::new(v).file_name())
                        .and_then(|v| v.to_str())
                        == Some(expected)
                })
        }
        AuthorityAction::Reserved { resource_type, .. } => rule.operation_kind == *resource_type,
        _ => false,
    }
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
                "identity_binding",
                "install_extension",
                "adopt_extension",
                "disable_extension",
                "projection_execution",
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
    fn complete_v02(mut bundle: AuthorityPolicyBundle) -> AuthorityPolicyBundle {
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
        const SOD: &[(&str, &str)] = &[
            ("production_proposer_approver", "pr_merge"),
            (
                "semantic_debt_requester_acceptor",
                "settlement_authority_mutation",
            ),
            ("break_glass_requester_approver", "policy_mutation"),
            ("key_generator_approver", "identity_minting"),
            ("capitalization_requester_approver", "artifact_transition"),
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
        bundle.sod_rules = SOD
            .iter()
            .map(|(name, operation)| SodRule {
                name: (*name).into(),
                requester_role: "R-DEV".into(),
                approver_role: "R-SO".into(),
                allow_same_principal: false,
                operation_kind: Some((*operation).into()),
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
}
