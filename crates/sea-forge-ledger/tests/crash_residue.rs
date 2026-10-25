//! Crash residue versus damage. A crash mid-append may leave one torn final record (or an MMR cache
//! behind complete entries); that is repairable offline. Corruption before the tail, or a tampered
//! complete entry, is never truncated or "repaired": repair refuses and writes nothing.

use sea_forge_ledger::types::LedgerStream;
use serde_json::json;
use std::fs;
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;

fn seeded(root: &Path, count: u64) -> LedgerStream {
    let stream = LedgerStream::open(root, "main", "w1").unwrap();
    for i in 0..count {
        stream
            .append("test", vec![], json!({"n": i}), vec![])
            .unwrap();
    }
    stream
}

fn append_raw(stream: &LedgerStream, bytes: &[u8]) {
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(stream.entries_path())
        .unwrap();
    file.write_all(bytes).unwrap();
}

fn quarantine_dir(root: &Path) -> std::path::PathBuf {
    root.join("ledgers").join("quarantine")
}

fn quarantine_files(root: &Path) -> usize {
    fs::read_dir(quarantine_dir(root)).unwrap().count()
}

#[test]
fn torn_final_record_is_residue_repaired_offline_and_verifies_after() {
    let tmp = tempdir().unwrap();
    let stream = seeded(tmp.path(), 3);
    let torn = b"{\"entry_ulid\":\"half";
    append_raw(&stream, torn);

    assert!(stream.verify().is_err());
    let residue = stream.crash_residue().unwrap();
    assert_eq!(residue.torn_bytes, torn.len());
    assert_eq!(residue.torn_records, 1);
    assert!(!residue.mmr_stale);
    assert!(residue.is_repairable());

    let repair = stream.repair_tail().unwrap();
    assert_eq!(repair.kept_records, 3);
    assert_eq!(repair.quarantined_bytes, torn.len());
    assert_eq!(repair.quarantined_records, 1);
    let q_path = repair.quarantine_path.expect("residue is quarantined");
    assert_eq!(fs::read(&q_path).unwrap(), torn);
    assert_eq!(stream.read_entries().unwrap().len(), 3);
    stream.verify().unwrap();

    // Idempotent: a second repair finds nothing to do.
    let again = stream.repair_tail().unwrap();
    assert_eq!(again.quarantined_bytes, 0);
    assert!(!again.mmr_rebuilt);
    stream.verify().unwrap();
}

#[test]
fn complete_record_missing_only_its_newline_is_the_one_residue_record() {
    let tmp = tempdir().unwrap();
    let stream = seeded(tmp.path(), 2);
    // Remove the newline of the last record: the record is complete but unterminated.
    let content = fs::read(stream.entries_path()).unwrap();
    fs::write(stream.entries_path(), &content[..content.len() - 1]).unwrap();

    let residue = stream.crash_residue().unwrap();
    assert_eq!(residue.torn_records, 1);
    let repair = stream.repair_tail().unwrap();
    assert_eq!(repair.quarantined_records, 1);
    assert_eq!(stream.read_entries().unwrap().len(), 1);
    stream.verify().unwrap();
}

#[test]
fn mmr_behind_complete_entries_is_rebuilt_and_nothing_is_quarantined() {
    let tmp = tempdir().unwrap();
    let stream = seeded(tmp.path(), 2);
    let mmr_path = tmp.path().join("ledgers/main/mmr.json");
    let stale = fs::read(&mmr_path).unwrap();
    stream
        .append("test", vec![], json!({"n": 2}), vec![])
        .unwrap();
    // Crash window: the entry was written, the cache still reflects two entries.
    fs::write(&mmr_path, &stale).unwrap();

    assert!(stream.verify().is_err());
    let residue = stream.crash_residue().unwrap();
    assert_eq!(residue.torn_bytes, 0);
    assert!(residue.mmr_stale);
    assert!(residue.is_repairable());

    let repair = stream.repair_tail().unwrap();
    assert!(repair.mmr_rebuilt);
    assert_eq!(repair.quarantined_bytes, 0);
    assert_eq!(quarantine_files(tmp.path()), 0);
    stream.verify().unwrap();
}

#[test]
fn clean_ledger_has_no_residue_and_repair_changes_nothing() {
    let tmp = tempdir().unwrap();
    let stream = seeded(tmp.path(), 3);
    let before = fs::read(stream.entries_path()).unwrap();

    stream.verify().unwrap();
    let residue = stream.crash_residue().unwrap();
    assert!(!residue.is_repairable());
    let repair = stream.repair_tail().unwrap();
    assert_eq!(repair.quarantined_bytes, 0);
    assert!(!repair.mmr_rebuilt);
    assert_eq!(fs::read(stream.entries_path()).unwrap(), before);
}

#[test]
fn corruption_before_the_tail_is_refused_and_never_truncated() {
    let tmp = tempdir().unwrap();
    let stream = seeded(tmp.path(), 3);
    let first_line = fs::read(stream.entries_path())
        .unwrap()
        .split(|b| *b == b'\n')
        .next()
        .unwrap()
        .to_vec();
    // A corrupt record, then more complete data: this is not an interrupted append.
    append_raw(&stream, b"{garbage\n");
    append_raw(&stream, &first_line);
    append_raw(&stream, b"\n");
    let before = fs::read(stream.entries_path()).unwrap();

    let err = stream.repair_tail().unwrap_err().to_string();
    assert!(
        err.contains("damaged beyond an interrupted append"),
        "{err}"
    );
    assert!(stream.crash_residue().is_err());
    assert!(stream.quarantine_incomplete_tail().is_err());
    assert_eq!(fs::read(stream.entries_path()).unwrap(), before);
    assert_eq!(quarantine_files(tmp.path()), 0);
}

#[test]
fn newline_terminated_corrupt_line_is_refused_and_never_truncated() {
    let tmp = tempdir().unwrap();
    let stream = seeded(tmp.path(), 3);
    // An append writes the entry and its newline in one call, so a torn append cannot end in a
    // newline: a complete-looking corrupt line is damage, not crash residue.
    append_raw(&stream, b"{garbage\n");
    let before = fs::read(stream.entries_path()).unwrap();

    let err = stream.repair_tail().unwrap_err().to_string();
    assert!(
        err.contains("damaged beyond an interrupted append"),
        "{err}"
    );
    assert!(stream.crash_residue().is_err());
    assert_eq!(fs::read(stream.entries_path()).unwrap(), before);
    assert_eq!(quarantine_files(tmp.path()), 0);
}

#[test]
fn tampered_complete_entry_is_never_repaired() {
    let tmp = tempdir().unwrap();
    let stream = seeded(tmp.path(), 4);
    // Parseable, wrong hash: a middle entry's payload changed in place.
    let content = fs::read_to_string(stream.entries_path()).unwrap();
    let tampered = content.replacen("\"n\":1}", "\"n\":99}", 1);
    assert_ne!(
        tampered, content,
        "fixture must contain the payload to tamper"
    );
    fs::write(stream.entries_path(), &tampered).unwrap();
    // The same tamper with a torn tail on top must still not be cut or rebuilt over.
    append_raw(&stream, b"{\"entry_ulid\":\"half");
    let before = fs::read(stream.entries_path()).unwrap();

    assert!(stream.verify().is_err());
    assert!(stream.crash_residue().is_err());
    assert!(stream.repair_tail().is_err());
    assert_eq!(fs::read(stream.entries_path()).unwrap(), before);
    assert_eq!(quarantine_files(tmp.path()), 0);
    assert!(
        stream.verify().is_err(),
        "tampered history still fails verify"
    );
}
