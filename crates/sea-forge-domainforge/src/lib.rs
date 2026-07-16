//! SEA Forge first-party DomainForge semantic adapter (spec-full §7.0a).
//!
//! Public contract (synchronous, side-effect-free):
//!   load_validate(SeaSourceSet) -> DomainModel
//!   evaluate(DomainModel, CanonicalActionRequest, TrustedFacts) -> GovernanceVerdict
//!   project(DomainModel, ProjectionRequest) -> SortedMap<RelativePath, Bytes>

#![forbid(unsafe_code)]

use domainforge_core::parser::{parse_to_graph_with_options, ParseOptions};
use domainforge_core::policy::Severity;
use sea_forge_core::errors::ForgeError;
use sea_forge_core::types::ProjectionKind;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// A source file in a `SeaSourceSet`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceFile {
    pub uri: String,
    pub sha256: String,
    pub content: String,
}

/// One entry `.sea` source and any namespace-registry or imported `.sea` files.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SeaSourceSet {
    pub entry_uri: String,
    pub files: Vec<SourceFile>,
}

/// In-memory derived view of a validated semantic model (not persisted truth).
#[derive(Clone, Debug)]
pub struct DomainModel {
    pub graph: domainforge_core::graph::Graph,
    pub model_ref: DomainModelRef,
}

/// Stable reference to a validated semantic input.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DomainModelRef {
    pub source_refs: Vec<SourceRef>,
    pub domainforge_version: String,
    pub adapter_descriptor_sha256: String,
    pub parse_options_sha256: String,
    pub semantic_model_sha256: String,
    pub concept_refs: Vec<String>,
    #[serde(default)]
    pub class_refs: Vec<String>,
    pub validation_evidence_refs: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceRef {
    pub uri: String,
    pub sha256: String,
}

/// Normalized candidate disposition from a DomainForge authority result.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CandidateDisposition {
    Allow,
    Deny,
    Escalate,
}

/// DomainForge authority verdict trace preserved as evidence.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainForgeTrace {
    pub raw_decision: String,
    pub normalized_disposition: CandidateDisposition,
    pub reason: String,
    pub evidence_refs: Vec<String>,
}

/// Descriptor hash for this adapter (fixed for a given adapter version).
pub const ADAPTER_DESCRIPTOR_SHA256: &str =
    "sha256:0000000000000000000000000000000000000000000000000000000000000001";

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn sha256_content(content: &str) -> String {
    sha256_hex(content.as_bytes())
}

fn sha256_json(value: &Value) -> Result<String, ForgeError> {
    let bytes = serde_json::to_vec(value).map_err(|e| ForgeError::Serialization(e.to_string()))?;
    Ok(sha256_hex(&bytes))
}

/// Load and validate a `SeaSourceSet` through `domainforge-core`.
///
/// Returns a `DomainModel` on success, or `domain_model_error` on any parse,
/// validation, or source-hash failure. No side effects.
pub fn load_validate(source_set: &SeaSourceSet) -> Result<DomainModel, ForgeError> {
    // Find the entry source.
    let entry = source_set
        .files
        .iter()
        .find(|f| f.uri == source_set.entry_uri)
        .ok_or_else(|| ForgeError::Input("entry source not found in source set".into()))?;

    // Verify source hash matches declared hash.
    let computed_hash = sha256_content(&entry.content);
    if computed_hash != entry.sha256 {
        return Err(ForgeError::Internal(format!(
            "domain_model_error: source hash drift for {entry_uri}: declared={declared} computed={computed_hash}",
            entry_uri = entry.uri,
            declared = entry.sha256,
        )));
    }

    // Parse through domainforge-core.
    let options = ParseOptions::default();
    let graph = parse_to_graph_with_options(&entry.content, &options)
        .map_err(|e| ForgeError::Internal(format!("domain_model_error: parse failed: {e}")))?;

    // Validate the graph.
    let validation = graph.validate();
    if validation.error_count > 0 {
        let errors: Vec<String> = validation
            .violations
            .iter()
            .filter(|v| v.severity == Severity::Error)
            .map(|v| format!("{}: {}", v.policy_name, v.message))
            .collect();
        return Err(ForgeError::Internal(format!(
            "domain_model_error: semantic validation failed: {}",
            errors.join("; ")
        )));
    }

    // Build source_refs (sorted).
    let mut source_refs: Vec<SourceRef> = source_set
        .files
        .iter()
        .map(|f| SourceRef {
            uri: f.uri.clone(),
            sha256: f.sha256.clone(),
        })
        .collect();
    source_refs.sort();

    // Build parse_options_sha256.
    let parse_options_value = serde_json::json!({
        "namespace_registry": null,
        "entry_path": source_set.entry_uri,
    });
    let parse_options_sha256 = sha256_json(&parse_options_value)?;

    // Build semantic_model_sha256 over the canonical 4-tuple.
    let canonical_tuple = serde_json::json!({
        "domainforge_version": domainforge_core::VERSION,
        "adapter_descriptor_sha256": ADAPTER_DESCRIPTOR_SHA256,
        "parse_options_sha256": parse_options_sha256,
        "source_refs": source_refs,
    });
    let semantic_model_sha256 = sha256_json(&canonical_tuple)?;

    // Extract concept refs from the graph (sorted).
    let mut concept_refs: Vec<String> = graph
        .all_entities()
        .iter()
        .map(|e| e.name().to_string())
        .collect();
    concept_refs.extend(graph.all_resources().iter().map(|r| r.name().to_string()));
    concept_refs.sort();
    concept_refs.dedup();
    let mut class_refs: Vec<String> = graph
        .all_resources()
        .iter()
        .map(|resource| resource.name().to_string())
        .collect();
    class_refs.sort();
    class_refs.dedup();

    let model_ref = DomainModelRef {
        source_refs,
        domainforge_version: domainforge_core::VERSION.to_string(),
        adapter_descriptor_sha256: ADAPTER_DESCRIPTOR_SHA256.into(),
        parse_options_sha256,
        semantic_model_sha256,
        concept_refs,
        class_refs,
        validation_evidence_refs: vec![format!(
            "validation:error_count={}",
            validation.error_count
        )],
    };

    Ok(DomainModel { graph, model_ref })
}

/// Normalize a DomainForge authority decision to a SEA Forge candidate disposition.
///
/// | DomainForge result | SEA Forge candidate disposition |
/// |---|---|
/// | `Reject` or `Deny` | `deny` |
/// | `Escalate` | `escalate` |
/// | `Allow` | `allow` |
/// | `NotApplicable` | no candidate; `deny` if policy requires DomainForge |
pub fn normalize_authority(raw_decision: &str) -> CandidateDisposition {
    match raw_decision {
        "Allow" => CandidateDisposition::Allow,
        "Reject" | "Deny" => CandidateDisposition::Deny,
        "Escalate" => CandidateDisposition::Escalate,
        _ => CandidateDisposition::Deny, // NotApplicable or unknown → deny-if-required
    }
}

pub fn evaluate_authority(
    model: &DomainModel,
    operation_kind: &str,
    resource_id: &str,
    evidence_refs: Vec<String>,
) -> Result<DomainForgeTrace, ForgeError> {
    if evidence_refs.is_empty() || model.model_ref.validation_evidence_refs.is_empty() {
        return Err(ForgeError::Input(
            "DomainForge authority evaluation requires validation evidence".into(),
        ));
    }
    let semantic_target = std::path::Path::new(resource_id)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .is_some_and(|target| {
            model
                .graph
                .all_entities()
                .iter()
                .any(|entity| entity.name().eq_ignore_ascii_case(target))
                || model
                    .graph
                    .all_resources()
                    .iter()
                    .any(|resource| resource.name().eq_ignore_ascii_case(target))
        });
    let raw_decision = match (operation_kind, semantic_target) {
        ("write_file", true) => "Allow",
        ("write_file", false) => "Reject",
        ("execute_command", _) => "NotApplicable",
        _ => "NotApplicable",
    };
    Ok(DomainForgeTrace {
        raw_decision: raw_decision.into(),
        normalized_disposition: normalize_authority(raw_decision),
        reason: "DomainForge evaluated validated model against canonical action".into(),
        evidence_refs,
    })
}

/// In-memory projection of a DomainModel into CALM or RDF.
///
/// Pure: no filesystem writes, no network calls, no CLI invocations.
/// Returns a sorted map of relative_path → content (§7.0a, §10.4a).
pub fn project(
    model: &DomainModel,
    kind: &ProjectionKind,
) -> Result<BTreeMap<String, String>, ForgeError> {
    match kind {
        ProjectionKind::Calm => {
            let mut calm_value = domainforge_core::calm::export(&model.graph).map_err(|e| {
                ForgeError::Internal(format!("domain_model_error: CALM projection failed: {e}"))
            })?;
            // Strip non-deterministic timestamp for byte-identical regeneration (§10.7).
            // domainforge-core adds sea:timestamp to metadata; remove it so the same
            // graph produces identical bytes across calls.
            if let Some(metadata) = calm_value
                .get_mut("metadata")
                .and_then(|m| m.as_object_mut())
            {
                metadata.remove("sea:timestamp");
            }
            let mut map = BTreeMap::new();
            let encoded = serde_json::to_vec_pretty(&calm_value)
                .map_err(|e| ForgeError::Serialization(e.to_string()))?;
            map.insert(
                "calm.json".into(),
                String::from_utf8_lossy(&encoded).into_owned(),
            );
            Ok(map)
        }
        ProjectionKind::Rdf => {
            let kg =
                domainforge_core::kg::KnowledgeGraph::from_graph(&model.graph).map_err(|e| {
                    ForgeError::Internal(format!("domain_model_error: KG build failed: {e}"))
                })?;
            let turtle = kg.to_turtle();
            let rdf_xml = kg.to_rdf_xml();
            let mut map = BTreeMap::new();
            map.insert("model.ttl".into(), turtle);
            map.insert("model.rdf".into(), rdf_xml);
            Ok(map)
        }
        _ => Err(ForgeError::Input(format!(
            "unsupported projection kind for DomainForge adapter: {kind:?}"
        ))),
    }
}
