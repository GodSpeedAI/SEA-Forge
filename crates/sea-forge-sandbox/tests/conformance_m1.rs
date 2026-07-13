use sea_forge_core::types::{ExecutionRequest, ExecutionStatus, Operation};
use sea_forge_sandbox::{ExecutionSandbox, JailSandbox, SandboxClass, SandboxSpec};
use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_dir(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("sea-forge-m1-{name}-{nonce}"));
    fs::create_dir_all(&path).unwrap();
    path
}

fn request(argv: Vec<String>) -> ExecutionRequest {
    let mut env = BTreeMap::new();
    if let Ok(path) = std::env::var("PATH") {
        env.insert("PATH".into(), path);
    }
    env.insert("HOME".into(), "/tmp".into());
    ExecutionRequest {
        plan_item_id: "item_01".into(),
        operation: Operation::ExecuteCommand {
            argv,
            cwd: ".".into(),
        },
        timeout_secs: 10,
        env,
        compensating_controls: vec![],
    }
}

#[test]
fn jail_blocks_write_outside_workspace() {
    let parent = temp_dir("escape");
    let workspace = parent.join("workspace");
    let artifacts = parent.join("artifacts");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&artifacts).unwrap();

    let jail = match JailSandbox::new() {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Skipping jail test: {e}");
            return;
        }
    };
    let spec = SandboxSpec {
        workspace_root: workspace.clone(),
        artifacts_root: artifacts.clone(),
    };
    let handle = jail.prepare(&spec).unwrap();
    // Try to write outside the workspace via a relative path escape.
    let result = jail
        .execute(
            &handle,
            &request(vec![
                "sh".into(),
                "-c".into(),
                "echo escaped > ../outside.txt".into(),
            ]),
        )
        .unwrap();
    let _ = jail.destroy(handle);

    // The write must have been blocked by the OS.
    assert!(
        !parent.join("outside.txt").exists(),
        "jail failed to block write outside workspace"
    );
    // The command must not have succeeded normally.
    assert_ne!(
        result.status,
        ExecutionStatus::Completed,
        "sandbox violation should not be a normal completion"
    );

    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn jail_class_is_jail() {
    let jail = match JailSandbox::new() {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Skipping jail test: {e}");
            return;
        }
    };
    assert_eq!(jail.class(), SandboxClass::Jail);
}

#[test]
fn jail_unavailable_on_unsupported_platform_returns_error() {
    // On non-Linux, JailSandbox::new() must return an error.
    #[cfg(not(target_os = "linux"))]
    {
        let result = JailSandbox::new();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().class, "unsupported_sandbox_class_error");
    }
    // On Linux, if Landlock is not supported by the kernel, the error is
    // detected at execute time rather than at construction time.
    #[cfg(target_os = "linux")]
    {
        let _ = JailSandbox::new();
    }
}

#[test]
fn local_class_is_local() {
    let local = sea_forge_sandbox::LocalSandbox;
    assert_eq!(local.class(), SandboxClass::Local);
}

#[test]
fn untrusted_argv0_on_local_is_schema_error() {
    // Re-assert from Task 5: a 0.2 policy granting `local` to an untrusted
    // argv0 is a schema_error.
    use sea_forge_authority::AuthorityPolicyBundle;
    use sea_forge_core::RECORD_VERSION;

    let tmp = temp_dir("schema");
    let sources_dir = tmp.join("sources");
    fs::create_dir_all(&sources_dir).unwrap();

    const SURFACES: &[&str] = &[
        "authority_hooks",
        "identity_map",
        "domain_model",
        "file_access",
        "api_allowlist",
        "git_commit",
        "pr_merge",
        "prompt_risk",
        "memory_recall",
        "spec_pipeline",
        "artifact_transition",
        "attestation",
        "approval",
        "settlement_authority",
        "capability_promotion",
        "deployment",
        "secret_access",
        "policy_mutation",
        "evidence_mutation",
    ];
    const ROLES: &[&str] = &["R-DS", "R-AG", "R-LC", "R-SO", "R-RM", "R-DEV", "R-AA"];
    const SOD: &[(&str, &str)] = &[
        ("production_proposer_approver", "pr_merge"),
        (
            "semantic_debt_requester_acceptor",
            "settlement_authority_mutation",
        ),
        ("break_glass_requester_approver", "policy_mutation"),
        ("key_generator_approver", "identity_minting"),
        ("capitalization_requester_approver", "artifact_transition"),
    ];

    let sources: Vec<serde_json::Value> = SURFACES
        .iter()
        .map(|surface| {
            fs::write(sources_dir.join(surface), "").unwrap();
            serde_json::json!({
                "surface": surface,
                "path": format!("sources/{surface}"),
                "sha256": sea_forge_evidence::sha256_bytes(b"")
            })
        })
        .collect();

    let mut bundle: AuthorityPolicyBundle = serde_json::from_value(serde_json::json!({
        "version": RECORD_VERSION,
        "bundle_id": "test_schema",
        "policy_bundle_hash": "",
        "policy_bundle_version": "1",
        "loaded_at": "2026-07-13T00:00:00Z",
        "source_base": tmp.to_string_lossy(),
        "sources": sources,
        "roles": ROLES.iter().map(|r| ((*r).to_owned(), vec!["fixture"])).collect::<std::collections::BTreeMap<_, _>>(),
        "permissions": ROLES.iter().map(|r| serde_json::json!({"role": r, "operation_kind": "*"})).collect::<Vec<_>>(),
        "sod_rules": SOD.iter().map(|(n, op)| serde_json::json!({"name": n, "requester_role": "R-DEV", "approver_role": "R-SO", "operation_kind": op})).collect::<Vec<_>>(),
        "identity_bindings": [{"principal": "operator_local", "actor_type": "human", "role": "operator"}],
        "policy_engines": [],
        "rules": [
            {"name": "allow-untrusted-local", "verdict": "allow", "actor_role": "operator", "operation_kind": "execute_command", "argv0": "untrusted-binary", "sandbox_class": "local"},
        ],
    })).unwrap();
    bundle.refresh_policy_bundle_hash().unwrap();

    let path = tmp.join("policy.json");
    fs::write(&path, serde_json::to_vec_pretty(&bundle).unwrap()).unwrap();

    let result = AuthorityPolicyBundle::load(&path);
    assert!(
        result.is_err(),
        "policy granting local to untrusted argv0 must be rejected"
    );
    let err = result.unwrap_err();
    assert_eq!(err.class(), "schema_error");

    fs::remove_dir_all(tmp).unwrap();
}
