//! Sandbox-command authority: `execute_sandbox_command` is a typed action that
//! is authorized only for its own rule and only for an OpenShell sandbox. It
//! never widens host command execution, and host commands never reach it.

use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::errors::ForgeError;
use sea_forge_core::types::{Actor, ActorRole, AuthorityAction, AuthorityDecision, Verdict};
use std::path::Path;

const CELL: &str = "cell-alpha";
const MODEL: &str = "sha256:model-1";
const POLICY: &str = "sha256:policy-1";
const IMAGE: &str = "registry.example/cell:1";

/// The four binding dimensions every sandbox-command rule must pin.
const FULL_BINDINGS: [(&str, &str); 4] = [
    ("cell_id", "\"cell-alpha\""),
    ("model_hash", "\"sha256:model-1\""),
    ("openshell_policy_digest", "\"sha256:policy-1\""),
    ("sandbox_image", "\"registry.example/cell:1\""),
];

/// Renders a policy with one rule. `bindings` holds raw YAML flow-sequence
/// contents, so a test can omit, duplicate or blank a value.
fn policy_with(
    operation_kind: &str,
    sandbox_class: &str,
    argv0: Option<&str>,
    bindings: &[(&str, &str)],
) -> String {
    let argv0_line = argv0
        .map(|argv0| format!("    argv0: \"{argv0}\"\n"))
        .unwrap_or_default();
    let mut yaml = format!(
        "version: \"0.1\"\nrules:\n  - name: sandbox-shell\n    verdict: allow\n    actor_role: operator\n    operation_kind: {operation_kind}\n    sandbox_class: {sandbox_class}\n{argv0_line}    boundary_constraints:\n"
    );
    if bindings.is_empty() {
        yaml.push_str("      {}\n");
    }
    for (dimension, values) in bindings {
        yaml.push_str(&format!("      {dimension}: [{values}]\n"));
    }
    yaml
}

/// The valid sandbox-command policy: `/bin/sh` inside the pinned Cell.
fn valid_policy() -> String {
    policy_with(
        "execute_sandbox_command",
        "openshell",
        Some("/bin/sh"),
        &FULL_BINDINGS,
    )
}

fn bundle_of(yaml: &str) -> AuthorityPolicyBundle {
    sea_forge_core::yaml::from_str(yaml).unwrap()
}

fn sandbox(
    argv: &[&str],
    cwd: &str,
    cell_id: &str,
    model_hash: &str,
    policy_digest: &str,
    image: &str,
) -> AuthorityAction {
    AuthorityAction::ExecuteSandboxCommand {
        argv: argv.iter().map(|arg| arg.to_string()).collect(),
        cwd: cwd.into(),
        cell_id: cell_id.into(),
        model_hash: model_hash.into(),
        policy_digest: policy_digest.into(),
        image: image.into(),
    }
}

/// A sandbox command that matches the valid policy exactly.
fn matching(argv: &[&str]) -> AuthorityAction {
    sandbox(argv, ".", CELL, MODEL, POLICY, IMAGE)
}

fn decide(
    engine: &PolicyAuthorityEngine,
    bundle: &AuthorityPolicyBundle,
    action: &AuthorityAction,
) -> AuthorityDecision {
    let actor = Actor {
        actor_id: "operator_local".into(),
        role: ActorRole::Operator,
    };
    let binding = bundle.resolve_identity(&actor.actor_id, actor.role.clone());
    engine
        .evaluate(AuthorityEvaluation {
            actor: &actor,
            binding,
            run_id: "run_sandbox_command",
            case_id: "case_sandbox_command",
            plan_item_id: "item_sandbox_command",
            sequence: 1,
            action,
            workspace_root: Path::new("/tmp/workspace"),
            evidence_refs: vec!["intent:sandbox_command".into()],
            artifacts_root: None,
            timeout_secs: None,
            env_keys: Default::default(),
            domainforge_candidate: None,
            opa_candidate: None,
            environment: None,
        })
        .unwrap()
}

fn load_error(yaml: &str) -> ForgeError {
    PolicyAuthorityEngine::new(bundle_of(yaml))
        .err()
        .expect("policy must fail validation")
}

#[test]
fn host_command_restriction_is_unchanged() {
    // The only rule present is the sandbox rule for /bin/sh. A host command
    // with the same argv0 must still be refused by the host executable gate.
    let yaml = valid_policy();
    let bundle = bundle_of(&yaml);
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let action = AuthorityAction::ExecuteCommand {
        argv: vec!["/bin/sh".into(), "-c".into(), "true".into()],
        cwd: ".".into(),
    };

    let decision = decide(&engine, &bundle, &action);

    assert_eq!(decision.verdict, Verdict::Deny);
    assert!(decision
        .reason_codes
        .iter()
        .any(|code| code == "untrusted_executable"));
    assert_ne!(decision.sandbox_class_granted.as_deref(), Some("openshell"));
}

#[test]
fn sandbox_class_on_a_host_command_rule_does_not_load() {
    let yaml = policy_with(
        "execute_command",
        "openshell",
        Some("/bin/sh"),
        &FULL_BINDINGS,
    );

    let error = load_error(&yaml);

    assert_eq!(error.class(), "schema_error");
    assert!(
        error
            .to_string()
            .contains("sandbox class openshell applies only to execute_sandbox_command"),
        "unexpected message: {error}"
    );
}

#[test]
fn sandbox_command_rule_requires_every_binding() {
    for (dimension, _) in FULL_BINDINGS {
        let bindings: Vec<(&str, &str)> = FULL_BINDINGS
            .into_iter()
            .filter(|(name, _)| *name != dimension)
            .collect();
        let yaml = policy_with(
            "execute_sandbox_command",
            "openshell",
            Some("/bin/sh"),
            &bindings,
        );
        let error = load_error(&yaml);
        assert_eq!(error.class(), "schema_error", "missing {dimension}");
    }

    // Two values for one dimension.
    let mut two_values = FULL_BINDINGS.to_vec();
    two_values[0] = ("cell_id", "\"cell-alpha\", \"cell-beta\"");
    let yaml = policy_with(
        "execute_sandbox_command",
        "openshell",
        Some("/bin/sh"),
        &two_values,
    );
    assert_eq!(
        load_error(&yaml).class(),
        "schema_error",
        "two cell_id values"
    );

    // An empty value for one dimension.
    let mut empty_value = FULL_BINDINGS.to_vec();
    empty_value[3] = ("sandbox_image", "\"\"");
    let yaml = policy_with(
        "execute_sandbox_command",
        "openshell",
        Some("/bin/sh"),
        &empty_value,
    );
    assert_eq!(load_error(&yaml).class(), "schema_error", "empty image");

    // No argv0.
    let yaml = policy_with("execute_sandbox_command", "openshell", None, &FULL_BINDINGS);
    assert_eq!(load_error(&yaml).class(), "schema_error", "no argv0");

    // A sandbox command rule that names a class other than openshell.
    let yaml = policy_with(
        "execute_sandbox_command",
        "jail",
        Some("/bin/sh"),
        &FULL_BINDINGS,
    );
    assert_eq!(load_error(&yaml).class(), "schema_error", "jail class");

    // The valid policy still loads.
    PolicyAuthorityEngine::new(bundle_of(&valid_policy())).unwrap();
}

#[test]
fn matching_sandbox_command_is_allowed_for_openshell_only() {
    let yaml = valid_policy();
    let bundle = bundle_of(&yaml);
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();

    let decision = decide(&engine, &bundle, &matching(&["/bin/sh", "-c", "true"]));

    assert_eq!(decision.verdict, Verdict::Allow);
    assert_eq!(decision.sandbox_class_granted.as_deref(), Some("openshell"));
    assert_eq!(
        decision.boundary_constraints.get("cell_id"),
        Some(&vec![CELL.to_string()])
    );
    assert_eq!(
        decision.boundary_constraints.get("model_hash"),
        Some(&vec![MODEL.to_string()])
    );
    assert_eq!(
        decision.boundary_constraints.get("openshell_policy_digest"),
        Some(&vec![POLICY.to_string()])
    );
    assert_eq!(
        decision.boundary_constraints.get("sandbox_image"),
        Some(&vec![IMAGE.to_string()])
    );
}

#[test]
fn mismatched_sandbox_command_is_denied() {
    let yaml = valid_policy();
    let bundle = bundle_of(&yaml);
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let cases = [
        (
            "cell_id",
            sandbox(
                &["/bin/sh", "-c", "true"],
                ".",
                "cell-beta",
                MODEL,
                POLICY,
                IMAGE,
            ),
        ),
        (
            "model_hash",
            sandbox(
                &["/bin/sh", "-c", "true"],
                ".",
                CELL,
                "sha256:model-2",
                POLICY,
                IMAGE,
            ),
        ),
        (
            "policy_digest",
            sandbox(
                &["/bin/sh", "-c", "true"],
                ".",
                CELL,
                MODEL,
                "sha256:policy-2",
                IMAGE,
            ),
        ),
        (
            "image",
            sandbox(
                &["/bin/sh", "-c", "true"],
                ".",
                CELL,
                MODEL,
                POLICY,
                "registry.example/other:2",
            ),
        ),
        (
            "argv0 /bin/bash",
            sandbox(
                &["/bin/bash", "-c", "true"],
                ".",
                CELL,
                MODEL,
                POLICY,
                IMAGE,
            ),
        ),
        (
            "argv0 sh without its path",
            sandbox(&["sh", "-c", "true"], ".", CELL, MODEL, POLICY, IMAGE),
        ),
    ];

    for (label, action) in &cases {
        let decision = decide(&engine, &bundle, action);
        assert_eq!(
            decision.verdict,
            Verdict::Deny,
            "{label} must not be allowed"
        );
        assert_ne!(
            decision.sandbox_class_granted.as_deref(),
            Some("openshell"),
            "{label} must not receive an openshell grant"
        );
    }
}

#[test]
fn malformed_sandbox_command_is_denied() {
    let yaml = valid_policy();
    let bundle = bundle_of(&yaml);
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let cases = [
        ("empty argv", matching(&[])),
        ("empty argv[0]", matching(&["", "-c", "true"])),
        (
            "blank cell_id",
            sandbox(&["/bin/sh", "-c", "true"], ".", "", MODEL, POLICY, IMAGE),
        ),
        (
            "image with trailing space",
            sandbox(
                &["/bin/sh", "-c", "true"],
                ".",
                CELL,
                MODEL,
                POLICY,
                "registry.example/cell:1 ",
            ),
        ),
        (
            "parent-relative cwd",
            sandbox(
                &["/bin/sh", "-c", "true"],
                "../x",
                CELL,
                MODEL,
                POLICY,
                IMAGE,
            ),
        ),
        (
            "absolute cwd",
            sandbox(
                &["/bin/sh", "-c", "true"],
                "/abs",
                CELL,
                MODEL,
                POLICY,
                IMAGE,
            ),
        ),
    ];

    for (label, action) in &cases {
        let decision = decide(&engine, &bundle, action);
        assert_eq!(decision.verdict, Verdict::Deny, "{label} must be denied");
    }
}

#[test]
fn host_command_never_matches_a_sandbox_rule() {
    let yaml = valid_policy();
    let bundle = bundle_of(&yaml);
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let exe = std::env::current_exe()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let action = AuthorityAction::ExecuteCommand {
        argv: vec![exe],
        cwd: ".".into(),
    };

    let decision = decide(&engine, &bundle, &action);

    assert_ne!(decision.verdict, Verdict::Allow);
    assert_ne!(decision.sandbox_class_granted.as_deref(), Some("openshell"));
}
