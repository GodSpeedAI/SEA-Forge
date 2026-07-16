use std::{collections::BTreeSet, fs, process::Command};

#[test]
fn produced_semantic_envelope_has_documented_cep0008_flat_profile_divergence() {
    let schema: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/cep/semantic-envelope.schema.json")).unwrap();
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("state");
    let policy = temp.path().join("policy.yaml");
    fs::write(
        &policy,
        "version: \"0.1\"\nidentity:\n  source: task17\n  allow_unresolved: false\nrules:\n  - name: allow-write\n    verdict: allow\n    actor_role: operator\n    operation_kind: write_file\n    path_prefix: \"\"\n  - name: allow-validate\n    verdict: allow\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "run",
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            "--intent",
            "Generate and validate a simple DomainForge .sea model",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let run_id = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .find_map(|line| line.strip_prefix("run_id="))
        .unwrap()
        .to_owned();
    let envelope: serde_json::Value = serde_json::from_slice(
        &fs::read(
            root.join("runs")
                .join(run_id)
                .join("semantic-envelope.json"),
        )
        .unwrap(),
    )
    .unwrap();

    let required: BTreeSet<&str> = schema["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|field| field.as_str().unwrap())
        .collect();
    let allowed: BTreeSet<&str> = schema["properties"]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    let actual: BTreeSet<&str> = envelope
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();

    let missing: Vec<&str> = required.difference(&actual).copied().collect();
    let additional: Vec<&str> = actual.difference(&allowed).copied().collect();
    assert_eq!(
        missing,
        [
            "event_id",
            "occurred_at",
            "payload",
            "provenance",
            "schema_version",
            "source_agent"
        ]
    );
    assert_eq!(
        additional,
        [
            "artifact_refs",
            "attribution",
            "authority_decisions",
            "capability_delta",
            "case_ref",
            "cell_id",
            "evidence_refs",
            "extension_refs",
            "intent",
            "plan_ref",
            "projection_refs",
            "run_id",
            "settlement_ref",
            "version"
        ]
    );
}
