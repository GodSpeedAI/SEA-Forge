//! Thoth `ask` command (spec-adlc-thoth §11.1, §8.6).
//!
//! `sea-forge ask <kind> <subject> [--purpose] [--case] [--json]`
//! Exit codes: 0 answered, 4 denied (governance success), 2 usage, 1 error.

use sea_forge_core::errors::ForgeError;
use sea_forge_self_model::store;
use sea_forge_thoth::{
    engine::{answer_question, CapabilityInfo, DisclosurePolicy, SnapshotView},
    protocol::*,
};
use std::path::Path;

/// Parse a string into a QuestionKind. Returns Err on unknown kind.
fn parse_kind(s: &str) -> Result<QuestionKind, ForgeError> {
    match s {
        "ask_capability" => Ok(QuestionKind::AskCapability),
        "ask_operation_requirements" => Ok(QuestionKind::AskOperationRequirements),
        "ask_authority_requirements" => Ok(QuestionKind::AskAuthorityRequirements),
        "ask_projection_support" => Ok(QuestionKind::AskProjectionSupport),
        "ask_environment_status" => Ok(QuestionKind::AskEnvironmentStatus),
        "ask_failure_explanation" => Ok(QuestionKind::AskFailureExplanation),
        "ask_evidence_for_claim" => Ok(QuestionKind::AskEvidenceForClaim),
        "ask_available_affordances" => Ok(QuestionKind::AskAvailableAffordances),
        "ask_why_denied" => Ok(QuestionKind::AskWhyDenied),
        _ => Err(ForgeError::Input(format!("unknown question kind: {s}"))),
    }
}

/// Map a ClaimClass to its policy-surface string name.
fn class_to_str(c: &ClaimClass) -> &'static str {
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

/// A SnapshotView backed by the self-model store. Currently returns a minimal
/// view — real capability data requires joining the self-model composed model
/// with the ledger's CapabilityRecords (additive as the self-model matures).
struct SelfModelSnapshotView {
    snapshot_ref: String,
    stale: bool,
}

impl SnapshotView for SelfModelSnapshotView {
    fn snapshot_ref(&self) -> &str {
        &self.snapshot_ref
    }
    fn is_stale(&self) -> bool {
        self.stale
    }
    fn capability(&self, _name: &str) -> Option<CapabilityInfo> {
        // ponytail: no capability data yet; the self-model composed model
        // join with ledger CapabilityRecords is additive. Queries about
        // specific capabilities return None → unsupported.
        None
    }
    fn declared_capabilities(&self) -> Vec<String> {
        vec![]
    }
    fn environment_status(&self, _tool: &str) -> Option<sea_forge_thoth::engine::EnvironmentInfo> {
        None
    }
}

/// A DisclosurePolicy backed by the authority surface.
struct SurfacePolicy<'a> {
    surface: &'a sea_forge_authority::SelfDisclosureSurface,
}

impl DisclosurePolicy for SurfacePolicy<'_> {
    fn permits(&self, actor_id: &str, class: &ClaimClass) -> bool {
        // Map actor_id to role — for now use the actor_id directly.
        // The surface stores actor_role strings; the CLI --actor flag carries
        // the role.
        self.surface.permits(actor_id, class_to_str(class))
    }
}

/// Run the `ask` command. Returns exit code.
pub fn run(
    kind_str: &str,
    subject: &str,
    purpose: &str,
    case_id: Option<&str>,
    json: bool,
    root: &Path,
    actor: &str,
) -> Result<u8, ForgeError> {
    let kind = parse_kind(kind_str).map_err(|e| {
        eprintln!("sea-forge ask: {e}");
        e
    })?;

    // Load the current self-model snapshot for the snapshot ref.
    store::validate(root)?;
    let snapshot = store::current_snapshot(root)?.ok_or_else(|| {
        ForgeError::SelfModel("no self-model snapshot; run 'sea-forge self-model rebuild'".into())
    })?;
    let view = SelfModelSnapshotView {
        snapshot_ref: snapshot.snapshot_id.clone(),
        stale: store::is_stale(root)?,
    };

    // Load the authority policy bundle's self_disclosure surface.
    let policy_path = root.join("authority/active-policy.json");
    let surface = if policy_path.exists() {
        let text = std::fs::read_to_string(&policy_path)
            .map_err(|e| ForgeError::io("read authority policy", e))?;
        let bundle: sea_forge_authority::AuthorityPolicyBundle =
            serde_json::from_str(&text).map_err(|e| ForgeError::Serialization(e.to_string()))?;
        bundle.policy_surfaces.self_disclosure
    } else {
        // No policy ⇒ absent surface ⇒ deny-all (§8.2, T11.8).
        sea_forge_authority::SelfDisclosureSurface::default()
    };

    let policy = SurfacePolicy { surface: &surface };

    let question = ThothQuestion {
        question_id: format!("thq_{}", chrono::Utc::now().timestamp()),
        kind,
        subject: subject.into(),
        actor_id: actor.into(),
        process_id: "cli".into(),
        case_id: case_id.map(str::to_owned),
        purpose: purpose.into(),
        asked_at: chrono::Utc::now().to_rfc3339(),
    };

    let answer = answer_question(
        &question,
        &view,
        &policy,
        &format!("tha_{}", chrono::Utc::now().timestamp()),
        &chrono::Utc::now().to_rfc3339(),
    );

    // Output
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&answer)
                .map_err(|e| ForgeError::Serialization(e.to_string()))?
        );
    } else {
        print_human(&answer);
    }

    // Exit code: 0 answered/partial, 4 denied
    Ok(match answer.disposition {
        Disposition::Denied => 4,
        Disposition::Answered | Disposition::Partial => 0,
    })
}

fn print_human(answer: &ThothAnswer) {
    println!("disposition: {:?}", answer.disposition);
    println!("freshness: {:?}", answer.freshness);
    println!("assurance: {}", answer.assurance);
    println!("snapshot: {}", answer.snapshot_ref);
    for claim in &answer.claims {
        println!(
            "  {}: {:?} ({}) — {} [{}]",
            claim.subject,
            claim.status,
            claim.claim_class.to_string_rs(),
            claim.statement,
            claim.evidence_refs.join(", ")
        );
    }
    if !answer.omitted_claim_classes.is_empty() {
        print!("omitted: ");
        for c in &answer.omitted_claim_classes {
            print!("{c:?} ");
        }
        println!();
    }
    println!("notice: {}", answer.authority_notice);
}

// Helper trait for ClaimClass Debug-like rendering.
trait ClaimClassRender {
    fn to_string_rs(&self) -> String;
}

impl ClaimClassRender for ClaimClass {
    fn to_string_rs(&self) -> String {
        format!("{self:?}").to_lowercase()
    }
}
