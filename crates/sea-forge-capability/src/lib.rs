use sea_forge_core::{
    errors::ForgeError,
    types::{ExtensionDescriptor, ExtensionKind, SemanticEnvelope, SettlementStatus},
};

pub mod promotion;

pub use promotion::{
    append_capability_record, build_capability_record, compute_policy_hash, default_v02_policy,
    format_fixed, load_declarations, load_envelopes, parse_fixed, rebuild_capability,
    require_proven, save_policy,
};

pub fn validate_extension_descriptor(descriptor: &ExtensionDescriptor) -> Result<(), ForgeError> {
    const SURFACES: &[&str] = &[
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
    let valid_hash = |value: &str| {
        value.strip_prefix("sha256:").is_some_and(|hex| {
            hex.len() == 64
                && hex
                    .chars()
                    .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        })
    };
    if !SURFACES.contains(&descriptor.authority_surface.as_str()) {
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
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::Path,
};
pub fn append(path: &Path, envelope: &SemanticEnvelope) -> Result<(), ForgeError> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| ForgeError::io(format!("open {}", path.display()), e))?;
    let mut encoded = serde_json::to_vec(envelope)?;
    encoded.push(b'\n');
    file.write_all(&encoded)
        .and_then(|_| file.flush())
        .map_err(|e| ForgeError::io("append capability envelope", e))
}
pub struct RecallQuery<'a> {
    pub query: &'a str,
    pub entity: Option<&'a str>,
    pub process: Option<&'a str>,
    pub result: Option<SettlementStatus>,
    pub limit: usize,
}
pub fn recall(
    path: &Path,
    query: RecallQuery<'_>,
) -> Result<(Vec<SemanticEnvelope>, usize), ForgeError> {
    let file = File::open(path).map_err(|error| ForgeError::io("open capability memory", error))?;
    let mut malformed = 0;
    let mut values: Vec<SemanticEnvelope> = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line.map_err(|error| ForgeError::io("read capability memory", error))?;
        match serde_json::from_str(&line) {
            Ok(e) => values.push(e),
            Err(_) => malformed += 1,
        }
    }
    let needle = query.query.to_lowercase();
    values.reverse();
    values.retain(|e| {
        (e.intent.summary.to_lowercase().contains(&needle)
            || e.capability_delta
                .attempted_capability
                .to_lowercase()
                .contains(&needle))
            && query.entity.is_none_or(|v| e.attribution.entity_id == v)
            && query.process.is_none_or(|v| e.attribution.process_id == v)
            && query
                .result
                .as_ref()
                .is_none_or(|v| &e.capability_delta.result == v)
    });
    values.truncate(query.limit);
    Ok((values, malformed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_forge_core::types::ContractRef;
    #[test]
    fn projection_extension_contract_requires_known_surface_and_hashes() {
        let descriptor = ExtensionDescriptor {
            extension_id: "ext_abcdef".into(),
            kind: ExtensionKind::ProjectionAdapter,
            name: "sea-projection".into(),
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
        };
        assert!(validate_extension_descriptor(&descriptor).is_ok());
    }
}
