//! `verified_snapshot_locked`: one pinned, locked, chain-verified read.
use sea_forge_ledger::types::LedgerStream;
use std::fs;

fn stream_with_rows(root: &std::path::Path) -> LedgerStream {
    let stream = LedgerStream::open(root, "snap", "test").unwrap();
    for i in 0..3 {
        stream
            .commit_typed(
                "note",
                vec![format!("n{i}")],
                &serde_json::json!({"i": i}),
                vec![],
            )
            .unwrap();
    }
    stream
}

#[test]
fn a_clean_stream_is_returned_in_full() {
    let root = tempfile::tempdir().unwrap();
    let stream = stream_with_rows(root.path());
    assert_eq!(stream.verified_snapshot_locked().unwrap().len(), 3);
}

#[test]
fn an_edited_earlier_row_is_refused() {
    let root = tempfile::tempdir().unwrap();
    let stream = stream_with_rows(root.path());
    let path = stream.entries_path();
    let text = fs::read_to_string(&path).unwrap();
    fs::write(&path, text.replacen("\"i\":0", "\"i\":9", 1)).unwrap();
    assert!(stream.verified_snapshot_locked().is_err());
}

#[test]
fn a_missing_lock_file_is_an_error_not_an_unlocked_read() {
    let root = tempfile::tempdir().unwrap();
    let stream = stream_with_rows(root.path());
    fs::remove_file(root.path().join("ledgers/snap/stream.lock")).unwrap();
    assert!(
        stream.read_entries().is_ok(),
        "the ordinary reader still falls back"
    );
    assert!(stream.verified_snapshot_locked().is_err());
}

#[cfg(unix)]
#[test]
fn a_symlinked_entries_file_is_refused() {
    let root = tempfile::tempdir().unwrap();
    let stream = stream_with_rows(root.path());
    let path = stream.entries_path();
    let copy = root.path().join("elsewhere.jsonl");
    fs::copy(&path, &copy).unwrap();
    fs::remove_file(&path).unwrap();
    std::os::unix::fs::symlink(&copy, &path).unwrap();
    assert!(stream.verified_snapshot_locked().is_err());
}
