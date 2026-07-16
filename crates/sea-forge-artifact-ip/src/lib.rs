//! Artifact-to-IP recognition overlay (M8). Ledger records are the only truth;
//! catalog and capital files are deterministic views rebuilt from them.
#![forbid(unsafe_code)]

use sea_forge_authority::ActionGrant;
use sea_forge_capability::promotion::{declaration_qualifies, default_v02_policy};
use sea_forge_core::{errors::ForgeError, types::*};
use sea_forge_ledger::{
    types::{hash_canonical, payload_hash},
    CommittedRecordRef, LedgerManager, LedgerStream,
};
use sea_forge_settlement::{append_declaration_ledgered_once, SettlementAuthority};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

pub const M8_RECORD_VERSION: &str = "0.2";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IdentityStatus {
    PreMint,
    Attested,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransitionKind {
    Synthesize,
    Productize,
    Capitalize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransitionMode {
    Derive,
    Promote,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleStatus {
    Active,
    Superseded,
    Retired,
    Quarantined,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum SemanticReferenceClass {
    Concept,
    Class,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SemanticAnchor {
    pub class: SemanticReferenceClass,
    pub reference: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactLifecycleInput {
    pub artifact_id: String,
    pub previous_status: LifecycleStatus,
    pub new_status: LifecycleStatus,
    pub reason: String,
    pub changed_at: String,
    pub actor_id: String,
    pub case_id: String,
    pub run_id: String,
    pub plan_item_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactLifecycleRecord {
    pub version: String,
    pub artifact_id: String,
    pub previous_status: LifecycleStatus,
    pub new_status: LifecycleStatus,
    pub reason: String,
    pub changed_at: String,
    pub actor_id: String,
    pub case_id: String,
    pub run_id: String,
    pub plan_item_id: String,
    pub authority_decision_refs: Vec<String>,
    pub authority_decision_hash: String,
    pub authority_action_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactRegistrationRecord {
    pub version: String,
    pub artifact_id: String,
    pub lineage_id: String,
    pub content_identity: String,
    pub descriptor_hash: String,
    pub identity_scheme: String,
    pub artifact_type: ArtifactType,
    pub name: String,
    pub artifact_version: String,
    pub content_sha256: String,
    pub declared_stage: Option<ArtifactStage>,
    pub legacy_pre_mint_identity: String,
    pub owner: String,
    pub license: String,
    pub review_status: ReviewStatus,
    pub source_evidence_refs: Vec<String>,
    pub source_run_ids: Vec<String>,
    pub case_id: String,
    pub derived_from: Vec<String>,
    pub created_at: String,
}

impl ArtifactRegistrationRecord {
    pub fn recognized_stage(&self) -> ArtifactStage {
        ArtifactStage::Cognitive
    }
}

#[derive(Clone, Debug)]
pub struct ArtifactRegistrationInput {
    pub descriptor: ArtifactDescriptor,
    pub name: String,
    pub version: String,
    pub source_evidence_refs: Vec<String>,
    pub source_run_ids: Vec<String>,
    pub case_id: String,
    pub derived_from: Vec<String>,
    pub created_at: String,
}

#[derive(Clone, Debug)]
pub struct VerifiedArtifactRegistrationInput {
    pub name: String,
    pub version: String,
    pub case_id: String,
    pub derived_from: Vec<String>,
    pub created_at: String,
}

/// Immutable reference to value evidence. Capital value must be demonstrated
/// outside the case that produced the artifact, never by a mutable counter.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValueEvidenceRef {
    pub reference: String,
    pub case_id: String,
    pub kind: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValueEvidenceRecord {
    pub version: String,
    pub value_evidence_id: String,
    pub kind: String,
    pub case_id: String,
    pub evidence_refs: Vec<String>,
    pub accepted: bool,
    pub created_at: String,
    pub record_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValueEvidenceSourceRecord {
    pub version: String,
    pub evidence_id: String,
    pub kind: String,
    pub case_id: String,
    pub originating_case_id: String,
    pub evidence_refs: Vec<String>,
    pub source_run_id: String,
    pub settlement_ref: String,
    pub created_at: String,
    pub record_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RightsProfileRecord {
    pub version: String,
    pub rights_profile_id: String,
    pub artifact_id: String,
    pub license: String,
    pub review_status: ReviewStatus,
    pub accepted: bool,
    pub authority_decision_ref: String,
    pub authority_decision_hash: String,
    pub authority_action_hash: String,
    pub created_at: String,
    pub record_hash: String,
}

pub fn canonical_rights_action(
    registration: &ArtifactRegistrationRecord,
    required_review: &[String],
) -> AuthorityAction {
    AuthorityAction::Reserved {
        resource_type: "review_artifact_rights".into(),
        resource_id: registration.artifact_id.clone(),
        parameters: json!({
            "artifact_id": registration.artifact_id,
            "license": registration.license,
            "required_review": required_review,
        }),
    }
}

pub struct AuthorizedRightsProfile {
    root: std::path::PathBuf,
    actor: String,
    decision_ref: String,
    decision_hash: String,
    action_hash: String,
    artifact_id: String,
    license: String,
    required_review: Vec<String>,
    action: AuthorityAction,
}

#[allow(clippy::too_many_arguments)]
pub fn authorize_rights_profile(
    grant: ActionGrant,
    action: &AuthorityAction,
    root: &Path,
    actor: &str,
    registration: &ArtifactRegistrationRecord,
    required_review: &[String],
    decision: &AuthorityDecision,
    committed: &CommittedRecordRef,
) -> Result<AuthorizedRightsProfile, ForgeError> {
    let expected = canonical_rights_action(registration, required_review);
    let decision_hash = payload_hash(&serde_json::to_value(decision)?)?;
    if action != &expected
        || decision.operation != expected
        || decision.verdict != Verdict::Allow
        || committed.record_kind() != "authority_decision"
        || committed.payload_hash() != decision_hash
    {
        return Err(m8_error(
            "artifact_reference_error",
            "rights review authority does not match the exact action and decision",
        ));
    }
    grant.authorize(action, "rights_review", "rights_review", root)?;
    Ok(AuthorizedRightsProfile {
        root: root.into(),
        actor: actor.into(),
        decision_ref: decision.decision_id.clone(),
        decision_hash,
        action_hash: hash_canonical(&expected)?,
        artifact_id: registration.artifact_id.clone(),
        license: registration.license.clone(),
        required_review: required_review.to_vec(),
        action: expected,
    })
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GateResult {
    pub gate: String,
    pub status: String,
    pub basis: String,
}

fn default_plan_item_id() -> String {
    "transition".to_string()
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransitionToken {
    pub version: String,
    pub transition_token_id: String,
    pub transition_kind: TransitionKind,
    pub mode: TransitionMode,
    pub from_stage: ArtifactStage,
    pub to_stage: ArtifactStage,
    pub source_artifact_ids: Vec<String>,
    pub result_artifact_id: String,
    pub derived_from: Vec<String>,
    pub parent_transition_token_ids: Vec<String>,
    pub input_content_identities: Vec<String>,
    pub output_content_identity: String,
    pub gate_profile_ref: String,
    pub gate_profile_hash: String,
    pub stage_gate_results: Vec<GateResult>,
    pub actor_id: String,
    pub approver_id: Option<String>,
    #[serde(default)]
    pub approval_ref: Option<String>,
    pub authority_decision_refs: Vec<String>,
    pub criteria_ref: String,
    pub evidence_refs: Vec<String>,
    pub settlement_ref: String,
    pub strong_declaration_refs: Vec<String>,
    pub value_evidence_refs: Vec<String>,
    #[serde(default)]
    pub value_evidence: Vec<ValueEvidenceRef>,
    pub rights_profile_ref: Option<String>,
    pub semantic_refs: Vec<SemanticAnchor>,
    #[serde(default)]
    pub semantic_model_ref: Option<String>,
    pub identity_status_before: IdentityStatus,
    pub identity_status_after: IdentityStatus,
    pub attestation_ref: Option<String>,
    pub degraded_controls: Vec<String>,
    pub case_id: String,
    pub run_id: String,
    #[serde(default = "default_plan_item_id")]
    pub plan_item_id: String,
    pub created_at: String,
    pub transition_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransitionInput {
    pub transition_token_id: String,
    pub transition_kind: TransitionKind,
    pub mode: TransitionMode,
    pub from_stage: ArtifactStage,
    pub to_stage: ArtifactStage,
    pub source_artifact_ids: Vec<String>,
    pub result_artifact_id: String,
    pub derived_from: Vec<String>,
    pub input_content_identities: Vec<String>,
    pub output_content_identity: String,
    pub parent_transition_token_ids: Vec<String>,
    pub gate_profile_ref: String,
    pub gate_profile_hash: String,
    pub actor_id: String,
    pub approver_id: Option<String>,
    #[serde(default)]
    pub approval_ref: Option<String>,
    pub authority_decision_refs: Vec<String>,
    pub criteria_ref: String,
    pub evidence_refs: Vec<String>,
    pub settlement_ref: String,
    pub strong_declaration_refs: Vec<String>,
    pub value_evidence_refs: Vec<String>,
    #[serde(default)]
    pub value_evidence: Vec<ValueEvidenceRef>,
    pub rights_profile_ref: Option<String>,
    pub semantic_refs: Vec<SemanticAnchor>,
    #[serde(default)]
    pub semantic_model_ref: Option<String>,
    pub identity_status_before: IdentityStatus,
    pub identity_status_after: IdentityStatus,
    pub attestation_ref: Option<String>,
    pub degraded_controls: Vec<String>,
    pub case_id: String,
    pub run_id: String,
    #[serde(default = "default_plan_item_id")]
    pub plan_item_id: String,
    pub created_at: String,
}

/// Caller-controlled transition intent. Governance and lifecycle references are
/// allocated and bound by the kernel after this strict schema is accepted.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TransitionProposal {
    pub transition_token_id: String,
    pub transition_kind: TransitionKind,
    pub mode: TransitionMode,
    pub from_stage: ArtifactStage,
    pub to_stage: ArtifactStage,
    pub source_artifact_ids: Vec<String>,
    pub result_artifact_id: String,
    pub derived_from: Vec<String>,
    pub input_content_identities: Vec<String>,
    pub output_content_identity: String,
    pub parent_transition_token_ids: Vec<String>,
    pub gate_profile_ref: String,
    pub gate_profile_hash: String,
    #[serde(default)]
    pub value_evidence_refs: Vec<String>,
    #[serde(default)]
    pub value_evidence: Vec<ValueEvidenceRef>,
    #[serde(default)]
    pub rights_profile_ref: Option<String>,
    #[serde(default)]
    pub semantic_refs: Vec<SemanticAnchor>,
    #[serde(default)]
    pub semantic_model_ref: Option<String>,
    pub identity_status_before: IdentityStatus,
    pub identity_status_after: IdentityStatus,
    #[serde(default)]
    pub attestation_ref: Option<String>,
    #[serde(default)]
    pub degraded_controls: Vec<String>,
}

impl From<&TransitionInput> for TransitionProposal {
    fn from(input: &TransitionInput) -> Self {
        Self {
            transition_token_id: input.transition_token_id.clone(),
            transition_kind: input.transition_kind.clone(),
            mode: input.mode.clone(),
            from_stage: input.from_stage.clone(),
            to_stage: input.to_stage.clone(),
            source_artifact_ids: input.source_artifact_ids.clone(),
            result_artifact_id: input.result_artifact_id.clone(),
            derived_from: input.derived_from.clone(),
            input_content_identities: input.input_content_identities.clone(),
            output_content_identity: input.output_content_identity.clone(),
            parent_transition_token_ids: input.parent_transition_token_ids.clone(),
            gate_profile_ref: input.gate_profile_ref.clone(),
            gate_profile_hash: input.gate_profile_hash.clone(),
            value_evidence_refs: input.value_evidence_refs.clone(),
            value_evidence: input.value_evidence.clone(),
            rights_profile_ref: input.rights_profile_ref.clone(),
            semantic_refs: input.semantic_refs.clone(),
            semantic_model_ref: input.semantic_model_ref.clone(),
            identity_status_before: input.identity_status_before.clone(),
            identity_status_after: input.identity_status_after.clone(),
            attestation_ref: input.attestation_ref.clone(),
            degraded_controls: input.degraded_controls.clone(),
        }
    }
}

impl TransitionProposal {
    pub fn into_transition_input(self, actor_id: String, created_at: String) -> TransitionInput {
        TransitionInput {
            transition_token_id: self.transition_token_id,
            transition_kind: self.transition_kind,
            mode: self.mode,
            from_stage: self.from_stage,
            to_stage: self.to_stage,
            source_artifact_ids: self.source_artifact_ids,
            result_artifact_id: self.result_artifact_id,
            derived_from: self.derived_from,
            input_content_identities: self.input_content_identities,
            output_content_identity: self.output_content_identity,
            parent_transition_token_ids: self.parent_transition_token_ids,
            gate_profile_ref: self.gate_profile_ref,
            gate_profile_hash: self.gate_profile_hash,
            actor_id,
            approver_id: None,
            approval_ref: None,
            authority_decision_refs: vec![],
            criteria_ref: String::new(),
            evidence_refs: vec![],
            settlement_ref: String::new(),
            strong_declaration_refs: vec![],
            value_evidence_refs: self.value_evidence_refs,
            value_evidence: self.value_evidence,
            rights_profile_ref: self.rights_profile_ref,
            semantic_refs: self.semantic_refs,
            semantic_model_ref: self.semantic_model_ref,
            identity_status_before: self.identity_status_before,
            identity_status_after: self.identity_status_after,
            attestation_ref: self.attestation_ref,
            degraded_controls: self.degraded_controls,
            case_id: String::new(),
            run_id: String::new(),
            plan_item_id: default_plan_item_id(),
            created_at,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PendingArtifactTransition {
    pub version: String,
    pub pending_key: String,
    pub case_id: String,
    pub run_id: String,
    pub plan_item_id: String,
    pub criteria_ref: String,
    pub criteria_sha256: String,
    pub criteria_record_hash: String,
    pub authority_decision_ref: String,
    pub authority_decision_hash: String,
    pub authority_action_hash: String,
    pub approval_ref: String,
    pub proposal: TransitionProposal,
    pub proposal_snapshot_hash: String,
    pub profile_hash: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactTransitionTerminalStatus {
    Completed,
    Rejected,
    Expired,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactTransitionTerminalReason {
    ApprovalRejected,
    ApprovalExpired,
    SettlementRejected,
    TransitionCommitted,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactTransitionTerminal {
    pub version: String,
    pub pending_key: String,
    pub case_id: String,
    pub run_id: String,
    pub status: ArtifactTransitionTerminalStatus,
    pub reason: ArtifactTransitionTerminalReason,
    pub transition_token_ref: Option<String>,
    pub declaration_ref: Option<String>,
    pub terminal_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactTransitionClaimManifest {
    pub version: String,
    pub pending_key: String,
    pub case_id: String,
    pub run_id: String,
    pub plan_item_id: String,
    pub criteria_ref: String,
    pub criteria_sha256: String,
    pub criteria_record_hash: String,
    pub authority_decision_ref: String,
    pub authority_decision_hash: String,
    pub authority_action_hash: String,
    pub approval_ref: String,
    pub proposal_snapshot_hash: String,
    pub profile_hash: String,
    pub evidence_refs: Vec<String>,
    pub settlement_ref: String,
    pub created_at: String,
    pub claim_manifest_hash: String,
}

pub fn seal_pending_transition(record: &mut PendingArtifactTransition) -> Result<(), ForgeError> {
    record.proposal_snapshot_hash = hash_canonical(&record.proposal)?;
    record
        .profile_hash
        .clone_from(&record.proposal.gate_profile_hash);
    record.pending_key = hash_canonical(&json!({
        "case_id": record.case_id,
        "run_id": record.run_id,
        "criteria_sha256": record.criteria_sha256,
        "criteria_record_hash": record.criteria_record_hash,
        "authority_decision_ref": record.authority_decision_ref,
        "authority_decision_hash": record.authority_decision_hash,
        "authority_action_hash": record.authority_action_hash,
        "proposal_snapshot_hash": record.proposal_snapshot_hash,
        "profile_hash": record.profile_hash,
    }))?;
    Ok(())
}

fn validate_pending_transition(record: &PendingArtifactTransition) -> Result<(), ForgeError> {
    let mut expected = record.clone();
    seal_pending_transition(&mut expected)?;
    if record.version != M8_RECORD_VERSION
        || record.pending_key != expected.pending_key
        || record.proposal_snapshot_hash != expected.proposal_snapshot_hash
        || record.profile_hash != expected.profile_hash
        || record.case_id.is_empty()
        || record.run_id.is_empty()
        || record.plan_item_id.is_empty()
        || record.criteria_ref.is_empty()
        || record.authority_decision_ref.is_empty()
        || record.approval_ref.is_empty()
    {
        return Err(m8_error(
            "artifact_pending_transition_error",
            "pending transition binding is invalid",
        ));
    }
    Ok(())
}

pub fn commit_pending_transition(
    root: &Path,
    actor: &str,
    record: &PendingArtifactTransition,
) -> Result<CommittedRecordRef, ForgeError> {
    validate_pending_transition(record)?;
    let stream = LedgerStream::open(root, format!("case-{}", record.case_id), actor)?;
    stream.verify()?;
    let entries = stream.read_entries()?;
    if entries
        .iter()
        .any(|entry| entry.record_kind == "artifact_transition_terminal")
    {
        return Err(m8_error(
            "artifact_pending_transition_error",
            "terminal transition cannot be resumed",
        ));
    }
    let existing = entries
        .iter()
        .filter(|entry| entry.record_kind == "pending_artifact_transition")
        .collect::<Vec<_>>();
    if existing.len() > 1
        || existing.first().is_some_and(|entry| {
            entry.payload.get("pending_key").and_then(Value::as_str)
                != Some(record.pending_key.as_str())
        })
    {
        return Err(m8_error(
            "artifact_pending_transition_error",
            "case has multiple or conflicting pending transitions",
        ));
    }
    stream.commit_typed_once(
        "pending_artifact_transition",
        &record.pending_key,
        vec![record.case_id.clone(), record.run_id.clone()],
        record,
        vec![record.authority_decision_ref.clone()],
    )
}

pub fn load_pending_transition(
    root: &Path,
    actor: &str,
    case_id: &str,
) -> Result<PendingArtifactTransition, ForgeError> {
    let stream = LedgerStream::open(root, format!("case-{case_id}"), actor)?;
    stream.verify()?;
    let entries = stream.read_entries()?;
    if entries
        .iter()
        .any(|entry| entry.record_kind == "artifact_transition_terminal")
    {
        return Err(m8_error(
            "artifact_pending_transition_error",
            "pending transition is terminal",
        ));
    }
    let mut pending = entries
        .iter()
        .filter(|entry| entry.record_kind == "pending_artifact_transition");
    let record: PendingArtifactTransition = pending
        .next()
        .ok_or_else(|| {
            m8_error(
                "artifact_pending_transition_error",
                "pending transition is missing",
            )
        })
        .and_then(|entry| serde_json::from_value(entry.payload.clone()).map_err(Into::into))?;
    if pending.next().is_some() || record.case_id != case_id {
        return Err(m8_error(
            "artifact_pending_transition_error",
            "case has multiple or conflicting pending transitions",
        ));
    }
    validate_pending_transition(&record)?;
    Ok(record)
}

pub fn load_transition_terminal(
    root: &Path,
    actor: &str,
    case_id: &str,
) -> Result<Option<ArtifactTransitionTerminal>, ForgeError> {
    let stream = LedgerStream::open(root, format!("case-{case_id}"), actor)?;
    stream.verify()?;
    let mut terminals = stream
        .read_entries()?
        .into_iter()
        .filter(|entry| entry.record_kind == "artifact_transition_terminal");
    let terminal = terminals
        .next()
        .map(|entry| serde_json::from_value(entry.payload))
        .transpose()?;
    if terminals.next().is_some() {
        return Err(m8_error(
            "artifact_pending_transition_error",
            "case has multiple terminal transition records",
        ));
    }
    if terminal
        .as_ref()
        .is_some_and(|terminal: &ArtifactTransitionTerminal| terminal.case_id != case_id)
    {
        return Err(m8_error(
            "artifact_pending_transition_error",
            "terminal transition belongs to another case",
        ));
    }
    Ok(terminal)
}

pub fn commit_transition_terminal(
    root: &Path,
    actor: &str,
    terminal: &ArtifactTransitionTerminal,
) -> Result<CommittedRecordRef, ForgeError> {
    // ponytail: a retry of an already-terminal case must reach the idempotent
    // path, not fail inside load_pending_transition. Inspect the existing
    // terminal record first: identical -> return its committed ref; differing
    // -> reject as a conflict. New/conflicting transitions fall through.
    if let Some(existing) = load_transition_terminal(root, actor, &terminal.case_id)? {
        if existing != *terminal {
            return Err(m8_error(
                "artifact_pending_transition_error",
                "conflicting terminal transition record",
            ));
        }
        let stream = LedgerStream::open(root, format!("case-{}", terminal.case_id), actor)?;
        let prior = stream
            .read_entries()?
            .into_iter()
            .find(|entry| {
                entry.record_kind == "artifact_transition_terminal"
                    && entry.idempotency_key.as_deref() == Some(terminal.pending_key.as_str())
            })
            .ok_or_else(|| {
                m8_error(
                    "artifact_pending_transition_error",
                    "terminal transition ledger entry is missing",
                )
            })?;
        return Ok(prior.committed_ref());
    }
    let pending = load_pending_transition(root, actor, &terminal.case_id)?;
    if terminal.version != M8_RECORD_VERSION
        || terminal.pending_key != pending.pending_key
        || terminal.run_id != pending.run_id
    {
        return Err(m8_error(
            "artifact_pending_transition_error",
            "terminal record does not match pending transition",
        ));
    }
    LedgerStream::open(root, format!("case-{}", terminal.case_id), actor)?.commit_typed_once(
        "artifact_transition_terminal",
        &terminal.pending_key,
        vec![terminal.case_id.clone(), terminal.run_id.clone()],
        terminal,
        vec![pending.authority_decision_ref],
    )
}

pub fn seal_claim_manifest(
    manifest: &mut ArtifactTransitionClaimManifest,
) -> Result<(), ForgeError> {
    manifest.claim_manifest_hash.clear();
    manifest.claim_manifest_hash = hash_canonical(manifest)?;
    Ok(())
}

pub fn commit_claim_manifest(
    root: &Path,
    actor: &str,
    manifest: &ArtifactTransitionClaimManifest,
) -> Result<CommittedRecordRef, ForgeError> {
    let pending = load_pending_transition(root, actor, &manifest.case_id)?;
    let mut expected = manifest.clone();
    seal_claim_manifest(&mut expected)?;
    if manifest.version != M8_RECORD_VERSION
        || manifest.claim_manifest_hash != expected.claim_manifest_hash
        || manifest.pending_key != pending.pending_key
        || manifest.run_id != pending.run_id
        || manifest.plan_item_id != pending.plan_item_id
        || manifest.criteria_ref != pending.criteria_ref
        || manifest.criteria_sha256 != pending.criteria_sha256
        || manifest.criteria_record_hash != pending.criteria_record_hash
        || manifest.authority_decision_ref != pending.authority_decision_ref
        || manifest.authority_decision_hash != pending.authority_decision_hash
        || manifest.authority_action_hash != pending.authority_action_hash
        || manifest.approval_ref != pending.approval_ref
        || manifest.proposal_snapshot_hash != pending.proposal_snapshot_hash
        || manifest.profile_hash != pending.profile_hash
    {
        return Err(m8_error(
            "artifact_claim_manifest_error",
            "claim manifest does not match pending transition",
        ));
    }
    LedgerStream::open(root, format!("case-{}", manifest.case_id), actor)?.commit_typed_once(
        "artifact_transition_claim_manifest",
        &manifest.pending_key,
        vec![manifest.case_id.clone(), manifest.run_id.clone()],
        manifest,
        vec![manifest.authority_decision_ref.clone()],
    )
}

pub struct ResumedArtifactTransition {
    pub manifest: ArtifactTransitionClaimManifest,
    pub declaration: SettlementDeclaration,
    pub token: TransitionToken,
}

pub fn load_completed_transition(
    root: &Path,
    actor: &str,
    case_id: &str,
) -> Result<Option<ResumedArtifactTransition>, ForgeError> {
    let Some(terminal) = load_transition_terminal(root, actor, case_id)? else {
        return Ok(None);
    };
    if terminal.status != ArtifactTransitionTerminalStatus::Completed
        || terminal.reason != ArtifactTransitionTerminalReason::TransitionCommitted
    {
        return Ok(None);
    }
    let token_ref = terminal.transition_token_ref.as_deref().ok_or_else(|| {
        m8_error(
            "artifact_pending_transition_error",
            "completed transition terminal has no token reference",
        )
    })?;
    let declaration_ref = terminal.declaration_ref.as_deref().ok_or_else(|| {
        m8_error(
            "artifact_pending_transition_error",
            "completed transition terminal has no declaration reference",
        )
    })?;
    let case_entries =
        LedgerStream::open(root, format!("case-{case_id}"), actor)?.read_entries()?;
    let manifest: ArtifactTransitionClaimManifest = case_entries
        .iter()
        .find(|entry| {
            entry.record_kind == "artifact_transition_claim_manifest"
                && entry.payload.get("pending_key").and_then(Value::as_str)
                    == Some(terminal.pending_key.as_str())
        })
        .ok_or_else(|| m8_error("artifact_claim_manifest_error", "claim manifest is missing"))
        .and_then(|entry| serde_json::from_value(entry.payload.clone()).map_err(Into::into))?;
    let mut expected_manifest = manifest.clone();
    seal_claim_manifest(&mut expected_manifest)?;
    if manifest.version != M8_RECORD_VERSION
        || manifest.claim_manifest_hash != expected_manifest.claim_manifest_hash
        || manifest.pending_key != terminal.pending_key
        || manifest.case_id != terminal.case_id
        || manifest.run_id != terminal.run_id
    {
        return Err(m8_error(
            "artifact_claim_manifest_error",
            "completed claim manifest is invalid or detached from its terminal",
        ));
    }
    let declaration: SettlementDeclaration = case_entries
        .iter()
        .find(|entry| {
            entry.record_kind == "settlement_declaration"
                && entry.payload.get("declaration_id").and_then(Value::as_str)
                    == Some(declaration_ref)
        })
        .ok_or_else(|| {
            m8_error(
                "settlement_integrity_error",
                "completed declaration is missing",
            )
        })
        .and_then(|entry| serde_json::from_value(entry.payload.clone()).map_err(Into::into))?;
    if declaration.case_id != terminal.case_id
        || declaration.run_id != terminal.run_id
        || declaration.claim_manifest_sha256 != manifest.claim_manifest_hash
        || sea_forge_settlement::compute_declaration_hash(&declaration)?
            != declaration.declaration_hash
    {
        return Err(m8_error(
            "settlement_integrity_error",
            "completed declaration conflicts with its terminal or manifest",
        ));
    }
    let (_, tokens) = load_ledger_records(root, actor)?;
    let token = tokens
        .into_iter()
        .find(|token| {
            token.transition_token_id == token_ref
                && token.case_id == terminal.case_id
                && token.run_id == terminal.run_id
                && token.strong_declaration_refs == [declaration_ref]
        })
        .ok_or_else(|| {
            m8_error(
                "artifact_pending_transition_error",
                "completed transition token is missing or conflicting",
            )
        })?;
    Ok(Some(ResumedArtifactTransition {
        manifest,
        declaration,
        token,
    }))
}

#[allow(clippy::too_many_arguments)]
pub fn resume_pending_transition(
    root: &Path,
    actor: &str,
    pending: &PendingArtifactTransition,
    approval: &ApprovalRequest,
    criteria: &SettlementCriteriaRecord,
    settlement: &SettlementEvent,
    execution_started_at: &str,
    verifier_ref: &str,
    verifier_sha256: &str,
    workspace_root: &Path,
    declarer: Declarer,
    authority: &dyn SettlementAuthority,
    grant: ActionGrant,
) -> Result<ResumedArtifactTransition, ForgeError> {
    if approval.status != ApprovalStatus::Approved
        || approval.approval_id != pending.approval_ref
        || approval.resolved_by.as_deref().is_none_or(str::is_empty)
        || settlement.status != SettlementStatus::Accepted
        || settlement.run_id != pending.run_id
        || settlement.criteria_ref.as_deref() != Some(pending.criteria_ref.as_str())
        || criteria.criteria_id != pending.criteria_ref
    {
        return Err(m8_error(
            "artifact_pending_transition_error",
            "approved transition records do not match the pending transition",
        ));
    }
    let (registrations, tokens) = load_ledger_records(root, actor)?;
    let profile = read_gate_profile(
        root,
        &pending.proposal.gate_profile_ref,
        &pending.profile_hash,
    )?;
    let mut evidence_refs = pending
        .proposal
        .source_artifact_ids
        .iter()
        .map(|source| {
            registrations
                .iter()
                .find(|registration| registration.artifact_id == *source)
                .ok_or_else(|| m8_error("artifact_reference_error", "transition source is missing"))
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flat_map(|registration| registration.source_evidence_refs.clone())
        .collect::<Vec<_>>();
    evidence_refs.sort();
    evidence_refs.dedup();
    let mut manifest = ArtifactTransitionClaimManifest {
        version: M8_RECORD_VERSION.into(),
        pending_key: pending.pending_key.clone(),
        case_id: pending.case_id.clone(),
        run_id: pending.run_id.clone(),
        plan_item_id: pending.plan_item_id.clone(),
        criteria_ref: pending.criteria_ref.clone(),
        criteria_sha256: pending.criteria_sha256.clone(),
        criteria_record_hash: pending.criteria_record_hash.clone(),
        authority_decision_ref: pending.authority_decision_ref.clone(),
        authority_decision_hash: pending.authority_decision_hash.clone(),
        authority_action_hash: pending.authority_action_hash.clone(),
        approval_ref: pending.approval_ref.clone(),
        proposal_snapshot_hash: pending.proposal_snapshot_hash.clone(),
        profile_hash: pending.profile_hash.clone(),
        evidence_refs: evidence_refs.clone(),
        settlement_ref: settlement.settlement_id.clone(),
        created_at: settlement.settled_at.clone(),
        claim_manifest_hash: String::new(),
    };
    seal_claim_manifest(&mut manifest)?;
    commit_claim_manifest(root, actor, &manifest)?;

    let stream = LedgerStream::open(root, format!("case-{}", pending.case_id), actor)?;
    let declarations = stream
        .read_entries()?
        .into_iter()
        .filter(|entry| entry.record_kind == "settlement_declaration")
        .map(|entry| serde_json::from_value::<SettlementDeclaration>(entry.payload))
        .collect::<Result<Vec<_>, _>>()?;
    let matching = declarations
        .into_iter()
        .filter(|declaration| {
            declaration.case_id == pending.case_id
                && declaration.run_id == pending.run_id
                && declaration.claim_manifest_sha256 == manifest.claim_manifest_hash
                && declaration.declarer.authority_ref == authority.adapter_ref()
        })
        .collect::<Vec<_>>();
    if matching.len() > 1 {
        return Err(m8_error(
            "settlement_integrity_error",
            "multiple declarations match one pending transition",
        ));
    }
    let declaration = if let Some(existing) = matching.into_iter().next() {
        existing
    } else {
        let request = SettlementDeclarationRequest {
            settlement_ref: settlement.settlement_id.clone(),
            run_id: pending.run_id.clone(),
            case_id: pending.case_id.clone(),
            plan_item_id: pending.plan_item_id.clone(),
            claim_manifest_sha256: manifest.claim_manifest_hash.clone(),
            criteria_ref: criteria.criteria_id.clone(),
            criteria_sha256: criteria.criteria_sha256.clone(),
            criteria_record_hash: criteria.criteria_record_hash.clone(),
            criteria_declared_at: criteria.declared_at.clone(),
            execution_started_at: execution_started_at.into(),
            job_contract_ref: None,
            origin_refs: criteria.origin_refs.clone(),
            verifier_ref: verifier_ref.into(),
            verifier_sha256: verifier_sha256.into(),
            acting_entity_id: actor.into(),
            requested_strength: SettlementStrength::Strong,
            declarer,
            variation_tags: BTreeMap::new(),
            disruption_tags: vec![],
            orchestration_burden: None,
            source_evidence_refs: evidence_refs.clone(),
        };
        let declaration = authority.declare(&request)?;
        append_declaration_ledgered_once(
            root,
            &root.join("declarations.jsonl"),
            actor,
            &declaration,
        )?;
        declaration
    };
    if !declaration_qualifies(&declaration, &default_v02_policy())
        || declaration.claim_manifest_sha256 != manifest.claim_manifest_hash
        || declaration.declarer.authority_ref != authority.adapter_ref()
    {
        return Err(m8_error(
            "settlement_integrity_error",
            "strong authority declaration does not qualify",
        ));
    }

    if let Some(existing) = tokens
        .iter()
        .find(|token| token.transition_token_id == pending.proposal.transition_token_id)
    {
        if existing.case_id == pending.case_id
            && existing.run_id == pending.run_id
            && existing.criteria_ref == pending.criteria_ref
            && existing.settlement_ref == settlement.settlement_id
            && existing.strong_declaration_refs == [declaration.declaration_id.as_str()]
        {
            // Reconstruct the token's proposal and require its canonical hash to
            // match the pending snapshot, so a replayed token cannot silently
            // substitute a different transition shape.
            let reconstructed: TransitionInput =
                serde_json::from_value(serde_json::to_value(existing)?).map_err(|error| {
                    m8_error(
                        "artifact_pending_transition_error",
                        format!("existing token cannot reconstruct proposal: {error}"),
                    )
                })?;
            let proposal = TransitionProposal::from(&reconstructed);
            if hash_canonical(&proposal)? != pending.proposal_snapshot_hash {
                return Err(m8_error(
                    "artifact_pending_transition_error",
                    "existing transition token proposal snapshot does not match pending transition",
                ));
            }
            return Ok(ResumedArtifactTransition {
                manifest,
                declaration,
                token: existing.clone(),
            });
        }
        return Err(m8_error(
            "artifact_pending_transition_error",
            "transition token replay conflicts with the pending transition",
        ));
    }

    let source_licenses = pending
        .proposal
        .source_artifact_ids
        .iter()
        .map(|source| {
            registrations
                .iter()
                .find(|registration| registration.artifact_id == *source)
                .map(|registration| registration.license.clone())
                .ok_or_else(|| m8_error("artifact_reference_error", "source license is missing"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut input = pending
        .proposal
        .clone()
        .into_transition_input(actor.into(), settlement.settled_at.clone());
    input.approver_id.clone_from(&approval.resolved_by);
    input.approval_ref = Some(approval.approval_id.clone());
    input.criteria_ref = criteria.criteria_id.clone();
    input.evidence_refs = evidence_refs;
    input.settlement_ref = settlement.settlement_id.clone();
    input.strong_declaration_refs = vec![declaration.declaration_id.clone()];
    input.case_id = pending.case_id.clone();
    input.run_id = pending.run_id.clone();
    let action = canonical_transition_action(&input, &source_licenses, &profile);
    let authorization = authorize_transition_in_workspace(
        grant,
        &action,
        root,
        workspace_root,
        actor,
        &input,
        &source_licenses,
        &profile,
        pending.authority_decision_ref.clone(),
        &pending.run_id,
        &pending.case_id,
        &pending.plan_item_id,
    )?;
    let token = append_transition_once(
        authorization,
        input,
        &registrations,
        &tokens,
        &profile,
        &pending.pending_key,
    )?;
    Ok(ResumedArtifactTransition {
        manifest,
        declaration,
        token,
    })
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ArtifactGateProfile {
    pub version: String,
    pub profile_ref: String,
    pub artifact_types: Vec<ArtifactType>,
    pub transition_kind: TransitionKind,
    #[serde(default)]
    pub required_metadata: Vec<String>,
    #[serde(default)]
    pub evaluator_refs: Vec<String>,
    #[serde(default)]
    pub evaluator_thresholds: BTreeMap<String, f64>,
    #[serde(default)]
    pub semantic_reference_classes: Vec<SemanticReferenceClass>,
    #[serde(default)]
    pub required_rights_review: Vec<String>,
    #[serde(default)]
    pub required_settlement_strength: Option<String>,
    #[serde(default)]
    pub qualifying_value_evidence_kinds: Vec<String>,
    #[serde(default)]
    pub requires_approval: bool,
    #[serde(default)]
    pub attestation_mode: Option<String>,
}

pub fn content_identity(
    artifact_type: &ArtifactType,
    content_sha256: &str,
) -> Result<String, ForgeError> {
    hash_canonical(&json!({"artifact_type": artifact_type, "content_sha256": content_sha256}))
}

pub fn register(
    mut input: ArtifactRegistrationInput,
) -> Result<ArtifactRegistrationRecord, ForgeError> {
    // ponytail: canonicalize derived_from (sort+dedup) at the choke point so
    // equivalent source sets match in rebuild regardless of caller order (lib.rs:3594).
    input.derived_from.sort();
    input.derived_from.dedup();
    // ponytail: reject path separators/traversal so the capital projection
    // ({artifact_id}.json at lib.rs:3714) cannot escape ip/capital.
    if input.descriptor.artifact_id.is_empty()
        || input.descriptor.artifact_id.contains('/')
        || input.descriptor.artifact_id.contains('\\')
        || input.descriptor.artifact_id.contains('\0')
        || input.descriptor.artifact_id == "."
        || input.descriptor.artifact_id == ".."
    {
        return Err(m8_error(
            "artifact_registration_error",
            "artifact_id must be a safe single-segment filename",
        ));
    }
    if input.source_evidence_refs.is_empty() || input.source_run_ids.is_empty() {
        return Err(m8_error(
            "artifact_registration_error",
            "verified work-product evidence is required",
        ));
    }
    let descriptor_hash = hash_canonical(&input.descriptor)?;
    let content_identity = content_identity(
        &input.descriptor.artifact_type,
        &input.descriptor.content_sha256,
    )?;
    Ok(ArtifactRegistrationRecord {
        version: M8_RECORD_VERSION.into(),
        lineage_id: content_identity.clone(),
        content_identity,
        descriptor_hash,
        identity_scheme: "artifact-identity-v2".into(),
        artifact_id: input.descriptor.artifact_id,
        artifact_type: input.descriptor.artifact_type,
        name: input.name,
        artifact_version: input.version,
        content_sha256: input.descriptor.content_sha256,
        declared_stage: input.descriptor.stage,
        legacy_pre_mint_identity: input.descriptor.pre_mint_identity,
        owner: input.descriptor.owner,
        license: input.descriptor.license,
        review_status: input.descriptor.review_status,
        source_evidence_refs: input.source_evidence_refs,
        source_run_ids: input.source_run_ids,
        case_id: input.case_id,
        derived_from: input.derived_from,
        created_at: input.created_at,
    })
}

fn m8_error(class: &'static str, message: impl Into<String>) -> ForgeError {
    ForgeError::Plan {
        class,
        message: message.into(),
    }
}

/// Register only the descriptor captured inside verified artifact evidence.
/// The caller supplies the verified ledger/evidence record, never a path or
/// narrative reference, so stdout and stderr cannot enter the catalog.
pub fn register_verified(
    evidence: &EvidenceRecord,
    name: String,
    version: String,
    case_id: String,
    derived_from: Vec<String>,
    created_at: String,
) -> Result<ArtifactRegistrationRecord, ForgeError> {
    if evidence.kind != EvidenceKind::Artifact
        || !evidence.uri.starts_with("artifacts/")
        || evidence.sha256.is_none()
    {
        return Err(m8_error(
            "artifact_registration_error",
            "durable artifact evidence is required",
        ));
    }
    let descriptor = evidence
        .metadata
        .get("artifact")
        .cloned()
        .ok_or_else(|| {
            m8_error(
                "artifact_registration_error",
                "descriptor metadata is required",
            )
        })
        .and_then(|value| {
            serde_json::from_value::<ArtifactDescriptor>(value).map_err(Into::into)
        })?;
    if descriptor.producer.run_id != evidence.run_id
        || evidence.sha256.as_deref() != Some(descriptor.content_sha256.as_str())
    {
        return Err(m8_error(
            "artifact_registration_error",
            "evidence and descriptor do not agree",
        ));
    }
    register(ArtifactRegistrationInput {
        descriptor,
        name,
        version,
        source_evidence_refs: vec![evidence.evidence_id.clone()],
        source_run_ids: vec![evidence.run_id.clone()],
        case_id,
        derived_from,
        created_at,
    })
}

pub fn append_registration(
    root: &Path,
    actor: &str,
    evidence_id: &str,
    input: VerifiedArtifactRegistrationInput,
) -> Result<ArtifactRegistrationRecord, ForgeError> {
    let stream = LedgerStream::open(root, "artifact-ip", actor)?;
    stream.verify()?;
    // Resolve the EvidenceRecord by evidence_id from a verified existing
    // run/case ledger. The caller may not supply or commit evidence here;
    // the pipeline is the sole producer of artifact_evidence.
    let facts = verified_ledger_facts(root)?;
    let evidence_value = facts
        .find("artifact_evidence", "evidence_id", evidence_id)
        .ok_or_else(|| {
            m8_error(
                "artifact_registration_error",
                format!("verified artifact evidence {evidence_id} is not ledgered"),
            )
        })?;
    let evidence: EvidenceRecord =
        serde_json::from_value(evidence_value.clone()).map_err(|error| {
            m8_error(
                "artifact_registration_error",
                format!("ledgered artifact evidence {evidence_id} is invalid: {error}"),
            )
        })?;
    let mut record = register_verified(
        &evidence,
        input.name,
        input.version,
        input.case_id,
        input.derived_from,
        input.created_at,
    )?;
    if !record.derived_from.is_empty() {
        let existing = stream.read_entries()?;
        let registrations = existing
            .iter()
            .filter(|entry| entry.record_kind == "artifact_registration")
            .map(|entry| {
                serde_json::from_value::<ArtifactRegistrationRecord>(entry.payload.clone())
            })
            .collect::<Result<Vec<_>, _>>()?;
        let lineages = record
            .derived_from
            .iter()
            .map(|source| {
                registrations
                    .iter()
                    .find(|registration| registration.artifact_id == *source)
                    .map(|registration| registration.lineage_id.clone())
                    .ok_or_else(|| {
                        m8_error(
                            "artifact_reference_error",
                            format!("derived source registration {source} is missing"),
                        )
                    })
            })
            .collect::<Result<BTreeSet<_>, _>>()?;
        if lineages.len() != 1 {
            return Err(m8_error(
                "artifact_lineage_error",
                "derived sources do not share one lineage",
            ));
        }
        record.lineage_id = lineages
            .into_iter()
            .next()
            .ok_or_else(|| m8_error("artifact_lineage_error", "derived lineage is missing"))?;
    }
    stream.commit_typed(
        "artifact_registration",
        vec![record.artifact_id.clone()],
        &record,
        vec![],
    )?;
    Ok(record)
}

pub fn transition(mut input: TransitionInput) -> Result<TransitionToken, ForgeError> {
    validate_transition(&input)?;
    if input.mode == TransitionMode::Derive {
        input.derived_from.sort();
        input.derived_from.dedup();
    }
    let mut token = TransitionToken {
        version: M8_RECORD_VERSION.into(),
        transition_token_id: input.transition_token_id,
        transition_kind: input.transition_kind,
        mode: input.mode,
        from_stage: input.from_stage,
        to_stage: input.to_stage,
        source_artifact_ids: input.source_artifact_ids,
        result_artifact_id: input.result_artifact_id,
        derived_from: input.derived_from,
        parent_transition_token_ids: input.parent_transition_token_ids,
        input_content_identities: input.input_content_identities,
        output_content_identity: input.output_content_identity,
        gate_profile_ref: input.gate_profile_ref,
        gate_profile_hash: input.gate_profile_hash,
        stage_gate_results: vec![GateResult {
            gate: "artifact_gate".into(),
            status: "accepted".into(),
            basis: "governed_case_settled".into(),
        }],
        actor_id: input.actor_id,
        approver_id: input.approver_id,
        approval_ref: input.approval_ref,
        authority_decision_refs: input.authority_decision_refs,
        criteria_ref: input.criteria_ref,
        evidence_refs: input.evidence_refs,
        settlement_ref: input.settlement_ref,
        strong_declaration_refs: input.strong_declaration_refs,
        value_evidence_refs: input.value_evidence_refs,
        value_evidence: input.value_evidence,
        rights_profile_ref: input.rights_profile_ref,
        semantic_refs: input.semantic_refs,
        semantic_model_ref: input.semantic_model_ref,
        identity_status_before: input.identity_status_before,
        identity_status_after: input.identity_status_after,
        attestation_ref: input.attestation_ref,
        degraded_controls: input.degraded_controls,
        case_id: input.case_id,
        run_id: input.run_id,
        plan_item_id: input.plan_item_id,
        created_at: input.created_at,
        transition_hash: String::new(),
    };
    token.transition_hash = token_hash(&token)?;
    Ok(token)
}

fn token_hash(token: &TransitionToken) -> Result<String, ForgeError> {
    let mut payload = token.clone();
    payload.transition_hash.clear();
    hash_canonical(&payload)
}

fn validate_transition_shape(input: &TransitionInput) -> Result<(), ForgeError> {
    let expected = match &input.transition_kind {
        TransitionKind::Synthesize => (ArtifactStage::Cognitive, ArtifactStage::Intellectual),
        TransitionKind::Productize => (ArtifactStage::Intellectual, ArtifactStage::Product),
        TransitionKind::Capitalize => (ArtifactStage::Product, ArtifactStage::Capital),
    };
    if (input.from_stage.clone(), input.to_stage.clone()) != expected {
        return Err(m8_error(
            "artifact_transition_error",
            "illegal maturity edge",
        ));
    }
    if input.source_artifact_ids.is_empty()
        || input.input_content_identities.is_empty()
        || input.gate_profile_ref.is_empty()
        || input.gate_profile_hash.is_empty()
    {
        return Err(m8_error(
            "artifact_transition_error",
            "governed transition references are required",
        ));
    }
    match &input.mode {
        TransitionMode::Promote => {
            if input.source_artifact_ids.len() != 1
                || input.input_content_identities.len() != 1
                || input.result_artifact_id != input.source_artifact_ids[0]
                || input.output_content_identity != input.input_content_identities[0]
            {
                return Err(m8_error(
                    "artifact_transition_error",
                    "promotion must preserve one artifact and content identity",
                ));
            }
        }
        TransitionMode::Derive => {
            let source_ids = input.source_artifact_ids.iter().collect::<BTreeSet<_>>();
            let derived_from = input.derived_from.iter().collect::<BTreeSet<_>>();
            if input.result_artifact_id.is_empty()
                || input.source_artifact_ids.len() != input.input_content_identities.len()
                || source_ids.len() != input.source_artifact_ids.len()
                || source_ids != derived_from
                || input.derived_from.len() != input.source_artifact_ids.len()
                || input
                    .source_artifact_ids
                    .iter()
                    .any(|source| source == &input.result_artifact_id)
                || input
                    .input_content_identities
                    .iter()
                    .any(|identity| identity == &input.output_content_identity)
            {
                return Err(m8_error(
                    "artifact_transition_error",
                    "derivation requires exact sources and new artifact/content identities",
                ));
            }
        }
    }
    if input.transition_kind == TransitionKind::Capitalize
        && (input.mode != TransitionMode::Promote
            || input.value_evidence_refs.is_empty()
            || input.rights_profile_ref.is_none()
            || input.semantic_refs.is_empty()
            || input.semantic_model_ref.is_none())
    {
        return Err(m8_error(
            "artifact_transition_error",
            "capitalization requires promote, rights, semantic, and value evidence inputs",
        ));
    }
    Ok(())
}

fn validate_transition(input: &TransitionInput) -> Result<(), ForgeError> {
    validate_transition_shape(input)?;
    if input.authority_decision_refs.is_empty()
        || input.criteria_ref.is_empty()
        || input.settlement_ref.is_empty()
        || (input.transition_kind == TransitionKind::Capitalize
            && (input
                .approver_id
                .as_deref()
                .is_none_or(|id| id == input.actor_id.as_str())
                || input.strong_declaration_refs.is_empty()))
    {
        return Err(m8_error(
            "artifact_transition_error",
            "governed transition references are required",
        ));
    }
    Ok(())
}

/// Validate the full source state before a transform can run. This is the
/// no-teleport boundary: callers invoke their transform only after it returns.
pub fn validate_before_transform<F>(
    input: &TransitionInput,
    registrations: &[ArtifactRegistrationRecord],
    tokens: &[TransitionToken],
    profile: &ArtifactGateProfile,
    transform: F,
) -> Result<(), ForgeError>
where
    F: FnOnce() -> Result<(), ForgeError>,
{
    validate_transition(input)?;
    validate_transition_sources(input, registrations, tokens, profile, transform)
}

pub fn validate_proposal_before_transform<F>(
    input: &TransitionInput,
    registrations: &[ArtifactRegistrationRecord],
    tokens: &[TransitionToken],
    profile: &ArtifactGateProfile,
    transform: F,
) -> Result<(), ForgeError>
where
    F: FnOnce() -> Result<(), ForgeError>,
{
    validate_transition_shape(input)?;
    validate_transition_sources(input, registrations, tokens, profile, transform)
}

fn validate_transition_sources<F>(
    input: &TransitionInput,
    registrations: &[ArtifactRegistrationRecord],
    tokens: &[TransitionToken],
    profile: &ArtifactGateProfile,
    transform: F,
) -> Result<(), ForgeError>
where
    F: FnOnce() -> Result<(), ForgeError>,
{
    if profile.transition_kind != input.transition_kind {
        return Err(m8_error(
            "artifact_gate_error",
            "profile and transition kind differ",
        ));
    }
    let states = rebuild(registrations, tokens)?;
    for (index, source_id) in input.source_artifact_ids.iter().enumerate() {
        let registration = registrations
            .iter()
            .find(|record| &record.artifact_id == source_id)
            .ok_or_else(|| m8_error("artifact_transition_error", "source missing"))?;
        if input.input_content_identities.get(index) != Some(&registration.content_identity) {
            return Err(m8_error(
                "artifact_transition_error",
                "source artifact and content identity pair differ",
            ));
        }
        if !profile.artifact_types.contains(&registration.artifact_type)
            || states
                .get(source_id)
                .is_none_or(|state| state.recognized_stage != input.from_stage)
        {
            return Err(m8_error(
                "artifact_transition_error",
                "stale or unauthorized source stage",
            ));
        }
    }
    if input.transition_kind == TransitionKind::Capitalize
        && (profile
            .qualifying_value_evidence_kinds
            .iter()
            .any(|kind| kind == "quality")
            || input.value_evidence.iter().any(|evidence| {
                evidence.kind == "quality"
                    || evidence.case_id == input.case_id
                    || !profile
                        .qualifying_value_evidence_kinds
                        .contains(&evidence.kind)
            }))
    {
        return Err(m8_error(
            "artifact_transition_error",
            "capitalization value evidence is not qualifying",
        ));
    }
    if input.from_stage != ArtifactStage::Cognitive {
        for source in &input.source_artifact_ids {
            let has_parent = input.parent_transition_token_ids.iter().any(|parent_id| {
                tokens
                    .iter()
                    .find(|token| token.transition_token_id == *parent_id)
                    .is_some_and(|parent| {
                        parent.to_stage == input.from_stage && parent.result_artifact_id == *source
                    })
            });
            if !has_parent {
                return Err(m8_error(
                    "artifact_lineage_error",
                    "non-cognitive source lacks an accepted parent token",
                ));
            }
        }
    }
    transform()
}

pub struct AuthorizedTransition {
    root: std::path::PathBuf,
    actor: String,
    action: AuthorityAction,
    decision_id: String,
    input_hash: String,
}

pub struct AuthorizedArtifactLifecycle {
    root: std::path::PathBuf,
    actor: String,
    action: AuthorityAction,
    decision_ref: String,
    decision_hash: String,
    input_hash: String,
}

pub fn canonical_lifecycle_action(input: &ArtifactLifecycleInput) -> AuthorityAction {
    AuthorityAction::Reserved {
        resource_type: "artifact_lifecycle".into(),
        resource_id: input.artifact_id.clone(),
        parameters: json!({
            "artifact_id": input.artifact_id,
            "previous_status": input.previous_status,
            "new_status": input.new_status,
            "reason": input.reason,
            "changed_at": input.changed_at,
            "actor_id": input.actor_id,
        }),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn authorize_lifecycle(
    grant: ActionGrant,
    action: &AuthorityAction,
    root: &Path,
    actor: &str,
    input: &ArtifactLifecycleInput,
    decision: &AuthorityDecision,
    committed: &CommittedRecordRef,
) -> Result<AuthorizedArtifactLifecycle, ForgeError> {
    let expected = canonical_lifecycle_action(input);
    let decision_hash = payload_hash(&serde_json::to_value(decision)?)?;
    if action != &expected
        || decision.operation != expected
        || decision.decision_id.is_empty()
        || decision.verdict != Verdict::Allow
        || decision.run_id != input.run_id
        || decision.plan_item_id != input.plan_item_id
        || decision.audit_record.case_id.as_deref() != Some(input.case_id.as_str())
        || committed.record_kind() != "authority_decision"
        || committed.payload_hash() != decision_hash
    {
        return Err(m8_error(
            "artifact_lifecycle_error",
            "lifecycle authority does not match the exact action and context",
        ));
    }
    grant.authorize(action, &input.run_id, &input.plan_item_id, root)?;
    Ok(AuthorizedArtifactLifecycle {
        root: root.into(),
        actor: actor.into(),
        action: expected,
        decision_ref: decision.decision_id.clone(),
        decision_hash,
        input_hash: hash_canonical(input)?,
    })
}

pub fn append_lifecycle(
    authorization: AuthorizedArtifactLifecycle,
    input: ArtifactLifecycleInput,
) -> Result<ArtifactLifecycleRecord, ForgeError> {
    if hash_canonical(&input)? != authorization.input_hash
        || canonical_lifecycle_action(&input) != authorization.action
        || input.reason.trim().is_empty()
        || input.changed_at.trim().is_empty()
        || input.previous_status == input.new_status
    {
        return Err(m8_error(
            "artifact_lifecycle_error",
            "lifecycle input is empty, unchanged, or substituted",
        ));
    }
    let stream = LedgerStream::open(&authorization.root, "artifact-ip", &authorization.actor)?;
    stream.verify()?;
    let mut registrations: Vec<ArtifactRegistrationRecord> = Vec::new();
    let mut tokens: Vec<TransitionToken> = Vec::new();
    let mut lifecycle: Vec<ArtifactLifecycleRecord> = Vec::new();
    for entry in stream.read_entries()? {
        match entry.record_kind.as_str() {
            "artifact_registration" => registrations.push(serde_json::from_value(entry.payload)?),
            "artifact_transition" => tokens.push(serde_json::from_value(entry.payload)?),
            "artifact_lifecycle" => lifecycle.push(serde_json::from_value(entry.payload)?),
            _ => {}
        }
    }
    let facts = verified_ledger_facts(&authorization.root)?;
    for record in &lifecycle {
        validate_lifecycle_authority(&facts, record)?;
    }
    let states = rebuild_with_lifecycle(&registrations, &tokens, &lifecycle)?;
    if states
        .get(&input.artifact_id)
        .is_none_or(|state| state.lifecycle_status != input.previous_status)
    {
        return Err(m8_error(
            "artifact_lifecycle_error",
            "lifecycle previous status does not match rebuilt ledger state",
        ));
    }
    let record = ArtifactLifecycleRecord {
        version: M8_RECORD_VERSION.into(),
        artifact_id: input.artifact_id,
        previous_status: input.previous_status,
        new_status: input.new_status,
        reason: input.reason,
        changed_at: input.changed_at,
        actor_id: input.actor_id,
        case_id: input.case_id,
        run_id: input.run_id,
        plan_item_id: input.plan_item_id,
        authority_decision_refs: vec![authorization.decision_ref],
        authority_decision_hash: authorization.decision_hash,
        authority_action_hash: hash_canonical(&authorization.action)?,
    };
    validate_lifecycle_authority(&facts, &record)?;
    stream.commit_typed(
        "artifact_lifecycle",
        vec![record.artifact_id.clone()],
        &record,
        record.authority_decision_refs.clone(),
    )?;
    Ok(record)
}

pub fn canonical_transition_action(
    input: &TransitionInput,
    source_licenses: &[String],
    profile: &ArtifactGateProfile,
) -> AuthorityAction {
    let mut licenses = source_licenses.to_vec();
    licenses.sort();
    licenses.dedup();
    AuthorityAction::Reserved {
        resource_type: "transition_artifact_stage".into(),
        resource_id: input.result_artifact_id.clone(),
        parameters: json!({
            "transition_kind": input.transition_kind,
            "from_stage": input.from_stage,
            "to_stage": input.to_stage,
            "mode": input.mode,
            "source_artifact_ids": input.source_artifact_ids,
            "result_artifact_id": input.result_artifact_id,
            "input_content_identities": input.input_content_identities,
            "output_content_identity": input.output_content_identity,
            "gate_profile_ref": input.gate_profile_ref,
            "required_settlement_strength": profile.required_settlement_strength.as_deref().unwrap_or("local"),
            "qualifying_value_evidence_kinds": profile.qualifying_value_evidence_kinds,
            "requires_approval": profile.requires_approval
                || profile.required_settlement_strength.as_deref() == Some("strong"),
            "actor_id": input.actor_id,
            "licenses": licenses,
        }),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn authorize_transition(
    grant: ActionGrant,
    action: &AuthorityAction,
    root: &Path,
    actor: &str,
    input: &TransitionInput,
    source_licenses: &[String],
    profile: &ArtifactGateProfile,
    decision_id: String,
    run_id: &str,
    case_id: &str,
    plan_item_id: &str,
) -> Result<AuthorizedTransition, ForgeError> {
    authorize_transition_in_workspace(
        grant,
        action,
        root,
        root,
        actor,
        input,
        source_licenses,
        profile,
        decision_id,
        run_id,
        case_id,
        plan_item_id,
    )
}

#[allow(clippy::too_many_arguments)]
fn authorize_transition_in_workspace(
    grant: ActionGrant,
    action: &AuthorityAction,
    root: &Path,
    workspace_root: &Path,
    actor: &str,
    input: &TransitionInput,
    source_licenses: &[String],
    profile: &ArtifactGateProfile,
    decision_id: String,
    run_id: &str,
    case_id: &str,
    plan_item_id: &str,
) -> Result<AuthorizedTransition, ForgeError> {
    let expected = canonical_transition_action(input, source_licenses, profile);
    if action != &expected || input.run_id != run_id || input.case_id != case_id {
        return Err(m8_error(
            "artifact_authority_error",
            "transition grant does not match canonical transition input/context",
        ));
    }
    grant.authorize(action, run_id, plan_item_id, workspace_root)?;
    Ok(AuthorizedTransition {
        root: root.into(),
        actor: actor.into(),
        action: expected,
        decision_id,
        input_hash: hash_canonical(input)?,
    })
}

pub fn append_transition(
    authorization: AuthorizedTransition,
    input: TransitionInput,
    registrations: &[ArtifactRegistrationRecord],
    tokens: &[TransitionToken],
    profile: &ArtifactGateProfile,
) -> Result<TransitionToken, ForgeError> {
    append_transition_inner(authorization, input, registrations, tokens, profile, None)
}

fn append_transition_once(
    authorization: AuthorizedTransition,
    input: TransitionInput,
    registrations: &[ArtifactRegistrationRecord],
    tokens: &[TransitionToken],
    profile: &ArtifactGateProfile,
    idempotency_key: &str,
) -> Result<TransitionToken, ForgeError> {
    append_transition_inner(
        authorization,
        input,
        registrations,
        tokens,
        profile,
        Some(idempotency_key),
    )
}

fn append_transition_inner(
    authorization: AuthorizedTransition,
    mut input: TransitionInput,
    registrations: &[ArtifactRegistrationRecord],
    tokens: &[TransitionToken],
    profile: &ArtifactGateProfile,
    idempotency_key: Option<&str>,
) -> Result<TransitionToken, ForgeError> {
    let source_licenses = input
        .source_artifact_ids
        .iter()
        .map(|source| {
            registrations
                .iter()
                .find(|registration| registration.artifact_id == *source)
                .map(|registration| registration.license.clone())
                .ok_or_else(|| m8_error("artifact_reference_error", "source license is missing"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if canonical_transition_action(&input, &source_licenses, profile) != authorization.action
        || hash_canonical(&input)? != authorization.input_hash
    {
        return Err(m8_error(
            "artifact_authority_error",
            "authorized transition was substituted before commit",
        ));
    }
    input.authority_decision_refs = vec![authorization.decision_id];
    validate_before_transform(&input, registrations, tokens, profile, || Ok(()))?;
    let token = transition(input)?;
    validate_ledger_references(
        &authorization.root,
        &authorization.actor,
        registrations,
        tokens,
        &token,
        profile,
    )?;
    let stream = LedgerStream::open(&authorization.root, "artifact-ip", &authorization.actor)?;
    if let Some(idempotency_key) = idempotency_key {
        stream.commit_typed_once(
            "artifact_transition",
            idempotency_key,
            vec![token.result_artifact_id.clone()],
            &token,
            token.authority_decision_refs.clone(),
        )?;
    } else {
        stream.commit_typed(
            "artifact_transition",
            vec![token.result_artifact_id.clone()],
            &token,
            token.authority_decision_refs.clone(),
        )?;
    }
    Ok(token)
}

pub fn append_value_evidence(
    root: &Path,
    actor: &str,
    mut record: ValueEvidenceRecord,
) -> Result<ValueEvidenceRecord, ForgeError> {
    if record.evidence_refs.is_empty() {
        return Err(m8_error(
            "artifact_reference_error",
            "value evidence requires underlying accepted evidence",
        ));
    }
    let facts = verified_ledger_facts(root)?;
    for evidence_ref in &record.evidence_refs {
        let evidence = require_record(
            &facts,
            "artifact_value_source_evidence",
            "evidence_id",
            evidence_ref,
        )?;
        verify_embedded_hash(evidence, "record_hash", "artifact_reference_error")?;
        let source: ValueEvidenceSourceRecord =
            serde_json::from_value(evidence.clone()).map_err(|error| {
                m8_error(
                    "artifact_reference_error",
                    format!("value evidence source is invalid: {error}"),
                )
            })?;
        validate_value_evidence_source(&facts, &source, &source.originating_case_id)?;
        if evidence.get("kind").and_then(Value::as_str) != Some(record.kind.as_str())
            || evidence.get("case_id").and_then(Value::as_str) != Some(record.case_id.as_str())
        {
            return Err(m8_error(
                "artifact_reference_error",
                "underlying value evidence is rejected or has mismatched kind/case",
            ));
        }
    }
    record.accepted = true;
    record.version = M8_RECORD_VERSION.into();
    record.record_hash.clear();
    record.record_hash = hash_canonical(&record)?;
    LedgerStream::open(root, "artifact-ip", actor)?.commit_typed(
        "artifact_value_evidence",
        vec![record.value_evidence_id.clone(), record.case_id.clone()],
        &record,
        vec![],
    )?;
    Ok(record)
}

pub fn append_value_evidence_source(
    root: &Path,
    actor: &str,
    mut record: ValueEvidenceSourceRecord,
) -> Result<ValueEvidenceSourceRecord, ForgeError> {
    let facts = verified_ledger_facts(root)?;
    validate_value_evidence_source(&facts, &record, &record.originating_case_id)?;
    record.version = M8_RECORD_VERSION.into();
    record.record_hash.clear();
    record.record_hash = hash_canonical(&record)?;
    LedgerStream::open(root, "artifact-ip", actor)?.commit_typed(
        "artifact_value_source_evidence",
        vec![record.evidence_id.clone(), record.case_id.clone()],
        &record,
        vec![],
    )?;
    Ok(record)
}

pub fn append_rights_profile(
    authorization: AuthorizedRightsProfile,
    mut record: RightsProfileRecord,
) -> Result<RightsProfileRecord, ForgeError> {
    let review_label = match record.review_status {
        ReviewStatus::Draft => "draft",
        ReviewStatus::Reviewed => "reviewed",
        ReviewStatus::Approved => "approved",
        ReviewStatus::Capitalized => "capitalized",
    };
    if record.artifact_id != authorization.artifact_id
        || record.license != authorization.license
        || (!authorization.required_review.is_empty()
            && !authorization
                .required_review
                .iter()
                .any(|status| status == review_label))
        || authorization.action
            != canonical_rights_action(
                &ArtifactRegistrationRecord {
                    version: M8_RECORD_VERSION.into(),
                    artifact_id: authorization.artifact_id.clone(),
                    lineage_id: String::new(),
                    content_identity: String::new(),
                    descriptor_hash: String::new(),
                    identity_scheme: String::new(),
                    artifact_type: ArtifactType::SeaModel,
                    name: String::new(),
                    artifact_version: String::new(),
                    content_sha256: String::new(),
                    declared_stage: None,
                    legacy_pre_mint_identity: String::new(),
                    owner: String::new(),
                    license: authorization.license.clone(),
                    review_status: record.review_status.clone(),
                    source_evidence_refs: vec![],
                    source_run_ids: vec![],
                    case_id: String::new(),
                    derived_from: vec![],
                    created_at: String::new(),
                },
                &authorization.required_review,
            )
    {
        return Err(m8_error(
            "artifact_reference_error",
            "rights profile does not match authorized artifact, license, review, or action",
        ));
    }
    record.authority_decision_ref = authorization.decision_ref;
    record.authority_decision_hash = authorization.decision_hash;
    record.authority_action_hash = authorization.action_hash;
    record.version = M8_RECORD_VERSION.into();
    record.accepted = true;
    record.record_hash.clear();
    record.record_hash = hash_canonical(&record)?;
    let decision_ref = record.authority_decision_ref.clone();
    LedgerStream::open(&authorization.root, "artifact-ip", &authorization.actor)?.commit_typed(
        "artifact_rights_profile",
        vec![record.rights_profile_id.clone(), record.artifact_id.clone()],
        &record,
        vec![decision_ref],
    )?;
    Ok(record)
}

pub fn append_domain_model_ref(
    root: &Path,
    actor: &str,
    model: &sea_forge_domainforge::DomainModel,
) -> Result<(), ForgeError> {
    let model_ref = &model.model_ref;
    if model_ref.validation_evidence_refs.is_empty() || model_ref.source_refs.is_empty() {
        return Err(m8_error(
            "artifact_reference_error",
            "domain model ref requires nonempty validation evidence and source refs",
        ));
    }
    // Verify committed source evidence hashes: each source_ref's sha256 must
    // resolve to a ledgered artifact_evidence or run_evidence record.
    let facts = verified_ledger_facts(root)?;
    for source in &model_ref.source_refs {
        let committed = facts
            .by_kind
            .get("artifact_evidence")
            .into_iter()
            .flatten()
            .chain(facts.by_kind.get("run_evidence").into_iter().flatten())
            .any(|record| {
                record.get("sha256").and_then(Value::as_str) == Some(source.sha256.as_str())
            });
        if !committed {
            return Err(m8_error(
                "artifact_reference_error",
                format!(
                    "domain model source {} sha256 is not committed in the ledger",
                    source.uri
                ),
            ));
        }
    }
    LedgerStream::open(root, "artifact-ip", actor)?.commit_typed(
        "domain_model_ref",
        vec![model_ref.semantic_model_sha256.clone()],
        model_ref,
        vec![],
    )?;
    Ok(())
}

#[derive(Default)]
struct LedgerFacts {
    by_kind: BTreeMap<String, Vec<Value>>,
    by_kind_and_stream: BTreeMap<String, Vec<(String, Value)>>,
    attestations: BTreeMap<String, Value>,
}

impl LedgerFacts {
    fn find(&self, kind: &str, id_field: &str, id: &str) -> Option<&Value> {
        self.by_kind
            .get(kind)?
            .iter()
            .find(|record| record.get(id_field).and_then(Value::as_str) == Some(id))
    }

    fn find_in_stream(
        &self,
        kind: &str,
        stream_id: &str,
        id_field: &str,
        id: &str,
    ) -> Option<&Value> {
        self.by_kind_and_stream
            .get(kind)?
            .iter()
            .find_map(|(stream, record)| {
                (stream == stream_id && record.get(id_field).and_then(Value::as_str) == Some(id))
                    .then_some(record)
            })
    }
}

fn verified_ledger_facts(root: &Path) -> Result<LedgerFacts, ForgeError> {
    let manager = LedgerManager::new(root)?;
    let mut facts = LedgerFacts::default();
    for stream_id in manager.list_stream_ids()? {
        let stream = manager.open_stream(&stream_id, "artifact-ip-rebuild")?;
        stream.verify()?;
        for entry in stream.read_entries()? {
            if entry.record_kind == "artifact_identity_attestation" {
                facts.attestations.insert(
                    format!("ifl:token:{}:{}", entry.ledger_id, entry.append_ordinal),
                    entry.payload.clone(),
                );
            }
            facts
                .by_kind
                .entry(entry.record_kind.clone())
                .or_default()
                .push(entry.payload.clone());
            facts
                .by_kind_and_stream
                .entry(entry.record_kind)
                .or_default()
                .push((stream_id.clone(), entry.payload));
        }
    }
    Ok(facts)
}

fn validate_value_evidence_source(
    facts: &LedgerFacts,
    record: &ValueEvidenceSourceRecord,
    originating_case_id: &str,
) -> Result<(), ForgeError> {
    if record.evidence_refs.is_empty()
        || record.source_run_id.is_empty()
        || record.settlement_ref.is_empty()
        || record.case_id == originating_case_id
        || record.originating_case_id != originating_case_id
    {
        return Err(m8_error(
            "artifact_reference_error",
            "value source requires external run evidence and settlement",
        ));
    }
    let stream_id = format!("case-{}", record.case_id);
    for evidence_ref in &record.evidence_refs {
        let evidence = facts
            .find_in_stream("run_evidence", &stream_id, "evidence_id", evidence_ref)
            .ok_or_else(|| {
                m8_error(
                    "artifact_reference_error",
                    format!("ledgered run evidence {evidence_ref} is missing"),
                )
            })?;
        let evidence: EvidenceRecord =
            serde_json::from_value(evidence.clone()).map_err(|error| {
                m8_error(
                    "artifact_reference_error",
                    format!("run evidence {evidence_ref} is invalid: {error}"),
                )
            })?;
        if evidence.run_id != record.source_run_id
            || evidence.sha256.as_deref().is_none_or(str::is_empty)
            || evidence
                .metadata
                .get("canonical_value_evidence_kind")
                .and_then(Value::as_str)
                != Some(record.kind.as_str())
        {
            return Err(m8_error(
                "artifact_reference_error",
                "run evidence hash, run linkage, or canonical value kind mismatch",
            ));
        }
    }
    let settlement = facts
        .find_in_stream(
            "settlement_event",
            &stream_id,
            "settlement_id",
            &record.settlement_ref,
        )
        .ok_or_else(|| m8_error("artifact_reference_error", "source settlement is missing"))?;
    if settlement.get("status").and_then(Value::as_str) != Some("accepted")
        || settlement.get("run_id").and_then(Value::as_str) != Some(record.source_run_id.as_str())
    {
        return Err(m8_error(
            "artifact_reference_error",
            "source settlement is not accepted or linked to the source run",
        ));
    }
    Ok(())
}

fn verify_embedded_hash(value: &Value, field: &str, class: &'static str) -> Result<(), ForgeError> {
    let expected = value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| m8_error(class, format!("record is missing {field}")))?;
    let mut canonical = value.clone();
    canonical
        .as_object_mut()
        .ok_or_else(|| m8_error(class, "record is not an object"))?
        .insert(field.into(), Value::String(String::new()));
    if hash_canonical(&canonical)? != expected {
        return Err(m8_error(class, format!("{field} mismatch")));
    }
    Ok(())
}

fn require_record<'a>(
    facts: &'a LedgerFacts,
    kind: &str,
    id_field: &str,
    id: &str,
) -> Result<&'a Value, ForgeError> {
    facts.find(kind, id_field, id).ok_or_else(|| {
        m8_error(
            "artifact_reference_error",
            format!("ledgered {kind} {id} is missing"),
        )
    })
}

fn metadata_value_is_present(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::String(value) => !value.trim().is_empty(),
        Value::Array(value) => !value.is_empty(),
        Value::Object(value) => !value.is_empty(),
        Value::Bool(_) | Value::Number(_) => true,
    }
}

fn product_contract_field(selector: &str) -> Option<&str> {
    match selector {
        "product_contract.intended_consumer"
        | "product_contract.job_to_be_done"
        | "product_contract.usage_contract"
        | "product_contract.interface_contract"
        | "product_contract.operating_context"
        | "product_contract.compatibility_contract"
        | "product_contract.required_documentation"
        | "product_contract.acceptance_criteria"
        | "product_contract.maintenance_obligations"
        | "product_contract.safety_obligations"
        | "product_contract.recovery_obligations"
        | "product_contract.support_obligations" => selector.strip_prefix("product_contract."),
        _ => None,
    }
}

fn validate_required_metadata(
    facts: &LedgerFacts,
    registration: &ArtifactRegistrationRecord,
    profile: &ArtifactGateProfile,
) -> Result<(), ForgeError> {
    for selector in &profile.required_metadata {
        let field = product_contract_field(selector).ok_or_else(|| {
            m8_error(
                "artifact_gate_error",
                format!("unknown or empty required metadata selector {selector:?}"),
            )
        })?;
        let present = registration
            .source_evidence_refs
            .iter()
            .any(|evidence_ref| {
                facts
                    .find("artifact_evidence", "evidence_id", evidence_ref)
                    .and_then(|evidence| evidence.get("metadata"))
                    .and_then(|metadata| metadata.get("product_contract"))
                    .and_then(|contract| contract.get(field))
                    .is_some_and(metadata_value_is_present)
            });
        if !present {
            return Err(m8_error(
                "artifact_gate_error",
                format!("required result metadata {selector} is missing or empty"),
            ));
        }
    }
    Ok(())
}

fn validate_semantic_anchors(
    facts: &LedgerFacts,
    token: &TransitionToken,
    profile: &ArtifactGateProfile,
) -> Result<(), ForgeError> {
    if profile.semantic_reference_classes.is_empty() && token.semantic_refs.is_empty() {
        return Ok(());
    }
    let model_ref = token.semantic_model_ref.as_deref().ok_or_else(|| {
        m8_error(
            "artifact_gate_error",
            "semantic anchors require a ledgered DomainModelRef",
        )
    })?;
    let model: sea_forge_domainforge::DomainModelRef = serde_json::from_value(
        require_record(
            facts,
            "domain_model_ref",
            "semantic_model_sha256",
            model_ref,
        )?
        .clone(),
    )
    .map_err(|error| {
        m8_error(
            "artifact_gate_error",
            format!("ledgered DomainModelRef is invalid: {error}"),
        )
    })?;
    for required in &profile.semantic_reference_classes {
        if !token
            .semantic_refs
            .iter()
            .any(|anchor| &anchor.class == required)
        {
            return Err(m8_error(
                "artifact_gate_error",
                "required semantic anchor class is missing",
            ));
        }
    }
    for anchor in &token.semantic_refs {
        if anchor.reference.trim().is_empty()
            || !profile.semantic_reference_classes.contains(&anchor.class)
        {
            return Err(m8_error(
                "artifact_gate_error",
                "semantic anchor class is empty, unknown, or not allowed by the gate",
            ));
        }
        let known = match anchor.class {
            SemanticReferenceClass::Concept => model.concept_refs.contains(&anchor.reference),
            SemanticReferenceClass::Class => model.class_refs.contains(&anchor.reference),
        };
        if !known {
            return Err(m8_error(
                "artifact_gate_error",
                "semantic anchor is absent from the ledgered DomainModelRef",
            ));
        }
    }
    Ok(())
}

fn validate_lifecycle_authority(
    facts: &LedgerFacts,
    record: &ArtifactLifecycleRecord,
) -> Result<(), ForgeError> {
    if record.version != M8_RECORD_VERSION
        || record.authority_decision_refs.len() != 1
        || record.reason.trim().is_empty()
        || record.changed_at.trim().is_empty()
        || record.previous_status == record.new_status
    {
        return Err(m8_error(
            "artifact_lifecycle_error",
            "lifecycle record shape is invalid",
        ));
    }
    let decision_ref = &record.authority_decision_refs[0];
    let decision: AuthorityDecision = serde_json::from_value(
        facts
            .find("authority_decision", "decision_id", decision_ref)
            .ok_or_else(|| {
                m8_error(
                    "artifact_lifecycle_error",
                    "lifecycle authority decision is missing",
                )
            })?
            .clone(),
    )
    .map_err(|error| {
        m8_error(
            "artifact_lifecycle_error",
            format!("lifecycle authority decision is invalid: {error}"),
        )
    })?;
    let input = ArtifactLifecycleInput {
        artifact_id: record.artifact_id.clone(),
        previous_status: record.previous_status.clone(),
        new_status: record.new_status.clone(),
        reason: record.reason.clone(),
        changed_at: record.changed_at.clone(),
        actor_id: record.actor_id.clone(),
        case_id: record.case_id.clone(),
        run_id: record.run_id.clone(),
        plan_item_id: record.plan_item_id.clone(),
    };
    let action = canonical_lifecycle_action(&input);
    let action_hash = hash_canonical(&action)?;
    if payload_hash(&serde_json::to_value(&decision)?)? != record.authority_decision_hash
        || decision.operation != action
        || action_hash != record.authority_action_hash
        || decision.verdict != Verdict::Allow
        || decision.run_id != record.run_id
        || decision.plan_item_id != record.plan_item_id
        || decision.audit_record.case_id.as_deref() != Some(record.case_id.as_str())
        || decision.audit_record.run_id.as_deref() != Some(record.run_id.as_str())
        || decision
            .action_request
            .evidence
            .get("payload_hash")
            .and_then(Value::as_str)
            != Some(action_hash.as_str())
    {
        return Err(m8_error(
            "artifact_lifecycle_error",
            "lifecycle authority does not match the exact record action and context",
        ));
    }
    Ok(())
}

fn validate_attestation_authority(
    facts: &LedgerFacts,
    attestation: &Value,
) -> Result<(), ForgeError> {
    let artifact_id = attestation
        .get("artifact_id")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            m8_error(
                "artifact_attestation_error",
                "attestation artifact is missing",
            )
        })?;
    let content_identity = attestation
        .get("content_identity")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            m8_error(
                "artifact_attestation_error",
                "attestation content identity is missing",
            )
        })?;
    let descriptor_hash = attestation
        .get("descriptor_hash")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            m8_error(
                "artifact_attestation_error",
                "attestation descriptor hash is missing",
            )
        })?;
    let decision_ref = attestation
        .get("authority_decision_ref")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            m8_error(
                "artifact_attestation_error",
                "attestation authority decision is missing",
            )
        })?;
    let action: AuthorityAction = serde_json::from_value(
        attestation
            .get("authority_action")
            .cloned()
            .ok_or_else(|| {
                m8_error(
                    "artifact_attestation_error",
                    "attestation authority action is missing",
                )
            })?,
    )?;
    let action_hash = hash_canonical(&action)?;
    let decision: AuthorityDecision = serde_json::from_value(
        require_record(facts, "authority_decision", "decision_id", decision_ref)?.clone(),
    )?;
    let exact_action = matches!(
        &action,
        AuthorityAction::Reserved { resource_type, resource_id, parameters }
            if resource_type == "attest_artifact_identity"
                && resource_id == artifact_id
                && parameters.get("artifact_id").and_then(Value::as_str) == Some(artifact_id)
                && parameters.get("content_identity").and_then(Value::as_str) == Some(content_identity)
                && parameters.get("descriptor_hash").and_then(Value::as_str) == Some(descriptor_hash)
    );
    if !exact_action
        || attestation
            .get("authority_action_hash")
            .and_then(Value::as_str)
            != Some(action_hash.as_str())
        || attestation
            .get("authority_decision_hash")
            .and_then(Value::as_str)
            != Some(payload_hash(&serde_json::to_value(&decision)?)?.as_str())
        || decision.verdict != Verdict::Allow
        || decision.operation != action
        || decision.audit_record.case_id.as_deref() != Some("attestation_ingress")
        || decision.audit_record.run_id.as_deref() != Some("attestation_ingress")
        || decision.plan_item_id != "attestation_ingress"
        || decision
            .action_request
            .evidence
            .get("payload_hash")
            .and_then(Value::as_str)
            != Some(action_hash.as_str())
    {
        return Err(m8_error(
            "artifact_attestation_error",
            "attestation authority does not bind the exact action",
        ));
    }
    Ok(())
}

fn validate_ledger_references(
    root: &Path,
    _actor: &str,
    registrations: &[ArtifactRegistrationRecord],
    existing_tokens: &[TransitionToken],
    token: &TransitionToken,
    profile: &ArtifactGateProfile,
) -> Result<(), ForgeError> {
    let facts = verified_ledger_facts(root)?;
    for registration in registrations {
        let ledgered = require_record(
            &facts,
            "artifact_registration",
            "artifact_id",
            &registration.artifact_id,
        )?;
        if ledgered.get("descriptor_hash").and_then(Value::as_str)
            != Some(registration.descriptor_hash.as_str())
        {
            return Err(m8_error(
                "artifact_reference_error",
                "registration descriptor hash mismatch",
            ));
        }
        for evidence_ref in &registration.source_evidence_refs {
            let evidence =
                require_record(&facts, "artifact_evidence", "evidence_id", evidence_ref)?;
            let descriptor = evidence
                .get("metadata")
                .and_then(|metadata| metadata.get("artifact"))
                .ok_or_else(|| {
                    m8_error(
                        "artifact_reference_error",
                        "artifact evidence descriptor is missing",
                    )
                })?;
            if hash_canonical(descriptor)? != registration.descriptor_hash
                || evidence.get("run_id").and_then(Value::as_str)
                    != registration.source_run_ids.first().map(String::as_str)
            {
                return Err(m8_error(
                    "artifact_reference_error",
                    "artifact evidence descriptor hash or run linkage mismatch",
                ));
            }
        }
    }
    for source in &token.source_artifact_ids {
        require_record(&facts, "artifact_registration", "artifact_id", source)?;
    }
    for parent in &token.parent_transition_token_ids {
        let ledgered =
            require_record(&facts, "artifact_transition", "transition_token_id", parent)?;
        let known = existing_tokens
            .iter()
            .find(|candidate| candidate.transition_token_id == *parent)
            .ok_or_else(|| {
                m8_error(
                    "artifact_lineage_error",
                    format!("parent token {parent} is missing"),
                )
            })?;
        if ledgered.get("transition_hash").and_then(Value::as_str)
            != Some(known.transition_hash.as_str())
        {
            return Err(m8_error(
                "artifact_lineage_error",
                "parent token hash mismatch",
            ));
        }
    }
    let gate = require_record(
        &facts,
        "artifact_gate_profile",
        "profile_ref",
        &token.gate_profile_ref,
    )?;
    if hash_canonical(gate)? != token.gate_profile_hash {
        return Err(m8_error(
            "artifact_reference_error",
            "gate profile hash mismatch",
        ));
    }
    let transition_input: TransitionInput = serde_json::from_value(serde_json::to_value(token)?)
        .map_err(|error| {
            m8_error(
                "artifact_authority_error",
                format!("transition token cannot reconstruct authority input: {error}"),
            )
        })?;
    let source_licenses = token
        .source_artifact_ids
        .iter()
        .map(|source| {
            registrations
                .iter()
                .find(|registration| registration.artifact_id == *source)
                .map(|registration| registration.license.clone())
                .ok_or_else(|| {
                    m8_error(
                        "artifact_authority_error",
                        format!("source license for {source} is missing"),
                    )
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let exact_action = canonical_transition_action(&transition_input, &source_licenses, profile);
    let exact_action_hash = hash_canonical(&exact_action)?;
    let mut escalated_decision_ref = None;
    for decision_ref in &token.authority_decision_refs {
        let decision_value = facts
            .by_kind
            .get("authority_decision")
            .and_then(|records| {
                records.iter().find(|decision| {
                    decision.get("decision_id").and_then(Value::as_str)
                        == Some(decision_ref.as_str())
                        && decision.get("run_id").and_then(Value::as_str)
                            == Some(token.run_id.as_str())
                })
            })
            .ok_or_else(|| {
                m8_error(
                    "artifact_authority_error",
                    format!("authority decision {decision_ref} is missing"),
                )
            })?;
        let decision: AuthorityDecision =
            serde_json::from_value(decision_value.clone()).map_err(|error| {
                m8_error(
                    "artifact_authority_error",
                    format!("authority decision {decision_ref} is invalid: {error}"),
                )
            })?;
        let verdict_authorizes = decision.verdict == Verdict::Allow
            || (decision.verdict == Verdict::Escalate && token.approval_ref.is_some());
        if decision.verdict == Verdict::Escalate {
            escalated_decision_ref = Some(decision.decision_id.clone());
        }
        if !verdict_authorizes
            || decision.operation != exact_action
            || decision.run_id != token.run_id
            || decision.plan_item_id != token.plan_item_id
            || decision.audit_record.case_id.as_deref() != Some(token.case_id.as_str())
            || decision.audit_record.run_id.as_deref() != Some(token.run_id.as_str())
            || decision
                .action_request
                .context
                .get("run_id")
                .and_then(Value::as_str)
                != Some(token.run_id.as_str())
            || decision
                .action_request
                .context
                .get("plan_item_id")
                .and_then(Value::as_str)
                != Some(token.plan_item_id.as_str())
            || decision
                .action_request
                .evidence
                .get("payload_hash")
                .and_then(Value::as_str)
                != Some(exact_action_hash.as_str())
        {
            return Err(m8_error(
                "artifact_authority_error",
                "authority decision does not allow the exact transition action and context",
            ));
        }
    }
    let criteria = require_record(
        &facts,
        "settlement_criteria",
        "criteria_id",
        &token.criteria_ref,
    )?;
    verify_embedded_hash(criteria, "criteria_record_hash", "artifact_reference_error")?;
    let criteria_record: SettlementCriteriaRecord = serde_json::from_value(criteria.clone())
        .map_err(|error| {
            m8_error(
                "artifact_reference_error",
                format!("settlement criteria record is invalid: {error}"),
            )
        })?;
    if hash_canonical(&criteria_record.criteria)? != criteria_record.criteria_sha256 {
        return Err(m8_error(
            "artifact_reference_error",
            "criteria_sha256 mismatch",
        ));
    }
    if let Some(decision_ref) = escalated_decision_ref {
        let approval_ref = token.approval_ref.as_deref().ok_or_else(|| {
            m8_error(
                "artifact_reference_error",
                "escalated transition has no approval resolution",
            )
        })?;
        let approval: ApprovalRequest = serde_json::from_value(
            require_record(&facts, "approval_resolution", "approval_id", approval_ref)?.clone(),
        )
        .map_err(|error| {
            m8_error(
                "artifact_reference_error",
                format!("transition approval is invalid: {error}"),
            )
        })?;
        if approval.status != ApprovalStatus::Approved
            || approval.decision_id != decision_ref
            || approval.case_id != token.case_id
            || approval.run_id != token.run_id
            || approval.plan_item_id != token.plan_item_id
            || approval.criteria_ref.as_deref() != Some(token.criteria_ref.as_str())
            || approval.criteria_sha256.as_deref() != Some(criteria_record.criteria_sha256.as_str())
            || approval.criteria_record_hash.as_deref()
                != Some(criteria_record.criteria_record_hash.as_str())
            || approval.resolved_by.as_deref() != token.approver_id.as_deref()
            || approval.resolved_by.as_deref() == Some(token.actor_id.as_str())
            || approval
                .resolved_at
                .as_ref()
                .is_none_or(|resolved_at| resolved_at > &approval.expires_at)
        {
            return Err(m8_error(
                "artifact_reference_error",
                "transition approval is missing, rejected, expired, or detached",
            ));
        }
    }
    let settlement = require_record(
        &facts,
        "settlement_event",
        "settlement_id",
        &token.settlement_ref,
    )?;
    if settlement.get("status").and_then(Value::as_str) != Some("accepted")
        || settlement.get("run_id").and_then(Value::as_str) != Some(token.run_id.as_str())
        || settlement.get("criteria_ref").and_then(Value::as_str)
            != Some(token.criteria_ref.as_str())
    {
        return Err(m8_error(
            "artifact_reference_error",
            "settlement is not accepted or is detached from criteria/run",
        ));
    }
    let settlement_basis = settlement
        .get("basis")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    for evaluator_ref in &profile.evaluator_refs {
        let threshold = profile
            .evaluator_thresholds
            .get(evaluator_ref)
            .copied()
            .ok_or_else(|| {
                m8_error(
                    "artifact_reference_error",
                    format!("evaluator {evaluator_ref} has no gate threshold"),
                )
            })?;
        let prefix = format!("evaluator_score:{evaluator_ref}=");
        let score = settlement_basis
            .iter()
            .find_map(|basis| basis.strip_prefix(&prefix))
            .and_then(|score| score.parse::<f64>().ok())
            .ok_or_else(|| {
                m8_error(
                    "artifact_reference_error",
                    format!("accepted settlement lacks evaluator {evaluator_ref}"),
                )
            })?;
        if score < threshold {
            return Err(m8_error(
                "artifact_reference_error",
                format!("evaluator {evaluator_ref} is below threshold"),
            ));
        }
    }
    for evidence_ref in &token.evidence_refs {
        require_record(&facts, "artifact_evidence", "evidence_id", evidence_ref)?;
    }
    let result_registration = registrations
        .iter()
        .find(|registration| registration.artifact_id == token.result_artifact_id)
        .ok_or_else(|| {
            m8_error(
                "artifact_reference_error",
                "transition result registration is missing",
            )
        })?;
    if !profile
        .artifact_types
        .contains(&result_registration.artifact_type)
    {
        return Err(m8_error(
            "artifact_reference_error",
            "gate profile does not cover result artifact type",
        ));
    }
    validate_required_metadata(&facts, result_registration, profile)?;
    validate_semantic_anchors(&facts, token, profile)?;
    if profile.required_settlement_strength.as_deref() == Some("strong")
        && token.strong_declaration_refs.is_empty()
    {
        return Err(m8_error(
            "artifact_reference_error",
            "required strong settlement declaration is missing",
        ));
    }
    match profile.attestation_mode.as_deref() {
        Some("required") => {
            if token.identity_status_after != IdentityStatus::Attested {
                return Err(m8_error(
                    "artifact_attestation_error",
                    "required attestation is not attested",
                ));
            }
        }
        Some("pre_mint_only") if token.identity_status_after == IdentityStatus::PreMint => {
            if token.degraded_controls.is_empty() {
                return Err(m8_error(
                    "artifact_attestation_error",
                    "pre_mint degraded token has no compensating controls",
                ));
            }
        }
        _ => {}
    }
    if token.identity_status_after == IdentityStatus::Attested {
        let attestation_ref = token.attestation_ref.as_deref().ok_or_else(|| {
            m8_error(
                "artifact_attestation_error",
                "attested token has no attestation reference",
            )
        })?;
        let attestation = facts.attestations.get(attestation_ref).ok_or_else(|| {
            m8_error(
                "artifact_attestation_error",
                "attestation ledger entry is missing",
            )
        })?;
        validate_attestation_authority(&facts, attestation)?;
        if attestation.get("artifact_id").and_then(Value::as_str)
            != Some(result_registration.artifact_id.as_str())
            || attestation.get("content_identity").and_then(Value::as_str)
                != Some(result_registration.content_identity.as_str())
            || attestation.get("descriptor_hash").and_then(Value::as_str)
                != Some(result_registration.descriptor_hash.as_str())
        {
            return Err(m8_error(
                "artifact_attestation_error",
                "attestation ledger record does not match the artifact",
            ));
        }
    }
    for declaration_ref in &token.strong_declaration_refs {
        let declaration_value = require_record(
            &facts,
            "settlement_declaration",
            "declaration_id",
            declaration_ref,
        )?;
        let declaration: SettlementDeclaration = serde_json::from_value(declaration_value.clone())
            .map_err(|error| {
                m8_error(
                    "artifact_reference_error",
                    format!("strong settlement declaration is invalid: {error}"),
                )
            })?;
        let mut unhashed = declaration.clone();
        unhashed.declaration_hash.clear();
        let source_evidence_is_ledgered = declaration
            .source_evidence_refs
            .iter()
            .chain(&declaration.verification_evidence_refs)
            .all(|evidence_ref| {
                facts
                    .find("artifact_evidence", "evidence_id", evidence_ref)
                    .or_else(|| facts.find("run_evidence", "evidence_id", evidence_ref))
                    .is_some()
            });
        if hash_canonical(&unhashed)? != declaration.declaration_hash
            || !declaration_qualifies(&declaration, &default_v02_policy())
            || declaration.settlement_ref != token.settlement_ref
            || declaration.criteria_ref != token.criteria_ref
            || declaration.criteria_sha256 != criteria_record.criteria_sha256
            || declaration.criteria_record_hash != criteria_record.criteria_record_hash
            || declaration.criteria_declared_at != criteria_record.declared_at
            || declaration.origin_refs != criteria_record.origin_refs
            || declaration.run_id != token.run_id
            || declaration.case_id != token.case_id
            || declaration.plan_item_id != token.plan_item_id
            || declaration.claim_manifest_sha256.is_empty()
            || declaration.verifier_ref.is_empty()
            || declaration.verifier_sha256.is_empty()
            || declaration.declarer.actor_id.is_empty()
            || declaration.declarer.authority_ref.is_empty()
            || declaration.declarer.role.is_empty()
            || declaration.declarer.standing_basis.is_empty()
            || declaration.independence.acting_entity_id != token.actor_id
            || declaration.independence.basis.is_empty()
            || declaration.declarer.actor_id == declaration.independence.acting_entity_id
            || declaration.reliability.basis.is_empty()
            || declaration.source_evidence_refs.is_empty()
            || declaration.verification_evidence_refs.is_empty()
            || declaration
                .adapter_attestation_ref
                .as_deref()
                .is_none_or(str::is_empty)
            || !source_evidence_is_ledgered
        {
            return Err(m8_error(
                "artifact_reference_error",
                "strong settlement declaration is not qualifying",
            ));
        }
    }
    if token.transition_kind == TransitionKind::Capitalize {
        if token.value_evidence_refs.is_empty() {
            return Err(m8_error(
                "artifact_reference_error",
                "capitalization has no immutable value-evidence reference",
            ));
        }
        let approval_ref = token.approval_ref.as_deref().ok_or_else(|| {
            m8_error(
                "artifact_reference_error",
                "capitalization approval ref is missing",
            )
        })?;
        let approval: ApprovalRequest = serde_json::from_value(
            require_record(&facts, "approval_resolution", "approval_id", approval_ref)?.clone(),
        )
        .map_err(|error| {
            m8_error(
                "artifact_reference_error",
                format!("capitalization approval is invalid: {error}"),
            )
        })?;
        let resolved_before_expiry = approval
            .resolved_at
            .as_ref()
            .is_some_and(|resolved_at| resolved_at <= &approval.expires_at);
        if approval.status != ApprovalStatus::Approved
            || approval.case_id != token.case_id
            || approval.run_id != token.run_id
            || approval.plan_item_id != token.plan_item_id
            || approval.criteria_ref.as_deref() != Some(token.criteria_ref.as_str())
            || approval.criteria_sha256.as_deref() != Some(criteria_record.criteria_sha256.as_str())
            || approval.criteria_record_hash.as_deref()
                != Some(criteria_record.criteria_record_hash.as_str())
            || token.authority_decision_refs.len() != 1
            || token.authority_decision_refs.first() != Some(&approval.decision_id)
            || approval.resolved_by.as_deref() != token.approver_id.as_deref()
            || approval.resolved_by.as_deref() == Some(token.actor_id.as_str())
            || token.approver_id.as_deref() == Some(token.actor_id.as_str())
            || !resolved_before_expiry
        {
            return Err(m8_error(
                "artifact_reference_error",
                "capitalization approval is missing, rejected, or detached",
            ));
        }
        let rights_ref = token.rights_profile_ref.as_deref().ok_or_else(|| {
            m8_error(
                "artifact_reference_error",
                "capital rights profile is missing",
            )
        })?;
        let rights = require_record(
            &facts,
            "artifact_rights_profile",
            "rights_profile_id",
            rights_ref,
        )?;
        verify_embedded_hash(rights, "record_hash", "artifact_reference_error")?;
        let review_status = rights.get("review_status").and_then(Value::as_str);
        if rights.get("accepted").and_then(Value::as_bool) != Some(true)
            || rights.get("artifact_id").and_then(Value::as_str)
                != Some(result_registration.artifact_id.as_str())
            || rights.get("license").and_then(Value::as_str)
                != Some(result_registration.license.as_str())
            || !profile
                .required_rights_review
                .iter()
                .any(|required| Some(required.as_str()) == review_status)
        {
            return Err(m8_error(
                "artifact_reference_error",
                "rights profile does not match artifact license/review requirements",
            ));
        }
        // Capitalization rebuild resolves exact allowing action and record hash;
        // accepted bool alone is insufficient.
        let rights_decision_ref = rights
            .get("authority_decision_ref")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                m8_error(
                    "artifact_reference_error",
                    "rights profile has no authority decision ref",
                )
            })?;
        let rights_decision_hash = rights
            .get("authority_decision_hash")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                m8_error(
                    "artifact_reference_error",
                    "rights profile has no authority decision hash",
                )
            })?;
        let rights_decision: AuthorityDecision = serde_json::from_value(
            facts
                .by_kind
                .get("authority_decision")
                .into_iter()
                .flatten()
                .find(|decision| {
                    decision.get("decision_id").and_then(Value::as_str) == Some(rights_decision_ref)
                        && payload_hash(decision).ok().as_deref() == Some(rights_decision_hash)
                })
                .ok_or_else(|| {
                    m8_error(
                        "artifact_reference_error",
                        "rights profile authority decision is missing",
                    )
                })?
                .clone(),
        )
        .map_err(|error| {
            m8_error(
                "artifact_reference_error",
                format!("rights profile authority decision is invalid: {error}"),
            )
        })?;
        let expected_rights_action =
            canonical_rights_action(result_registration, &profile.required_rights_review);
        let rights_action_hash = hash_canonical(&expected_rights_action)?;
        if rights_decision.verdict != Verdict::Allow
            || rights.get("authority_action_hash").and_then(Value::as_str)
                != Some(rights_action_hash.as_str())
            || rights_decision.operation != expected_rights_action
            || rights_decision
                .action_request
                .evidence
                .get("payload_hash")
                .and_then(Value::as_str)
                != Some(rights_action_hash.as_str())
        {
            return Err(m8_error(
                "artifact_reference_error",
                "rights profile authority does not allow the exact rights review action",
            ));
        }
        for value_ref in &token.value_evidence_refs {
            let value = require_record(
                &facts,
                "artifact_value_evidence",
                "value_evidence_id",
                value_ref,
            )?;
            verify_embedded_hash(value, "record_hash", "artifact_reference_error")?;
            for evidence_ref in value
                .get("evidence_refs")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
            {
                let evidence = require_record(
                    &facts,
                    "artifact_value_source_evidence",
                    "evidence_id",
                    evidence_ref,
                )?;
                verify_embedded_hash(evidence, "record_hash", "artifact_reference_error")?;
                let source: ValueEvidenceSourceRecord = serde_json::from_value(evidence.clone())
                    .map_err(|error| {
                        m8_error(
                            "artifact_reference_error",
                            format!("value evidence source is invalid: {error}"),
                        )
                    })?;
                validate_value_evidence_source(&facts, &source, &token.case_id)?;
                if evidence.get("kind").and_then(Value::as_str)
                    != value.get("kind").and_then(Value::as_str)
                    || evidence.get("case_id").and_then(Value::as_str)
                        != value.get("case_id").and_then(Value::as_str)
                    || evidence.get("case_id").and_then(Value::as_str)
                        == Some(token.case_id.as_str())
                {
                    return Err(m8_error(
                        "artifact_reference_error",
                        "underlying value evidence is not accepted, linked, and outside-case",
                    ));
                }
            }
            if value.get("accepted").and_then(Value::as_bool) != Some(true)
                || value.get("case_id").and_then(Value::as_str) == Some(token.case_id.as_str())
                || value.get("kind").and_then(Value::as_str) == Some("quality")
                || profile
                    .qualifying_value_evidence_kinds
                    .iter()
                    .any(|kind| kind == "quality")
                || !profile
                    .qualifying_value_evidence_kinds
                    .iter()
                    .any(|kind| value.get("kind").and_then(Value::as_str) == Some(kind.as_str()))
            {
                return Err(m8_error(
                    "artifact_reference_error",
                    "value evidence is not accepted, qualifying, and outside-case",
                ));
            }
        }
    }
    Ok(())
}

pub fn store_gate_profile(
    root: &Path,
    profile: &ArtifactGateProfile,
) -> Result<String, ForgeError> {
    if profile.profile_ref.is_empty() {
        return Err(m8_error("artifact_gate_error", "empty profile ref"));
    }
    let hash = hash_canonical(profile)?;
    let path = gate_profile_path(root, &profile.profile_ref)?;
    fs::create_dir_all(path.parent().unwrap())
        .map_err(|e| ForgeError::io("create gate profiles", e))?;
    let bytes = serde_json::to_vec_pretty(profile)?;
    if path.exists()
        && fs::read(&path).map_err(|e| ForgeError::io("read gate profile", e))? != bytes
    {
        return Err(m8_error(
            "artifact_gate_error",
            "referenced profile is immutable",
        ));
    }
    if !path.exists() {
        fs::write(path, bytes).map_err(|e| ForgeError::io("write gate profile", e))?;
    }
    Ok(hash)
}

fn gate_profile_path(root: &Path, profile_ref: &str) -> Result<std::path::PathBuf, ForgeError> {
    if profile_ref.is_empty()
        || profile_ref.contains('/')
        || profile_ref.contains('\\')
        || profile_ref == "."
        || profile_ref == ".."
    {
        return Err(m8_error("artifact_gate_error", "invalid profile ref"));
    }
    Ok(root
        .join("artifacts/gate-profiles")
        .join(format!("{profile_ref}.json")))
}

/// Verify the immutable gate snapshot and ledger it once, before authorizing a
/// transition that depends on it.
pub fn resolve_gate_profile(
    root: &Path,
    actor: &str,
    profile_ref: &str,
    expected_hash: &str,
) -> Result<ArtifactGateProfile, ForgeError> {
    let path = gate_profile_path(root, profile_ref)?;
    let profile: ArtifactGateProfile = serde_json::from_slice(
        &fs::read(&path).map_err(|error| ForgeError::io("read gate profile", error))?,
    )?;
    if profile.profile_ref != profile_ref || hash_canonical(&profile)? != expected_hash {
        return Err(m8_error(
            "artifact_gate_error",
            "gate profile hash mismatch",
        ));
    }
    let stream = LedgerStream::open(root, "artifact-ip", actor)?;
    let already_committed = stream.read_entries()?.iter().any(|entry| {
        entry.record_kind == "artifact_gate_profile"
            && entry
                .payload
                .get("profile_ref")
                .and_then(|value| value.as_str())
                == Some(profile_ref)
    });
    if !already_committed {
        stream.commit_typed(
            "artifact_gate_profile",
            vec![profile_ref.into()],
            &profile,
            vec![],
        )?;
    }
    Ok(profile)
}

pub fn read_gate_profile(
    root: &Path,
    profile_ref: &str,
    expected_hash: &str,
) -> Result<ArtifactGateProfile, ForgeError> {
    let path = gate_profile_path(root, profile_ref)?;
    let profile: ArtifactGateProfile = serde_json::from_slice(
        &fs::read(&path).map_err(|error| ForgeError::io("read gate profile", error))?,
    )?;
    if profile.profile_ref != profile_ref || hash_canonical(&profile)? != expected_hash {
        return Err(m8_error(
            "artifact_gate_error",
            "gate profile hash mismatch",
        ));
    }
    Ok(profile)
}

pub struct AuthorizedAttestation {
    root: std::path::PathBuf,
    ledger_id: String,
    registration: ArtifactRegistrationRecord,
    degraded_mode: String,
    degraded_controls: Vec<String>,
    authority_decision_ref: String,
    authority_decision_hash: String,
    authority_action: AuthorityAction,
}

impl AuthorizedAttestation {
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn ledger_id(&self) -> &str {
        &self.ledger_id
    }
    pub fn registration(&self) -> &ArtifactRegistrationRecord {
        &self.registration
    }
    pub fn degraded_mode(&self) -> &str {
        &self.degraded_mode
    }
    pub fn degraded_controls(&self) -> &[String] {
        &self.degraded_controls
    }
}

#[allow(clippy::too_many_arguments)]
pub fn authorize_attestation(
    grant: ActionGrant,
    action: &AuthorityAction,
    root: &Path,
    registration: &ArtifactRegistrationRecord,
    ledger_id: &str,
    degraded_mode: &str,
    degraded_controls: Vec<String>,
    decision: &AuthorityDecision,
    committed: &CommittedRecordRef,
) -> Result<AuthorizedAttestation, ForgeError> {
    let exact = matches!(
        action,
        AuthorityAction::Reserved {
            resource_type,
            resource_id,
            parameters,
        } if resource_type == "attest_artifact_identity"
            && resource_id == &registration.artifact_id
            && parameters.get("artifact_id").and_then(Value::as_str)
                == Some(registration.artifact_id.as_str())
            && parameters.get("content_identity").and_then(Value::as_str)
                == Some(registration.content_identity.as_str())
            && parameters.get("descriptor_hash").and_then(Value::as_str)
                == Some(registration.descriptor_hash.as_str())
            && parameters.get("ledger").and_then(Value::as_str) == Some(ledger_id)
            && parameters.get("degraded_mode").and_then(Value::as_str)
                == Some(degraded_mode)
    );
    let decision_hash = payload_hash(&serde_json::to_value(decision)?)?;
    if !exact
        || decision.operation != *action
        || decision.verdict != Verdict::Allow
        || committed.record_kind() != "authority_decision"
        || committed.payload_hash() != decision_hash
    {
        return Err(m8_error(
            "artifact_attestation_error",
            "attestation action parameters do not match the artifact",
        ));
    }
    grant.authorize(action, "attestation_ingress", "attestation_ingress", root)?;
    Ok(AuthorizedAttestation {
        root: root.into(),
        ledger_id: ledger_id.into(),
        registration: registration.clone(),
        degraded_mode: degraded_mode.into(),
        degraded_controls,
        authority_decision_ref: decision.decision_id.clone(),
        authority_decision_hash: decision_hash,
        authority_action: action.clone(),
    })
}

pub trait ArtifactAttestor {
    fn attest(&self, authorization: AuthorizedAttestation) -> Result<String, ForgeError>;
}
pub struct DefaultArtifactAttestor;
impl ArtifactAttestor for DefaultArtifactAttestor {
    fn attest(&self, authorization: AuthorizedAttestation) -> Result<String, ForgeError> {
        let registration = authorization.registration();
        let ledger_id = authorization.ledger_id();
        let committed = LedgerStream::open(authorization.root(), ledger_id, "ifl")?.commit_typed("artifact_identity_attestation", vec![registration.artifact_id.clone()], &json!({"artifact_id": registration.artifact_id, "content_identity": registration.content_identity, "descriptor_hash": registration.descriptor_hash, "authority_decision_ref": authorization.authority_decision_ref, "authority_decision_hash": authorization.authority_decision_hash, "authority_action": authorization.authority_action, "authority_action_hash": hash_canonical(&authorization.authority_action)?}), vec![authorization.authority_decision_ref.clone()])?;
        let ordinal = serde_json::to_value(committed)?
            .get("append_ordinal")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| {
                ForgeError::Internal("ledger did not expose attestation sequence".into())
            })?;
        Ok(format!("ifl:token:{ledger_id}:{ordinal}"))
    }
}
#[derive(Clone, Copy)]
pub enum AttestationPolicy {
    Required,
    PreMintOnly,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Attestation {
    pub status: IdentityStatus,
    pub attestation_ref: String,
    pub degraded_controls: Vec<String>,
}
pub fn attest(
    attestor: &dyn ArtifactAttestor,
    authorization: AuthorizedAttestation,
    policy: AttestationPolicy,
) -> Result<Attestation, ForgeError> {
    let degraded_mode = authorization.degraded_mode.clone();
    let degraded_controls = authorization.degraded_controls.clone();
    if matches!(policy, AttestationPolicy::Required) && degraded_mode != "forbidden" {
        return Err(m8_error(
            "artifact_attestation_error",
            "required attestation must forbid degraded mode",
        ));
    }
    if matches!(policy, AttestationPolicy::PreMintOnly)
        && (degraded_mode != "pre_mint_only" || degraded_controls.is_empty())
    {
        return Err(m8_error(
            "artifact_attestation_error",
            "pre_mint_only requires explicit degraded controls",
        ));
    }
    match attestor.attest(authorization) {
        Ok(attestation_ref) => Ok(Attestation {
            status: IdentityStatus::Attested,
            attestation_ref,
            degraded_controls: vec![],
        }),
        Err(_error) if matches!(policy, AttestationPolicy::PreMintOnly) => Ok(Attestation {
            status: IdentityStatus::PreMint,
            attestation_ref: String::new(),
            degraded_controls,
        }),
        Err(error) => Err(error),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactState {
    pub artifact_id: String,
    pub recognized_stage: ArtifactStage,
    pub identity_status: IdentityStatus,
    pub lifecycle_status: LifecycleStatus,
}
pub fn rebuild(
    registrations: &[ArtifactRegistrationRecord],
    tokens: &[TransitionToken],
) -> Result<BTreeMap<String, ArtifactState>, ForgeError> {
    let registrations_by_id: BTreeMap<&str, &ArtifactRegistrationRecord> = registrations
        .iter()
        .map(|registration| (registration.artifact_id.as_str(), registration))
        .collect();
    if registrations_by_id.len() != registrations.len() {
        return Err(m8_error(
            "artifact_lineage_error",
            "duplicate artifact registration",
        ));
    }
    for registration in registrations {
        if content_identity(&registration.artifact_type, &registration.content_sha256)?
            != registration.content_identity
        {
            return Err(m8_error(
                "artifact_lineage_error",
                "registration content identity mismatch",
            ));
        }
        if registration.derived_from.is_empty() {
            if registration.lineage_id != registration.content_identity {
                return Err(m8_error(
                    "artifact_lineage_error",
                    "root registration lineage does not derive from content identity",
                ));
            }
        } else {
            let source_lineages = registration
                .derived_from
                .iter()
                .map(|source| {
                    registrations_by_id
                        .get(source.as_str())
                        .map(|source| source.lineage_id.as_str())
                        .ok_or_else(|| {
                            m8_error(
                                "artifact_reference_error",
                                format!("derived source registration {source} is missing"),
                            )
                        })
                })
                .collect::<Result<BTreeSet<_>, _>>()?;
            if source_lineages.len() != 1
                || source_lineages.first().copied() != Some(registration.lineage_id.as_str())
            {
                return Err(m8_error(
                    "artifact_lineage_error",
                    "derived registration does not inherit one common lineage",
                ));
            }
        }
    }
    let token_by_id: BTreeMap<&str, &TransitionToken> = tokens
        .iter()
        .map(|token| (token.transition_token_id.as_str(), token))
        .collect();
    if token_by_id.len() != tokens.len() {
        return Err(m8_error(
            "artifact_lineage_error",
            "duplicate transition token",
        ));
    }
    for token in tokens {
        let transition_input: TransitionInput =
            serde_json::from_value(serde_json::to_value(token)?).map_err(|error| {
                m8_error(
                    "artifact_lineage_error",
                    format!("transition token shape is invalid: {error}"),
                )
            })?;
        validate_transition_shape(&transition_input)?;
        let requires_parent = token.from_stage != ArtifactStage::Cognitive;
        if requires_parent == token.parent_transition_token_ids.is_empty() {
            return Err(m8_error(
                "artifact_lineage_error",
                "incomplete transition parents",
            ));
        }
        for parent_id in &token.parent_transition_token_ids {
            let parent = token_by_id
                .get(parent_id.as_str())
                .ok_or_else(|| m8_error("artifact_lineage_error", "missing parent transition"))?;
            if parent.to_stage != token.from_stage
                || !token
                    .source_artifact_ids
                    .contains(&parent.result_artifact_id)
            {
                return Err(m8_error(
                    "artifact_lineage_error",
                    "invalid parent transition",
                ));
            }
        }
        // Each source must have at least one accepted parent establishing its
        // from_stage; no omitted parent (§M8 multi-source invariant).
        if requires_parent {
            for source in &token.source_artifact_ids {
                let has_parent = token.parent_transition_token_ids.iter().any(|parent_id| {
                    token_by_id
                        .get(parent_id.as_str())
                        .is_some_and(|parent| parent.result_artifact_id == *source)
                });
                if !has_parent {
                    return Err(m8_error(
                        "artifact_lineage_error",
                        "non-cognitive source lacks an accepted parent token",
                    ));
                }
            }
        }
    }
    fn visit(
        id: &str,
        tokens: &BTreeMap<&str, &TransitionToken>,
        visiting: &mut BTreeSet<String>,
        visited: &mut BTreeSet<String>,
    ) -> Result<(), ForgeError> {
        if visited.contains(id) {
            return Ok(());
        }
        if !visiting.insert(id.into()) {
            return Err(m8_error("artifact_lineage_error", "cycle"));
        }
        for parent in &tokens[id].parent_transition_token_ids {
            visit(parent, tokens, visiting, visited)?;
        }
        visiting.remove(id);
        visited.insert(id.into());
        Ok(())
    }
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for id in token_by_id.keys() {
        visit(id, &token_by_id, &mut visiting, &mut visited)?;
    }
    let mut states: BTreeMap<String, ArtifactState> = registrations
        .iter()
        .map(|r| {
            (
                r.artifact_id.clone(),
                ArtifactState {
                    artifact_id: r.artifact_id.clone(),
                    recognized_stage: ArtifactStage::Cognitive,
                    identity_status: IdentityStatus::PreMint,
                    lifecycle_status: LifecycleStatus::Active,
                },
            )
        })
        .collect();
    let mut seen_results = BTreeSet::new();
    let mut remaining: BTreeMap<&str, &TransitionToken> = token_by_id;
    while !remaining.is_empty() {
        let ready: Vec<&str> = remaining
            .iter()
            .filter_map(|(id, token)| {
                token
                    .parent_transition_token_ids
                    .iter()
                    .all(|parent| !remaining.contains_key(parent.as_str()))
                    .then_some(*id)
            })
            .collect();
        if ready.is_empty() {
            return Err(m8_error(
                "artifact_lineage_error",
                "incomplete transition chain",
            ));
        }
        for id in ready {
            let token = remaining.remove(id).ok_or_else(|| {
                ForgeError::Internal("artifact_lineage_error: ready token disappeared".into())
            })?;
            if token.transition_hash != token_hash(token)? {
                return Err(m8_error(
                    "artifact_lineage_error",
                    "tampered transition token",
                ));
            }
            let expected_edge = match token.transition_kind {
                TransitionKind::Synthesize => {
                    (ArtifactStage::Cognitive, ArtifactStage::Intellectual)
                }
                TransitionKind::Productize => (ArtifactStage::Intellectual, ArtifactStage::Product),
                TransitionKind::Capitalize => (ArtifactStage::Product, ArtifactStage::Capital),
            };
            if (token.from_stage.clone(), token.to_stage.clone()) != expected_edge {
                return Err(m8_error(
                    "artifact_lineage_error",
                    "transition has an illegal maturity edge",
                ));
            }
            if !seen_results.insert(format!("{}:{:?}", token.result_artifact_id, token.to_stage)) {
                return Err(m8_error(
                    "artifact_lineage_error",
                    "conflicting transition fork",
                ));
            }
            for (index, source) in token.source_artifact_ids.iter().enumerate() {
                let state = states.get(source).ok_or_else(|| {
                    m8_error("artifact_reference_error", "missing source registration")
                })?;
                let source_registration = registrations_by_id[source.as_str()];
                if token.input_content_identities.get(index)
                    != Some(&source_registration.content_identity)
                {
                    return Err(m8_error(
                        "artifact_lineage_error",
                        "transition input identity does not match source registration",
                    ));
                }
                if state.recognized_stage != token.from_stage {
                    return Err(m8_error(
                        "artifact_lineage_error",
                        "incomplete or stale transition chain",
                    ));
                }
            }
            let result_registration = registrations_by_id
                .get(token.result_artifact_id.as_str())
                .ok_or_else(|| {
                    m8_error("artifact_reference_error", "missing result registration")
                })?;
            let source_registration = token
                .source_artifact_ids
                .first()
                .and_then(|source| registrations_by_id.get(source.as_str()))
                .ok_or_else(|| {
                    m8_error(
                        "artifact_reference_error",
                        "transition has no registered source",
                    )
                })?;
            if result_registration.content_identity != token.output_content_identity
                || (token.mode == TransitionMode::Derive
                    && (result_registration.derived_from != token.derived_from
                        || result_registration.lineage_id != source_registration.lineage_id))
            {
                return Err(m8_error(
                    "artifact_lineage_error",
                    "transition result identity or lineage does not match registration",
                ));
            }
            let result = states
                .get_mut(&token.result_artifact_id)
                .ok_or_else(|| m8_error("artifact_reference_error", "missing result state"))?;
            result.recognized_stage = token.to_stage.clone();
            result.identity_status = token.identity_status_after.clone();
        }
    }
    Ok(states)
}

pub fn rebuild_with_lifecycle(
    registrations: &[ArtifactRegistrationRecord],
    tokens: &[TransitionToken],
    lifecycle: &[ArtifactLifecycleRecord],
) -> Result<BTreeMap<String, ArtifactState>, ForgeError> {
    let mut states = rebuild(registrations, tokens)?;
    for record in lifecycle {
        let state = states.get_mut(&record.artifact_id).ok_or_else(|| {
            m8_error(
                "artifact_lifecycle_error",
                "lifecycle record references an unknown artifact",
            )
        })?;
        if record.version != M8_RECORD_VERSION
            || record.previous_status == record.new_status
            || record.reason.trim().is_empty()
            || record.changed_at.trim().is_empty()
            || state.lifecycle_status != record.previous_status
        {
            return Err(m8_error(
                "artifact_lifecycle_error",
                "lifecycle record does not follow rebuilt append-order status",
            ));
        }
        state.lifecycle_status = record.new_status.clone();
    }
    Ok(states)
}

/// Rebuild every M8 materialized view from committed ledger payloads. Existing
/// files are overwritten deliberately; they never participate in reconstruction.
pub fn rebuild_materialized_views(
    root: &Path,
    actor: &str,
) -> Result<BTreeMap<String, ArtifactState>, ForgeError> {
    let stream = LedgerStream::open(root, "artifact-ip", actor)?;
    stream.verify()?;
    let entries = stream.read_entries()?;
    let mut registrations: Vec<ArtifactRegistrationRecord> = Vec::new();
    let mut tokens: Vec<TransitionToken> = Vec::new();
    let mut lifecycle: Vec<ArtifactLifecycleRecord> = Vec::new();
    for entry in entries {
        match entry.record_kind.as_str() {
            "artifact_registration" => registrations.push(serde_json::from_value(entry.payload)?),
            "artifact_transition" => tokens.push(serde_json::from_value(entry.payload)?),
            "artifact_lifecycle" => lifecycle.push(serde_json::from_value(entry.payload)?),
            _ => {}
        }
    }
    let facts = verified_ledger_facts(root)?;
    for token in &tokens {
        let profile_value = require_record(
            &facts,
            "artifact_gate_profile",
            "profile_ref",
            &token.gate_profile_ref,
        )?;
        let profile: ArtifactGateProfile = serde_json::from_value(profile_value.clone())?;
        validate_ledger_references(root, actor, &registrations, &tokens, token, &profile)?;
    }
    for record in &lifecycle {
        validate_lifecycle_authority(&facts, record)?;
    }
    let mut states = rebuild_with_lifecycle(&registrations, &tokens, &lifecycle)?;
    for attestation in facts.attestations.values() {
        validate_attestation_authority(&facts, attestation)?;
        let artifact_id = attestation.get("artifact_id").and_then(Value::as_str);
        let matches = artifact_id
            .and_then(|id| registrations.iter().find(|r| r.artifact_id == id))
            .is_some_and(|registration| {
                attestation.get("content_identity").and_then(Value::as_str)
                    == Some(registration.content_identity.as_str())
                    && attestation.get("descriptor_hash").and_then(Value::as_str)
                        == Some(registration.descriptor_hash.as_str())
            });
        if !matches {
            return Err(m8_error(
                "artifact_attestation_error",
                "attestation does not match the registered artifact identity",
            ));
        }
        if let Some(state) = artifact_id.and_then(|id| states.get_mut(id)) {
            state.identity_status = IdentityStatus::Attested;
        }
    }
    let mut catalog = Vec::new();
    for state in states.values() {
        serde_json::to_writer(&mut catalog, state)?;
        catalog.push(b'\n');
    }
    let catalog_path = root.join("artifacts/catalog.jsonl");
    let parent = catalog_path
        .parent()
        .ok_or_else(|| ForgeError::Internal("catalog path lacks parent".into()))?;
    fs::create_dir_all(parent)
        .map_err(|error| ForgeError::io("create catalog directory", error))?;
    fs::write(&catalog_path, catalog).map_err(|error| ForgeError::io("write catalog", error))?;
    let stale_state = root.join("artifacts/state.json");
    if stale_state.exists() {
        fs::remove_file(&stale_state)
            .map_err(|error| ForgeError::io("remove stale artifact state", error))?;
    }
    let capital_dir = root.join("ip/capital");
    if capital_dir.exists() {
        fs::remove_dir_all(&capital_dir)
            .map_err(|error| ForgeError::io("clear capital projection", error))?;
    }
    for state in states
        .values()
        .filter(|state| state.recognized_stage == ArtifactStage::Capital)
    {
        let path = root
            .join("ip/capital")
            .join(format!("{}.json", state.artifact_id));
        let parent = path
            .parent()
            .ok_or_else(|| ForgeError::Internal("capital path lacks parent".into()))?;
        fs::create_dir_all(parent)
            .map_err(|error| ForgeError::io("create capital directory", error))?;
        fs::write(path, serde_json::to_vec_pretty(state)?)
            .map_err(|error| ForgeError::io("write capital projection", error))?;
    }
    Ok(states)
}

pub fn load_ledger_records(
    root: &Path,
    actor: &str,
) -> Result<(Vec<ArtifactRegistrationRecord>, Vec<TransitionToken>), ForgeError> {
    let stream = LedgerStream::open(root, "artifact-ip", actor)?;
    stream.verify()?;
    let mut registrations = Vec::new();
    let mut tokens = Vec::new();
    for entry in stream.read_entries()? {
        match entry.record_kind.as_str() {
            "artifact_registration" => registrations.push(serde_json::from_value(entry.payload)?),
            "artifact_transition" => tokens.push(serde_json::from_value(entry.payload)?),
            _ => {}
        }
    }
    Ok((registrations, tokens))
}
