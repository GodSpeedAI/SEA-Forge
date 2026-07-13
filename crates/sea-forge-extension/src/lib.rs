//! SEA Forge extension descriptor and registry contracts (spec-full §7.0b).

#![forbid(unsafe_code)]

use sea_forge_core::errors::ForgeError;
use sea_forge_core::types::{ExtensionDescriptor, ExtensionKind};
use sea_forge_ledger::{CommittedRecordRef, LedgerStream};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub use sea_forge_core::types::{ContractRef, ProjectionKind, ProjectionRef, ProjectionStatus};

// ---- Registry ----

/// Extension registry persisted at `.sea-forge/extensions/registry.json`.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtensionRegistry {
    pub version: String,
    pub updated_at: String,
    pub extensions: Vec<RegistryEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RegistryEntry {
    pub extension_id: String,
    pub version: String,
    pub descriptor_sha256: String,
    pub trust_level: TrustLevel,
    pub status: ExtensionStatus,
    #[serde(default)]
    pub authority_ref: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TrustLevel {
    BuiltIn,
    FirstParty,
    Workspace,
    Imported,
    Quarantined,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionStatus {
    Active,
    Disabled,
    Quarantined,
    Superseded,
}

pub struct AdoptionAuthorization<'a> {
    pub grant: sea_forge_authority::ActionGrant,
    pub run_id: &'a str,
    pub plan_item_id: &'a str,
    pub workspace_root: &'a Path,
    pub authority_ref: &'a CommittedRecordRef,
}

impl ExtensionRegistry {
    /// Load from `.sea-forge/extensions/registry.json`, or return an empty registry.
    pub fn load(root: &Path) -> Result<Self, ForgeError> {
        let path = Self::path(root);
        if !path.exists() {
            return Ok(Self {
                version: "0.2".into(),
                updated_at: chrono_now(),
                extensions: Vec::new(),
            });
        }
        let bytes = fs::read(&path).map_err(|e| ForgeError::io("read registry", e))?;
        serde_json::from_slice(&bytes)
            .map_err(|e| ForgeError::Serialization(format!("parse registry: {e}")))
    }

    /// Save to `.sea-forge/extensions/registry.json`.
    pub fn save(
        &self,
        root: &Path,
        stream: &LedgerStream,
        authority_refs: Vec<String>,
    ) -> Result<CommittedRecordRef, ForgeError> {
        let path = Self::path(root);
        let bytes = serde_json::to_vec_pretty(self)
            .map_err(|e| ForgeError::Serialization(e.to_string()))?;
        let committed = stream.commit_typed(
            "extension_registry",
            self.extensions
                .iter()
                .map(|entry| format!("{}@{}", entry.extension_id, entry.version))
                .collect(),
            self,
            authority_refs,
        )?;
        stream.materialize_view(&committed, &path, &bytes)?;
        Ok(committed)
    }

    fn path(root: &Path) -> PathBuf {
        root.join("extensions").join("registry.json")
    }

    /// Register a built-in extension descriptor.
    pub fn register_built_in(
        &mut self,
        descriptor: &ExtensionDescriptor,
    ) -> Result<(), ForgeError> {
        validate_descriptor(descriptor)?;
        let descriptor_hash = hash_descriptor(descriptor)?;
        // Check if already registered.
        if self
            .extensions
            .iter()
            .any(|e| e.extension_id == descriptor.extension_id && e.version == descriptor.version)
        {
            return Ok(()); // Idempotent.
        }
        self.extensions.push(RegistryEntry {
            extension_id: descriptor.extension_id.clone(),
            version: descriptor.version.clone(),
            descriptor_sha256: descriptor_hash,
            trust_level: TrustLevel::BuiltIn,
            status: ExtensionStatus::Active,
            authority_ref: None,
        });
        self.updated_at = chrono_now();
        Ok(())
    }

    /// Import an extension descriptor. Imported extensions start `disabled`.
    pub fn import(
        &mut self,
        descriptor: &ExtensionDescriptor,
        trust_level: TrustLevel,
    ) -> Result<(), ForgeError> {
        validate_descriptor(descriptor)?;
        let descriptor_hash = hash_descriptor(descriptor)?;
        self.extensions.push(RegistryEntry {
            extension_id: descriptor.extension_id.clone(),
            version: descriptor.version.clone(),
            descriptor_sha256: descriptor_hash,
            trust_level,
            status: ExtensionStatus::Disabled,
            authority_ref: None,
        });
        self.updated_at = chrono_now();
        Ok(())
    }

    /// Adopt an imported extension (status: disabled → active).
    /// This is the authority-checked operation that makes an imported extension usable.
    pub fn adopt(
        &mut self,
        extension_id: &str,
        version: &str,
        authorization: AdoptionAuthorization<'_>,
    ) -> Result<(), ForgeError> {
        authorization.grant.authorize(
            &sea_forge_core::types::AuthorityAction::Reserved {
                resource_type: "adopt_extension".into(),
                resource_id: format!("{extension_id}@{version}"),
                parameters: serde_json::json!({}),
            },
            authorization.run_id,
            authorization.plan_item_id,
            authorization.workspace_root,
        )?;
        let entry = self
            .extensions
            .iter_mut()
            .find(|e| e.extension_id == extension_id && e.version == version)
            .ok_or_else(|| ForgeError::Input("extension not found in registry".into()))?;
        if entry.status != ExtensionStatus::Disabled {
            return Err(ForgeError::Input(format!(
                "extension {extension_id}@{version} is not disabled (status: {:?})",
                entry.status
            )));
        }
        entry.status = ExtensionStatus::Active;
        entry.authority_ref = Some(authorization.authority_ref.entry_ulid().into());
        self.updated_at = chrono_now();
        Ok(())
    }

    /// Check if an extension is active.
    pub fn is_active(&self, extension_id: &str, version: &str) -> bool {
        self.extensions.iter().any(|e| {
            e.extension_id == extension_id
                && e.version == version
                && e.status == ExtensionStatus::Active
        })
    }
}

// ---- Install record ----

/// Extension install record persisted at `descriptors/<extension_id>@<version>.json`.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtensionInstallRecord {
    pub descriptor: ExtensionDescriptor,
    pub installed_by: String,
    pub installed_at: String,
    pub source_uri: String,
    pub source_sha256: String,
    pub authority_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub settlement_ref: Option<String>,
    pub compatibility: Compatibility,
    pub adopted_from_bundle_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Compatibility {
    pub min_kernel_version: String,
    pub max_kernel_version: Option<String>,
    pub required_record_versions: Vec<String>,
    pub required_authority_surfaces: Vec<String>,
}

// ---- Projection adapter ABI ----

/// Stable contract for projection adapters (spec-full §7.0b).
/// No implementations beyond a test stub; the trait must be stable before plugins exist.
pub trait ProjectionAdapter {
    fn plan(&self, inputs: &ProjectionInputs) -> Result<ProjectionPlan, ForgeError>;
    fn project(&self, plan: &ProjectionPlan) -> Result<ProjectionOutput, ForgeError>;
    fn validate(&self, output: &ProjectionOutput) -> Result<(), ForgeError>;
    fn explain(&self) -> String;
    fn rebuild_hash(&self, inputs: &ProjectionInputs) -> Result<String, ForgeError>;
}

/// Inputs to a projection adapter.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProjectionInputs {
    pub source_refs: Vec<String>,
    pub descriptor_sha256: String,
    pub input_hash: String,
    pub adapter_ref: String,
}

/// Plan for a projection run.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProjectionPlan {
    pub adapter_ref: String,
    pub steps: Vec<String>,
    pub input_hash: String,
}

/// Output of a projection run.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProjectionOutput {
    pub output_refs: Vec<OutputRef>,
    pub quarantine_refs: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OutputRef {
    pub projection_kind: String,
    pub output_uri: String,
    pub output_sha256: String,
}

/// A persisted projection record (spec-full §7.0b).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProjectionRecord {
    pub projection_id: String,
    pub projection_kind: String,
    pub adapter_ref: String,
    pub case_id: Option<String>,
    pub run_id: Option<String>,
    pub domain_model_ref: Option<String>,
    pub source_refs: Vec<String>,
    pub input_hash: String,
    pub output_refs: Vec<OutputRef>,
    pub quarantine_refs: Vec<String>,
    pub validation: ProjectionValidation,
    pub authority_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub settlement_ref: Option<String>,
    pub created_at: String,
    pub rebuild_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProjectionValidation {
    pub status: String,
    pub validator_ref: String,
    pub basis: Vec<String>,
}

// ---- Validation ----

/// Validate an extension descriptor (spec-full §7.0b, §12 M0).
pub fn validate_descriptor(descriptor: &ExtensionDescriptor) -> Result<(), ForgeError> {
    let valid_hash = |value: &str| {
        value.strip_prefix("sha256:").is_some_and(|hex| {
            hex.len() == 64
                && hex
                    .chars()
                    .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        })
    };

    let canonical_surfaces = [
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

    if !canonical_surfaces.contains(&descriptor.authority_surface.as_str()) {
        return Err(ForgeError::Input(
            "extension authority_surface is not canonical".into(),
        ));
    }
    if !valid_hash(&descriptor.input_contract.sha256)
        || !valid_hash(&descriptor.output_contract.sha256)
    {
        return Err(ForgeError::Input(
            "extension contract hash is invalid".into(),
        ));
    }
    if descriptor.kind == ExtensionKind::ProjectionAdapter && !descriptor.deterministic {
        return Err(ForgeError::Input(
            "projection adapters must be deterministic".into(),
        ));
    }
    Ok(())
}

/// Compute SHA-256 of a descriptor's canonical serialization.
fn hash_descriptor(descriptor: &ExtensionDescriptor) -> Result<String, ForgeError> {
    let bytes =
        serde_json::to_vec(descriptor).map_err(|e| ForgeError::Serialization(e.to_string()))?;
    Ok(format!("sha256:{:x}", sha2::Sha256::digest(&bytes)))
}

fn chrono_now() -> String {
    // Avoid pulling chrono directly; use a simple approach.
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| format!("unix:{}", d.as_secs()))
        .unwrap_or_else(|_| "unknown".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
    use sea_forge_core::types::{
        Actor, ActorRole, ActorType, AuthorityAction, BindingResolution, IdentityBinding,
    };
    use sea_forge_core::types::{ContractRef, ExtensionKind};

    fn valid_descriptor() -> ExtensionDescriptor {
        ExtensionDescriptor {
            extension_id: "ext_demo".into(),
            kind: ExtensionKind::ProjectionAdapter,
            name: "demo-projection".into(),
            version: "0.1".into(),
            provider: "built-in".into(),
            capabilities: vec!["spec_projection".into()],
            authority_surface: "spec_projection".into(),
            input_contract: ContractRef {
                schema: "schema/input.json".into(),
                sha256: format!("sha256:{}", "a".repeat(64)),
            },
            output_contract: ContractRef {
                schema: "schema/output.json".into(),
                sha256: format!("sha256:{}", "b".repeat(64)),
            },
            deterministic: true,
            installed_at: None,
        }
    }

    #[test]
    fn validate_built_in_descriptor() {
        assert!(validate_descriptor(&valid_descriptor()).is_ok());
    }

    #[test]
    fn reject_non_deterministic_projection_adapter() {
        let mut d = valid_descriptor();
        d.deterministic = false;
        assert!(validate_descriptor(&d).is_err());
    }

    #[test]
    fn reject_non_canonical_authority_surface() {
        let mut d = valid_descriptor();
        d.authority_surface = "unknown_surface".into();
        assert!(validate_descriptor(&d).is_err());
    }

    #[test]
    fn empty_registry_validates() {
        let tmp = tempfile::tempdir().unwrap();
        let reg = ExtensionRegistry::load(tmp.path()).unwrap();
        assert!(reg.extensions.is_empty());
    }

    #[test]
    fn register_built_in_makes_active() {
        let mut reg = ExtensionRegistry {
            version: "0.2".into(),
            updated_at: "now".into(),
            extensions: Vec::new(),
        };
        reg.register_built_in(&valid_descriptor()).unwrap();
        assert!(reg.is_active("ext_demo", "0.1"));
    }

    #[test]
    fn imported_extension_starts_disabled() {
        let mut reg = ExtensionRegistry {
            version: "0.2".into(),
            updated_at: "now".into(),
            extensions: Vec::new(),
        };
        reg.import(&valid_descriptor(), TrustLevel::Imported)
            .unwrap();
        assert!(!reg.is_active("ext_demo", "0.1"));
    }

    #[test]
    fn adopt_makes_imported_active() {
        let tmp = tempfile::tempdir().unwrap();
        let stream = LedgerStream::open(tmp.path(), "extensions", "test").unwrap();
        let engine = PolicyAuthorityEngine::new(
            serde_yaml::from_str::<AuthorityPolicyBundle>(
                "version: \"0.2\"\nrules:\n  - name: adopt\n    verdict: allow\n    actor_role: operator\n    operation_kind: adopt_extension\n",
            )
            .unwrap(),
        )
        .unwrap();
        let actor = Actor {
            actor_id: "operator".into(),
            role: ActorRole::Operator,
        };
        let action = AuthorityAction::Reserved {
            resource_type: "adopt_extension".into(),
            resource_id: "ext_demo@0.1".into(),
            parameters: serde_json::json!({}),
        };
        let decision = engine
            .evaluate(AuthorityEvaluation {
                actor: &actor,
                binding: IdentityBinding {
                    principal: "operator".into(),
                    actor_type: ActorType::Human,
                    binding_resolution: BindingResolution::Exact,
                    identity_binding_source: "test".into(),
                    sponsor: None,
                },
                run_id: "run_test",
                plan_item_id: "item_test",
                sequence: 1,
                action: &action,
                workspace_root: tmp.path(),
            })
            .unwrap();
        let authority_ref = stream
            .commit_typed("authority_decision", vec![], &decision, vec![])
            .unwrap();
        let grant = engine
            .grant(&decision, &authority_ref, &action, tmp.path())
            .unwrap();
        let mut reg = ExtensionRegistry {
            version: "0.2".into(),
            updated_at: "now".into(),
            extensions: Vec::new(),
        };
        reg.import(&valid_descriptor(), TrustLevel::Imported)
            .unwrap();
        assert!(!reg.is_active("ext_demo", "0.1"));
        reg.adopt(
            "ext_demo",
            "0.1",
            AdoptionAuthorization {
                grant,
                run_id: "run_test",
                plan_item_id: "item_test",
                workspace_root: tmp.path(),
                authority_ref: &authority_ref,
            },
        )
        .unwrap();
        assert!(reg.is_active("ext_demo", "0.1"));
        assert_eq!(
            reg.extensions[0].authority_ref.as_deref(),
            Some(authority_ref.entry_ulid())
        );
    }

    #[test]
    fn authority_surface_maps_to_policy_surface() {
        let d = valid_descriptor();
        let policy_surfaces = [
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
        assert!(
            policy_surfaces.contains(&d.authority_surface.as_str()),
            "authority_surface must map to a policy surface"
        );
    }
}
