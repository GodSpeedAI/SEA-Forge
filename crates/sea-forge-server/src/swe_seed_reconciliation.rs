//! SWE_SEED declaration reconciliation (spec-agent-orchestration M16, plan
//! Task 18).
//!
//! `sea_forge_settlement::append_declaration_ledgered_once` is append-only:
//! it cannot rewrite the `agent_task_evidence` record a SWE_SEED-harnessed
//! delegation already settled. A qualifying declaration can therefore arrive
//! after settlement — through the server-owned path in `delegation.rs`, or
//! appended directly by an out-of-process actor while the server is absent.
//! This module is the single, pure, idempotent join between the two: given
//! a case ledger, it correlates every harvested-evidence run against every
//! verified declaration that matches it, and commits/materializes the
//! correlation exactly once per distinct declaration set. It is safe to call
//! repeatedly (immediately after settlement, at server startup, or at
//! read time) — an unchanged declaration set commits nothing new.

use sea_forge_core::{errors::ForgeError, types::SettlementDeclaration};
use sea_forge_ledger::{types::hash_canonical, CommittedRecordRef, LedgerStream};
use serde::Serialize;
use serde_json::Value;
use std::path::Path;

#[derive(Serialize)]
struct SweSeedClaimManifest<'a> {
    case_id: &'a str,
    run_id: &'a str,
    plan_item_id: &'a str,
    settlement_id: &'a str,
    transcript_sha256: &'a str,
    harvested_refs: &'a [String],
}

/// The exact claim a SWE_SEED declaration must match to correlate with a
/// run's harvested evidence. Computed identically by the production
/// declaration submitter and this reconciler so a declaration whose
/// `claim_manifest_sha256` was forged, relabeled, or replayed against a
/// different run/settlement never correlates.
pub fn swe_seed_claim_manifest_sha256(
    case_id: &str,
    run_id: &str,
    plan_item_id: &str,
    settlement_id: &str,
    transcript_sha256: &str,
    harvested_refs: &[String],
) -> Result<String, ForgeError> {
    hash_canonical(&SweSeedClaimManifest {
        case_id,
        run_id,
        plan_item_id,
        settlement_id,
        transcript_sha256,
        harvested_refs,
    })
}

#[derive(Serialize, Clone)]
pub struct SweSeedCorrelation {
    pub version: &'static str,
    pub case_id: String,
    pub run_id: String,
    pub plan_item_id: String,
    pub harvested_refs: Vec<String>,
    pub declaration_ids: Vec<String>,
}

pub struct SweSeedReconciliationOutcome {
    pub run_id: String,
    pub committed: CommittedRecordRef,
    pub declaration_ids: Vec<String>,
}

/// Append a SWE_SEED declaration (whoever the caller is — the server's own
/// production ingress or a resumed out-of-process transport) and immediately
/// reconcile it against the same case's harvested evidence. The append and
/// the reconciliation share no mutable state beyond the ledger itself, so
/// this is equally correct whether called from a live server or a bare CLI
/// invocation while the server is absent.
pub fn append_and_reconcile_swe_seed_declaration(
    root: &Path,
    actor: &str,
    decl: &SettlementDeclaration,
) -> Result<Vec<SweSeedReconciliationOutcome>, ForgeError> {
    sea_forge_settlement::append_declaration_ledgered_once(
        root,
        &root.join("declarations.jsonl"),
        actor,
        decl,
    )?;
    reconcile_swe_seed_declarations(root, &decl.case_id)
}

/// Pure, idempotent reconciler. Reads every `agent_task_evidence` record in
/// the case ledger that carries non-empty `harvested_refs`, joins it against
/// every ledger-verified `settlement_declaration` whose claim manifest
/// matches that exact run/settlement, and commits one `swe_seed_correlation`
/// record per distinct resulting declaration set (idempotency-keyed on the
/// declaration set itself, so an unchanged run commits nothing new and a
/// newly-arrived declaration produces one additional append-only record).
pub fn reconcile_swe_seed_declarations(
    root: &Path,
    case_id: &str,
) -> Result<Vec<SweSeedReconciliationOutcome>, ForgeError> {
    let ledger = LedgerStream::open(root, format!("case-{case_id}"), "sea-forge-server")?;
    ledger.verify()?;
    let entries = ledger.read_entries()?;

    let declarations: Vec<SettlementDeclaration> = entries
        .iter()
        .filter(|entry| entry.record_kind == "settlement_declaration")
        .map(|entry| serde_json::from_value(entry.payload.clone()))
        .collect::<Result<_, _>>()
        .map_err(|error: serde_json::Error| ForgeError::Serialization(error.to_string()))?;

    let mut outcomes = Vec::new();
    for evidence_entry in entries
        .iter()
        .filter(|entry| entry.record_kind == "agent_task_evidence")
    {
        let harvested_refs: Vec<String> = evidence_entry
            .payload
            .get("harvested_refs")
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default();
        if harvested_refs.is_empty() {
            continue;
        }
        let run_id = evidence_entry
            .payload
            .get("run_id")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ForgeError::Internal(
                    "ledger_integrity_error: agent_task_evidence missing run_id".into(),
                )
            })?
            .to_string();
        let plan_item_id = evidence_entry
            .payload
            .get("item_id")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ForgeError::Internal(
                    "ledger_integrity_error: agent_task_evidence missing item_id".into(),
                )
            })?
            .to_string();
        let transcript_sha256 = evidence_entry
            .payload
            .get("transcript_sha256")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let settlement_id = entries
            .iter()
            .find(|entry| {
                entry.record_kind == "settlement"
                    && entry.payload.get("run_id").and_then(Value::as_str) == Some(run_id.as_str())
            })
            .and_then(|entry| entry.payload.get("settlement_id").and_then(Value::as_str))
            .unwrap_or_default()
            .to_string();

        let expected_manifest = swe_seed_claim_manifest_sha256(
            case_id,
            &run_id,
            &plan_item_id,
            &settlement_id,
            &transcript_sha256,
            &harvested_refs,
        )?;

        let mut matching: Vec<&SettlementDeclaration> = declarations
            .iter()
            .filter(|decl| {
                decl.case_id == case_id
                    && decl.run_id == run_id
                    && decl.plan_item_id == plan_item_id
                    && decl.verifier_ref.contains("swe_seed")
                    && decl.claim_manifest_sha256 == expected_manifest
            })
            .collect();
        matching.sort_by(|a, b| a.declaration_id.cmp(&b.declaration_id));
        let declaration_ids: Vec<String> = matching
            .iter()
            .map(|decl| decl.declaration_id.clone())
            .collect();

        let record = SweSeedCorrelation {
            version: "0.2",
            case_id: case_id.into(),
            run_id: run_id.clone(),
            plan_item_id,
            harvested_refs,
            declaration_ids: declaration_ids.clone(),
        };
        let idempotency_key = format!(
            "{case_id}:{run_id}:sha256:{:x}",
            <sha2::Sha256 as sha2::Digest>::digest(declaration_ids.join(",").as_bytes())
        );
        let committed = ledger.commit_typed_once(
            "swe_seed_correlation",
            idempotency_key,
            vec![case_id.into(), run_id.clone()],
            &record,
            vec![evidence_entry.entry_ulid.clone()],
        )?;
        let bytes = serde_json::to_vec_pretty(&record)?;
        ledger.materialize_view(
            &committed,
            &root
                .join("runs")
                .join(&run_id)
                .join("swe-seed-correlation.json"),
            &bytes,
        )?;
        outcomes.push(SweSeedReconciliationOutcome {
            run_id,
            committed,
            declaration_ids,
        });
    }
    Ok(outcomes)
}

/// Reconcile every case ledger under `root`. Invoked at server startup
/// (mirroring `recover_cancelled_delegations`) so a declaration appended
/// entirely out-of-process while the server was absent is joined before any
/// completion claim is read.
pub fn reconcile_all_cases(root: &Path) -> Result<(), ForgeError> {
    let ledgers_dir = root.join("ledgers");
    if !ledgers_dir.exists() {
        return Ok(());
    }
    for entry in
        std::fs::read_dir(&ledgers_dir).map_err(|error| ForgeError::io("read ledgers", error))?
    {
        let entry = entry.map_err(|error| ForgeError::io("read ledger entry", error))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let is_dir = entry
            .file_type()
            .map_err(|error| ForgeError::io("read ledger entry type", error))?
            .is_dir();
        let Some(case_id) = is_dir.then(|| name.strip_prefix("case-")).flatten() else {
            continue;
        };
        reconcile_swe_seed_declarations(root, case_id)?;
    }
    Ok(())
}

pub struct SweSeedCompletion {
    pub run_id: String,
    pub case_id: String,
    pub harvested: bool,
    pub declaration_ids: Vec<String>,
}

/// Read-time freshness check: reconcile the run's case first (joining any
/// declaration appended since the last reconciliation), then report whether
/// the run's harvested SWE_SEED evidence has a correlated declaration.
pub fn verify_swe_seed_completion(
    root: &Path,
    run_id: &str,
) -> Result<SweSeedCompletion, ForgeError> {
    let plan_path = root.join("runs").join(run_id).join("plan.json");
    let plan: sea_forge_core::types::CasePlan = serde_json::from_slice(
        &std::fs::read(&plan_path)
            .map_err(|error| ForgeError::io("read run plan for swe_seed completion", error))?,
    )?;
    reconcile_swe_seed_declarations(root, &plan.case_id)?;

    let ledger = LedgerStream::open(root, format!("case-{}", plan.case_id), "sea-forge-server")?;
    let entries = ledger.read_entries()?;
    let harvested = entries.iter().any(|entry| {
        entry.record_kind == "agent_task_evidence"
            && entry.payload.get("run_id").and_then(Value::as_str) == Some(run_id)
            && entry
                .payload
                .get("harvested_refs")
                .and_then(Value::as_array)
                .is_some_and(|refs| !refs.is_empty())
    });
    let declaration_ids = entries
        .iter()
        .filter(|entry| {
            entry.record_kind == "swe_seed_correlation"
                && entry.payload.get("run_id").and_then(Value::as_str) == Some(run_id)
        })
        .filter_map(|entry| {
            entry
                .payload
                .get("declaration_ids")
                .and_then(|value| serde_json::from_value::<Vec<String>>(value.clone()).ok())
        })
        .next_back()
        .unwrap_or_default();

    Ok(SweSeedCompletion {
        run_id: run_id.into(),
        case_id: plan.case_id,
        harvested,
        declaration_ids,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use sea_forge_core::types::{
        DeclarationIndependence, DeclarationReliability, DeclarationStatus, Declarer,
        SettlementStrength,
    };

    fn declaration(
        case_id: &str,
        run_id: &str,
        plan_item_id: &str,
        claim_manifest_sha256: &str,
        verifier_ref: &str,
    ) -> SettlementDeclaration {
        let mut decl = SettlementDeclaration {
            version: "0.2".into(),
            declaration_id: sea_forge_core::ids::random_id("sdec").unwrap(),
            settlement_ref: "set_x".into(),
            run_id: run_id.into(),
            case_id: case_id.into(),
            plan_item_id: plan_item_id.into(),
            claim_manifest_sha256: claim_manifest_sha256.into(),
            status: DeclarationStatus::Accepted,
            strength: SettlementStrength::Strong,
            qualifies_for_capability: true,
            criteria_ref: "crit_x".into(),
            criteria_sha256: "sha256:crit".into(),
            criteria_record_hash: "sha256:crit-record".into(),
            criteria_declared_at: Utc::now().to_rfc3339(),
            job_contract_ref: None,
            origin_refs: vec![],
            verifier_ref: verifier_ref.into(),
            verifier_sha256: "sha256:verifier".into(),
            verification_evidence_refs: vec![],
            declarer: Declarer {
                actor_id: "swe_seed_verifier".into(),
                authority_ref: "swe_seed_test".into(),
                role: "R-AA".into(),
                standing_basis: "test".into(),
            },
            independence: DeclarationIndependence {
                acting_entity_id: "operator_local".into(),
                independent: true,
                basis: "test".into(),
            },
            reliability: DeclarationReliability {
                feedback_delay_ms: 0,
                attribution_confidence: "1.000000".into(),
                gaming_exposure: "0.000000".into(),
                hidden_debt_blindness: "0.000000".into(),
                weight: "1.000000".into(),
                basis: "test".into(),
            },
            variation_tags: Default::default(),
            disruption_tags: vec![],
            orchestration_burden: None,
            issued_at: Utc::now().to_rfc3339(),
            source_evidence_refs: vec![],
            adapter_attestation_ref: None,
            authored_by: None,
            declaration_hash: String::new(),
        };
        decl.declaration_hash = sea_forge_settlement::compute_declaration_hash(&decl).unwrap();
        decl
    }

    fn commit_evidence_and_settlement(
        root: &Path,
        case_id: &str,
        run_id: &str,
        plan_item_id: &str,
        harvested_refs: Vec<String>,
    ) -> String {
        let ledger = LedgerStream::open(root, format!("case-{case_id}"), "test").unwrap();
        let evidence = serde_json::json!({
            "run_id": run_id,
            "item_id": plan_item_id,
            "case_id": case_id,
            "endpoint_ref": "ep",
            "turns_used": 1,
            "termination": "completed",
            "transcript_sha256": "sha256:transcript",
            "summary": {"turn_count": 1, "tool_calls": 0, "final_excerpt": ""},
            "artifact_ref": null,
            "harvested_refs": harvested_refs,
        });
        ledger
            .commit_typed(
                "agent_task_evidence",
                vec![case_id.into(), run_id.into(), plan_item_id.into()],
                &evidence,
                vec![],
            )
            .unwrap();
        let settlement = serde_json::json!({
            "version": "0.2",
            "settlement_id": "set_x",
            "run_id": run_id,
            "status": "accepted",
            "basis": ["delegation_completed"],
            "review_required": false,
            "settled_at": Utc::now().to_rfc3339(),
            "criteria_ref": null,
            "case_id": case_id,
            "item_id": plan_item_id,
        });
        ledger
            .commit_typed(
                "settlement",
                vec![run_id.into(), plan_item_id.into()],
                &settlement,
                vec![],
            )
            .unwrap();
        swe_seed_claim_manifest_sha256(
            case_id,
            run_id,
            plan_item_id,
            "set_x",
            "sha256:transcript",
            &["a:sha256:1".to_string()],
        )
        .unwrap()
    }

    #[test]
    fn mismatched_run_never_correlates() {
        let root = tempfile::tempdir().unwrap();
        let manifest = commit_evidence_and_settlement(
            root.path(),
            "case_a",
            "run_a",
            "item_acp",
            vec!["a:sha256:1".into()],
        );
        let decl = declaration(
            "case_a",
            "run_other",
            "item_acp",
            &manifest,
            "swe_seed_test",
        );
        sea_forge_settlement::append_declaration_ledgered_once(
            root.path(),
            &root.path().join("declarations.jsonl"),
            "test",
            &decl,
        )
        .unwrap();
        let outcomes = reconcile_swe_seed_declarations(root.path(), "case_a").unwrap();
        assert_eq!(outcomes.len(), 1);
        assert!(outcomes[0].declaration_ids.is_empty());
    }

    #[test]
    fn mismatched_verifier_never_correlates() {
        let root = tempfile::tempdir().unwrap();
        let manifest = commit_evidence_and_settlement(
            root.path(),
            "case_a",
            "run_a",
            "item_acp",
            vec!["a:sha256:1".into()],
        );
        let decl = declaration(
            "case_a",
            "run_a",
            "item_acp",
            &manifest,
            "some_other_verifier",
        );
        sea_forge_settlement::append_declaration_ledgered_once(
            root.path(),
            &root.path().join("declarations.jsonl"),
            "test",
            &decl,
        )
        .unwrap();
        let outcomes = reconcile_swe_seed_declarations(root.path(), "case_a").unwrap();
        assert!(outcomes[0].declaration_ids.is_empty());
    }

    #[test]
    fn mismatched_hash_never_correlates() {
        let root = tempfile::tempdir().unwrap();
        commit_evidence_and_settlement(
            root.path(),
            "case_a",
            "run_a",
            "item_acp",
            vec!["a:sha256:1".into()],
        );
        let decl = declaration(
            "case_a",
            "run_a",
            "item_acp",
            "sha256:not-the-real-manifest",
            "swe_seed_test",
        );
        sea_forge_settlement::append_declaration_ledgered_once(
            root.path(),
            &root.path().join("declarations.jsonl"),
            "test",
            &decl,
        )
        .unwrap();
        let outcomes = reconcile_swe_seed_declarations(root.path(), "case_a").unwrap();
        assert!(outcomes[0].declaration_ids.is_empty());
    }

    #[test]
    fn declaration_before_evidence_correlates_once_evidence_lands() {
        let root = tempfile::tempdir().unwrap();
        let expected = swe_seed_claim_manifest_sha256(
            "case_a",
            "run_a",
            "item_acp",
            "set_x",
            "sha256:transcript",
            &["a:sha256:1".to_string()],
        )
        .unwrap();
        let decl = declaration("case_a", "run_a", "item_acp", &expected, "swe_seed_test");
        sea_forge_settlement::append_declaration_ledgered_once(
            root.path(),
            &root.path().join("declarations.jsonl"),
            "test",
            &decl,
        )
        .unwrap();
        // No evidence yet: nothing to correlate against.
        assert!(reconcile_swe_seed_declarations(root.path(), "case_a")
            .unwrap()
            .is_empty());

        commit_evidence_and_settlement(
            root.path(),
            "case_a",
            "run_a",
            "item_acp",
            vec!["a:sha256:1".into()],
        );
        let outcomes = reconcile_swe_seed_declarations(root.path(), "case_a").unwrap();
        assert_eq!(outcomes[0].declaration_ids, vec![decl.declaration_id]);
        let view = root
            .path()
            .join("runs")
            .join("run_a")
            .join("swe-seed-correlation.json");
        assert!(view.exists());
    }

    #[test]
    fn repeated_reconcile_is_idempotent_no_duplicate_entries() {
        let root = tempfile::tempdir().unwrap();
        let manifest = commit_evidence_and_settlement(
            root.path(),
            "case_a",
            "run_a",
            "item_acp",
            vec!["a:sha256:1".into()],
        );
        let decl = declaration("case_a", "run_a", "item_acp", &manifest, "swe_seed_test");
        sea_forge_settlement::append_declaration_ledgered_once(
            root.path(),
            &root.path().join("declarations.jsonl"),
            "test",
            &decl,
        )
        .unwrap();
        reconcile_swe_seed_declarations(root.path(), "case_a").unwrap();
        reconcile_swe_seed_declarations(root.path(), "case_a").unwrap();
        reconcile_swe_seed_declarations(root.path(), "case_a").unwrap();
        let ledger = LedgerStream::open(root.path(), "case-case_a", "test").unwrap();
        let count = ledger
            .read_entries()
            .unwrap()
            .into_iter()
            .filter(|entry| entry.record_kind == "swe_seed_correlation")
            .count();
        assert_eq!(count, 1, "unchanged declaration set must not re-append");
    }

    #[test]
    fn two_runs_in_one_case_correlate_independently() {
        let root = tempfile::tempdir().unwrap();
        let manifest_a = commit_evidence_and_settlement(
            root.path(),
            "case_a",
            "run_a",
            "item_a",
            vec!["a:sha256:1".into()],
        );
        commit_evidence_and_settlement(
            root.path(),
            "case_a",
            "run_b",
            "item_b",
            vec!["b:sha256:2".into()],
        );
        let decl = declaration("case_a", "run_a", "item_a", &manifest_a, "swe_seed_test");
        sea_forge_settlement::append_declaration_ledgered_once(
            root.path(),
            &root.path().join("declarations.jsonl"),
            "test",
            &decl,
        )
        .unwrap();
        let outcomes = reconcile_swe_seed_declarations(root.path(), "case_a").unwrap();
        let run_a = outcomes.iter().find(|o| o.run_id == "run_a").unwrap();
        let run_b = outcomes.iter().find(|o| o.run_id == "run_b").unwrap();
        assert_eq!(run_a.declaration_ids, vec![decl.declaration_id]);
        assert!(run_b.declaration_ids.is_empty());
    }
}
