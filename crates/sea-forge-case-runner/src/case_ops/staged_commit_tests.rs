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

#[cfg(windows)]
#[test]
fn reader_opened_before_selector_replacement_reads_the_complete_old_generation() {
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    commit_staged(root, CASE, create_case(), None, &dir_sync).unwrap();
    commit_staged(root, CASE, run_file(b"old"), None, &dir_sync).unwrap();
    let old = versioned_store::open(root, CASE).unwrap().unwrap();
    let old_index = fs::read(index_path(&old)).unwrap();

    thread::scope(|scope| {
        let (opened_tx, opened_rx) = mpsc::channel();
        let (resume_tx, resume_rx) = mpsc::channel();
        let reader = scope.spawn(move || {
            versioned_store::open_after_selector_open(root, CASE, || {
                opened_tx.send(()).unwrap();
                resume_rx
                    .recv_timeout(Duration::from_secs(30))
                    .expect("writer did not release reader");
            })
        });
        opened_rx
            .recv_timeout(Duration::from_secs(30))
            .expect("reader did not open selector");
        let result = commit_staged(root, CASE, run_file(b"new"), None, &dir_sync);
        // Release even if publication returned an error; channel disconnect also
        // releases the reader if this thread panics before sending.
        let _ = resume_tx.send(());
        result.unwrap();
        let pinned = reader.join().unwrap().unwrap().unwrap();
        assert_eq!(pinned, old);
        assert!(load_verified(&pinned, false).unwrap().strict);
        assert_eq!(fs::read(index_path(&pinned)).unwrap(), old_index);
        assert_eq!(
            fs::read(pinned.join("runs/run-1/record.txt")).unwrap(),
            b"old"
        );
    });

    let current = versioned_store::open(root, CASE).unwrap().unwrap();
    assert_ne!(current, old);
    assert!(load_verified(&current, false).unwrap().strict);
    assert_eq!(
        fs::read(current.join("runs/run-1/record.txt")).unwrap(),
        b"new"
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
        let relative = path
            .strip_prefix(root)
            .unwrap()
            .iter()
            .map(|part| part.to_str().unwrap())
            .collect::<Vec<_>>()
            .join("/");
        files.insert(relative, hex::encode(Sha256::digest(&bytes)));
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

// Process interruption here exercises application restart, not power-loss durability.
#[test]
#[cfg_attr(
    not(any(unix, windows)),
    ignore = "requires a supported native publisher"
)]
fn interrupted_publication_can_be_retried() {
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    const CHILD_ROOT: &str = "SEA_FDE_RECOVERY_CHILD_ROOT";
    const CHILD_BOUNDARY: &str = "SEA_FDE_RECOVERY_CHILD_BOUNDARY";
    const SENTINEL: i32 = 86;
    const TEST_NAME: &str =
        "case_ops::case_guard::staged_commit_tests::interrupted_publication_can_be_retried";

    if let Some(root) = std::env::var_os(CHILD_ROOT) {
        let root = std::path::PathBuf::from(root);
        let boundary = std::env::var(CHILD_BOUNDARY).expect("child boundary");
        let storage = root.join(".case-storage");
        let case = storage.join(CASE);
        let (watched, occurrence, batch) = match boundary.as_str() {
            "before-first" => (storage, 1, create_case()),
            "after-first" => (storage, 2, create_case()),
            "before-replace" => (case.join("generations"), 2, run_file(b"child")),
            "after-replace" => (case, 2, run_file(b"child")),
            _ => panic!("unknown child boundary"),
        };
        let calls = Cell::new(0);
        let sync = |path: &Path| {
            if path == watched {
                let count = calls.get() + 1;
                calls.set(count);
                // The first matching sync seals the stage.
                // The second follows a rename in the publisher.
                if count == occurrence {
                    std::process::exit(SENTINEL);
                }
            }
            dir_sync(path)
        };
        commit_staged(&root, CASE, batch, None, &sync)
            .expect("child must reach the selected publication boundary");
        panic!("child passed the selected publication boundary");
    }

    let interrupt_at = |root: &Path, boundary: &str| {
        let mut child = Command::new(std::env::current_exe().unwrap())
            .arg("--exact")
            .arg(TEST_NAME)
            .env(CHILD_ROOT, root)
            .env(CHILD_BOUNDARY, boundary)
            .stdout(Stdio::null())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(60);
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(20));
                }
                result => {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("child did not finish at the boundary: {result:?}");
                }
            }
        };
        assert_eq!(
            status.code(),
            Some(SENTINEL),
            "child missed {boundary} boundary"
        );
    };

    for (boundary, published) in [("before-first", false), ("after-first", true)] {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path();
        interrupt_at(root, boundary);
        let selected = versioned_store::open(root, CASE).unwrap();
        assert_eq!(
            selected.is_some(),
            published,
            "unexpected snapshot at {boundary}"
        );
        if let Some(snapshot) = selected {
            assert!(load_verified(&snapshot, false).unwrap().strict);
        }

        let (batch, generation) = if published {
            (run_file(b"retry"), "2")
        } else {
            (create_case(), "1")
        };
        commit_staged(root, CASE, batch, None, &dir_sync).unwrap();
        let retry = versioned_store::open(root, CASE).unwrap().unwrap();
        let generations = root.join(".case-storage").join(CASE).join("generations");
        assert!(retry.starts_with(generations.join(generation)));
        assert!(load_verified(&retry, false).unwrap().strict);
    }

    for boundary in ["before-replace", "after-replace"] {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path();
        commit_staged(root, CASE, create_case(), None, &dir_sync).unwrap();
        commit_staged(root, CASE, run_file(b"old"), None, &dir_sync).unwrap();
        let old = versioned_store::open(root, CASE).unwrap().unwrap();
        assert!(load_verified(&old, false).unwrap().strict);
        let case = root.join(".case-storage").join(CASE);
        let generations = case.join("generations");

        interrupt_at(root, boundary);

        let selected = versioned_store::open(root, CASE).unwrap().unwrap();
        let installed = generations.join("3").join("data/cases").join(CASE);
        assert_eq!(fs::read(old.join("runs/run-1/record.txt")).unwrap(), b"old");
        assert_eq!(
            fs::read(installed.join("runs/run-1/record.txt")).unwrap(),
            b"child"
        );
        assert!(load_verified(&installed, false).unwrap().strict);
        if boundary == "before-replace" {
            assert_eq!(selected, old);
        } else {
            assert_eq!(selected, installed);
        }
        assert!(load_verified(&selected, false).unwrap().strict);

        commit_staged(root, CASE, run_file(b"retry"), None, &dir_sync).unwrap();
        let retry = versioned_store::open(root, CASE).unwrap().unwrap();
        assert!(retry.starts_with(generations.join("4")));
        assert!(load_verified(&retry, false).unwrap().strict);
        assert_eq!(
            fs::read(retry.join("runs/run-1/record.txt")).unwrap(),
            b"retry"
        );
        assert_eq!(fs::read(old.join("runs/run-1/record.txt")).unwrap(), b"old");
        assert_eq!(
            fs::read(installed.join("runs/run-1/record.txt")).unwrap(),
            b"child"
        );
    }
}

#[test]
#[cfg(any(unix, windows))]
fn staged_spool_verifies_bytes_before_publication() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    commit_staged(root, CASE, create_case(), None, &dir_sync).unwrap();
    commit_staged(root, CASE, run_file(b"previous"), None, &dir_sync).unwrap();
    let old = versioned_store::open(root, CASE).unwrap().unwrap();
    let bytes = vec![0xbc; 40_000];
    let changed = SealedRunFile::capture(root, &mut bytes.as_slice()).unwrap();
    {
        let mut file = changed.spool.file.lock().unwrap();
        file.rewind().unwrap();
        file.write_all(b"changed").unwrap();
    }
    let batch = Batch::new().with(Op::RunFileFromSpool {
        rel: "run-1/record.txt".into(),
        source: changed,
    });
    assert!(matches!(
        commit_staged(root, CASE, batch, None, &dir_sync),
        Err(CommitError::Refused(_))
    ));
    assert_eq!(
        versioned_store::open(root, CASE).unwrap(),
        Some(old.clone())
    );
    assert_eq!(
        fs::read(old.join("runs/run-1/record.txt")).unwrap(),
        b"previous"
    );

    let clean = SealedRunFile::capture(root, &mut bytes.as_slice()).unwrap();
    assert_eq!(clean.digest(), hex::encode(Sha256::digest(&bytes)));
    let batch = Batch::new()
        .with(Op::RunFileFromSpool {
            rel: "run-1/record.txt".into(),
            source: clean.clone(),
        })
        .with(Op::RunFileFromSpool {
            rel: "run-1/double.txt".into(),
            source: clean,
        });
    commit_staged(root, CASE, batch, None, &dir_sync).unwrap();
    let selected = versioned_store::open(root, CASE).unwrap().unwrap();
    assert_eq!(
        fs::read(selected.join("runs/run-1/record.txt")).unwrap(),
        bytes
    );
    assert_eq!(
        fs::read(selected.join("runs/run-1/double.txt")).unwrap(),
        bytes
    );
}

#[test]
fn capture_copy_propagates_storage_full_after_a_real_partial_write() {
    struct LimitedFile {
        file: File,
        remaining: usize,
    }

    impl Write for LimitedFile {
        fn write(&mut self, bytes: &[u8]) -> std_io::Result<usize> {
            if self.remaining == 0 {
                return Err(std_io::ErrorKind::StorageFull.into());
            }
            let written = self.file.write(&bytes[..bytes.len().min(self.remaining)])?;
            self.remaining -= written;
            Ok(written)
        }

        fn flush(&mut self) -> std_io::Result<()> {
            self.file.flush()
        }
    }

    let mut destination = LimitedFile {
        file: tempfile::tempfile().unwrap(),
        remaining: 7,
    };
    let error = copy_and_hash(&mut &b"matched then more bytes"[..], &mut destination).unwrap_err();
    assert_eq!(error.kind(), std_io::ErrorKind::StorageFull);
    let mut prefix = Vec::new();
    destination.file.rewind().unwrap();
    destination.file.read_to_end(&mut prefix).unwrap();
    assert_eq!(prefix, b"matched");
}

// The external controller kills the guest at the marker and boots it again on
// the same disk. Holding this thread alive prevents process-exit cleanup.
#[cfg(target_os = "linux")]
#[test]
#[ignore = "requires a disposable guest and an external reset controller"]
fn guest_crash_publication_probe() {
    use std::io::Write as _;

    let root = std::path::PathBuf::from(std::env::var_os("SEA_FDE_CRASH_ROOT").unwrap());
    let mode = std::env::var("SEA_FDE_CRASH_MODE").unwrap();
    let phase = std::env::var("SEA_FDE_CRASH_PHASE").unwrap();
    let storage = root.join(".case-storage");
    let case = storage.join(CASE);
    let reset_boundary = match phase.as_str() {
        "before-first" => Some((storage, 1)),
        "after-first" => Some((storage, 2)),
        "before-replace" => Some((case.join("generations"), 2)),
        "after-replace" => Some((case.clone(), 2)),
        "stage-created-first" | "acked-first" | "acked-replace" => None,
        _ => panic!("unknown reset phase: {phase}"),
    };
    let first = phase.ends_with("-first");

    if mode == "write" {
        assert!(versioned_store::open(&root, CASE).unwrap().is_none());
        if !first {
            commit_staged(&root, CASE, create_case(), None, &dir_sync).unwrap();
            commit_staged(&root, CASE, run_file(b"old"), None, &dir_sync).unwrap();
        }
        let batch = if first {
            create_case()
        } else {
            run_file(b"new")
        };
        let calls = Cell::new(0);
        let sync = |path: &Path| {
            if let Some((watched, occurrence)) = &reset_boundary {
                if path == watched {
                    calls.set(calls.get() + 1);
                    if calls.get() == *occurrence {
                        wait_for_guest_reset();
                    }
                }
            }
            dir_sync(path)
        };
        commit_staged_observed(&root, CASE, batch, None, &sync, |staged| {
            if phase == "stage-created-first" {
                assert!(staged.is_dir());
                assert!(staged.starts_with(root.join(".case-storage")));
                assert!(versioned_store::open(&root, CASE).unwrap().is_none());
                wait_for_guest_reset();
            }
        })
        .unwrap();
        assert!(
            reset_boundary.is_none() && phase != "stage-created-first",
            "missed reset boundary: {phase}"
        );
        wait_for_guest_reset();
    }
    assert_eq!(mode, "verify");

    let selected = versioned_store::open(&root, CASE).unwrap();
    let record = Path::new("runs/run-1/record.txt");
    if let Some(snapshot) = &selected {
        assert!(load_verified(snapshot, false).unwrap().strict);
        if first {
            assert!(!snapshot.join(record).exists());
            assert_eq!(*snapshot, case.join("generations/1/data/cases").join(CASE));
        } else {
            let version = fs::read(snapshot.join(record)).unwrap();
            let valid = match phase.as_str() {
                "before-replace" => version == b"old",
                "after-replace" => version == b"old" || version == b"new",
                "acked-replace" => version == b"new",
                _ => unreachable!(),
            };
            assert!(valid, "invalid recovered record at {phase}: {version:?}");
            let generation = if version == b"old" { "2" } else { "3" };
            assert_eq!(
                *snapshot,
                case.join("generations")
                    .join(generation)
                    .join("data/cases")
                    .join(CASE)
            );
        }
    }
    match phase.as_str() {
        "stage-created-first" | "before-first" => assert!(selected.is_none()),
        "after-first" => {}
        _ => assert!(selected.is_some()),
    }

    if selected.is_none() {
        commit_staged(&root, CASE, create_case(), None, &dir_sync).unwrap();
    }
    commit_staged(&root, CASE, run_file(b"retry"), None, &dir_sync).unwrap();
    let retried = versioned_store::open(&root, CASE).unwrap().unwrap();
    assert!(load_verified(&retried, false).unwrap().strict);
    assert_eq!(fs::read(retried.join(record)).unwrap(), b"retry");
    println!("\nSEA_FDE_RECOVERY_OK");
    std::io::stdout().flush().unwrap();
}

#[cfg(target_os = "linux")]
fn wait_for_guest_reset() -> ! {
    use std::io::Write as _;
    println!("\nSEA_FDE_RESET_READY");
    std::io::stdout().flush().unwrap();
    loop {
        std::thread::park();
    }
}

#[test]
#[cfg_attr(
    not(any(unix, windows)),
    ignore = "requires a supported native publisher"
)]
fn promotion_reads_staged_case_pin_after_repin() {
    use crate::promotion::{case_pin, init_store, pin_case, repin};

    let fixture = tempfile::tempdir().unwrap();
    let cell = fixture.path().join("cell");
    let store = fixture.path().join("promotion-store");
    fs::create_dir_all(&cell).unwrap();
    init_store(&store, "world-a").unwrap();
    commit_staged(
        &cell,
        CASE,
        create_case().with(Op::RunFile {
            rel: "pin.json".into(),
            bytes: br#"{"world_ref":"world-a"}"#.to_vec(),
        }),
        None,
        &dir_sync,
    )
    .unwrap();
    let case = cell.join("cases").join(CASE);
    fs::create_dir_all(case.join("runs")).unwrap();
    fs::write(
        case.join("runs/pin.json"),
        br#"{"world_ref":"world-legacy"}"#,
    )
    .unwrap();
    repin(&store, "world-b", "test").unwrap();
    assert_eq!(case_pin(&case).as_deref(), Some("world-a"));
    assert_eq!(pin_case(&store, &case).unwrap(), "world-a");
    let selected = versioned_store::open(&cell, CASE).unwrap().unwrap();
    assert_eq!(
        fs::read(selected.join("runs/pin.json")).unwrap(),
        br#"{"world_ref":"world-a"}"#
    );

    let namespace = versioned_store::namespace(&cell, CASE).unwrap().unwrap();
    fs::write(namespace.join("current.json"), b"invalid selector").unwrap();
    assert!(matches!(
        pin_case(&store, &case),
        Err(crate::promotion::SagaError::Uncertain(_))
    ));
    assert_eq!(
        fs::read(case.join("runs/pin.json")).unwrap(),
        br#"{"world_ref":"world-legacy"}"#
    );
    #[cfg(unix)]
    {
        let alias = fixture.path().join("aliased-cases");
        std::os::unix::fs::symlink(cell.join("cases"), &alias).unwrap();
        assert!(matches!(
            pin_case(&store, &alias.join(CASE)),
            Err(crate::promotion::SagaError::Uncertain(_))
        ));
    }
}

#[test]
fn promotion_refuses_malformed_existing_legacy_pin() {
    use crate::promotion::{init_store, pin_case, SagaError};

    let fixture = tempfile::tempdir().unwrap();
    let store = fixture.path().join("store");
    let case = fixture.path().join("cases").join(CASE);
    init_store(&store, "world-a").unwrap();
    fs::create_dir_all(case.join("runs")).unwrap();
    let pin = case.join("runs/pin.json");
    fs::write(&pin, b"not json").unwrap();
    assert!(matches!(
        pin_case(&store, &case),
        Err(SagaError::Uncertain(_))
    ));
    assert_eq!(fs::read(pin).unwrap(), b"not json");
}

#[test]
fn promotion_preserves_non_versioned_case_paths() {
    use crate::promotion::{case_pin, init_store, pin_case, repin};

    let fixture = tempfile::tempdir().unwrap();
    let store = fixture.path().join("promotion-store");
    let case = fixture.path().join("cases/customer.v1");
    init_store(&store, "world-a").unwrap();
    assert_eq!(case_pin(&case), None);
    assert_eq!(pin_case(&store, &case).unwrap(), "world-a");
    repin(&store, "world-b", "test").unwrap();
    assert_eq!(case_pin(&case).as_deref(), Some("world-a"));
    assert_eq!(pin_case(&store, &case).unwrap(), "world-a");
}
