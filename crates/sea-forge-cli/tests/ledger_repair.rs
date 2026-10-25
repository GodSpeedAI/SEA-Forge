//! `sea-forge ledger verify` (read-only, hints at repair for crash residue) and `sea-forge ledger repair`
//! (offline, refuses damage, exits non-zero when verify still fails).

use sea_forge_ledger::types::LedgerStream;
use serde_json::json;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::tempdir;

fn cli(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .arg("ledger")
        .arg("--root")
        .arg(root)
        .args(args)
        .output()
        .unwrap()
}

fn seeded(root: &Path) -> LedgerStream {
    let stream = LedgerStream::open(root, "main", "cli-test").unwrap();
    for i in 0..3 {
        stream
            .append("test", vec![], json!({"n": i}), vec![])
            .unwrap();
    }
    stream
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn crash_residue_fails_verify_with_the_repair_hint_then_repair_verifies() {
    let tmp = tempdir().unwrap();
    let stream = seeded(tmp.path());
    let entries = stream.entries_path();
    let mut file = fs::OpenOptions::new().append(true).open(&entries).unwrap();
    file.write_all(b"{\"entry_ulid\":\"half").unwrap();
    drop(file);

    let verify = cli(tmp.path(), &["verify", "main"]);
    assert_eq!(verify.status.code(), Some(1));
    let out = stdout(&verify);
    assert!(out.contains("VERIFICATION FAILED"), "{out}");
    assert!(out.contains("crash residue"), "{out}");
    assert!(
        out.contains("ledger --root") && out.contains("repair main"),
        "{out}"
    );
    // Verify is read-only: the torn bytes are still there.
    assert!(fs::read_to_string(&entries).unwrap().ends_with("half"));

    let repair = cli(tmp.path(), &["repair", "main"]);
    assert_eq!(repair.status.code(), Some(0), "{}", stdout(&repair));
    let out = stdout(&repair);
    assert!(out.contains("quarantined 19 bytes (1 record(s))"), "{out}");
    assert!(out.contains("verified"), "{out}");

    let verify = cli(tmp.path(), &["verify", "main"]);
    assert_eq!(verify.status.code(), Some(0), "{}", stdout(&verify));
}

#[test]
fn a_tampered_complete_entry_is_never_repaired_and_verify_gets_no_hint() {
    let tmp = tempdir().unwrap();
    let stream = seeded(tmp.path());
    let entries = stream.entries_path();
    let content = fs::read_to_string(&entries).unwrap();
    fs::write(&entries, content.replacen("\"n\":1}", "\"n\":99}", 1)).unwrap();
    let before = fs::read(&entries).unwrap();

    let verify = cli(tmp.path(), &["verify", "main"]);
    assert_eq!(verify.status.code(), Some(1));
    assert!(!stdout(&verify).contains("hint:"), "{}", stdout(&verify));

    let repair = cli(tmp.path(), &["repair", "main"]);
    assert_ne!(repair.status.code(), Some(0), "{}", stdout(&repair));
    assert_eq!(fs::read(&entries).unwrap(), before);
}
