//! M8 transitions use a normal settling case; the token is committed last.

use sea_forge_artifact_ip::{
    append_transition, attest, authorize_attestation, authorize_transition,
    canonical_transition_action, commit_pending_transition, load_ledger_records, read_gate_profile,
    resolve_gate_profile, seal_pending_transition, validate_proposal_before_transform,
    AttestationPolicy, DefaultArtifactAttestor, PendingArtifactTransition, TransitionInput,
    TransitionKind, TransitionProposal,
};
use sea_forge_core::{
    errors::ForgeError,
    ids::random_id,
    types::{
        AuthorityAction, CasePlan, ItemKind, ItemMarkers, PlanItem, SettlementCriteria,
        SettlementEvent, SettlementStatus,
    },
    RECORD_VERSION,
};
use std::{fs, path::Path};

pub fn transition_command(
    root: &Path,
    policy: &Path,
    actor: &str,
    kind: TransitionKind,
    input_path: &Path,
) -> Result<u8, ForgeError> {
    let proposal: TransitionProposal = serde_json::from_slice(
        &fs::read(input_path).map_err(|error| ForgeError::io("read transition input", error))?,
    )?;
    let mut input = proposal
        .clone()
        .into_transition_input(actor.into(), chrono::Utc::now().to_rfc3339());
    if input.transition_kind != kind {
        return Err(ForgeError::Input(
            "artifact_transition_error: command and transition kind differ".into(),
        ));
    }

    // This is deliberately before profile ledgering, authority records, and
    // proposal creation: rejected transitions have no case/run/token effects.
    let (registrations, tokens) = load_ledger_records(root, actor)?;
    let profile = read_gate_profile(root, &input.gate_profile_ref, &input.gate_profile_hash)?;
    validate_proposal_before_transform(&input, &registrations, &tokens, &profile, || Ok(()))?;
    let source_licenses = input
        .source_artifact_ids
        .iter()
        .map(|source| {
            registrations
                .iter()
                .find(|registration| registration.artifact_id == *source)
                .map(|registration| registration.license.clone())
                .ok_or_else(|| ForgeError::Plan {
                    class: "artifact_reference_error",
                    message: format!("transition source registration {source} is missing"),
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    input.evidence_refs.extend(
        registrations
            .iter()
            .filter(|registration| {
                input
                    .source_artifact_ids
                    .contains(&registration.artifact_id)
            })
            .flat_map(|registration| registration.source_evidence_refs.clone()),
    );
    input.evidence_refs.sort();
    input.evidence_refs.dedup();

    resolve_gate_profile(
        root,
        actor,
        &input.gate_profile_ref,
        &input.gate_profile_hash,
    )?;
    let preflight_action = canonical_transition_action(&input, &source_licenses, &profile);
    let requires_external_gate = profile.requires_approval
        || profile.required_settlement_strength.as_deref() == Some("strong");
    if !requires_external_gate {
        super::mediated::authorize_action(root, policy, actor, &preflight_action)?;
    }

    let proposal_id = random_id("plan")?;
    let proposal_path = root
        .join("artifact-transition-plans")
        .join(format!("{proposal_id}.json"));
    let plan = transition_plan(&proposal_id, &input, &profile)?;
    crate::plan_pipeline::write_json(&proposal_path, &plan)?;
    let options = crate::plan_pipeline::PlanRunOptions {
        plan: proposal_path,
        policy: policy.into(),
        root: root.into(),
        timeout_secs: 60,
        entity: actor.into(),
        process: "artifact_transition".into(),
        intent_summary: Some(format!(
            "{:?} {:?} {:?} to {} under {} ({})",
            input.transition_kind,
            input.mode,
            input.source_artifact_ids,
            input.result_artifact_id,
            input.gate_profile_ref,
            input.gate_profile_hash,
        )),
        origin_evidence_refs: input.evidence_refs.clone(),
    };
    let outcome = if requires_external_gate {
        let pending_root = root.to_path_buf();
        let pending_actor = actor.to_owned();
        let pending_proposal = proposal.clone();
        crate::plan_pipeline::run_plan_with_approval_extension(
            options,
            crate::plan_pipeline::PlanApprovalExtension {
                plan_item_id: "transition".into(),
                action: preflight_action,
                before_awaiting_approval: Box::new(move |context| {
                    if context.committed_approval.payload_hash()
                        != sea_forge_ledger::types::payload_hash(&serde_json::to_value(
                            &context.approval,
                        )?)?
                    {
                        return Err(ForgeError::Internal(
                            "committed approval payload changed before pending transition".into(),
                        ));
                    }
                    let mut pending = PendingArtifactTransition {
                        version: sea_forge_artifact_ip::M8_RECORD_VERSION.into(),
                        pending_key: String::new(),
                        case_id: context.case_id,
                        run_id: context.run_id,
                        plan_item_id: context.plan_item_id,
                        criteria_ref: context.criteria.criteria_id,
                        criteria_sha256: context.criteria.criteria_sha256,
                        criteria_record_hash: context.criteria.criteria_record_hash,
                        authority_decision_ref: context.decision.decision_id,
                        authority_decision_hash: context.committed_decision.payload_hash().into(),
                        authority_action_hash: sea_forge_ledger::types::hash_canonical(
                            &context.decision.operation,
                        )?,
                        approval_ref: context.approval.approval_id,
                        proposal: pending_proposal,
                        proposal_snapshot_hash: String::new(),
                        profile_hash: String::new(),
                        created_at: chrono::Utc::now().to_rfc3339(),
                    };
                    seal_pending_transition(&mut pending)?;
                    commit_pending_transition(&pending_root, &pending_actor, &pending)?;
                    Ok(())
                }),
            },
        )?
    } else {
        crate::plan_pipeline::run_plan(options)?
    };
    if requires_external_gate {
        println!("case_id={}", outcome.case_id);
        println!("case_state={}", outcome.state);
        return Ok(outcome.exit_code);
    }
    if outcome.exit_code != 0 || outcome.state != "completed" {
        return Err(ForgeError::Input(
            "artifact_transition_error: governed transition case did not complete".into(),
        ));
    }
    let case_dir = root.join("cases").join(&outcome.case_id);
    let case: sea_forge_core::types::Case = serde_json::from_slice(
        &fs::read(case_dir.join("case.json"))
            .map_err(|error| ForgeError::io("read transition case", error))?,
    )?;
    let run_id = case
        .run_ids
        .first()
        .cloned()
        .ok_or_else(|| ForgeError::Internal("completed transition case has no run".into()))?;
    let run_dir = case_dir.join("runs").join(&run_id);
    let settlement: SettlementEvent = serde_json::from_slice(
        &fs::read(run_dir.join("settlement.json"))
            .map_err(|error| ForgeError::io("read transition settlement", error))?,
    )?;
    if settlement.status != SettlementStatus::Accepted {
        return Err(ForgeError::Input(
            "artifact_transition_error: governed transition settlement was not accepted".into(),
        ));
    }
    let case_plan: CasePlan = serde_json::from_slice(
        &fs::read(case_dir.join("plan.json"))
            .map_err(|error| ForgeError::io("read transition plan", error))?,
    )?;
    let item = case_plan
        .items
        .first()
        .ok_or_else(|| ForgeError::Internal("transition plan is empty".into()))?;
    input.case_id = outcome.case_id;
    input.run_id = run_id;
    input.criteria_ref = item
        .settlement_criteria_ref
        .clone()
        .ok_or_else(|| ForgeError::Internal("transition criteria reference missing".into()))?;
    input.settlement_ref = settlement.settlement_id;
    input.authority_decision_refs.clear();
    let final_action = canonical_transition_action(&input, &source_licenses, &profile);
    let final_run_id = input.run_id.clone();
    let final_case_id = input.case_id.clone();
    let token = super::mediated::with_authorized_action_in_context(
        root,
        policy,
        actor,
        &final_action,
        super::mediated::AuthorityContext {
            run_id: &final_run_id,
            case_id: &final_case_id,
            plan_item_id: "transition",
            sequence: 2,
        },
        |grant, decision_id| {
            let authorization = authorize_transition(
                grant,
                &final_action,
                root,
                actor,
                &input,
                &source_licenses,
                &profile,
                decision_id,
                &input.run_id,
                &input.case_id,
                "transition",
            )?;
            append_transition(authorization, input, &registrations, &tokens, &profile)
        },
    )?;
    println!("{}", serde_json::to_string_pretty(&token)?);
    Ok(0)
}

pub struct AttestOptions<'a> {
    pub root: &'a Path,
    pub policy: &'a Path,
    pub actor: &'a str,
    pub artifact_id: &'a str,
    pub ledger: &'a str,
    pub degraded_mode: &'a str,
    pub requester_role: &'a str,
    pub approver_role: &'a str,
    pub degraded_controls: Vec<String>,
}

pub fn attest_command(options: AttestOptions<'_>) -> Result<u8, ForgeError> {
    let (registrations, _) = load_ledger_records(options.root, options.actor)?;
    let registration = registrations
        .iter()
        .find(|registration| registration.artifact_id == options.artifact_id)
        .ok_or_else(|| ForgeError::Plan {
            class: "artifact_reference_error",
            message: format!("artifact {} is not registered", options.artifact_id),
        })?;
    let action = AuthorityAction::Reserved {
        resource_type: "attest_artifact_identity".into(),
        resource_id: registration.artifact_id.clone(),
        parameters: serde_json::json!({
            "artifact_id": registration.artifact_id,
            "content_identity": registration.content_identity,
            "descriptor_hash": registration.descriptor_hash,
            "ledger": options.ledger,
            "degraded_mode": options.degraded_mode,
            "degraded_controls": options.degraded_controls,
            "requester_role": options.requester_role,
            "approver_role": options.approver_role,
        }),
    };
    let result = super::mediated::with_authorized_action_binding(
        options.root,
        options.policy,
        options.actor,
        &action,
        |grant, decision, committed| {
            let authorization = authorize_attestation(
                grant,
                &action,
                options.root,
                registration,
                options.ledger,
                options.degraded_mode,
                options.degraded_controls,
                &decision,
                &committed,
            )?;
            attest(
                &DefaultArtifactAttestor,
                authorization,
                if options.degraded_mode == "pre_mint_only" {
                    AttestationPolicy::PreMintOnly
                } else {
                    AttestationPolicy::Required
                },
            )
        },
    )?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(0)
}

fn transition_plan(
    plan_id: &str,
    input: &TransitionInput,
    profile: &sea_forge_artifact_ip::ArtifactGateProfile,
) -> Result<CasePlan, ForgeError> {
    let [evaluator] = profile.evaluator_refs.as_slice() else {
        return Err(ForgeError::Plan {
            class: "artifact_gate_error",
            message: "transition profile must declare exactly one evaluator".into(),
        });
    };
    let (environment, _) = evaluator.rsplit_once('.').ok_or_else(|| ForgeError::Plan {
        class: "artifact_gate_error",
        message: format!("invalid profile evaluator reference {evaluator}"),
    })?;
    if !profile.evaluator_thresholds.contains_key(evaluator) {
        return Err(ForgeError::Plan {
            class: "artifact_gate_error",
            message: format!("profile evaluator {evaluator} has no threshold"),
        });
    }
    Ok(CasePlan {
        version: RECORD_VERSION.into(),
        plan_id: plan_id.into(),
        case_id: "case_transition_proposal".into(),
        run_id: "run_transition_proposal".into(),
        intent_id: "int_transition_proposal".into(),
        template_ref: None,
        job_contract_ref: None,
        items: vec![PlanItem {
            plan_item_id: "transition".into(),
            name: format!(
                "{:?} {:?} {} through {}",
                input.transition_kind, input.mode, input.result_artifact_id, profile.profile_ref
            ),
            operations: vec![],
            entry_criteria: vec![],
            exit_criteria: vec![],
            settlement_criteria: SettlementCriteria {
                require_approval: profile.requires_approval
                    || profile.required_settlement_strength.as_deref() == Some("strong"),
                evaluator: Some(evaluator.clone()),
                ..Default::default()
            },
            settlement_criteria_ref: None,
            item_kind: ItemKind::SandboxedTask,
            sandbox_class: Some("local".into()),
            parent_stage: None,
            markers: ItemMarkers {
                required: true,
                ..Default::default()
            },
            max_instances: 1,
            depends_on: vec![],
            environment: Some(environment.into()),
        }],
    })
}
