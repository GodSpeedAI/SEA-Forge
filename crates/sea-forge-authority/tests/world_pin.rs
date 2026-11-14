//! WP-18: promote and re-pin a world over the existing approval-gated
//! `semantic_change` transition, with a durable pin record. NT-05 (four
//! refusals, one code each), NT-07 (evidence stays with its world), NT-08 and
//! NT-21 (a failing or absent regression matrix blocks), AT-12 (proposer is not
//! the approver, failed attempts and non-promotion retained, recovery by
//! re-pin through the same transition), NT-04 after-half (the pin record is
//! what makes the candidate world current).

use sea_forge_authority::cep::{
    proposal_digest, repin_digest, CepAuthorityService, PinRecord, PinRefusal, PromoteRequest,
    RepinRequest, CLAIM_PROPOSAL_DIGEST, CLAIM_REPIN_DIGEST,
};
use sea_forge_authority::{AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{Actor, ActorRole};
use sea_forge_domainforge::{SeaSourceSet, SourceFile, WorldRegistry};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const BASE: &str = "@namespace \"t\"\nentity \"Tank\" { key id: uuid }\n";
const CHANGED: &str =
    "@namespace \"t\"\nentity \"Tank\" { key id: uuid }\nentity \"Pump\" { key id: uuid }\n";
const OTHER: &str = "@namespace \"t\"\nentity \"Valve\" { key id: uuid }\n";
const OPEN: &str = "version: \"0.1\"\nrules:\n  \
  - name: open\n    verdict: allow\n    actor_role: service\n    operation_kind: world_transition\n  \
  - name: actions\n    verdict: allow\n    actor_role: service\n    operation_kind: cognate_action\n  \
  - name: approvers\n    verdict: allow\n    actor_role: operator\n    operation_kind: approval_resolution\n";
/// Policy that would let the requesting service identity resolve approvals too: only the identity
/// rule (the requester cannot resolve its own request) stands between it and its own approval.
const OPEN_SELF: &str = "version: \"0.1\"\nrules:\n  \
  - name: open\n    verdict: allow\n    actor_role: service\n    operation_kind: world_transition\n  \
  - name: approvers\n    verdict: allow\n    actor_role: operator\n    operation_kind: approval_resolution\n  \
  - name: anyone\n    verdict: allow\n    actor_role: service\n    operation_kind: approval_resolution\n";
const DENYING: &str = "version: \"0.1\"\nrules:\n  \
  - name: no\n    verdict: deny\n    actor_role: service\n    operation_kind: world_transition\n";

struct Cell {
    root: tempfile::TempDir,
    worlds: WorldRegistry,
    v1: String,
    v2: String,
    v3: String,
}

fn register(worlds: &mut WorldRegistry, src: &str) -> String {
    let set = SeaSourceSet {
        entry_uri: "demo.sea".into(),
        files: vec![SourceFile {
            uri: "demo.sea".into(),
            sha256: hex::encode(Sha256::digest(src.as_bytes())),
            content: src.into(),
        }],
    };
    worlds
        .register_source_set("demo", &set)
        .unwrap()
        .to_string()
}

fn cell() -> Cell {
    let mut worlds = WorldRegistry::new();
    let (v1, v2, v3) = (
        register(&mut worlds, BASE),
        register(&mut worlds, CHANGED),
        register(&mut worlds, OTHER),
    );
    Cell {
        root: tempfile::tempdir().unwrap(),
        worlds,
        v1,
        v2,
        v3,
    }
}

fn with_ttl<T>(
    c: &Cell,
    policy: &str,
    ttl: chrono::Duration,
    f: impl FnOnce(&CepAuthorityService<'_>) -> T,
) -> T {
    let bundle: AuthorityPolicyBundle = sea_forge_core::yaml::from_str(policy).unwrap();
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let service = CepAuthorityService {
        engine: &engine,
        bundle: &bundle,
        worlds: &c.worlds,
        root: c.root.path(),
        approval_ttl: ttl,
        criteria: Default::default(),
    };
    f(&service)
}

fn with<T>(c: &Cell, policy: &str, f: impl FnOnce(&CepAuthorityService<'_>) -> T) -> T {
    with_ttl(c, policy, chrono::Duration::hours(24), f)
}

fn service() -> Actor {
    Actor {
        actor_id: "cognate-service".into(),
        role: ActorRole::Service,
    }
}
fn operator(id: &str) -> Actor {
    Actor {
        actor_id: id.into(),
        role: ActorRole::Operator,
    }
}

// ---- RFC 8785 for the documents the test builds (ASCII keys, integers)
fn jcs(v: &Value) -> String {
    match v {
        Value::Object(m) => {
            let mut k: Vec<&String> = m.keys().collect();
            k.sort();
            format!(
                "{{{}}}",
                k.iter()
                    .map(|k| format!("{}:{}", serde_json::to_string(k).unwrap(), jcs(&m[*k])))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }
        Value::Array(a) => format!("[{}]", a.iter().map(jcs).collect::<Vec<_>>().join(",")),
        other => serde_json::to_string(other).unwrap(),
    }
}
fn hex_of(v: &Value) -> String {
    hex::encode(Sha256::digest(jcs(v).as_bytes()))
}

fn proposal(old: &str, cand: &str, proposer: &str, note: &str) -> Value {
    let mut p = json!({
        "contract": "revision-proposal.v1", "proposer": proposer, "scope": "case:case1",
        "old_world_ref": old, "candidate_world_ref": cand,
        "held_out_rule": {"rule_id": "rule:s1-heldout", "sha256": format!("sha256:{}", "3".repeat(64))},
        "diagnosis": {"episodes": ["episode:case1:stepA:1@sha256:11"], "residual_ids": ["residual:a"], "occurrence_ids": ["sea-forge:a1"]},
        "held_out": {"episodes": ["episode:case1:stepB:1@sha256:22"], "residual_ids": ["residual:b"], "occurrence_ids": ["sea-forge:b1"]},
        "intended_changes": [{"element": "flow-move-purge", "change": "add_declaration", "supersedes": [note]}],
        "semantic_diff_sha256": format!("sha256:{}", "4".repeat(64)),
    });
    p["proposal_id"] = json!(format!("proposal:{}", hex_of(&p)));
    p
}

fn matrix_for(
    p: &Value,
    rows: Vec<Value>,
    verdict: &str,
    refusal: Value,
    escalated: bool,
) -> Value {
    let id = p["proposal_id"].as_str().unwrap();
    let mut m = json!({
        "contract": "regression-matrix.v1", "proposal_id": id,
        "proposal_digest": format!("sha256:{}", &id["proposal:".len()..]),
        "old_world_ref": p["old_world_ref"], "candidate_world_ref": p["candidate_world_ref"],
        "old_comparisons": ["comparison:o"], "new_comparisons": ["comparison:n"],
        "rows": rows, "summary": {"pass": 1, "fail": 0, "escalated": 0, "gains": 0},
        "verdict": verdict, "failure_codes": [], "refusal_code": refusal, "escalated": escalated,
    });
    m["matrix_id"] = json!(format!("matrix:{}", hex_of(&m)));
    m
}

fn row(result: &str, code: Value, escalated: bool) -> Value {
    json!({"element": "e|structural|x", "episode_ref": "episode:case1:stepA:1", "comparison": "structural",
           "old_expectation": "x", "new_expectation": "x", "old_class": "confirmed", "new_class": "confirmed",
           "result": result, "code": code, "reason": "r", "intended": false, "escalated": escalated, "reported": null})
}

fn good_matrix(p: &Value) -> Value {
    matrix_for(
        p,
        vec![row("pass", Value::Null, false)],
        "pass",
        Value::Null,
        false,
    )
}

fn request(
    source: &str,
    op: &str,
    target: &str,
    claims: Value,
    approval: Option<&str>,
    lineage: &[&str],
) -> Value {
    let mut ext = json!({
        "operation_id": op, "correlation_id": "corr-1",
        "operation_kind": "world_transition", "operation_name": "transition", "resource_id": target,
        "claims": claims,
    });
    if let Some(a) = approval {
        ext["approval_id"] = json!(a);
    }
    json!({
        "envelope_id": format!("env-authority_request-{op}"),
        "cep_version": "1.0.0", "envelope_version": "1.0", "envelope_kind": "authority_request",
        "created_at": "2026-10-05T12:00:00Z", "created_by": "cognate",
        "scope": {"evaluation_context": "cognate-authority_request", "world_ref": source},
        "boundary_record": {"scope": "s", "included_sections": ["authority"], "excluded_sections": [],
            "known_omissions": ["payload not carried"], "unknowns": [], "redactions": [],
            "compression_notes": [], "out_of_scope_entities": [], "classification": "internal", "limitations": []},
        "completeness_status": "partial", "omission_status": "marked",
        "provenance_refs": [format!("prov-{op}")], "validation_status": "valid", "lineage_refs": lineage,
        "provenance": [{"provenance_id": format!("prov-{op}"), "source_system_refs": ["cognate"],
            "producer_refs": ["cognate"], "production_method": "governed_operation", "created_at": "2026-10-05T12:00:00Z"}],
        "authority": [{"authority_id": "auth-1", "subject_actor_ref": "actor:alice"}],
        "transformations": [{"transformation_id": "tr-1", "transformation_type": "action_invocation"}],
        "references": [{"ref_id": "r1", "ref_type": "artifact_id", "target_uri_or_id": target,
            "availability_status": "available", "integrity_status": "verifiable"}],
        "states": [{"state_id": "s1", "state_type": "declared_state", "subject_ref": "run:1"}],
        "extensions": {
            "cep.profile": {"profile_id": "godspeed.authority_request", "profile_version": "1.0.0"},
            "godspeed.authority_request": ext,
            "godspeed.world_transition": {"target_world_ref": target, "transition_kind": "semantic_change",
                "reason": "adopt the reviewed model", "compatibility": "compatible"}
        }
    })
}

/// A cognate_action decision made in `world`: evidence that must stay with it.
fn action_request(world: &str, op: &str) -> Value {
    let mut r = request(world, op, "res-1", json!({"note": "x"}), None, &[]);
    r["extensions"]["godspeed.authority_request"]["operation_kind"] = json!("action");
    r["extensions"]["godspeed.authority_request"]["operation_name"] = json!("run");
    r["extensions"]["godspeed.authority_request"]
        .as_object_mut()
        .unwrap()
        .remove("claims");
    r["extensions"]
        .as_object_mut()
        .unwrap()
        .remove("godspeed.world_transition");
    r
}

/// The escalation (`op`), approval by `approver`, and the second request that
/// spends it. Returns (approval_id, envelope_to_promote_with).
fn escalate_and_approve(
    c: &Cell,
    source: &str,
    target: &str,
    claim_key: &str,
    digest: &str,
    op: &str,
) -> (String, Value) {
    let claims = json!({ claim_key: digest });
    let first = request(source, op, target, claims.clone(), None, &[]);
    let d = with(c, OPEN, |s| s.decide(&first, &service())).unwrap();
    let approval = d.envelope["extensions"]["godspeed.authority_decision"]["approval_id"]
        .as_str()
        .unwrap()
        .to_string();
    with(c, OPEN, |s| {
        s.resolve_approval(&approval, true, &operator("op-alice"))
    })
    .unwrap();
    let second = request(
        source,
        &format!("{op}-b"),
        target,
        claims,
        Some(&approval),
        &[&format!("env-authority_request-{op}")],
    );
    (approval, second)
}

fn promote(c: &Cell, p: &Value, m: &Value, env: &Value) -> Result<PinRecord, PinRefusal> {
    let baseline = c.v1.clone();
    with(c, OPEN, |s| {
        s.promote(
            &PromoteRequest {
                proposal: p.clone(),
                matrix: m.clone(),
                envelope: env.clone(),
            },
            &service(),
            Some(&baseline),
        )
    })
}

fn state(c: &Cell) -> (Option<String>, usize, usize) {
    let baseline = c.v1.clone();
    with(c, OPEN, |s| {
        let st = s.pin_state(Some(&baseline)).unwrap();
        (st.current, st.history.len(), s.transitions().unwrap().len())
    })
}

fn code(r: Result<PinRecord, PinRefusal>) -> &'static str {
    r.expect_err("expected a refusal").code
}

fn digest_of(p: &Value) -> String {
    proposal_digest(p).unwrap()
}

// ----------------------------------------------------------------- AT-12

#[test]
fn at12_promotion_goes_through_the_gated_transition_and_is_a_durable_pin_record() {
    let c = cell();
    let p = proposal(&c.v1, &c.v2, "cognate-service", "undeclared:x");
    let m = good_matrix(&p);
    assert_eq!(
        state(&c),
        (Some(c.v1.clone()), 0, 0),
        "before promotion the baseline is pinned"
    );
    let (approval, env) = escalate_and_approve(
        &c,
        &c.v1,
        &c.v2,
        CLAIM_PROPOSAL_DIGEST,
        &digest_of(&p),
        "p-1",
    );
    let pin = promote(&c, &p, &m, &env).unwrap();

    assert_eq!(pin.kind, "promoted");
    assert_eq!(
        (pin.from_world_ref.as_str(), pin.to_world_ref.as_str()),
        (c.v1.as_str(), c.v2.as_str())
    );
    assert_eq!(pin.proposer, "cognate-service");
    assert_eq!(pin.approver, "op-alice");
    assert_ne!(pin.proposer, pin.approver, "proposer is not the approver");
    assert_eq!(pin.approval_id, approval);
    assert_eq!(pin.bound_digest, digest_of(&p));
    assert_eq!(pin.matrix_id.as_deref(), m["matrix_id"].as_str());
    // The pin is carried by the existing transition: exactly one allowed transition, and the pin cites it.
    let facts = with(&c, OPEN, |s| s.transitions()).unwrap();
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].ledger_entry, pin.transition_ref);
    assert_eq!(facts[0].record["transition_kind"], "semantic_change");
    assert_eq!(facts[0].approval_id.as_deref(), Some(approval.as_str()));
    // NT-04 after-half: the candidate world is current only through the pin record.
    assert_eq!(state(&c), (Some(c.v2.clone()), 1, 1));
    let baseline = c.v1.clone();
    let st = with(&c, OPEN, |s| s.pin_state(Some(&baseline))).unwrap();
    assert_eq!(st.history, vec![pin.clone()]);
    // Durable: a fresh service over the same ledger sees it.
    let again = with(&c, OPEN, |s| s.pin_state(Some(&baseline))).unwrap();
    assert_eq!(again.current.as_deref(), Some(c.v2.as_str()));
    // The same approval cannot be spent on a second promotion.
    assert_eq!(code(promote(&c, &p, &m, &env)), "PIN_STALE");
}

#[test]
fn nt04_without_a_recorded_transition_the_candidate_is_never_current() {
    let c = cell();
    let baseline = c.v1.clone();
    let st = with(&c, OPEN, |s| s.pin_state(Some(&baseline))).unwrap();
    assert_eq!(st.current.as_deref(), Some(c.v1.as_str()));
    assert!(st.history.is_empty());
    // An escalation alone (nothing approved) does not move the pin.
    let p = proposal(&c.v1, &c.v2, "cognate-service", "undeclared:x");
    let first = request(
        &c.v1,
        "n-1",
        &c.v2,
        json!({CLAIM_PROPOSAL_DIGEST: digest_of(&p)}),
        None,
        &[],
    );
    with(&c, OPEN, |s| s.decide(&first, &service())).unwrap();
    assert_eq!(state(&c), (Some(c.v1.clone()), 0, 0));
}

#[test]
fn at12_proposer_is_not_the_approver() {
    let c = cell();
    let p = proposal(&c.v1, &c.v2, "cognate-service", "undeclared:x");
    let first = request(
        &c.v1,
        "s-1",
        &c.v2,
        json!({CLAIM_PROPOSAL_DIGEST: digest_of(&p)}),
        None,
        &[],
    );
    let d = with(&c, OPEN, |s| s.decide(&first, &service())).unwrap();
    let approval = d.envelope["extensions"]["godspeed.authority_decision"]["approval_id"]
        .as_str()
        .unwrap()
        .to_string();
    // The proposer (the requester) cannot approve its own proposal ...
    let own = with(&c, OPEN_SELF, |s| {
        s.resolve_approval(&approval, true, &service())
    });
    assert!(
        own.is_err(),
        "the identity rule refuses it although policy would allow the role"
    );
    // ... so no decision exists and the promotion cannot proceed.
    let env = request(
        &c.v1,
        "s-1-b",
        &c.v2,
        json!({CLAIM_PROPOSAL_DIGEST: digest_of(&p)}),
        Some(&approval),
        &["env-authority_request-s-1"],
    );
    assert_eq!(
        code(promote(&c, &p, &good_matrix(&p), &env)),
        "AUTHORITY_UNAVAILABLE"
    );
    // Another principal making the request for someone else's proposal is refused.
    let other = proposal(&c.v1, &c.v2, "someone-else", "undeclared:y");
    let (_, env2) = escalate_and_approve(
        &c,
        &c.v1,
        &c.v2,
        CLAIM_PROPOSAL_DIGEST,
        &digest_of(&other),
        "s-2",
    );
    assert_eq!(
        code(promote(&c, &other, &good_matrix(&other), &env2)),
        "PROPOSER_MISMATCH"
    );
    assert_eq!(state(&c), (Some(c.v1.clone()), 0, 0));
}

#[test]
fn at12_both_worlds_must_be_registered() {
    let c = cell();
    let ghost = format!("world:demo@sha256:{}", "e".repeat(64));
    let p = proposal(&c.v1, &ghost, "cognate-service", "undeclared:x");
    let env = request(
        &c.v1,
        "g-1",
        &ghost,
        json!({CLAIM_PROPOSAL_DIGEST: digest_of(&p)}),
        Some("apr-g-0"),
        &[],
    );
    let r = promote(&c, &p, &good_matrix(&p), &env).unwrap_err();
    assert_eq!(r.code, "TRANSITION_REFUSED", "{r}");
    assert_eq!(state(&c), (Some(c.v1.clone()), 0, 0));
}

#[test]
fn at12_refused_attempts_are_retained_and_so_is_a_non_promotion_with_its_divergence() {
    let c = cell();
    let p = proposal(&c.v1, &c.v2, "cognate-service", "undeclared:x");
    let env = request(
        &c.v1,
        "r-1",
        &c.v2,
        json!({CLAIM_PROPOSAL_DIGEST: digest_of(&p)}),
        Some("apr-nope"),
        &[],
    );
    assert_eq!(
        code(promote(&c, &p, &good_matrix(&p), &env)),
        "AUTHORITY_UNAVAILABLE"
    );
    assert_eq!(
        code(promote(&c, &p, &Value::Null, &env)),
        "REGRESSION_FAILED"
    );
    let attempts = with(&c, OPEN, |s| s.pin_attempts()).unwrap();
    let codes: Vec<&str> = attempts
        .iter()
        .map(|a| a["code"].as_str().unwrap())
        .collect();
    assert_eq!(codes, vec!["AUTHORITY_UNAVAILABLE", "REGRESSION_FAILED"]);
    assert_eq!(attempts[0]["bound_digest"], json!(digest_of(&p)));
    assert_eq!(attempts[0]["target_world_ref"], json!(c.v2));
    // Non-promotion: the divergence and the rationale are kept; no empty rationale.
    let rec = with(&c, OPEN, |s| {
        s.decline(
            &p,
            "the gain is real but the held-out episode is thin",
            &service(),
        )
    })
    .unwrap();
    assert_eq!(
        rec["rationale"],
        "the gain is real but the held-out episode is thin"
    );
    assert_eq!(
        rec["divergence"]["diagnosis_residuals"],
        p["diagnosis"]["residual_ids"]
    );
    assert_eq!(
        rec["divergence"]["held_out_residuals"],
        p["held_out"]["residual_ids"]
    );
    assert_eq!(
        with(&c, OPEN, |s| s.decline(&p, "  ", &service()))
            .unwrap_err()
            .code,
        "RATIONALE_REQUIRED"
    );
    assert_eq!(state(&c), (Some(c.v1.clone()), 0, 0));
}

#[test]
fn at12_the_digest_of_a_realitytrace_proposal_is_the_one_realitytrace_computed() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/world_pin");
    let p: Value =
        serde_json::from_slice(&std::fs::read(dir.join("rt-revision-proposal.json")).unwrap())
            .unwrap();
    let m: Value =
        serde_json::from_slice(&std::fs::read(dir.join("rt-regression-matrix.json")).unwrap())
            .unwrap();
    let digest = proposal_digest(&p)
        .expect("RealityTrace's proposal_id is the RFC 8785 digest SEA-Forge recomputes");
    assert_eq!(m["proposal_digest"], json!(digest));
    // A forged field is refused.
    let mut forged = p.clone();
    forged["proposer"] = json!("someone-else");
    assert_eq!(
        proposal_digest(&forged).unwrap_err().code,
        "PROPOSAL_INVALID"
    );
}

// ----------------------------------------------------------------- NT-05

#[test]
fn nt05_denied_unavailable_expired_and_digest_mismatched_decisions_cannot_promote() {
    let c = cell();
    let p = proposal(&c.v1, &c.v2, "cognate-service", "undeclared:x");
    let m = good_matrix(&p);
    let d = digest_of(&p);
    let untouched = |c: &Cell| {
        assert_eq!(
            state(c),
            (Some(c.v1.clone()), 0, 0),
            "the registry/pin did not change"
        )
    };

    // (a) denied: the approver rejected it.
    let first = request(
        &c.v1,
        "a-1",
        &c.v2,
        json!({CLAIM_PROPOSAL_DIGEST: d}),
        None,
        &[],
    );
    let dec = with(&c, OPEN, |s| s.decide(&first, &service())).unwrap();
    let apr = dec.envelope["extensions"]["godspeed.authority_decision"]["approval_id"]
        .as_str()
        .unwrap()
        .to_string();
    with(&c, OPEN, |s| {
        s.resolve_approval(&apr, false, &operator("op-alice"))
    })
    .unwrap();
    let env = request(
        &c.v1,
        "a-1-b",
        &c.v2,
        json!({CLAIM_PROPOSAL_DIGEST: d}),
        Some(&apr),
        &["env-authority_request-a-1"],
    );
    assert_eq!(code(promote(&c, &p, &m, &env)), "PROMOTION_DENIED");
    // (a') denied by policy: nothing to approve.
    let denied_c = cell();
    let first = request(
        &denied_c.v1,
        "a-2",
        &denied_c.v2,
        json!({CLAIM_PROPOSAL_DIGEST: d}),
        None,
        &[],
    );
    let p2 = proposal(
        &denied_c.v1,
        &denied_c.v2,
        "cognate-service",
        "undeclared:x",
    );
    let first = {
        let mut f = first;
        f["extensions"]["godspeed.authority_request"]["claims"] =
            json!({CLAIM_PROPOSAL_DIGEST: digest_of(&p2)});
        f
    };
    with(&denied_c, DENYING, |s| s.decide(&first, &service())).unwrap();
    let env = request(
        &denied_c.v1,
        "a-2-b",
        &denied_c.v2,
        json!({CLAIM_PROPOSAL_DIGEST: digest_of(&p2)}),
        Some("apr-a-2"),
        &["env-authority_request-a-2"],
    );
    let baseline = denied_c.v1.clone();
    let r = with(&denied_c, DENYING, |s| {
        s.promote(
            &PromoteRequest {
                proposal: p2.clone(),
                matrix: good_matrix(&p2),
                envelope: env.clone(),
            },
            &service(),
            Some(&baseline),
        )
    });
    assert_eq!(code(r), "PROMOTION_DENIED");
    untouched(&c);

    // (b) unavailable: still pending, never decided, and no approval named. Unknown is never approved.
    let c = cell();
    let first = request(
        &c.v1,
        "b-1",
        &c.v2,
        json!({CLAIM_PROPOSAL_DIGEST: d}),
        None,
        &[],
    );
    with(&c, OPEN, |s| s.decide(&first, &service())).unwrap();
    let pending = request(
        &c.v1,
        "b-1-b",
        &c.v2,
        json!({CLAIM_PROPOSAL_DIGEST: d}),
        Some("apr-b-1"),
        &["env-authority_request-b-1"],
    );
    assert_eq!(code(promote(&c, &p, &m, &pending)), "AUTHORITY_UNAVAILABLE");
    let unknown = request(
        &c.v1,
        "b-2-b",
        &c.v2,
        json!({CLAIM_PROPOSAL_DIGEST: d}),
        Some("apr-never-asked"),
        &[],
    );
    assert_eq!(code(promote(&c, &p, &m, &unknown)), "AUTHORITY_UNAVAILABLE");
    let none = request(
        &c.v1,
        "b-3",
        &c.v2,
        json!({CLAIM_PROPOSAL_DIGEST: d}),
        None,
        &[],
    );
    assert_eq!(code(promote(&c, &p, &m, &none)), "AUTHORITY_UNAVAILABLE");
    untouched(&c);

    // (c) expired: approved, then the window closed.
    let c = cell();
    let (_, env) = escalate_and_approve(&c, &c.v1, &c.v2, CLAIM_PROPOSAL_DIGEST, &d, "c-1");
    let baseline = c.v1.clone();
    let expired = with_ttl(&c, OPEN, chrono::Duration::seconds(-1), |s| {
        s.promote(
            &PromoteRequest {
                proposal: p.clone(),
                matrix: m.clone(),
                envelope: env.clone(),
            },
            &service(),
            Some(&baseline),
        )
    });
    assert_eq!(code(expired), "DECISION_EXPIRED");
    untouched(&c);

    // (d) digest mismatch: the approval was granted for another proposal of the same candidate.
    let c = cell();
    let other = proposal(&c.v1, &c.v2, "cognate-service", "undeclared:other");
    let (_, env) = escalate_and_approve(
        &c,
        &c.v1,
        &c.v2,
        CLAIM_PROPOSAL_DIGEST,
        &digest_of(&other),
        "d-1",
    );
    assert_eq!(code(promote(&c, &p, &m, &env)), "DECISION_DIGEST_MISMATCH");
    // ... and an approval granted with no digest claim at all binds to nothing.
    let c2 = cell();
    let first = request(&c2.v1, "d-2", &c2.v2, json!({"note": "x"}), None, &[]);
    let dec = with(&c2, OPEN, |s| s.decide(&first, &service())).unwrap();
    let apr = dec.envelope["extensions"]["godspeed.authority_decision"]["approval_id"]
        .as_str()
        .unwrap()
        .to_string();
    with(&c2, OPEN, |s| {
        s.resolve_approval(&apr, true, &operator("op-alice"))
    })
    .unwrap();
    let env = request(
        &c2.v1,
        "d-2-b",
        &c2.v2,
        json!({CLAIM_PROPOSAL_DIGEST: d}),
        Some(&apr),
        &["env-authority_request-d-2"],
    );
    assert_eq!(code(promote(&c2, &p, &m, &env)), "DECISION_DIGEST_MISMATCH");
    untouched(&c);
    untouched(&c2);
}

// ----------------------------------------------------------- NT-08, NT-21

#[test]
fn nt08_a_failing_or_absent_regression_matrix_blocks_even_with_a_valid_approval() {
    let c = cell();
    let p = proposal(&c.v1, &c.v2, "cognate-service", "undeclared:x");
    let (_, env) = escalate_and_approve(
        &c,
        &c.v1,
        &c.v2,
        CLAIM_PROPOSAL_DIGEST,
        &digest_of(&p),
        "m-1",
    );
    let failing = matrix_for(
        &p,
        vec![row("fail", json!("REGRESSION_FAILED"), false)],
        "fail",
        json!("REGRESSION_FAILED"),
        false,
    );
    assert_eq!(code(promote(&c, &p, &failing, &env)), "REGRESSION_FAILED");
    assert_eq!(
        code(promote(&c, &p, &Value::Null, &env)),
        "REGRESSION_FAILED",
        "absent"
    );
    assert_eq!(
        code(promote(&c, &p, &json!({}), &env)),
        "REGRESSION_FAILED",
        "empty"
    );
    // A matrix of another proposal.
    let other = proposal(&c.v1, &c.v2, "cognate-service", "undeclared:other");
    assert_eq!(
        code(promote(&c, &p, &good_matrix(&other), &env)),
        "REGRESSION_FAILED"
    );
    // A matrix that says pass over a failing row, or whose id is not its digest.
    let lying = matrix_for(
        &p,
        vec![row("fail", json!("REGRESSION_FAILED"), false)],
        "pass",
        Value::Null,
        false,
    );
    assert_eq!(code(promote(&c, &p, &lying, &env)), "REGRESSION_FAILED");
    let mut forged = good_matrix(&p);
    forged["summary"]["pass"] = json!(99);
    assert_eq!(code(promote(&c, &p, &forged, &env)), "REGRESSION_FAILED");
    assert_eq!(
        state(&c),
        (Some(c.v1.clone()), 0, 0),
        "nothing was decided: the approval is still unspent"
    );
    // The same request with a passing matrix then succeeds: the refusals were about the matrix alone.
    assert_eq!(
        promote(&c, &p, &good_matrix(&p), &env)
            .unwrap()
            .to_world_ref,
        c.v2
    );
}

#[test]
fn nt21_lost_capability_and_unlisted_weakening_each_refuse_promotion_with_their_own_code() {
    let c = cell();
    let p = proposal(&c.v1, &c.v2, "cognate-service", "undeclared:x");
    let (_, env) = escalate_and_approve(
        &c,
        &c.v1,
        &c.v2,
        CLAIM_PROPOSAL_DIGEST,
        &digest_of(&p),
        "w-1",
    );
    let lost = matrix_for(
        &p,
        vec![row("fail", json!("LOST_CAPABILITY"), false)],
        "fail",
        json!("LOST_CAPABILITY"),
        false,
    );
    assert_eq!(code(promote(&c, &p, &lost, &env)), "LOST_CAPABILITY");
    let weak = matrix_for(
        &p,
        vec![row("fail", json!("UNLISTED_WEAKENING"), false)],
        "fail",
        json!("UNLISTED_WEAKENING"),
        false,
    );
    assert_eq!(code(promote(&c, &p, &weak, &env)), "UNLISTED_WEAKENING");
    // A listed weakening needs the escalated approval class, which this path does not have: refused, not downgraded.
    let listed = matrix_for(
        &p,
        vec![row("pass", Value::Null, true)],
        "pass",
        Value::Null,
        true,
    );
    assert_eq!(
        code(promote(&c, &p, &listed, &env)),
        "ESCALATED_APPROVAL_REQUIRED"
    );
    assert_eq!(state(&c), (Some(c.v1.clone()), 0, 0));
}

// ----------------------------------------------------------------- NT-07

#[test]
fn nt07_settled_evidence_stays_bound_to_its_world_after_promotion_and_after_re_pin() {
    let c = cell();
    // Evidence settled under v1: a decision recorded in the old world.
    let ev = with(&c, OPEN, |s| {
        s.decide(&action_request(&c.v1, "ev-1"), &service())
    })
    .unwrap();
    let entry = ev.committed.entry_ulid().to_string();
    let read = |c: &Cell| with(c, OPEN, |s| s.evidence_world(&entry)).unwrap();
    assert_eq!(read(&c).as_deref(), Some(c.v1.as_str()));

    let p = proposal(&c.v1, &c.v2, "cognate-service", "undeclared:x");
    let (_, env) = escalate_and_approve(
        &c,
        &c.v1,
        &c.v2,
        CLAIM_PROPOSAL_DIGEST,
        &digest_of(&p),
        "e-1",
    );
    promote(&c, &p, &good_matrix(&p), &env).unwrap();
    assert_eq!(state(&c).0.as_deref(), Some(c.v2.as_str()));
    assert_eq!(
        read(&c).as_deref(),
        Some(c.v1.as_str()),
        "after promotion: still the old world_ref"
    );

    // Re-pin to v1: the pin moves back; the evidence did not move either way.
    let baseline = c.v1.clone();
    let rd = repin_digest(
        &c.v2,
        &c.v1,
        "the revision lost the held-out case",
        "pin-e-1-b",
    );
    let (_, renv) = escalate_and_approve(&c, &c.v2, &c.v1, CLAIM_REPIN_DIGEST, &rd, "e-2");
    let pin = with(&c, OPEN, |s| {
        s.repin(
            &RepinRequest {
                to_world_ref: c.v1.clone(),
                reason: "the revision lost the held-out case".into(),
                envelope: renv,
            },
            &service(),
            Some(&baseline),
        )
    })
    .unwrap();
    assert_eq!(pin.kind, "repinned");
    assert_eq!(state(&c).0.as_deref(), Some(c.v1.as_str()));
    assert_eq!(
        read(&c).as_deref(),
        Some(c.v1.as_str()),
        "after re-pin: still the old world_ref"
    );
    // The decision entry itself is unchanged in the append-only ledger.
    let ev_world = with(&c, OPEN, |s| s.evidence_world(pin.transition_ref.as_str())).unwrap();
    assert_eq!(
        ev_world.as_deref(),
        Some(c.v2.as_str()),
        "the promotion's own decision is bound to the world it was made in"
    );
}

// -------------------------------------------------------------- recovery

#[test]
fn recovery_re_pin_goes_through_the_same_gated_transition_and_is_a_record() {
    let c = cell();
    let baseline = c.v1.clone();
    let p = proposal(&c.v1, &c.v2, "cognate-service", "undeclared:x");
    let (_, env) = escalate_and_approve(
        &c,
        &c.v1,
        &c.v2,
        CLAIM_PROPOSAL_DIGEST,
        &digest_of(&p),
        "q-1",
    );
    let promoted = promote(&c, &p, &good_matrix(&p), &env).unwrap();
    let reason = "held-out regression";
    let rd = repin_digest(&c.v2, &c.v1, reason, &promoted.pin_id);
    let (apr, renv) = escalate_and_approve(&c, &c.v2, &c.v1, CLAIM_REPIN_DIGEST, &rd, "q-2");
    let repin = |c: &Cell, to: &str, reason: &str, env: &Value| {
        with(c, OPEN, |s| {
            s.repin(
                &RepinRequest {
                    to_world_ref: to.into(),
                    reason: reason.into(),
                    envelope: env.clone(),
                },
                &service(),
                Some(&baseline),
            )
        })
    };
    // A world never pinned before is not a re-pin target; a reason is required; the digest binds the reason.
    assert_eq!(
        repin(&c, &c.v3, reason, &renv).unwrap_err().code,
        "REPIN_TARGET_UNKNOWN"
    );
    assert_eq!(
        repin(&c, &c.v1, " ", &renv).unwrap_err().code,
        "REPIN_REASON_REQUIRED"
    );
    assert_eq!(
        repin(&c, &c.v1, "another reason", &renv).unwrap_err().code,
        "DECISION_DIGEST_MISMATCH"
    );
    let pin = repin(&c, &c.v1, reason, &renv).unwrap();
    assert_eq!(pin.kind, "repinned");
    assert_eq!(
        pin.supersedes_pin.as_deref(),
        Some(promoted.pin_id.as_str())
    );
    assert_eq!(pin.approval_id, apr);
    assert_eq!(pin.approver, "op-alice");
    let facts = with(&c, OPEN, |s| s.transitions()).unwrap();
    assert_eq!(
        facts.len(),
        2,
        "promotion and re-pin are two allowed transitions of the same kind"
    );
    assert!(facts
        .iter()
        .all(|f| f.record["transition_kind"] == "semantic_change"));
    assert_eq!(state(&c).0.as_deref(), Some(c.v1.as_str()));
    // Without an approval a re-pin is refused.
    let none = request(
        &c.v1,
        "q-3",
        &c.v2,
        json!({CLAIM_REPIN_DIGEST: "x"}),
        None,
        &[],
    );
    assert_eq!(
        repin(&c, &c.v2, "again", &none).unwrap_err().code,
        "AUTHORITY_UNAVAILABLE"
    );
}

#[test]
fn a_promotion_against_a_stale_pin_is_refused() {
    let c = cell();
    let p = proposal(&c.v1, &c.v2, "cognate-service", "undeclared:x");
    let (_, env) = escalate_and_approve(
        &c,
        &c.v1,
        &c.v2,
        CLAIM_PROPOSAL_DIGEST,
        &digest_of(&p),
        "t-1",
    );
    promote(&c, &p, &good_matrix(&p), &env).unwrap();
    // A proposal still written against v1 while v2 is pinned.
    let stale = proposal(&c.v1, &c.v3, "cognate-service", "undeclared:z");
    let (_, env2) = escalate_and_approve(
        &c,
        &c.v1,
        &c.v3,
        CLAIM_PROPOSAL_DIGEST,
        &digest_of(&stale),
        "t-2",
    );
    assert_eq!(
        code(promote(&c, &stale, &good_matrix(&stale), &env2)),
        "PIN_STALE"
    );
}

#[test]
fn a_crash_between_the_transition_and_the_pin_is_finished_not_decided_twice() {
    let c = cell();
    let p = proposal(&c.v1, &c.v2, "cognate-service", "undeclared:x");
    let (_, env) = escalate_and_approve(
        &c,
        &c.v1,
        &c.v2,
        CLAIM_PROPOSAL_DIGEST,
        &digest_of(&p),
        "k-1",
    );
    // The transition is decided (approval spent) but the process died before the pin was written.
    with(&c, OPEN, |s| s.decide(&env, &service())).unwrap();
    assert_eq!(state(&c), (Some(c.v1.clone()), 0, 1));
    let pin = promote(&c, &p, &good_matrix(&p), &env).unwrap();
    assert_eq!(pin.to_world_ref, c.v2);
    assert_eq!(
        state(&c),
        (Some(c.v2.clone()), 1, 1),
        "one transition, one pin"
    );
    let facts = with(&c, OPEN, |s| s.transitions()).unwrap();
    assert_eq!(facts[0].ledger_entry, pin.transition_ref);
}
