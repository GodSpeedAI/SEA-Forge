use sea_forge_core::{errors::ForgeError, ids::random_id, types::*, RECORD_VERSION};
use sea_forge_ledger::types::hash_canonical;
use std::collections::BTreeMap;

use crate::templates::PlanTemplate;

/// Compute the canonical SHA-256 of a `SettlementCriteria` value.
pub fn compute_criteria_sha256(criteria: &SettlementCriteria) -> Result<String, ForgeError> {
    hash_canonical(criteria)
}

/// Compute the canonical SHA-256 of a `SettlementCriteriaRecord` excluding only
/// the `criteria_record_hash` field.
pub fn compute_record_hash(record: &SettlementCriteriaRecord) -> Result<String, ForgeError> {
    let mut copy = record.clone();
    copy.criteria_record_hash.clear();
    hash_canonical(&copy)
}

/// Derive a `SettlementCriteriaRecord` from an `Intent`, reusing the Intent as
/// the origin. This is the default path for planner-built plans; no
/// JobContract is synthesized.
pub fn derive_from_intent(
    intent: &Intent,
    item: &PlanItem,
    actor_ref: &str,
    declared_at: &str,
) -> Result<SettlementCriteriaRecord, ForgeError> {
    let criteria_id = random_id("crit")?;
    let criteria = item.settlement_criteria.clone();
    let criteria_sha256 = compute_criteria_sha256(&criteria)?;
    let origin = OriginRef {
        kind: OriginRefKind::Intent,
        reference: intent.intent_id.clone(),
        sha256: hash_canonical(intent)?,
        role: OriginRole::DesiredResult,
        evidence_refs: vec![],
    };
    let mut record = SettlementCriteriaRecord {
        version: RECORD_VERSION.into(),
        criteria_id,
        criteria,
        origin_refs: vec![origin],
        derivation: CriteriaDerivation {
            method: DerivationMethod::DeterministicPlanner,
            actor_ref: actor_ref.into(),
            producer_ref: None,
            rationale: format!(
                "criteria for '{}' derived from intent '{}'",
                item.name, intent.summary
            ),
        },
        declared_at: declared_at.into(),
        criteria_sha256,
        criteria_record_hash: String::new(),
    };
    record.criteria_record_hash = compute_record_hash(&record)?;
    Ok(record)
}

/// Derive a `SettlementCriteriaRecord` from a `PlanTemplate` plus its
/// originating `Intent`. Reuses the immutable template version and the Intent
/// as origins; template-level `origin_refs` are included when present.
pub fn derive_from_template(
    template: &PlanTemplate,
    item: &PlanItem,
    intent: &Intent,
    actor_ref: &str,
    declared_at: &str,
) -> Result<SettlementCriteriaRecord, ForgeError> {
    let criteria_id = random_id("crit")?;
    let criteria = item.settlement_criteria.clone();
    let criteria_sha256 = compute_criteria_sha256(&criteria)?;
    let template_ref = format!("{}@{}", template.name, template.version);
    let mut origin_refs = vec![
        OriginRef {
            kind: OriginRefKind::Intent,
            reference: intent.intent_id.clone(),
            sha256: hash_canonical(intent)?,
            role: OriginRole::DesiredResult,
            evidence_refs: vec![],
        },
        OriginRef {
            kind: OriginRefKind::PlanTemplate,
            reference: template_ref.clone(),
            sha256: hash_canonical(template)?,
            role: OriginRole::AcceptanceSource,
            evidence_refs: vec![],
        },
    ];
    for origin in &template.origin_refs {
        origin_refs.push(origin.clone());
    }
    origin_refs.sort_by(|left, right| left.reference.cmp(&right.reference));
    let mut record = SettlementCriteriaRecord {
        version: RECORD_VERSION.into(),
        criteria_id,
        criteria,
        origin_refs,
        derivation: CriteriaDerivation {
            method: DerivationMethod::PlanTemplate,
            actor_ref: actor_ref.into(),
            producer_ref: Some(template_ref.clone()),
            rationale: format!(
                "criteria for '{}' instantiated from template '{}'",
                item.name, template_ref
            ),
        },
        declared_at: declared_at.into(),
        criteria_sha256,
        criteria_record_hash: String::new(),
    };
    record.criteria_record_hash = compute_record_hash(&record)?;
    Ok(record)
}

/// Lookup interface for criteria records used during verification.
pub trait CriteriaLookup {
    fn lookup(&self, criteria_id: &str) -> Option<SettlementCriteriaRecord>;
}

impl CriteriaLookup for BTreeMap<String, SettlementCriteriaRecord> {
    fn lookup(&self, criteria_id: &str) -> Option<SettlementCriteriaRecord> {
        self.get(criteria_id).cloned()
    }
}

impl CriteriaLookup for Vec<SettlementCriteriaRecord> {
    fn lookup(&self, criteria_id: &str) -> Option<SettlementCriteriaRecord> {
        self.iter()
            .find(|record| record.criteria_id == criteria_id)
            .cloned()
    }
}

/// Verify a PlanItem's settlement criteria provenance.
///
/// - `settlement_criteria_ref` must be present and resolve.
/// - The embedded criteria snapshot must hash to the record's `criteria_sha256`.
/// - Every `OriginRef` must be non-empty and have a hash.
pub fn verify_item_criteria(
    item: &PlanItem,
    lookup: &impl CriteriaLookup,
) -> Result<SettlementCriteriaRecord, ForgeError> {
    let criteria_ref = item
        .settlement_criteria_ref
        .as_deref()
        .ok_or_else(|| ForgeError::Plan {
            class: "criteria_provenance_error",
            message: format!(
                "plan item {} has no settlement_criteria_ref",
                item.plan_item_id
            ),
        })?;
    if criteria_ref.is_empty() {
        return Err(ForgeError::Plan {
            class: "criteria_provenance_error",
            message: format!(
                "plan item {} has an empty settlement_criteria_ref",
                item.plan_item_id
            ),
        });
    }
    let record = lookup
        .lookup(criteria_ref)
        .ok_or_else(|| ForgeError::Plan {
            class: "criteria_provenance_error",
            message: format!(
                "criteria record {criteria_ref} not found for plan item {}",
                item.plan_item_id
            ),
        })?;
    let expected = compute_criteria_sha256(&item.settlement_criteria)?;
    if record.criteria_sha256 != expected {
        return Err(ForgeError::Plan {
            class: "criteria_provenance_error",
            message: format!(
                "embedded criteria snapshot for {} does not match criteria_sha256",
                item.plan_item_id
            ),
        });
    }
    if record.origin_refs.is_empty() {
        return Err(ForgeError::Plan {
            class: "criteria_provenance_error",
            message: format!("criteria record {} has no origin_refs", record.criteria_id),
        });
    }
    for origin in &record.origin_refs {
        if origin.reference.is_empty() || origin.sha256.is_empty() {
            return Err(ForgeError::Plan {
                class: "criteria_provenance_error",
                message: format!(
                    "incomplete origin ref in criteria record {}",
                    record.criteria_id
                ),
            });
        }
    }
    Ok(record)
}

/// Verify every PlanItem that has a `settlement_criteria_ref`.
/// Items without a ref are treated as legacy-unattributed and are skipped.
pub fn verify_plan_criteria(
    plan: &CasePlan,
    lookup: &impl CriteriaLookup,
) -> Result<Vec<SettlementCriteriaRecord>, ForgeError> {
    let mut records = Vec::new();
    for item in &plan.items {
        if item.settlement_criteria_ref.is_none() {
            continue;
        }
        records.push(verify_item_criteria(item, lookup)?);
    }
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn demo_intent() -> Intent {
        Intent {
            intent_id: "int_abcdef".into(),
            summary: "generate and validate sea model".into(),
            actor_id: "entity".into(),
            process_id: "process".into(),
            created_at: "2026-07-13T00:00:00Z".into(),
        }
    }

    fn demo_item() -> PlanItem {
        PlanItem {
            plan_item_id: "item_01".into(),
            name: "generate_and_validate".into(),
            operations: vec![],
            entry_criteria: vec![],
            exit_criteria: vec![],
            settlement_criteria: SettlementCriteria {
                require_exit_zero: true,
                required_artifacts: vec!["model.sea".into()],
                stdout_must_contain: Some("valid".into()),
                ..Default::default()
            },
            settlement_criteria_ref: None,
            item_kind: ItemKind::SandboxedTask,
            sandbox_class: None,
            parent_stage: None,
            markers: Default::default(),
            max_instances: 1,
            depends_on: vec![],
            environment: None,
        }
    }

    #[test]
    fn criteria_hash_is_deterministic() {
        let item = demo_item();
        let a = compute_criteria_sha256(&item.settlement_criteria).unwrap();
        let b = compute_criteria_sha256(&item.settlement_criteria).unwrap();
        assert!(!a.is_empty());
        assert_eq!(a, b);
    }

    #[test]
    fn derived_record_hashes_to_itself() {
        let intent = demo_intent();
        let item = demo_item();
        let record = derive_from_intent(&intent, &item, "entity", "2026-07-13T00:00:00Z").unwrap();
        assert_eq!(
            record.criteria_record_hash,
            compute_record_hash(&record).unwrap()
        );
        assert!(!record.origin_refs.is_empty());
    }
}
