use serde::{Deserialize, Serialize};

// ── Claim model (§7.3) ──

/// Total order for capping: each rung is more confident than the one below.
/// Missing evidence caps, never elevates (§7.3, §16.1).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ClaimStatus {
    Unknown,
    Unsupported,
    Declared,
    Installed,
    Available,
    Validated,
    Demonstrated,
}

/// Orthogonal status flags that modify the base ClaimStatus without elevating
/// it (§7.3).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct ClaimFlags {
    #[serde(default)]
    pub degraded: bool,
    #[serde(default)]
    pub deprecated: bool,
    #[serde(default)]
    pub quarantined: bool,
    #[serde(default)]
    pub disabled_by_policy: bool,
    #[serde(default)]
    pub unavailable_in_environment: bool,
    #[serde(default)]
    pub proven_narrow_variation: bool,
}

/// Disclosure vocabulary. The last four classes default to deny for every
/// actor; there is no built-in rule that allows them (§7.3).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ClaimClass {
    Identity,
    Architecture,
    DeclaredCapability,
    InstalledCapability,
    DemonstratedCapability,
    AuthorityRequirements,
    EnvironmentStatus,
    FailureCondition,
    // High-risk classes — deny by default, schema friction to grant (§8.2).
    SecurityImplementation,
    CustomerPrivate,
    CredentialBearing,
    PolicyThresholds,
}

/// Map a claim class to its policy-surface string name (§8.2). Total: every
/// variant has a name. Shared by the CLI ask adapter and the Thoth service so
/// both derive identical policy-surface lookups (T14A/T14B).
pub fn claim_class_to_surface_str(c: &ClaimClass) -> &'static str {
    match c {
        ClaimClass::Identity => "identity",
        ClaimClass::Architecture => "architecture",
        ClaimClass::DeclaredCapability => "declared_capability",
        ClaimClass::InstalledCapability => "installed_capability",
        ClaimClass::DemonstratedCapability => "demonstrated_capability",
        ClaimClass::AuthorityRequirements => "authority_requirements",
        ClaimClass::EnvironmentStatus => "environment_status",
        ClaimClass::FailureCondition => "failure_condition",
        ClaimClass::SecurityImplementation => "security_implementation",
        ClaimClass::CustomerPrivate => "customer_private",
        ClaimClass::CredentialBearing => "credential_bearing",
        ClaimClass::PolicyThresholds => "policy_thresholds",
    }
}

/// The four high-risk classes that require explicit_high_risk: true and a
/// compensating control (§8.2).
pub fn is_high_risk(class: &ClaimClass) -> bool {
    matches!(
        class,
        ClaimClass::SecurityImplementation
            | ClaimClass::CustomerPrivate
            | ClaimClass::CredentialBearing
            | ClaimClass::PolicyThresholds
    )
}

/// A single evidence-linked claim about the self-model (§7.3).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GroundedClaim {
    pub claim_id: String,
    pub claim_class: ClaimClass,
    /// Concept ref or capability name.
    pub subject: String,
    pub status: ClaimStatus,
    #[serde(default)]
    pub flags: ClaimFlags,
    /// Template-generated from typed fields — never free-form model output.
    pub statement: String,
    pub snapshot_ref: String,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub settlement_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_record_ref: Option<String>,
    #[serde(default)]
    pub limitations: Vec<String>,
    /// Immutable actor identity when Thoth authors the claim (§10.3).
    /// Enables SoD: Thoth cannot settle or promote a claim it authored.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authored_by: Option<String>,
}

// ── Question model (§7.4) ──

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum QuestionKind {
    AskCapability,
    AskOperationRequirements,
    AskAuthorityRequirements,
    AskProjectionSupport,
    AskEnvironmentStatus,
    AskFailureExplanation,
    AskEvidenceForClaim,
    AskAvailableAffordances,
    AskWhyDenied,
}

/// Parse the CLI/server wire string form of a question kind. Shared by both
/// ingresses (T14B) so an unknown-kind input is rejected identically
/// regardless of which adapter received it.
pub fn parse_question_kind(s: &str) -> Option<QuestionKind> {
    match s {
        "ask_capability" => Some(QuestionKind::AskCapability),
        "ask_operation_requirements" => Some(QuestionKind::AskOperationRequirements),
        "ask_authority_requirements" => Some(QuestionKind::AskAuthorityRequirements),
        "ask_projection_support" => Some(QuestionKind::AskProjectionSupport),
        "ask_environment_status" => Some(QuestionKind::AskEnvironmentStatus),
        "ask_failure_explanation" => Some(QuestionKind::AskFailureExplanation),
        "ask_evidence_for_claim" => Some(QuestionKind::AskEvidenceForClaim),
        "ask_available_affordances" => Some(QuestionKind::AskAvailableAffordances),
        "ask_why_denied" => Some(QuestionKind::AskWhyDenied),
        _ => None,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ThothQuestion {
    pub question_id: String,
    pub kind: QuestionKind,
    /// Typed per kind (concept ref, capability name, etc.).
    pub subject: String,
    pub actor_id: String,
    pub process_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub case_id: Option<String>,
    /// ≤ 500 chars.
    pub purpose: String,
    pub asked_at: String,
}

/// Compile-time mapping from question kind to candidate claim classes (§9.3).
/// Classification is total: every kind maps to a fixed set. There is no
/// "unclassified question" path.
pub fn candidate_classes(kind: &QuestionKind) -> Vec<ClaimClass> {
    use ClaimClass::*;
    match kind {
        QuestionKind::AskCapability => vec![
            DeclaredCapability,
            InstalledCapability,
            DemonstratedCapability,
        ],
        QuestionKind::AskOperationRequirements | QuestionKind::AskAuthorityRequirements => {
            vec![AuthorityRequirements]
        }
        QuestionKind::AskProjectionSupport => vec![DeclaredCapability, InstalledCapability],
        QuestionKind::AskEnvironmentStatus => vec![EnvironmentStatus],
        QuestionKind::AskFailureExplanation => vec![FailureCondition],
        QuestionKind::AskEvidenceForClaim => vec![DemonstratedCapability],
        QuestionKind::AskAvailableAffordances => {
            vec![Identity, Architecture, DeclaredCapability]
        }
        // ask_why_denied reads only the recorded denial; no candidate classes.
        QuestionKind::AskWhyDenied => vec![],
    }
}

// ── Disclosure and answer model (§7.4) ──

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct DisclosurePlan {
    pub question_id: String,
    pub authority_decision_ref: String,
    pub permitted_claim_classes: Vec<ClaimClass>,
    /// Sorted concept-ref/namespace patterns the bounded query may touch.
    #[serde(default)]
    pub permitted_regions: Vec<String>,
    pub omitted_claim_classes: Vec<ClaimClass>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Disposition {
    Answered,
    Partial,
    Denied,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Freshness {
    Current,
    Stale,
}

/// Fixed authority notice: answers confer no execution authority (§7.4).
pub const AUTHORITY_NOTICE: &str = "This answer confers no execution authority.";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ThothAnswer {
    pub answer_id: String,
    pub question_id: String,
    pub disposition: Disposition,
    /// Empty when denied.
    #[serde(default)]
    pub claims: Vec<GroundedClaim>,
    #[serde(default)]
    pub omitted_claim_classes: Vec<ClaimClass>,
    pub snapshot_ref: String,
    pub freshness: Freshness,
    /// Ledger assurance vocabulary (spec-full §7.0c).
    pub assurance: String,
    #[serde(default)]
    pub limitations: Vec<String>,
    pub authority_notice: String,
    pub answered_at: String,
}

impl ThothAnswer {
    /// Construct a denied answer. Carries only the denied classes and
    /// `ask_why_denied` eligibility — no subject-derived detail beyond what
    /// the question itself contained (§7.4, §10.3).
    pub fn denied(
        question_id: &str,
        answer_id: &str,
        denied_classes: Vec<ClaimClass>,
        snapshot_ref: &str,
        answered_at: &str,
    ) -> Self {
        Self {
            answer_id: answer_id.into(),
            question_id: question_id.into(),
            disposition: Disposition::Denied,
            claims: vec![],
            omitted_claim_classes: denied_classes,
            snapshot_ref: snapshot_ref.into(),
            freshness: Freshness::Current,
            assurance: "local_tamper_evident".into(),
            limitations: vec![],
            authority_notice: AUTHORITY_NOTICE.into(),
            answered_at: answered_at.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classification_is_total() {
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
        for kind in &kinds {
            // Every kind must map to a deterministic set (possibly empty).
            let a = candidate_classes(kind);
            let b = candidate_classes(kind);
            assert_eq!(a, b, "classification must be deterministic");
        }
    }

    #[test]
    fn high_risk_classes_identified() {
        assert!(is_high_risk(&ClaimClass::SecurityImplementation));
        assert!(is_high_risk(&ClaimClass::CustomerPrivate));
        assert!(is_high_risk(&ClaimClass::CredentialBearing));
        assert!(is_high_risk(&ClaimClass::PolicyThresholds));
        assert!(!is_high_risk(&ClaimClass::Identity));
        assert!(!is_high_risk(&ClaimClass::DeclaredCapability));
    }

    #[test]
    fn denied_answer_carries_no_claims() {
        let ans = ThothAnswer::denied(
            "thq_01",
            "tha_01",
            vec![ClaimClass::SecurityImplementation],
            "smsnap_01",
            "2026-07-17T00:00:00Z",
        );
        assert_eq!(ans.disposition, Disposition::Denied);
        assert!(ans.claims.is_empty());
        assert_eq!(ans.authority_notice, AUTHORITY_NOTICE);
    }

    #[test]
    fn claim_status_ordering() {
        assert!(ClaimStatus::Demonstrated > ClaimStatus::Validated);
        assert!(ClaimStatus::Validated > ClaimStatus::Available);
        assert!(ClaimStatus::Available > ClaimStatus::Installed);
        assert!(ClaimStatus::Installed > ClaimStatus::Declared);
        assert!(ClaimStatus::Declared > ClaimStatus::Unsupported);
        assert!(ClaimStatus::Unsupported > ClaimStatus::Unknown);
    }
}
