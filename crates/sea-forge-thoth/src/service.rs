//! The one real, joined Thoth service (spec-adlc-thoth §7.4, §10.3, M11
//! service, T14A/T14B). Joins the Task 11 self-model snapshot with real
//! ledger-verified capability, projection, and probe records; derives a
//! `DisclosurePolicy` from the authority `self_disclosure` surface's matched
//! grant; enforces the grant's freshness requirement before any bounded
//! query; and commits the complete question-to-answer evidence chain to one
//! acting ledger stream. CLI and server (T14B) call `ask` — neither ingress
//! constructs its own `SnapshotView` or loads policy directly.

use crate::engine::{
    answer_question, CapabilityInfo, DisclosurePolicy, EnvironmentInfo, RecordedAuthorityDecision,
    SnapshotView,
};
use crate::protocol::*;
use sea_forge_authority::AuthorityPolicyBundle;
use sea_forge_capability::promotion::{
    build_capability_record, default_v02_policy, load_declarations, load_envelopes,
};
use sea_forge_core::errors::ForgeError;
use sea_forge_core::ids::random_id;
use sea_forge_core::types::CapabilityPromotionPolicy;
use sea_forge_ledger::LedgerStream;
use sea_forge_self_model::{
    bundled, load_composed, store, CellRealization, ComposedModel, ProbeResult,
};
use std::path::{Path, PathBuf};

/// Ledger stream committing every question/decision/disclosure/answer this
/// service produces (§10.3, T14A step 4). One stream per installation root,
/// mirroring the `self-model`/`memory-recalls` stream-per-concern precedent.
const ASK_LEDGER_STREAM: &str = "thoth-asks";

/// Purpose length ceiling (§7.4: "≤ 500 chars").
const MAX_PURPOSE_LEN: usize = 500;

fn policy_path(root: &Path) -> PathBuf {
    root.join("authority").join("active-policy.json")
}

fn map_capability_status(status: &sea_forge_core::types::CapabilityStatus) -> ClaimStatus {
    use sea_forge_core::types::CapabilityStatus::*;
    match status {
        // Proven/Metabolized are the policy-thresholded, fully-qualified
        // rungs — the only statuses honest enough to report the ladder's top
        // rung. Demonstrated (capability-promotion sense: at least one
        // qualifying declaration, thresholds not fully met) maps one rung
        // below so a partially-evidenced capability is never conflated with
        // a policy-proven one. Attempted (no qualifying evidence at all)
        // maps to the floor "declared" rung.
        Proven | Metabolized => ClaimStatus::Demonstrated,
        Demonstrated => ClaimStatus::Validated,
        Attempted => ClaimStatus::Declared,
    }
}

/// Real, ledger-joined `SnapshotView`: no caller-supplied status anywhere.
/// Capability truth is rebuilt from ledgered declarations/envelopes; concept
/// existence comes from the verified composed model; environment truth comes
/// from the cell realization's evidenced toolchain probes (T14A step 2).
pub struct LedgerSnapshotView {
    root: PathBuf,
    snapshot_id: String,
    stale: bool,
    composed: ComposedModel,
    cell: Option<CellRealization>,
    policy: CapabilityPromotionPolicy,
}

impl LedgerSnapshotView {
    /// Open the view over `root`'s current, re-verified self-model snapshot.
    /// Fails if no snapshot has ever been built — there is no synthesized
    /// fallback status.
    pub fn open(root: &Path) -> Result<Self, ForgeError> {
        store::validate(root)?;
        let snapshot = store::current_snapshot(root)?.ok_or_else(|| {
            ForgeError::SelfModel(
                "no self-model snapshot; run 'sea-forge self-model rebuild'".into(),
            )
        })?;
        let stale = store::is_stale(root)?;
        let composed = load_composed(&bundled())?;
        let cell = store::read_cell_realization(root)?;
        Ok(Self {
            root: root.to_path_buf(),
            snapshot_id: snapshot.snapshot_id,
            stale,
            composed,
            cell,
            policy: default_v02_policy(),
        })
    }
}

impl SnapshotView for LedgerSnapshotView {
    fn snapshot_ref(&self) -> &str {
        &self.snapshot_id
    }

    fn is_stale(&self) -> bool {
        self.stale
    }

    fn capability(&self, name: &str) -> Option<CapabilityInfo> {
        if !self.composed.concept_exists(name) {
            return None;
        }
        // Pure rebuild over ledgered compatibility views — unlike
        // `rebuild_capability`, this never writes a materialized record as a
        // side effect of a read-only disclosure query, and tolerates an
        // installation that has never attempted this capability (no
        // `capabilities.jsonl` yet).
        // F-12: the passed root is the state root; capabilities join directly.
        let envelopes_path = self.root.join("capabilities.jsonl");
        let envelopes = if envelopes_path.exists() {
            load_envelopes(&envelopes_path).ok()?
        } else {
            vec![]
        };
        let declarations =
            load_declarations(&self.root.join("settlement/declarations.jsonl")).ok()?;
        let record = build_capability_record(
            name,
            &envelopes,
            &declarations,
            &self.policy,
            &chrono::Utc::now().to_rfc3339(),
        )
        .ok()?;
        // Evidence comes from whichever ledgered source actually backs the
        // status: envelope run IDs (attempt history) and, since a capability
        // can be qualifying-demonstrated from declarations alone with zero
        // envelopes, the qualifying declarations' own evidence refs too.
        let mut evidence_refs: Vec<String> = record
            .evidence_sample
            .iter()
            .map(|e| e.run_id.clone())
            .collect();
        let mut settlement_refs: Vec<String> = record
            .evidence_sample
            .iter()
            .filter_map(|e| e.declaration_id.clone())
            .collect();
        for decl in declarations.iter().filter(|d| d.plan_item_id == name) {
            evidence_refs.extend(decl.verification_evidence_refs.iter().cloned());
            evidence_refs.extend(decl.source_evidence_refs.iter().cloned());
            settlement_refs.push(decl.declaration_id.clone());
        }
        evidence_refs.sort();
        evidence_refs.dedup();
        settlement_refs.sort();
        settlement_refs.dedup();
        Some(CapabilityInfo {
            name: name.to_string(),
            status: map_capability_status(&record.status),
            flags: Default::default(),
            evidence_refs,
            settlement_refs,
            // No materialized capability record is committed by this
            // read-only rebuild (see comment above), so there is no real
            // ledger ref to cite yet — absence here is honest, not a
            // placeholder.
            capability_record_ref: None,
            limitations: record.contraction_reasons.clone(),
        })
    }

    fn declared_capabilities(&self) -> Vec<String> {
        self.composed.concepts()
    }

    fn environment_status(&self, tool: &str) -> Option<EnvironmentInfo> {
        let probe = self
            .cell
            .as_ref()?
            .toolchain_probes
            .iter()
            .find(|p| p.tool == tool)?;
        Some(EnvironmentInfo {
            tool: tool.to_string(),
            available: probe.result == ProbeResult::Available,
            probe_evidence_ref: probe.evidence_ref.clone(),
        })
    }

    fn recorded_authority_decision(
        &self,
        authority_decision_ref: &str,
    ) -> Option<RecordedAuthorityDecision> {
        let stream = LedgerStream::open(&self.root, ASK_LEDGER_STREAM, "thoth-service").ok()?;
        stream.verify().ok()?;
        stream.read_entries().ok()?.into_iter().find_map(|entry| {
            if entry.record_kind != "self_disclosure_decision" {
                return None;
            }
            let decision: RecordedAuthorityDecision = serde_json::from_value(entry.payload).ok()?;
            (decision.authority_decision_ref == authority_decision_ref).then_some(decision)
        })
    }
}

/// `DisclosurePolicy` derived from the authority bundle's `self_disclosure`
/// surface, bound to one actor for the lifetime of one `ask` call. Absent
/// policy file ⇒ an empty (deny-all) surface, matching the surface's own
/// documented deny-by-default absence rule (§8.2).
/// The grant keys one asker may match: the raw actor id (a grant may be
/// authored against a principal directly) plus the wire spelling of every role
/// the authority bundle binds to that principal. Grants are keyed by *role*
/// (`SelfDisclosureGrant.actor_role`); matching them against an actor id was
/// exactly the F-23 over-denial — a bundle binding `operator_local` to
/// `operator` authorized nothing, because `"operator_local" != "operator"`, so
/// lawfully granted disclosures came back denied.
fn grant_keys_for(bundle: &AuthorityPolicyBundle, actor_id: &str) -> Vec<String> {
    let mut keys = vec![actor_id.to_string()];
    for binding in &bundle.identity_bindings {
        if binding.principal == actor_id {
            // The same spelling the server identity gate reports (`operator`,
            // `R-SO`, …): serde's serialization of the typed enum, so the key
            // vocabulary stays defined once, by the enum's own attributes.
            if let Some(wire) = serde_json::to_value(&binding.role)
                .ok()
                .and_then(|value| value.as_str().map(str::to_owned))
            {
                if !keys.contains(&wire) {
                    keys.push(wire);
                }
            }
        }
    }
    keys
}

struct SurfacePolicy {
    surface: sea_forge_authority::SelfDisclosureSurface,
    /// Every key this asker may match a grant on (see [`grant_keys_for`]).
    grant_keys: Vec<String>,
}

impl DisclosurePolicy for SurfacePolicy {
    fn permits(&self, _actor_id: &str, class: &ClaimClass) -> bool {
        let class = claim_class_to_surface_str(class);
        self.grant_keys
            .iter()
            .any(|key| self.surface.permits(key, class))
    }

    fn requires_fresh(&self) -> bool {
        // Conservative: if any grant this actor holds demands a fresh
        // snapshot, the question requires one. A per-class split would need
        // the trait to carry the class into `requires_fresh`, which would
        // change `answer_question`'s established single-gate shape; failing
        // toward "requires fresh" is always safe, never a wider disclosure.
        self.surface.grants.iter().any(|g| {
            self.grant_keys.contains(&g.actor_role) && g.require_fresh_snapshot == Some(true)
        })
    }
}

fn load_surface_policy(root: &Path, actor_id: &str) -> Result<SurfacePolicy, ForgeError> {
    let path = policy_path(root);
    let (surface, grant_keys) = if path.exists() {
        let bundle = AuthorityPolicyBundle::load(&path)?;
        let keys = grant_keys_for(&bundle, actor_id);
        (bundle.policy_surfaces.self_disclosure, keys)
    } else {
        (
            sea_forge_authority::SelfDisclosureSurface::default(),
            vec![actor_id.to_string()],
        )
    };
    Ok(SurfacePolicy {
        surface,
        grant_keys,
    })
}

fn reason_code_for(disposition: &Disposition) -> &'static str {
    match disposition {
        Disposition::Denied => "claim_class_not_granted",
        Disposition::Partial => "partial_grant_some_classes_denied",
        Disposition::Answered => "fully_granted",
    }
}

/// The one mediated Thoth ask entrypoint. Validates inputs, opens the real
/// snapshot view, derives the disclosure policy, resolves the answer, and
/// commits the full question → disclosure-plan → decision → answer chain
/// before returning (§10.3, T14A). CLI and server adapters (T14B) call this
/// and do nothing else governance-related.
pub fn ask(
    root: &Path,
    actor_id: &str,
    kind: QuestionKind,
    subject: &str,
    purpose: &str,
    case_id: Option<&str>,
) -> Result<ThothAnswer, ForgeError> {
    if actor_id.trim().is_empty() {
        return Err(ForgeError::Input(
            "ask requires a non-empty actor identity".into(),
        ));
    }
    if subject.trim().is_empty() {
        return Err(ForgeError::Input(
            "ask requires a non-empty typed subject".into(),
        ));
    }
    if purpose.len() > MAX_PURPOSE_LEN {
        return Err(ForgeError::Input(format!(
            "purpose exceeds {MAX_PURPOSE_LEN} chars"
        )));
    }
    if case_id.is_some_and(|c| c.trim().is_empty()) {
        return Err(ForgeError::Input(
            "case_id, when given, must be non-empty (read-scope reference)".into(),
        ));
    }

    let view = LedgerSnapshotView::open(root)?;
    let policy = load_surface_policy(root, actor_id)?;

    let question_id = random_id("thq")?;
    let asked_at = chrono::Utc::now().to_rfc3339();
    let question = ThothQuestion {
        question_id: question_id.clone(),
        kind: kind.clone(),
        subject: subject.to_string(),
        actor_id: actor_id.to_string(),
        process_id: "thoth-service".into(),
        case_id: case_id.map(str::to_owned),
        purpose: purpose.to_string(),
        asked_at,
    };

    let stream = LedgerStream::open(root, ASK_LEDGER_STREAM, actor_id)?;
    let question_committed = stream.commit_typed(
        "self_disclosure_question",
        vec![question_id.clone()],
        &question,
        vec![],
    )?;

    let answer_id = random_id("tha")?;
    let answered_at = chrono::Utc::now().to_rfc3339();
    let answer = answer_question(&question, &view, &policy, &answer_id, &answered_at);

    let candidates = candidate_classes(&kind);
    let permitted: Vec<ClaimClass> = candidates
        .iter()
        .filter(|c| !answer.omitted_claim_classes.contains(c))
        .cloned()
        .collect();
    let plan = DisclosurePlan {
        question_id: question_id.clone(),
        authority_decision_ref: answer_id.clone(),
        permitted_claim_classes: permitted,
        permitted_regions: vec![],
        omitted_claim_classes: answer.omitted_claim_classes.clone(),
    };
    let plan_committed = stream.commit_typed(
        "self_disclosure_plan",
        vec![question_id.clone()],
        &plan,
        vec![question_committed.entry_ulid().to_string()],
    )?;

    let decision = RecordedAuthorityDecision {
        authority_decision_ref: answer_id.clone(),
        denied_classes: answer.omitted_claim_classes.clone(),
        reason_code: reason_code_for(&answer.disposition).to_string(),
    };
    let decision_committed = stream.commit_typed(
        "self_disclosure_decision",
        vec![answer_id.clone()],
        &decision,
        vec![plan_committed.entry_ulid().to_string()],
    )?;
    stream.commit_typed(
        "self_disclosure_answer",
        vec![answer_id.clone()],
        &answer,
        vec![decision_committed.entry_ulid().to_string()],
    )?;

    Ok(answer)
}
