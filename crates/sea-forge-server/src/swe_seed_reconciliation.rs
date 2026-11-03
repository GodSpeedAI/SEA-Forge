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

use sea_forge_authority::cep::DeclarationAuthorization;
use sea_forge_core::{errors::ForgeError, types::SettlementDeclaration};
use sea_forge_ledger::{types::hash_canonical, CommittedRecordRef, LedgerStream};
use serde::Serialize;
use serde_json::Value;
use std::path::Path;

/// Audit record for a declaration that was refused as an authorisation at reconciliation time.
#[derive(Serialize)]
struct SettlementQuarantine<'a> {
    case_id: &'a str,
    run_id: &'a str,
    declaration_id: &'a str,
    declaration_hash: &'a str,
    reason: &'a str,
}

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

    // Every declaration in the case is verified against the authority ledger before it may correlate. An external
    // declaration without a verified authorisation is quarantined (recorded once, idempotently) and never joins.
    let verdicts = sea_forge_authority::cep::verify_case_declarations(root, &declarations)
        .map_err(|error| ForgeError::Internal(format!("settlement authorisation: {error}")))?;
    let mut authorized: Vec<&SettlementDeclaration> = Vec::new();
    for (declaration, verdict) in declarations.iter().zip(&verdicts) {
        match verdict {
            DeclarationAuthorization::Quarantined(reason) => {
                ledger.commit_typed_once(
                    "settlement_quarantined",
                    format!(
                        "settlement-quarantined:{}:{}",
                        declaration.declaration_id,
                        reason.as_str()
                    ),
                    vec![
                        case_id.into(),
                        declaration.run_id.clone(),
                        declaration.declaration_id.clone(),
                    ],
                    &SettlementQuarantine {
                        case_id,
                        run_id: &declaration.run_id,
                        declaration_id: &declaration.declaration_id,
                        declaration_hash: &declaration.declaration_hash,
                        reason: reason.as_str(),
                    },
                    vec![],
                )?;
            }
            DeclarationAuthorization::Local | DeclarationAuthorization::Authorized { .. } => {
                authorized.push(declaration)
            }
        }
    }

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

        let mut matching: Vec<&SettlementDeclaration> = authorized
            .iter()
            .copied()
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
            "{case_id}:{run_id}:sha256:{}",
            hex::encode(<sha2::Sha256 as sha2::Digest>::digest(
                declaration_ids.join(",").as_bytes()
            ))
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
    // The declaration ids come from this reconciliation's own verdicts, not from the last stored correlation: a
    // stored correlation can name a declaration since found unauthorised, and an identical set is never re-appended.
    let outcomes = reconcile_swe_seed_declarations(root, &plan.case_id)?;
    let declaration_ids = outcomes
        .iter()
        .rfind(|outcome| outcome.run_id == run_id)
        .map(|outcome| outcome.declaration_ids.clone())
        .unwrap_or_default();

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
    use sea_forge_authority::cep::{
        authorize_settlement_declaration, complete_settlement_resume, reconcile_operation_claims,
        settlement_approval_id, settlement_status, CepAuthorityService, SettlementCriteria,
        SettlementGate, SettlementIngress, SettlementOperation, SettlementStatus,
        SettlementTransport, VerifiedSettlementIngress,
    };
    use sea_forge_authority::{AuthorityPolicyBundle, PolicyAuthorityEngine};
    use sea_forge_core::types::{Actor, ActorRole};
    use sea_forge_core::types::{
        DeclarationIndependence, DeclarationReliability, DeclarationStatus, Declarer,
        SettlementStrength,
    };
    use sea_forge_domainforge::WorldRegistry;
    use serde_json::json;

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
            authorization_ref: None,
            authorization_request_sha256: None,
            declaration_hash: String::new(),
        };
        decl.declaration_hash = sea_forge_settlement::compute_declaration_hash(&decl).unwrap();
        decl
    }

    /// A declaration as the local judge makes it: local strength, not qualifying, with no external provenance.
    /// Production creates local declarations only this way (`LocalSettlementAuthority::declare`).
    fn local_declaration(
        case_id: &str,
        run_id: &str,
        plan_item_id: &str,
        claim_manifest_sha256: &str,
        verifier_ref: &str,
    ) -> SettlementDeclaration {
        let mut decl = declaration(
            case_id,
            run_id,
            plan_item_id,
            claim_manifest_sha256,
            verifier_ref,
        );
        decl.strength = SettlementStrength::Local;
        decl.qualifies_for_capability = false;
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

    const APPROVAL_POLICY: &str = "version: \"0.1\"\nrules:\n  - name: external-settlement\n    verdict: allow\n    requires_approval: true\n    actor_role: service\n    operation_kind: settlement_declaration\n    boundary_constraints:\n      settlement_authority_class: [external]\n  - name: approve\n    verdict: allow\n    actor_role: operator\n    operation_kind: approval_resolution\n";

    /// Gate `run_id` under `case-a`, presenting the declared content its own ingress hashes to.
    fn gate_as(
        engine: &PolicyAuthorityEngine,
        bundle: &AuthorityPolicyBundle,
        root: &Path,
        run_id: &str,
        ingress: &VerifiedSettlementIngress<'_>,
        ttl: chrono::Duration,
    ) -> Result<SettlementGate, sea_forge_authority::cep::CepAuthorityError> {
        let declared = format!("sha256:content-{run_id}");
        authorize_settlement_declaration(
            engine,
            bundle,
            root,
            SettlementOperation {
                case_id: "case_a",
                operation_id: run_id,
                declared_request_sha256: &declared,
            },
            ingress,
            ttl,
        )
    }

    /// Gate one external declaration for `run_id` (pending), approve it as a distinct operator, and claim it.
    /// Returns the claimed pending record's id and request hash.
    fn claim_approved(root: &Path, run_id: &str) -> (String, String) {
        // A fresh engine per call, as production does: an engine remembers an escalation it issued.
        let bundle: AuthorityPolicyBundle =
            sea_forge_core::yaml::from_str(APPROVAL_POLICY).unwrap();
        let fresh = || PolicyAuthorityEngine::new(bundle.clone()).unwrap();
        let gate_actor = Actor {
            actor_id: "cognate-service".into(),
            role: ActorRole::Service,
        };
        let ingress = VerifiedSettlementIngress::verified(
            SettlementIngress::SweSeedDeclaration,
            SettlementTransport::SweSeedAdapter,
            None,
            &gate_actor,
            vec![format!("operation:{run_id}")],
        )
        .unwrap();
        let ttl = chrono::Duration::hours(24);
        let pending = match gate_as(&fresh(), &bundle, root, run_id, &ingress, ttl).unwrap() {
            SettlementGate::Pending { record, .. } => *record,
            other => panic!("expected a pending settlement, got {other:?}"),
        };
        let worlds = WorldRegistry::new();
        let engine = fresh();
        let service = CepAuthorityService {
            engine: &engine,
            bundle: &bundle,
            worlds: &worlds,
            root,
            approval_ttl: ttl,
            criteria: SettlementCriteria::default(),
        };
        let approver = Actor {
            actor_id: "op-approver".into(),
            role: ActorRole::Operator,
        };
        service
            .resolve_approval(
                &settlement_approval_id(&pending.pending_id),
                true,
                &approver,
            )
            .unwrap();
        match gate_as(&fresh(), &bundle, root, run_id, &ingress, ttl).unwrap() {
            SettlementGate::Resumed { pending_id, .. } => {
                assert_eq!(pending_id, pending.pending_id)
            }
            other => panic!("expected a claimed settlement, got {other:?}"),
        }
        (pending.pending_id, pending.request_sha256)
    }

    /// Append a declaration under a claimed attempt, binding it to that claim. The attempt is left open.
    fn append_under_claim(root: &Path, mut decl: SettlementDeclaration) -> SettlementDeclaration {
        let (pending_id, request_sha256) = claim_approved(root, &decl.run_id);
        decl.authorization_ref = Some(pending_id);
        decl.authorization_request_sha256 = Some(request_sha256);
        decl.declaration_hash = sea_forge_settlement::compute_declaration_hash(&decl).unwrap();
        sea_forge_settlement::append_declaration_ledgered_once(
            root,
            &root.join("declarations.jsonl"),
            "test",
            &decl,
        )
        .unwrap();
        decl
    }

    /// The full lifecycle for one declaration: claim, append under the claim, and record its completion.
    fn append_authorized(root: &Path, decl: SettlementDeclaration) -> SettlementDeclaration {
        let decl = append_under_claim(root, decl);
        let pending_id = decl.authorization_ref.clone().unwrap();
        complete_settlement_resume(root, &pending_id, &decl.declaration_id).unwrap();
        decl
    }

    /// Append straight to the case ledger, past the settlement writer's boundary.
    fn out_of_process_append(root: &Path, decl: &SettlementDeclaration) {
        LedgerStream::open(
            root,
            format!("case-{}", decl.case_id),
            "out_of_process_operator",
        )
        .unwrap()
        .commit_typed(
            "settlement_declaration",
            vec![
                decl.case_id.clone(),
                decl.settlement_ref.clone(),
                decl.declaration_id.clone(),
            ],
            decl,
            vec![decl.declarer.authority_ref.clone()],
        )
        .unwrap();
    }

    #[test]
    fn external_declaration_without_provenance_is_refused_by_the_writer() {
        let root = tempfile::tempdir().unwrap();
        let manifest = commit_evidence_and_settlement(
            root.path(),
            "case_a",
            "run_a",
            "item_acp",
            vec!["a:sha256:1".into()],
        );
        let decl = declaration("case_a", "run_a", "item_acp", &manifest, "swe_seed_test");
        let error = sea_forge_settlement::append_declaration_ledgered_once(
            root.path(),
            &root.path().join("declarations.jsonl"),
            "test",
            &decl,
        )
        .unwrap_err();
        assert!(
            matches!(&error, ForgeError::Plan { class, .. } if *class == "settlement_authorization_missing"),
            "{error:?}"
        );
        let written = LedgerStream::open(root.path(), "case-case_a", "test")
            .unwrap()
            .read_entries()
            .unwrap()
            .into_iter()
            .filter(|entry| entry.record_kind == "settlement_declaration")
            .count();
        assert_eq!(written, 0, "a refused declaration is not written");
    }

    fn quarantines(root: &Path, case_id: &str) -> Vec<Value> {
        LedgerStream::open(root, format!("case-{case_id}"), "test")
            .unwrap()
            .read_entries()
            .unwrap()
            .into_iter()
            .filter(|entry| entry.record_kind == "settlement_quarantined")
            .map(|entry| entry.payload)
            .collect()
    }

    #[test]
    fn unauthorised_external_declaration_is_quarantined_at_startup_idempotently() {
        let root = tempfile::tempdir().unwrap();
        let manifest = commit_evidence_and_settlement(
            root.path(),
            "case_a",
            "run_a",
            "item_acp",
            vec!["a:sha256:1".into()],
        );
        // Legacy or out-of-process: a SWE_SEED declaration with no authorisation provenance.
        let decl = declaration("case_a", "run_a", "item_acp", &manifest, "swe_seed_test");
        out_of_process_append(root.path(), &decl);
        reconcile_all_cases(root.path()).unwrap();
        reconcile_all_cases(root.path()).unwrap();

        let found = quarantines(root.path(), "case_a");
        assert_eq!(
            found.len(),
            1,
            "one quarantine, however many reconciliations"
        );
        assert_eq!(found[0]["declaration_id"], json!(decl.declaration_id));
        assert_eq!(found[0]["reason"], json!("missing_authorization"));
        let entries = LedgerStream::open(root.path(), "case-case_a", "test")
            .unwrap()
            .read_entries()
            .unwrap();
        assert!(entries
            .iter()
            .filter(|entry| entry.record_kind == "swe_seed_correlation")
            .all(|entry| !entry.payload.to_string().contains(&decl.declaration_id)));
    }

    #[test]
    fn approval_for_one_run_does_not_authorise_another() {
        let root = tempfile::tempdir().unwrap();
        let manifest_a = commit_evidence_and_settlement(
            root.path(),
            "case_a",
            "run_a",
            "item_a",
            vec!["a:sha256:1".into()],
        );
        let manifest_b = commit_evidence_and_settlement(
            root.path(),
            "case_a",
            "run_b",
            "item_b",
            vec!["b:sha256:2".into()],
        );
        let authorised = append_authorized(
            root.path(),
            declaration("case_a", "run_a", "item_a", &manifest_a, "swe_seed_test"),
        );
        // Splice run_a's approved authorisation onto a declaration for run_b.
        let mut replay = declaration("case_a", "run_b", "item_b", &manifest_b, "swe_seed_test");
        replay.authorization_ref = authorised.authorization_ref.clone();
        replay.authorization_request_sha256 = authorised.authorization_request_sha256.clone();
        replay.declaration_hash = sea_forge_settlement::compute_declaration_hash(&replay).unwrap();
        out_of_process_append(root.path(), &replay);

        let outcomes = reconcile_swe_seed_declarations(root.path(), "case_a").unwrap();
        let run_a = outcomes.iter().find(|o| o.run_id == "run_a").unwrap();
        let run_b = outcomes.iter().find(|o| o.run_id == "run_b").unwrap();
        assert_eq!(
            run_a.declaration_ids,
            vec![authorised.declaration_id.clone()]
        );
        assert!(run_b.declaration_ids.is_empty());
        let found = quarantines(root.path(), "case_a");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0]["declaration_id"], json!(replay.declaration_id));
        assert_eq!(found[0]["reason"], json!("authorization_mismatch"));
    }

    #[test]
    fn local_declarations_keep_their_behaviour_and_are_not_quarantined() {
        let root = tempfile::tempdir().unwrap();
        let manifest = commit_evidence_and_settlement(
            root.path(),
            "case_a",
            "run_a",
            "item_acp",
            vec!["a:sha256:1".into()],
        );
        let decl = local_declaration("case_a", "run_a", "item_acp", &manifest, "local_judge");
        sea_forge_settlement::append_declaration_ledgered_once(
            root.path(),
            &root.path().join("declarations.jsonl"),
            "local",
            &decl,
        )
        .unwrap();
        reconcile_all_cases(root.path()).unwrap();
        assert!(quarantines(root.path(), "case_a").is_empty());
    }

    #[test]
    fn a_declaration_in_another_case_cannot_complete_a_claim() {
        let root = tempfile::tempdir().unwrap();
        let manifest = commit_evidence_and_settlement(
            root.path(),
            "case_a",
            "run_a",
            "item_acp",
            vec!["a:sha256:1".into()],
        );
        // case_a claims run_a under an approval and has no recorded outcome yet.
        let (pending_id, request_sha256) = claim_approved(root.path(), "run_a");
        // A declaration in case_b names case_a's claimed pending record and its exact request hash.
        let mut foreign = declaration("case_b", "run_a", "item_acp", &manifest, "swe_seed_test");
        foreign.authorization_ref = Some(pending_id.clone());
        foreign.authorization_request_sha256 = Some(request_sha256);
        foreign.declaration_hash =
            sea_forge_settlement::compute_declaration_hash(&foreign).unwrap();
        LedgerStream::open(root.path(), "case-case_b", "test")
            .unwrap()
            .commit_typed("settlement_declaration", vec![], &foreign, vec![])
            .unwrap();

        let verdicts = sea_forge_authority::cep::verify_case_declarations(
            root.path(),
            std::slice::from_ref(&foreign),
        )
        .unwrap();
        assert_eq!(
            verdicts,
            vec![DeclarationAuthorization::Quarantined(
                sea_forge_authority::cep::DeclarationRefusal::AuthorizationMismatch
            )]
        );
        // Reconciling case_b must not settle case_a's claim.
        reconcile_operation_claims(root.path(), "case_b", "run_a").unwrap();
        assert!(matches!(
            settlement_status(root.path(), &pending_id).unwrap(),
            SettlementStatus::Resuming { .. }
        ));
    }

    #[test]
    fn interrupted_claim_completes_from_its_authorised_declaration() {
        let root = tempfile::tempdir().unwrap();
        let manifest = commit_evidence_and_settlement(
            root.path(),
            "case_a",
            "run_a",
            "item_acp",
            vec!["a:sha256:1".into()],
        );
        // The declaration is appended, then the process dies before recording the completion.
        let decl = append_under_claim(
            root.path(),
            declaration("case_a", "run_a", "item_acp", &manifest, "swe_seed_test"),
        );
        let pending_id = decl.authorization_ref.clone().unwrap();
        assert!(matches!(
            settlement_status(root.path(), &pending_id).unwrap(),
            SettlementStatus::Resuming { .. }
        ));

        reconcile_operation_claims(root.path(), "case_a", "run_a").unwrap();

        assert_eq!(
            settlement_status(root.path(), &pending_id).unwrap(),
            SettlementStatus::Completed {
                declaration_ref: decl.declaration_id.clone()
            }
        );
        let outcomes = reconcile_swe_seed_declarations(root.path(), "case_a").unwrap();
        let run_a = outcomes.iter().find(|o| o.run_id == "run_a").unwrap();
        assert_eq!(run_a.declaration_ids, vec![decl.declaration_id.clone()]);
    }

    #[test]
    fn interrupted_claim_without_a_declaration_fails_and_is_never_redeclared() {
        let root = tempfile::tempdir().unwrap();
        commit_evidence_and_settlement(
            root.path(),
            "case_a",
            "run_a",
            "item_acp",
            vec!["a:sha256:1".into()],
        );
        let (pending_id, _) = claim_approved(root.path(), "run_a");

        reconcile_operation_claims(root.path(), "case_a", "run_a").unwrap();

        assert!(matches!(
            settlement_status(root.path(), &pending_id).unwrap(),
            SettlementStatus::Failed { .. }
        ));
        let bundle: AuthorityPolicyBundle =
            sea_forge_core::yaml::from_str(APPROVAL_POLICY).unwrap();
        let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
        let gate_actor = Actor {
            actor_id: "cognate-service".into(),
            role: ActorRole::Service,
        };
        let ingress = VerifiedSettlementIngress::verified(
            SettlementIngress::SweSeedDeclaration,
            SettlementTransport::SweSeedAdapter,
            None,
            &gate_actor,
            vec!["operation:run_a".into()],
        )
        .unwrap();
        assert!(gate_as(
            &engine,
            &bundle,
            root.path(),
            "run_a",
            &ingress,
            chrono::Duration::hours(24),
        )
        .is_err());
    }

    #[test]
    fn an_external_declaration_relabelled_local_is_quarantined() {
        let root = tempfile::tempdir().unwrap();
        let manifest = commit_evidence_and_settlement(
            root.path(),
            "case_a",
            "run_a",
            "item_acp",
            vec!["a:sha256:1".into()],
        );
        // An external settlement is pending for the run.
        claim_approved(root.path(), "run_a");
        // A declaration for the same run, presented as local, appended out of process.
        let decl = local_declaration("case_a", "run_a", "item_acp", &manifest, "local_judge");
        out_of_process_append(root.path(), &decl);

        reconcile_all_cases(root.path()).unwrap();

        let found = quarantines(root.path(), "case_a");
        assert!(
            found
                .iter()
                .any(|q| q["declaration_id"] == json!(decl.declaration_id)
                    && q["reason"] == json!("class_mismatch")),
            "{found:?}"
        );
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
        append_authorized(root.path(), decl);
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
        append_authorized(root.path(), decl);
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
        append_authorized(root.path(), decl);
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
        let decl = append_authorized(root.path(), decl);
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
        append_authorized(root.path(), decl);
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
        let decl = append_authorized(root.path(), decl);
        let outcomes = reconcile_swe_seed_declarations(root.path(), "case_a").unwrap();
        let run_a = outcomes.iter().find(|o| o.run_id == "run_a").unwrap();
        let run_b = outcomes.iter().find(|o| o.run_id == "run_b").unwrap();
        assert_eq!(run_a.declaration_ids, vec![decl.declaration_id]);
        assert!(run_b.declaration_ids.is_empty());
    }
}
