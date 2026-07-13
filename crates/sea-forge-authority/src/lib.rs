use chrono::Utc;
use sea_forge_core::{errors::ForgeError, ids::random_id, types::*, RECORD_VERSION};
use sea_forge_ledger::{types::payload_hash, CommittedRecordRef};
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

fn issued_decision_key(decision: &AuthorityDecision) -> String {
    format!(
        "{}:{}:{}",
        decision.decision_id,
        decision.determinism.policy_bundle_hash,
        decision.determinism.action_request_hash
    )
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
    expires_at: chrono::DateTime<Utc>,
}

impl ActionGrant {
    pub fn authorize(
        self,
        action: &AuthorityAction,
        run_id: &str,
        plan_item_id: &str,
        workspace_root: &Path,
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
        Ok(())
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
    #[serde(default = "default_deny")]
    pub default: String,
    #[serde(default)]
    pub identity: IdentityPolicy,
    #[serde(default)]
    pub policy_surfaces: PolicySurfaces,
    pub rules: Vec<PolicyRule>,
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
}

impl AuthorityPolicyBundle {
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
        let bundle: Self = serde_yaml::from_value(yaml).map_err(|_| ForgeError::Config {
            class: "schema_error",
            path: path.into(),
            message: "policy does not match the required schema".into(),
        })?;
        bundle.validate(path)?;
        Ok(bundle)
    }
    fn validate(&self, path: &Path) -> Result<(), ForgeError> {
        let schema = |message: String| ForgeError::Config {
            class: "schema_error",
            path: path.into(),
            message,
        };
        if self.version != RECORD_VERSION {
            return Err(schema("version must equal 0.1".into()));
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
        for rule in &self.rules {
            if rule.name.is_empty() || !names.insert(&rule.name) {
                return Err(schema("rule names must be non-empty and unique".into()));
            }
            if !matches!(
                rule.operation_kind.as_str(),
                "write_file" | "execute_command"
            ) {
                return Err(ForgeError::Config {
                    class: "unsupported_kind_error",
                    path: path.into(),
                    message: "policy contains an unsupported operation_kind".into(),
                });
            }
            if rule.operation_kind == "execute_command"
                && rule.verdict == Verdict::Allow
                && rule.argv0.is_none()
            {
                return Err(schema("allow execute_command rule requires argv0".into()));
            }
        }
        Ok(())
    }
}

pub struct PolicyAuthorityEngine {
    bundle: AuthorityPolicyBundle,
    bundle_hash: String,
    issued_decisions: Mutex<HashSet<String>>,
}

pub struct AuthorityEvaluation<'a> {
    pub actor: &'a Actor,
    pub binding: IdentityBinding,
    pub run_id: &'a str,
    pub plan_item_id: &'a str,
    pub sequence: usize,
    pub action: &'a AuthorityAction,
    pub workspace_root: &'a Path,
}
impl PolicyAuthorityEngine {
    pub fn new(bundle: AuthorityPolicyBundle) -> Result<Self, ForgeError> {
        bundle.validate(Path::new("<in-memory-policy>"))?;
        let bundle_hash = hash_canonical(&bundle)?;
        Ok(Self {
            bundle,
            bundle_hash,
            issued_decisions: Mutex::new(HashSet::new()),
        })
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
        workspace_root: &Path,
    ) -> Result<ActionGrant, ForgeError> {
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
        let decision_hash = issued_decision_key(decision);
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
        Ok(ActionGrant {
            action: action.clone(),
            run_id: decision.run_id.clone(),
            plan_item_id: decision.plan_item_id.clone(),
            workspace_root: workspace_root.to_path_buf(),
            expires_at: Utc::now() + chrono::Duration::minutes(5),
        })
    }

    pub fn evaluate_at(
        &self,
        input: AuthorityEvaluation<'_>,
        action_id: String,
        timestamp: String,
    ) -> Result<AuthorityDecision, ForgeError> {
        let AuthorityEvaluation {
            actor,
            binding,
            run_id,
            plan_item_id,
            sequence,
            action,
            workspace_root,
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
            context: json!({"repo":null,"branch":null,"environment":"local-slice","workspace_root":workspace_root,"source_platform":"cli","channel":"cli"}),
            evidence: json!({"identity_binding_source":binding.identity_binding_source,"tool_trace_ref":null,"payload_hash":hash_canonical(action)?}),
        };
        let identity_valid = binding.principal == actor.actor_id
            && binding.actor_type == actor_type_for_role(&actor.role);
        let (verdict, matched, reasons, refs, next) =
            if binding.binding_resolution == BindingResolution::Unresolved || !identity_valid {
                (
                    Verdict::Escalate,
                    None,
                    vec!["identity_unresolved".into()],
                    vec!["identity-binding:unresolved".into()],
                    vec!["complete-onboarding".into()],
                )
            } else if matches!(action, AuthorityAction::Unclassified { .. })
                || malformed_action(action)
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
            } else if matches!(
                action,
                AuthorityAction::GitCommit { .. }
                    | AuthorityAction::GithubPr { .. }
                    | AuthorityAction::Reserved { .. }
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
                (
                    rule.verdict.clone(),
                    Some(rule.name.clone()),
                    vec![format!(
                        "policy_{}",
                        match rule.verdict {
                            Verdict::Allow => "allow",
                            Verdict::Deny => "deny",
                            Verdict::Escalate => "escalate",
                        }
                    )],
                    vec![format!("local-policy:{}", rule.name)],
                    if rule.verdict == Verdict::Escalate {
                        vec!["require-human-approval".into()]
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
        let disposition = match verdict {
            Verdict::Allow => NormalizedDisposition::Allow,
            Verdict::Deny => NormalizedDisposition::Deny,
            Verdict::Escalate => NormalizedDisposition::Escalate,
        };
        let reason = reasons.join(",");
        let decision = AuthorityDecision {
            version: RECORD_VERSION.into(),
            decision_id: format!("auth_{sequence:02}"),
            run_id: run_id.into(),
            plan_item_id: plan_item_id.into(),
            action_id,
            correlation_id: run_id.into(),
            operation: redacted_action(action),
            outcome: verdict.clone(),
            verdict: verdict.clone(),
            normalized_disposition: disposition,
            matched_rule: matched,
            reason_codes: reasons,
            reason: reason.clone(),
            policy_refs: refs,
            required_next_steps: next,
            identity_binding: binding.clone(),
            determinism: Determinism {
                policy_bundle_hash: self.bundle_hash.clone(),
                action_request_hash: hash_canonical(&request)?,
                identity_binding_hash: hash_canonical(&binding)?,
            },
            action_request: request,
            audit_record: AuditRecord {
                engine: "sea-forge-authority".into(),
                disposition: format!("{verdict:?}").to_lowercase(),
                subject: actor.actor_id.clone(),
                reason,
                evidence_refs: vec![],
                recorded_at: timestamp.clone(),
            },
            decided_at: timestamp,
        };
        self.issued_decisions
            .lock()
            .map_err(|_| ForgeError::Internal("authority issuer state poisoned".into()))?
            .insert(issued_decision_key(&decision));
        Ok(decision)
    }
}
fn actor_type_for_role(role: &ActorRole) -> ActorType {
    match role {
        ActorRole::Operator => ActorType::Human,
        ActorRole::Agent => ActorType::Agent,
        ActorRole::System => ActorType::System,
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
    fn actor() -> Actor {
        Actor {
            actor_id: "operator_local".into(),
            role: ActorRole::Operator,
        }
    }
    fn binding() -> IdentityBinding {
        IdentityBinding {
            principal: "operator_local".into(),
            actor_type: ActorType::Human,
            binding_resolution: BindingResolution::LocalDefault,
            identity_binding_source: "local-slice-default".into(),
            sponsor: None,
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
                    plan_item_id: "item_01",
                    sequence: 1,
                    action,
                    workspace_root: Path::new("/tmp/workspace"),
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
                    plan_item_id: "item_01",
                    sequence: 1,
                    action,
                    workspace_root: Path::new("/tmp/workspace"),
                },
                "act_abcdef".into(),
                "2026-07-10T12:00:00Z".into(),
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
                    plan_item_id: "item_01",
                    sequence: 1,
                    action: &action,
                    workspace_root: Path::new("/tmp/workspace"),
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
    fn forged_identity_and_substitute_executable_cannot_use_allow_rules() {
        let engine = engine();
        let actor = actor();
        let action = AuthorityAction::WriteFile {
            path: "model.sea".into(),
            content_hint: "fixed".into(),
        };
        let forged = IdentityBinding {
            principal: "someone_else".into(),
            actor_type: ActorType::Service,
            binding_resolution: BindingResolution::Exact,
            identity_binding_source: "forged".into(),
            sponsor: None,
        };
        let decision = engine
            .evaluate_at(
                AuthorityEvaluation {
                    actor: &actor,
                    binding: forged,
                    run_id: "run_20260710T120000Z_abcdef",
                    plan_item_id: "item_01",
                    sequence: 1,
                    action: &action,
                    workspace_root: Path::new("/tmp/workspace"),
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
                    plan_item_id: "item_01",
                    sequence: 1,
                    action: &substitute,
                    workspace_root: Path::new("/tmp/workspace"),
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
        let grant = engine
            .grant(&decision, &committed, &action, Path::new("/tmp/workspace"))
            .unwrap();
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
            .grant(
                &decision,
                &committed,
                &substitute,
                Path::new("/tmp/workspace"),
            )
            .is_err());
        let denied = evaluate_with(
            &engine,
            &AuthorityAction::WriteFile {
                path: ".env".into(),
                content_hint: "fixed".into(),
            },
        );
        let (_root, committed) = commit(&denied);
        assert!(engine
            .grant(
                &denied,
                &committed,
                &denied.operation,
                Path::new("/tmp/workspace"),
            )
            .is_err());
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
