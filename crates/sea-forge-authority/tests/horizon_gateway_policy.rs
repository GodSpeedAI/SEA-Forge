//! The gateway policy Horizon renders (`scripts/gateway-setup.ts`, `renderPolicyYaml`), run through the real SEA-Forge
//! authority engine and settlement gate. The fixture is the renderer's output, byte for byte; a change to the
//! renderer must update it, and the test fails if the external approval gate stops escalating.

use sea_forge_authority::cep::{
    authorize_settlement_declaration, settlement_approval_id, CepAuthorityService,
    SettlementAuthorityClass, SettlementCriteria, SettlementGate, SettlementIngress,
    SettlementOperation, SettlementTransport, VerifiedSettlementIngress,
};
use sea_forge_authority::{AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{Actor, ActorRole};
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

#[test]
fn a_gateway_cannot_approve_its_own_external_declaration() {
    let root = tempfile::tempdir().unwrap();
    let caller = gateway();
    let pending =
        pending_id_of(gate(RENDERED, root.path(), &external_ingress(&caller), OP).unwrap());
    // The gateway is a service actor. It has no approval_resolution rule, so the policy refuses it.
    assert!(resolve(root.path(), RENDERED, &pending, true, &caller).is_err());
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
