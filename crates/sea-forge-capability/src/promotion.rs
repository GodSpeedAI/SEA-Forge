use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::Path,
};

use sea_forge_core::{errors::ForgeError, types::*, RECORD_VERSION};
use sea_forge_ledger::types::hash_canonical;

// ── Fixed-point arithmetic (6 decimal places, i64 millionths) ──

const SCALE: i64 = 1_000_000;

pub fn parse_fixed(s: &str) -> i64 {
    let s = s.trim();
    let (sign, rest) = if let Some(stripped) = s.strip_prefix('-') {
        (-1, stripped)
    } else {
        (1, s)
    };
    let parts: Vec<&str> = rest.split('.').collect();
    let int_part: i64 = parts
        .first()
        .filter(|p| !p.is_empty())
        .map(|p| p.parse::<i64>().unwrap_or(0))
        .unwrap_or(0);
    let frac_part = parts.get(1).copied().unwrap_or("");
    let frac_padded = format!("{:0<6}", &frac_part[..frac_part.len().min(6)]);
    let frac: i64 = frac_padded.parse().unwrap_or(0);
    sign * (int_part * SCALE + frac)
}

pub fn format_fixed(v: i64) -> String {
    let sign = if v < 0 { "-" } else { "" };
    let abs = v.unsigned_abs();
    format!("{}{}.{:06}", sign, abs / SCALE as u64, abs % SCALE as u64)
}

fn clamp(v: i64) -> i64 {
    v.clamp(0, SCALE)
}

// ── Default v0.2 promotion policy ──

pub fn default_v02_policy() -> CapabilityPromotionPolicy {
    let mut policy = CapabilityPromotionPolicy {
        version: RECORD_VERSION.into(),
        name: "default_v02".into(),
        capability_pattern: "*".into(),
        policy_sha256: String::new(),
        min_declarations: 3,
        min_total_weight: "2.400000".into(),
        min_reliability_weight: "0.800000".into(),
        max_regression_weight: "0.500000".into(),
        required_variation_dimensions: vec!["model_scale".into(), "input_type".into()],
        min_distinct_values_per_dimension: 2,
        required_disruptions: vec!["disk_full".into()],
        require_burden_reduction: true,
        min_confidence: "0.000000".into(),
    };
    policy.policy_sha256 = compute_policy_hash(&policy).unwrap_or_default();
    policy
}

pub fn compute_policy_hash(policy: &CapabilityPromotionPolicy) -> Result<String, ForgeError> {
    let mut copy = policy.clone();
    copy.policy_sha256.clear();
    hash_canonical(&copy)
}

/// Save a policy snapshot to `.sea-forge/capabilities/policies/<sha256>.json`.
pub fn save_policy(root: &Path, policy: &CapabilityPromotionPolicy) -> Result<(), ForgeError> {
    let dir = root.join(".sea-forge/capabilities/policies");
    std::fs::create_dir_all(&dir).map_err(|e| ForgeError::io("create policy dir", e))?;
    let path = dir.join(format!("{}.json", policy.policy_sha256));
    let json = serde_json::to_vec_pretty(policy)?;
    std::fs::write(&path, json).map_err(|e| ForgeError::io("write policy", e))
}

/// Load a policy snapshot by hash.
pub fn load_policy(
    root: &Path,
    policy_sha256: &str,
) -> Result<CapabilityPromotionPolicy, ForgeError> {
    let path = root
        .join(".sea-forge/capabilities/policies")
        .join(format!("{policy_sha256}.json"));
    let data = std::fs::read(&path)
        .map_err(|e| ForgeError::io(format!("read policy {policy_sha256}"), e))?;
    let policy: CapabilityPromotionPolicy =
        serde_json::from_slice(&data).map_err(|e| ForgeError::Serialization(e.to_string()))?;
    let computed = compute_policy_hash(&policy)?;
    if computed != policy.policy_sha256 {
        return Err(ForgeError::Internal(format!(
            "policy hash mismatch: expected {policy_sha256}, computed {computed}"
        )));
    }
    Ok(policy)
}

// ── Declaration qualification (policy-dependent) ──

/// A declaration contributes qualifying weight iff all §7.2.1 conditions hold:
/// - status is accepted, strength is strong, qualifies_for_capability is true
/// - independent is true
/// - weight meets policy min_reliability_weight
/// - not legacy_unattributed_criteria (criteria_ref is non-empty)
/// - every variation/disruption/burden tag cites source evidence
pub fn declaration_qualifies(
    decl: &SettlementDeclaration,
    policy: &CapabilityPromotionPolicy,
) -> bool {
    // SoD boundary (spec-adlc-thoth §10.3, T13B), re-checked independently of
    // settlement's own accept-time check: a declaration copied or replayed
    // directly into the promotion pipeline (bypassing `declare()`) still
    // cannot count toward promoting a capability its own author declared.
    if sea_forge_core::types::validate_claim_authorship_sod(
        decl.authored_by.as_deref(),
        &decl.declarer.actor_id,
    )
    .is_err()
    {
        return false;
    }
    if decl.status != DeclarationStatus::Accepted {
        return false;
    }
    if decl.strength != SettlementStrength::Strong {
        return false;
    }
    if !decl.qualifies_for_capability {
        return false;
    }
    if !decl.independence.independent {
        return false;
    }
    let weight = parse_fixed(&decl.reliability.weight);
    let min_weight = parse_fixed(&policy.min_reliability_weight);
    if weight < min_weight {
        return false;
    }
    if decl.criteria_ref.is_empty() {
        return false;
    }
    // Variation/disruption/burden tags must cite source evidence
    if !decl.variation_tags.is_empty() && decl.source_evidence_refs.is_empty() {
        return false;
    }
    true
}

// ── Capability record projection ──

/// Load envelopes from `capabilities.jsonl`.
pub fn load_envelopes(path: &Path) -> Result<Vec<SemanticEnvelope>, ForgeError> {
    let file =
        File::open(path).map_err(|e| ForgeError::io(format!("open {}", path.display()), e))?;
    let mut envelopes = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line.map_err(|e| ForgeError::io("read envelopes", e))?;
        if line.trim().is_empty() {
            continue;
        }
        let env: SemanticEnvelope =
            serde_json::from_str(&line).map_err(|e| ForgeError::Serialization(e.to_string()))?;
        envelopes.push(env);
    }
    Ok(envelopes)
}

/// Build a CapabilityRecord from envelopes, declarations, and a promotion policy.
/// This is the pure projection — same inputs always produce the same record
/// modulo `rebuilt_at`.
pub fn build_capability_record(
    capability_name: &str,
    envelopes: &[SemanticEnvelope],
    declarations: &[SettlementDeclaration],
    policy: &CapabilityPromotionPolicy,
    rebuilt_at: &str,
) -> CapabilityRecord {
    // Filter declarations for this capability (by plan_item_id match)
    let cap_decls: Vec<&SettlementDeclaration> = declarations
        .iter()
        .filter(|d| {
            // Declaration's plan_item_id should map to the capability.
            // For now, we match on capability_name appearing in plan_item_id or
            // match all declarations if capability_name is "*".
            d.plan_item_id.contains(capability_name) || capability_name == "*"
        })
        .collect();

    // Filter envelopes for this capability
    let cap_envs: Vec<&SemanticEnvelope> = envelopes
        .iter()
        .filter(|e| e.capability_delta.attempted_capability == capability_name)
        .collect();

    // Raw counts from envelopes
    let counts = CapabilityCounts {
        accepted: cap_envs
            .iter()
            .filter(|e| e.capability_delta.result == SettlementStatus::Accepted)
            .count() as u64,
        rejected: cap_envs
            .iter()
            .filter(|e| e.capability_delta.result == SettlementStatus::Rejected)
            .count() as u64,
        escalated: cap_envs
            .iter()
            .filter(|e| e.capability_delta.result == SettlementStatus::Escalated)
            .count() as u64,
    };

    // Qualifying declarations
    let qualifying_decls: Vec<&SettlementDeclaration> = cap_decls
        .iter()
        .copied()
        .filter(|d| declaration_qualifies(d, policy))
        .collect();

    let rejected_strong_decls: Vec<&SettlementDeclaration> = cap_decls
        .iter()
        .copied()
        .filter(|d| {
            d.strength == SettlementStrength::Strong && d.status == DeclarationStatus::Rejected
        })
        .collect();

    // Weights (fixed-point)
    let accepted_weight: i64 = qualifying_decls
        .iter()
        .map(|d| parse_fixed(&d.reliability.weight))
        .sum();
    let regression_weight: i64 = rejected_strong_decls
        .iter()
        .map(|d| parse_fixed(&d.reliability.weight))
        .sum();
    let total_weight = accepted_weight + regression_weight;

    let qualifying = CapabilityQualifying {
        declaration_count: qualifying_decls.len() as u64,
        total_weight: format_fixed(total_weight),
        accepted_weight: format_fixed(accepted_weight),
        regression_weight: format_fixed(regression_weight),
    };

    // Variation coverage
    let mut covered_values: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for d in &qualifying_decls {
        for (dim, val) in &d.variation_tags {
            covered_values
                .entry(dim.clone())
                .or_default()
                .insert(val.clone());
        }
    }
    let covered_values_map: BTreeMap<String, Vec<String>> = covered_values
        .iter()
        .map(|(k, v)| (k.clone(), v.iter().cloned().collect()))
        .collect();
    let dims_covered = policy
        .required_variation_dimensions
        .iter()
        .filter(|dim| {
            covered_values
                .get(*dim)
                .is_some_and(|vals| vals.len() as u64 >= policy.min_distinct_values_per_dimension)
        })
        .count();
    let coverage_ratio = if policy.required_variation_dimensions.is_empty() {
        SCALE
    } else {
        SCALE * dims_covered as i64 / policy.required_variation_dimensions.len() as i64
    };

    let variation = CapabilityVariation {
        required_dimensions: policy.required_variation_dimensions.clone(),
        covered_values: covered_values_map,
        coverage_ratio: format_fixed(coverage_ratio),
    };

    // Recovery
    let recovered_disruptions: Vec<String> = policy
        .required_disruptions
        .iter()
        .filter(|req| {
            qualifying_decls
                .iter()
                .any(|d| d.disruption_tags.iter().any(|t| t == *req))
        })
        .cloned()
        .collect();
    let recovery_ratio = if policy.required_disruptions.is_empty() {
        SCALE
    } else {
        SCALE * recovered_disruptions.len() as i64 / policy.required_disruptions.len() as i64
    };

    let recovery = CapabilityRecovery {
        required_disruptions: policy.required_disruptions.clone(),
        recovered_disruptions,
        recovery_ratio: format_fixed(recovery_ratio),
    };

    // Orchestration burden
    let baseline_burden = qualifying_decls
        .iter()
        .filter_map(|d| d.orchestration_burden.as_deref())
        .max_by(|a, b| parse_fixed(a).cmp(&parse_fixed(b)));
    let current_burden = qualifying_decls
        .iter()
        .filter_map(|d| d.orchestration_burden.as_deref())
        .min_by(|a, b| parse_fixed(a).cmp(&parse_fixed(b)));
    let reduction = match (baseline_burden, current_burden) {
        (Some(b), Some(c)) => {
            let diff = parse_fixed(b) - parse_fixed(c);
            Some(format_fixed(diff))
        }
        _ => None,
    };

    let orchestration = CapabilityOrchestration {
        baseline_burden: baseline_burden.map(|s| s.into()),
        current_burden: current_burden.map(|s| s.into()),
        reduction,
    };

    // Confidence: reliability_ratio * coverage_ratio * recovery_ratio * burden_factor
    let denom = accepted_weight + regression_weight;
    let reliability_ratio = if denom == 0 {
        0
    } else {
        accepted_weight * SCALE / denom
    };
    let burden_factor = if policy.require_burden_reduction {
        match (
            &orchestration.baseline_burden,
            &orchestration.current_burden,
        ) {
            (Some(b), Some(c)) if parse_fixed(b) > parse_fixed(c) => SCALE,
            _ => 0,
        }
    } else {
        SCALE
    };
    let confidence = clamp(
        reliability_ratio * coverage_ratio / SCALE * recovery_ratio / SCALE * burden_factor / SCALE,
    );

    // Status determination
    let min_total = parse_fixed(&policy.min_total_weight);
    let max_regression = parse_fixed(&policy.max_regression_weight);
    let min_conf = parse_fixed(&policy.min_confidence);

    let meets_declarations = qualifying_decls.len() as u64 >= policy.min_declarations;
    let meets_total_weight = accepted_weight >= min_total;
    let meets_regression = regression_weight < max_regression;
    let meets_confidence = confidence >= min_conf;
    let meets_burden = !policy.require_burden_reduction || burden_factor == SCALE;

    let mut contraction_reasons: Vec<String> = Vec::new();

    let status = if meets_declarations
        && meets_total_weight
        && meets_regression
        && meets_confidence
        && meets_burden
        && coverage_ratio == SCALE
        && recovery_ratio == SCALE
    {
        CapabilityStatus::Proven
    } else if qualifying_decls.is_empty()
        && counts.accepted == 0
        && counts.rejected == 0
        && counts.escalated == 0
    {
        CapabilityStatus::Attempted
    } else if !qualifying_decls.is_empty() {
        CapabilityStatus::Demonstrated
    } else {
        CapabilityStatus::Attempted
    };

    // Contraction detection: if we had proven/demonstrated but now don't
    if status == CapabilityStatus::Attempted && !qualifying_decls.is_empty() {
        contraction_reasons.push("reliability_below_threshold".into());
    }
    if !meets_regression && regression_weight > 0 {
        contraction_reasons.push("regression".into());
    }
    if coverage_ratio < SCALE && !qualifying_decls.is_empty() {
        contraction_reasons.push("evidence_invalidated".into());
    }
    contraction_reasons.sort();
    contraction_reasons.dedup();

    // Evidence sample (≤ 10 most recent)
    let mut evidence_sample: Vec<CapabilityEvidenceSampleItem> = cap_envs
        .iter()
        .rev()
        .take(10)
        .map(|e| {
            let decl_for_env = cap_decls.iter().find(|d| d.run_id == e.run_id);
            CapabilityEvidenceSampleItem {
                run_id: e.run_id.clone(),
                settlement_status: format!("{:?}", e.capability_delta.result).to_lowercase(),
                declaration_id: decl_for_env.map(|d| d.declaration_id.clone()),
                weight: decl_for_env.map(|d| d.reliability.weight.clone()),
            }
        })
        .collect();
    evidence_sample.reverse();

    // first_seen / last_seen from envelopes
    let first_seen = cap_envs
        .iter()
        .map(|e| e.intent.created_at.clone())
        .min()
        .unwrap_or_default();
    let last_seen = cap_envs
        .iter()
        .map(|e| e.intent.created_at.clone())
        .max()
        .unwrap_or_default();

    CapabilityRecord {
        version: RECORD_VERSION.into(),
        capability_name: capability_name.into(),
        first_seen,
        last_seen,
        counts,
        status,
        promotion_policy_ref: policy.name.clone(),
        promotion_policy_sha256: policy.policy_sha256.clone(),
        qualifying,
        variation,
        recovery,
        orchestration,
        confidence: format_fixed(confidence),
        contraction_reasons,
        evidence_sample,
        rebuilt_at: rebuilt_at.into(),
    }
}

/// Rebuild all capability records from envelopes + declarations + policy.
/// Writes to `.sea-forge/capabilities/<name>.json`.
/// Same inputs always produce byte-identical output modulo `rebuilt_at`.
pub fn rebuild_capability(
    root: &Path,
    capability_name: &str,
    policy: &CapabilityPromotionPolicy,
) -> Result<CapabilityRecord, ForgeError> {
    let envelopes = load_envelopes(&root.join(".sea-forge/capabilities.jsonl"))?;
    let declarations = load_declarations(&root.join(".sea-forge/settlement/declarations.jsonl"))?;
    let record = build_capability_record(
        capability_name,
        &envelopes,
        &declarations,
        policy,
        &chrono::Utc::now().to_rfc3339(),
    );
    // Write the record
    let dir = root.join(".sea-forge/capabilities");
    std::fs::create_dir_all(&dir).map_err(|e| ForgeError::io("create capability dir", e))?;
    let path = dir.join(format!("{capability_name}.json"));
    let json = serde_json::to_vec_pretty(&record)?;
    std::fs::write(&path, json).map_err(|e| ForgeError::io("write capability record", e))?;
    Ok(record)
}

/// Check require_proven: deny unless the capability's rebuilt record has
/// status at least `proven` under the referenced promotion-policy hash.
/// Returns Ok(()) if proven, Err with citation otherwise.
pub fn require_proven(
    root: &Path,
    capability_name: &str,
    policy_ref: &str,
    policy_sha256: &str,
) -> Result<CapabilityRecord, ForgeError> {
    let policy = load_policy(root, policy_sha256)?;
    if policy.name != policy_ref {
        return Err(ForgeError::Plan {
            class: "settlement_integrity_error",
            message: format!(
                "policy name mismatch: expected {policy_ref}, found {}",
                policy.name
            ),
        });
    }
    let record = rebuild_capability(root, capability_name, &policy)?;
    if record.status != CapabilityStatus::Proven && record.status != CapabilityStatus::Metabolized {
        return Err(ForgeError::Plan {
            class: "settlement_integrity_error",
            message: format!(
                "require_proven denied: capability '{}' has status {:?} (not proven) under policy {} sha256:{}",
                capability_name, record.status, policy_ref, policy_sha256
            ),
        });
    }
    Ok(record)
}

/// Load declarations from a JSONL file. Returns empty vec if file does not exist.
pub fn load_declarations(path: &Path) -> Result<Vec<SettlementDeclaration>, ForgeError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file =
        File::open(path).map_err(|e| ForgeError::io(format!("open {}", path.display()), e))?;
    let mut decls = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line.map_err(|e| ForgeError::io("read declarations", e))?;
        if line.trim().is_empty() {
            continue;
        }
        let decl: SettlementDeclaration =
            serde_json::from_str(&line).map_err(|e| ForgeError::Serialization(e.to_string()))?;
        decls.push(decl);
    }
    Ok(decls)
}

/// Append a capability record to a JSONL file.
pub fn append_capability_record(path: &Path, record: &CapabilityRecord) -> Result<(), ForgeError> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| ForgeError::io(format!("open {}", path.display()), e))?;
    let mut encoded = serde_json::to_vec(record)?;
    encoded.push(b'\n');
    file.write_all(&encoded)
        .and_then(|_| file.flush())
        .map_err(|e| ForgeError::io("append capability record", e))
}
