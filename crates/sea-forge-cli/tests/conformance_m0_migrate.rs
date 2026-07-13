use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_root(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "sea-forge-migrate-{name}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

fn v01_policy(parent: &std::path::Path) -> PathBuf {
    let path = parent.join("policy.yaml");
    fs::write(
        &path,
        "version: \"0.1\"\nidentity:\n  source: test-binding\n  allow_unresolved: false\nrules:\n  - name: allow-model-write\n    verdict: allow\n    actor_role: operator\n    operation_kind: write_file\n    path_prefix: \"\"\n  - name: allow-self-validate\n    verdict: allow\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n",
    )
    .unwrap();
    path
}

fn run_v01_fixture(root: &std::path::Path, policy: &std::path::Path, intent: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "run",
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            "--intent",
            intent,
        ])
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    stdout
        .lines()
        .find_map(|l| l.strip_prefix("run_id="))
        .unwrap()
        .to_string()
}

fn collect_files_recursive(dir: &std::path::Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursive(&path, files);
        } else if path.is_file() {
            files.push(path);
        }
    }
}

#[test]
fn conformance_m0_migrate_imports_v01_root_losslessly_and_blocks_re_migration() {
    let parent = temp_root("v01");
    let root = parent.join("state");
    let policy = v01_policy(&parent);
    let run_id = run_v01_fixture(
        &root,
        &policy,
        "Generate and validate a simple DomainForge .sea model",
    );

    let envelope: serde_json::Value = serde_json::from_slice(
        &fs::read(
            root.join("runs")
                .join(&run_id)
                .join("semantic-envelope.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let case_id = envelope["case_ref"].as_str().unwrap();

    // Enumerate all v0.1 files (relative to root) before migration.
    let mut raw_files: Vec<PathBuf> = Vec::new();
    collect_files_recursive(&root, &mut raw_files);
    let mut original_files: Vec<String> = raw_files
        .into_iter()
        .map(|p| p.strip_prefix(&root).unwrap().to_str().unwrap().to_string())
        .filter(|rel_str| {
            rel_str != "migration.json"
                && !rel_str.starts_with("ledgers/")
                && !rel_str.starts_with("quarantine/")
                && !rel_str.starts_with("authority/")
                && rel_str != "capabilities.jsonl"
        })
        .collect();
    original_files.sort();

    let migrate = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args(["migrate", "--root", root.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        migrate.status.success(),
        "migrate stderr: {}",
        String::from_utf8_lossy(&migrate.stderr)
    );
    let stdout = String::from_utf8(migrate.stdout).unwrap();
    assert!(stdout.contains("migrated"));
    assert!(stdout.contains("stream=legacy-import"));

    assert!(root.join("migration.json").is_file());

    // Every legacy file has a ledger entry.
    let entries_path = root
        .join("ledgers")
        .join("legacy-import")
        .join("entries.jsonl");
    let entries: Vec<serde_json::Value> = fs::read_to_string(&entries_path)
        .unwrap()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(entries.len(), original_files.len());
    for entry in &entries {
        assert_eq!(entry["record_kind"], "legacy_import");
        assert!(entry["payload"]["path"].as_str().is_some());
        assert!(entry["payload"]["sha256"].as_str().is_some());
        assert!(entry["payload"]["size_bytes"].as_u64().is_some());
    }

    // Run directory relocated under its case.
    let migrated_run = root.join("cases").join(case_id).join("runs").join(&run_id);
    assert!(migrated_run.is_dir());
    assert!(migrated_run.join("plan.json").is_file());
    assert!(migrated_run.join("settlement.json").is_file());

    // Case file relocated into case directory.
    let migrated_case = root.join("cases").join(case_id).join("case.json");
    assert!(migrated_case.is_file());
    let case: serde_json::Value =
        serde_json::from_slice(&fs::read(&migrated_case).unwrap()).unwrap();
    assert_eq!(case["case_id"], case_id);
    assert_eq!(case["run_ids"], serde_json::json!([run_id]));

    // Case-level event log created.
    assert!(root
        .join("cases")
        .join(case_id)
        .join("case-events.jsonl")
        .is_file());

    // Ledger verify passes.
    let verify = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "ledger",
            "--root",
            root.to_str().unwrap(),
            "verify",
            "legacy-import",
        ])
        .output()
        .unwrap();
    assert!(
        verify.status.success(),
        "verify stderr: {}",
        String::from_utf8_lossy(&verify.stderr)
    );

    // Re-migration is refused.
    let repeat = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args(["migrate", "--root", root.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!repeat.status.success());
    assert!(String::from_utf8_lossy(&repeat.stderr).contains("refusing to re-migrate"));

    // Corrupting a migrated file breaks ledger verification.
    let plan = migrated_run.join("plan.json");
    let original = fs::read_to_string(&plan).unwrap();
    fs::write(&plan, original.replace("plan_id", "PLAN_ID")).unwrap();
    let verify_corrupt = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "ledger",
            "--root",
            root.to_str().unwrap(),
            "verify",
            "legacy-import",
        ])
        .output()
        .unwrap();
    assert!(!verify_corrupt.status.success());
    let verify_stdout = String::from_utf8_lossy(&verify_corrupt.stdout);
    assert!(
        verify_stdout.contains("ledger_integrity_error")
            || verify_stdout.contains("sha256 mismatch")
            || verify_stdout.contains("size mismatch"),
        "stdout: {verify_stdout}"
    );

    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn conformance_m0_migrate_inspect_reports_legacy_digest_only() {
    let parent = temp_root("inspect-migrated");
    let root = parent.join("state");
    let policy = v01_policy(&parent);
    let run_id = run_v01_fixture(
        &root,
        &policy,
        "Generate and validate a simple DomainForge .sea model",
    );

    let migrate = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args(["migrate", "--root", root.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(migrate.status.success());

    let inspect = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args(["inspect", &run_id, "--root", root.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        inspect.status.success(),
        "inspect stderr: {}",
        String::from_utf8_lossy(&inspect.stderr)
    );
    let inspected = String::from_utf8(inspect.stdout).unwrap();
    assert!(inspected.contains("legacy_digest_only"));

    fs::remove_dir_all(parent).unwrap();
}
