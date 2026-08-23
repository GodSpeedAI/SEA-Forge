//! Task 14A: the real, joined Thoth service (spec-adlc-thoth §7.4, §10.3,
//! M11 service). Exercises `sea_forge_thoth::service::ask` end to end over a
//! real self-model snapshot, real capability records rebuilt from ledgered
//! declarations/envelopes, and a real authority policy bundle — no
//! caller-supplied status, no test double `SnapshotView`.

use sea_forge_core::types::{
    DeclarationIndependence, DeclarationReliability, DeclarationStatus, Declarer,
    SettlementDeclaration, SettlementStrength,
};
use sea_forge_self_model::{
    bundled, load_composed,
    store::{self, RebuildInputs},
    ExtensionState, ProbeResult, ToolchainProbe,
};
use sea_forge_thoth::protocol::*;
use sea_forge_thoth::service::ask;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

const ACTOR: &str = "agent_test";

fn temp_root(name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("sea-forge-thoth-service-{name}-{nonce}"));
    fs::create_dir_all(&root).unwrap();
    root
}

/// A real, bundled concept name — resolved dynamically so the test never
/// hardcodes a name the composed model doesn't actually declare.
fn real_capability_name() -> String {
    let composed = load_composed(&bundled()).unwrap();
    composed
        .concepts()
        .into_iter()
        .next()
        .expect("bundled composed model must declare at least one concept")
}

fn init_self_model(root: &Path, probes: Vec<ToolchainProbe>) {
    let inputs = RebuildInputs {
        cell_id: "cell_test",
        active_extensions: vec![ExtensionState {
            descriptor_ref: "ext_test".into(),
            status: "active".into(),
        }],
        environments_present: vec!["demo_env@0.1.0".into()],
        probes,
        sandbox_classes_available: vec!["local".into()],
        created_at: "2026-07-23T00:00:00Z",
        capability_projection_sha256: "sha256:test",
        actor_id: ACTOR,
    };
    store::ensure_init(root, &inputs).unwrap();
}

/// Append one real, qualifying (accepted/strong/independent) declaration for
/// `capability_name` directly to the compatibility view
/// `rebuild_capability` reads, so `CapabilityRecord.status` becomes at least
/// `Demonstrated` without needing to satisfy the full `default_v02_policy`
/// promotion thresholds (proving is exercised in `conformance_m4a.rs`).
fn append_qualifying_declaration(root: &Path, capability_name: &str) {
    let dir = root.join("settlement");
    fs::create_dir_all(&dir).unwrap();
    let decl = SettlementDeclaration {
        version: "0.1".into(),
        declaration_id: "decl_test_01".into(),
        settlement_ref: "set_test_01".into(),
        run_id: "run_test_01".into(),
        case_id: "case_test_01".into(),
        plan_item_id: capability_name.to_string(),
        claim_manifest_sha256: "sha256:manifest".into(),
        status: DeclarationStatus::Accepted,
        strength: SettlementStrength::Strong,
        qualifies_for_capability: true,
        criteria_ref: "crit_test_01".into(),
        criteria_sha256: "sha256:criteria".into(),
        criteria_record_hash: "sha256:criteria-record".into(),
        criteria_declared_at: "2026-07-22T00:00:00Z".into(),
        job_contract_ref: None,
        origin_refs: vec![],
        verifier_ref: "builtin:local".into(),
        verifier_sha256: "sha256:verifier".into(),
        verification_evidence_refs: vec!["evd_test_01".into()],
        declarer: Declarer {
            actor_id: "operator_test".into(),
            authority_ref: "swe_seed".into(),
            role: "R-AA".into(),
            standing_basis: "external_verification".into(),
        },
        independence: DeclarationIndependence {
            acting_entity_id: "entity_test".into(),
            independent: true,
            basis: "separate_actor".into(),
        },
        reliability: DeclarationReliability {
            feedback_delay_ms: 0,
            attribution_confidence: "1.000000".into(),
            gaming_exposure: "0.000000".into(),
            hidden_debt_blindness: "0.000000".into(),
            weight: "1.000000".into(),
            basis: "test_fixture".into(),
        },
        variation_tags: Default::default(),
        disruption_tags: vec![],
        orchestration_burden: None,
        issued_at: "2026-07-23T00:00:00Z".into(),
        source_evidence_refs: vec!["evd_test_01".into()],
        adapter_attestation_ref: None,
        authored_by: None,
        declaration_hash: "sha256:decl".into(),
    };
    let path = dir.join("declarations.jsonl");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .unwrap();
    let mut line = serde_json::to_vec(&decl).unwrap();
    line.push(b'\n');
    file.write_all(&line).unwrap();
}

fn write_policy(root: &Path, yaml: &str) {
    let dir = root.join("authority");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("active-policy.json"), yaml).unwrap();
}

const FULL_GRANT: &str = r#"
version: "0.1"
rules: []
policy_surfaces:
  self_disclosure:
    mode: deny-by-default
    grants:
      - name: full-grant
        actor_role: agent_test
        claim_classes:
          - declared_capability
          - installed_capability
          - demonstrated_capability
          - environment_status
          - authority_requirements
          - failure_condition
"#;

const DECLARED_ONLY_GRANT: &str = r#"
version: "0.1"
rules: []
policy_surfaces:
  self_disclosure:
    mode: deny-by-default
    grants:
      - name: declared-only
        actor_role: agent_test
        claim_classes:
          - declared_capability
"#;

const FRESH_REQUIRED_GRANT: &str = r#"
version: "0.1"
rules: []
policy_surfaces:
  self_disclosure:
    mode: deny-by-default
    grants:
      - name: fresh-required
        actor_role: agent_test
        claim_classes:
          - declared_capability
          - installed_capability
          - demonstrated_capability
        require_fresh_snapshot: true
"#;

#[test]
fn t14a_demonstrated_capability_reported_with_real_evidence() {
    let root = temp_root("demonstrated");
    init_self_model(&root, vec![]);
    let name = real_capability_name();
    append_qualifying_declaration(&root, &name);
    write_policy(&root, FULL_GRANT);

    let ans = ask(
        &root,
        ACTOR,
        QuestionKind::AskCapability,
        &name,
        "test",
        None,
    )
    .unwrap();

    assert_eq!(ans.disposition, Disposition::Answered);
    assert_eq!(ans.claims.len(), 1);
    assert!(
        ans.claims[0].status >= ClaimStatus::Validated,
        "real qualifying evidence must report at least Validated, got {:?}",
        ans.claims[0].status
    );
    assert!(!ans.claims[0].evidence_refs.is_empty());
    fs::remove_dir_all(root).ok();
}

#[test]
fn t14a_attempted_only_capability_never_reports_demonstrated() {
    let root = temp_root("attempted_only");
    init_self_model(&root, vec![]);
    let name = real_capability_name();
    // No declarations/envelopes appended: the capability is declared (it's a
    // real composed-model concept) but never attempted.
    write_policy(&root, FULL_GRANT);

    let ans = ask(
        &root,
        ACTOR,
        QuestionKind::AskCapability,
        &name,
        "test",
        None,
    )
    .unwrap();

    assert_eq!(ans.disposition, Disposition::Answered);
    assert_eq!(ans.claims.len(), 1);
    assert!(
        ans.claims[0].status < ClaimStatus::Validated,
        "no real evidence must never report Validated/Demonstrated, got {:?}",
        ans.claims[0].status
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn t14a_unavailable_environment_probe_is_disclosed() {
    let root = temp_root("unavailable_env");
    init_self_model(
        &root,
        vec![ToolchainProbe {
            tool: "some_toolchain".into(),
            required_by: "test".into(),
            probe_command_ref: "probe:some_toolchain".into(),
            result: ProbeResult::Unavailable,
            evidence_ref: Some("evd_probe_01".into()),
        }],
    );
    write_policy(&root, FULL_GRANT);

    let ans = ask(
        &root,
        ACTOR,
        QuestionKind::AskEnvironmentStatus,
        "some_toolchain",
        "test",
        None,
    )
    .unwrap();

    assert_eq!(ans.disposition, Disposition::Answered);
    assert_eq!(ans.claims.len(), 1);
    assert!(ans.claims[0].flags.unavailable_in_environment);
    fs::remove_dir_all(root).ok();
}

#[test]
fn t14a_absent_policy_denies() {
    let root = temp_root("absent_policy");
    init_self_model(&root, vec![]);
    let name = real_capability_name();

    let ans = ask(
        &root,
        ACTOR,
        QuestionKind::AskCapability,
        &name,
        "test",
        None,
    )
    .unwrap();

    assert_eq!(ans.disposition, Disposition::Denied);
    assert!(ans.claims.is_empty());
    fs::remove_dir_all(root).ok();
}

#[test]
fn t14a_partial_grant_caps_to_declared_only() {
    let root = temp_root("partial_grant");
    init_self_model(&root, vec![]);
    let name = real_capability_name();
    append_qualifying_declaration(&root, &name);
    write_policy(&root, DECLARED_ONLY_GRANT);

    let ans = ask(
        &root,
        ACTOR,
        QuestionKind::AskCapability,
        &name,
        "test",
        None,
    )
    .unwrap();

    assert_eq!(ans.disposition, Disposition::Partial);
    assert_eq!(ans.claims.len(), 1);
    assert_eq!(ans.claims[0].claim_class, ClaimClass::DeclaredCapability);
    assert!(ans.claims[0].status <= ClaimStatus::Declared);
    fs::remove_dir_all(root).ok();
}

#[test]
fn t14a_stale_required_refuses_before_any_query() {
    let root = temp_root("stale_required");
    init_self_model(&root, vec![]);
    let name = real_capability_name();
    append_qualifying_declaration(&root, &name);
    write_policy(&root, FRESH_REQUIRED_GRANT);
    store::mark_current_stale(&root, "test_forced_stale").unwrap();

    let ans = ask(
        &root,
        ACTOR,
        QuestionKind::AskCapability,
        &name,
        "test",
        None,
    )
    .unwrap();

    assert_eq!(ans.disposition, Disposition::Denied);
    assert_eq!(ans.freshness, Freshness::Stale);
    assert!(ans.claims.is_empty());
    fs::remove_dir_all(root).ok();
}

#[test]
fn t14a_replay_is_stable_and_ledgered() {
    let root = temp_root("replay");
    init_self_model(&root, vec![]);
    let name = real_capability_name();
    append_qualifying_declaration(&root, &name);
    write_policy(&root, FULL_GRANT);

    let a1 = ask(
        &root,
        ACTOR,
        QuestionKind::AskCapability,
        &name,
        "test",
        None,
    )
    .unwrap();
    let a2 = ask(
        &root,
        ACTOR,
        QuestionKind::AskCapability,
        &name,
        "test",
        None,
    )
    .unwrap();

    assert_eq!(a1.disposition, a2.disposition);
    assert_eq!(a1.claims, a2.claims);
    fs::remove_dir_all(root).ok();
}

const ROLE_BOUND_GRANT: &str = r#"
version: "0.1"
rules: []
identity_bindings:
  - principal: agent_test
    actor_type: human
    role: R-SO
policy_surfaces:
  self_disclosure:
    mode: deny-by-default
    grants:
      - name: so-role-grant
        actor_role: R-SO
        claim_classes:
          - declared_capability
"#;

// F-23 regression: a grant authored for a ROLE must permit an actor the
// bundle binds to that role. Before the fix the asker's *id* was matched
// against `grant.actor_role`, so this lawfully granted disclosure was denied.
#[test]
fn t23_role_authored_grant_permits_a_bound_actor() {
    let root = temp_root("role_bound");
    init_self_model(&root, vec![]);
    let name = real_capability_name();
    write_policy(&root, ROLE_BOUND_GRANT);

    let ans = ask(
        &root,
        ACTOR,
        QuestionKind::AskCapability,
        &name,
        "test",
        None,
    )
    .unwrap();

    // The role matched, so the granted class discloses; classes outside the
    // grant stay withheld, which is exactly `Partial`, not `Answered`.
    assert_eq!(ans.disposition, Disposition::Partial);
    assert!(!ans.claims.is_empty(), "the granted class must disclose");
    assert!(
        !ans.omitted_claim_classes.is_empty(),
        "classes outside the grant must stay withheld"
    );
    fs::remove_dir_all(root).ok();
}

// F-23 flip side: the role-authored grant must NOT leak to an actor the
// bundle does not bind to that role — deny-by-default still governs.
#[test]
fn t23_unbound_actor_cannot_use_another_principals_role_grant() {
    let root = temp_root("role_unbound");
    init_self_model(&root, vec![]);
    let name = real_capability_name();
    write_policy(&root, ROLE_BOUND_GRANT);

    let ans = ask(
        &root,
        "someone_else",
        QuestionKind::AskCapability,
        &name,
        "test",
        None,
    )
    .unwrap();

    assert_eq!(ans.disposition, Disposition::Denied);
    fs::remove_dir_all(root).ok();
}
