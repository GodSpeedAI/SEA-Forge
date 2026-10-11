use sea_forge_artifact_ip::*;
use sea_forge_authority::ActionGrant;
use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::{
    errors::ForgeError,
    types::{
        Actor, ActorRole, ApprovalRequest, ApprovalStatus, ArtifactDescriptor, ArtifactProducer,
        ArtifactStage, ArtifactType, AuthorityAction, CriteriaDerivation, DeclarationIndependence,
        DeclarationReliability, DeclarationStatus, Declarer, DerivationMethod, EvidenceKind,
        EvidenceRecord, OriginRef, OriginRefKind, OriginRole, ReviewStatus, SettlementCriteria,
        SettlementCriteriaRecord, SettlementDeclaration, SettlementStrength,
    },
};
use sea_forge_domainforge::{load_validate, SeaSourceSet, SourceFile};
use sea_forge_ledger::{types::hash_canonical, LedgerStream};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn domain_model_for_anchors() -> sea_forge_domainforge::DomainModel {
    let content = "@namespace \"demo\"\n@version \"1.0.0\"\nEntity \"concept:artifact\" in demo\nResource \"class:artifact\" units in demo\nFlow \"class:artifact\" from \"concept:artifact\" to \"concept:artifact\" quantity 1\n";
    let sha = sha256_hex(content.as_bytes());
    let source_set = SeaSourceSet {
        entry_uri: "demo.sea".into(),
        files: vec![SourceFile {
            uri: "demo.sea".into(),
            sha256: sha.clone(),
            content: content.into(),
        }],
    };
    load_validate(&source_set).unwrap()
}

fn commit_domain_model(
    root: &std::path::Path,
    actor: &str,
    model: &sea_forge_domainforge::DomainModel,
) {
    for source in &model.model_ref.source_refs {
        LedgerStream::open(root, "artifact-ip", actor)
            .unwrap()
            .commit_typed(
                "artifact_evidence",
                vec![format!("evi_source_{}", source.uri)],
                &EvidenceRecord {
                    version: "0.2".into(),
                    evidence_id: format!("evi_source_{}", source.uri),
                    run_id: "run_source".into(),
                    kind: EvidenceKind::Artifact,
                    uri: source.uri.clone(),
                    sha256: Some(source.sha256.clone()),
                    source_event_id: "evt_source".into(),
                    metadata: BTreeMap::new(),
                    cell_id: None,
                },
                vec![],
            )
            .unwrap();
    }
    append_domain_model_ref(root, actor, model).unwrap();
}

fn append_rights_profile_with_authority(
    root: &std::path::Path,
    actor: &str,
    _input: &TransitionInput,
    record: RightsProfileRecord,
) -> Result<RightsProfileRecord, ForgeError> {
    let (ledger_registrations, _) = load_ledger_records(root, actor).unwrap();
    let registration = ledger_registrations
        .iter()
        .find(|r| r.artifact_id == record.artifact_id)
        .unwrap();
    let action = canonical_rights_action(registration, &["approved".into()]);
    let bundle: AuthorityPolicyBundle = serde_yaml::from_str(
        "version: \"0.1\"\nrules:\n  - name: rights\n    verdict: allow\n    actor_role: operator\n    operation_kind: review_artifact_rights\n",
    )
    .unwrap();
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let authority_actor = Actor {
        actor_id: actor.into(),
        role: ActorRole::Operator,
    };
    let decision = engine
        .evaluate(AuthorityEvaluation {
            actor: &authority_actor,
            binding: bundle.resolve_identity(actor, ActorRole::Operator),
            run_id: "rights_review",
            case_id: "rights_review",
            plan_item_id: "rights_review",
            sequence: 1,
            action: &action,
            workspace_root: root,
            evidence_refs: vec![],
            artifacts_root: None,
            timeout_secs: None,
            env_keys: Default::default(),
            domainforge_candidate: None,
            environment: None,
        })
        .unwrap();
    let committed = LedgerStream::open(root, "case-governance", actor)
        .unwrap()
        .commit_typed("authority_decision", vec![], &decision, vec![])
        .unwrap();
    let grant = engine.grant(&decision, &committed, &action, None).unwrap();
    let authorization = authorize_rights_profile(
        grant,
        &action,
        root,
        actor,
        registration,
        &["approved".into()],
        &decision,
        &committed,
    )
    .unwrap();
    append_rights_profile(authorization, record)
}

fn commit_artifact_evidence(
    root: &std::path::Path,
    actor: &str,
    descriptor: ArtifactDescriptor,
) -> EvidenceRecord {
    let ev = evidence(descriptor);
    LedgerStream::open(root, "artifact-ip", actor)
        .unwrap()
        .commit_typed(
            "artifact_evidence",
            vec![ev.evidence_id.clone()],
            &ev,
            vec![],
        )
        .unwrap();
    ev
}

fn descriptor(id: &str, content: &str) -> ArtifactDescriptor {
    ArtifactDescriptor {
        artifact_id: id.into(),
        artifact_type: ArtifactType::SeaModel,
        stage: Some(ArtifactStage::Intellectual),
        producer: ArtifactProducer {
            entity_id: "operator".into(),
            process_id: "cli".into(),
            run_id: "run_source".into(),
            plan_item_id: "item_source".into(),
        },
        owner: "operator".into(),
        license: "internal".into(),
        review_status: ReviewStatus::Approved,
        source_refs: vec!["evi_work_product".into()],
        content_sha256: content.into(),
        pre_mint_identity: "ifl:hash:legacy".into(),
    }
}

fn evidence(descriptor: ArtifactDescriptor) -> EvidenceRecord {
    let evidence_id = format!("evi_{}", descriptor.artifact_id);
    EvidenceRecord {
        version: "0.2".into(),
        evidence_id,
        run_id: "run_source".into(),
        kind: EvidenceKind::Artifact,
        uri: "artifacts/model.sea".into(),
        sha256: Some(descriptor.content_sha256.clone()),
        source_event_id: "evt_1".into(),
        metadata: BTreeMap::from([("artifact".into(), serde_json::to_value(descriptor).unwrap())]),
        cell_id: None,
    }
}

fn profile(kind: TransitionKind) -> ArtifactGateProfile {
    ArtifactGateProfile {
        version: "0.2".into(),
        profile_ref: format!("{kind:?}@1"),
        artifact_types: vec![ArtifactType::SeaModel],
        transition_kind: kind.clone(),
        required_metadata: vec![],
        evaluator_refs: vec!["demo_env.score_eval".into()],
        evaluator_thresholds: BTreeMap::from([("demo_env.score_eval".into(), 1.0)]),
        semantic_reference_classes: (kind == TransitionKind::Capitalize)
            .then_some(SemanticReferenceClass::Concept)
            .into_iter()
            .collect(),
        required_rights_review: vec!["approved".into()],
        required_settlement_strength: (kind == TransitionKind::Capitalize).then(|| "strong".into()),
        qualifying_value_evidence_kinds: vec!["adoption".into()],
        requires_approval: kind == TransitionKind::Capitalize,
        attestation_mode: None,
    }
}

fn input(
    id: &str,
    kind: TransitionKind,
    mode: TransitionMode,
    source: &ArtifactRegistrationRecord,
    result_id: &str,
    output_identity: String,
    parents: Vec<String>,
) -> TransitionInput {
    let (from_stage, to_stage) = match kind {
        TransitionKind::Synthesize => (ArtifactStage::Cognitive, ArtifactStage::Intellectual),
        TransitionKind::Productize => (ArtifactStage::Intellectual, ArtifactStage::Product),
        TransitionKind::Capitalize => (ArtifactStage::Product, ArtifactStage::Capital),
    };
    TransitionInput {
        transition_token_id: id.into(),
        transition_kind: kind.clone(),
        mode: mode.clone(),
        from_stage,
        to_stage,
        source_artifact_ids: vec![source.artifact_id.clone()],
        result_artifact_id: result_id.into(),
        derived_from: if mode == TransitionMode::Derive {
            vec![source.artifact_id.clone()]
        } else {
            vec![]
        },
        input_content_identities: vec![source.content_identity.clone()],
        output_content_identity: output_identity,
        parent_transition_token_ids: parents,
        gate_profile_ref: "profile".into(),
        gate_profile_hash: "sha256:profile".into(),
        actor_id: "requester".into(),
        approver_id: (kind == TransitionKind::Capitalize).then(|| "approver".into()),
        approval_ref: (kind == TransitionKind::Capitalize).then(|| "approval_1".into()),
        authority_decision_refs: vec!["dec_1".into()],
        criteria_ref: "criteria_1".into(),
        evidence_refs: vec!["evi_gate".into()],
        settlement_ref: "settlement_accepted".into(),
        strong_declaration_refs: if kind == TransitionKind::Capitalize {
            vec!["decl_strong".into()]
        } else {
            vec![]
        },
        value_evidence_refs: if kind == TransitionKind::Capitalize {
            vec!["value_adoption".into()]
        } else {
            vec![]
        },
        value_evidence: if kind == TransitionKind::Capitalize {
            vec![ValueEvidenceRef {
                reference: "evi_adoption".into(),
                case_id: "case_other".into(),
                kind: "adoption".into(),
            }]
        } else {
            vec![]
        },
        rights_profile_ref: (kind == TransitionKind::Capitalize).then(|| "rights@1".into()),
        semantic_refs: if kind == TransitionKind::Capitalize {
            vec![SemanticAnchor {
                class: SemanticReferenceClass::Concept,
                reference: "concept:artifact".into(),
            }]
        } else {
            vec![]
        },
        semantic_model_ref: (kind == TransitionKind::Capitalize).then(|| "sha256:semantic".into()),
        identity_status_before: IdentityStatus::PreMint,
        identity_status_after: IdentityStatus::PreMint,
        attestation_ref: None,
        degraded_controls: vec![],
        case_id: "case_source".into(),
        run_id: "run_transition".into(),
        plan_item_id: "transition".into(),
        created_at: "2026-07-15T00:01:00Z".into(),
    }
}

fn ledger_governance(
    root: &std::path::Path,
    input: &mut TransitionInput,
    gate: &ArtifactGateProfile,
) {
    ledger_governance_with_value_kind(root, input, gate, Some("adoption"));
}

fn ledger_governance_with_value_kind(
    root: &std::path::Path,
    input: &mut TransitionInput,
    gate: &ArtifactGateProfile,
    value_kind: Option<&str>,
) {
    ledger_governance_with_options(root, input, gate, value_kind, None);
}

fn ledger_governance_with_options(
    root: &std::path::Path,
    input: &mut TransitionInput,
    gate: &ArtifactGateProfile,
    value_kind: Option<&str>,
    approval_hashes: Option<(Option<String>, Option<String>)>,
) {
    let stream = LedgerStream::open(root, "case-governance", "operator").unwrap();
    let gate_hash = hash_canonical(gate).unwrap();
    LedgerStream::open(root, "artifact-ip", "operator")
        .unwrap()
        .commit_typed(
            "artifact_gate_profile",
            vec![gate.profile_ref.clone()],
            gate,
            vec![],
        )
        .unwrap();
    input.gate_profile_ref.clone_from(&gate.profile_ref);
    input.gate_profile_hash = gate_hash;
    input.authority_decision_refs = vec![format!("auth_{:02}", authority_sequence(input))];
    let criteria = test_criteria_record(input);
    let criteria_sha256 = criteria.criteria_sha256.clone();
    let criteria_record_hash = criteria.criteria_record_hash.clone();
    let (approval_criteria_sha256, approval_criteria_record_hash) =
        approval_hashes.unwrap_or_else(|| {
            (
                Some(criteria_sha256.clone()),
                Some(criteria_record_hash.clone()),
            )
        });
    stream
        .commit_typed_once(
            "settlement_criteria",
            &input.criteria_ref,
            vec![input.criteria_ref.clone()],
            &criteria,
            vec![],
        )
        .unwrap();
    stream
        .commit_typed(
            "settlement_event",
            vec![input.settlement_ref.clone()],
            &serde_json::json!({
                "settlement_id": input.settlement_ref,
                "status": "accepted",
                "run_id": input.run_id,
                "criteria_ref": input.criteria_ref,
                "basis": gate.evaluator_refs.iter().map(|evaluator| {
                    format!("evaluator_score:{evaluator}=1")
                }).collect::<Vec<_>>(),
            }),
            vec![],
        )
        .unwrap();
    LedgerStream::open(root, "artifact-ip", "operator")
        .unwrap()
        .commit_typed(
            "artifact_evidence",
            vec!["evi_gate".into()],
            &EvidenceRecord {
                evidence_id: "evi_gate".into(),
                ..evidence(descriptor("art_gate", "gate"))
            },
            vec![],
        )
        .unwrap();
    if input.transition_kind == TransitionKind::Capitalize {
        let approval = ApprovalRequest {
            approval_id: "approval_1".into(),
            version: "0.2".into(),
            run_id: input.run_id.clone(),
            status: ApprovalStatus::Approved,
            case_id: input.case_id.clone(),
            decision_id: input.authority_decision_refs[0].clone(),
            plan_item_id: "transition".into(),
            criteria_ref: Some(input.criteria_ref.clone()),
            criteria_sha256: approval_criteria_sha256,
            criteria_record_hash: approval_criteria_record_hash,
            job_contract_ref: None,
            requested_at: "2026-07-15T00:00:00Z".into(),
            expires_at: "2099-07-16T00:00:00Z".into(),
            resolved_by: input.approver_id.clone(),
            resolved_at: Some("2026-07-15T00:01:00Z".into()),
            note: None,
        };
        stream
            .commit_typed_once(
                "approval_resolution",
                "approval_1",
                vec!["approval_1".into()],
                &approval,
                vec![],
            )
            .unwrap();
        let declaration = strong_declaration(input, &criteria);
        stream
            .commit_typed(
                "settlement_declaration",
                vec!["decl_strong".into()],
                &declaration,
                vec![],
            )
            .unwrap();
        input.value_evidence.clear();
        input.value_evidence_refs.clear();
        if let Some(kind) = value_kind {
            let source_id = format!("value_{kind}_source");
            let value_id = format!("value_{kind}");
            let source_run_id = format!("run_{kind}_source");
            let source_settlement_ref = format!("settlement_{kind}_source");
            let source_evidence_ref = format!("evi_{kind}_source");
            let source_stream = LedgerStream::open(root, "case-case_other", "operator").unwrap();
            source_stream
                .commit_typed(
                    "run_evidence",
                    vec![source_evidence_ref.clone()],
                    &EvidenceRecord {
                        version: "0.2".into(),
                        evidence_id: source_evidence_ref.clone(),
                        run_id: source_run_id.clone(),
                        kind: EvidenceKind::ExecutionResult,
                        uri: format!("runs/{source_run_id}/evidence.json"),
                        sha256: Some(format!("sha256:{kind}")),
                        source_event_id: format!("evt_{kind}"),
                        metadata: BTreeMap::from([(
                            "canonical_value_evidence_kind".into(),
                            kind.into(),
                        )]),
                        cell_id: None,
                    },
                    vec![],
                )
                .unwrap();
            source_stream
                .commit_typed(
                    "settlement_event",
                    vec![source_settlement_ref.clone()],
                    &serde_json::json!({
                        "version": "0.2",
                        "settlement_id": source_settlement_ref,
                        "run_id": source_run_id,
                        "status": "accepted",
                        "basis": ["source_value_demonstrated"],
                        "review_required": false,
                        "settled_at": "now",
                        "criteria_ref": "criteria_source",
                    }),
                    vec![],
                )
                .unwrap();
            append_value_evidence_source(
                root,
                "operator",
                ValueEvidenceSourceRecord {
                    version: String::new(),
                    evidence_id: source_id.clone(),
                    kind: kind.into(),
                    case_id: "case_other".into(),
                    originating_case_id: input.case_id.clone(),
                    evidence_refs: vec![source_evidence_ref],
                    source_run_id,
                    settlement_ref: source_settlement_ref,
                    created_at: "now".into(),
                    record_hash: String::new(),
                },
            )
            .unwrap();
            append_value_evidence(
                root,
                "operator",
                ValueEvidenceRecord {
                    version: String::new(),
                    value_evidence_id: value_id.clone(),
                    kind: kind.into(),
                    case_id: "case_other".into(),
                    evidence_refs: vec![source_id],
                    accepted: true,
                    created_at: "now".into(),
                    record_hash: String::new(),
                },
            )
            .unwrap();
            input.value_evidence_refs = vec![value_id.clone()];
            input.value_evidence = vec![ValueEvidenceRef {
                reference: value_id,
                case_id: "case_other".into(),
                kind: kind.into(),
            }];
        }
        append_rights_profile_with_authority(
            root,
            "operator",
            input,
            RightsProfileRecord {
                version: String::new(),
                rights_profile_id: "rights@1".into(),
                artifact_id: input.result_artifact_id.clone(),
                license: "internal".into(),
                review_status: ReviewStatus::Approved,
                accepted: true,
                authority_decision_ref: String::new(),
                authority_decision_hash: String::new(),
                authority_action_hash: String::new(),
                created_at: "now".into(),
                record_hash: String::new(),
            },
        )
        .unwrap();
        let model = domain_model_for_anchors();
        commit_domain_model(root, "operator", &model);
        input.semantic_model_ref = Some(model.model_ref.semantic_model_sha256.clone());
    }
    // Note: the caller is responsible for invoking `authorized_transition`
    // after this helper returns. We deliberately do NOT grant here, because
    // re-granting a capitalize (escalate) transition would trip the
    // approval-grant consumption dedup in `grant_after_approval`.
}

fn strong_declaration(
    input: &TransitionInput,
    criteria: &SettlementCriteriaRecord,
) -> SettlementDeclaration {
    let mut declaration = SettlementDeclaration {
        version: "0.2".into(),
        declaration_id: "decl_strong".into(),
        settlement_ref: input.settlement_ref.clone(),
        run_id: input.run_id.clone(),
        case_id: input.case_id.clone(),
        plan_item_id: "transition".into(),
        claim_manifest_sha256: "sha256:claim-manifest".into(),
        status: DeclarationStatus::Accepted,
        strength: SettlementStrength::Strong,
        qualifies_for_capability: true,
        criteria_ref: input.criteria_ref.clone(),
        criteria_sha256: criteria.criteria_sha256.clone(),
        criteria_record_hash: criteria.criteria_record_hash.clone(),
        criteria_declared_at: criteria.declared_at.clone(),
        job_contract_ref: None,
        origin_refs: criteria.origin_refs.clone(),
        verifier_ref: "swe_seed@1".into(),
        verifier_sha256: "sha256:verifier".into(),
        verification_evidence_refs: vec!["evi_gate".into()],
        declarer: Declarer {
            actor_id: "external-declarer".into(),
            authority_ref: "swe_seed".into(),
            role: "R-AA".into(),
            standing_basis: "trusted_external_settlement_authority".into(),
        },
        independence: DeclarationIndependence {
            acting_entity_id: input.actor_id.clone(),
            independent: true,
            basis: "external_declarer_differs_from_actor".into(),
        },
        reliability: DeclarationReliability {
            feedback_delay_ms: 1,
            attribution_confidence: "1.000000".into(),
            gaming_exposure: "0.000000".into(),
            hidden_debt_blindness: "0.100000".into(),
            weight: "0.900000".into(),
            basis: "swe_seed_adapter".into(),
        },
        variation_tags: BTreeMap::new(),
        disruption_tags: vec![],
        orchestration_burden: None,
        issued_at: "2026-07-15T00:01:00Z".into(),
        source_evidence_refs: vec!["evi_gate".into()],
        adapter_attestation_ref: Some("attestation:swe-seed:1".into()),
        authored_by: None,
        declaration_hash: String::new(),
    };
    declaration.declaration_hash = hash_canonical(&declaration).unwrap();
    declaration
}

fn test_criteria_record(input: &TransitionInput) -> SettlementCriteriaRecord {
    let criteria = SettlementCriteria {
        require_exit_zero: true,
        ..Default::default()
    };
    let mut record = SettlementCriteriaRecord {
        version: "0.2".into(),
        criteria_id: input.criteria_ref.clone(),
        criteria_sha256: hash_canonical(&criteria).unwrap(),
        criteria,
        origin_refs: vec![OriginRef {
            kind: OriginRefKind::ImplementationDefined,
            reference: "m8-test-transition".into(),
            sha256: "sha256:origin".into(),
            role: OriginRole::AcceptanceSource,
            evidence_refs: vec![],
            domain_model_ref: None,
        }],
        derivation: CriteriaDerivation {
            method: DerivationMethod::ImplementationDefined,
            actor_ref: input.actor_id.clone(),
            producer_ref: None,
            rationale: "M8 conformance fixture".into(),
        },
        declared_at: "2026-07-15T00:00:00Z".into(),
        criteria_record_hash: String::new(),
    };
    record.criteria_record_hash = hash_canonical(&record).unwrap();
    record
}

fn authorized_transition(
    root: &std::path::Path,
    input: &TransitionInput,
    registrations: &[ArtifactRegistrationRecord],
    gate: &ArtifactGateProfile,
) -> AuthorizedTransition {
    let licenses = input
        .source_artifact_ids
        .iter()
        .map(|source| {
            registrations
                .iter()
                .find(|registration| registration.artifact_id == *source)
                .unwrap()
                .license
                .clone()
        })
        .collect::<Vec<_>>();
    let transition_kind = match input.transition_kind {
        TransitionKind::Synthesize => "synthesize",
        TransitionKind::Productize => "productize",
        TransitionKind::Capitalize => "capitalize",
    };
    let from_stage = match input.from_stage {
        ArtifactStage::Cognitive => "cognitive",
        ArtifactStage::Intellectual => "intellectual",
        ArtifactStage::Product => "product",
        ArtifactStage::Capital => "capital",
    };
    let to_stage = match input.to_stage {
        ArtifactStage::Cognitive => "cognitive",
        ArtifactStage::Intellectual => "intellectual",
        ArtifactStage::Product => "product",
        ArtifactStage::Capital => "capital",
    };
    let mode = match input.mode {
        TransitionMode::Derive => "derive",
        TransitionMode::Promote => "promote",
    };
    let evidence = format!("[{}]", gate.qualifying_value_evidence_kinds.join(","));
    let license_allowlist = format!("[{}]", licenses.join(","));
    let policy = format!(
        "version: \"0.1\"\nrules:\n  - name: transition\n    verdict: allow\n    actor_role: operator\n    operation_kind: transition_artifact_stage\n    transition_kind: {transition_kind}\n    from_stage: {from_stage}\n    to_stage: {to_stage}\n    modes: [{mode}]\n    gate_profile_ref: {}\n    required_settlement_strength: {}\n    qualifying_value_evidence_kinds: {evidence}\n    requires_approval: {}\n    license_allowlist: {license_allowlist}\n",
        gate.profile_ref,
        gate.required_settlement_strength.as_deref().unwrap_or("local"),
        gate.requires_approval,
    );
    let bundle: AuthorityPolicyBundle = serde_yaml::from_str(&policy).unwrap();
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let actor = Actor {
        actor_id: input.actor_id.clone(),
        role: ActorRole::Operator,
    };
    let action = canonical_transition_action(input, &licenses, gate);
    let decision = engine
        .evaluate(AuthorityEvaluation {
            actor: &actor,
            binding: bundle.resolve_identity(&input.actor_id, ActorRole::Operator),
            run_id: &input.run_id,
            case_id: &input.case_id,
            plan_item_id: "transition",
            sequence: authority_sequence(input),
            action: &action,
            workspace_root: root,
            evidence_refs: vec![],
            artifacts_root: None,
            timeout_secs: None,
            env_keys: Default::default(),
            domainforge_candidate: None,
            environment: None,
        })
        .unwrap();
    let decision_id = decision.decision_id.clone();
    let stream = LedgerStream::open(root, "case-governance", &input.actor_id).unwrap();
    let committed = stream
        .commit_typed("authority_decision", vec![], &decision, vec![])
        .unwrap();
    let grant = if decision.verdict == sea_forge_core::types::Verdict::Escalate {
        let criteria = test_criteria_record(input);
        let committed_criteria = stream
            .commit_typed_once(
                "settlement_criteria",
                &input.criteria_ref,
                vec![input.criteria_ref.clone()],
                &criteria,
                vec![],
            )
            .unwrap();
        let approval = ApprovalRequest {
            version: "0.2".into(),
            approval_id: "approval_1".into(),
            run_id: input.run_id.clone(),
            case_id: input.case_id.clone(),
            decision_id: decision_id.clone(),
            plan_item_id: "transition".into(),
            criteria_ref: Some(input.criteria_ref.clone()),
            criteria_sha256: Some(criteria.criteria_sha256.clone()),
            criteria_record_hash: Some(criteria.criteria_record_hash.clone()),
            job_contract_ref: None,
            requested_at: "2026-07-15T00:00:00Z".into(),
            expires_at: "2099-07-16T00:00:00Z".into(),
            status: ApprovalStatus::Approved,
            resolved_by: input.approver_id.clone(),
            resolved_at: Some("2026-07-15T00:01:00Z".into()),
            note: None,
        };
        let committed_approval = stream
            .commit_typed_once(
                "approval_resolution",
                "approval_1",
                vec!["approval_1".into()],
                &approval,
                vec![],
            )
            .unwrap();
        engine
            .grant_after_approval(
                &decision,
                &committed,
                &action,
                &approval,
                &committed_approval,
                &criteria,
                &committed_criteria,
                None,
                &stream,
            )
            .unwrap()
    } else {
        engine.grant(&decision, &committed, &action, None).unwrap()
    };
    authorize_transition(
        grant,
        &action,
        root,
        &input.actor_id,
        input,
        &licenses,
        gate,
        decision_id,
        &input.run_id,
        &input.case_id,
        "transition",
    )
    .unwrap()
}

fn authority_sequence(input: &TransitionInput) -> usize {
    match input.transition_kind {
        TransitionKind::Synthesize => 1,
        TransitionKind::Productize => 2,
        TransitionKind::Capitalize => 3,
    }
}

fn ledgered_chain_through_product(
    root: &std::path::Path,
    artifact_id: &str,
) -> (ArtifactRegistrationRecord, Vec<TransitionToken>) {
    let registration = append_registration(
        root,
        "operator",
        &commit_artifact_evidence(root, "operator", descriptor(artifact_id, artifact_id))
            .evidence_id,
        VerifiedArtifactRegistrationInput {
            name: format!("{artifact_id}.sea"),
            version: "1".into(),
            case_id: "case_source".into(),
            derived_from: vec![],
            created_at: "now".into(),
        },
    )
    .unwrap();
    let registrations = std::slice::from_ref(&registration);
    let mut tokens = Vec::new();
    for (id, kind) in [
        (
            format!("tok_{artifact_id}_intellectual"),
            TransitionKind::Synthesize,
        ),
        (
            format!("tok_{artifact_id}_product"),
            TransitionKind::Productize,
        ),
    ] {
        let mut transition_input = input(
            &id,
            kind.clone(),
            TransitionMode::Promote,
            &registration,
            &registration.artifact_id,
            registration.content_identity.clone(),
            tokens
                .last()
                .map(|parent: &TransitionToken| vec![parent.transition_token_id.clone()])
                .unwrap_or_default(),
        );
        let gate = profile(kind);
        ledger_governance(root, &mut transition_input, &gate);
        let token = append_transition(
            authorized_transition(root, &transition_input, registrations, &gate),
            transition_input,
            registrations,
            &tokens,
            &gate,
        )
        .unwrap();
        tokens.push(token);
    }
    (registration, tokens)
}

fn attempt_capital(
    root: &std::path::Path,
    registration: &ArtifactRegistrationRecord,
    tokens: &[TransitionToken],
    gate: &ArtifactGateProfile,
    value_kind: Option<&str>,
) -> Result<TransitionToken, ForgeError> {
    let mut capital_input = input(
        &format!("tok_{}_capital", registration.artifact_id),
        TransitionKind::Capitalize,
        TransitionMode::Promote,
        registration,
        &registration.artifact_id,
        registration.content_identity.clone(),
        vec![tokens.last().unwrap().transition_token_id.clone()],
    );
    ledger_governance_with_value_kind(root, &mut capital_input, gate, value_kind);
    append_transition(
        authorized_transition(
            root,
            &capital_input,
            std::slice::from_ref(registration),
            gate,
        ),
        capital_input,
        std::slice::from_ref(registration),
        tokens,
        gate,
    )
}

fn attempt_synthesize_with_metadata(
    required_metadata: Vec<String>,
    product_contract: serde_json::Value,
) -> Result<TransitionToken, ForgeError> {
    let root = tempfile::tempdir().unwrap();
    let artifact_descriptor = descriptor("art_metadata", "metadata");
    let mut artifact_evidence = evidence(artifact_descriptor);
    artifact_evidence
        .metadata
        .insert("product_contract".into(), product_contract);
    LedgerStream::open(root.path(), "artifact-ip", "operator")
        .unwrap()
        .commit_typed(
            "artifact_evidence",
            vec![artifact_evidence.evidence_id.clone()],
            &artifact_evidence,
            vec![],
        )
        .unwrap();
    let registration = append_registration(
        root.path(),
        "operator",
        &artifact_evidence.evidence_id,
        VerifiedArtifactRegistrationInput {
            name: "metadata.sea".into(),
            version: "1".into(),
            case_id: "case_source".into(),
            derived_from: vec![],
            created_at: "now".into(),
        },
    )
    .unwrap();
    let mut gate = profile(TransitionKind::Synthesize);
    gate.required_metadata = required_metadata;
    let mut transition_input = input(
        "tok_metadata",
        TransitionKind::Synthesize,
        TransitionMode::Promote,
        &registration,
        &registration.artifact_id,
        registration.content_identity.clone(),
        vec![],
    );
    ledger_governance(root.path(), &mut transition_input, &gate);
    append_transition(
        authorized_transition(
            root.path(),
            &transition_input,
            std::slice::from_ref(&registration),
            &gate,
        ),
        transition_input,
        std::slice::from_ref(&registration),
        &[],
        &gate,
    )
}

#[test]
fn conformance_m8_required_metadata_rejects_missing_or_empty_product_contract_field() {
    for product_contract in [
        serde_json::json!({}),
        serde_json::json!({"job_to_be_done": null}),
        serde_json::json!({"job_to_be_done": "   "}),
    ] {
        let error = attempt_synthesize_with_metadata(
            vec!["product_contract.job_to_be_done".into()],
            product_contract,
        )
        .unwrap_err();
        assert_eq!(error.class(), "artifact_gate_error");
    }
}

#[test]
fn conformance_m8_required_metadata_rejects_unknown_selector() {
    for selector in ["", "product_contract.future_field"] {
        let error = attempt_synthesize_with_metadata(
            vec![selector.into()],
            serde_json::json!({"future_field": "untyped"}),
        )
        .unwrap_err();
        assert_eq!(error.class(), "artifact_gate_error");
    }
}

fn attempt_capital_with_semantics(
    classes: Vec<SemanticReferenceClass>,
    anchors: Vec<SemanticAnchor>,
) -> Result<TransitionToken, ForgeError> {
    let root = tempfile::tempdir().unwrap();
    let (registration, tokens) = ledgered_chain_through_product(root.path(), "art_semantic");
    let mut gate = profile(TransitionKind::Capitalize);
    gate.semantic_reference_classes = classes;
    let mut capital_input = input(
        "tok_semantic_capital",
        TransitionKind::Capitalize,
        TransitionMode::Promote,
        &registration,
        &registration.artifact_id,
        registration.content_identity.clone(),
        vec![tokens.last().unwrap().transition_token_id.clone()],
    );
    capital_input.semantic_refs = anchors;
    ledger_governance(root.path(), &mut capital_input, &gate);
    append_transition(
        authorized_transition(
            root.path(),
            &capital_input,
            std::slice::from_ref(&registration),
            &gate,
        ),
        capital_input,
        std::slice::from_ref(&registration),
        &tokens,
        &gate,
    )
}

#[test]
fn conformance_m8_semantic_anchor_class_must_match_gate() {
    let error = attempt_capital_with_semantics(
        vec![SemanticReferenceClass::Class],
        vec![SemanticAnchor {
            class: SemanticReferenceClass::Concept,
            reference: "concept:artifact".into(),
        }],
    )
    .unwrap_err();
    assert_eq!(error.class(), "artifact_gate_error");
}

#[test]
fn conformance_m8_semantic_anchor_must_resolve_in_ledgered_domain_model() {
    let error = attempt_capital_with_semantics(
        vec![SemanticReferenceClass::Concept],
        vec![SemanticAnchor {
            class: SemanticReferenceClass::Concept,
            reference: "concept:unknown".into(),
        }],
    )
    .unwrap_err();
    assert_eq!(error.class(), "artifact_gate_error");
}

fn retire_artifact(root: &std::path::Path, artifact_id: &str) -> ArtifactLifecycleRecord {
    let input = ArtifactLifecycleInput {
        artifact_id: artifact_id.into(),
        previous_status: LifecycleStatus::Active,
        new_status: LifecycleStatus::Retired,
        reason: "replaced by supported release".into(),
        changed_at: "2026-07-15T12:00:00Z".into(),
        actor_id: "operator".into(),
        case_id: "case_lifecycle".into(),
        run_id: "run_lifecycle".into(),
        plan_item_id: "lifecycle".into(),
    };
    let action = canonical_lifecycle_action(&input);
    let bundle: AuthorityPolicyBundle = serde_yaml::from_str(
        "version: \"0.1\"\nrules:\n  - name: lifecycle\n    verdict: allow\n    actor_role: operator\n    operation_kind: artifact_lifecycle\n",
    )
    .unwrap();
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let actor = Actor {
        actor_id: input.actor_id.clone(),
        role: ActorRole::Operator,
    };
    let decision = engine
        .evaluate(AuthorityEvaluation {
            actor: &actor,
            binding: bundle.resolve_identity(&input.actor_id, ActorRole::Operator),
            run_id: &input.run_id,
            case_id: &input.case_id,
            plan_item_id: &input.plan_item_id,
            sequence: 1,
            action: &action,
            workspace_root: root,
            evidence_refs: vec![],
            artifacts_root: None,
            timeout_secs: None,
            env_keys: Default::default(),
            domainforge_candidate: None,
            environment: None,
        })
        .unwrap();
    let committed = LedgerStream::open(root, "authority-lifecycle", "operator")
        .unwrap()
        .commit_typed("authority_decision", vec![], &decision, vec![])
        .unwrap();
    let grant = engine.grant(&decision, &committed, &action, None).unwrap();
    append_lifecycle(
        authorize_lifecycle(
            grant, &action, root, "operator", &input, &decision, &committed,
        )
        .unwrap(),
        input,
    )
    .unwrap()
}

#[test]
fn conformance_m8_retirement_preserves_capital_maturity_and_ignores_forged_view() {
    let root = tempfile::tempdir().unwrap();
    let (registration, tokens) = ledgered_chain_through_product(root.path(), "art_retired_capital");
    attempt_capital(
        root.path(),
        &registration,
        &tokens,
        &profile(TransitionKind::Capitalize),
        Some("adoption"),
    )
    .unwrap();
    let lifecycle = retire_artifact(root.path(), &registration.artifact_id);
    assert_eq!(lifecycle.previous_status, LifecycleStatus::Active);
    assert_eq!(lifecycle.new_status, LifecycleStatus::Retired);

    std::fs::create_dir_all(root.path().join("artifacts")).unwrap();
    std::fs::write(
        root.path().join("artifacts/catalog.jsonl"),
        serde_json::json!({
            "artifact_id": registration.artifact_id,
            "recognized_stage": "cognitive",
            "lifecycle_status": "active"
        })
        .to_string(),
    )
    .unwrap();
    let rebuilt = rebuild_materialized_views(root.path(), "operator").unwrap();
    let state = &rebuilt[&registration.artifact_id];
    assert_eq!(state.recognized_stage, ArtifactStage::Capital);
    assert_eq!(state.lifecycle_status, LifecycleStatus::Retired);
}

fn assert_capital_projection(root: &std::path::Path, artifact_id: &str, expected: bool) {
    let states = rebuild_materialized_views(root, "operator").unwrap();
    assert_eq!(
        states[artifact_id].recognized_stage == ArtifactStage::Capital,
        expected
    );
    assert_eq!(
        root.join("ip/capital")
            .join(format!("{artifact_id}.json"))
            .exists(),
        expected
    );
}

#[test]
fn conformance_m8_transition_proposal_rejects_caller_governance_fields() {
    let proposal = serde_json::json!({
        "transition_token_id": "tok_pending",
        "transition_kind": "capitalize",
        "mode": "promote",
        "from_stage": "product",
        "to_stage": "capital",
        "source_artifact_ids": ["art_product"],
        "result_artifact_id": "art_product",
        "derived_from": [],
        "input_content_identities": ["sha256:content"],
        "output_content_identity": "sha256:content",
        "parent_transition_token_ids": ["tok_product"],
        "gate_profile_ref": "capitalize@1",
        "gate_profile_hash": "sha256:profile",
        "value_evidence_refs": ["value_adoption"],
        "value_evidence": [{"reference":"value_adoption","case_id":"case_other","kind":"adoption"}],
        "rights_profile_ref": "rights@1",
        "semantic_refs": [{"class": "concept", "reference": "concept:artifact"}],
        "semantic_model_ref": "sha256:semantic",
        "identity_status_before": "pre_mint",
        "identity_status_after": "pre_mint",
        "attestation_ref": null,
        "degraded_controls": []
    });

    for field in [
        "actor_id",
        "approver_id",
        "approval_ref",
        "authority_decision_refs",
        "criteria_ref",
        "evidence_refs",
        "settlement_ref",
        "strong_declaration_refs",
        "case_id",
        "run_id",
        "created_at",
    ] {
        let mut hostile = proposal.clone();
        hostile[field] = serde_json::json!("caller-controlled");
        assert!(
            serde_json::from_value::<TransitionProposal>(hostile).is_err(),
            "caller governance field {field} must be rejected"
        );
    }
}

#[test]
fn conformance_m8_pending_transition_is_idempotent_and_terminal_is_final() {
    let root = tempfile::tempdir().unwrap();
    let proposal = TransitionProposal::from(&input(
        "tok_pending",
        TransitionKind::Capitalize,
        TransitionMode::Promote,
        &register_verified(
            &evidence(descriptor("art_pending", "pending")),
            "pending.sea".into(),
            "1".into(),
            "case_source".into(),
            vec![],
            "now".into(),
        )
        .unwrap(),
        "art_pending",
        content_identity(&ArtifactType::SeaModel, "pending").unwrap(),
        vec!["tok_product".into()],
    ));
    let mut pending = PendingArtifactTransition {
        version: M8_RECORD_VERSION.into(),
        pending_key: String::new(),
        case_id: "case_pending".into(),
        run_id: "run_pending".into(),
        plan_item_id: "transition".into(),
        criteria_ref: "criteria_pending".into(),
        criteria_sha256: "sha256:criteria".into(),
        criteria_record_hash: "sha256:criteria-record".into(),
        authority_decision_ref: "auth_01".into(),
        authority_decision_hash: "sha256:decision".into(),
        authority_action_hash: "sha256:action".into(),
        approval_ref: "apr_0001".into(),
        proposal,
        proposal_snapshot_hash: String::new(),
        profile_hash: "sha256:profile".into(),
        created_at: "2026-07-15T00:00:00Z".into(),
    };
    seal_pending_transition(&mut pending).unwrap();

    let first = commit_pending_transition(root.path(), "operator", &pending).unwrap();
    let replay = commit_pending_transition(root.path(), "operator", &pending).unwrap();
    assert_eq!(first.entry_ulid(), replay.entry_ulid());
    assert_eq!(
        load_pending_transition(root.path(), "operator", "case_pending")
            .unwrap()
            .pending_key,
        pending.pending_key
    );

    let mut conflict = pending.clone();
    conflict.approval_ref = "apr_other".into();
    assert!(commit_pending_transition(root.path(), "operator", &conflict).is_err());

    commit_transition_terminal(
        root.path(),
        "operator",
        &ArtifactTransitionTerminal {
            version: M8_RECORD_VERSION.into(),
            pending_key: pending.pending_key,
            case_id: "case_pending".into(),
            run_id: "run_pending".into(),
            status: ArtifactTransitionTerminalStatus::Rejected,
            reason: ArtifactTransitionTerminalReason::ApprovalRejected,
            transition_token_ref: None,
            declaration_ref: None,
            terminal_at: "2026-07-15T00:01:00Z".into(),
        },
    )
    .unwrap();
    assert!(load_pending_transition(root.path(), "operator", "case_pending").is_err());
}

#[test]
fn conformance_m8_registration_only_accepts_verified_descriptor_evidence() {
    let original = descriptor("art_model", "abc");
    let registration = register_verified(
        &evidence(original.clone()),
        "model.sea".into(),
        "1".into(),
        "case_source".into(),
        vec![],
        "now".into(),
    )
    .unwrap();
    assert_eq!(
        registration.declared_stage,
        Some(ArtifactStage::Intellectual)
    );
    assert_eq!(registration.recognized_stage(), ArtifactStage::Cognitive);
    assert_eq!(
        registration.legacy_pre_mint_identity,
        original.pre_mint_identity
    );
    assert_eq!(
        registration.content_identity,
        content_identity(&ArtifactType::SeaModel, "abc").unwrap()
    );

    let mut stdout = evidence(original);
    stdout.kind = EvidenceKind::ExecutionResult;
    assert!(register_verified(
        &stdout,
        "x".into(),
        "1".into(),
        "case".into(),
        vec![],
        "now".into()
    )
    .is_err());
    let mut arbitrary = evidence(descriptor("art_other", "def"));
    arbitrary.uri = "stdout.txt".into();
    assert!(register_verified(
        &arbitrary,
        "x".into(),
        "1".into(),
        "case".into(),
        vec![],
        "now".into()
    )
    .is_err());
}

fn derive_input_with_two_sources() -> (TransitionInput, Vec<ArtifactRegistrationRecord>) {
    let first = register_verified(
        &evidence(descriptor("art_derive_a", "derive-a")),
        "derive-a.sea".into(),
        "1".into(),
        "case_source".into(),
        vec![],
        "now".into(),
    )
    .unwrap();
    let second = register_verified(
        &evidence(descriptor("art_derive_b", "derive-b")),
        "derive-b.sea".into(),
        "1".into(),
        "case_source".into(),
        vec![],
        "now".into(),
    )
    .unwrap();
    let mut derived = input(
        "tok_derive",
        TransitionKind::Synthesize,
        TransitionMode::Derive,
        &first,
        "art_derive_result",
        "sha256:derive-result".into(),
        vec![],
    );
    derived.source_artifact_ids = vec![first.artifact_id.clone(), second.artifact_id.clone()];
    derived.input_content_identities = vec![
        first.content_identity.clone(),
        second.content_identity.clone(),
    ];
    derived.derived_from = derived.source_artifact_ids.clone();
    (derived, vec![first, second])
}

#[test]
fn conformance_m8_domain_model_ref_requires_real_validated_model() {
    let root = tempfile::tempdir().unwrap();
    // Hostile: caller-constructed DomainModelRef with empty validation
    // evidence and empty source_refs must be rejected. Only a real DomainModel
    // from load_validate may be ledgered.
    let model = domain_model_for_anchors();
    // Source not committed yet → must fail.
    let error = append_domain_model_ref(root.path(), "operator", &model).unwrap_err();
    assert_eq!(error.class(), "artifact_reference_error");
}

#[test]
fn conformance_m8_registration_resolves_evidence_from_verified_ledger_not_caller() {
    let root = tempfile::tempdir().unwrap();
    // append_registration must reject an evidence_id that is not ledgered;
    // it may not accept or commit caller-supplied evidence.
    let error = append_registration(
        root.path(),
        "operator",
        "evi_unledgered",
        VerifiedArtifactRegistrationInput {
            name: "unledgered.sea".into(),
            version: "1".into(),
            case_id: "case_source".into(),
            derived_from: vec![],
            created_at: "now".into(),
        },
    )
    .unwrap_err();
    assert_eq!(error.class(), "artifact_registration_error");
    // No artifact-ip stream should have been created.
    assert!(!root
        .path()
        .join("ledgers/artifact-ip/entries.jsonl")
        .exists());
}

#[test]
fn conformance_m8_rights_profile_fabricated_accepted_without_authority_is_rejected() {
    let root = tempfile::tempdir().unwrap();
    let (registration, tokens) = ledgered_chain_through_product(root.path(), "art_rights_fab");
    // Commit a rights profile directly with accepted=true but NO authority
    // binding, bypassing append_rights_profile.
    let mut rights = RightsProfileRecord {
        version: M8_RECORD_VERSION.into(),
        rights_profile_id: "rights_fab".into(),
        artifact_id: registration.artifact_id.clone(),
        license: "internal".into(),
        review_status: ReviewStatus::Approved,
        accepted: true,
        authority_decision_ref: String::new(),
        authority_decision_hash: String::new(),
        authority_action_hash: String::new(),
        created_at: "now".into(),
        record_hash: String::new(),
    };
    rights.record_hash = hash_canonical(&rights).unwrap();
    LedgerStream::open(root.path(), "artifact-ip", "operator")
        .unwrap()
        .commit_typed(
            "artifact_rights_profile",
            vec!["rights_fab".into()],
            &rights,
            vec![],
        )
        .unwrap();
    // Build capital input that references the fabricated rights profile.
    let mut capital_input = input(
        "tok_rights_fab_capital",
        TransitionKind::Capitalize,
        TransitionMode::Promote,
        &registration,
        &registration.artifact_id,
        registration.content_identity.clone(),
        vec![tokens.last().unwrap().transition_token_id.clone()],
    );
    capital_input.rights_profile_ref = Some("rights_fab".into());
    let gate = profile(TransitionKind::Capitalize);
    ledger_governance_with_value_kind(root.path(), &mut capital_input, &gate, Some("adoption"));
    // Override the value evidence and semantic refs that governance set up.
    capital_input.rights_profile_ref = Some("rights_fab".into());
    // Attempt transition through the governance path — rebuild in
    // validate_ledger_references must reject the fabricated rights profile.
    let registrations = std::slice::from_ref(&registration);
    let authorization = authorized_transition(root.path(), &capital_input, registrations, &gate);
    let error =
        append_transition(authorization, capital_input, registrations, &tokens, &gate).unwrap_err();
    assert_eq!(error.class(), "artifact_reference_error");
}

#[test]
fn conformance_m8_derive_rejects_source_identity_count_mismatch() {
    let (mut derived, _) = derive_input_with_two_sources();
    derived.input_content_identities.pop();
    assert_eq!(
        transition(derived).unwrap_err().class(),
        "artifact_transition_error"
    );
}

#[test]
fn conformance_m8_derive_rejects_source_identity_pair_mismatch() {
    let (mut derived, registrations) = derive_input_with_two_sources();
    derived.input_content_identities.swap(0, 1);
    let error = validate_before_transform(
        &derived,
        &registrations,
        &[],
        &profile(TransitionKind::Synthesize),
        || Ok(()),
    )
    .unwrap_err();
    assert_eq!(error.class(), "artifact_transition_error");
}

#[test]
fn conformance_m8_derive_rejects_result_artifact_reusing_any_source_id() {
    let (mut derived, _) = derive_input_with_two_sources();
    derived.result_artifact_id = derived.source_artifact_ids[1].clone();
    assert_eq!(
        transition(derived).unwrap_err().class(),
        "artifact_transition_error"
    );
}

#[test]
fn conformance_m8_derive_rejects_incomplete_derived_from() {
    let (mut derived, _) = derive_input_with_two_sources();
    derived.derived_from.pop();
    assert_eq!(
        transition(derived).unwrap_err().class(),
        "artifact_transition_error"
    );
}

#[test]
fn conformance_m8_derive_rejects_output_identity_matching_any_input() {
    let (mut derived, _) = derive_input_with_two_sources();
    derived.output_content_identity = derived.input_content_identities[1].clone();
    assert_eq!(
        transition(derived).unwrap_err().class(),
        "artifact_transition_error"
    );
}

#[test]
fn conformance_m8_promote_derive_product_capital_and_rebuild_are_governed() {
    let root = tempfile::tempdir().unwrap();
    let cognitive = append_registration(
        root.path(),
        "operator",
        &commit_artifact_evidence(root.path(), "operator", descriptor("art_model", "abc"))
            .evidence_id,
        VerifiedArtifactRegistrationInput {
            name: "model.sea".into(),
            version: "1".into(),
            case_id: "case_source".into(),
            derived_from: vec![],
            created_at: "now".into(),
        },
    )
    .unwrap();
    let intellectual = transition(input(
        "tok_int",
        TransitionKind::Synthesize,
        TransitionMode::Promote,
        &cognitive,
        &cognitive.artifact_id,
        cognitive.content_identity.clone(),
        vec![],
    ))
    .unwrap();
    let product = append_registration(
        root.path(),
        "operator",
        &commit_artifact_evidence(root.path(), "operator", descriptor("art_product", "def"))
            .evidence_id,
        VerifiedArtifactRegistrationInput {
            name: "package".into(),
            version: "2".into(),
            case_id: "case_source".into(),
            derived_from: vec![cognitive.artifact_id.clone()],
            created_at: "now".into(),
        },
    )
    .unwrap();
    let product_token = transition(input(
        "tok_product",
        TransitionKind::Productize,
        TransitionMode::Derive,
        &cognitive,
        &product.artifact_id,
        product.content_identity.clone(),
        vec!["tok_int".into()],
    ))
    .unwrap();
    let capital = transition(input(
        "tok_capital",
        TransitionKind::Capitalize,
        TransitionMode::Promote,
        &product,
        &product.artifact_id,
        product.content_identity.clone(),
        vec!["tok_product".into()],
    ))
    .unwrap();

    let state = rebuild(
        &[cognitive.clone(), product.clone()],
        &[capital.clone(), product_token.clone(), intellectual.clone()],
    )
    .unwrap();
    assert_eq!(
        state[&cognitive.artifact_id].recognized_stage,
        ArtifactStage::Intellectual
    );
    assert_eq!(
        state[&product.artifact_id].recognized_stage,
        ArtifactStage::Capital
    );

    let mut intellectual_input = input(
        "tok_int",
        TransitionKind::Synthesize,
        TransitionMode::Promote,
        &cognitive,
        &cognitive.artifact_id,
        cognitive.content_identity.clone(),
        vec![],
    );
    let intellectual_profile = profile(TransitionKind::Synthesize);
    ledger_governance(root.path(), &mut intellectual_input, &intellectual_profile);
    let intellectual_authorization = authorized_transition(
        root.path(),
        &intellectual_input,
        &[cognitive.clone(), product.clone()],
        &intellectual_profile,
    );
    let ledger_intellectual = append_transition(
        intellectual_authorization,
        intellectual_input,
        &[cognitive.clone(), product.clone()],
        &[],
        &intellectual_profile,
    )
    .unwrap();
    let mut product_input = input(
        "tok_product",
        TransitionKind::Productize,
        TransitionMode::Derive,
        &cognitive,
        &product.artifact_id,
        product.content_identity.clone(),
        vec!["tok_int".into()],
    );
    let product_profile = profile(TransitionKind::Productize);
    ledger_governance(root.path(), &mut product_input, &product_profile);
    let product_authorization = authorized_transition(
        root.path(),
        &product_input,
        &[cognitive.clone(), product.clone()],
        &product_profile,
    );
    let ledger_product = append_transition(
        product_authorization,
        product_input,
        &[cognitive.clone(), product.clone()],
        std::slice::from_ref(&ledger_intellectual),
        &product_profile,
    )
    .unwrap();
    let mut capital_input = input(
        "tok_capital",
        TransitionKind::Capitalize,
        TransitionMode::Promote,
        &product,
        &product.artifact_id,
        product.content_identity.clone(),
        vec!["tok_product".into()],
    );
    let capital_profile = profile(TransitionKind::Capitalize);
    ledger_governance(root.path(), &mut capital_input, &capital_profile);
    let capital_authorization = authorized_transition(
        root.path(),
        &capital_input,
        &[cognitive.clone(), product.clone()],
        &capital_profile,
    );
    append_transition(
        capital_authorization,
        capital_input,
        &[cognitive.clone(), product.clone()],
        &[ledger_intellectual, ledger_product],
        &capital_profile,
    )
    .unwrap();
    std::fs::create_dir_all(root.path().join("artifacts")).unwrap();
    std::fs::write(root.path().join("artifacts/catalog.jsonl"), "forged").unwrap();
    std::fs::write(root.path().join("artifacts/state.json"), "forged").unwrap();
    let rebuilt = rebuild_materialized_views(root.path(), "operator").unwrap();
    assert_eq!(
        rebuilt[&product.artifact_id].recognized_stage,
        ArtifactStage::Capital
    );
    assert_ne!(
        std::fs::read_to_string(root.path().join("artifacts/catalog.jsonl")).unwrap(),
        "forged"
    );
}

#[test]
fn conformance_m8_rejects_teleport_before_transform_and_dag_corruption() {
    let cognitive = register_verified(
        &evidence(descriptor("art_model", "abc")),
        "model".into(),
        "1".into(),
        "case_source".into(),
        vec![],
        "now".into(),
    )
    .unwrap();
    let mut transformed = false;
    let err = validate_before_transform(
        &input(
            "bad",
            TransitionKind::Productize,
            TransitionMode::Promote,
            &cognitive,
            &cognitive.artifact_id,
            cognitive.content_identity.clone(),
            vec![],
        ),
        std::slice::from_ref(&cognitive),
        &[],
        &profile(TransitionKind::Productize),
        || {
            transformed = true;
            Ok(())
        },
    );
    assert!(err.is_err());
    assert!(
        !transformed,
        "transform must not run before no-teleport validation"
    );

    let mut token = transition(input(
        "tampered",
        TransitionKind::Synthesize,
        TransitionMode::Promote,
        &cognitive,
        &cognitive.artifact_id,
        cognitive.content_identity.clone(),
        vec![],
    ))
    .unwrap();
    token.parent_transition_token_ids.push("tampered".into());
    assert!(rebuild(&[cognitive], &[token]).is_err());
}

struct FailingAttestor;
impl ArtifactAttestor for FailingAttestor {
    fn attest(&self, _authorization: AuthorizedAttestation) -> Result<String, ForgeError> {
        Err(ForgeError::Internal("ifl unavailable".into()))
    }
}

fn attestation_action(
    registration: &ArtifactRegistrationRecord,
    degraded_mode: &str,
) -> AuthorityAction {
    AuthorityAction::Reserved {
        resource_type: "attest_artifact_identity".into(),
        resource_id: registration.artifact_id.clone(),
        parameters: serde_json::json!({
            "artifact_id": registration.artifact_id,
            "content_identity": registration.content_identity,
            "descriptor_hash": registration.descriptor_hash,
            "ledger": "ifl",
            "degraded_mode": degraded_mode,
            "requester_role": "operator",
            "approver_role": "R-SO",
        }),
    }
}

fn authorized_attestation(
    root: &std::path::Path,
    registration: &ArtifactRegistrationRecord,
    degraded_mode: &str,
    controls: Vec<String>,
) -> AuthorizedAttestation {
    let (grant, action, decision, committed) = attestation_grant(root, registration, degraded_mode);
    authorize_attestation(
        grant,
        &action,
        root,
        registration,
        "ifl",
        degraded_mode,
        controls,
        &decision,
        &committed,
    )
    .unwrap()
}

fn attestation_grant(
    root: &std::path::Path,
    registration: &ArtifactRegistrationRecord,
    degraded_mode: &str,
) -> (
    ActionGrant,
    AuthorityAction,
    sea_forge_core::types::AuthorityDecision,
    sea_forge_ledger::CommittedRecordRef,
) {
    let bundle: AuthorityPolicyBundle = serde_yaml::from_str(
        r#"version: "0.1"
rules:
  - name: attest
    verdict: allow
    actor_role: operator
    operation_kind: attest_artifact_identity
    ledger: ifl
    requester_roles: [operator]
    approver_roles: [R-SO]
    degraded_mode: forbidden
  - name: attest-degraded
    verdict: allow
    actor_role: operator
    operation_kind: attest_artifact_identity
    ledger: ifl
    requester_roles: [operator]
    approver_roles: [R-SO]
    degraded_mode: pre_mint_only
"#,
    )
    .unwrap();
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let actor = Actor {
        actor_id: "operator".into(),
        role: ActorRole::Operator,
    };
    let action = attestation_action(registration, degraded_mode);
    let decision = engine
        .evaluate(AuthorityEvaluation {
            actor: &actor,
            binding: bundle.resolve_identity("operator", ActorRole::Operator),
            run_id: "attestation_ingress",
            case_id: "attestation_ingress",
            plan_item_id: "attestation_ingress",
            sequence: 1,
            action: &action,
            workspace_root: root,
            evidence_refs: vec![],
            artifacts_root: None,
            timeout_secs: None,
            env_keys: Default::default(),
            domainforge_candidate: None,
            environment: None,
        })
        .unwrap();
    assert_eq!(decision.verdict, sea_forge_core::types::Verdict::Allow);
    let committed = LedgerStream::open(root, "authority-attestation", "operator")
        .unwrap()
        .commit_typed("authority_decision", vec![], &decision, vec![])
        .unwrap();
    let grant = engine.grant(&decision, &committed, &action, None).unwrap();
    (grant, action, decision, committed)
}

#[test]
fn conformance_m8_attestation_required_and_degraded() {
    let root = tempfile::tempdir().unwrap();
    let registration = register_verified(
        &evidence(descriptor("art_model", "abc")),
        "model".into(),
        "1".into(),
        "case".into(),
        vec![],
        "now".into(),
    )
    .unwrap();
    let real = attest(
        &DefaultArtifactAttestor,
        authorized_attestation(root.path(), &registration, "forbidden", vec![]),
        AttestationPolicy::Required,
    )
    .unwrap();
    assert_eq!(real.status, IdentityStatus::Attested);
    assert!(real.attestation_ref.starts_with("ifl:token:ifl:"));
    assert!(real
        .attestation_ref
        .rsplit(':')
        .next()
        .is_some_and(|ordinal| ordinal.parse::<u64>().is_ok()));
    assert!(attest(
        &FailingAttestor,
        authorized_attestation(root.path(), &registration, "forbidden", vec![]),
        AttestationPolicy::Required
    )
    .is_err());
    assert_eq!(
        attest(
            &FailingAttestor,
            authorized_attestation(
                root.path(),
                &registration,
                "pre_mint_only",
                vec!["manual_review".into()]
            ),
            AttestationPolicy::PreMintOnly
        )
        .unwrap()
        .status,
        IdentityStatus::PreMint
    );
}

#[test]
fn conformance_m8_degraded_attestation_never_invents_missing_controls() {
    let root = tempfile::tempdir().unwrap();
    let registration = register_verified(
        &evidence(descriptor("art_degraded", "degraded")),
        "degraded.sea".into(),
        "1".into(),
        "case".into(),
        vec![],
        "now".into(),
    )
    .unwrap();
    assert!(attest(
        &FailingAttestor,
        authorized_attestation(root.path(), &registration, "pre_mint_only", vec![]),
        AttestationPolicy::PreMintOnly
    )
    .is_err());
}

#[test]
fn conformance_m8_forged_attestation_action_cannot_reach_backend() {
    let root = tempfile::tempdir().unwrap();
    let registration = register_verified(
        &evidence(descriptor("art_forged", "forged")),
        "forged.sea".into(),
        "1".into(),
        "case".into(),
        vec![],
        "now".into(),
    )
    .unwrap();
    let (grant, mut action, decision, committed) =
        attestation_grant(root.path(), &registration, "forbidden");
    if let AuthorityAction::Reserved { parameters, .. } = &mut action {
        parameters["content_identity"] = "sha256:forged".into();
    }
    assert!(authorize_attestation(
        grant,
        &action,
        root.path(),
        &registration,
        "ifl",
        "forbidden",
        vec![],
        &decision,
        &committed,
    )
    .is_err());
    assert!(!root.path().join("ledgers/ifl/entries.jsonl").exists());
}

#[test]
fn conformance_m8_attested_cognitive_stays_cognitive_and_controls_are_explicit() {
    let root = tempfile::tempdir().unwrap();
    let registration = register_verified(
        &evidence(descriptor("art_cognitive", "cognitive")),
        "cognitive.sea".into(),
        "1".into(),
        "case".into(),
        vec![],
        "now".into(),
    )
    .unwrap();
    let attested = attest(
        &DefaultArtifactAttestor,
        authorized_attestation(root.path(), &registration, "forbidden", vec![]),
        AttestationPolicy::Required,
    )
    .unwrap();
    assert_eq!(attested.status, IdentityStatus::Attested);
    assert_eq!(
        rebuild(std::slice::from_ref(&registration), &[]).unwrap()[&registration.artifact_id]
            .recognized_stage,
        ArtifactStage::Cognitive
    );

    let degraded = attest(
        &FailingAttestor,
        authorized_attestation(
            root.path(),
            &registration,
            "pre_mint_only",
            vec!["manual_review".into()],
        ),
        AttestationPolicy::PreMintOnly,
    )
    .unwrap();
    let mut transition_input = input(
        "tok_degraded",
        TransitionKind::Synthesize,
        TransitionMode::Promote,
        &registration,
        &registration.artifact_id,
        registration.content_identity.clone(),
        vec![],
    );
    transition_input.degraded_controls = degraded.degraded_controls.clone();
    assert_eq!(
        transition(transition_input).unwrap().degraded_controls,
        vec!["manual_review"]
    );
}

#[test]
fn conformance_m8_attested_token_resolves_ledger_record() {
    let root = tempfile::tempdir().unwrap();
    let registration = append_registration(
        root.path(),
        "operator",
        &commit_artifact_evidence(
            root.path(),
            "operator",
            descriptor("art_attested_token", "attested-token"),
        )
        .evidence_id,
        VerifiedArtifactRegistrationInput {
            name: "attested.sea".into(),
            version: "1".into(),
            case_id: "case".into(),
            derived_from: vec![],
            created_at: "now".into(),
        },
    )
    .unwrap();
    let attestation = attest(
        &DefaultArtifactAttestor,
        authorized_attestation(root.path(), &registration, "forbidden", vec![]),
        AttestationPolicy::Required,
    )
    .unwrap();
    let mut transition_input = input(
        "tok_attested",
        TransitionKind::Synthesize,
        TransitionMode::Promote,
        &registration,
        &registration.artifact_id,
        registration.content_identity.clone(),
        vec![],
    );
    transition_input.identity_status_after = IdentityStatus::Attested;
    transition_input.attestation_ref = Some(attestation.attestation_ref);
    let mut gate = profile(TransitionKind::Synthesize);
    gate.attestation_mode = Some("required".into());
    ledger_governance(root.path(), &mut transition_input, &gate);
    let authorization = authorized_transition(
        root.path(),
        &transition_input,
        std::slice::from_ref(&registration),
        &gate,
    );
    assert_eq!(
        append_transition(
            authorization,
            transition_input,
            std::slice::from_ref(&registration),
            &[],
            &gate,
        )
        .unwrap()
        .identity_status_after,
        IdentityStatus::Attested
    );
}

#[test]
fn conformance_m8_pre_mint_token_requires_explicit_controls() {
    let root = tempfile::tempdir().unwrap();
    let registration = append_registration(
        root.path(),
        "operator",
        &commit_artifact_evidence(
            root.path(),
            "operator",
            descriptor("art_pre_mint", "pre-mint"),
        )
        .evidence_id,
        VerifiedArtifactRegistrationInput {
            name: "pre-mint.sea".into(),
            version: "1".into(),
            case_id: "case".into(),
            derived_from: vec![],
            created_at: "now".into(),
        },
    )
    .unwrap();
    let mut transition_input = input(
        "tok_pre_mint",
        TransitionKind::Synthesize,
        TransitionMode::Promote,
        &registration,
        &registration.artifact_id,
        registration.content_identity.clone(),
        vec![],
    );
    let mut gate = profile(TransitionKind::Synthesize);
    gate.attestation_mode = Some("pre_mint_only".into());
    ledger_governance(root.path(), &mut transition_input, &gate);
    let missing_controls_authorization = authorized_transition(
        root.path(),
        &transition_input,
        std::slice::from_ref(&registration),
        &gate,
    );
    let error = append_transition(
        missing_controls_authorization,
        transition_input.clone(),
        std::slice::from_ref(&registration),
        &[],
        &gate,
    )
    .unwrap_err();
    assert_eq!(error.class(), "artifact_attestation_error");
    transition_input.degraded_controls = vec!["manual_review".into()];
    let controlled_authorization = authorized_transition(
        root.path(),
        &transition_input,
        std::slice::from_ref(&registration),
        &gate,
    );
    assert_eq!(
        append_transition(
            controlled_authorization,
            transition_input,
            std::slice::from_ref(&registration),
            &[],
            &gate,
        )
        .unwrap()
        .degraded_controls,
        vec!["manual_review"]
    );
}

#[test]
fn conformance_m8_unresolved_governance_reference_rejects_transition() {
    let root = tempfile::tempdir().unwrap();
    let registration = append_registration(
        root.path(),
        "operator",
        &commit_artifact_evidence(
            root.path(),
            "operator",
            descriptor("art_hostile", "hostile"),
        )
        .evidence_id,
        VerifiedArtifactRegistrationInput {
            name: "hostile.sea".into(),
            version: "1".into(),
            case_id: "case_source".into(),
            derived_from: vec![],
            created_at: "now".into(),
        },
    )
    .unwrap();
    let mut hostile_input = input(
        "tok_hostile",
        TransitionKind::Synthesize,
        TransitionMode::Promote,
        &registration,
        &registration.artifact_id,
        registration.content_identity.clone(),
        vec![],
    );
    let hostile_profile = profile(TransitionKind::Synthesize);
    hostile_input.gate_profile_ref = hostile_profile.profile_ref.clone();
    hostile_input.gate_profile_hash = hash_canonical(&hostile_profile).unwrap();
    let authorization = authorized_transition(
        root.path(),
        &hostile_input,
        std::slice::from_ref(&registration),
        &hostile_profile,
    );
    let error = append_transition(
        authorization,
        hostile_input,
        std::slice::from_ref(&registration),
        &[],
        &hostile_profile,
    )
    .unwrap_err();
    assert_eq!(error.class(), "artifact_reference_error");
}

#[test]
fn conformance_m8_rebuild_rejects_allow_for_substituted_transition_action() {
    let root = tempfile::tempdir().unwrap();
    let registration = append_registration(
        root.path(),
        "operator",
        &commit_artifact_evidence(
            root.path(),
            "operator",
            descriptor("art_substituted", "substituted"),
        )
        .evidence_id,
        VerifiedArtifactRegistrationInput {
            name: "substituted.sea".into(),
            version: "1".into(),
            case_id: "case_source".into(),
            derived_from: vec![],
            created_at: "now".into(),
        },
    )
    .unwrap();
    let mut transition_input = input(
        "tok_substituted",
        TransitionKind::Synthesize,
        TransitionMode::Promote,
        &registration,
        &registration.artifact_id,
        registration.content_identity.clone(),
        vec![],
    );
    let gate = profile(TransitionKind::Synthesize);
    ledger_governance(root.path(), &mut transition_input, &gate);
    drop(authorized_transition(
        root.path(),
        &transition_input,
        std::slice::from_ref(&registration),
        &gate,
    ));
    transition_input.actor_id = "substituted-requester".into();
    let token = transition(transition_input).unwrap();
    LedgerStream::open(root.path(), "artifact-ip", "operator")
        .unwrap()
        .commit_typed(
            "artifact_transition",
            vec![registration.artifact_id.clone()],
            &token,
            token.authority_decision_refs.clone(),
        )
        .unwrap();

    let error = rebuild_materialized_views(root.path(), "operator").unwrap_err();
    assert_eq!(error.class(), "artifact_authority_error");
}

#[test]
fn conformance_m8_value_evidence_acceptance_is_derived_from_sources() {
    let root = tempfile::tempdir().unwrap();
    let cognitive = append_registration(
        root.path(),
        "operator",
        &commit_artifact_evidence(root.path(), "operator", descriptor("art_root", "root"))
            .evidence_id,
        VerifiedArtifactRegistrationInput {
            name: "root.sea".into(),
            version: "1".into(),
            case_id: "case_source".into(),
            derived_from: vec![],
            created_at: "now".into(),
        },
    )
    .unwrap();
    let product = append_registration(
        root.path(),
        "operator",
        &commit_artifact_evidence(
            root.path(),
            "operator",
            descriptor("art_product_hostile", "product"),
        )
        .evidence_id,
        VerifiedArtifactRegistrationInput {
            name: "product.sea".into(),
            version: "2".into(),
            case_id: "case_source".into(),
            derived_from: vec![cognitive.artifact_id.clone()],
            created_at: "now".into(),
        },
    )
    .unwrap();
    let mut intellectual_input = input(
        "tok_root_int",
        TransitionKind::Synthesize,
        TransitionMode::Promote,
        &cognitive,
        &cognitive.artifact_id,
        cognitive.content_identity.clone(),
        vec![],
    );
    let intellectual_profile = profile(TransitionKind::Synthesize);
    ledger_governance(root.path(), &mut intellectual_input, &intellectual_profile);
    let intellectual_authorization = authorized_transition(
        root.path(),
        &intellectual_input,
        &[cognitive.clone(), product.clone()],
        &intellectual_profile,
    );
    let intellectual = append_transition(
        intellectual_authorization,
        intellectual_input,
        &[cognitive.clone(), product.clone()],
        &[],
        &intellectual_profile,
    )
    .unwrap();
    let mut product_input = input(
        "tok_root_product",
        TransitionKind::Productize,
        TransitionMode::Derive,
        &cognitive,
        &product.artifact_id,
        product.content_identity.clone(),
        vec![intellectual.transition_token_id.clone()],
    );
    let product_profile = profile(TransitionKind::Productize);
    ledger_governance(root.path(), &mut product_input, &product_profile);
    let product_authorization = authorized_transition(
        root.path(),
        &product_input,
        &[cognitive.clone(), product.clone()],
        &product_profile,
    );
    let product_token = append_transition(
        product_authorization,
        product_input,
        &[cognitive.clone(), product.clone()],
        std::slice::from_ref(&intellectual),
        &product_profile,
    )
    .unwrap();
    let mut capital_input = input(
        "tok_hostile_capital",
        TransitionKind::Capitalize,
        TransitionMode::Promote,
        &product,
        &product.artifact_id,
        product.content_identity.clone(),
        vec![product_token.transition_token_id.clone()],
    );
    let capital_profile = profile(TransitionKind::Capitalize);
    ledger_governance(root.path(), &mut capital_input, &capital_profile);
    drop(authorized_transition(
        root.path(),
        &capital_input,
        &[cognitive.clone(), product.clone()],
        &capital_profile,
    ));
    let derived = append_value_evidence(
        root.path(),
        "operator",
        ValueEvidenceRecord {
            version: String::new(),
            value_evidence_id: "value_rejected".into(),
            kind: "adoption".into(),
            case_id: "case_other".into(),
            evidence_refs: vec!["value_adoption_source".into()],
            accepted: false,
            created_at: "now".into(),
            record_hash: String::new(),
        },
    )
    .unwrap();
    assert!(derived.accepted);
    capital_input.value_evidence_refs = vec!["value_rejected".into()];
    capital_input.value_evidence = vec![ValueEvidenceRef {
        reference: "value_rejected".into(),
        case_id: "case_other".into(),
        kind: "adoption".into(),
    }];
    let capital = transition(capital_input).unwrap();
    LedgerStream::open(root.path(), "artifact-ip", "operator")
        .unwrap()
        .commit_typed(
            "artifact_transition",
            vec![product.artifact_id.clone()],
            &capital,
            capital.authority_decision_refs.clone(),
        )
        .unwrap();
    rebuild_materialized_views(root.path(), "operator").unwrap();
    assert_capital_projection(root.path(), &product.artifact_id, true);
}

#[test]
fn conformance_m8_fabricated_accepted_value_source_without_run_records_is_rejected() {
    let root = tempfile::tempdir().unwrap();
    let error = append_value_evidence_source(
        root.path(),
        "operator",
        ValueEvidenceSourceRecord {
            version: String::new(),
            evidence_id: "fabricated_source".into(),
            kind: "adoption".into(),
            case_id: "case_other".into(),
            originating_case_id: "case_source".into(),
            evidence_refs: vec!["evi_missing".into()],
            source_run_id: "run_missing".into(),
            settlement_ref: "settlement_missing".into(),
            created_at: "now".into(),
            record_hash: String::new(),
        },
    )
    .unwrap_err();
    assert_eq!(error.class(), "artifact_reference_error");
    assert!(!root
        .path()
        .join("ledgers/artifact-ip/entries.jsonl")
        .exists());
}

#[test]
fn conformance_m8_rebuild_rejects_approval_for_unrelated_decision_and_criteria() {
    let root = tempfile::tempdir().unwrap();
    let registration = append_registration(
        root.path(),
        "operator",
        &commit_artifact_evidence(
            root.path(),
            "operator",
            descriptor("art_approval_binding", "approval-binding"),
        )
        .evidence_id,
        VerifiedArtifactRegistrationInput {
            name: "approval-binding.sea".into(),
            version: "1".into(),
            case_id: "case_source".into(),
            derived_from: vec![],
            created_at: "now".into(),
        },
    )
    .unwrap();
    let registrations = std::slice::from_ref(&registration);

    let mut intellectual_input = input(
        "tok_approval_intellectual",
        TransitionKind::Synthesize,
        TransitionMode::Promote,
        &registration,
        &registration.artifact_id,
        registration.content_identity.clone(),
        vec![],
    );
    let intellectual_profile = profile(TransitionKind::Synthesize);
    ledger_governance(root.path(), &mut intellectual_input, &intellectual_profile);
    let intellectual = append_transition(
        authorized_transition(
            root.path(),
            &intellectual_input,
            registrations,
            &intellectual_profile,
        ),
        intellectual_input,
        registrations,
        &[],
        &intellectual_profile,
    )
    .unwrap();

    let mut product_input = input(
        "tok_approval_product",
        TransitionKind::Productize,
        TransitionMode::Promote,
        &registration,
        &registration.artifact_id,
        registration.content_identity.clone(),
        vec![intellectual.transition_token_id.clone()],
    );
    let product_profile = profile(TransitionKind::Productize);
    ledger_governance(root.path(), &mut product_input, &product_profile);
    let product = append_transition(
        authorized_transition(root.path(), &product_input, registrations, &product_profile),
        product_input,
        registrations,
        std::slice::from_ref(&intellectual),
        &product_profile,
    )
    .unwrap();

    let mut capital_input = input(
        "tok_approval_capital",
        TransitionKind::Capitalize,
        TransitionMode::Promote,
        &registration,
        &registration.artifact_id,
        registration.content_identity.clone(),
        vec![product.transition_token_id.clone()],
    );
    let capital_profile = profile(TransitionKind::Capitalize);
    capital_input.approval_ref = Some("approval_unrelated".into());
    ledger_governance(root.path(), &mut capital_input, &capital_profile);
    drop(authorized_transition(
        root.path(),
        &capital_input,
        registrations,
        &capital_profile,
    ));
    let unrelated = ApprovalRequest {
        version: "0.2".into(),
        approval_id: "approval_unrelated".into(),
        run_id: capital_input.run_id.clone(),
        case_id: capital_input.case_id.clone(),
        decision_id: "auth_unrelated".into(),
        plan_item_id: "transition".into(),
        criteria_ref: Some("criteria_unrelated".into()),
        criteria_sha256: None,
        criteria_record_hash: None,
        job_contract_ref: None,
        requested_at: "2026-07-15T00:00:00Z".into(),
        expires_at: "2026-07-16T00:00:00Z".into(),
        status: ApprovalStatus::Approved,
        resolved_by: capital_input.approver_id.clone(),
        resolved_at: Some("2026-07-15T00:01:00Z".into()),
        note: None,
    };
    LedgerStream::open(root.path(), "case-governance", "approver")
        .unwrap()
        .commit_typed(
            "approval_resolution",
            vec![unrelated.approval_id.clone()],
            &unrelated,
            vec![unrelated.decision_id.clone()],
        )
        .unwrap();
    let capital = transition(capital_input).unwrap();
    LedgerStream::open(root.path(), "artifact-ip", "operator")
        .unwrap()
        .commit_typed(
            "artifact_transition",
            vec![registration.artifact_id.clone()],
            &capital,
            capital.authority_decision_refs.clone(),
        )
        .unwrap();

    let error = rebuild_materialized_views(root.path(), "operator").unwrap_err();
    assert_eq!(error.class(), "artifact_reference_error");
}

#[test]
fn conformance_m8_rebuild_rejects_approval_with_missing_or_wrong_criteria_hashes() {
    for wrong_field in ["criteria_sha256", "criteria_record_hash"] {
        let root = tempfile::tempdir().unwrap();
        let (registration, tokens) =
            ledgered_chain_through_product(root.path(), "art_approval_hash");
        let gate = profile(TransitionKind::Capitalize);
        let mut capital_input = input(
            "tok_approval_hash_capital",
            TransitionKind::Capitalize,
            TransitionMode::Promote,
            &registration,
            &registration.artifact_id,
            registration.content_identity.clone(),
            vec![tokens.last().unwrap().transition_token_id.clone()],
        );
        let criteria = test_criteria_record(&capital_input);
        let approval_hashes = if wrong_field == "criteria_sha256" {
            (None, Some(criteria.criteria_record_hash))
        } else {
            (
                Some(criteria.criteria_sha256),
                Some("sha256:wrong-record".into()),
            )
        };
        ledger_governance(root.path(), &mut capital_input, &gate);
        drop(authorized_transition(
            root.path(),
            &capital_input,
            std::slice::from_ref(&registration),
            &gate,
        ));
        let wrong_approval = ApprovalRequest {
            version: "0.2".into(),
            approval_id: "approval_wrong_hash".into(),
            run_id: capital_input.run_id.clone(),
            case_id: capital_input.case_id.clone(),
            decision_id: capital_input.authority_decision_refs[0].clone(),
            plan_item_id: "transition".into(),
            criteria_ref: Some(capital_input.criteria_ref.clone()),
            criteria_sha256: approval_hashes.0,
            criteria_record_hash: approval_hashes.1,
            job_contract_ref: None,
            requested_at: "2026-07-15T00:00:00Z".into(),
            expires_at: "2099-07-16T00:00:00Z".into(),
            status: ApprovalStatus::Approved,
            resolved_by: capital_input.approver_id.clone(),
            resolved_at: Some("2026-07-15T00:01:00Z".into()),
            note: None,
        };
        LedgerStream::open(root.path(), "case-governance", "operator")
            .unwrap()
            .commit_typed(
                "approval_resolution",
                vec![wrong_approval.approval_id.clone()],
                &wrong_approval,
                vec![wrong_approval.decision_id.clone()],
            )
            .unwrap();
        capital_input.approval_ref = Some(wrong_approval.approval_id);

        let capital = transition(capital_input).unwrap();
        LedgerStream::open(root.path(), "artifact-ip", "operator")
            .unwrap()
            .commit_typed(
                "artifact_transition",
                vec![registration.artifact_id.clone()],
                &capital,
                capital.authority_decision_refs.clone(),
            )
            .unwrap();
        assert_eq!(
            rebuild_materialized_views(root.path(), "operator")
                .unwrap_err()
                .class(),
            "artifact_reference_error"
        );
    }
}

#[test]
fn conformance_m8_non_cognitive_multi_source_requires_parent_for_each_source() {
    // Hostile: a non-cognitive derive with two sources where source B was
    // independently transitioned to from_stage but its parent token is
    // omitted from the token's parent list. Rebuild must reject the omitted
    // parent even though both sources reach from_stage independently.
    let root = register(ArtifactRegistrationInput {
        descriptor: descriptor("art_omitted_root", "omitted-root"),
        name: "root.sea".into(),
        version: "1".into(),
        source_evidence_refs: vec!["evi_root".into()],
        source_run_ids: vec!["run_root".into()],
        case_id: "case_source".into(),
        derived_from: vec![],
        created_at: "now".into(),
    })
    .unwrap();
    let root_lineage = root.lineage_id.clone();
    // Force all derived registrations onto the same lineage so the structural
    // lineage check passes; we are testing the parent invariant, not lineage.
    let mut source_a = register(ArtifactRegistrationInput {
        descriptor: descriptor("art_omitted_a", "omitted-a"),
        name: "a.sea".into(),
        version: "1".into(),
        source_evidence_refs: vec!["evi_a".into()],
        source_run_ids: vec!["run_root".into()],
        case_id: "case_source".into(),
        derived_from: vec![root.artifact_id.clone()],
        created_at: "now".into(),
    })
    .unwrap();
    source_a.lineage_id = root_lineage.clone();
    let mut source_b = register(ArtifactRegistrationInput {
        descriptor: descriptor("art_omitted_b", "omitted-b"),
        name: "b.sea".into(),
        version: "1".into(),
        source_evidence_refs: vec!["evi_b".into()],
        source_run_ids: vec!["run_root".into()],
        case_id: "case_source".into(),
        derived_from: vec![root.artifact_id.clone()],
        created_at: "now".into(),
    })
    .unwrap();
    source_b.lineage_id = root_lineage.clone();
    let mut result = register(ArtifactRegistrationInput {
        descriptor: descriptor("art_omitted_result", "omitted-result"),
        name: "result.sea".into(),
        version: "1".into(),
        source_evidence_refs: vec!["evi_result".into()],
        source_run_ids: vec!["run_root".into()],
        case_id: "case_source".into(),
        derived_from: vec![source_a.artifact_id.clone(), source_b.artifact_id.clone()],
        created_at: "now".into(),
    })
    .unwrap();
    result.lineage_id = root_lineage;

    let parent_a = transition(input(
        "tok_omitted_a_int",
        TransitionKind::Synthesize,
        TransitionMode::Promote,
        &source_a,
        &source_a.artifact_id,
        source_a.content_identity.clone(),
        vec![],
    ))
    .unwrap();
    let parent_b = transition(input(
        "tok_omitted_b_int",
        TransitionKind::Synthesize,
        TransitionMode::Promote,
        &source_b,
        &source_b.artifact_id,
        source_b.content_identity.clone(),
        vec![],
    ))
    .unwrap();
    let mut derived = input(
        "tok_omitted_product",
        TransitionKind::Productize,
        TransitionMode::Derive,
        &source_a,
        &result.artifact_id,
        result.content_identity.clone(),
        vec!["tok_omitted_a_int".into()],
    );
    derived.source_artifact_ids = vec![source_a.artifact_id.clone(), source_b.artifact_id.clone()];
    derived.input_content_identities = vec![
        source_a.content_identity.clone(),
        source_b.content_identity.clone(),
    ];
    derived.derived_from = derived.source_artifact_ids.clone();
    let mut transformed = false;
    let error = validate_before_transform(
        &derived,
        &[
            root.clone(),
            source_a.clone(),
            source_b.clone(),
            result.clone(),
        ],
        &[parent_a.clone(), parent_b.clone()],
        &profile(TransitionKind::Productize),
        || {
            transformed = true;
            Ok(())
        },
    )
    .unwrap_err();
    assert_eq!(error.class(), "artifact_lineage_error");
    assert!(
        !transformed,
        "missing source parent must block append before effects"
    );
    let token = transition(derived).unwrap();
    let error = rebuild(
        &[root, source_a, source_b, result],
        &[parent_a, parent_b, token],
    )
    .unwrap_err();
    assert_eq!(error.class(), "artifact_lineage_error");
}

#[test]
fn conformance_m8_rebuild_rejects_attestation_without_exact_authority_binding() {
    let root = tempfile::tempdir().unwrap();
    let registration = append_registration(
        root.path(),
        "operator",
        &commit_artifact_evidence(
            root.path(),
            "operator",
            descriptor("art_attestation_binding", "attestation-binding"),
        )
        .evidence_id,
        VerifiedArtifactRegistrationInput {
            name: "binding.sea".into(),
            version: "1".into(),
            case_id: "case_source".into(),
            derived_from: vec![],
            created_at: "now".into(),
        },
    )
    .unwrap();
    LedgerStream::open(root.path(), "ifl", "ifl")
        .unwrap()
        .commit_typed(
            "artifact_identity_attestation",
            vec![registration.artifact_id.clone()],
            &serde_json::json!({
                "artifact_id": registration.artifact_id,
                "content_identity": registration.content_identity,
                "descriptor_hash": registration.descriptor_hash,
            }),
            vec![],
        )
        .unwrap();

    assert_eq!(
        rebuild_materialized_views(root.path(), "operator")
            .unwrap_err()
            .class(),
        "artifact_attestation_error"
    );
}

#[test]
fn deleting_middle_token_fails_complete_chain() {
    let root = tempfile::tempdir().unwrap();
    let (registration, tokens) = ledgered_chain_through_product(root.path(), "art_chain");
    let gate = profile(TransitionKind::Capitalize);
    attempt_capital(root.path(), &registration, &tokens, &gate, Some("adoption")).unwrap();
    let (registrations, complete_tokens) = load_ledger_records(root.path(), "operator").unwrap();
    assert_eq!(
        rebuild(&registrations, &complete_tokens).unwrap()[&registration.artifact_id]
            .recognized_stage,
        ArtifactStage::Capital
    );

    let without_product = complete_tokens
        .into_iter()
        .filter(|token| token.transition_kind != TransitionKind::Productize)
        .collect::<Vec<_>>();
    assert_eq!(
        rebuild(&registrations, &without_product)
            .unwrap_err()
            .class(),
        "artifact_lineage_error"
    );
}

#[test]
fn quality_only_capital_rejected() {
    let root = tempfile::tempdir().unwrap();
    let (registration, tokens) = ledgered_chain_through_product(root.path(), "art_quality_only");
    let mut quality_profile = profile(TransitionKind::Capitalize);
    quality_profile.qualifying_value_evidence_kinds = vec!["quality".into()];
    let error = attempt_capital(
        root.path(),
        &registration,
        &tokens,
        &quality_profile,
        Some("quality"),
    )
    .unwrap_err();
    assert_eq!(error.class(), "artifact_transition_error");
    let ledger =
        std::fs::read_to_string(root.path().join("ledgers/artifact-ip/entries.jsonl")).unwrap();
    assert!(ledger.contains("value_quality"));
    assert!(!ledger.contains("tok_art_quality_only_capital"));
    let quality = LedgerStream::open(root.path(), "artifact-ip", "operator")
        .unwrap()
        .read_entries()
        .unwrap()
        .into_iter()
        .find(|entry| {
            entry.record_kind == "artifact_value_evidence"
                && entry.payload["value_evidence_id"] == "value_quality"
        })
        .unwrap();
    assert_eq!(quality.payload["accepted"], true);
    assert_capital_projection(root.path(), &registration.artifact_id, false);
}

#[test]
fn approval_only_capital_rejected() {
    let valid_root = tempfile::tempdir().unwrap();
    let (valid_registration, valid_tokens) =
        ledgered_chain_through_product(valid_root.path(), "art_approval_valid");
    let gate = profile(TransitionKind::Capitalize);
    attempt_capital(
        valid_root.path(),
        &valid_registration,
        &valid_tokens,
        &gate,
        Some("adoption"),
    )
    .unwrap();
    assert_capital_projection(valid_root.path(), &valid_registration.artifact_id, true);

    let hostile_root = tempfile::tempdir().unwrap();
    let (registration, tokens) =
        ledgered_chain_through_product(hostile_root.path(), "art_approval_hostile");
    let error =
        attempt_capital(hostile_root.path(), &registration, &tokens, &gate, None).unwrap_err();
    assert_eq!(error.class(), "artifact_transition_error");
    assert_capital_projection(hostile_root.path(), &registration.artifact_id, false);
}

#[test]
fn content_changing_capitalize_rejected() {
    let registration = register_verified(
        &evidence(descriptor("art_change", "change")),
        "change.sea".into(),
        "1".into(),
        "case".into(),
        vec![],
        "now".into(),
    )
    .unwrap();
    assert!(transition(input(
        "tok_change",
        TransitionKind::Capitalize,
        TransitionMode::Promote,
        &registration,
        &registration.artifact_id,
        "sha256:changed".into(),
        vec!["tok_product".into()],
    ))
    .is_err());
}

#[test]
fn forged_stale_state_catalog_capital_files_removed_overwritten_by_rebuild() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("artifacts")).unwrap();
    std::fs::create_dir_all(root.path().join("ip/capital")).unwrap();
    std::fs::write(root.path().join("artifacts/catalog.jsonl"), "forged").unwrap();
    std::fs::write(root.path().join("ip/capital/stale.json"), "forged").unwrap();
    rebuild_materialized_views(root.path(), "operator").unwrap();
    assert_eq!(
        std::fs::read_to_string(root.path().join("artifacts/catalog.jsonl")).unwrap(),
        ""
    );
    assert!(!root.path().join("ip/capital/stale.json").exists());
    assert!(!root.path().join("artifacts/state.json").exists());
}

#[test]
fn attestation_without_passed_maturity_gate_leaves_stage_unchanged() {
    let root = tempfile::tempdir().unwrap();
    let registration = register_verified(
        &evidence(descriptor("art_identity_only", "identity")),
        "identity.sea".into(),
        "1".into(),
        "case".into(),
        vec![],
        "now".into(),
    )
    .unwrap();
    attest(
        &DefaultArtifactAttestor,
        authorized_attestation(root.path(), &registration, "forbidden", vec![]),
        AttestationPolicy::Required,
    )
    .unwrap();
    assert_eq!(
        rebuild(&[registration], &[])
            .unwrap()
            .values()
            .next()
            .unwrap()
            .recognized_stage,
        ArtifactStage::Cognitive
    );
}

#[test]
fn exact_transition_grant_cannot_be_reused_for_substituted_input() {
    let root = tempfile::tempdir().unwrap();
    let registration = append_registration(
        root.path(),
        "operator",
        &commit_artifact_evidence(root.path(), "operator", descriptor("art_exact", "exact"))
            .evidence_id,
        VerifiedArtifactRegistrationInput {
            name: "exact.sea".into(),
            version: "1".into(),
            case_id: "case_source".into(),
            derived_from: vec![],
            created_at: "now".into(),
        },
    )
    .unwrap();
    let mut transition_input = input(
        "tok_exact",
        TransitionKind::Synthesize,
        TransitionMode::Promote,
        &registration,
        &registration.artifact_id,
        registration.content_identity.clone(),
        vec![],
    );
    let gate = profile(TransitionKind::Synthesize);
    ledger_governance(root.path(), &mut transition_input, &gate);
    let authorization = authorized_transition(
        root.path(),
        &transition_input,
        std::slice::from_ref(&registration),
        &gate,
    );
    transition_input.output_content_identity = "sha256:substituted".into();
    let error = append_transition(
        authorization,
        transition_input,
        std::slice::from_ref(&registration),
        &[],
        &gate,
    )
    .unwrap_err();
    assert_eq!(error.class(), "artifact_authority_error");
    assert!(
        !std::fs::read_to_string(root.path().join("ledgers/artifact-ip/entries.jsonl"))
            .unwrap()
            .contains("tok_exact")
    );
}

#[test]
fn all_source_licenses_must_match_transition_policy() {
    let first = register_verified(
        &evidence(descriptor("art_license_a", "a")),
        "a.sea".into(),
        "1".into(),
        "case".into(),
        vec![],
        "now".into(),
    )
    .unwrap();
    let mut second_descriptor = descriptor("art_license_b", "b");
    second_descriptor.license = "proprietary".into();
    let second = register_verified(
        &evidence(second_descriptor),
        "b.sea".into(),
        "1".into(),
        "case".into(),
        vec![],
        "now".into(),
    )
    .unwrap();
    let mut transition_input = input(
        "tok_licenses",
        TransitionKind::Synthesize,
        TransitionMode::Derive,
        &first,
        "art_result",
        "sha256:result".into(),
        vec![],
    );
    transition_input.source_artifact_ids = vec![first.artifact_id, second.artifact_id];
    transition_input.input_content_identities =
        vec![first.content_identity, second.content_identity];
    transition_input.derived_from = transition_input.source_artifact_ids.clone();
    let gate = profile(TransitionKind::Synthesize);
    transition_input.gate_profile_ref = gate.profile_ref.clone();
    transition_input.gate_profile_hash = hash_canonical(&gate).unwrap();
    let action = canonical_transition_action(
        &transition_input,
        &["internal".into(), "proprietary".into()],
        &gate,
    );
    let bundle: AuthorityPolicyBundle = serde_yaml::from_str(&format!(
        "version: \"0.1\"\nrules:\n  - name: transition\n    verdict: allow\n    actor_role: operator\n    operation_kind: transition_artifact_stage\n    transition_kind: synthesize\n    from_stage: cognitive\n    to_stage: intellectual\n    modes: [derive]\n    gate_profile_ref: {}\n    required_settlement_strength: local\n    qualifying_value_evidence_kinds: [adoption]\n    requires_approval: false\n    license_allowlist: [internal]\n",
        gate.profile_ref
    ))
    .unwrap();
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let actor = Actor {
        actor_id: "requester".into(),
        role: ActorRole::Operator,
    };
    let decision = engine
        .evaluate(AuthorityEvaluation {
            actor: &actor,
            binding: bundle.resolve_identity("requester", ActorRole::Operator),
            run_id: "run",
            case_id: "case",
            plan_item_id: "transition",
            sequence: 1,
            action: &action,
            workspace_root: std::path::Path::new("."),
            evidence_refs: vec![],
            artifacts_root: None,
            timeout_secs: None,
            env_keys: Default::default(),
            domainforge_candidate: None,
            environment: None,
        })
        .unwrap();
    assert_eq!(decision.verdict, sea_forge_core::types::Verdict::Deny);
}

#[test]
fn value_evidence_without_underlying_accepted_evidence_is_rejected() {
    let root = tempfile::tempdir().unwrap();
    let error = append_value_evidence(
        root.path(),
        "operator",
        ValueEvidenceRecord {
            version: String::new(),
            value_evidence_id: "value_empty".into(),
            kind: "adoption".into(),
            case_id: "case_other".into(),
            evidence_refs: vec![],
            accepted: true,
            created_at: "now".into(),
            record_hash: String::new(),
        },
    )
    .unwrap_err();
    assert_eq!(error.class(), "artifact_reference_error");
}

#[test]
fn conformance_m8_partial_strong_declaration_is_rejected() {
    let root = tempfile::tempdir().unwrap();
    let (registration, tokens) = ledgered_chain_through_product(root.path(), "art_partial_decl");
    let gate = profile(TransitionKind::Capitalize);
    let mut capital_input = input(
        "tok_partial_decl_capital",
        TransitionKind::Capitalize,
        TransitionMode::Promote,
        &registration,
        &registration.artifact_id,
        registration.content_identity.clone(),
        vec![tokens.last().unwrap().transition_token_id.clone()],
    );
    ledger_governance(root.path(), &mut capital_input, &gate);
    capital_input.strong_declaration_refs = vec!["decl_partial".into()];
    let mut partial = serde_json::json!({
        "declaration_id": "decl_partial",
        "declaration_hash": "",
        "status": "accepted",
        "strength": "strong",
        "settlement_ref": capital_input.settlement_ref,
        "criteria_ref": capital_input.criteria_ref,
        "run_id": capital_input.run_id,
        "qualifies_for_capability": true,
        "independence": {"independent": true},
        "reliability": {"weight": "0.900000"},
    });
    partial["declaration_hash"] = hash_canonical(&partial).unwrap().into();
    LedgerStream::open(root.path(), "case-governance", "operator")
        .unwrap()
        .commit_typed(
            "settlement_declaration",
            vec!["decl_partial".into()],
            &partial,
            vec![],
        )
        .unwrap();

    let error = append_transition(
        authorized_transition(
            root.path(),
            &capital_input,
            std::slice::from_ref(&registration),
            &gate,
        ),
        capital_input,
        std::slice::from_ref(&registration),
        &tokens,
        &gate,
    )
    .unwrap_err();
    assert_eq!(error.class(), "artifact_reference_error");
}

#[test]
fn conformance_m8_generic_execution_result_is_not_value_evidence() {
    let root = tempfile::tempdir().unwrap();
    commit_value_source_run(root.path(), BTreeMap::new());

    let error =
        append_value_evidence_source(root.path(), "operator", value_source_record("adoption"))
            .unwrap_err();
    assert_eq!(error.class(), "artifact_reference_error");
}

#[test]
fn conformance_m8_relabelled_value_evidence_is_rejected() {
    let root = tempfile::tempdir().unwrap();
    commit_value_source_run(
        root.path(),
        BTreeMap::from([("canonical_value_evidence_kind".into(), "reuse".into())]),
    );

    let error =
        append_value_evidence_source(root.path(), "operator", value_source_record("adoption"))
            .unwrap_err();
    assert_eq!(error.class(), "artifact_reference_error");
}

fn commit_value_source_run(root: &std::path::Path, metadata: BTreeMap<String, serde_json::Value>) {
    let stream = LedgerStream::open(root, "case-case_other", "operator").unwrap();
    stream
        .commit_typed(
            "run_evidence",
            vec!["evi_value_source".into()],
            &EvidenceRecord {
                version: "0.2".into(),
                evidence_id: "evi_value_source".into(),
                run_id: "run_value_source".into(),
                kind: EvidenceKind::ExecutionResult,
                uri: "runs/run_value_source/evidence.json".into(),
                sha256: Some("sha256:value-source".into()),
                source_event_id: "evt_value_source".into(),
                metadata,
                cell_id: None,
            },
            vec![],
        )
        .unwrap();
    stream
        .commit_typed(
            "settlement_event",
            vec!["settlement_value_source".into()],
            &serde_json::json!({
                "version": "0.2",
                "settlement_id": "settlement_value_source",
                "run_id": "run_value_source",
                "status": "accepted",
                "basis": ["source_value_demonstrated"],
                "review_required": false,
                "settled_at": "now",
                "criteria_ref": "criteria_source",
            }),
            vec![],
        )
        .unwrap();
}

fn value_source_record(kind: &str) -> ValueEvidenceSourceRecord {
    ValueEvidenceSourceRecord {
        version: String::new(),
        evidence_id: "value_source".into(),
        kind: kind.into(),
        case_id: "case_other".into(),
        originating_case_id: "case_source".into(),
        evidence_refs: vec!["evi_value_source".into()],
        source_run_id: "run_value_source".into(),
        settlement_ref: "settlement_value_source".into(),
        created_at: "now".into(),
        record_hash: String::new(),
    }
}
