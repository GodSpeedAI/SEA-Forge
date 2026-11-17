//! The Field Case document shape shared by the case opener and the server
//! (`contracts/fde/field-case.v1.json`, SOP 4.2). The group and field keys are
//! the SOP labels in snake case; a test in `sea-forge-server` checks this table
//! against the frozen schema, so a drift fails there.

use serde_json::{json, Value};

/// The template a Field Case is opened from.
pub const TEMPLATE_REF: &str = "fde.field_case@1";

/// The SOP the document follows (`sop` member of the document).
pub const SOP_DOCUMENT: &str = "GS-FDE-SOP-001";
pub const SOP_VERSION: &str = "0.4.1";
pub const SOP_SHA256: &str = "043ae33ae6001da14f518feeb4ca2d23e280da2769d8378b36827f80d6057001";

/// `(group, fields)` in SOP 4.2 order: 45 fields in six groups.
pub const GROUPS: &[(&str, &[&str])] = &[
    (
        "purpose_and_outcome",
        &[
            "owner_sponsor",
            "signal",
            "stated_problem",
            "adopted_problem",
            "problem_class",
            "jtbd_outcome",
            "business_significance",
            "success_direction",
            "non_goals",
            "constraints",
            "decision_this_work_can_unlock",
        ],
    ),
    (
        "framing",
        &[
            "current_framing",
            "what_it_reveals",
            "what_it_hides",
            "better_question",
            "better_problem_if_successful",
        ],
    ),
    (
        "world_and_boundary",
        &[
            "world_ref_and_version",
            "checkpoint",
            "people_and_systems",
            "observed_conditions",
            "statements_needing_confirmation",
            "known_unknowns",
            "interfaces",
            "time_horizon",
        ],
    ),
    (
        "observer_and_authority",
        &[
            "case_lead",
            "criterion_owner",
            "delegated_actors",
            "independent_verifier",
            "settlement_and_disposition_authority",
            "risk_and_external_effect_limits",
            "authority_surface",
        ],
    ),
    (
        "current_horizon",
        &[
            "evidence_and_decisions",
            "contested_distinctions",
            "hypotheses",
            "possible_next_moves",
            "blocked_moves_with_reason",
            "available_and_missing_affordances",
            "budgets",
            "ignored_payment",
            "outstanding_obligations",
        ],
    ),
    (
        "continuity",
        &[
            "previous_episode",
            "worktree_session_artifact_links",
            "open_questions",
            "current_decision",
            "next_affordable_move",
        ],
    ),
];

/// The value of a field nobody has stated yet.
pub fn unknown_field() -> Value {
    json!({"status": "unknown", "source_ref": {"kind": "none", "ref": "none"}})
}

/// Whether `path` (`<group>.<field>`) names a field of the document.
pub fn is_field_path(path: &str) -> bool {
    path.split_once('.').is_some_and(|(group, field)| {
        GROUPS
            .iter()
            .any(|(g, fields)| *g == group && fields.contains(&field))
    })
}

/// The document of a case at open: revision 1, every field `unknown`.
pub fn opened_document(case_id: &str) -> Value {
    let mut fields = serde_json::Map::new();
    for (group, names) in GROUPS {
        let mut g = serde_json::Map::new();
        for name in *names {
            g.insert((*name).into(), unknown_field());
        }
        fields.insert((*group).into(), Value::Object(g));
    }
    json!({
        "case_id": case_id,
        "template_ref": TEMPLATE_REF,
        "revision": 1,
        "sop": {"document": SOP_DOCUMENT, "version": SOP_VERSION, "sha256": SOP_SHA256},
        "fields": Value::Object(fields),
    })
}
