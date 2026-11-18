//! Run real guard admission, application, and strict verification in a private
//! generation. Production callers continue to use the legacy writer.

use super::*;
use crate::case_ops::versioned_store;
use std::cell::Cell;
use std::collections::BTreeMap;
use std::fs;
use std::io as std_io;

const CASE: &str = "case-0001";

#[cfg(not(windows))]
fn dir_sync(path: &Path) -> std_io::Result<()> {
    File::open(path)?.sync_all()
}

fn create_case() -> Batch {
    Batch::new()
        .with(Op::Case(json!({
            "case_id": CASE,
            "state": "active",
        })))
        .with(Op::Plan(json!({
            "version": "0.2",
            "plan_id": "plan-1",
            "case_id": CASE,
            "run_id": "run-1",
            "intent_id": "intent-1",
            "items": [],
            "template_ref": "fde.field_case@1",
        })))
        .with(Op::Event(hypothetical(&TraceKind::CaseCreated, None)))
}

fn run_file(bytes: &[u8]) -> Batch {
    Batch::new().with(Op::RunFile {
        rel: "run-1/record.txt".into(),
        bytes: bytes.to_vec(),
    })
}

#[test]
#[cfg_attr(
    not(any(unix, windows)),
    ignore = "requires a supported native publisher"
)]
fn invalid_staged_state_keeps_the_case_namespace_absent() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    let invalid = create_case().with(Op::Case(json!({
        "case_id": "different-case",
        "state": "active",
    })));
    let error = commit_staged(root, CASE, invalid, None, &dir_sync).unwrap_err();
    assert!(matches!(
        error,
        CommitError::Refused(Refusal {
            code: "STORE_INTEGRITY",
            ..
        })
    ));
    assert!(versioned_store::open(root, CASE).unwrap().is_none());
    assert!(root
        .join(".case-storage")
        .read_dir()
        .unwrap()
        .next()
        .is_none());
}

#[test]
#[cfg_attr(
    not(any(unix, windows)),
    ignore = "requires a supported native publisher"
)]
fn successful_batches_select_complete_generations() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    commit_staged(root, CASE, create_case(), None, &dir_sync).unwrap();
    let first = versioned_store::open(root, CASE).unwrap().unwrap();
    assert!(load_verified(&first, false).unwrap().strict);
    assert!(!root.join("cases").join(CASE).exists());

    commit_staged(root, CASE, run_file(b"sealed"), None, &dir_sync).unwrap();
    let second = versioned_store::open(root, CASE).unwrap().unwrap();
    assert_ne!(first, second);
    assert!(!first.join("runs/run-1/record.txt").exists());
    assert_eq!(
        fs::read(second.join("runs/run-1/record.txt")).unwrap(),
        b"sealed"
    );
    assert!(load_verified(&second, false).unwrap().strict);
}

#[test]
#[cfg_attr(
    not(any(unix, windows)),
    ignore = "requires a supported native publisher"
)]
fn concurrent_readers_each_get_one_complete_batch() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Barrier;
    use std::thread;

    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    commit_staged(root, CASE, create_case(), None, &dir_sync).unwrap();
    let before = versioned_store::open(root, CASE).unwrap().unwrap();
    let start = Barrier::new(5);
    let finished = AtomicBool::new(false);

    thread::scope(|scope| {
        for _ in 0..4 {
            scope.spawn(|| {
                start.wait();
                loop {
                    let selected = versioned_store::open(root, CASE).unwrap().unwrap();
                    assert!(load_verified(&selected, false).unwrap().strict);
                    let record = selected.join("runs/run-1/record.txt");
                    if selected == before {
                        assert!(!record.exists());
                    } else {
                        assert_eq!(fs::read(record).unwrap(), b"complete");
                    }
                    if finished.load(Ordering::Acquire) {
                        break;
                    }
                }
            });
        }
        start.wait();
        let result = commit_staged(root, CASE, run_file(b"complete"), None, &dir_sync);
        finished.store(true, Ordering::Release);
        result.unwrap();
    });
    let after = versioned_store::open(root, CASE).unwrap().unwrap();
    assert_ne!(before, after);
    assert_eq!(
        fs::read(after.join("runs/run-1/record.txt")).unwrap(),
        b"complete"
    );
}

#[test]
#[cfg_attr(
    not(any(unix, windows)),
    ignore = "requires a supported native publisher"
)]
fn migrate_verified_legacy_files_into_a_complete_generation() {
    let fixture = tempfile::tempdir().unwrap();
    let root = &fixture.path().canonicalize().unwrap();
    let legacy = root.join("cases").join(CASE);
    commit(&legacy, create_case()).unwrap();
    let old_artifact = legacy.join("runs/run-old/record.txt");
    commit(
        &legacy,
        Batch::new().with(Op::RunFile {
            rel: "run-old/record.txt".into(),
            bytes: b"legacy-run".to_vec(),
        }),
    )
    .unwrap();
    let mut files = BTreeMap::new();
    let paths = fs::read_dir(&legacy)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_file())
        .chain([index_path(&legacy), old_artifact.clone()]);
    for path in paths {
        let bytes = fs::read(&path).unwrap();
        let relative = path.strip_prefix(root).unwrap().to_str().unwrap();
        files.insert(relative.to_owned(), hex::encode(Sha256::digest(&bytes)));
    }

    let incomplete = files
        .iter()
        .filter(|(path, _)| !path.contains("run-old"))
        .map(|(path, digest)| (path.clone(), digest.clone()))
        .collect();
    let error = commit_staged(
        root,
        CASE,
        run_file(b"incomplete"),
        Some(&incomplete),
        &dir_sync,
    )
    .unwrap_err();
    assert!(matches!(
        error,
        CommitError::Refused(Refusal {
            code: "STORE_INTEGRITY",
            ..
        })
    ));
    assert!(versioned_store::open(root, CASE).unwrap().is_none());
    assert_eq!(fs::read(&old_artifact).unwrap(), b"legacy-run");

    commit_staged(root, CASE, run_file(b"versioned"), Some(&files), &dir_sync).unwrap();
    let selected = versioned_store::open(root, CASE).unwrap().unwrap();
    assert!(load_verified(&selected, false).unwrap().strict);
    for name in ["case.json", "plan.json", EVENTS, RECORDS, DOCUMENT] {
        assert_eq!(
            fs::read(legacy.join(name)).unwrap(),
            fs::read(selected.join(name)).unwrap()
        );
    }
    assert_eq!(
        fs::read(selected.join("runs/run-1/record.txt")).unwrap(),
        b"versioned"
    );
    assert_eq!(
        fs::read(selected.join("runs/run-old/record.txt")).unwrap(),
        b"legacy-run"
    );
    assert!(!legacy.join("runs/run-1/record.txt").exists());
}

#[test]
#[cfg_attr(
    not(any(unix, windows)),
    ignore = "requires a supported native publisher"
)]
fn durability_errors_straddle_the_selector_publication_point() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    let case = root.join(".case-storage").join(CASE);
    commit_staged(root, CASE, create_case(), None, &dir_sync).unwrap();
    let selected = versioned_store::open(root, CASE).unwrap().unwrap();
    let generations = case.join("generations");
    let failed_path_calls = Cell::new(0);
    let fail_second_sync_of = |boundary: &Path, path: &Path| {
        if path == boundary {
            let count = failed_path_calls.get() + 1;
            failed_path_calls.set(count);
            if count == 2 {
                return Err(std_io::Error::other("simulated directory sync failure"));
            }
        }
        dir_sync(path)
    };

    let error = commit_staged(root, CASE, run_file(b"unselected"), None, &|path| {
        fail_second_sync_of(&generations, path)
    })
    .unwrap_err();
    assert!(matches!(
        error,
        CommitError::Refused(Refusal {
            code: "STORE_INTEGRITY",
            ..
        })
    ));
    assert_eq!(
        versioned_store::open(root, CASE).unwrap(),
        Some(selected.clone())
    );
    assert!(generations.join("2").is_dir());

    failed_path_calls.set(0);
    let error = commit_staged(root, CASE, run_file(b"visible"), None, &|path| {
        fail_second_sync_of(&case, path)
    })
    .unwrap_err();
    assert!(matches!(error, CommitError::PublicationUncertain(_)));
    let current = versioned_store::open(root, CASE).unwrap().unwrap();
    assert_eq!(
        fs::read(current.join("runs/run-1/record.txt")).unwrap(),
        b"visible"
    );
    assert_ne!(current, selected);
    assert!(current.starts_with(generations.join("3")));
}
