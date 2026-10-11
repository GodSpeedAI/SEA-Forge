//! SEA Forge extension descriptor and registry contracts (spec-full §7.0b).

#![forbid(unsafe_code)]

pub mod cep0008;

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

/// Extension registry persisted at `<root>/extensions/registry.json`, where
/// `root` is the state root (F-12).
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

pub struct ExtensionAuthorization<'a> {
    pub grant: sea_forge_authority::ActionGrant,
    pub run_id: &'a str,
    pub plan_item_id: &'a str,
    pub workspace_root: &'a Path,
    pub authority_ref: &'a CommittedRecordRef,
}

impl ExtensionRegistry {
    /// Load from `<root>/extensions/registry.json`, or return an empty registry.
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
        let registry: Self = serde_json::from_slice(&bytes)
            .map_err(|e| ForgeError::Serialization(format!("parse registry: {e}")))?;
        registry.validate_no_duplicates()?;
        Ok(registry)
    }

    /// Load the registry and prove the file is what the ledger committed
    /// (SUP-08): the bytes' payload hash must equal the newest
    /// `extension_registry` record in `stream`. A rewritten or rolled-back
    /// `registry.json` — e.g. `quarantined` flipped back to `active`, or a
    /// stripped trust level — is refused instead of silently honored. The
    /// ledger, not raw disk, is the read authority.
    ///
    /// An absent registry file is still a fresh installation (no record can
    /// exist for it either); verification applies only to present files.
    pub fn load_verified(root: &Path, stream: &LedgerStream) -> Result<Self, ForgeError> {
        let registry = Self::load(root)?;
        if !Self::path(root).exists() {
            return Ok(registry);
        }
        let last_committed = stream
            .read_entries()?
            .into_iter()
            .rfind(|entry| entry.record_kind == "extension_registry");
        let Some(entry) = last_committed else {
            return Err(ForgeError::Input(
                "extension registry exists but no `extension_registry` ledger record ever \
                 committed it; refusing to trust unattested registry bytes"
                    .into(),
            ));
        };
        let value = serde_json::to_value(&registry)
            .map_err(|e| ForgeError::Serialization(e.to_string()))?;
        let actual = sea_forge_ledger::types::payload_hash(&value)?;
        if entry.payload_hash != actual {
            return Err(ForgeError::Input(format!(
                "extension registry does not match its latest ledger record \
                 (declared {}, committed {}); refusing to honor modified registry bytes",
                entry.payload_hash, actual
            )));
        }
        Ok(registry)
    }

    /// SUP-08: two entries sharing `(id, version)` make "which descriptor is
    /// installed" ambiguous; reject at the boundary rather than first-match.
    fn validate_no_duplicates(&self) -> Result<(), ForgeError> {
        let mut seen = std::collections::BTreeSet::new();
        for entry in &self.extensions {
            if !seen.insert(format!("{}@{}", entry.extension_id, entry.version)) {
                return Err(ForgeError::Input(format!(
                    "registry holds duplicate entries for {}@{}",
                    entry.extension_id, entry.version
                )));
            }
        }
        Ok(())
    }

    /// Save to `<root>/extensions/registry.json`.
    pub fn save(
        &self,
        root: &Path,
        stream: &LedgerStream,
        authority_ref: &CommittedRecordRef,
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
            vec![authority_ref.entry_ulid().into()],
        )?;
        stream.materialize_view(&committed, &path, &bytes)?;
        Ok(committed)
    }

    fn path(root: &Path) -> PathBuf {
        root.join("extensions").join("registry.json")
    }

    /// Whether a registry file exists under this state root.
    ///
    /// SUP-07: callers must be able to distinguish "registry absent" (the
    /// cell's extension state has never been attested) from "registry
    /// present and empty" — collapsing both into zero extensions fabricated
    /// a coherent-looking cell that nothing had ever attested.
    pub fn exists(root: &Path) -> bool {
        Self::path(root).exists()
    }

    /// Register a built-in extension descriptor.
    pub fn register_built_in(
        &mut self,
        descriptor: &ExtensionDescriptor,
    ) -> Result<(), ForgeError> {
        validate_descriptor(descriptor)?;
        let descriptor_hash = hash_descriptor(descriptor)?;
        // SUP-09i: idempotent only when the descriptor is byte-identical —
        // same `(id, version)` with a different hash is a build bug (a
        // built-in changed under a pinned version) and must fail loudly, not
        // silently keep the old implementation.
        if let Some(existing) = self
            .extensions
            .iter()
            .find(|e| e.extension_id == descriptor.extension_id && e.version == descriptor.version)
        {
            if existing.descriptor_sha256 == descriptor_hash {
                return Ok(()); // Idempotent re-registration.
            }
            return Err(ForgeError::Input(format!(
                "built-in {}@{} changed content without a version bump; \
                 this is a compile-time pinning bug",
                descriptor.extension_id, descriptor.version
            )));
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

    /// Register an immutable runtime adapter. A descriptor change must use a
    /// new version; silently repointing an installed endpoint (same version,
    /// different descriptor hash) is forbidden. A versioned change replaces
    /// the prior entry in place.
    pub fn register_immutable_runtime_adapter(
        &mut self,
        descriptor: &ExtensionDescriptor,
    ) -> Result<(), ForgeError> {
        if descriptor.kind != ExtensionKind::RuntimeAdapter {
            return Err(ForgeError::Input(
                "agent endpoint descriptor must be a runtime_adapter".into(),
            ));
        }
        validate_descriptor(descriptor)?;
        let hash = hash_descriptor(descriptor)?;
        if let Some(existing_idx) = self
            .extensions
            .iter()
            .position(|entry| entry.extension_id == descriptor.extension_id)
        {
            let existing = &self.extensions[existing_idx];
            if existing.version == descriptor.version {
                if existing.descriptor_sha256 == hash {
                    return Ok(()); // Idempotent re-registration.
                }
                return Err(ForgeError::Input(
                    "registered runtime adapter is immutable for a given version; bump the version to change the descriptor".into(),
                ));
            }
            // SUP-08: this path has no authority check of its own — a
            // quarantined entry must never be resurrected here as
            // FirstParty+Active. Resolution runs through the authority-checked
            // `adopt`/`import_authorized` paths instead.
            if existing.status == ExtensionStatus::Quarantined {
                return Err(ForgeError::Input(format!(
                    "{}@{} is quarantined and cannot be replaced by registration; \
                     resolve the quarantine through the authorized adoption path",
                    descriptor.extension_id, descriptor.version
                )));
            }
            // Versioned change: replace in place.
            self.extensions[existing_idx] = RegistryEntry {
                extension_id: descriptor.extension_id.clone(),
                version: descriptor.version.clone(),
                descriptor_sha256: hash,
                trust_level: TrustLevel::FirstParty,
                status: ExtensionStatus::Active,
                authority_ref: None,
            };
            self.updated_at = chrono_now();
            return Ok(());
        }
        self.extensions.push(RegistryEntry {
            extension_id: descriptor.extension_id.clone(),
            version: descriptor.version.clone(),
            descriptor_sha256: hash,
            trust_level: TrustLevel::FirstParty,
            status: ExtensionStatus::Active,
            authority_ref: None,
        });
        self.updated_at = chrono_now();
        Ok(())
    }

    /// Import an extension descriptor. Imported extensions start `disabled`.
    fn import(
        &mut self,
        descriptor: &ExtensionDescriptor,
        trust_level: TrustLevel,
    ) -> Result<(), ForgeError> {
        validate_descriptor(descriptor)?;
        let descriptor_hash = hash_descriptor(descriptor)?;
        // SUP-09i: imports must not create duplicate `(id, version)` entries —
        // since the duplicate guard at load, a second import of the same
        // descriptor would permanently brick `registry.json` for every later
        // reader. Identical content is idempotent; differing content and any
        // quarantined/superseded target are refused.
        if let Some(existing) = self
            .extensions
            .iter()
            .find(|e| e.extension_id == descriptor.extension_id && e.version == descriptor.version)
        {
            if existing.descriptor_sha256 == descriptor_hash
                && matches!(
                    existing.status,
                    ExtensionStatus::Active | ExtensionStatus::Disabled
                )
            {
                return Ok(());
            }
            return Err(ForgeError::Input(format!(
                "cannot import {}@{}: an entry already exists (status {:?}) with a                  different hash or terminal standing",
                descriptor.extension_id, descriptor.version, existing.status
            )));
        }
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

    pub fn import_authorized(
        &mut self,
        descriptor: &ExtensionDescriptor,
        trust_level: TrustLevel,
        authorization: ExtensionAuthorization<'_>,
    ) -> Result<(), ForgeError> {
        authorization.grant.authorize(
            &sea_forge_core::types::AuthorityAction::Reserved {
                resource_type: "install_extension".into(),
                resource_id: format!("{}@{}", descriptor.extension_id, descriptor.version),
                parameters: serde_json::json!({}),
            },
            authorization.run_id,
            authorization.plan_item_id,
            authorization.workspace_root,
        )?;
        self.import(descriptor, trust_level)?;
        let entry = self
            .extensions
            .last_mut()
            .ok_or_else(|| ForgeError::Internal("import did not create registry entry".into()))?;
        entry.authority_ref = Some(authorization.authority_ref.entry_ulid().into());
        Ok(())
    }

    /// Adopt an imported extension (status: disabled → active).
    /// This is the authority-checked operation that makes an imported extension usable.
    pub fn adopt(
        &mut self,
        extension_id: &str,
        version: &str,
        authorization: ExtensionAuthorization<'_>,
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
    // SUP-09i: identity grammar. `extension_id` joins registry keys and
    // filesystem-adjacent projections, so it uses the shared id-segment
    // grammar; versions get their own predicate (they legitimately carry
    // dots, which the path-safety grammar forbids).
    if !sea_forge_core::path::valid_id_segment(&descriptor.extension_id, 128) {
        return Err(ForgeError::Input(
            "extension_id must be a safe id segment ([A-Za-z0-9_-], <=128)".into(),
        ));
    }
    let valid_version = |version: &str| {
        version.len() <= 64
            && version.starts_with(|c: char| c.is_ascii_alphanumeric())
            && version
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
    };
    if !valid_version(&descriptor.version) {
        return Err(ForgeError::Input(
            "extension version must be alphanumeric with . _ - separators (<=64)".into(),
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
    use sea_forge_core::types::{Actor, ActorRole, AuthorityAction};
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
        let bundle = serde_yaml::from_str::<AuthorityPolicyBundle>(
                "version: \"0.1\"\nrules:\n  - name: adopt\n    verdict: allow\n    actor_role: operator\n    operation_kind: adopt_extension\n",
            )
            .unwrap();
        let binding = bundle.resolve_identity("operator", ActorRole::Operator);
        let engine = PolicyAuthorityEngine::new(bundle).unwrap();
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
                binding,
                run_id: "run_test",
                case_id: "case_test",
                plan_item_id: "item_test",
                sequence: 1,
                action: &action,
                workspace_root: tmp.path(),
                evidence_refs: vec![],
                artifacts_root: None,
                timeout_secs: None,
                env_keys: Default::default(),
                domainforge_candidate: None,
                environment: None,
            })
            .unwrap();
        let authority_ref = stream
            .commit_typed("authority_decision", vec![], &decision, vec![])
            .unwrap();
        let grant = engine
            .grant(&decision, &authority_ref, &action, None)
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
            ExtensionAuthorization {
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

    fn runtime_adapter(id: &str, version: &str, sha_hex: &str) -> ExtensionDescriptor {
        let input_sha = format!("sha256:{sha_hex}");
        ExtensionDescriptor {
            extension_id: id.into(),
            kind: ExtensionKind::RuntimeAdapter,
            name: format!("{id} adapter"),
            version: version.into(),
            provider: "sea-forge-agent".into(),
            capabilities: vec!["agent_probe".into()],
            authority_surface: "external_api".into(),
            input_contract: ContractRef {
                schema: "sea-forge-agent.endpoint.v1".into(),
                sha256: input_sha,
            },
            output_contract: ContractRef {
                schema: "sea-forge-agent.probe.v1".into(),
                sha256: format!("sha256:{}", "b".repeat(64)),
            },
            deterministic: false,
            installed_at: None,
        }
    }

    #[test]
    fn immutable_runtime_adapter_is_idempotent_for_same_version_and_hash() {
        let mut registry = ExtensionRegistry {
            version: "0.2".into(),
            updated_at: "now".into(),
            extensions: Vec::new(),
        };
        let descriptor = runtime_adapter("agent_endpoint_local", "0.1.0", &"a".repeat(64));
        registry
            .register_immutable_runtime_adapter(&descriptor)
            .unwrap();
        // Re-registering the identical descriptor is a no-op.
        registry
            .register_immutable_runtime_adapter(&descriptor)
            .unwrap();
        assert_eq!(
            registry
                .extensions
                .iter()
                .filter(|e| e.extension_id == "agent_endpoint_local")
                .count(),
            1
        );
    }

    #[test]
    fn immutable_runtime_adapter_rejects_same_version_hash_change() {
        let mut registry = ExtensionRegistry {
            version: "0.2".into(),
            updated_at: "now".into(),
            extensions: Vec::new(),
        };
        registry
            .register_immutable_runtime_adapter(&runtime_adapter(
                "agent_endpoint_local",
                "0.1.0",
                &"a".repeat(64),
            ))
            .unwrap();
        let err = registry
            .register_immutable_runtime_adapter(&runtime_adapter(
                "agent_endpoint_local",
                "0.1.0",
                &"b".repeat(64),
            ))
            .unwrap_err();
        assert!(
            matches!(err, ForgeError::Input(ref m) if m.contains("immutable for a given version"))
        );
    }

    #[test]
    fn immutable_runtime_adapter_allows_versioned_upgrade_in_place() {
        let mut registry = ExtensionRegistry {
            version: "0.2".into(),
            updated_at: "now".into(),
            extensions: Vec::new(),
        };
        registry
            .register_immutable_runtime_adapter(&runtime_adapter(
                "agent_endpoint_local",
                "0.1.0",
                &"a".repeat(64),
            ))
            .unwrap();
        let first_hash = registry
            .extensions
            .iter()
            .find(|e| e.extension_id == "agent_endpoint_local")
            .unwrap()
            .descriptor_sha256
            .clone();
        registry
            .register_immutable_runtime_adapter(&runtime_adapter(
                "agent_endpoint_local",
                "0.2.0",
                &"b".repeat(64),
            ))
            .unwrap();
        let entries: Vec<_> = registry
            .extensions
            .iter()
            .filter(|e| e.extension_id == "agent_endpoint_local")
            .collect();
        assert_eq!(
            entries.len(),
            1,
            "versioned upgrade replaces, not duplicates"
        );
        assert_eq!(entries[0].version, "0.2.0");
        assert_ne!(
            entries[0].descriptor_sha256, first_hash,
            "versioned upgrade must rebind the descriptor hash"
        );
    }

    #[test]
    fn immutable_runtime_adapter_rejects_non_runtime_kind() {
        let mut registry = ExtensionRegistry {
            version: "0.2".into(),
            updated_at: "now".into(),
            extensions: Vec::new(),
        };
        let mut descriptor = runtime_adapter("agent_endpoint_local", "0.1.0", &"a".repeat(64));
        descriptor.kind = ExtensionKind::ProjectionAdapter;
        let err = registry
            .register_immutable_runtime_adapter(&descriptor)
            .unwrap_err();
        assert!(matches!(err, ForgeError::Input(ref m) if m.contains("runtime_adapter")));
    }

    // SUP-08: a rewritten registry.json is refused when it disagrees with its
    // latest `extension_registry` ledger record — raw-disk trust on load was
    // the defect (quarantined→active rollbacks were silently honored).
    #[test]
    fn load_verified_refuses_registry_bytes_the_ledger_never_committed() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let stream = LedgerStream::open(root, "extension-test", "tester").unwrap();

        let mut reg = ExtensionRegistry {
            version: "0.2".into(),
            updated_at: "now".into(),
            extensions: Vec::new(),
        };
        reg.register_immutable_runtime_adapter(&runtime_adapter(
            "agent_endpoint_a",
            "0.1",
            &"aa".repeat(32),
        ))
        .unwrap();
        let authority = stream
            .commit_typed("authority_decision", vec![], &serde_json::json!({}), vec![])
            .unwrap();
        reg.save(root, &stream, &authority).unwrap();
        ExtensionRegistry::load_verified(root, &stream).unwrap(); // honest state passes

        // Tamper: flip the entry to Active after a quarantine-style rewrite.
        reg.extensions[0].status = ExtensionStatus::Quarantined;
        reg.save(root, &stream, &authority).unwrap();
        // Now forge the file back to Active without committing.
        reg.extensions[0].status = ExtensionStatus::Active;
        let path = ExtensionRegistry::path(root);
        fs::write(&path, serde_json::to_vec_pretty(&reg).unwrap()).unwrap();
        let err = ExtensionRegistry::load_verified(root, &stream)
            .expect_err("forged registry bytes must not pass ledger verification");
        assert!(
            err.to_string()
                .contains("does not match its latest ledger record"),
            "{err}"
        );

        // A registry file with NO ledger record at all is refused outright.
        let tmp2 = tempfile::tempdir().unwrap();
        let stream2 = LedgerStream::open(tmp2.path(), "extension-test2", "tester").unwrap();
        let root2 = tmp2.path().join("state");
        fs::create_dir_all(root2.join("extensions")).unwrap();
        fs::write(
            root2.join("extensions").join("registry.json"),
            br#"{"version":"0.2","updated_at":"now","extensions":[]}"#,
        )
        .unwrap();
        let err = ExtensionRegistry::load_verified(&root2, &stream2)
            .expect_err("an unattested registry must be refused");
        assert!(
            err.to_string().contains("unattested registry bytes"),
            "{err}"
        );
    }

    // SUP-08: registration can never resurrect a quarantined runtime adapter
    // as FirstParty+Active — that replace path has no authority check of its
    // own; resolution belongs to the authorized adoption paths.
    #[test]
    fn quarantined_runtime_adapter_cannot_be_replaced_by_registration() {
        let mut reg = ExtensionRegistry {
            version: "0.2".into(),
            updated_at: "now".into(),
            extensions: Vec::new(),
        };
        reg.register_immutable_runtime_adapter(&runtime_adapter(
            "agent_endpoint_a",
            "0.1",
            &"aa".repeat(32),
        ))
        .unwrap();
        reg.extensions[0].status = ExtensionStatus::Quarantined;

        // Same version + same bytes: the idempotent no-op must NOT resurrect
        // the entry either.
        reg.register_immutable_runtime_adapter(&runtime_adapter(
            "agent_endpoint_a",
            "0.1",
            &"aa".repeat(32),
        ))
        .unwrap();
        assert_eq!(
            reg.extensions[0].status,
            ExtensionStatus::Quarantined,
            "an idempotent re-registration must never clear a quarantine"
        );

        // A versioned change over a quarantined entry would have replaced it
        // wholesale with FirstParty+Active; it is refused outright now.
        let err = reg
            .register_immutable_runtime_adapter(&runtime_adapter(
                "agent_endpoint_a",
                "0.2",
                &"bb".repeat(32),
            ))
            .unwrap_err();
        assert!(
            err.to_string()
                .contains("is quarantined and cannot be replaced"),
            "{err}"
        );
        assert_eq!(reg.extensions[0].status, ExtensionStatus::Quarantined);
    }
    // ── SUP-09i: identity grammar + registration/import invariants ──

    #[test]
    fn validate_descriptor_rejects_unsafe_extension_id() {
        let mut d = runtime_adapter("../escape", "0.1", &"aa".repeat(32));
        let error = validate_descriptor(&d).expect_err("traversal id");
        assert!(error.to_string().contains("safe id segment"), "{error}");
        d.extension_id = "".into();
        let error = validate_descriptor(&d).expect_err("empty id");
        assert!(error.to_string().contains("safe id segment"), "{error}");
    }

    #[test]
    fn validate_descriptor_rejects_unsafe_version() {
        for bad in ["", "../1", "v 1", "0.1 x"] {
            let d = runtime_adapter("ext_demo", bad, &"aa".repeat(32));
            let error = validate_descriptor(&d)
                .err()
                .unwrap_or_else(|| panic!("version {bad:?}"));
            assert!(
                error.to_string().contains("version must be alphanumeric"),
                "{bad}: {error}"
            );
        }
        // Dots and underscores are legitimate version spellings.
        assert!(
            validate_descriptor(&runtime_adapter("ext_demo", "0.2_rc-1", &"aa".repeat(32))).is_ok()
        );
    }

    #[test]
    fn register_built_in_hash_mismatch_is_error_not_silent_idempotence() {
        let mut reg = ExtensionRegistry {
            version: "0.2".into(),
            updated_at: "now".into(),
            extensions: Vec::new(),
        };
        reg.register_built_in(&runtime_adapter("ext_builtin", "1.0", &"aa".repeat(32)))
            .unwrap();
        // Same id/version, different content → hard error, not silent success.
        let error = reg
            .register_built_in(&runtime_adapter("ext_builtin", "1.0", &"bb".repeat(32)))
            .expect_err("content change without a version bump must fail");
        assert!(
            error
                .to_string()
                .contains("changed content without a version bump"),
            "{error}"
        );
        // Identical re-registration stays idempotent.
        reg.register_built_in(&runtime_adapter("ext_builtin", "1.0", &"aa".repeat(32)))
            .unwrap();
    }

    #[test]
    fn import_same_id_version_twice_is_rejected_and_never_duplicates() {
        let mut reg = ExtensionRegistry {
            version: "0.2".into(),
            updated_at: "now".into(),
            extensions: Vec::new(),
        };
        reg.import(
            &runtime_adapter("ext_imp", "0.3", &"cc".repeat(32)),
            TrustLevel::FirstParty,
        )
        .unwrap();
        let error = reg
            .import(
                &runtime_adapter("ext_imp", "0.3", &"dd".repeat(32)),
                TrustLevel::FirstParty,
            )
            .expect_err("a second import with different content must be refused");
        assert!(error.to_string().contains("cannot import"), "{error}");
        // Idempotent identical import is allowed and adds no entry.
        reg.import(
            &runtime_adapter("ext_imp", "0.3", &"cc".repeat(32)),
            TrustLevel::FirstParty,
        )
        .unwrap();
        assert_eq!(reg.extensions.len(), 1);
    }
}
