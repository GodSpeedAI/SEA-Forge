//! `thoth.ask` answer projection (E13, epic journey 3).
//!
//! `sea_forge_thoth::protocol::ThothAnswer` is the kernel's own type and stays
//! that way: deriving `JsonSchema` on it would pull `schemars` across the
//! kernel boundary. This module is the SFWP-layer *view* of it — the same
//! relationship `run_views` and `case_views` have to the records they project.
//!
//! The projection is deliberately lossless about the parts an operator needs to
//! judge an answer's standing. Epic story 3.9 requires every answer to disclose
//! its freshness, assurance, limitations, omitted claim classes, and that
//! knowledge confers no execution authority — so all five travel, and none is
//! summarised away into a boolean.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// One grounded claim, with the records that back it.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct ClaimView {
    pub claim_id: String,
    /// Wire spelling, e.g. `demonstrated_capability`.
    pub claim_class: String,
    pub subject: String,
    /// Wire spelling of the capability ladder position, e.g. `demonstrated`.
    pub status: String,
    /// Template-generated from typed fields. Never free-form model output —
    /// which is why it is safe to render as prose.
    pub statement: String,
    pub snapshot_ref: String,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub settlement_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_record_ref: Option<String>,
}

/// The answer to one `thoth.ask`, as the Workbench reads it.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct ThothAnswerView {
    pub answer_id: String,
    pub question_id: String,
    /// `answered`, `partial`, or `denied`. A denial is a governed outcome with
    /// its own shape, not an error (epic invariant 3).
    pub disposition: String,
    #[serde(default)]
    pub claims: Vec<ClaimView>,
    /// Classes this answer was not permitted to speak to. Present on a partial
    /// answer as well as a denied one — an operator has to be able to see that
    /// what they received is not the whole picture.
    #[serde(default)]
    pub omitted_claim_classes: Vec<String>,
    pub snapshot_ref: String,
    /// `current` or `stale`.
    pub freshness: String,
    /// Ledger assurance vocabulary (spec-full §7.0c).
    pub assurance: String,
    #[serde(default)]
    pub limitations: Vec<String>,
    /// The fixed notice that an answer grants nothing.
    pub authority_notice: String,
    pub answered_at: String,
}

/// Serialize a value to its wire string, falling back to the debug form.
///
/// Used for the enums this view flattens to strings. The fallback never fires
/// for the types involved — each is a plain unit-variant enum — but returning a
/// `Result` here would push an impossible error case onto every call site.
fn wire(value: &impl Serialize) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| "unknown".into())
}

impl From<sea_forge_thoth::protocol::ThothAnswer> for ThothAnswerView {
    fn from(answer: sea_forge_thoth::protocol::ThothAnswer) -> Self {
        Self {
            answer_id: answer.answer_id,
            question_id: answer.question_id,
            disposition: wire(&answer.disposition),
            claims: answer
                .claims
                .into_iter()
                .map(|claim| ClaimView {
                    claim_id: claim.claim_id,
                    claim_class: wire(&claim.claim_class),
                    subject: claim.subject,
                    status: wire(&claim.status),
                    statement: claim.statement,
                    snapshot_ref: claim.snapshot_ref,
                    evidence_refs: claim.evidence_refs,
                    settlement_refs: claim.settlement_refs,
                    capability_record_ref: claim.capability_record_ref,
                })
                .collect(),
            omitted_claim_classes: answer.omitted_claim_classes.iter().map(wire).collect(),
            snapshot_ref: answer.snapshot_ref,
            freshness: wire(&answer.freshness),
            assurance: answer.assurance,
            limitations: answer.limitations,
            authority_notice: answer.authority_notice,
            answered_at: answer.answered_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_forge_thoth::protocol::{ClaimClass, ThothAnswer};

    /// A denial must keep carrying the disclosures that let an operator judge
    /// it. Projecting a denied answer down to "denied" would drop exactly the
    /// fields story 3.9 requires, and the surface would have nothing honest to
    /// show beyond the word.
    #[test]
    fn a_denied_answer_keeps_its_omitted_classes_and_notice() {
        let view: ThothAnswerView = ThothAnswer::denied(
            "q_1",
            "a_1",
            vec![ClaimClass::SecurityImplementation],
            "snap_1",
            "2026-07-31T00:00:00Z",
        )
        .into();

        assert_eq!(view.disposition, "denied");
        assert!(view.claims.is_empty());
        assert_eq!(view.omitted_claim_classes, ["security_implementation"]);
        assert!(
            !view.authority_notice.is_empty(),
            "the notice that an answer grants nothing must survive projection"
        );
        assert_eq!(view.freshness, "current");
        assert!(!view.assurance.is_empty());
    }
}
