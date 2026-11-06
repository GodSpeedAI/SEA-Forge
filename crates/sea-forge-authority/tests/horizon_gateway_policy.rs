//! The gateway policy Horizon renders (`scripts/gateway-setup.ts`, `renderPolicyYaml`), run through the real SEA-Forge
//! authority engine and settlement gate. The fixture is the renderer's output, byte for byte; a change to the
//! renderer must update it, and the test fails if the external approval gate stops escalating.

use sea_forge_authority::cep::{
    authorize_settlement_declaration, settlement_approval_id, CepAuthorityService,
    SettlementAuthorityClass, SettlementCriteria, SettlementGate, SettlementIngress,
    SettlementOperation, SettlementTransport, VerifiedSettlementIngress,
};
use sea_forge_authority::AuthorityEvaluation;
use sea_forge_authority::{AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{Actor, ActorRole, AuthorityAction, Verdict};
use sea_forge_domainforge::WorldRegistry;

const RENDERED: &str = include_str!("fixtures/horizon-gateway-policy.yaml");
const OP: &str = "op-settle-horizon";
const CONTENT: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn engine(policy: &str) -> (AuthorityPolicyBundle, PolicyAuthorityEngine) {
    let bundle: AuthorityPolicyBundle = sea_forge_core::yaml::from_str(policy).unwrap();
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    (bundle, engine)
}

fn gateway() -> Actor {
    Actor {
        actor_id: "gateway".into(),
        role: ActorRole::Service,
    }
}

fn operator(id: &str) -> Actor {
    Actor {
        actor_id: id.into(),
        role: ActorRole::Operator,
    }
}

/// The SWE_SEED ingress is external. The CEP settlement ingress has no transport and is local.
fn external_ingress(caller: &Actor) -> VerifiedSettlementIngress<'_> {
    VerifiedSettlementIngress::verified(
        SettlementIngress::SweSeedDeclaration,
        SettlementTransport::SweSeedAdapter,
        None,
        caller,
        vec![],
    )
    .unwrap()
}

fn local_ingress(caller: &Actor) -> VerifiedSettlementIngress<'_> {
    VerifiedSettlementIngress::verified(
        SettlementIngress::CepSettlement,
        SettlementTransport::None,
        None,
        caller,
        vec![],
    )
    .unwrap()
}

fn gate(
    policy: &str,
    root: &std::path::Path,
    ingress: &VerifiedSettlementIngress<'_>,
    operation_id: &str,
) -> Result<SettlementGate, sea_forge_authority::cep::CepAuthorityError> {
    let (bundle, engine) = engine(policy);
    authorize_settlement_declaration(
        &engine,
        &bundle,
        root,
        SettlementOperation {
            case_id: "case_horizon",
            operation_id,
            declared_request_sha256: CONTENT,
        },
        ingress,
        chrono::Duration::hours(24),
    )
}

fn pending_id_of(gate: SettlementGate) -> String {
    match gate {
        SettlementGate::Pending { record, .. } => record.pending_id,
        other => panic!("expected a pending approval, got {other:?}"),
    }
}

fn resolve(
    root: &std::path::Path,
    policy: &str,
    pending_id: &str,
    approve: bool,
    resolver: &Actor,
) -> Result<serde_json::Value, sea_forge_authority::cep::CepAuthorityError> {
    let (bundle, engine) = engine(policy);
    let worlds = WorldRegistry::new();
    let service = CepAuthorityService {
        engine: &engine,
        bundle: &bundle,
        worlds: &worlds,
        root,
        approval_ttl: chrono::Duration::hours(24),
        criteria: SettlementCriteria::default(),
    };
    service.resolve_approval(&settlement_approval_id(pending_id), approve, resolver)
}

#[test]
fn the_fixture_is_the_rendered_policy_with_the_gate_before_the_fallback() {
    let gated = RENDERED
        .find("name: settlement-external-approval\n")
        .unwrap();
    let fallback = RENDERED.find("name: settlement, verdict: allow").unwrap();
    assert!(
        gated < fallback,
        "first-match order: the gate must precede the ungated fallback"
    );
    assert_eq!(RENDERED.matches("requires_approval: true").count(), 2);
}

#[test]
fn a_gateway_external_declaration_escalates_and_is_not_declared() {
    let root = tempfile::tempdir().unwrap();
    let caller = gateway();
    let pending = gate(
        RENDERED,
        root.path(),
        &external_ingress(&caller),
        "run-ext-1",
    )
    .unwrap();
    assert!(matches!(pending, SettlementGate::Pending { .. }));
}

#[test]
fn a_local_declaration_is_not_escalated_by_the_external_gate() {
    let root = tempfile::tempdir().unwrap();
    let caller = gateway();
    let allowed = gate(
        RENDERED,
        root.path(),
        &local_ingress(&caller),
        "run-local-1",
    )
    .unwrap();
    assert!(
        matches!(
            allowed,
            SettlementGate::Allowed(SettlementAuthorityClass::Local)
        ),
        "local declarations still match the ungated fallback, got {allowed:?}"
    );
}

#[test]
fn an_operator_requested_external_declaration_escalates_too() {
    let root = tempfile::tempdir().unwrap();
    let requester = operator("op-alice");
    let pending = gate(
        RENDERED,
        root.path(),
        &external_ingress(&requester),
        "run-op-1",
    );
    assert!(matches!(pending.unwrap(), SettlementGate::Pending { .. }));
}

/// A refused resolver must leave the approval unresolved. Refusal-audit entries are allowed; a resolution is not.
fn assert_still_pending(root: &std::path::Path, pending_id: &str) {
    assert_eq!(
        sea_forge_authority::cep::settlement_status(root, pending_id).unwrap(),
        sea_forge_authority::cep::SettlementStatus::Pending,
        "a refused resolver must not resolve the approval"
    );
}

#[test]
fn a_gateway_cannot_approve_its_own_external_declaration() {
    let root = tempfile::tempdir().unwrap();
    let caller = gateway();
    let pending =
        pending_id_of(gate(RENDERED, root.path(), &external_ingress(&caller), OP).unwrap());
    // The gateway is a service actor. It has no approval_resolution rule, so the policy refuses it.
    assert!(resolve(root.path(), RENDERED, &pending, true, &caller).is_err());
    assert_still_pending(root.path(), &pending);
}

#[test]
fn a_service_actor_without_an_approval_resolution_rule_cannot_resolve_an_operator_request() {
    let root = tempfile::tempdir().unwrap();
    let requester = operator("op-alice");
    let pending =
        pending_id_of(gate(RENDERED, root.path(), &external_ingress(&requester), OP).unwrap());
    // The gateway is not the requester here, so only policy refuses it: it has no approval_resolution rule.
    let refused = resolve(root.path(), RENDERED, &pending, true, &gateway());
    let text = refused.unwrap_err().to_string();
    assert!(text.contains("may not resolve"), "{text}");
    assert_still_pending(root.path(), &pending);
}

#[test]
fn an_operator_cannot_approve_their_own_request() {
    let root = tempfile::tempdir().unwrap();
    let requester = operator("op-alice");
    let pending =
        pending_id_of(gate(RENDERED, root.path(), &external_ingress(&requester), OP).unwrap());
    // Authorised to approve by policy, yet refused as the requester: separation is enforced by SEA-Forge.
    let refused = resolve(root.path(), RENDERED, &pending, true, &requester);
    assert!(refused.is_err());
    let text = refused.unwrap_err().to_string();
    assert!(
        text.contains("self_approval_refused") || text.contains("requester"),
        "{text}"
    );
    assert_still_pending(root.path(), &pending);
}

#[test]
fn a_different_operator_can_approve_an_operator_requested_declaration() {
    let root = tempfile::tempdir().unwrap();
    let requester = operator("op-alice");
    let pending =
        pending_id_of(gate(RENDERED, root.path(), &external_ingress(&requester), OP).unwrap());
    let approver = operator("op-bob");
    resolve(root.path(), RENDERED, &pending, true, &approver)
        .expect("a second operator may approve");
}

#[test]
fn an_ungated_external_allow_placed_before_the_gate_is_refused_at_policy_load() {
    // The hazard the gate depends on: a class-scoped allow without approval, placed first, would win the
    // first-match and skip escalation. The loader refuses such a rule, so the gate cannot be shadowed by it.
    let shadowed = RENDERED.replacen(
        "  # External settlement",
        "  - name: shadow\n    verdict: allow\n    actor_role: service\n    operation_kind: settlement_declaration\n    boundary_constraints:\n      settlement_authority_class: [external]\n  # External settlement",
        1,
    );
    let bundle: AuthorityPolicyBundle = sea_forge_core::yaml::from_str(&shadowed).unwrap();
    assert!(PolicyAuthorityEngine::new(bundle).is_err());
}

#[test]
fn an_unscoped_service_allow_placed_before_the_gate_does_not_shadow_external_declarations() {
    // A plain service allow has no class constraint, so it matches only local declarations. External ones still
    // reach the gate and escalate.
    let plain_first = RENDERED.replacen(
        "  # External settlement",
        "  - { name: shadow-local, verdict: allow, actor_role: service, operation_kind: settlement_declaration }\n  # External settlement",
        1,
    );
    let root = tempfile::tempdir().unwrap();
    let caller = gateway();
    let pending = gate(
        &plain_first,
        root.path(),
        &external_ingress(&caller),
        "run-plain-first",
    )
    .unwrap();
    assert!(matches!(pending, SettlementGate::Pending { .. }));
}

/// Evaluate a raw `settlement_declaration` action under the rendered policy, so the class parameter itself is
/// under test. `None` leaves `settlement_authority_class` out of the parameters.
fn class_verdict(class: Option<&str>) -> Verdict {
    let root = tempfile::tempdir().unwrap();
    let (bundle, engine) = engine(RENDERED);
    let mut parameters =
        serde_json::json!({"operation_id": OP, "surface": "settlement_declaration"});
    if let Some(class) = class {
        parameters["settlement_authority_class"] = class.into();
    }
    let action = AuthorityAction::Reserved {
        resource_type: "settlement_declaration".into(),
        resource_id: OP.into(),
        parameters,
    };
    let actor = gateway();
    let binding = bundle.resolve_identity(&actor.actor_id, actor.role.clone());
    engine
        .evaluate(AuthorityEvaluation {
            actor: &actor,
            binding,
            run_id: OP,
            case_id: "cognate",
            plan_item_id: OP,
            sequence: 1,
            action: &action,
            workspace_root: root.path(),
            evidence_refs: vec![],
            artifacts_root: None,
            timeout_secs: None,
            env_keys: Default::default(),
            opa_candidate: None,
            environment: None,
        })
        .unwrap()
        .verdict
}

/// Pins the behaviour Horizon issue #19 asks about: the rendered fallback rule carries no class constraint, yet a
/// missing or unrecognised class is not allowed. An unconstrained rule matches only the local class, so the
/// fallback does not fail open. If the engine ever changed that, this test fails.
#[test]
fn the_unconstrained_fallback_denies_a_missing_or_unknown_class() {
    // Deny, not merely "not Allow": an Escalate would make an invalid class eligible for approval.
    assert_eq!(class_verdict(None), Verdict::Deny, "a missing class");
    for unknown in ["internal", "External", "LOCAL", "", "local ", "remote"] {
        assert_eq!(
            class_verdict(Some(unknown)),
            Verdict::Deny,
            "class {unknown:?} must be denied"
        );
    }
}

#[test]
fn the_fallback_allows_local_and_the_gate_requires_approval_for_external() {
    assert_eq!(class_verdict(Some("local")), Verdict::Allow);
    assert_eq!(class_verdict(Some("external")), Verdict::Escalate);
}
