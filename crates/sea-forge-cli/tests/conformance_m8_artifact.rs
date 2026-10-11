use sea_forge_artifact_ip::{
    append_domain_model_ref, append_registration, append_rights_profile, append_value_evidence,
    append_value_evidence_source, authorize_rights_profile, canonical_rights_action,
    seal_pending_transition, store_gate_profile, ArtifactGateProfile, IdentityStatus,
    PendingArtifactTransition, RightsProfileRecord, SemanticAnchor, SemanticReferenceClass,
    TransitionInput, TransitionKind, TransitionMode, TransitionProposal, ValueEvidenceRecord,
    ValueEvidenceRef, ValueEvidenceSourceRecord, VerifiedArtifactRegistrationInput,
};
use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{
    Actor, ActorRole, ApprovalRequest, ArtifactDescriptor, ArtifactProducer, ArtifactStage,
    ArtifactType, EvidenceKind, EvidenceRecord, ReviewStatus,
};
use sea_forge_ledger::LedgerStream;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn descriptor() -> ArtifactDescriptor {
    ArtifactDescriptor {
        artifact_id: "art_model".into(),
        artifact_type: ArtifactType::SeaModel,
        stage: Some(ArtifactStage::Intellectual),
        producer: ArtifactProducer {
            entity_id: "operator_local".into(),
            process_id: "cli".into(),
            run_id: "run_source".into(),
            plan_item_id: "item".into(),
        },
        owner: "operator_local".into(),
        license: "internal".into(),
        review_status: ReviewStatus::Approved,
        source_refs: vec![],
        content_sha256: "abc".into(),
        pre_mint_identity: "ifl:hash:legacy".into(),
    }
}

fn evidence() -> EvidenceRecord {
    let descriptor = descriptor();
    EvidenceRecord {
        version: "0.2".into(),
        evidence_id: "evi_artifact".into(),
        run_id: "run_source".into(),
        kind: EvidenceKind::Artifact,
        uri: "artifacts/model.sea".into(),
        sha256: Some("abc".into()),
        source_event_id: "evt".into(),
        metadata: BTreeMap::from([("artifact".into(), serde_json::to_value(descriptor).unwrap())]),
        cell_id: None,
    }
}

fn commit_evidence(root: &Path) -> String {
    let ev = evidence();
    LedgerStream::open(root, "artifact-ip", "operator_local")
        .unwrap()
        .commit_typed(
            "artifact_evidence",
            vec![ev.evidence_id.clone()],
            &ev,
            vec![],
        )
        .unwrap();
    ev.evidence_id
}

fn policy(path: &Path) {
    let bin = env!("CARGO_BIN_EXE_sea-forge");
    fs::write(
        path,
        format!(
            r#"version: "0.1"
identity:
  source: test
  allow_unresolved: false
identity_bindings:
  - principal: operator_local
    actor_type: human
    role: operator
  - principal: security_officer
    actor_type: human
    role: R-SO
sod_rules:
  - name: capitalization_requester_approver
    requester_role: operator
    approver_role: R-SO
    operation_kind: transition_artifact_stage
    transition_kind: capitalize
    allow_same_principal: false
settlement_authorities:
  - authority_ref: swe_seed_test
    adapter: swe_seed
    command: ["{bin}", "internal-test-swe-seed"]
    timeout_secs: 5
    trust_anchor_ref: test-anchor
    declarer_actor_id: swe_seed_verifier
    permitted_declarer_roles: [R-AA]
    standing_basis: test-attested-verifier
    may_issue_strong: true
rules:
  - name: transition
    verdict: allow
    actor_role: operator
    operation_kind: transition_artifact_stage
    transition_kind: synthesize
    from_stage: cognitive
    to_stage: intellectual
    modes: [promote]
    gate_profile_ref: synthesize@1
    required_settlement_strength: local
    qualifying_value_evidence_kinds: []
    requires_approval: false
    license_allowlist: [internal]
  - name: write
    verdict: allow
    actor_role: operator
    operation_kind: write_file
    path_prefix: ""
  - name: evaluate
    verdict: allow
    actor_role: operator
    operation_kind: execute_command
    argv0: sea-forge
    environment: transition_env@0.1.0
  - name: attest
    verdict: allow
    actor_role: operator
    operation_kind: attest_artifact_identity
    ledger: ifl
    requester_roles: [operator]
    approver_roles: [R-SO]
    degraded_mode: forbidden
  - name: resolve-approval
    verdict: allow
    actor_role: R-SO
    operation_kind: approval_resolution
"#
        ),
    )
    .unwrap();
}

fn profile() -> ArtifactGateProfile {
    ArtifactGateProfile {
        version: "0.2".into(),
        profile_ref: "synthesize@1".into(),
        artifact_types: vec![ArtifactType::SeaModel],
        transition_kind: TransitionKind::Synthesize,
        required_metadata: vec![],
        evaluator_refs: vec!["transition_env@0.1.0.stage_gate".into()],
        evaluator_thresholds: BTreeMap::from([("transition_env@0.1.0.stage_gate".into(), 1.0)]),
        semantic_reference_classes: vec![],
        required_rights_review: vec![],
        required_settlement_strength: Some("local".into()),
        qualifying_value_evidence_kinds: vec![],
        requires_approval: false,
        attestation_mode: None,
    }
}

fn environment(root: &Path, bin: &Path, succeeds: bool) {
    let path = root.join("environments/transition_env@0.1.0.yaml");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let argv = if succeeds {
        vec![
            bin.display().to_string(),
            "internal-test-sleep".into(),
            "0".into(),
        ]
    } else {
        vec![bin.display().to_string(), "internal-test-sleep".into()]
    };
    fs::write(
        path,
        serde_yaml::to_string(&sea_forge_sandbox::EnvironmentSpec {
            name: "transition_env".into(),
            version: "0.1.0".into(),
            description: String::new(),
            base: vec![],
            provides: sea_forge_sandbox::environment::Provides {
                commands: vec!["sea-forge".into()],
            },
            evaluators: BTreeMap::from([(
                "stage_gate".into(),
                sea_forge_sandbox::Evaluator::Command {
                    argv,
                    score_from: sea_forge_sandbox::environment::ScoreFrom::Exit,
                },
            )]),
        })
        .unwrap(),
    )
    .unwrap();
}

fn input(
    registration: &sea_forge_artifact_ip::ArtifactRegistrationRecord,
    hash: String,
    kind: TransitionKind,
) -> TransitionInput {
    let (from_stage, to_stage) = match kind {
        TransitionKind::Synthesize => (ArtifactStage::Cognitive, ArtifactStage::Intellectual),
        TransitionKind::Productize => (ArtifactStage::Intellectual, ArtifactStage::Product),
        TransitionKind::Capitalize => (ArtifactStage::Product, ArtifactStage::Capital),
    };
    TransitionInput {
        transition_token_id: "tok_synth".into(),
        transition_kind: kind,
        mode: TransitionMode::Promote,
        from_stage,
        to_stage,
        source_artifact_ids: vec![registration.artifact_id.clone()],
        result_artifact_id: registration.artifact_id.clone(),
        derived_from: vec![],
        input_content_identities: vec![registration.content_identity.clone()],
        output_content_identity: registration.content_identity.clone(),
        parent_transition_token_ids: vec![],
        gate_profile_ref: "synthesize@1".into(),
        gate_profile_hash: hash,
        actor_id: "operator_local".into(),
        approver_id: None,
        approval_ref: None,
        authority_decision_refs: vec!["placeholder".into()],
        criteria_ref: "placeholder".into(),
        evidence_refs: vec![],
        settlement_ref: "placeholder".into(),
        strong_declaration_refs: vec![],
        value_evidence_refs: vec![],
        value_evidence: vec![],
        rights_profile_ref: None,
        semantic_refs: vec![],
        semantic_model_ref: None,
        identity_status_before: IdentityStatus::PreMint,
        identity_status_after: IdentityStatus::PreMint,
        attestation_ref: None,
        degraded_controls: vec![],
        case_id: "case_source".into(),
        run_id: "run_source".into(),
        plan_item_id: "transition".into(),
        created_at: "now".into(),
    }
}

fn proposal(
    registration: &sea_forge_artifact_ip::ArtifactRegistrationRecord,
    hash: String,
    kind: TransitionKind,
) -> TransitionProposal {
    TransitionProposal::from(&input(registration, hash, kind))
}

fn run_governed_transition(bin: &Path, root: &Path, policy: &Path, kind: &str, input_path: &Path) {
    let output = Command::new(bin)
        .args([
            "artifact",
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            kind,
            input_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{} {}",
        kind,
        String::from_utf8_lossy(&output.stderr)
    );
}

fn last_transition_token_id(root: &Path) -> String {
    let ledger = fs::read_to_string(root.join("ledgers/artifact-ip/entries.jsonl")).unwrap();
    let last = ledger
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .rfind(|entry| entry["record_kind"] == "artifact_transition")
        .unwrap();
    last["payload"]["transition_token_id"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn append_authorized_rights_profile(
    root: &Path,
    registration: &sea_forge_artifact_ip::ArtifactRegistrationRecord,
) {
    let action = canonical_rights_action(registration, &["approved".into()]);
    let bundle: AuthorityPolicyBundle = serde_yaml::from_str(
        "version: \"0.1\"\nrules:\n  - name: rights\n    verdict: allow\n    actor_role: operator\n    operation_kind: review_artifact_rights\n",
    )
    .unwrap();
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let actor = Actor {
        actor_id: "operator_local".into(),
        role: ActorRole::Operator,
    };
    let decision = engine
        .evaluate(AuthorityEvaluation {
            actor: &actor,
            binding: bundle.resolve_identity(&actor.actor_id, actor.role.clone()),
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
    let committed = LedgerStream::open(root, "case-governance", "operator_local")
        .unwrap()
        .commit_typed("authority_decision", vec![], &decision, vec![])
        .unwrap();
    let grant = engine.grant(&decision, &committed, &action, None).unwrap();
    let authorization = authorize_rights_profile(
        grant,
        &action,
        root,
        "operator_local",
        registration,
        &["approved".into()],
        &decision,
        &committed,
    )
    .unwrap();
    append_rights_profile(
        authorization,
        RightsProfileRecord {
            version: String::new(),
            rights_profile_id: "rights@1".into(),
            artifact_id: registration.artifact_id.clone(),
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
}

fn append_validated_domain_model(root: &Path) -> String {
    let content = "@namespace \"demo\"\n@version \"1.0.0\"\nEntity \"concept:artifact\" in demo\nResource \"class:artifact\" units in demo\nFlow \"class:artifact\" from \"concept:artifact\" to \"concept:artifact\" quantity 1\n";
    let model = sea_forge_domainforge::load_validate(&sea_forge_domainforge::SeaSourceSet {
        entry_uri: "demo.sea".into(),
        files: vec![sea_forge_domainforge::SourceFile {
            uri: "demo.sea".into(),
            sha256: "418a156fd7e1237309bf4bdae37cb6a96993062a76d98bca8f950330fb59a4cc".into(),
            content: content.into(),
        }],
    })
    .unwrap();
    for source in &model.model_ref.source_refs {
        LedgerStream::open(root, "artifact-ip", "operator_local")
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
    append_domain_model_ref(root, "operator_local", &model).unwrap();
    model.model_ref.semantic_model_sha256
}

#[test]
fn conformance_m8_cli_rejects_caller_governance_refs_before_side_effects() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("state");
    let policy_path = temp.path().join("policy.yaml");
    policy(&policy_path);
    let input_path = temp.path().join("hostile.json");
    fs::write(
        &input_path,
        serde_json::json!({
            "transition_token_id": "tok_hostile",
            "transition_kind": "synthesize",
            "mode": "promote",
            "from_stage": "cognitive",
            "to_stage": "intellectual",
            "source_artifact_ids": ["art_model"],
            "result_artifact_id": "art_model",
            "derived_from": [],
            "input_content_identities": ["sha256:content"],
            "output_content_identity": "sha256:content",
            "parent_transition_token_ids": [],
            "gate_profile_ref": "synthesize@1",
            "gate_profile_hash": "sha256:profile",
            "value_evidence_refs": [],
            "value_evidence": [],
            "rights_profile_ref": null,
            "semantic_refs": [],
            "semantic_model_ref": null,
            "identity_status_before": "pre_mint",
            "identity_status_after": "pre_mint",
            "attestation_ref": null,
            "degraded_controls": [],
            "criteria_ref": "caller-controlled"
        })
        .to_string(),
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "artifact",
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy_path.to_str().unwrap(),
            "synthesize",
            input_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        !root.exists(),
        "schema rejection must precede all state effects"
    );
}

#[test]
fn conformance_m8_cli_capitalize_commits_one_pending_and_approval_then_exits_5() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("state");
    let policy_path = temp.path().join("policy.yaml");
    fs::write(
        &policy_path,
        format!(
            r#"version: "0.1"
identity:
  source: test
  allow_unresolved: false
identity_bindings:
  - principal: operator_local
    actor_type: human
    role: operator
  - principal: security_officer
    actor_type: human
    role: R-SO
sod_rules:
  - name: capitalization_requester_approver
    requester_role: operator
    approver_role: R-SO
    operation_kind: transition_artifact_stage
    transition_kind: capitalize
    allow_same_principal: false
settlement_authorities:
  - authority_ref: swe_seed_test
    adapter: swe_seed
    command: ["{}", "internal-test-swe-seed"]
    timeout_secs: 5
    trust_anchor_ref: test-anchor
    declarer_actor_id: swe_seed_verifier
    permitted_declarer_roles: [R-AA]
    standing_basis: test-attested-verifier
    may_issue_strong: true
rules:
  - name: synthesize
    verdict: allow
    actor_role: operator
    operation_kind: transition_artifact_stage
    transition_kind: synthesize
    from_stage: cognitive
    to_stage: intellectual
    modes: [promote]
    gate_profile_ref: synthesize@1
    required_settlement_strength: local
    qualifying_value_evidence_kinds: []
    requires_approval: false
    license_allowlist: [internal]
  - name: productize
    verdict: allow
    actor_role: operator
    operation_kind: transition_artifact_stage
    transition_kind: productize
    from_stage: intellectual
    to_stage: product
    modes: [promote]
    gate_profile_ref: productize@1
    required_settlement_strength: local
    qualifying_value_evidence_kinds: []
    requires_approval: false
    license_allowlist: [internal]
  - name: capitalize
    verdict: allow
    actor_role: operator
    operation_kind: transition_artifact_stage
    transition_kind: capitalize
    from_stage: product
    to_stage: capital
    modes: [promote]
    gate_profile_ref: capitalize@1
    required_settlement_strength: strong
    qualifying_value_evidence_kinds: [adoption]
    requires_approval: true
    license_allowlist: [internal]
  - name: evaluate
    verdict: allow
    actor_role: operator
    operation_kind: execute_command
    argv0: sea-forge
    environment: transition_env@0.1.0
  - name: resolve-approval
    verdict: allow
    actor_role: R-SO
    operation_kind: approval_resolution
"#,
            env!("CARGO_BIN_EXE_sea-forge")
        ),
    )
    .unwrap();
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_sea-forge"));
    environment(&root, &bin, true);
    let registration = append_registration(
        &root,
        "operator_local",
        &commit_evidence(&root).clone(),
        VerifiedArtifactRegistrationInput {
            name: "model.sea".into(),
            version: "1".into(),
            case_id: "case_source".into(),
            derived_from: vec![],
            created_at: "now".into(),
        },
    )
    .unwrap();
    let semantic_model_ref = append_validated_domain_model(&root);
    let synth_hash = store_gate_profile(&root, &profile()).unwrap();
    let synth_path = temp.path().join("synthesize.json");
    fs::write(
        &synth_path,
        serde_json::to_vec(&proposal(
            &registration,
            synth_hash,
            TransitionKind::Synthesize,
        ))
        .unwrap(),
    )
    .unwrap();
    run_governed_transition(&bin, &root, &policy_path, "synthesize", &synth_path);
    let synth_token_id = last_transition_token_id(&root);

    let mut productize_profile = profile();
    productize_profile.profile_ref = "productize@1".into();
    productize_profile.transition_kind = TransitionKind::Productize;
    let productize_hash = store_gate_profile(&root, &productize_profile).unwrap();
    let mut productize_input = input(&registration, productize_hash, TransitionKind::Productize);
    productize_input.gate_profile_ref = "productize@1".into();
    productize_input.transition_token_id = "tok_product".into();
    productize_input.parent_transition_token_ids = vec![synth_token_id];
    let productize_path = temp.path().join("productize.json");
    fs::write(
        &productize_path,
        serde_json::to_vec(&TransitionProposal::from(&productize_input)).unwrap(),
    )
    .unwrap();
    run_governed_transition(&bin, &root, &policy_path, "productize", &productize_path);
    let product_token_id = last_transition_token_id(&root);

    let mut gate = profile();
    gate.profile_ref = "capitalize@1".into();
    gate.transition_kind = TransitionKind::Capitalize;
    gate.required_settlement_strength = Some("strong".into());
    gate.qualifying_value_evidence_kinds = vec!["adoption".into()];
    gate.required_rights_review = vec!["approved".into()];
    gate.semantic_reference_classes = vec![SemanticReferenceClass::Concept];
    gate.requires_approval = true;
    let hash = store_gate_profile(&root, &gate).unwrap();
    let mut capital = input(&registration, hash, TransitionKind::Capitalize);
    capital.gate_profile_ref = "capitalize@1".into();
    capital.transition_token_id = "tok_capital".into();
    capital.parent_transition_token_ids = vec![product_token_id];
    capital.value_evidence_refs = vec!["value_adoption".into()];
    capital.value_evidence = vec![ValueEvidenceRef {
        reference: "value_adoption".into(),
        case_id: "case_other".into(),
        kind: "adoption".into(),
    }];
    capital.rights_profile_ref = Some("rights@1".into());
    capital.semantic_refs = vec![SemanticAnchor {
        class: SemanticReferenceClass::Concept,
        reference: "concept:artifact".into(),
    }];
    capital.semantic_model_ref = Some(semantic_model_ref);
    let input_path = temp.path().join("capitalize.json");
    fs::write(
        &input_path,
        serde_json::to_vec(&TransitionProposal::from(&capital)).unwrap(),
    )
    .unwrap();

    let output = Command::new(&bin)
        .args([
            "artifact",
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy_path.to_str().unwrap(),
            "capitalize",
            input_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(5),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let case_id = String::from_utf8_lossy(&output.stdout)
        .lines()
        .find_map(|line| line.strip_prefix("case_id="))
        .unwrap()
        .to_owned();
    let case_ledger = root.join("ledgers").join(format!("case-{case_id}"));
    let entries = fs::read_to_string(case_ledger.join("entries.jsonl")).unwrap();
    let records = entries
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        records
            .iter()
            .filter(|entry| entry["record_kind"] == "pending_artifact_transition")
            .count(),
        1
    );
    assert_eq!(
        records
            .iter()
            .filter(|entry| entry["record_kind"] == "approval_request")
            .count(),
        1
    );
    let decision = records
        .iter()
        .find(|entry| {
            entry["record_kind"] == "authority_decision"
                && entry["payload"]["operation"]["resource_type"] == "transition_artifact_stage"
        })
        .unwrap();
    let pending = records
        .iter()
        .find(|entry| entry["record_kind"] == "pending_artifact_transition")
        .unwrap();
    let criteria = records
        .iter()
        .find(|entry| entry["record_kind"] == "settlement_criteria")
        .unwrap();
    let approval = records
        .iter()
        .find(|entry| entry["record_kind"] == "approval_request")
        .unwrap();
    assert_eq!(decision["payload"]["verdict"], "escalate");
    assert_eq!(
        decision["payload"]["operation"]["parameters"]["transition_kind"],
        "capitalize"
    );
    assert_eq!(
        pending["payload"]["case_id"],
        decision["payload"]["audit_record"]["case_id"]
    );
    assert_eq!(pending["payload"]["run_id"], decision["payload"]["run_id"]);
    assert_eq!(
        pending["payload"]["authority_decision_ref"],
        decision["payload"]["decision_id"]
    );
    assert_eq!(
        pending["payload"]["authority_decision_hash"],
        decision["payload_hash"]
    );
    assert_eq!(
        pending["payload"]["approval_ref"],
        approval["payload"]["approval_id"]
    );
    assert_eq!(
        pending["payload"]["criteria_sha256"],
        criteria["payload"]["criteria_sha256"]
    );
    assert_eq!(
        pending["payload"]["criteria_record_hash"],
        criteria["payload"]["criteria_record_hash"]
    );

    let source_run_id = "run_adoption_source";
    let source_settlement_ref = "settlement_adoption_source";
    let source_evidence_ref = "evi_adoption_source";
    let source_stream = LedgerStream::open(&root, "case-case_other", "operator_local").unwrap();
    source_stream
        .commit_typed(
            "run_evidence",
            vec![source_evidence_ref.into()],
            &EvidenceRecord {
                version: "0.2".into(),
                evidence_id: source_evidence_ref.into(),
                run_id: source_run_id.into(),
                kind: EvidenceKind::ExecutionResult,
                uri: "runs/run_adoption_source/evidence.json".into(),
                sha256: Some("sha256:adoption".into()),
                source_event_id: "evt_adoption".into(),
                metadata: BTreeMap::from([(
                    "canonical_value_evidence_kind".into(),
                    "adoption".into(),
                )]),
                cell_id: None,
            },
            vec![],
        )
        .unwrap();
    source_stream
        .commit_typed(
            "settlement_event",
            vec![source_settlement_ref.into()],
            &serde_json::json!({
                "version": "0.2", "settlement_id": source_settlement_ref,
                "run_id": source_run_id, "status": "accepted",
                "basis": ["source_value_demonstrated"], "review_required": false,
                "settled_at": "now", "criteria_ref": "criteria_source"
            }),
            vec![],
        )
        .unwrap();
    append_value_evidence_source(
        &root,
        "operator_local",
        ValueEvidenceSourceRecord {
            version: String::new(),
            evidence_id: "value_adoption_source".into(),
            kind: "adoption".into(),
            case_id: "case_other".into(),
            originating_case_id: case_id.clone(),
            evidence_refs: vec![source_evidence_ref.into()],
            source_run_id: source_run_id.into(),
            settlement_ref: source_settlement_ref.into(),
            created_at: "now".into(),
            record_hash: String::new(),
        },
    )
    .unwrap();
    append_value_evidence(
        &root,
        "operator_local",
        ValueEvidenceRecord {
            version: String::new(),
            value_evidence_id: "value_adoption".into(),
            kind: "adoption".into(),
            case_id: "case_other".into(),
            evidence_refs: vec!["value_adoption_source".into()],
            accepted: false,
            created_at: "now".into(),
            record_hash: String::new(),
        },
    )
    .unwrap();
    append_authorized_rights_profile(&root, &registration);
    let rights = LedgerStream::open(&root, "artifact-ip", "operator_local")
        .unwrap()
        .read_entries()
        .unwrap()
        .into_iter()
        .find(|entry| entry.record_kind == "artifact_rights_profile")
        .unwrap();
    assert_eq!(
        rights.payload["authority_action_hash"],
        sea_forge_ledger::types::hash_canonical(&canonical_rights_action(
            &registration,
            &["approved".into()],
        ))
        .unwrap(),
    );
    let approval_id = approval["payload"]["approval_id"].as_str().unwrap();
    let approved = Command::new(&bin)
        .args([
            "approve",
            &case_id,
            approval_id,
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy_path.to_str().unwrap(),
            "--actor",
            "security_officer",
        ])
        .output()
        .unwrap();
    assert!(approved.status.success());
    let resumed = Command::new(&bin)
        .args([
            "resume",
            &case_id,
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        resumed.status.success(),
        "{}",
        String::from_utf8_lossy(&resumed.stderr)
    );
    let case_entries = fs::read_to_string(case_ledger.join("entries.jsonl")).unwrap();
    assert_eq!(
        case_entries
            .matches("\"record_kind\":\"artifact_transition_claim_manifest\"")
            .count(),
        1
    );
    assert_eq!(
        case_entries
            .matches("\"record_kind\":\"settlement_declaration\"")
            .count(),
        1
    );
    let artifact_entries =
        fs::read_to_string(root.join("ledgers/artifact-ip/entries.jsonl")).unwrap();
    assert_eq!(
        artifact_entries
            .matches("\"record_kind\":\"artifact_transition\"")
            .count(),
        3
    );
}

#[test]
fn conformance_m8_cli_transition_uses_completed_case_and_rejects_teleport_without_case() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("state");
    let policy_path = temp.path().join("policy.yaml");
    policy(&policy_path);
    let registration = append_registration(
        &root,
        "operator_local",
        &commit_evidence(&root).clone(),
        VerifiedArtifactRegistrationInput {
            name: "model.sea".into(),
            version: "1".into(),
            case_id: "case_source".into(),
            derived_from: vec![],
            created_at: "now".into(),
        },
    )
    .unwrap();
    let hash = store_gate_profile(&root, &profile()).unwrap();
    let input_path = temp.path().join("input.json");
    fs::write(
        &input_path,
        serde_json::to_vec(&proposal(&registration, hash, TransitionKind::Synthesize)).unwrap(),
    )
    .unwrap();
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_sea-forge"));
    environment(&root, &bin, true);
    let unauthorized = Command::new(&bin)
        .args([
            "artifact",
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy_path.to_str().unwrap(),
            "attest",
            &registration.artifact_id,
            "--requester-role",
            "R-AA",
        ])
        .output()
        .unwrap();
    assert!(!unauthorized.status.success());
    assert!(!root.join("ledgers/ifl/entries.jsonl").exists());
    let authorized = Command::new(&bin)
        .args([
            "artifact",
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy_path.to_str().unwrap(),
            "attest",
            &registration.artifact_id,
        ])
        .output()
        .unwrap();
    assert!(
        authorized.status.success(),
        "{}",
        String::from_utf8_lossy(&authorized.stderr)
    );
    assert!(fs::read_to_string(root.join("ledgers/ifl/entries.jsonl"))
        .unwrap()
        .contains("artifact_identity_attestation"));
    let allowed = Command::new(&bin)
        .args([
            "artifact",
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy_path.to_str().unwrap(),
            "synthesize",
            input_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        allowed.status.success(),
        "{}",
        String::from_utf8_lossy(&allowed.stderr)
    );
    let cases = fs::read_dir(root.join("cases")).unwrap().count();
    assert_eq!(cases, 1);
    let artifact_ledger =
        fs::read_to_string(root.join("ledgers/artifact-ip/entries.jsonl")).unwrap();
    let transition_entry = artifact_ledger
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .find(|entry| entry["record_kind"] == "artifact_transition")
        .unwrap();
    assert_eq!(
        transition_entry["payload"]["authority_decision_refs"],
        serde_json::json!(["auth_02"])
    );

    let bad_path = temp.path().join("bad.json");
    let mut bad = input(
        &registration,
        store_gate_profile(&root, &profile()).unwrap(),
        TransitionKind::Productize,
    );
    bad.from_stage = ArtifactStage::Cognitive;
    bad.to_stage = ArtifactStage::Product;
    fs::write(
        &bad_path,
        serde_json::to_vec(&TransitionProposal::from(&bad)).unwrap(),
    )
    .unwrap();
    let denied = Command::new(&bin)
        .args([
            "artifact",
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy_path.to_str().unwrap(),
            "productize",
            bad_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!denied.status.success());
    assert_eq!(fs::read_dir(root.join("cases")).unwrap().count(), cases);
}

#[test]
fn conformance_m8_cli_profile_evaluator_controls_token_commit() {
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_sea-forge"));
    for succeeds in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("state");
        let policy_path = temp.path().join("policy.yaml");
        policy(&policy_path);
        environment(&root, &bin, succeeds);
        let registration = append_registration(
            &root,
            "operator_local",
            &commit_evidence(&root).clone(),
            VerifiedArtifactRegistrationInput {
                name: "model.sea".into(),
                version: "1".into(),
                case_id: "case_source".into(),
                derived_from: vec![],
                created_at: "now".into(),
            },
        )
        .unwrap();
        let hash = store_gate_profile(&root, &profile()).unwrap();
        let input_path = temp.path().join("input.json");
        fs::write(
            &input_path,
            serde_json::to_vec(&proposal(&registration, hash, TransitionKind::Synthesize)).unwrap(),
        )
        .unwrap();

        let output = Command::new(&bin)
            .args([
                "artifact",
                "--root",
                root.to_str().unwrap(),
                "--policy",
                policy_path.to_str().unwrap(),
                "synthesize",
                input_path.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert_eq!(
            output.status.success(),
            succeeds,
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );

        let artifact_ledger =
            fs::read_to_string(root.join("ledgers/artifact-ip/entries.jsonl")).unwrap();
        assert_eq!(artifact_ledger.contains("artifact_transition"), succeeds);
        let case_ledger = fs::read_dir(root.join("ledgers"))
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .find(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("case-case_"))
            })
            .unwrap();
        let entries = fs::read_to_string(case_ledger.join("entries.jsonl")).unwrap();
        assert!(entries.contains(&format!(
            "evaluator_score:transition_env@0.1.0.stage_gate={}",
            u8::from(succeeds)
        )));
        assert!(entries.contains("evi_artifact"));
        let case_dir = fs::read_dir(root.join("cases"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let plan: serde_json::Value =
            serde_json::from_slice(&fs::read(case_dir.join("plan.json")).unwrap()).unwrap();
        assert_eq!(plan["items"][0]["operations"], serde_json::json!([]));
        assert_eq!(
            plan["items"][0]["settlement_criteria"]["evaluator"],
            "transition_env@0.1.0.stage_gate"
        );
        assert_eq!(plan["items"][0]["environment"], "transition_env@0.1.0");
        assert!(!serde_json::to_string(&plan)
            .unwrap()
            .contains("transition.ok"));
    }
}

#[test]
fn conformance_m8_cli_parks_external_gate_profiles_without_fabricating_records() {
    for (requires_approval, expires) in [(true, false), (true, true), (false, false)] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("state");
        let policy_path = temp.path().join("policy.yaml");
        policy(&policy_path);
        let bin = PathBuf::from(env!("CARGO_BIN_EXE_sea-forge"));
        environment(&root, &bin, true);
        let registration = append_registration(
            &root,
            "operator_local",
            &commit_evidence(&root).clone(),
            VerifiedArtifactRegistrationInput {
                name: "model.sea".into(),
                version: "1".into(),
                case_id: "case_source".into(),
                derived_from: vec![],
                created_at: "now".into(),
            },
        )
        .unwrap();
        let mut gate = profile();
        gate.requires_approval = requires_approval;
        gate.required_settlement_strength = (!requires_approval).then(|| "strong".into());
        let hash = store_gate_profile(&root, &gate).unwrap();
        let mut policy_text = fs::read_to_string(&policy_path)
            .unwrap()
            .replace("requires_approval: false", "requires_approval: true");
        let outage_file = temp.path().join("swe-seed.outage");
        if gate.required_settlement_strength.as_deref() == Some("strong") {
            policy_text = policy_text.replace(
                "required_settlement_strength: local",
                "required_settlement_strength: strong",
            );
            policy_text = policy_text.replace(
                "internal-test-swe-seed\"]",
                &format!(
                    "internal-test-swe-seed\", \"--outage-file\", \"{}\"]",
                    outage_file.display()
                ),
            );
            fs::write(&outage_file, b"outage").unwrap();
        }
        fs::write(&policy_path, policy_text).unwrap();
        let input_path = temp.path().join("input.json");
        fs::write(
            &input_path,
            serde_json::to_vec(&proposal(&registration, hash, TransitionKind::Synthesize)).unwrap(),
        )
        .unwrap();

        let output = Command::new(&bin)
            .args([
                "artifact",
                "--root",
                root.to_str().unwrap(),
                "--policy",
                policy_path.to_str().unwrap(),
                "synthesize",
                input_path.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(5),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("case_state=awaiting_approval"));
        let case_id = String::from_utf8_lossy(&output.stdout)
            .lines()
            .find_map(|line| line.strip_prefix("case_id="))
            .unwrap()
            .to_owned();
        assert_eq!(fs::read_dir(root.join("cases")).unwrap().count(), 1);
        let case_ledger = fs::read_dir(root.join("ledgers"))
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .find(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("case-case_"))
            })
            .unwrap();
        let entries = fs::read_to_string(case_ledger.join("entries.jsonl")).unwrap();
        let record_kinds = entries
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
            .map(|entry| entry["record_kind"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>();
        assert_eq!(
            record_kinds
                .iter()
                .filter(|kind| *kind == "pending_artifact_transition")
                .count(),
            1
        );
        assert_eq!(
            record_kinds
                .iter()
                .filter(|kind| *kind == "approval_request")
                .count(),
            1
        );
        let artifact_ledger =
            fs::read_to_string(root.join("ledgers/artifact-ip/entries.jsonl")).unwrap();
        assert!(!artifact_ledger.contains("artifact_transition"));
        assert!(!artifact_ledger.contains("settlement_declaration"));

        if gate.required_settlement_strength.as_deref() == Some("strong") {
            let approval_id = entries
                .lines()
                .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
                .find(|entry| entry["record_kind"] == "approval_request")
                .unwrap()["payload"]["approval_id"]
                .as_str()
                .unwrap()
                .to_owned();
            let approved = Command::new(&bin)
                .args([
                    "approve",
                    &case_id,
                    &approval_id,
                    "--root",
                    root.to_str().unwrap(),
                    "--policy",
                    policy_path.to_str().unwrap(),
                    "--actor",
                    "security_officer",
                ])
                .output()
                .unwrap();
            assert!(
                approved.status.success(),
                "{}",
                String::from_utf8_lossy(&approved.stderr)
            );
            let resume_args = [
                "resume",
                &case_id,
                "--root",
                root.to_str().unwrap(),
                "--policy",
                policy_path.to_str().unwrap(),
            ];
            let parked = Command::new(&bin).args(resume_args).output().unwrap();
            assert_eq!(parked.status.code(), Some(5));
            let outage_case_entries =
                fs::read_to_string(case_ledger.join("entries.jsonl")).unwrap();
            assert!(!outage_case_entries.contains("\"record_kind\":\"settlement_declaration\""));
            assert!(
                !fs::read_to_string(root.join("ledgers/artifact-ip/entries.jsonl"))
                    .unwrap()
                    .contains("\"record_kind\":\"artifact_transition\"")
            );
            fs::remove_file(&outage_file).unwrap();
            let resumed = Command::new(&bin).args(resume_args).output().unwrap();
            assert!(
                resumed.status.success(),
                "{}",
                String::from_utf8_lossy(&resumed.stderr)
            );
            assert!(String::from_utf8_lossy(&resumed.stdout).contains("case_state=completed"));
            let replayed = Command::new(&bin).args(resume_args).output().unwrap();
            assert!(
                replayed.status.success(),
                "{}",
                String::from_utf8_lossy(&replayed.stderr)
            );
            let case_entries = fs::read_to_string(case_ledger.join("entries.jsonl")).unwrap();
            assert_eq!(
                case_entries
                    .matches("\"record_kind\":\"artifact_transition_claim_manifest\"")
                    .count(),
                1
            );
            assert_eq!(
                case_entries
                    .matches("\"record_kind\":\"settlement_declaration\"")
                    .count(),
                1
            );
            assert_eq!(
                case_entries
                    .matches("\"record_kind\":\"artifact_transition_terminal\"")
                    .count(),
                1
            );
            let artifact_entries =
                fs::read_to_string(root.join("ledgers/artifact-ip/entries.jsonl")).unwrap();
            assert_eq!(
                artifact_entries
                    .matches("\"record_kind\":\"artifact_transition\"")
                    .count(),
                1
            );
        } else {
            let approval_id = entries
                .lines()
                .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
                .find(|entry| entry["record_kind"] == "approval_request")
                .unwrap()["payload"]["approval_id"]
                .as_str()
                .unwrap()
                .to_owned();
            if expires {
                let mut resolution: sea_forge_core::types::ApprovalRequest = entries
                    .lines()
                    .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
                    .find(|entry| entry["record_kind"] == "approval_request")
                    .map(|entry| serde_json::from_value(entry["payload"].clone()).unwrap())
                    .unwrap();
                resolution.status = sea_forge_core::types::ApprovalStatus::Expired;
                resolution.resolved_at = Some(resolution.expires_at.clone());
                LedgerStream::open(&root, format!("case-{case_id}"), "security_officer")
                    .unwrap()
                    .commit_typed_once(
                        "approval_resolution",
                        &approval_id,
                        vec![case_id.clone(), approval_id.clone()],
                        &resolution,
                        vec![resolution.decision_id.clone()],
                    )
                    .unwrap();
            } else {
                let rejected = Command::new(&bin)
                    .args([
                        "reject",
                        &case_id,
                        &approval_id,
                        "--root",
                        root.to_str().unwrap(),
                        "--policy",
                        policy_path.to_str().unwrap(),
                        "--actor",
                        "security_officer",
                    ])
                    .output()
                    .unwrap();
                assert!(rejected.status.success());
            }
            let resumed = Command::new(&bin)
                .args([
                    "resume",
                    &case_id,
                    "--root",
                    root.to_str().unwrap(),
                    "--policy",
                    policy_path.to_str().unwrap(),
                ])
                .output()
                .unwrap();
            assert_eq!(resumed.status.code(), Some(4));
            let case_entries = fs::read_to_string(case_ledger.join("entries.jsonl")).unwrap();
            assert!(case_entries.contains(if expires {
                "approval_expired"
            } else {
                "approval_rejected"
            }));
            assert!(!case_entries.contains("\"record_kind\":\"settlement_declaration\""));
            assert!(
                !fs::read_to_string(root.join("ledgers/artifact-ip/entries.jsonl"))
                    .unwrap()
                    .contains("\"record_kind\":\"artifact_transition\"")
            );
        }
    }
}

#[test]
fn conformance_m8_resume_expired_pending_approval_terminalizes_once() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("state");
    let source_root = temp.path().join("source-state");
    let policy_path = temp.path().join("policy.yaml");
    policy(&policy_path);
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_sea-forge"));
    environment(&source_root, &bin, true);
    let registration = append_registration(
        &source_root,
        "operator_local",
        &commit_evidence(&source_root),
        VerifiedArtifactRegistrationInput {
            name: "model.sea".into(),
            version: "1".into(),
            case_id: "case_source".into(),
            derived_from: vec![],
            created_at: "now".into(),
        },
    )
    .unwrap();
    let mut gate = profile();
    gate.requires_approval = true;
    let hash = store_gate_profile(&source_root, &gate).unwrap();
    let input_path = temp.path().join("input.json");
    fs::write(
        &input_path,
        serde_json::to_vec(&proposal(&registration, hash, TransitionKind::Synthesize)).unwrap(),
    )
    .unwrap();
    let policy_text = fs::read_to_string(&policy_path)
        .unwrap()
        .replace("requires_approval: false", "requires_approval: true");
    fs::write(&policy_path, policy_text).unwrap();
    let parked = Command::new(&bin)
        .args([
            "artifact",
            "--root",
            source_root.to_str().unwrap(),
            "--policy",
            policy_path.to_str().unwrap(),
            "synthesize",
            input_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(parked.status.code(), Some(5));
    let case_id = String::from_utf8(parked.stdout)
        .unwrap()
        .lines()
        .find_map(|line| line.strip_prefix("case_id="))
        .unwrap()
        .to_owned();
    let source_stream =
        LedgerStream::open(&source_root, format!("case-{case_id}"), "test").unwrap();
    let source_entries = source_stream.read_entries().unwrap();
    let mut pending: PendingArtifactTransition = serde_json::from_value(
        source_entries
            .iter()
            .find(|entry| entry.record_kind == "pending_artifact_transition")
            .unwrap()
            .payload
            .clone(),
    )
    .unwrap();
    let mut approval: ApprovalRequest = serde_json::from_value(
        source_entries
            .iter()
            .find(|entry| entry.record_kind == "approval_request")
            .unwrap()
            .payload
            .clone(),
    )
    .unwrap();
    approval.expires_at = "2026-01-01T00:00:00Z".into();
    pending.approval_ref = approval.approval_id.clone();
    seal_pending_transition(&mut pending).unwrap();

    let case_dir = root.join("cases").join(&case_id);
    fs::create_dir_all(&case_dir).unwrap();
    fs::copy(
        source_root.join("cases").join(&case_id).join("case.json"),
        case_dir.join("case.json"),
    )
    .unwrap();
    let stream = LedgerStream::open(&root, format!("case-{case_id}"), "operator_local").unwrap();
    stream
        .commit_typed("approval_request", vec![case_id.clone()], &approval, vec![])
        .unwrap();
    stream
        .commit_typed(
            "pending_artifact_transition",
            vec![case_id.clone(), pending.run_id.clone()],
            &pending,
            vec![pending.authority_decision_ref.clone()],
        )
        .unwrap();

    let resume_args = [
        "resume",
        &case_id,
        "--root",
        root.to_str().unwrap(),
        "--policy",
        policy_path.to_str().unwrap(),
    ];
    let resumed = Command::new(&bin).args(resume_args).output().unwrap();
    assert_eq!(resumed.status.code(), Some(4));
    assert!(String::from_utf8_lossy(&resumed.stdout).contains("case_state=terminated"));
    assert!(!String::from_utf8_lossy(&resumed.stdout).contains("transition_token_id="));
    let entries = fs::read_to_string(
        root.join("ledgers")
            .join(format!("case-{case_id}"))
            .join("entries.jsonl"),
    )
    .unwrap();
    assert_eq!(
        entries
            .matches("\"record_kind\":\"approval_resolution\"")
            .count(),
        1
    );
    assert_eq!(
        entries
            .matches("\"record_kind\":\"artifact_transition_terminal\"")
            .count(),
        1
    );
    assert!(entries.contains("\"status\":\"expired\""));
    let ledger_entries = entries
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    let resolution: ApprovalRequest = serde_json::from_value(
        ledger_entries
            .iter()
            .find(|entry| entry["record_kind"] == "approval_resolution")
            .unwrap()["payload"]
            .clone(),
    )
    .unwrap();
    assert_eq!(resolution.approval_id, approval.approval_id);
    assert_eq!(resolution.decision_id, approval.decision_id);
    assert_eq!(resolution.criteria_ref, approval.criteria_ref);
    assert_eq!(resolution.criteria_sha256, approval.criteria_sha256);
    assert_eq!(
        resolution.criteria_record_hash,
        approval.criteria_record_hash
    );
    let terminal = ledger_entries
        .iter()
        .find(|entry| entry["record_kind"] == "artifact_transition_terminal")
        .unwrap();
    assert!(terminal["payload"]["transition_token_ref"].is_null());
    assert!(terminal["payload"]["declaration_ref"].is_null());

    let replayed = Command::new(&bin).args(resume_args).output().unwrap();
    assert_eq!(replayed.status.code(), Some(4));
    let replayed_entries = fs::read_to_string(
        root.join("ledgers")
            .join(format!("case-{case_id}"))
            .join("entries.jsonl"),
    )
    .unwrap();
    assert_eq!(replayed_entries, entries);
}
