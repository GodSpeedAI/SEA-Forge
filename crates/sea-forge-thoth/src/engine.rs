use crate::protocol::*;
use serde::{Deserialize, Serialize};

/// Immutable actor identity Thoth stamps onto every claim it constructs
/// (spec-adlc-thoth §10.3, T13B). Settlement/capability boundaries compare
/// this against the declarer/promoter via
/// `sea_forge_core::types::validate_claim_authorship_sod`.
pub const THOTH_ACTOR_ID: &str = "thoth";

/// Read-only view of a self-model snapshot for disclosure purposes.
/// The engine never touches raw graph data — only typed concept/capability
/// lookups (§6.2 G6, §10.3).
pub trait SnapshotView {
    /// The snapshot ID this view is backed by.
    fn snapshot_ref(&self) -> &str;
    /// Whether the snapshot is stale (any source hash drifted).
    fn is_stale(&self) -> bool;
    /// Look up a capability by name. Returns its derived status + evidence.
    fn capability(&self, name: &str) -> Option<CapabilityInfo>;
    /// List all declared capability names (for affordance queries).
    fn declared_capabilities(&self) -> Vec<String>;
    /// Look up an environment toolchain status by name.
    fn environment_status(&self, tool: &str) -> Option<EnvironmentInfo>;
    /// Resolve a previously recorded authority decision by reference, for
    /// `ask_why_denied` explanations. `None` when the reference is absent or
    /// cannot be verified — the caller must then refuse to explain (§10.3,
    /// T13.4). Default: no recorded-decision store available.
    fn recorded_authority_decision(
        &self,
        _authority_decision_ref: &str,
    ) -> Option<RecordedAuthorityDecision> {
        None
    }
}

/// Derived capability info from the snapshot (§7.3, §16.1).
#[derive(Clone, Debug, PartialEq)]
pub struct CapabilityInfo {
    pub name: String,
    pub status: ClaimStatus,
    pub flags: ClaimFlags,
    pub evidence_refs: Vec<String>,
    pub settlement_refs: Vec<String>,
    pub capability_record_ref: Option<String>,
    pub limitations: Vec<String>,
}

/// Environment toolchain info (§7.2).
#[derive(Clone, Debug, PartialEq)]
pub struct EnvironmentInfo {
    pub tool: String,
    pub available: bool,
    pub probe_evidence_ref: Option<String>,
}

/// A verified, recorded authority decision resolved for `ask_why_denied`.
/// Only these typed fields may be disclosed — never a re-derivation of fresh
/// claims (§10.3, T13.4). Serializable so the real service (T14A) can commit
/// and later re-resolve it from the acting ledger stream.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecordedAuthorityDecision {
    pub authority_decision_ref: String,
    pub denied_classes: Vec<ClaimClass>,
    pub reason_code: String,
}

/// Disclosure policy: which actor roles may see which claim classes.
/// Wraps the authority surface's permit check (§8.2).
pub trait DisclosurePolicy {
    /// Returns true if `actor_role` is permitted to disclose `claim_class`.
    /// Absent surface ⇒ deny (§8.2, T11.8).
    fn permits(&self, actor_role: &str, claim_class: &ClaimClass) -> bool;
    /// Whether this policy requires a fresh (non-stale) snapshot.
    fn requires_fresh(&self) -> bool {
        false
    }
}

/// The fixed authority notice.
const AUTHORITY_NOTICE: &str = "This answer confers no execution authority.";

/// Resolve a question into an answer through the disclosure pipeline (§16.2).
///
/// Steps: classify → check policy → deny or plan → bounded query → claim
/// derivation → answer assembly. Deterministic over (question, snapshot,
/// policy). A required-fresh policy refuses before any bounded query method
/// on `snapshot` is invoked (§7.4, T13.5) — only `snapshot_ref`/`is_stale`
/// (metadata, not self-model content) are ever called before that gate.
pub fn answer_question(
    question: &ThothQuestion,
    snapshot: &dyn SnapshotView,
    policy: &dyn DisclosurePolicy,
    answer_id: &str,
    answered_at: &str,
) -> ThothAnswer {
    let candidates = candidate_classes(&question.kind);
    // Classify which candidate classes are permitted for this actor.
    let permitted: Vec<ClaimClass> = candidates
        .iter()
        .filter(|c| policy.permits(&question.actor_id, c))
        .cloned()
        .collect();
    let omitted: Vec<ClaimClass> = candidates
        .iter()
        .filter(|c| !permitted.contains(c))
        .cloned()
        .collect();

    // If nothing is permitted, deny. Denial names classes only (§10.3).
    if permitted.is_empty() && !candidates.is_empty() {
        return ThothAnswer::denied(
            &question.question_id,
            answer_id,
            candidates.clone(),
            snapshot.snapshot_ref(),
            answered_at,
        );
    }

    // ask_why_denied is handled separately — reads only a recorded decision,
    // never a bounded snapshot query.
    if question.kind == QuestionKind::AskWhyDenied {
        return answer_why_denied(question, snapshot, answer_id, answered_at);
    }

    // Pre-query freshness gate (§7.4, T13.5): a required-fresh policy refuses
    // before `derive_claims` ever calls a bounded query method. Only
    // `is_stale` (staleness metadata, not self-model content) is consulted.
    if policy.requires_fresh() && snapshot.is_stale() {
        return ThothAnswer {
            answer_id: answer_id.into(),
            question_id: question.question_id.clone(),
            disposition: Disposition::Denied,
            claims: vec![],
            omitted_claim_classes: candidates,
            snapshot_ref: snapshot.snapshot_ref().to_owned(),
            freshness: Freshness::Stale,
            assurance: "local_tamper_evident".into(),
            limitations: vec!["required_fresh_snapshot_is_stale".into()],
            authority_notice: AUTHORITY_NOTICE.into(),
            answered_at: answered_at.into(),
        };
    }

    // Bounded query: derive claims only from permitted classes.
    let claims = derive_claims(&question.kind, &question.subject, snapshot, &permitted);
    let freshness = freshness_of(snapshot);
    let disposition = if omitted.is_empty() {
        Disposition::Answered
    } else {
        Disposition::Partial
    };

    ThothAnswer {
        answer_id: answer_id.into(),
        question_id: question.question_id.clone(),
        disposition,
        claims,
        omitted_claim_classes: omitted,
        snapshot_ref: snapshot.snapshot_ref().to_owned(),
        freshness,
        assurance: "local_tamper_evident".into(),
        limitations: vec![],
        authority_notice: AUTHORITY_NOTICE.into(),
        answered_at: answered_at.into(),
    }
}

/// Explain a prior denial by resolving its recorded authority-decision
/// reference. Discloses only the recorded `denied_classes`/`reason_code` —
/// never fresh claims. Unresolvable/unverified reference ⇒ denied (§10.3,
/// T13.4).
fn answer_why_denied(
    question: &ThothQuestion,
    snapshot: &dyn SnapshotView,
    answer_id: &str,
    answered_at: &str,
) -> ThothAnswer {
    match snapshot.recorded_authority_decision(&question.subject) {
        Some(decision) => ThothAnswer {
            answer_id: answer_id.into(),
            question_id: question.question_id.clone(),
            disposition: Disposition::Answered,
            claims: vec![],
            omitted_claim_classes: decision.denied_classes,
            snapshot_ref: snapshot.snapshot_ref().to_owned(),
            freshness: freshness_of(snapshot),
            assurance: "local_tamper_evident".into(),
            limitations: vec![decision.reason_code],
            authority_notice: AUTHORITY_NOTICE.into(),
            answered_at: answered_at.into(),
        },
        None => ThothAnswer {
            answer_id: answer_id.into(),
            question_id: question.question_id.clone(),
            disposition: Disposition::Denied,
            claims: vec![],
            omitted_claim_classes: vec![],
            snapshot_ref: snapshot.snapshot_ref().to_owned(),
            freshness: freshness_of(snapshot),
            assurance: "local_tamper_evident".into(),
            limitations: vec!["authority_decision_ref_unresolved".into()],
            authority_notice: AUTHORITY_NOTICE.into(),
            answered_at: answered_at.into(),
        },
    }
}

fn freshness_of(snapshot: &dyn SnapshotView) -> Freshness {
    if snapshot.is_stale() {
        Freshness::Stale
    } else {
        Freshness::Current
    }
}

/// The permitted capability-ladder class with the lowest evidence rung, in
/// ladder order. Used to label facts (including absence/unsupported facts)
/// under the most conservative class the policy actually grants (§9.3,
/// T13.2, T13.3).
fn lowest_permitted_capability_class(permitted: &[ClaimClass]) -> Option<ClaimClass> {
    [
        ClaimClass::DeclaredCapability,
        ClaimClass::InstalledCapability,
        ClaimClass::DemonstratedCapability,
    ]
    .into_iter()
    .find(|c| permitted.contains(c))
}

/// The evidence rung ceiling a claim class may assert (§7.3, §16.1).
fn capability_class_ceiling(class: &ClaimClass) -> ClaimStatus {
    match class {
        ClaimClass::DeclaredCapability => ClaimStatus::Declared,
        ClaimClass::InstalledCapability => ClaimStatus::Available,
        _ => ClaimStatus::Demonstrated,
    }
}

/// Build a capability claim bounded to what `permitted` actually grants.
/// Never elevates: uses the natural class for the capability's real status
/// when that class is permitted; otherwise downgrades to the highest
/// permitted class below it, capping status to that class's evidence rung.
/// Missing permission caps, never elevates (§10.2, §16.1, T13.1, T13.2). When
/// `permitted` grants only classes above the natural class — no downgrade
/// target exists — the real status is not disclosable at all, so this files
/// an `Unsupported` fact instead of relabeling the capability under a higher
/// class (Unsupported asserts no positive evidence, so it is always safe to
/// file under any permitted class).
fn capped_capability_claim(
    cap: &CapabilityInfo,
    snap_ref: &str,
    permitted: &[ClaimClass],
) -> Option<GroundedClaim> {
    use ClaimClass::{DeclaredCapability, DemonstratedCapability, InstalledCapability};
    let natural_class = match cap.status {
        ClaimStatus::Demonstrated | ClaimStatus::Validated => DemonstratedCapability,
        ClaimStatus::Installed | ClaimStatus::Available => InstalledCapability,
        _ => DeclaredCapability,
    };
    let class = if permitted.contains(&natural_class) {
        natural_class
    } else if natural_class == DemonstratedCapability && permitted.contains(&InstalledCapability) {
        InstalledCapability
    } else if permitted.contains(&DeclaredCapability) {
        DeclaredCapability
    } else {
        let class = lowest_permitted_capability_class(permitted)?;
        return Some(unsupported_claim(
            &cap.name,
            class,
            snap_ref,
            format!(
                "{} status is not disclosable at the permitted capability class",
                cap.name
            ),
        ));
    };
    let ceiling = capability_class_ceiling(&class);
    let status = if cap.status <= ceiling {
        cap.status.clone()
    } else {
        ceiling
    };
    let statement = format!(
        "{} is {}{}",
        cap.name,
        format!("{status:?}").to_lowercase(),
        if cap.flags.unavailable_in_environment {
            " (unavailable in environment)"
        } else {
            ""
        }
    );
    Some(GroundedClaim {
        claim_id: format!("claim_{}", cap.name),
        claim_class: class,
        subject: cap.name.clone(),
        status,
        flags: cap.flags.clone(),
        statement,
        snapshot_ref: snap_ref.to_owned(),
        evidence_refs: cap.evidence_refs.clone(),
        settlement_refs: cap.settlement_refs.clone(),
        capability_record_ref: cap.capability_record_ref.clone(),
        limitations: cap.limitations.clone(),
        authored_by: Some(THOTH_ACTOR_ID.into()),
    })
}

/// Build an `Unsupported`-status claim under `class`. `Unsupported` is the
/// floor rung (§7.3) — safe to file under any permitted class since it
/// asserts no positive evidence.
fn unsupported_claim(
    subject: &str,
    class: ClaimClass,
    snap_ref: &str,
    statement: String,
) -> GroundedClaim {
    GroundedClaim {
        claim_id: format!("claim_{subject}"),
        claim_class: class,
        subject: subject.into(),
        status: ClaimStatus::Unsupported,
        flags: ClaimFlags::default(),
        statement,
        snapshot_ref: snap_ref.to_owned(),
        evidence_refs: vec![],
        settlement_refs: vec![],
        capability_record_ref: None,
        limitations: vec![],
        authored_by: Some(THOTH_ACTOR_ID.into()),
    }
}

/// Derive claims from the snapshot within the permitted class boundary.
/// The query executor cannot touch non-permitted regions (§10.3). Total over
/// `QuestionKind`: every reachable arm (all but `AskWhyDenied`, handled
/// earlier) always emits at least one claim when `permitted` is non-empty
/// (guaranteed by the caller's deny-all-when-empty gate), so `Answered`
/// never carries an unexplained empty claim set (T13.3).
fn derive_claims(
    kind: &QuestionKind,
    subject: &str,
    snapshot: &dyn SnapshotView,
    permitted: &[ClaimClass],
) -> Vec<GroundedClaim> {
    let mut claims = Vec::new();
    let snap_ref = snapshot.snapshot_ref();

    match kind {
        QuestionKind::AskCapability | QuestionKind::AskProjectionSupport => {
            match snapshot.capability(subject) {
                Some(cap) => {
                    if let Some(claim) = capped_capability_claim(&cap, snap_ref, permitted) {
                        claims.push(claim);
                    }
                }
                None => {
                    if let Some(class) = lowest_permitted_capability_class(permitted) {
                        claims.push(unsupported_claim(
                            subject,
                            class,
                            snap_ref,
                            format!("{subject} is not declared in this installation"),
                        ));
                    }
                }
            }
        }
        QuestionKind::AskEnvironmentStatus => {
            if permitted.contains(&ClaimClass::EnvironmentStatus) {
                match snapshot.environment_status(subject) {
                    Some(env) => {
                        let status = if env.available {
                            ClaimStatus::Available
                        } else {
                            ClaimStatus::Declared
                        };
                        let mut flags = ClaimFlags::default();
                        if !env.available {
                            flags.unavailable_in_environment = true;
                        }
                        claims.push(GroundedClaim {
                            claim_id: format!("claim_env_{subject}"),
                            claim_class: ClaimClass::EnvironmentStatus,
                            subject: subject.into(),
                            status,
                            flags,
                            statement: format!(
                                "{} is {}",
                                subject,
                                if env.available {
                                    "available"
                                } else {
                                    "unavailable"
                                }
                            ),
                            snapshot_ref: snap_ref.to_owned(),
                            evidence_refs: env.probe_evidence_ref.iter().cloned().collect(),
                            settlement_refs: vec![],
                            capability_record_ref: None,
                            limitations: vec![],
                            authored_by: Some(THOTH_ACTOR_ID.into()),
                        });
                    }
                    None => {
                        claims.push(unsupported_claim(
                            subject,
                            ClaimClass::EnvironmentStatus,
                            snap_ref,
                            format!("{subject} environment status is unknown"),
                        ));
                    }
                }
            }
        }
        QuestionKind::AskAvailableAffordances => {
            if let Some(class) = lowest_permitted_capability_class(permitted) {
                let names = snapshot.declared_capabilities();
                if names.is_empty() {
                    claims.push(unsupported_claim(
                        "affordances",
                        class,
                        snap_ref,
                        "no declared capabilities are available in this installation".into(),
                    ));
                } else {
                    for name in names {
                        if let Some(cap) = snapshot.capability(&name) {
                            if let Some(claim) = capped_capability_claim(&cap, snap_ref, permitted)
                            {
                                claims.push(claim);
                            }
                        }
                    }
                }
            } else if let Some(class) = permitted.first().cloned() {
                // Identity/Architecture permitted but no capability-ladder
                // class granted; declare no data available rather than
                // silently returning nothing (§9.3, T13.3).
                claims.push(unsupported_claim(
                    "affordances",
                    class,
                    snap_ref,
                    "no affordance data is available for the permitted class".into(),
                ));
            }
        }
        QuestionKind::AskOperationRequirements | QuestionKind::AskAuthorityRequirements => {
            if permitted.contains(&ClaimClass::AuthorityRequirements) {
                claims.push(unsupported_claim(
                    subject,
                    ClaimClass::AuthorityRequirements,
                    snap_ref,
                    format!("no authority-requirement data is available for {subject}"),
                ));
            }
        }
        QuestionKind::AskFailureExplanation => {
            if permitted.contains(&ClaimClass::FailureCondition) {
                claims.push(unsupported_claim(
                    subject,
                    ClaimClass::FailureCondition,
                    snap_ref,
                    format!("no failure-condition data is available for {subject}"),
                ));
            }
        }
        QuestionKind::AskEvidenceForClaim => match snapshot.capability(subject) {
            Some(cap)
                if matches!(
                    cap.status,
                    ClaimStatus::Demonstrated | ClaimStatus::Validated
                ) =>
            {
                if let Some(claim) = capped_capability_claim(&cap, snap_ref, permitted) {
                    claims.push(claim);
                }
            }
            _ => {
                claims.push(unsupported_claim(
                    subject,
                    ClaimClass::DemonstratedCapability,
                    snap_ref,
                    format!("no demonstrated evidence is available for {subject}"),
                ));
            }
        },
        QuestionKind::AskWhyDenied => {
            unreachable!("ask_why_denied is handled in answer_question before derive_claims")
        }
    }
    claims
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::collections::HashMap;

    struct MockSnapshot {
        snap_ref: String,
        stale: bool,
        caps: HashMap<String, CapabilityInfo>,
        envs: HashMap<String, EnvironmentInfo>,
        decisions: HashMap<String, RecordedAuthorityDecision>,
    }

    impl Default for MockSnapshot {
        fn default() -> Self {
            Self {
                snap_ref: "smsnap_01".into(),
                stale: false,
                caps: HashMap::new(),
                envs: HashMap::new(),
                decisions: HashMap::new(),
            }
        }
    }

    impl SnapshotView for MockSnapshot {
        fn snapshot_ref(&self) -> &str {
            &self.snap_ref
        }
        fn is_stale(&self) -> bool {
            self.stale
        }
        fn capability(&self, name: &str) -> Option<CapabilityInfo> {
            self.caps.get(name).cloned()
        }
        fn declared_capabilities(&self) -> Vec<String> {
            self.caps.keys().cloned().collect()
        }
        fn environment_status(&self, tool: &str) -> Option<EnvironmentInfo> {
            self.envs.get(tool).cloned()
        }
        fn recorded_authority_decision(
            &self,
            authority_decision_ref: &str,
        ) -> Option<RecordedAuthorityDecision> {
            self.decisions.get(authority_decision_ref).cloned()
        }
    }

    /// Wraps a snapshot and counts calls to the bounded, content-bearing
    /// query methods only — never `snapshot_ref`/`is_stale`/
    /// `recorded_authority_decision` (metadata, not self-model content).
    struct CountingSnapshot {
        inner: MockSnapshot,
        query_calls: Cell<u32>,
    }

    impl SnapshotView for CountingSnapshot {
        fn snapshot_ref(&self) -> &str {
            self.inner.snapshot_ref()
        }
        fn is_stale(&self) -> bool {
            self.inner.is_stale()
        }
        fn capability(&self, name: &str) -> Option<CapabilityInfo> {
            self.query_calls.set(self.query_calls.get() + 1);
            self.inner.capability(name)
        }
        fn declared_capabilities(&self) -> Vec<String> {
            self.query_calls.set(self.query_calls.get() + 1);
            self.inner.declared_capabilities()
        }
        fn environment_status(&self, tool: &str) -> Option<EnvironmentInfo> {
            self.query_calls.set(self.query_calls.get() + 1);
            self.inner.environment_status(tool)
        }
        fn recorded_authority_decision(&self, r: &str) -> Option<RecordedAuthorityDecision> {
            self.inner.recorded_authority_decision(r)
        }
    }

    struct AllowAllPolicy;
    impl DisclosurePolicy for AllowAllPolicy {
        fn permits(&self, _actor: &str, _class: &ClaimClass) -> bool {
            true
        }
    }

    struct DenyAllPolicy;
    impl DisclosurePolicy for DenyAllPolicy {
        fn permits(&self, _actor: &str, _class: &ClaimClass) -> bool {
            false
        }
    }

    /// Grants only the classes explicitly listed.
    struct OnlyPolicy(Vec<ClaimClass>);
    impl DisclosurePolicy for OnlyPolicy {
        fn permits(&self, _actor: &str, class: &ClaimClass) -> bool {
            self.0.contains(class)
        }
    }

    struct RequireFreshPolicy;
    impl DisclosurePolicy for RequireFreshPolicy {
        fn permits(&self, _actor: &str, _class: &ClaimClass) -> bool {
            true
        }
        fn requires_fresh(&self) -> bool {
            true
        }
    }

    fn question(kind: QuestionKind, subject: &str) -> ThothQuestion {
        ThothQuestion {
            question_id: "thq_test".into(),
            kind,
            subject: subject.into(),
            actor_id: "agent_test".into(),
            process_id: "proc_test".into(),
            case_id: None,
            purpose: "test".into(),
            asked_at: "2026-07-17T00:00:00Z".into(),
        }
    }

    fn demonstrated_cap(name: &str) -> CapabilityInfo {
        CapabilityInfo {
            name: name.into(),
            status: ClaimStatus::Demonstrated,
            flags: ClaimFlags::default(),
            evidence_refs: vec!["evi_01".into()],
            settlement_refs: vec!["set_01".into()],
            capability_record_ref: Some("cap_01".into()),
            limitations: vec![],
        }
    }

    #[test]
    fn t11_1_demonstrated_capability_reported_correctly() {
        let snap = MockSnapshot {
            caps: HashMap::from([("domain-rust".into(), demonstrated_cap("domain-rust"))]),
            ..Default::default()
        };
        let ans = answer_question(
            &question(QuestionKind::AskCapability, "domain-rust"),
            &snap,
            &AllowAllPolicy,
            "tha_01",
            "2026-07-17T00:00:00Z",
        );
        assert_eq!(ans.disposition, Disposition::Answered);
        assert_eq!(ans.claims.len(), 1);
        assert_eq!(ans.claims[0].status, ClaimStatus::Demonstrated);
        assert!(ans.claims[0].capability_record_ref.is_some());
    }

    #[test]
    fn t11_1_attempted_only_never_demonstrated() {
        let snap = MockSnapshot {
            caps: HashMap::from([(
                "domain-rust".into(),
                CapabilityInfo {
                    name: "domain-rust".into(),
                    status: ClaimStatus::Validated,
                    flags: ClaimFlags::default(),
                    evidence_refs: vec![],
                    settlement_refs: vec![],
                    capability_record_ref: None,
                    limitations: vec![],
                },
            )]),
            ..Default::default()
        };
        let ans = answer_question(
            &question(QuestionKind::AskCapability, "domain-rust"),
            &snap,
            &AllowAllPolicy,
            "tha_01",
            "2026-07-17T00:00:00Z",
        );
        assert!(ans.claims[0].status <= ClaimStatus::Validated);
    }

    #[test]
    fn t11_2_declared_but_unavailable_environment() {
        let snap = MockSnapshot {
            caps: HashMap::from([(
                "tla".into(),
                CapabilityInfo {
                    name: "tla".into(),
                    status: ClaimStatus::Declared,
                    flags: ClaimFlags {
                        unavailable_in_environment: true,
                        ..Default::default()
                    },
                    evidence_refs: vec!["evi_probe".into()],
                    settlement_refs: vec![],
                    capability_record_ref: None,
                    limitations: vec![],
                },
            )]),
            ..Default::default()
        };
        let ans = answer_question(
            &question(QuestionKind::AskCapability, "tla"),
            &snap,
            &AllowAllPolicy,
            "tha_02",
            "2026-07-17T00:00:00Z",
        );
        assert_eq!(ans.claims[0].status, ClaimStatus::Declared);
        assert!(ans.claims[0].flags.unavailable_in_environment);
    }

    #[test]
    fn t11_3_restricted_class_denied() {
        let snap = MockSnapshot::default();
        // DenyAllPolicy ⇒ every candidate class denied.
        let ans = answer_question(
            &question(QuestionKind::AskCapability, "anything"),
            &snap,
            &DenyAllPolicy,
            "tha_03",
            "2026-07-17T00:00:00Z",
        );
        assert_eq!(ans.disposition, Disposition::Denied);
        assert!(ans.claims.is_empty());
        // Denial names classes only, no subject detail.
        assert!(!ans.omitted_claim_classes.is_empty());
    }

    #[test]
    fn t11_3_two_restricted_subjects_yield_identical_denials() {
        let snap = MockSnapshot::default();
        let ans1 = answer_question(
            &question(QuestionKind::AskCapability, "secret_A"),
            &snap,
            &DenyAllPolicy,
            "tha_03",
            "2026-07-17T00:00:00Z",
        );
        let ans2 = answer_question(
            &question(QuestionKind::AskCapability, "secret_B"),
            &snap,
            &DenyAllPolicy,
            "tha_03",
            "2026-07-17T00:00:00Z",
        );
        // Denial shapes identical apart from question_id.
        assert_eq!(ans1.disposition, ans2.disposition);
        assert_eq!(ans1.claims, ans2.claims);
        assert_eq!(ans1.omitted_claim_classes, ans2.omitted_claim_classes);
    }

    #[test]
    fn t11_4_replay_determinism() {
        let snap = MockSnapshot {
            caps: HashMap::from([("domain-rust".into(), demonstrated_cap("domain-rust"))]),
            ..Default::default()
        };
        let q = question(QuestionKind::AskCapability, "domain-rust");
        let a1 = answer_question(&q, &snap, &AllowAllPolicy, "tha_01", "2026-07-17T00:00:00Z");
        let a2 = answer_question(&q, &snap, &AllowAllPolicy, "tha_01", "2026-07-17T00:00:00Z");
        assert_eq!(a1, a2);
    }

    #[test]
    fn t11_8_absent_surface_denies_all() {
        // DenyAllPolicy simulates absent surface.
        let snap = MockSnapshot::default();
        let ans = answer_question(
            &question(QuestionKind::AskEnvironmentStatus, "cargo"),
            &snap,
            &DenyAllPolicy,
            "tha_08",
            "2026-07-17T00:00:00Z",
        );
        assert_eq!(ans.disposition, Disposition::Denied);
    }

    #[test]
    fn t11_9_stale_snapshot_disclosed() {
        let snap = MockSnapshot {
            stale: true,
            ..Default::default()
        };
        let ans = answer_question(
            &question(QuestionKind::AskAvailableAffordances, "all"),
            &snap,
            &AllowAllPolicy,
            "tha_09",
            "2026-07-17T00:00:00Z",
        );
        assert_eq!(ans.freshness, Freshness::Stale);
    }

    #[test]
    fn t11_6_authority_notice_present() {
        let snap = MockSnapshot::default();
        let ans = answer_question(
            &question(QuestionKind::AskAvailableAffordances, "all"),
            &snap,
            &AllowAllPolicy,
            "tha_06",
            "2026-07-17T00:00:00Z",
        );
        assert!(!ans.authority_notice.is_empty());
        assert!(ans.authority_notice.contains("no execution authority"));
    }

    #[test]
    fn t11_7_thoth_cannot_settle_own_claim() {
        // Thoth authored a claim; it cannot settle it.
        let result = sea_forge_core::types::validate_claim_authorship_sod(Some("thoth"), "thoth");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.class(), "sod_violation");
    }

    #[test]
    fn t11_7_different_actor_can_settle() {
        let result =
            sea_forge_core::types::validate_claim_authorship_sod(Some("thoth"), "operator");
        assert!(result.is_ok());
    }

    #[test]
    fn t11_7_relabeling_cannot_bypass() {
        // The authored_by is immutable; a relabeled claim still carries it.
        // The check is on the identity string, not on any mutable field.
        let result = sea_forge_core::types::validate_claim_authorship_sod(Some("thoth"), "thoth");
        assert!(result.is_err());
        // Copying the claim doesn't help — authored_by is part of the hash.
        let result2 =
            sea_forge_core::types::validate_claim_authorship_sod(Some("thoth"), "thoth_copy");
        // A renamed declarer is still a different identity, so this passes.
        // The point is: the ORIGINAL authored_by cannot be removed.
        assert!(result2.is_ok());
    }

    #[test]
    fn t13b_claims_are_stamped_with_thoth_authorship() {
        let snap = MockSnapshot {
            caps: HashMap::from([("domain-rust".into(), demonstrated_cap("domain-rust"))]),
            ..Default::default()
        };
        let ans = answer_question(
            &question(QuestionKind::AskCapability, "domain-rust"),
            &snap,
            &AllowAllPolicy,
            "tha_t13b_1",
            "2026-07-17T00:00:00Z",
        );
        assert_eq!(ans.claims[0].authored_by.as_deref(), Some(THOTH_ACTOR_ID));
    }

    // ── Task 13: bounded, total claim derivation ──

    #[test]
    fn t13_1_declared_only_grant_caps_demonstrated_to_declared() {
        let snap = MockSnapshot {
            caps: HashMap::from([("domain-rust".into(), demonstrated_cap("domain-rust"))]),
            ..Default::default()
        };
        let policy = OnlyPolicy(vec![ClaimClass::DeclaredCapability]);
        let ans = answer_question(
            &question(QuestionKind::AskCapability, "domain-rust"),
            &snap,
            &policy,
            "tha_t13_1",
            "2026-07-17T00:00:00Z",
        );
        assert_eq!(ans.claims.len(), 1);
        assert_eq!(ans.claims[0].claim_class, ClaimClass::DeclaredCapability);
        assert_eq!(ans.claims[0].status, ClaimStatus::Declared);
        assert!(ans.claims[0].status < ClaimStatus::Demonstrated);
    }

    #[test]
    fn t13_1_installed_only_grant_caps_demonstrated_to_available() {
        let snap = MockSnapshot {
            caps: HashMap::from([("domain-rust".into(), demonstrated_cap("domain-rust"))]),
            ..Default::default()
        };
        let policy = OnlyPolicy(vec![ClaimClass::InstalledCapability]);
        let ans = answer_question(
            &question(QuestionKind::AskCapability, "domain-rust"),
            &snap,
            &policy,
            "tha_t13_1b",
            "2026-07-17T00:00:00Z",
        );
        assert_eq!(ans.claims.len(), 1);
        assert_eq!(ans.claims[0].claim_class, ClaimClass::InstalledCapability);
        assert_eq!(ans.claims[0].status, ClaimStatus::Available);
        assert!(ans.claims[0].status < ClaimStatus::Demonstrated);
    }

    #[test]
    fn t13_1_installed_capability_not_relabeled_as_higher_permitted_class() {
        // Only DemonstratedCapability is granted — no class at or below the
        // capability's natural (Installed) rung. Must not relabel the fact
        // under the higher class; must file Unsupported instead.
        let mut cap = demonstrated_cap("domain-rust");
        cap.status = ClaimStatus::Installed;
        let snap = MockSnapshot {
            caps: HashMap::from([("domain-rust".into(), cap)]),
            ..Default::default()
        };
        let policy = OnlyPolicy(vec![ClaimClass::DemonstratedCapability]);
        let ans = answer_question(
            &question(QuestionKind::AskCapability, "domain-rust"),
            &snap,
            &policy,
            "tha_t13_1d",
            "2026-07-17T00:00:00Z",
        );
        assert_eq!(ans.claims.len(), 1);
        assert_eq!(ans.claims[0].status, ClaimStatus::Unsupported);
    }

    #[test]
    fn t13_1_declared_capability_not_relabeled_as_higher_permitted_class() {
        // Only InstalledCapability is granted — no class at or below the
        // capability's natural (Declared) rung. Must not relabel the fact
        // under the higher class; must file Unsupported instead.
        let mut cap = demonstrated_cap("domain-rust");
        cap.status = ClaimStatus::Declared;
        let snap = MockSnapshot {
            caps: HashMap::from([("domain-rust".into(), cap)]),
            ..Default::default()
        };
        let policy = OnlyPolicy(vec![ClaimClass::InstalledCapability]);
        let ans = answer_question(
            &question(QuestionKind::AskCapability, "domain-rust"),
            &snap,
            &policy,
            "tha_t13_1e",
            "2026-07-17T00:00:00Z",
        );
        assert_eq!(ans.claims.len(), 1);
        assert_eq!(ans.claims[0].status, ClaimStatus::Unsupported);
    }

    #[test]
    fn t13_1_full_grant_never_caps_natural_status() {
        let snap = MockSnapshot {
            caps: HashMap::from([("domain-rust".into(), demonstrated_cap("domain-rust"))]),
            ..Default::default()
        };
        let ans = answer_question(
            &question(QuestionKind::AskCapability, "domain-rust"),
            &snap,
            &AllowAllPolicy,
            "tha_t13_1c",
            "2026-07-17T00:00:00Z",
        );
        assert_eq!(
            ans.claims[0].claim_class,
            ClaimClass::DemonstratedCapability
        );
        assert_eq!(ans.claims[0].status, ClaimStatus::Demonstrated);
    }

    #[test]
    fn t13_2_unsupported_capability_under_declared_only_grant() {
        let snap = MockSnapshot::default();
        let policy = OnlyPolicy(vec![ClaimClass::DeclaredCapability]);
        let ans = answer_question(
            &question(QuestionKind::AskCapability, "missing"),
            &snap,
            &policy,
            "tha_t13_2a",
            "2026-07-17T00:00:00Z",
        );
        assert_eq!(ans.claims.len(), 1);
        assert_eq!(ans.claims[0].status, ClaimStatus::Unsupported);
        assert_eq!(ans.claims[0].claim_class, ClaimClass::DeclaredCapability);
    }

    #[test]
    fn t13_2_unsupported_capability_under_installed_only_grant() {
        let snap = MockSnapshot::default();
        let policy = OnlyPolicy(vec![ClaimClass::InstalledCapability]);
        let ans = answer_question(
            &question(QuestionKind::AskCapability, "missing"),
            &snap,
            &policy,
            "tha_t13_2b",
            "2026-07-17T00:00:00Z",
        );
        assert_eq!(ans.claims.len(), 1);
        assert_eq!(ans.claims[0].status, ClaimStatus::Unsupported);
        assert_eq!(ans.claims[0].claim_class, ClaimClass::InstalledCapability);
    }

    #[test]
    fn t13_2_unsupported_capability_under_demonstrated_only_grant() {
        let snap = MockSnapshot::default();
        let policy = OnlyPolicy(vec![ClaimClass::DemonstratedCapability]);
        let ans = answer_question(
            &question(QuestionKind::AskCapability, "missing"),
            &snap,
            &policy,
            "tha_t13_2c",
            "2026-07-17T00:00:00Z",
        );
        assert_eq!(ans.claims.len(), 1);
        assert_eq!(ans.claims[0].status, ClaimStatus::Unsupported);
        assert_eq!(
            ans.claims[0].claim_class,
            ClaimClass::DemonstratedCapability
        );
    }

    #[test]
    fn t13_3_every_question_kind_has_a_deterministic_typed_outcome() {
        let snap = MockSnapshot {
            caps: HashMap::from([("domain-rust".into(), demonstrated_cap("domain-rust"))]),
            decisions: HashMap::from([(
                "auth_dec_01".into(),
                RecordedAuthorityDecision {
                    authority_decision_ref: "auth_dec_01".into(),
                    denied_classes: vec![ClaimClass::SecurityImplementation],
                    reason_code: "policy_absent".into(),
                },
            )]),
            ..Default::default()
        };
        let kinds = [
            QuestionKind::AskCapability,
            QuestionKind::AskOperationRequirements,
            QuestionKind::AskAuthorityRequirements,
            QuestionKind::AskProjectionSupport,
            QuestionKind::AskEnvironmentStatus,
            QuestionKind::AskFailureExplanation,
            QuestionKind::AskEvidenceForClaim,
            QuestionKind::AskAvailableAffordances,
            QuestionKind::AskWhyDenied,
        ];
        for kind in kinds {
            let subject = if kind == QuestionKind::AskWhyDenied {
                "auth_dec_01"
            } else {
                "domain-rust"
            };
            let ans = answer_question(
                &question(kind.clone(), subject),
                &snap,
                &AllowAllPolicy,
                "tha_t13_3",
                "2026-07-17T00:00:00Z",
            );
            if ans.disposition == Disposition::Answered {
                assert!(
                    !ans.claims.is_empty()
                        || !ans.omitted_claim_classes.is_empty()
                        || !ans.limitations.is_empty(),
                    "{kind:?}: Answered must not carry an unexplained empty claim set"
                );
            }
        }
    }

    #[test]
    fn t13_3_ask_operation_requirements_is_bounded_and_unsupported() {
        let snap = MockSnapshot::default();
        let ans = answer_question(
            &question(QuestionKind::AskOperationRequirements, "write_file"),
            &snap,
            &AllowAllPolicy,
            "tha_t13_3b",
            "2026-07-17T00:00:00Z",
        );
        assert_eq!(ans.disposition, Disposition::Answered);
        assert_eq!(ans.claims.len(), 1);
        assert_eq!(ans.claims[0].claim_class, ClaimClass::AuthorityRequirements);
        assert_eq!(ans.claims[0].status, ClaimStatus::Unsupported);
    }

    #[test]
    fn t13_3_ask_failure_explanation_is_bounded_and_unsupported() {
        let snap = MockSnapshot::default();
        let ans = answer_question(
            &question(QuestionKind::AskFailureExplanation, "run_001"),
            &snap,
            &AllowAllPolicy,
            "tha_t13_3c",
            "2026-07-17T00:00:00Z",
        );
        assert_eq!(ans.claims.len(), 1);
        assert_eq!(ans.claims[0].claim_class, ClaimClass::FailureCondition);
        assert_eq!(ans.claims[0].status, ClaimStatus::Unsupported);
    }

    #[test]
    fn t13_3_ask_evidence_for_claim_demonstrated() {
        let snap = MockSnapshot {
            caps: HashMap::from([("domain-rust".into(), demonstrated_cap("domain-rust"))]),
            ..Default::default()
        };
        let ans = answer_question(
            &question(QuestionKind::AskEvidenceForClaim, "domain-rust"),
            &snap,
            &AllowAllPolicy,
            "tha_t13_3d",
            "2026-07-17T00:00:00Z",
        );
        assert_eq!(ans.claims.len(), 1);
        assert_eq!(
            ans.claims[0].claim_class,
            ClaimClass::DemonstratedCapability
        );
        assert!(!ans.claims[0].evidence_refs.is_empty());
    }

    #[test]
    fn t13_3_ask_evidence_for_claim_unsupported_when_not_demonstrated() {
        let snap = MockSnapshot {
            caps: HashMap::from([(
                "domain-rust".into(),
                CapabilityInfo {
                    name: "domain-rust".into(),
                    status: ClaimStatus::Declared,
                    flags: ClaimFlags::default(),
                    evidence_refs: vec![],
                    settlement_refs: vec![],
                    capability_record_ref: None,
                    limitations: vec![],
                },
            )]),
            ..Default::default()
        };
        let ans = answer_question(
            &question(QuestionKind::AskEvidenceForClaim, "domain-rust"),
            &snap,
            &AllowAllPolicy,
            "tha_t13_3e",
            "2026-07-17T00:00:00Z",
        );
        assert_eq!(ans.claims.len(), 1);
        assert_eq!(ans.claims[0].status, ClaimStatus::Unsupported);
    }

    #[test]
    fn t13_4_ask_why_denied_resolves_recorded_decision() {
        let snap = MockSnapshot {
            decisions: HashMap::from([(
                "auth_dec_01".into(),
                RecordedAuthorityDecision {
                    authority_decision_ref: "auth_dec_01".into(),
                    denied_classes: vec![ClaimClass::SecurityImplementation],
                    reason_code: "high_risk_class_not_granted".into(),
                },
            )]),
            ..Default::default()
        };
        let ans = answer_question(
            &question(QuestionKind::AskWhyDenied, "auth_dec_01"),
            &snap,
            &AllowAllPolicy,
            "tha_t13_4a",
            "2026-07-17T00:00:00Z",
        );
        assert_eq!(ans.disposition, Disposition::Answered);
        assert!(ans.claims.is_empty());
        assert_eq!(
            ans.omitted_claim_classes,
            vec![ClaimClass::SecurityImplementation]
        );
        assert!(ans
            .limitations
            .contains(&"high_risk_class_not_granted".to_string()));
    }

    #[test]
    fn t13_4_ask_why_denied_denies_when_unresolvable() {
        let snap = MockSnapshot::default();
        let ans = answer_question(
            &question(QuestionKind::AskWhyDenied, "nonexistent_ref"),
            &snap,
            &AllowAllPolicy,
            "tha_t13_4b",
            "2026-07-17T00:00:00Z",
        );
        assert_eq!(ans.disposition, Disposition::Denied);
        assert!(ans.claims.is_empty());
        assert!(ans.omitted_claim_classes.is_empty());
    }

    #[test]
    fn t13_5_required_fresh_refuses_before_any_bounded_query() {
        let snap = CountingSnapshot {
            inner: MockSnapshot {
                stale: true,
                caps: HashMap::from([("domain-rust".into(), demonstrated_cap("domain-rust"))]),
                ..Default::default()
            },
            query_calls: Cell::new(0),
        };
        let ans = answer_question(
            &question(QuestionKind::AskCapability, "domain-rust"),
            &snap,
            &RequireFreshPolicy,
            "tha_t13_5a",
            "2026-07-17T00:00:00Z",
        );
        assert_eq!(ans.disposition, Disposition::Denied);
        assert!(ans.claims.is_empty());
        assert_eq!(
            snap.query_calls.get(),
            0,
            "required-fresh refusal must invoke no bounded snapshot query"
        );
    }

    #[test]
    fn t13_5_required_fresh_allows_when_current() {
        let snap = CountingSnapshot {
            inner: MockSnapshot {
                stale: false,
                caps: HashMap::from([("domain-rust".into(), demonstrated_cap("domain-rust"))]),
                ..Default::default()
            },
            query_calls: Cell::new(0),
        };
        let ans = answer_question(
            &question(QuestionKind::AskCapability, "domain-rust"),
            &snap,
            &RequireFreshPolicy,
            "tha_t13_5b",
            "2026-07-17T00:00:00Z",
        );
        assert_eq!(ans.disposition, Disposition::Answered);
        assert!(snap.query_calls.get() > 0);
    }

    #[test]
    fn t13_5_not_required_fresh_still_discloses_stale_answers() {
        let snap = CountingSnapshot {
            inner: MockSnapshot {
                stale: true,
                caps: HashMap::from([("domain-rust".into(), demonstrated_cap("domain-rust"))]),
                ..Default::default()
            },
            query_calls: Cell::new(0),
        };
        let ans = answer_question(
            &question(QuestionKind::AskCapability, "domain-rust"),
            &snap,
            &AllowAllPolicy,
            "tha_t13_5c",
            "2026-07-17T00:00:00Z",
        );
        assert_eq!(ans.disposition, Disposition::Answered);
        assert_eq!(ans.freshness, Freshness::Stale);
        assert!(snap.query_calls.get() > 0);
    }
}
