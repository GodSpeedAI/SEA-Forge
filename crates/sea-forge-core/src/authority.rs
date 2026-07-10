use crate::{
    errors::ForgeError, evidence::hash_canonical, ids::random_id, types::*, RECORD_VERSION,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::Path,
};

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
    pub external_api: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub git_commit: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub github_pr: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub prompt_risk: BTreeMap<String, serde_json::Value>,
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
    vec!["**/src/gen/**", ".git/**", ".env*", "**/*secret*"]
        .into_iter()
        .map(str::to_owned)
        .collect()
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
        let bundle: Self = serde_yaml::from_str(&text).map_err(|e| ForgeError::Config {
            class: "parse_error",
            path: path.into(),
            message: e.to_string(),
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
                    message: format!("unsupported operation_kind {}", rule.operation_kind),
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
}

pub struct AuthorityEvaluation<'a> {
    pub actor: &'a Actor,
    pub binding: IdentityBinding,
    pub run_id: &'a str,
    pub plan_item_id: &'a str,
    pub sequence: usize,
    pub operation: &'a Operation,
    pub workspace_root: &'a Path,
}
impl PolicyAuthorityEngine {
    pub fn new(bundle: AuthorityPolicyBundle) -> Result<Self, ForgeError> {
        let bundle_hash = hash_canonical(&bundle)?;
        Ok(Self {
            bundle,
            bundle_hash,
        })
    }
    pub fn evaluate(
        &self,
        input: AuthorityEvaluation<'_>,
    ) -> Result<AuthorityDecision, ForgeError> {
        let AuthorityEvaluation {
            actor,
            binding,
            run_id,
            plan_item_id,
            sequence,
            operation,
            workspace_root,
        } = input;
        let now = Utc::now().to_rfc3339();
        let action_id = random_id("act")?;
        let (kind, resource_id, parameters) = match operation {
            Operation::WriteFile { path, content_hint } => (
                "write_file",
                path.clone(),
                json!({"path":path,"content_hint":content_hint}),
            ),
            Operation::ExecuteCommand { argv, cwd } => (
                "execute_command",
                argv.first().cloned().unwrap_or_default(),
                json!({"argv":argv,"cwd":cwd}),
            ),
        };
        let request = AuthorityRequest {
            schema_version: "cam.v1".into(),
            action_id: action_id.clone(),
            correlation_id: run_id.into(),
            timestamp_utc: now.clone(),
            actor: json!({"actor_id":actor.actor_id,"actor_type":binding.actor_type,"principal":binding.principal}),
            action: json!({"tool_name":"sea-forge-cli","operation":kind,"resource_type":if kind=="write_file"{"file"}else{"shell_cmd"},"resource_id":resource_id,"parameters":parameters}),
            context: json!({"repo":null,"branch":null,"environment":"local-slice","workspace_root":workspace_root,"source_platform":"cli","channel":"cli"}),
            evidence: json!({"identity_binding_source":binding.identity_binding_source,"tool_trace_ref":null,"payload_hash":hash_canonical(operation)?}),
        };
        let (verdict, matched, reasons, refs, next) =
            if binding.binding_resolution == BindingResolution::Unresolved {
                (
                    Verdict::Escalate,
                    None,
                    vec!["identity_unresolved".into()],
                    vec!["identity-binding:unresolved".into()],
                    vec!["complete-onboarding".into()],
                )
            } else if hard_denied(operation, &self.bundle.policy_surfaces.file.deny_write) {
                (
                    Verdict::Deny,
                    None,
                    vec!["file_policy_deny".into(), "generated_zone_denied".into()],
                    vec!["file-access-policy:deny".into()],
                    vec![],
                )
            } else if let Some(rule) = self
                .bundle
                .rules
                .iter()
                .find(|r| matches_rule(r, actor, operation))
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
        Ok(AuthorityDecision {
            version: RECORD_VERSION.into(),
            decision_id: format!("auth_{sequence:02}"),
            run_id: run_id.into(),
            plan_item_id: plan_item_id.into(),
            action_id,
            correlation_id: run_id.into(),
            operation: operation.clone(),
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
                recorded_at: now.clone(),
            },
            decided_at: now,
        })
    }
}
fn matches_rule(rule: &PolicyRule, actor: &Actor, operation: &Operation) -> bool {
    if rule.actor_role != actor.role {
        return false;
    }
    match operation {
        Operation::WriteFile { path, .. } => {
            let prefix = rule.path_prefix.as_deref().unwrap_or("");
            rule.operation_kind == "write_file" && (prefix.is_empty() || path.starts_with(prefix))
        }
        Operation::ExecuteCommand { argv, .. } => {
            rule.operation_kind == "execute_command"
                && rule.argv0.as_deref().is_none_or(|expected| {
                    argv.first()
                        .and_then(|v| Path::new(v).file_name())
                        .and_then(|v| v.to_str())
                        == Some(expected)
                })
        }
    }
}
fn hard_denied(operation: &Operation, patterns: &[String]) -> bool {
    match operation {
        Operation::WriteFile { path, .. } => patterns.iter().any(|p| path_denied(path, p)),
        _ => false,
    }
}
fn path_denied(path: &str, pattern: &str) -> bool {
    path.starts_with(".git/")
        || path.starts_with(".env")
        || path.to_lowercase().contains("secret")
        || (pattern.contains("src/gen")
            && path
                .split('/')
                .collect::<Vec<_>>()
                .windows(2)
                .any(|w| w == ["src", "gen"]))
        || pattern == path
}
