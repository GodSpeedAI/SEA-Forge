use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::Path,
};

use chrono::Utc;
use sea_forge_core::{errors::ForgeError, ids::random_id, types::*, RECORD_VERSION};
use sea_forge_ledger::types::hash_canonical;

/// Compute the declaration hash over canonical content excluding `declaration_hash`.
pub fn compute_declaration_hash(decl: &SettlementDeclaration) -> Result<String, ForgeError> {
    let mut copy = decl.clone();
    copy.declaration_hash.clear();
    hash_canonical(&copy)
}

/// Append a declaration to a JSONL file.
pub fn append_declaration(path: &Path, decl: &SettlementDeclaration) -> Result<(), ForgeError> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| ForgeError::io(format!("open {}", path.display()), e))?;
    let mut encoded = serde_json::to_vec(decl)?;
    encoded.push(b'\n');
    file.write_all(&encoded)
        .and_then(|_| file.flush())
        .map_err(|e| ForgeError::io("append declaration", e))
}

/// Load all declarations from a JSONL file. Returns empty vec if file does not exist.
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

/// Trait for settlement authorities (spec §7.2.1).
pub trait SettlementAuthority {
    fn declare(
        &self,
        req: &SettlementDeclarationRequest,
    ) -> Result<SettlementDeclaration, ForgeError>;
    fn adapter_ref(&self) -> &str;
}

/// Local settlement authority — strength is always `local`; cannot satisfy strong-settlement policy.
pub struct LocalSettlementAuthority {
    acting_entity_id: String,
}

impl LocalSettlementAuthority {
    pub fn new(acting_entity_id: &str) -> Self {
        Self {
            acting_entity_id: acting_entity_id.into(),
        }
    }
}

impl SettlementAuthority for LocalSettlementAuthority {
    fn adapter_ref(&self) -> &str {
        "local"
    }

    fn declare(
        &self,
        req: &SettlementDeclarationRequest,
    ) -> Result<SettlementDeclaration, ForgeError> {
        let independent = req.declarer.actor_id != req.acting_entity_id;
        let integrity = check_integrity(req);
        let status = if integrity.is_ok() {
            DeclarationStatus::Accepted
        } else {
            DeclarationStatus::Rejected
        };

        let mut decl = SettlementDeclaration {
            version: RECORD_VERSION.into(),
            declaration_id: random_id("sdec")?,
            settlement_ref: req.settlement_ref.clone(),
            run_id: req.run_id.clone(),
            case_id: req.case_id.clone(),
            plan_item_id: req.plan_item_id.clone(),
            claim_manifest_sha256: req.claim_manifest_sha256.clone(),
            status,
            strength: SettlementStrength::Local,
            qualifies_for_capability: false,
            criteria_ref: req.criteria_ref.clone(),
            criteria_sha256: req.criteria_sha256.clone(),
            criteria_record_hash: req.criteria_record_hash.clone(),
            criteria_declared_at: req.criteria_declared_at.clone(),
            job_contract_ref: req.job_contract_ref.clone(),
            origin_refs: req.origin_refs.clone(),
            verifier_ref: req.verifier_ref.clone(),
            verifier_sha256: req.verifier_sha256.clone(),
            verification_evidence_refs: req.source_evidence_refs.clone(),
            declarer: req.declarer.clone(),
            independence: DeclarationIndependence {
                acting_entity_id: self.acting_entity_id.clone(),
                independent,
                basis: if independent {
                    "declarer_differs_from_acting_entity".into()
                } else {
                    "self_declaration".into()
                },
            },
            reliability: DeclarationReliability {
                feedback_delay_ms: 0,
                attribution_confidence: "1.000000".into(),
                gaming_exposure: "0.000000".into(),
                hidden_debt_blindness: "1.000000".into(),
                weight: "0.000000".into(),
                basis: "local_adapter".into(),
            },
            variation_tags: req.variation_tags.clone(),
            disruption_tags: req.disruption_tags.clone(),
            orchestration_burden: req.orchestration_burden.clone(),
            issued_at: Utc::now().to_rfc3339(),
            source_evidence_refs: req.source_evidence_refs.clone(),
            adapter_attestation_ref: None,
            declaration_hash: String::new(),
        };
        decl.declaration_hash = compute_declaration_hash(&decl)?;
        Ok(decl)
    }
}

/// Response from a SWE_SEED transport, carrying the external authority's reliability assessment.
pub struct SweSeedResponse {
    pub attestation_ref: String,
    pub attribution_confidence: String,
    pub gaming_exposure: String,
    pub hidden_debt_blindness: String,
    pub feedback_delay_ms: u64,
}

/// Transport trait for SWE_SEED — real implementation submits to the external service;
/// test doubles return canned responses.
pub trait SweSeedTransport {
    fn submit(&self, req: &SettlementDeclarationRequest) -> Result<SweSeedResponse, ForgeError>;
}

/// SWE_SEED settlement authority — issues strong declarations backed by external verification.
pub struct SweSeedSettlementAuthority<T: SweSeedTransport> {
    transport: T,
    acting_entity_id: String,
}

impl<T: SweSeedTransport> SweSeedSettlementAuthority<T> {
    pub fn new(transport: T, acting_entity_id: &str) -> Self {
        Self {
            transport,
            acting_entity_id: acting_entity_id.into(),
        }
    }
}

impl<T: SweSeedTransport> SettlementAuthority for SweSeedSettlementAuthority<T> {
    fn adapter_ref(&self) -> &str {
        "swe_seed"
    }

    fn declare(
        &self,
        req: &SettlementDeclarationRequest,
    ) -> Result<SettlementDeclaration, ForgeError> {
        let independent = req.declarer.actor_id != req.acting_entity_id;
        let integrity = check_integrity(req);
        let swe_resp = self.transport.submit(req)?;

        let weight = compute_weight(
            &swe_resp.attribution_confidence,
            &swe_resp.gaming_exposure,
            &swe_resp.hidden_debt_blindness,
        );

        let qualifies = integrity.is_ok()
            && req.requested_strength == SettlementStrength::Strong
            && independent;

        let status = if integrity.is_ok() {
            DeclarationStatus::Accepted
        } else {
            DeclarationStatus::Rejected
        };

        let mut decl = SettlementDeclaration {
            version: RECORD_VERSION.into(),
            declaration_id: random_id("sdec")?,
            settlement_ref: req.settlement_ref.clone(),
            run_id: req.run_id.clone(),
            case_id: req.case_id.clone(),
            plan_item_id: req.plan_item_id.clone(),
            claim_manifest_sha256: req.claim_manifest_sha256.clone(),
            status,
            strength: SettlementStrength::Strong,
            qualifies_for_capability: qualifies,
            criteria_ref: req.criteria_ref.clone(),
            criteria_sha256: req.criteria_sha256.clone(),
            criteria_record_hash: req.criteria_record_hash.clone(),
            criteria_declared_at: req.criteria_declared_at.clone(),
            job_contract_ref: req.job_contract_ref.clone(),
            origin_refs: req.origin_refs.clone(),
            verifier_ref: req.verifier_ref.clone(),
            verifier_sha256: req.verifier_sha256.clone(),
            verification_evidence_refs: req.source_evidence_refs.clone(),
            declarer: req.declarer.clone(),
            independence: DeclarationIndependence {
                acting_entity_id: self.acting_entity_id.clone(),
                independent,
                basis: if independent {
                    "swe_seed_independent_verification".into()
                } else {
                    "self_declaration".into()
                },
            },
            reliability: DeclarationReliability {
                feedback_delay_ms: swe_resp.feedback_delay_ms,
                attribution_confidence: swe_resp.attribution_confidence,
                gaming_exposure: swe_resp.gaming_exposure,
                hidden_debt_blindness: swe_resp.hidden_debt_blindness,
                weight,
                basis: "swe_seed_adapter".into(),
            },
            variation_tags: req.variation_tags.clone(),
            disruption_tags: req.disruption_tags.clone(),
            orchestration_burden: req.orchestration_burden.clone(),
            issued_at: Utc::now().to_rfc3339(),
            source_evidence_refs: req.source_evidence_refs.clone(),
            adapter_attestation_ref: Some(swe_resp.attestation_ref),
            declaration_hash: String::new(),
        };
        decl.declaration_hash = compute_declaration_hash(&decl)?;
        Ok(decl)
    }
}

/// Integrity checks shared by all adapters (spec §7.2.1).
fn check_integrity(req: &SettlementDeclarationRequest) -> Result<(), ForgeError> {
    if req.criteria_declared_at > req.execution_started_at {
        return Err(ForgeError::Plan {
            class: "settlement_integrity_error",
            message: "criteria_declared_at follows execution_started_at (post-hoc criteria)".into(),
        });
    }
    if req.declarer.actor_id == req.acting_entity_id {
        return Err(ForgeError::Plan {
            class: "settlement_integrity_error",
            message: "declarer is the acting entity (self-declaration)".into(),
        });
    }
    if req.criteria_ref.is_empty() {
        return Err(ForgeError::Plan {
            class: "settlement_integrity_error",
            message: "missing criteria_ref (legacy_unattributed_criteria)".into(),
        });
    }
    if req.origin_refs.is_empty() {
        return Err(ForgeError::Plan {
            class: "settlement_integrity_error",
            message: "missing origin_refs".into(),
        });
    }
    Ok(())
}

/// Fixed-point weight: attribution_confidence * (1 - gaming_exposure) * (1 - hidden_debt_blindness).
/// Uses i64 millionths internally, clamped to [0, 1_000_000].
fn compute_weight(conf: &str, exposure: &str, blindness: &str) -> String {
    let c = parse_fixed(conf).unwrap_or(0);
    let e = parse_fixed(exposure).unwrap_or(0);
    let b = parse_fixed(blindness).unwrap_or(0);
    let weight = c * (1_000_000 - e) / 1_000_000 * (1_000_000 - b) / 1_000_000;
    format_fixed(weight.clamp(0, 1_000_000))
}

/// Parse a fixed-scale decimal string ("0.800000") into millionths (800000).
fn parse_fixed(s: &str) -> Result<i64, ()> {
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
    let frac_part = parts.get(1).unwrap_or(&"");
    let frac_padded = format!("{:0<6}", &frac_part[..frac_part.len().min(6)]);
    let frac: i64 = frac_padded.parse().unwrap_or(0);
    Ok(sign * (int_part * 1_000_000 + frac))
}

/// Format millionths (800000) as a fixed-scale decimal string ("0.800000").
fn format_fixed(v: i64) -> String {
    let sign = if v < 0 { "-" } else { "" };
    let abs = v.unsigned_abs();
    format!("{}{}.{:06}", sign, abs / 1_000_000, abs % 1_000_000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_point_roundtrip() {
        assert_eq!(parse_fixed("0.800000").unwrap(), 800_000);
        assert_eq!(parse_fixed("1.000000").unwrap(), 1_000_000);
        assert_eq!(parse_fixed("0.000000").unwrap(), 0);
        assert_eq!(format_fixed(800_000), "0.800000");
        assert_eq!(format_fixed(1_000_000), "1.000000");
    }

    #[test]
    fn weight_high_confidence_low_exposure() {
        let w = compute_weight("0.950000", "0.050000", "0.100000");
        let v = parse_fixed(&w).unwrap();
        // 0.95 * 0.95 * 0.90 = 0.812250 → 812250
        assert!(v >= 800_000, "expected weight >= 0.8, got {w}");
    }

    #[test]
    fn weight_high_exposure_reduces() {
        let w = compute_weight("0.950000", "0.800000", "0.100000");
        let v = parse_fixed(&w).unwrap();
        // 0.95 * 0.20 * 0.90 = 0.171 → 171000
        assert!(v < 200_000, "expected weight < 0.2, got {w}");
    }
}
